//! Herramientas de gestión de tags para el agente RIG

use crate::ai::tools::ToolError;
use crate::core::database::NotesDatabase;
use crate::core::frontmatter::{extract_all_tags, update_tags};
use anyhow::Result;
use rig::tool::Tool;
use serde::Deserialize;
use std::path::PathBuf;

// ==================== ADD TAG ====================

#[derive(Deserialize)]
pub struct AddTagArgs {
    pub name: String,
    pub tag: String,
}

pub struct AddTag {
    pub db_path: PathBuf,
}

impl Tool for AddTag {
    const NAME: &'static str = "add_tag";

    type Args = AddTagArgs;
    type Output = String;
    type Error = ToolError;

    async fn definition(&self, _prompt: String) -> rig::completion::ToolDefinition {
        rig::completion::ToolDefinition {
            name: "add_tag".to_string(),
            description: "Add a tag to a note. Tags are added to the frontmatter YAML.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "The name of the note"
                    },
                    "tag": {
                        "type": "string",
                        "description": "The tag to add (without # symbol)"
                    }
                },
                "required": ["name", "tag"]
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

                let mut tags = extract_all_tags(&content);

                // Check if tag already exists
                let clean_tag = args.tag.trim_start_matches('#').trim();
                if tags.contains(&clean_tag.to_string()) {
                    return Ok(format!(
                        "Tag '{}' already exists in note '{}'",
                        clean_tag, args.name
                    ));
                }

                tags.push(clean_tag.to_string());

                // Update frontmatter
                let new_content = update_tags(&content, tags).map_err(|e| anyhow::anyhow!(e))?;

                std::fs::write(&meta.path, &new_content).map_err(|e| anyhow::anyhow!(e))?;

                // Update database
                db.index_note(&args.name, &meta.path, &new_content, meta.folder.as_deref())
                    .map_err(|e| anyhow::anyhow!(e))?;

                Ok(format!("Tag '{}' added to note '{}'", clean_tag, args.name))
            } else {
                Err(anyhow::anyhow!("Note '{}' not found", args.name))
            }
        })
        .await
        .map_err(|e| ToolError(e.to_string()))??;

        Ok(result)
    }
}

impl AddTag {
    pub fn new(db_path: PathBuf) -> Self {
        Self { db_path }
    }
}

// ==================== REMOVE TAG ====================

#[derive(Deserialize)]
pub struct RemoveTagArgs {
    pub name: String,
    pub tag: String,
}

pub struct RemoveTag {
    pub db_path: PathBuf,
}

impl Tool for RemoveTag {
    const NAME: &'static str = "remove_tag";

    type Args = RemoveTagArgs;
    type Output = String;
    type Error = ToolError;

    async fn definition(&self, _prompt: String) -> rig::completion::ToolDefinition {
        rig::completion::ToolDefinition {
            name: "remove_tag".to_string(),
            description: "Remove a tag from a note.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "The name of the note"
                    },
                    "tag": {
                        "type": "string",
                        "description": "The tag to remove (without # symbol)"
                    }
                },
                "required": ["name", "tag"]
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

                let mut tags = extract_all_tags(&content);

                let clean_tag = args.tag.trim_start_matches('#').trim();

                // Remove the tag
                let original_len = tags.len();
                tags.retain(|t| t != clean_tag);

                if tags.len() == original_len {
                    return Ok(format!(
                        "Tag '{}' not found in note '{}'",
                        clean_tag, args.name
                    ));
                }

                // Update frontmatter
                let new_content = update_tags(&content, tags).map_err(|e| anyhow::anyhow!(e))?;

                std::fs::write(&meta.path, &new_content).map_err(|e| anyhow::anyhow!(e))?;

                // Update database
                db.index_note(&args.name, &meta.path, &new_content, meta.folder.as_deref())
                    .map_err(|e| anyhow::anyhow!(e))?;

                Ok(format!(
                    "Tag '{}' removed from note '{}'",
                    clean_tag, args.name
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

impl RemoveTag {
    pub fn new(db_path: PathBuf) -> Self {
        Self { db_path }
    }
}

// ==================== DUPLICATE NOTE ====================

#[derive(Deserialize)]
pub struct DuplicateNoteArgs {
    pub name: String,
    pub new_name: String,
}

pub struct DuplicateNote {
    pub db_path: PathBuf,
}

impl Tool for DuplicateNote {
    const NAME: &'static str = "duplicate_note";

    type Args = DuplicateNoteArgs;
    type Output = String;
    type Error = ToolError;

    async fn definition(&self, _prompt: String) -> rig::completion::ToolDefinition {
        rig::completion::ToolDefinition {
            name: "duplicate_note".to_string(),
            description: "Create a copy of an existing note with a new name.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "The name of the note to duplicate"
                    },
                    "new_name": {
                        "type": "string",
                        "description": "The name for the duplicated note"
                    }
                },
                "required": ["name", "new_name"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let db_path = self.db_path.clone();

        let result = tokio::task::spawn_blocking(move || -> anyhow::Result<String> {
            let db = NotesDatabase::new(&db_path).map_err(|e| anyhow::anyhow!(e))?;
            let metadata = db.get_note(&args.name).map_err(|e| anyhow::anyhow!(e))?;

            if let Some(meta) = metadata {
                // Check if new name already exists
                if db
                    .get_note(&args.new_name)
                    .map_err(|e| anyhow::anyhow!(e))?
                    .is_some()
                {
                    return Err(anyhow::anyhow!(
                        "A note named '{}' already exists",
                        args.new_name
                    ));
                }

                let content =
                    std::fs::read_to_string(&meta.path).map_err(|e| anyhow::anyhow!(e))?;

                // Create new file in same folder
                let note_path = PathBuf::from(&meta.path);
                let parent_dir = note_path
                    .parent()
                    .ok_or_else(|| anyhow::anyhow!("Invalid path"))?;
                let new_path = parent_dir.join(format!("{}.md", args.new_name));

                std::fs::write(&new_path, &content)
                    .map_err(|e| anyhow::anyhow!("Failed to create duplicate: {}", e))?;

                // Index new note
                db.index_note(
                    &args.new_name,
                    new_path.to_str().unwrap(),
                    &content,
                    meta.folder.as_deref(),
                )
                .map_err(|e| anyhow::anyhow!(e))?;

                Ok(format!(
                    "Note '{}' duplicated as '{}'",
                    args.name, args.new_name
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

impl DuplicateNote {
    pub fn new(db_path: PathBuf) -> Self {
        Self { db_path }
    }
}

// ==================== MERGE NOTES ====================

#[derive(Deserialize)]
pub struct MergeNotesArgs {
    pub names: Vec<String>,
    pub target_name: String,
}

pub struct MergeNotes {
    pub db_path: PathBuf,
}

impl Tool for MergeNotes {
    const NAME: &'static str = "merge_notes";

    type Args = MergeNotesArgs;
    type Output = String;
    type Error = ToolError;

    async fn definition(&self, _prompt: String) -> rig::completion::ToolDefinition {
        rig::completion::ToolDefinition {
            name: "merge_notes".to_string(),
            description: "Merge multiple notes into a single note. Source notes are preserved."
                .to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "names": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "List of note names to merge"
                    },
                    "target_name": {
                        "type": "string",
                        "description": "Name for the merged note"
                    }
                },
                "required": ["names", "target_name"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let db_path = self.db_path.clone();

        let result = tokio::task::spawn_blocking(move || -> anyhow::Result<String> {
            let db = NotesDatabase::new(&db_path).map_err(|e| anyhow::anyhow!(e))?;

            if args.names.is_empty() {
                return Err(anyhow::anyhow!("No notes specified to merge"));
            }

            // Check if target already exists
            if db
                .get_note(&args.target_name)
                .map_err(|e| anyhow::anyhow!(e))?
                .is_some()
            {
                return Err(anyhow::anyhow!(
                    "A note named '{}' already exists",
                    args.target_name
                ));
            }

            let mut merged_content = format!("# {}\n\n", args.target_name);
            merged_content.push_str("*Merged from: ");
            merged_content.push_str(&args.names.join(", "));
            merged_content.push_str("*\n\n---\n\n");

            let mut notes_path = None;

            for note_name in &args.names {
                let metadata = db.get_note(note_name).map_err(|e| anyhow::anyhow!(e))?;

                if let Some(meta) = metadata {
                    if notes_path.is_none() {
                        let path_buf = PathBuf::from(&meta.path);
                        notes_path = path_buf.parent().map(|p| p.to_path_buf());
                    }

                    let content = std::fs::read_to_string(&meta.path)
                        .map_err(|e| anyhow::anyhow!("Failed to read '{}': {}", note_name, e))?;

                    merged_content.push_str(&format!("## From: {}\n\n", note_name));
                    merged_content.push_str(&content);
                    merged_content.push_str("\n\n---\n\n");
                } else {
                    eprintln!("Warning: Note '{}' not found, skipping", note_name);
                }
            }

            // Create merged note
            let target_path = notes_path
                .ok_or_else(|| anyhow::anyhow!("Could not determine target path"))?
                .join(format!("{}.md", args.target_name));

            std::fs::write(&target_path, &merged_content)
                .map_err(|e| anyhow::anyhow!("Failed to create merged note: {}", e))?;

            // Index merged note
            db.index_note(
                &args.target_name,
                target_path.to_str().unwrap(),
                &merged_content,
                None,
            )
            .map_err(|e| anyhow::anyhow!(e))?;

            Ok(format!(
                "Merged {} notes into '{}'",
                args.names.len(),
                args.target_name
            ))
        })
        .await
        .map_err(|e| ToolError(e.to_string()))??;

        Ok(result)
    }
}

impl MergeNotes {
    pub fn new(db_path: PathBuf) -> Self {
        Self { db_path }
    }
}
