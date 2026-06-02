//! LSP adapter for the Snipper expansion engine.
//!
//! This crate bridges `snipper-context` and `snipper-core` to the Language
//! Server Protocol. LSP-specific types live **here only** — they must not
//! appear in `snipper-core` or `snipper-context` public APIs (INV-5).

#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::sync::Arc;

use snippercontext::{Backend as _, LexicalClass, TreeSitterBackend};
use snippercore::{
    built_in_csharp_command_rules, built_in_csharp_postfix_rules, built_in_csharp_prefix_rules,
    built_in_csharp_surround_rules, built_in_typescript_postfix_rules,
    built_in_typescript_prefix_rules, built_in_typescript_surround_rules, find_command,
    match_postfix, match_prefix, match_surround, RuleKind, SurroundContext,
};
use tokio::io::AsyncBufReadExt as _;
use tokio::io::AsyncWriteExt as _;
use tokio::sync::{Mutex, RwLock};
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, CodeActionParams,
    CodeActionProviderCapability, CodeActionResponse, CompletionItem, CompletionItemKind,
    CompletionOptions, CompletionParams, CompletionResponse, CompletionTextEdit,
    DidChangeTextDocumentParams, DidOpenTextDocumentParams, ExecuteCommandOptions,
    ExecuteCommandParams, InitializeParams, InitializeResult, InitializedParams, InsertTextFormat,
    MessageType, Position as LspPosition, Range as LspRange, ServerCapabilities, ServerInfo,
    TextDocumentSyncCapability, TextDocumentSyncKind, TextEdit as LspTextEdit, Url, WorkspaceEdit,
};
use tower_lsp::{Client, LanguageServer};

#[derive(Debug)]
struct DocumentState {
    text: String,
    language_id: String,
}

/// Live handle to the Roslyn sidecar subprocess (S8).
///
/// Communicates via JSON-RPC 2.0 over stdin/stdout (one JSON object per line).
struct SidecarState {
    /// Held to keep the subprocess alive; dropped when the handle is destroyed.
    _child: tokio::process::Child,
    stdin: tokio::process::ChildStdin,
    reader: tokio::io::BufReader<tokio::process::ChildStdout>,
    next_id: u64,
}

/// LSP server for the Snipper postfix expansion engine.
///
/// Bridges `snipper-context` (CST classification) and `snipper-core`
/// (template matching) to the Language Server Protocol.
///
/// LSP-specific types are confined to this crate — they never appear in
/// `snipper-core` or `snipper-context` public APIs (INV-5).
pub struct SnipperLsp {
    client: Client,
    docs: Arc<RwLock<HashMap<Url, DocumentState>>>,
    /// Roslyn sidecar state; `None` when the sidecar is not running or has crashed.
    sidecar: Arc<Mutex<Option<SidecarState>>>,
}

impl std::fmt::Debug for SnipperLsp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SnipperLsp").finish_non_exhaustive()
    }
}

impl SnipperLsp {
    /// Creates a new [`SnipperLsp`] bound to the given LSP [`Client`].
    #[must_use]
    pub fn new(client: Client) -> Self {
        Self {
            client,
            docs: Arc::new(RwLock::new(HashMap::new())),
            sidecar: Arc::new(Mutex::new(None)),
        }
    }
}

/// Try to locate the Roslyn sidecar executable.
///
/// Checks the `SNIPPER_ROSLYN` environment variable first; returns `None` when
/// neither the env var is set nor a known default path exists.
fn find_sidecar_executable() -> Option<std::path::PathBuf> {
    if let Ok(path) = std::env::var("SNIPPER_ROSLYN") {
        let p = std::path::PathBuf::from(&path);
        if p.exists() {
            return Some(p);
        }
    }
    None
}

/// Spawn the Roslyn sidecar subprocess and return a connected [`SidecarState`].
///
/// Returns `None` when the executable cannot be found or the spawn fails.
fn try_spawn_sidecar() -> Option<SidecarState> {
    let exe = find_sidecar_executable()?;
    let mut child = tokio::process::Command::new(&exe)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .ok()?;
    let stdin = child.stdin.take()?;
    let stdout = child.stdout.take()?;
    Some(SidecarState {
        _child: child,
        stdin,
        reader: tokio::io::BufReader::new(stdout),
        next_id: 0,
    })
}

/// Query the Roslyn sidecar for the receiver type at `offset` in `source`.
///
/// Returns a type hierarchy (concrete type + all interfaces/bases) as a list of
/// fully-qualified type name strings, or `None` when the sidecar is unavailable,
/// crashed, or times out after 200 ms.
async fn query_receiver_type(
    sidecar: &Mutex<Option<SidecarState>>,
    source: &str,
    offset: usize,
) -> Option<Vec<String>> {
    let mut guard = sidecar.lock().await;

    if guard.is_none() {
        *guard = try_spawn_sidecar();
    }

    let state = guard.as_mut()?;
    let id = state.next_id;
    state.next_id += 1;

    let req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "receiverType",
        "params": { "source": source, "offset": offset }
    });

    if state
        .stdin
        .write_all(format!("{req}\n").as_bytes())
        .await
        .is_err()
    {
        *guard = None;
        return None;
    }

    let mut line = String::new();
    let read_result = tokio::time::timeout(
        std::time::Duration::from_millis(200),
        state.reader.read_line(&mut line),
    )
    .await;

    if let Ok(Ok(_)) = read_result {
        let resp: serde_json::Value = serde_json::from_str(&line).ok()?;
        let types = resp["result"]["types"]
            .as_array()?
            .iter()
            .filter_map(|v| v.as_str().map(str::to_owned))
            .collect();
        Some(types)
    } else {
        *guard = None;
        None
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for SnipperLsp {
    async fn initialize(&self, _params: InitializeParams) -> Result<InitializeResult> {
        let commands: Vec<String> = built_in_csharp_command_rules()
            .into_iter()
            .filter(|r| r.kind == RuleKind::Command)
            .map(|r| format!("snipper.{}", r.trigger))
            .collect();

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(true),
                    trigger_characters: Some(vec![".".to_owned()]),
                    ..Default::default()
                }),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands,
                    ..Default::default()
                }),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "snipper-lsp".to_owned(),
                version: Some(env!("CARGO_PKG_VERSION").to_owned()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "snipper-lsp initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let language_id = params.text_document.language_id.clone();
        self.docs.write().await.insert(
            params.text_document.uri,
            DocumentState {
                text: params.text_document.text,
                language_id,
            },
        );

        // Eagerly warm the Roslyn sidecar when the first C# document opens so
        // that the workspace is ready before the first completion request.
        if matches!(params.text_document.language_id.as_str(), "csharp" | "cs") {
            let sidecar = Arc::clone(&self.sidecar);
            tokio::spawn(async move {
                let mut guard = sidecar.lock().await;
                if guard.is_none() {
                    *guard = try_spawn_sidecar();
                }
            });
        }
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        if let Some(change) = params.content_changes.into_iter().last() {
            let mut docs = self.docs.write().await;
            if let Some(doc) = docs.get_mut(&params.text_document.uri) {
                doc.text = change.text;
            }
        }
    }

    async fn execute_command(
        &self,
        params: ExecuteCommandParams,
    ) -> Result<Option<serde_json::Value>> {
        let command_rules = built_in_csharp_command_rules();
        let Some(suffix) = params.command.strip_prefix("snipper.") else {
            return Ok(None);
        };
        let Some(rule) = find_command(suffix, &command_rules) else {
            return Ok(None);
        };
        // Return the snippet body as a string result so that the editor
        // extension can insert it via its native snippet API (e.g.
        // editor.action.insertSnippet in VS Code) and activate tabstops.
        // workspace/applyEdit would insert the text verbatim and lose the
        // ${1:...}/$0 tabstop markers.
        Ok(Some(serde_json::Value::String(rule.body.clone())))
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = &params.text_document_position.text_document.uri;
        let lsp_pos = params.text_document_position.position;

        let maybe_doc = {
            let docs = self.docs.read().await;
            docs.get(uri)
                .map(|d| (d.text.clone(), d.language_id.clone()))
        };

        let Some((text, language_id)) = maybe_doc else {
            return Ok(None);
        };

        let byte_offset = lsp_pos_to_byte(&text, lsp_pos);

        let Some((lexical, postfix_ctx, prefix_ctx, postfix_rules, prefix_rules)) =
            classify_for_expansion(&text, &language_id, byte_offset)
        else {
            return Ok(Some(CompletionResponse::Array(vec![])));
        };

        let candidates = match lexical {
            LexicalClass::CodeAfterDot => {
                let Some(mut postfix) = postfix_ctx else {
                    return Ok(Some(CompletionResponse::Array(vec![])));
                };
                // Enrich postfix context with Roslyn receiver-type data (C# only).
                if matches!(language_id.as_str(), "csharp" | "cs") {
                    postfix.receiver_type =
                        query_receiver_type(&self.sidecar, &text, byte_offset).await;
                }
                match_postfix(&postfix, &postfix_rules)
            }
            LexicalClass::CodeBareIdentifier => {
                let Some(prefix) = prefix_ctx else {
                    return Ok(Some(CompletionResponse::Array(vec![])));
                };
                match_prefix(&prefix, &prefix_rules)
            }
            _ => vec![],
        };

        let items: Vec<CompletionItem> = candidates.into_iter().map(to_completion_item).collect();
        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn completion_resolve(&self, item: CompletionItem) -> Result<CompletionItem> {
        Ok(item)
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let uri = params.text_document.uri;
        let selection = params.range;

        // Empty selection → no surround candidates.
        if selection.start == selection.end {
            return Ok(Some(vec![]));
        }

        let maybe_doc = {
            let docs = self.docs.read().await;
            docs.get(&uri)
                .map(|d| (d.text.clone(), d.language_id.clone()))
        };
        let Some((text, language_id)) = maybe_doc else {
            return Ok(Some(vec![]));
        };

        let (backend, surround_rules) = match language_id.as_str() {
            "csharp" | "cs" => (
                TreeSitterBackend::csharp(),
                built_in_csharp_surround_rules(),
            ),
            "typescript" | "ts" | "typescriptreact" | "tsx" => (
                TreeSitterBackend::typescript(),
                built_in_typescript_surround_rules(),
            ),
            _ => return Ok(Some(vec![])),
        };

        // Prime directive: classify one byte into the selection.
        let start_byte = lsp_pos_to_byte(&text, selection.start);
        let probe = start_byte.saturating_add(1).min(text.len());
        let Ok(classified) = backend.classify(&text, probe) else {
            return Ok(Some(vec![]));
        };
        if classified.lexical.forbids_expansion() {
            return Ok(Some(vec![]));
        }

        let Some(selected_text) = extract_selected_text(&text, selection) else {
            return Ok(Some(vec![]));
        };

        let ctx = SurroundContext {
            selected_text,
            range: core_range_from_lsp(selection),
        };

        let actions = match_surround(&ctx, &surround_rules)
            .into_iter()
            .map(|c| {
                let edit = WorkspaceEdit::new(HashMap::from([(
                    uri.clone(),
                    vec![LspTextEdit {
                        range: core_range_to_lsp(c.edit.range),
                        new_text: c.edit.new_text,
                    }],
                )]));
                CodeActionOrCommand::CodeAction(CodeAction {
                    title: c.label,
                    kind: Some(CodeActionKind::REFACTOR),
                    edit: Some(edit),
                    ..Default::default()
                })
            })
            .collect();

        Ok(Some(actions))
    }
}

type ClassifyResult = Option<(
    LexicalClass,
    Option<snippercore::PostfixContext>,
    Option<snippercore::PrefixContext>,
    Vec<snippercore::Rule>,
    Vec<snippercore::Rule>,
)>;

/// Classify the cursor site and return the rule packs for the language.
///
/// Returns `None` when the language is unsupported or the CST parse fails.
/// The [`LexicalClass`] drives which context and rule pack the caller should use.
fn classify_for_expansion(source: &str, language_id: &str, byte_offset: usize) -> ClassifyResult {
    let (backend, postfix_rules, prefix_rules) = match language_id {
        "csharp" | "cs" => (
            TreeSitterBackend::csharp(),
            built_in_csharp_postfix_rules(),
            built_in_csharp_prefix_rules(),
        ),
        "typescript" | "ts" | "typescriptreact" | "tsx" => (
            TreeSitterBackend::typescript(),
            built_in_typescript_postfix_rules(),
            built_in_typescript_prefix_rules(),
        ),
        _ => return None,
    };
    let Ok(classified) = backend.classify(source, byte_offset) else {
        return None;
    };
    Some((
        classified.lexical,
        classified.postfix,
        classified.prefix,
        postfix_rules,
        prefix_rules,
    ))
}

fn to_completion_item(candidate: snippercore::Candidate) -> CompletionItem {
    CompletionItem {
        filter_text: Some(candidate.trigger),
        label: candidate.label,
        kind: Some(CompletionItemKind::SNIPPET),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        text_edit: Some(CompletionTextEdit::Edit(LspTextEdit {
            range: core_range_to_lsp(candidate.edit.range),
            new_text: candidate.edit.new_text,
        })),
        ..Default::default()
    }
}

const fn core_range_to_lsp(range: snippercore::Range) -> LspRange {
    LspRange {
        start: core_pos_to_lsp(range.start),
        end: core_pos_to_lsp(range.end),
    }
}

const fn core_pos_to_lsp(pos: snippercore::Position) -> LspPosition {
    LspPosition {
        line: pos.line,
        character: pos.character,
    }
}

/// Extract the text covered by `range` from `source`.
///
/// Returns `None` when the byte range derived from `range` is not aligned to
/// UTF-8 char boundaries (malformed client input).
fn extract_selected_text(source: &str, range: LspRange) -> Option<String> {
    let start = lsp_pos_to_byte(source, range.start);
    let end = lsp_pos_to_byte(source, range.end);
    source.get(start..end).map(str::to_owned)
}

const fn core_range_from_lsp(range: LspRange) -> snippercore::Range {
    snippercore::Range {
        start: snippercore::Position {
            line: range.start.line,
            character: range.start.character,
        },
        end: snippercore::Position {
            line: range.end.line,
            character: range.end.character,
        },
    }
}

/// Convert an LSP cursor position (line, UTF-16 character) to a byte offset in `source`.
///
/// Returns `source.len()` if the position is past the end of the source text.
fn lsp_pos_to_byte(source: &str, pos: LspPosition) -> usize {
    #[allow(clippy::cast_possible_truncation)]
    let line = pos.line as usize;
    #[allow(clippy::cast_possible_truncation)]
    let character = pos.character as usize;

    let line_start = source
        .split('\n')
        .take(line)
        .fold(0_usize, |acc, l| acc + l.len() + 1);

    if line_start >= source.len() {
        return source.len();
    }

    let rest = &source[line_start..];
    let mut utf16 = 0_usize;
    for (byte_off, ch) in rest.char_indices() {
        if utf16 >= character {
            return line_start + byte_off;
        }
        utf16 += ch.len_utf16();
    }
    line_start + rest.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built_in_scaffold_constructor_body_contains_tabstops() {
        let rules = built_in_csharp_command_rules();
        let rule = find_command("scaffoldConstructor", &rules)
            .expect("scaffoldConstructor must exist");
        assert!(
            rule.body.contains("${1:"),
            "body must contain at least one tabstop placeholder"
        );
        assert!(rule.body.contains("$0"), "body must contain a final cursor tabstop");
    }

    #[test]
    fn unknown_command_returns_none() {
        let rules = built_in_csharp_command_rules();
        assert!(find_command("doesNotExist", &rules).is_none());
    }

    #[test]
    fn completion_item_uses_trigger_as_filter_text() {
        let candidate = snippercore::Candidate {
            trigger: "fod".to_owned(),
            label: ".FirstOrDefault()".to_owned(),
            edit: snippercore::TextEdit {
                range: snippercore::Range {
                    start: snippercore::Position {
                        line: 0,
                        character: 6,
                    },
                    end: snippercore::Position {
                        line: 0,
                        character: 9,
                    },
                },
                new_text: "users.FirstOrDefault()".to_owned(),
            },
        };

        let item = to_completion_item(candidate);

        assert_eq!(item.label, ".FirstOrDefault()");
        assert_eq!(item.filter_text.as_deref(), Some("fod"));
    }
}
