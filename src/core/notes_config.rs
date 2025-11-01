use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Configuración del orden y organización de notas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotesConfig {
    /// Orden personalizado de las notas (nota -> posición)
    pub order: HashMap<String, usize>,
    /// Carpetas que están expandidas
    pub expanded_folders: Vec<String>,
}

impl Default for NotesConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl NotesConfig {
    /// Crea una nueva configuración vacía
    pub fn new() -> Self {
        Self {
            order: HashMap::new(),
            expanded_folders: Vec::new(),
        }
    }
    
    /// Carga la configuración desde un archivo
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: NotesConfig = serde_json::from_str(&content)?;
        Ok(config)
    }
    
    /// Guarda la configuración a un archivo
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
    
    /// Obtiene la posición de una nota en el orden personalizado
    pub fn get_position(&self, note_name: &str) -> Option<usize> {
        self.order.get(note_name).copied()
    }
    
    /// Establece la posición de una nota
    pub fn set_position(&mut self, note_name: String, position: usize) {
        self.order.insert(note_name, position);
    }
    
    /// Remueve una nota del orden
    pub fn remove_note(&mut self, note_name: &str) {
        self.order.remove(note_name);
    }
    
    /// Mueve una nota a una nueva posición, reordenando las demás
    pub fn move_note(&mut self, note_name: &str, new_position: usize) {
        // Obtener posición actual
        let old_position = self.get_position(note_name);
        
        // Actualizar posiciones de todas las notas afectadas
        if let Some(old_pos) = old_position {
            if old_pos < new_position {
                // Moviendo hacia abajo: decrementar posiciones entre old y new
                for (_name, pos) in self.order.iter_mut() {
                    if *pos > old_pos && *pos <= new_position {
                        *pos -= 1;
                    }
                }
            } else if old_pos > new_position {
                // Moviendo hacia arriba: incrementar posiciones entre new y old
                for (_name, pos) in self.order.iter_mut() {
                    if *pos >= new_position && *pos < old_pos {
                        *pos += 1;
                    }
                }
            }
        } else {
            // Nueva nota: incrementar todas las posiciones >= new_position
            for (_name, pos) in self.order.iter_mut() {
                if *pos >= new_position {
                    *pos += 1;
                }
            }
        }
        
        // Establecer nueva posición
        self.order.insert(note_name.to_string(), new_position);
    }
    
    /// Verifica si una carpeta está expandida
    pub fn is_folder_expanded(&self, folder: &str) -> bool {
        self.expanded_folders.contains(&folder.to_string())
    }
    
    /// Alterna el estado de expansión de una carpeta
    pub fn toggle_folder(&mut self, folder: String) {
        if let Some(pos) = self.expanded_folders.iter().position(|f| f == &folder) {
            self.expanded_folders.remove(pos);
        } else {
            self.expanded_folders.push(folder);
        }
    }
    
    /// Ruta por defecto del archivo de configuración
    pub fn default_path() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("notnative")
            .join("config.json")
    }
}
