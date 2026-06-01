//! LSP adapter for the Snipper expansion engine.
//!
//! This crate bridges `snipper-context` and `snipper-core` to the Language
//! Server Protocol. LSP-specific types live **here only** — they must not
//! appear in `snipper-core` or `snipper-context` public APIs (INV-5).

#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::sync::Arc;

use snippercontext::{Backend as _, LexicalClass, TreeSitterBackend};
use snippercore::{built_in_csharp_postfix_rules, match_postfix};
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, CompletionOptions, CompletionParams, CompletionResponse,
    CompletionTextEdit, DidChangeTextDocumentParams, DidOpenTextDocumentParams, InitializeParams,
    InitializeResult, InitializedParams, InsertTextFormat, MessageType, Position as LspPosition,
    Range as LspRange, ServerCapabilities, ServerInfo, TextDocumentSyncCapability,
    TextDocumentSyncKind, TextEdit as LspTextEdit, Url,
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
    if classified.lexical != LexicalClass::CodeAfterDot {
        return vec![];
    }
    let Some(postfix) = classified.postfix else {
        return vec![];
    };
    let rules = built_in_csharp_postfix_rules();
    match_postfix(&postfix, &rules)
}

fn to_completion_item(candidate: snippercore::Candidate) -> CompletionItem {
    CompletionItem {
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
