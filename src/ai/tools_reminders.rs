use crate::ai::tools::ToolError;
use crate::core::database::NotesDatabase;
use anyhow::Result;
use chrono::{Duration, Local};
use rig::tool::Tool;
use serde::Deserialize;
use std::path::PathBuf;

fn resolve_date(date_str: &str) -> String {
    let lower = date_str.to_lowercase();
    let now = Local::now();

    if lower.starts_with("tomorrow") || lower.starts_with("mañana") || lower.starts_with("manana")
    {
        let tomorrow = now + Duration::days(1);
        // Check if time is provided
        let parts: Vec<&str> = date_str.split_whitespace().collect();
        if parts.len() > 1 {
            // Assume second part is time
            return format!("{} {}", tomorrow.format("%Y-%m-%d"), parts[1]);
        } else {
            return format!("{} 09:00", tomorrow.format("%Y-%m-%d"));
        }
    } else if lower.starts_with("today") || lower.starts_with("hoy") {
        // Check if time is provided
        let parts: Vec<&str> = date_str.split_whitespace().collect();
        if parts.len() > 1 {
            return format!("{} {}", now.format("%Y-%m-%d"), parts[1]);
        } else {
            return format!("{} 09:00", now.format("%Y-%m-%d"));
        }
    }

    // Return as is if not relative (or if we don't handle it)
    date_str.to_string()
}

#[derive(Deserialize)]
pub struct CreateReminderArgs {
    pub note_name: String,
    pub text: String,
    pub date: String,
    pub priority: Option<String>,
    pub repeat: Option<String>,
}

pub struct CreateReminder {
    pub db_path: PathBuf,
}

impl Tool for CreateReminder {
    const NAME: &'static str = "create_reminder";

    type Args = CreateReminderArgs;
    type Output = String;
    type Error = ToolError;

    async fn definition(&self, _prompt: String) -> rig::completion::ToolDefinition {
        rig::completion::ToolDefinition {
            name: "create_reminder".to_string(),
            description: "Create a reminder in a specific note. The reminder will be appended to the note content.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "note_name": {
                        "type": "string",
                        "description": "The name of the note where the reminder will be added"
                    },
                    "text": {
                        "type": "string",
                        "description": "The reminder text/description"
                    },
                    "date": {
                        "type": "string",
                        "description": "The due date/time (e.g., 'tomorrow 09:00', '2025-11-20 15:00', 'hoy 18:00')"
                    },
                    "priority": {
                        "type": "string",
                        "description": "Optional priority: 'low', 'medium', 'high', 'urgent' (default: medium)"
                    },
                    "repeat": {
                        "type": "string",
                        "description": "Optional repeat pattern: 'daily', 'weekly', 'monthly'"
                    }
                },
                "required": ["note_name", "text", "date"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let db_path = self.db_path.clone();

        let result = tokio::task::spawn_blocking(move || -> anyhow::Result<String> {
            let db = NotesDatabase::new(&db_path).map_err(|e| anyhow::anyhow!(e))?;
            let metadata = db
                .get_note(&args.note_name)
                .map_err(|e| anyhow::anyhow!(e))?;

            if let Some(meta) = metadata {
                // Construct the reminder string
                // Format: !!RECORDAR(date [priority] [repeat=pattern]) text
                let resolved_date = resolve_date(&args.date);
                let mut params = resolved_date;

                if let Some(p) = args.priority {
                    if !p.is_empty() {
                        params.push_str(&format!(" {}", p));
                    }
                }

                if let Some(r) = args.repeat {
                    if !r.is_empty() {
                        params.push_str(&format!(" repeat={}", r));
                    }
                }

                // Format: !!RECORDAR(date [priority] [repeat=pattern], text)
                let reminder_line = format!("\n!!RECORDAR({}, {})\n", params, args.text);

                // Append to note
                let mut current_content =
                    std::fs::read_to_string(&meta.path).map_err(|e| anyhow::anyhow!(e))?;

                current_content.push_str(&reminder_line);

                std::fs::write(&meta.path, &current_content).map_err(|e| anyhow::anyhow!(e))?;

                // Update in DB
                db.index_note(
                    &args.note_name,
                    &meta.path,
                    &current_content,
                    meta.folder.as_deref(),
                )
                .map_err(|e| anyhow::anyhow!(e))?;

                Ok(format!(
                    "Reminder created in note '{}': {}",
                    args.note_name,
                    reminder_line.trim()
                ))
            } else {
                Err(anyhow::anyhow!("Note '{}' not found", args.note_name))
            }
        })
        .await
        .map_err(|e| ToolError(e.to_string()))??;

        Ok(result)
    }
}

impl CreateReminder {
    pub fn new(db_path: PathBuf) -> Self {
        Self { db_path }
    }
}

// ==================== DELETE REMINDER ====================

#[derive(Deserialize)]
pub struct DeleteReminderArgs {
    pub note_name: String,
    pub text_match: String,
}

pub struct DeleteReminder {
    pub db_path: PathBuf,
}

impl Tool for DeleteReminder {
    const NAME: &'static str = "delete_reminder";

    type Args = DeleteReminderArgs;
    type Output = String;
    type Error = ToolError;

    async fn definition(&self, _prompt: String) -> rig::completion::ToolDefinition {
        rig::completion::ToolDefinition {
            name: "delete_reminder".to_string(),
            description: "Delete a reminder from a note by matching its text content".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "note_name": {
                        "type": "string",
                        "description": "The name of the note containing the reminder"
                    },
                    "text_match": {
                        "type": "string",
                        "description": "Text to identify the reminder (e.g. the reminder description)"
                    }
                },
                "required": ["note_name", "text_match"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let db_path = self.db_path.clone();

        let result = tokio::task::spawn_blocking(move || -> anyhow::Result<String> {
            let db = NotesDatabase::new(&db_path).map_err(|e| anyhow::anyhow!(e))?;
            let metadata = db
                .get_note(&args.note_name)
                .map_err(|e| anyhow::anyhow!(e))?;

            if let Some(meta) = metadata {
                let current_content =
                    std::fs::read_to_string(&meta.path).map_err(|e| anyhow::anyhow!(e))?;

                let mut new_lines = Vec::new();
                let mut found = false;

                for line in current_content.lines() {
                    // Check if line is a reminder and contains the match text
                    if line.contains("!!RECORDAR") && line.contains(&args.text_match) {
                        found = true;
                        continue; // Skip this line (delete it)
                    }
                    new_lines.push(line);
                }

                if !found {
                    return Ok(format!(
                        "No reminder found matching '{}' in note '{}'",
                        args.text_match, args.note_name
                    ));
                }

                let new_content = new_lines.join("\n");
                std::fs::write(&meta.path, &new_content).map_err(|e| anyhow::anyhow!(e))?;

                // Update in DB
                db.index_note(
                    &args.note_name,
                    &meta.path,
                    &new_content,
                    meta.folder.as_deref(),
                )
                .map_err(|e| anyhow::anyhow!(e))?;

                Ok(format!(
                    "Reminder matching '{}' deleted from note '{}'",
                    args.text_match, args.note_name
                ))
            } else {
                Err(anyhow::anyhow!("Note '{}' not found", args.note_name))
            }
        })
        .await
        .map_err(|e| ToolError(e.to_string()))??;

        Ok(result)
    }
}

impl DeleteReminder {
    pub fn new(db_path: PathBuf) -> Self {
        Self { db_path }
    }
}

// ==================== MODIFY REMINDER ====================

#[derive(Deserialize)]
pub struct ModifyReminderArgs {
    pub note_name: String,
    pub original_text_match: String,
    pub new_date: String,
    pub new_text: String,
    pub new_priority: Option<String>,
    pub new_repeat: Option<String>,
}

pub struct ModifyReminder {
    pub db_path: PathBuf,
}

impl Tool for ModifyReminder {
    const NAME: &'static str = "modify_reminder";

    type Args = ModifyReminderArgs;
    type Output = String;
    type Error = ToolError;

    async fn definition(&self, _prompt: String) -> rig::completion::ToolDefinition {
        rig::completion::ToolDefinition {
            name: "modify_reminder".to_string(),
            description: "Modify an existing reminder. You must provide the new date and text (even if they are the same).".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "note_name": {
                        "type": "string",
                        "description": "The name of the note containing the reminder"
                    },
                    "original_text_match": {
                        "type": "string",
                        "description": "Text to identify the original reminder to modify"
                    },
                    "new_date": {
                        "type": "string",
                        "description": "The new date/time"
                    },
                    "new_text": {
                        "type": "string",
                        "description": "The new reminder text"
                    },
                    "new_priority": {
                        "type": "string",
                        "description": "Optional new priority"
                    },
                    "new_repeat": {
                        "type": "string",
                        "description": "Optional new repeat pattern"
                    }
                },
                "required": ["note_name", "original_text_match", "new_date", "new_text"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let db_path = self.db_path.clone();

        let result = tokio::task::spawn_blocking(move || -> anyhow::Result<String> {
            let db = NotesDatabase::new(&db_path).map_err(|e| anyhow::anyhow!(e))?;
            let metadata = db
                .get_note(&args.note_name)
                .map_err(|e| anyhow::anyhow!(e))?;

            if let Some(meta) = metadata {
                let current_content =
                    std::fs::read_to_string(&meta.path).map_err(|e| anyhow::anyhow!(e))?;

                let mut new_lines = Vec::new();
                let mut found = false;

                // Construct the new reminder string
                let resolved_date = resolve_date(&args.new_date);
                let mut params = resolved_date;

                if let Some(p) = args.new_priority {
                    if !p.is_empty() {
                        params.push_str(&format!(" {}", p));
                    }
                }

                if let Some(r) = args.new_repeat {
                    if !r.is_empty() {
                        params.push_str(&format!(" repeat={}", r));
                    }
                }

                let new_reminder_line = format!("!!RECORDAR({}, {})", params, args.new_text);

                for line in current_content.lines() {
                    // Check if line is a reminder and contains the match text
                    if !found
                        && line.contains("!!RECORDAR")
                        && line.contains(&args.original_text_match)
                    {
                        found = true;
                        new_lines.push(new_reminder_line.as_str());
                    } else {
                        new_lines.push(line);
                    }
                }

                if !found {
                    return Ok(format!(
                        "No reminder found matching '{}' in note '{}'",
                        args.original_text_match, args.note_name
                    ));
                }

                let new_content = new_lines.join("\n");
                std::fs::write(&meta.path, &new_content).map_err(|e| anyhow::anyhow!(e))?;

                // Update in DB
                db.index_note(
                    &args.note_name,
                    &meta.path,
                    &new_content,
                    meta.folder.as_deref(),
                )
                .map_err(|e| anyhow::anyhow!(e))?;

                Ok(format!("Reminder modified in note '{}'", args.note_name))
            } else {
                Err(anyhow::anyhow!("Note '{}' not found", args.note_name))
            }
        })
        .await
        .map_err(|e| ToolError(e.to_string()))??;

        Ok(result)
    }
}

impl ModifyReminder {
    pub fn new(db_path: PathBuf) -> Self {
        Self { db_path }
    }
}
