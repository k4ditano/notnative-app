//! Herramientas de análisis de notas para el agente RIG
//!
//! Incluye análisis estructural, conteo de palabras, generación de TOC, etc.

use crate::ai::tools::ToolError;
use crate::core::database::NotesDatabase;
use anyhow::Result;
use rig::tool::Tool;
use serde::Deserialize;
use std::path::PathBuf;

// ==================== WORD COUNT ====================

#[derive(Deserialize)]
pub struct GetWordCountArgs {
    pub name: String,
}

pub struct GetWordCount {
    pub db_path: PathBuf,
}

impl Tool for GetWordCount {
    const NAME: &'static str = "get_word_count";

    type Args = GetWordCountArgs;
    type Output = String;
    type Error = ToolError;

    async fn definition(&self, _prompt: String) -> rig::completion::ToolDefinition {
        rig::completion::ToolDefinition {
            name: "get_word_count".to_string(),
            description: "Get statistics about a note: word count, line count, character count"
                .to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "The name of the note"
                    }
                },
                "required": ["name"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let db_path = self.db_path.clone();

        let result = tokio::task::spawn_blocking(move || -> anyhow::Result<String> {
            let db = NotesDatabase::new(&db_path).map_err(|e| anyhow::anyhow!(e))?;
            let metadata = db.get_note(&args.name).map_err(|e| anyhow::anyhow!(e))?;

            if let Some(meta) = metadata {
                let content =
                    std::fs::read_to_string(&meta.path).map_err(|e| anyhow::anyhow!(e))?;

                let word_count = content.split_whitespace().count();
                let line_count = content.lines().count();
                let char_count = content.chars().count();

                Ok(format!(
                    "Statistics for '{}':\n- Words: {}\n- Lines: {}\n- Characters: {}",
                    args.name, word_count, line_count, char_count
                ))
            } else {
                Err(anyhow::anyhow!("Note '{}' not found", args.name))
            }
        })
        .await
        .map_err(|e| ToolError(e.to_string()))??;

        Ok(result)
    }
}

impl GetWordCount {
    pub fn new(db_path: PathBuf) -> Self {
        Self { db_path }
    }
}

// ==================== TABLE OF CONTENTS ====================

#[derive(Deserialize)]
pub struct GenerateTocArgs {
    pub name: String,
}

pub struct GenerateToc {
    pub db_path: PathBuf,
}

impl Tool for GenerateToc {
    const NAME: &'static str = "generate_table_of_contents";

    type Args = GenerateTocArgs;
    type Output = String;
    type Error = ToolError;

    async fn definition(&self, _prompt: String) -> rig::completion::ToolDefinition {
        rig::completion::ToolDefinition {
            name: "generate_table_of_contents".to_string(),
            description: "Generate a table of contents from the headings in a note".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "The name of the note"
                    }
                },
                "required": ["name"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let db_path = self.db_path.clone();

        let result = tokio::task::spawn_blocking(move || -> anyhow::Result<String> {
            let db = NotesDatabase::new(&db_path).map_err(|e| anyhow::anyhow!(e))?;
            let metadata = db.get_note(&args.name).map_err(|e| anyhow::anyhow!(e))?;

            if let Some(meta) = metadata {
                let content =
                    std::fs::read_to_string(&meta.path).map_err(|e| anyhow::anyhow!(e))?;

                let mut toc = String::from("Table of Contents:\n\n");
                let mut found_headings = false;

                for line in content.lines() {
                    if line.starts_with('#') {
                        found_headings = true;
                        let level = line.chars().take_while(|&c| c == '#').count();
                        let title = line.trim_start_matches('#').trim();
                        let indent = "  ".repeat(level.saturating_sub(1));
                        toc.push_str(&format!("{}- {}\n", indent, title));
                    }
                }

                if !found_headings {
                    toc.push_str("(No headings found in this note)");
                }

                Ok(toc)
            } else {
                Err(anyhow::anyhow!("Note '{}' not found", args.name))
            }
        })
        .await
        .map_err(|e| ToolError(e.to_string()))??;

        Ok(result)
    }
}

impl GenerateToc {
    pub fn new(db_path: PathBuf) -> Self {
        Self { db_path }
    }
}

// ==================== EXTRACT CODE BLOCKS ====================

#[derive(Deserialize)]
pub struct ExtractCodeBlocksArgs {
    pub name: String,
}

pub struct ExtractCodeBlocks {
    pub db_path: PathBuf,
}

impl Tool for ExtractCodeBlocks {
    const NAME: &'static str = "extract_code_blocks";

    type Args = ExtractCodeBlocksArgs;
    type Output = String;
    type Error = ToolError;

    async fn definition(&self, _prompt: String) -> rig::completion::ToolDefinition {
        rig::completion::ToolDefinition {
            name: "extract_code_blocks".to_string(),
            description: "Extract all code blocks from a note with their language identifiers"
                .to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "The name of the note"
                    }
                },
                "required": ["name"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let db_path = self.db_path.clone();

        let result = tokio::task::spawn_blocking(move || -> anyhow::Result<String> {
            let db = NotesDatabase::new(&db_path).map_err(|e| anyhow::anyhow!(e))?;
            let metadata = db.get_note(&args.name).map_err(|e| anyhow::anyhow!(e))?;

            if let Some(meta) = metadata {
                let content =
                    std::fs::read_to_string(&meta.path).map_err(|e| anyhow::anyhow!(e))?;

                let mut result = String::from("Code blocks found:\n\n");
                let mut in_code_block = false;
                let mut current_lang = String::new();
                let mut current_code = String::new();
                let mut block_count = 0;

                for line in content.lines() {
                    if line.starts_with("```") {
                        if in_code_block {
                            // End of code block
                            block_count += 1;
                            result
                                .push_str(&format!("Block #{} [{}]:\n", block_count, current_lang));
                            result.push_str(&current_code);
                            result.push_str("\n---\n\n");
                            current_code.clear();
                            current_lang.clear();
                            in_code_block = false;
                        } else {
                            // Start of code block
                            current_lang = line.trim_start_matches('`').trim().to_string();
                            if current_lang.is_empty() {
                                current_lang = "plain text".to_string();
                            }
                            in_code_block = true;
                        }
                    } else if in_code_block {
                        current_code.push_str(line);
                        current_code.push('\n');
                    }
                }

                if block_count == 0 {
                    result.push_str("(No code blocks found in this note)");
                }

                Ok(result)
            } else {
                Err(anyhow::anyhow!("Note '{}' not found", args.name))
            }
        })
        .await
        .map_err(|e| ToolError(e.to_string()))??;

        Ok(result)
    }
}

impl ExtractCodeBlocks {
    pub fn new(db_path: PathBuf) -> Self {
        Self { db_path }
    }
}

// ==================== ANALYZE NOTE STRUCTURE ====================

#[derive(Deserialize)]
pub struct AnalyzeNoteStructureArgs {
    pub name: String,
}

pub struct AnalyzeNoteStructure {
    pub db_path: PathBuf,
}

impl Tool for AnalyzeNoteStructure {
    const NAME: &'static str = "analyze_note_structure";

    type Args = AnalyzeNoteStructureArgs;
    type Output = String;
    type Error = ToolError;

    async fn definition(&self, _prompt: String) -> rig::completion::ToolDefinition {
        rig::completion::ToolDefinition {
            name: "analyze_note_structure".to_string(),
            description:
                "Analyze the structure of a note: headings, links, tags, lists, code blocks"
                    .to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "The name of the note"
                    }
                },
                "required": ["name"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let db_path = self.db_path.clone();

        let result = tokio::task::spawn_blocking(move || -> anyhow::Result<String> {
            let db = NotesDatabase::new(&db_path).map_err(|e| anyhow::anyhow!(e))?;
            let metadata = db.get_note(&args.name).map_err(|e| anyhow::anyhow!(e))?;

            if let Some(meta) = metadata {
                let content =
                    std::fs::read_to_string(&meta.path).map_err(|e| anyhow::anyhow!(e))?;

                let mut h1_count = 0;
                let mut h2_count = 0;
                let mut h3_count = 0;
                let mut link_count = 0;
                let mut tag_count = 0;
                let mut list_item_count = 0;
                let mut code_block_count = 0;
                let mut todo_count = 0;

                for line in content.lines() {
                    if line.starts_with("# ") {
                        h1_count += 1;
                    } else if line.starts_with("## ") {
                        h2_count += 1;
                    } else if line.starts_with("### ") {
                        h3_count += 1;
                    }

                    if line.starts_with("```") {
                        code_block_count += 1;
                    }

                    let trimmed = line.trim();
                    if trimmed.starts_with("- ")
                        || trimmed.starts_with("* ")
                        || trimmed.starts_with("+ ")
                    {
                        list_item_count += 1;
                    }

                    if trimmed.contains("- [ ]")
                        || trimmed.contains("- [x]")
                        || trimmed.contains("- [X]")
                    {
                        todo_count += 1;
                    }

                    link_count += line.matches("](").count();
                    tag_count += line.matches("#").count();
                }

                code_block_count /= 2; // Pairs of ```

                let analysis = format!(
                    "Structure Analysis for '{}':\n\n\
                    Headings:\n\
                    - H1: {}\n\
                    - H2: {}\n\
                    - H3: {}\n\n\
                    Content:\n\
                    - Markdown links: {}\n\
                    - Tags: {}\n\
                    - List items: {}\n\
                    - TODO items: {}\n\
                    - Code blocks: {}\n",
                    args.name,
                    h1_count,
                    h2_count,
                    h3_count,
                    link_count,
                    tag_count,
                    list_item_count,
                    todo_count,
                    code_block_count
                );

                Ok(analysis)
            } else {
                Err(anyhow::anyhow!("Note '{}' not found", args.name))
            }
        })
        .await
        .map_err(|e| ToolError(e.to_string()))??;

        Ok(result)
    }
}

impl AnalyzeNoteStructure {
    pub fn new(db_path: PathBuf) -> Self {
        Self { db_path }
    }
}

// ==================== FUZZY SEARCH ====================

#[derive(Deserialize)]
pub struct FuzzySearchArgs {
    pub query: String,
}

pub struct FuzzySearch {
    pub db_path: PathBuf,
}

impl Tool for FuzzySearch {
    const NAME: &'static str = "fuzzy_search";

    type Args = FuzzySearchArgs;
    type Output = String;
    type Error = ToolError;

    async fn definition(&self, _prompt: String) -> rig::completion::ToolDefinition {
        rig::completion::ToolDefinition {
            name: "fuzzy_search".to_string(),
            description:
                "Search notes by approximate name matching (handles typos, partial matches)"
                    .to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query (can be partial or contain typos)"
                    }
                },
                "required": ["query"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let db_path = self.db_path.clone();
        let query_clone = args.query.clone();

        let results = tokio::task::spawn_blocking(
            move || -> anyhow::Result<Vec<crate::core::database::NoteMetadata>> {
                let db = NotesDatabase::new(&db_path).map_err(|e| anyhow::anyhow!(e))?;
                let all_notes = db.list_notes(None).map_err(|e| anyhow::anyhow!(e))?;

                // Simple fuzzy match: case-insensitive contains
                let query_lower = query_clone.to_lowercase();
                let filtered: Vec<_> = all_notes
                    .into_iter()
                    .filter(|note| note.name.to_lowercase().contains(&query_lower))
                    .collect();
                Ok(filtered)
            },
        )
        .await
        .map_err(|e| ToolError(e.to_string()))??;

        if results.is_empty() {
            return Ok(format!("No notes found matching '{}'", args.query));
        }

        let mut output = format!("Fuzzy search results for '{}':\n", args.query);
        for note in results.iter().take(15) {
            output.push_str(&format!("- {}\n", note.name));
        }

        if results.len() > 15 {
            output.push_str(&format!("... and {} more results.", results.len() - 15));
        }

        Ok(output)
    }
}

impl FuzzySearch {
    pub fn new(db_path: PathBuf) -> Self {
        Self { db_path }
    }
}
