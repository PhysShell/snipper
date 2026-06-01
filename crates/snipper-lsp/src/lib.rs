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
    built_in_csharp_postfix_rules, built_in_csharp_prefix_rules, built_in_csharp_surround_rules,
    match_postfix, match_prefix, match_surround, SurroundContext,
};
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, CodeActionParams,
    CodeActionProviderCapability, CodeActionResponse, CompletionItem, CompletionItemKind,
    CompletionOptions, CompletionParams, CompletionResponse, CompletionTextEdit,
    DidChangeTextDocumentParams, DidOpenTextDocumentParams, InitializeParams, InitializeResult,
    InitializedParams, InsertTextFormat, MessageType, Position as LspPosition, Range as LspRange,
    ServerCapabilities, ServerInfo, TextDocumentSyncCapability, TextDocumentSyncKind,
    TextEdit as LspTextEdit, Url, WorkspaceEdit,
};
use tower_lsp::{Client, LanguageServer};

#[derive(Debug)]
struct DocumentState {
    text: String,
    language_id: String,
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
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for SnipperLsp {
    async fn initialize(&self, _params: InitializeParams) -> Result<InitializeResult> {
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
        self.docs.write().await.insert(
            params.text_document.uri,
            DocumentState {
                text: params.text_document.text,
                language_id: params.text_document.language_id,
            },
        );
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        if let Some(change) = params.content_changes.into_iter().last() {
            let mut docs = self.docs.write().await;
            if let Some(doc) = docs.get_mut(&params.text_document.uri) {
                doc.text = change.text;
            }
        }
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
        let items: Vec<CompletionItem> = expand_at(&text, &language_id, byte_offset)
            .into_iter()
            .map(to_completion_item)
            .collect();

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

        let backend = match language_id.as_str() {
            "csharp" | "cs" => TreeSitterBackend::csharp(),
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

        let actions = match_surround(&ctx, &built_in_csharp_surround_rules())
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

/// Expand postfix candidates at `byte_offset` in `source` for the given `language_id`.
///
/// Returns an empty [`Vec`] when the cursor is not at a `CodeAfterDot` site
/// (prime directive) or when the language is not supported.
fn expand_at(source: &str, language_id: &str, byte_offset: usize) -> Vec<snippercore::Candidate> {
    let backend = match language_id {
        "csharp" | "cs" => TreeSitterBackend::csharp(),
        _ => return vec![],
    };
    let Ok(classified) = backend.classify(source, byte_offset) else {
        return vec![];
    };
    match classified.lexical {
        LexicalClass::CodeAfterDot => {
            let Some(postfix) = classified.postfix else {
                return vec![];
            };
            match_postfix(&postfix, &built_in_csharp_postfix_rules())
        }
        LexicalClass::CodeBareIdentifier => {
            let Some(prefix) = classified.prefix else {
                return vec![];
            };
            match_prefix(&prefix, &built_in_csharp_prefix_rules())
        }
        _ => vec![],
    }
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
