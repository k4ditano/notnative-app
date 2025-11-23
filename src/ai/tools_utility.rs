//! Herramientas de utilidad general para el agente RIG

use crate::ai::tools::ToolError;
use crate::core::database::NotesDatabase;
use anyhow::Result;
use chrono::Local;
use rig::tool::Tool;
use serde::Deserialize;
use std::path::PathBuf;

// ==================== FIND AND REPLACE ====================

#[derive(Deserialize)]
pub struct FindAndReplaceArgs {
    pub name: String,
    pub find: String,
    pub replace: String,
}

pub struct FindAndReplace {
    pub db_path: PathBuf,
}

impl Tool for FindAndReplace {
    const NAME: &'static str = "find_and_replace";

    type Args = FindAndReplaceArgs;
    type Output = String;
    type Error = ToolError;

    async fn definition(&self, _prompt: String) -> rig::completion::ToolDefinition {
        rig::completion::ToolDefinition {
            name: "find_and_replace".to_string(),
            description: "Find and replace text in a note. Replaces all occurrences.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "The name of the note"
                    },
                    "find": {
                        "type": "string",
                        "description": "The text to find"
                    },
                    "replace": {
                        "type": "string",
                        "description": "The replacement text"
                    }
                },
                "required": ["name", "find", "replace"]
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

                let occurrences = content.matches(&args.find).count();

                if occurrences == 0 {
                    return Ok(format!(
                        "No occurrences of '{}' found in note '{}'",
                        args.find, args.name
                    ));
                }

                let new_content = content.replace(&args.find, &args.replace);

                std::fs::write(&meta.path, &new_content).map_err(|e| anyhow::anyhow!(e))?;

                // Update database
                db.index_note(&args.name, &meta.path, &new_content, meta.folder.as_deref())
                    .map_err(|e| anyhow::anyhow!(e))?;

                Ok(format!(
                    "Replaced {} occurrence(s) of '{}' with '{}' in note '{}'",
                    occurrences, args.find, args.replace, args.name
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

impl FindAndReplace {
    pub fn new(db_path: PathBuf) -> Self {
        Self { db_path }
    }
}

// ==================== CREATE DAILY NOTE ====================

#[derive(Deserialize)]
pub struct CreateDailyNoteArgs {
    #[serde(default)]
    pub folder: Option<String>,
}

pub struct CreateDailyNote {
    pub db_path: PathBuf,
    pub notes_dir: PathBuf,
}

impl Tool for CreateDailyNote {
    const NAME: &'static str = "create_daily_note";

    type Args = CreateDailyNoteArgs;
    type Output = String;
    type Error = ToolError;

    async fn definition(&self, _prompt: String) -> rig::completion::ToolDefinition {
        rig::completion::ToolDefinition {
            name: "create_daily_note".to_string(),
            description: "Create a daily note with today's date (YYYY-MM-DD format).".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "folder": {
                        "type": "string",
                        "description": "Optional folder path (e.g., 'Daily Notes')"
                    }
                }
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let db_path = self.db_path.clone();
        let notes_dir = self.notes_dir.clone();

        let result = tokio::task::spawn_blocking(move || -> anyhow::Result<String> {
            let db = NotesDatabase::new(&db_path).map_err(|e| anyhow::anyhow!(e))?;

            let today = Local::now().format("%Y-%m-%d").to_string();

            // Check if daily note already exists
            if db.get_note(&today).map_err(|e| anyhow::anyhow!(e))?.is_some() {
                return Ok(format!("Daily note for {} already exists", today));
            }

            let target_dir = if let Some(folder) = &args.folder {
                let folder_path = notes_dir.join(folder);
                std::fs::create_dir_all(&folder_path)
                    .map_err(|e| anyhow::anyhow!("Failed to create folder: {}", e))?;
                folder_path
            } else {
                notes_dir
            };

            let file_path = target_dir.join(format!("{}.md", today));

            // Create daily note template
            let content = format!(
                "---\ntags: [daily]\ndate: {}\n---\n\n# {}\n\n## Tasks\n- [ ] \n\n## Notes\n\n## Journal\n\n",
                today, today
            );

            std::fs::write(&file_path, &content)
                .map_err(|e| anyhow::anyhow!("Failed to create daily note: {}", e))?;

            // Index note
            db.index_note(&today, file_path.to_str().unwrap(), &content, args.folder.as_deref())
                .map_err(|e| anyhow::anyhow!(e))?;

            Ok(format!("Daily note '{}' created successfully", today))
        }).await.map_err(|e| ToolError(e.to_string()))??;

        Ok(result)
    }
}

impl CreateDailyNote {
    pub fn new(db_path: PathBuf, notes_dir: PathBuf) -> Self {
        Self { db_path, notes_dir }
    }
}

// ==================== GET SYSTEM DATE TIME ====================

#[derive(Deserialize)]
pub struct GetSystemDateTimeArgs {}

pub struct GetSystemDateTime;

impl Tool for GetSystemDateTime {
    const NAME: &'static str = "get_system_date_time";

    type Args = GetSystemDateTimeArgs;
    type Output = String;
    type Error = ToolError;

    async fn definition(&self, _prompt: String) -> rig::completion::ToolDefinition {
        rig::completion::ToolDefinition {
            name: "get_system_date_time".to_string(),
            description: "Get current system date and time in various formats.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        }
    }

    async fn call(&self, _args: Self::Args) -> Result<Self::Output, Self::Error> {
        let now = Local::now();

        Ok(format!(
            "Current Date & Time:\n\
            - ISO 8601: {}\n\
            - Date: {}\n\
            - Time: {}\n\
            - Unix timestamp: {}\n\
            - Day of week: {}\n\
            - Week number: {}",
            now.to_rfc3339(),
            now.format("%Y-%m-%d"),
            now.format("%H:%M:%S"),
            now.timestamp(),
            now.format("%A"),
            now.format("%W")
        ))
    }
}

impl GetSystemDateTime {
    pub fn new() -> Self {
        Self
    }
}

// ==================== GET APP INFO ====================

#[derive(Deserialize)]
pub struct GetAppInfoArgs {}

pub struct GetAppInfo {
    pub notes_dir: PathBuf,
}

impl Tool for GetAppInfo {
    const NAME: &'static str = "get_app_info";

    type Args = GetAppInfoArgs;
    type Output = String;
    type Error = ToolError;

    async fn definition(&self, _prompt: String) -> rig::completion::ToolDefinition {
        rig::completion::ToolDefinition {
            name: "get_app_info".to_string(),
            description: "Get information about the NotNative application.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        }
    }

    async fn call(&self, _args: Self::Args) -> Result<Self::Output, Self::Error> {
        let notes_path = self.notes_dir.to_str().unwrap_or("unknown");

        Ok(format!(
            "NotNative App Information:\n\
            - Version: {} (RIG Agent Edition)\n\
            - Notes directory: {}\n\
            - Features: Markdown notes, AI chat, MCP tools, RIG native tools\n\
            - AI Framework: RIG (Rust Intelligence Gateway)\n\
            - Database: SQLite with FTS5 and Vector embeddings",
            env!("CARGO_PKG_VERSION"),
            notes_path
        ))
    }
}

impl GetAppInfo {
    pub fn new(notes_dir: PathBuf) -> Self {
        Self { notes_dir }
    }
}

// ==================== GET WORKSPACE PATH ====================

#[derive(Deserialize)]
pub struct GetWorkspacePathArgs {}

pub struct GetWorkspacePath {
    pub notes_dir: PathBuf,
}

impl Tool for GetWorkspacePath {
    const NAME: &'static str = "get_workspace_path";

    type Args = GetWorkspacePathArgs;
    type Output = String;
    type Error = ToolError;

    async fn definition(&self, _prompt: String) -> rig::completion::ToolDefinition {
        rig::completion::ToolDefinition {
            name: "get_workspace_path".to_string(),
            description: "Get the absolute path to the notes workspace directory.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        }
    }

    async fn call(&self, _args: Self::Args) -> Result<Self::Output, Self::Error> {
        let path = self.notes_dir.to_str().unwrap_or("unknown");
        Ok(format!("Workspace path: {}", path))
    }
}

impl GetWorkspacePath {
    pub fn new(notes_dir: PathBuf) -> Self {
        Self { notes_dir }
    }
}
