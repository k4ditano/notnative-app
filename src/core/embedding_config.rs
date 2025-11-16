use serde::{Deserialize, Serialize};

/// Configuración para el sistema de embeddings y búsqueda semántica
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    /// Si el sistema de embeddings está habilitado
    pub enabled: bool,

    /// Proveedor de embeddings: "openrouter", "ollama"
    pub provider: String,

    /// Modelo a usar (ej: "qwen/qwen3-embedding-8b")
    pub model: String,

    /// API key para el proveedor (si es necesario)
    pub api_key: Option<String>,

    /// URL base de la API
    pub api_url: String,

    /// Dimensión de los vectores de embedding que genera el modelo
    pub dimension: usize,

    /// Si el caché de embeddings está habilitado
    pub cache_enabled: bool,

    /// Tamaño máximo de tokens por chunk
    pub max_chunk_tokens: usize,

    /// Overlap de tokens entre chunks
    pub overlap_tokens: usize,

    /// Threshold mínimo de similitud para considerar un match
    pub min_similarity: f32,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            enabled: true, // HABILITADO POR DEFECTO
            provider: "openrouter".to_string(),
            model: "qwen/qwen3-embedding-8b".to_string(),
            api_key: None, // Se tomará de ai_config si no se especifica
            api_url: "https://openrouter.ai/api/v1".to_string(),
            dimension: 4096, // qwen3-embedding-8b usa 4096 dimensiones
            cache_enabled: true,
            max_chunk_tokens: 512,
            overlap_tokens: 50,
            min_similarity: 0.3, // Threshold permisivo (30%)
        }
    }
}

impl EmbeddingConfig {
    /// Crea una nueva configuración con valores por defecto
    pub fn new() -> Self {
        Self::default()
    }

    /// Verifica si la configuración es válida para usar
    pub fn is_valid(&self) -> bool {
        if !self.enabled {
            return false;
        }

        // Verificar que haya API key si el proveedor lo requiere
        if self.provider == "openrouter" && self.api_key.is_none() {
            return false;
        }

        // Verificar que el modelo no esté vacío
        if self.model.is_empty() {
            return false;
        }

        // Verificar que la dimensión sea razonable
        if self.dimension == 0 || self.dimension > 4096 {
            return false;
        }

        true
    }

    /// Obtiene el endpoint completo para embeddings según el proveedor
    pub fn get_embeddings_endpoint(&self) -> String {
        match self.provider.as_str() {
            "openrouter" => format!("{}/embeddings", self.api_url),
            "ollama" => format!("{}/api/embeddings", self.api_url),
            _ => format!("{}/embeddings", self.api_url),
        }
    }

    /// Valida y sanitiza la configuración
    pub fn validate(&mut self) -> Result<(), String> {
        // Limpiar espacios en blanco
        self.model = self.model.trim().to_string();
        self.provider = self.provider.trim().to_lowercase();
        self.api_url = self.api_url.trim().to_string();

        // Verificar proveedor soportado
        if !["openrouter", "ollama"].contains(&self.provider.as_str()) {
            return Err(format!("Proveedor no soportado: {}", self.provider));
        }

        // Verificar dimensión válida
        if self.dimension == 0 {
            return Err("La dimensión debe ser mayor que 0".to_string());
        }

        // Verificar chunk tokens
        if self.max_chunk_tokens == 0 {
            return Err("max_chunk_tokens debe ser mayor que 0".to_string());
        }

        if self.overlap_tokens >= self.max_chunk_tokens {
            return Err("overlap_tokens debe ser menor que max_chunk_tokens".to_string());
        }

        // Verificar threshold
        if self.min_similarity < 0.0 || self.min_similarity > 1.0 {
            return Err("min_similarity debe estar entre 0.0 y 1.0".to_string());
        }

        Ok(())
    }
}

/// Estadísticas de indexación
#[derive(Debug, Clone, Default)]
pub struct IndexStats {
    pub total_notes: usize,
    pub indexed_notes: usize,
    pub total_chunks: usize,
    pub total_tokens: usize,
    pub skipped_notes: usize,
    pub errors: Vec<String>,
}

impl IndexStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_note(&mut self, chunks: usize, tokens: usize) {
        self.indexed_notes += 1;
        self.total_chunks += chunks;
        self.total_tokens += tokens;
    }

    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }

    pub fn skip_note(&mut self) {
        self.skipped_notes += 1;
    }

    pub fn success_rate(&self) -> f32 {
        if self.total_notes == 0 {
            return 0.0;
        }
        (self.indexed_notes as f32 / self.total_notes as f32) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = EmbeddingConfig::default();
        assert_eq!(config.enabled, true); // Habilitado por defecto
        assert_eq!(config.provider, "openrouter");
        assert_eq!(config.dimension, 4096); // qwen3-embedding-8b usa 4096 dimensiones
    }

    #[test]
    fn test_config_validation() {
        let mut config = EmbeddingConfig::default();
        config.enabled = true;

        // Sin API key, debería ser inválido para OpenRouter
        assert!(!config.is_valid());

        // Con API key, debería ser válido
        config.api_key = Some("test-key".to_string());
        assert!(config.is_valid());
    }

    #[test]
    fn test_validate_method() {
        let mut config = EmbeddingConfig::default();
        config.provider = " OpenRouter ".to_string();
        config.model = " qwen/test ".to_string();

        assert!(config.validate().is_ok());
        assert_eq!(config.provider, "openrouter");
        assert_eq!(config.model, "qwen/test");
    }

    #[test]
    fn test_invalid_dimension() {
        let mut config = EmbeddingConfig::default();
        config.dimension = 0;

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_index_stats() {
        let mut stats = IndexStats::new();
        stats.total_notes = 10;
        stats.add_note(5, 100);
        stats.add_note(3, 80);
        stats.skip_note();

        assert_eq!(stats.indexed_notes, 2);
        assert_eq!(stats.total_chunks, 8);
        assert_eq!(stats.total_tokens, 180);
        assert_eq!(stats.skipped_notes, 1);
        assert_eq!(stats.success_rate(), 20.0);
    }
}
