/// Representa los diferentes modos de edición estilo vim
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorMode {
    /// Modo normal: navegación y comandos de movimiento
    Normal,
    /// Modo inserción: edición de texto libre
    Insert,
    /// Modo comando: entrada de comandos con prefijo ':'
    Command,
    /// Modo visual: selección de texto (futuro)
    Visual,
}

impl EditorMode {
    /// Devuelve el nombre del modo para mostrar en UI
    pub fn name(&self) -> &'static str {
        match self {
            EditorMode::Normal => "NORMAL",
            EditorMode::Insert => "INSERT",
            EditorMode::Command => "COMMAND",
            EditorMode::Visual => "VISUAL",
        }
    }

    /// Devuelve si el modo permite edición directa de texto
    pub fn is_editable(&self) -> bool {
        matches!(self, EditorMode::Insert)
    }
}

impl Default for EditorMode {
    fn default() -> Self {
        EditorMode::Normal
    }
}
