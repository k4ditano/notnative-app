use anyhow::{Result, anyhow};

/// Un chunk de texto con información de contexto
#[derive(Debug, Clone)]
pub struct TextChunk {
    /// Texto del chunk
    pub text: String,
    /// Índice del chunk en el documento original (0-based)
    pub index: usize,
    /// Posición de inicio en el texto original (en caracteres)
    pub start_pos: usize,
    /// Posición de fin en el texto original (en caracteres)
    pub end_pos: usize,
    /// Número estimado de tokens
    pub token_count: usize,
}

/// Configuración para el chunking
#[derive(Debug, Clone)]
pub struct ChunkConfig {
    /// Máximo de tokens por chunk
    pub max_tokens: usize,
    /// Tokens de overlap entre chunks
    pub overlap_tokens: usize,
    /// Estimación de caracteres por token (promedio)
    pub chars_per_token: f32,
}

impl Default for ChunkConfig {
    fn default() -> Self {
        Self {
            max_tokens: 512,
            overlap_tokens: 50,
            chars_per_token: 4.0, // Estimación para español/inglés
        }
    }
}

impl ChunkConfig {
    /// Calcula el tamaño aproximado de chunk en caracteres
    pub fn chunk_size_chars(&self) -> usize {
        (self.max_tokens as f32 * self.chars_per_token) as usize
    }

    /// Calcula el tamaño de overlap en caracteres
    pub fn overlap_size_chars(&self) -> usize {
        (self.overlap_tokens as f32 * self.chars_per_token) as usize
    }

    /// Estima el número de tokens en un texto
    pub fn estimate_tokens(&self, text: &str) -> usize {
        (text.len() as f32 / self.chars_per_token).ceil() as usize
    }
}

/// Chunker de texto que divide documentos en fragmentos manejables
pub struct TextChunker {
    config: ChunkConfig,
}

impl TextChunker {
    /// Crea un nuevo chunker con la configuración por defecto
    pub fn new() -> Self {
        Self {
            config: ChunkConfig::default(),
        }
    }

    /// Crea un nuevo chunker con configuración personalizada
    pub fn with_config(config: ChunkConfig) -> Self {
        Self { config }
    }

    /// Divide un texto en chunks
    pub fn chunk_text(&self, text: &str) -> Result<Vec<TextChunk>> {
        if text.is_empty() {
            return Ok(Vec::new());
        }

        let estimated_tokens = self.config.estimate_tokens(text);

        // Si el texto es más pequeño que max_tokens, retornar un solo chunk
        if estimated_tokens <= self.config.max_tokens {
            return Ok(vec![TextChunk {
                text: text.to_string(),
                index: 0,
                start_pos: 0,
                end_pos: text.len(),
                token_count: estimated_tokens,
            }]);
        }

        // Dividir en chunks con overlap
        let mut chunks = Vec::new();
        let chunk_size = self.config.chunk_size_chars();
        let overlap_size = self.config.overlap_size_chars();
        let step_size = chunk_size.saturating_sub(overlap_size);

        if step_size == 0 {
            return Err(anyhow!("Overlap size es igual o mayor que chunk size"));
        }

        let mut index = 0;
        let mut start = 0;

        while start < text.len() {
            // Determinar el fin del chunk
            let mut end = (start + chunk_size).min(text.len());

            // Intentar romper en límites de palabras si no es el último chunk
            if end < text.len() {
                end = self.find_word_boundary(text, end);
            }

            // Extraer el chunk
            let chunk_text = &text[start..end];
            let token_count = self.config.estimate_tokens(chunk_text);

            chunks.push(TextChunk {
                text: chunk_text.to_string(),
                index,
                start_pos: start,
                end_pos: end,
                token_count,
            });

            index += 1;

            // Avanzar al siguiente chunk con overlap
            start += step_size;

            // Si el siguiente chunk sería muy pequeño, terminar
            if start >= text.len() {
                break;
            }

            // Evitar chunks demasiado pequeños al final
            let remaining = text.len() - start;
            if remaining < chunk_size / 4 && !chunks.is_empty() {
                // Extender el último chunk para incluir el texto restante
                if let Some(last_chunk) = chunks.last_mut() {
                    last_chunk.text = text[last_chunk.start_pos..].to_string();
                    last_chunk.end_pos = text.len();
                    last_chunk.token_count = self.config.estimate_tokens(&last_chunk.text);
                }
                break;
            }
        }

        Ok(chunks)
    }

    /// Encuentra el límite de palabra más cercano antes de la posición dada
    fn find_word_boundary(&self, text: &str, pos: usize) -> usize {
        // Buscar hacia atrás hasta 50 caracteres para encontrar un límite de palabra
        let search_start = pos.saturating_sub(50);
        let search_text = &text[search_start..pos];

        // Buscar el último espacio, salto de línea o puntuación
        if let Some(boundary_pos) = search_text
            .rfind(|c: char| c.is_whitespace() || matches!(c, '.' | '!' | '?' | ',' | ';' | ':'))
        {
            // Ajustar la posición relativa al texto original
            search_start + boundary_pos + 1
        } else {
            // Si no se encuentra límite, usar la posición original
            pos
        }
    }

    /// Divide un texto preservando párrafos cuando sea posible
    pub fn chunk_by_paragraphs(&self, text: &str) -> Result<Vec<TextChunk>> {
        if text.is_empty() {
            return Ok(Vec::new());
        }

        let estimated_tokens = self.config.estimate_tokens(text);

        // Si el texto es pequeño, retornar un solo chunk
        if estimated_tokens <= self.config.max_tokens {
            return Ok(vec![TextChunk {
                text: text.to_string(),
                index: 0,
                start_pos: 0,
                end_pos: text.len(),
                token_count: estimated_tokens,
            }]);
        }

        // Dividir en párrafos
        let paragraphs: Vec<&str> = text.split("\n\n").collect();

        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        let mut current_start = 0;
        let mut index = 0;

        for (i, paragraph) in paragraphs.iter().enumerate() {
            let para_tokens = self.config.estimate_tokens(paragraph);
            let current_tokens = self.config.estimate_tokens(&current_chunk);

            // Si el párrafo solo es muy grande, chunkearlo individualmente
            if para_tokens > self.config.max_tokens {
                // Guardar chunk actual si existe
                if !current_chunk.is_empty() {
                    let token_count = self.config.estimate_tokens(&current_chunk);
                    chunks.push(TextChunk {
                        text: current_chunk.clone(),
                        index,
                        start_pos: current_start,
                        end_pos: current_start + current_chunk.len(),
                        token_count,
                    });
                    index += 1;
                    current_chunk.clear();
                }

                // Chunkear el párrafo grande
                let para_chunks = self.chunk_text(paragraph)?;
                for mut para_chunk in para_chunks {
                    para_chunk.index = index;
                    chunks.push(para_chunk);
                    index += 1;
                }

                current_start += paragraph.len() + 2; // +2 por "\n\n"
                continue;
            }

            // Si agregar este párrafo excedería el límite, guardar chunk actual
            if current_tokens + para_tokens > self.config.max_tokens && !current_chunk.is_empty() {
                let token_count = self.config.estimate_tokens(&current_chunk);
                chunks.push(TextChunk {
                    text: current_chunk.clone(),
                    index,
                    start_pos: current_start,
                    end_pos: current_start + current_chunk.len(),
                    token_count,
                });
                index += 1;
                current_chunk.clear();
                current_start += current_chunk.len();
            }

            // Agregar párrafo al chunk actual
            if !current_chunk.is_empty() {
                current_chunk.push_str("\n\n");
            }
            current_chunk.push_str(paragraph);
        }

        // Agregar último chunk si existe
        if !current_chunk.is_empty() {
            let token_count = self.config.estimate_tokens(&current_chunk);
            chunks.push(TextChunk {
                text: current_chunk,
                index,
                start_pos: current_start,
                end_pos: text.len(),
                token_count,
            });
        }

        Ok(chunks)
    }

    /// Obtiene la configuración actual
    pub fn config(&self) -> &ChunkConfig {
        &self.config
    }
}

impl Default for TextChunker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_small_text() {
        let chunker = TextChunker::new();
        let text = "Este es un texto pequeño que no necesita chunking.";

        let chunks = chunker.chunk_text(text).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].text, text);
        assert_eq!(chunks[0].index, 0);
        assert_eq!(chunks[0].start_pos, 0);
        assert_eq!(chunks[0].end_pos, text.len());
    }

    #[test]
    fn test_empty_text() {
        let chunker = TextChunker::new();
        let text = "";

        let chunks = chunker.chunk_text(text).unwrap();
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_large_text_chunking() {
        let chunker = TextChunker::new();

        // Crear un texto largo (más de 512 tokens estimados)
        let text = "palabra ".repeat(3000); // ~3000 tokens

        let chunks = chunker.chunk_text(&text).unwrap();

        // Debería haber múltiples chunks
        assert!(chunks.len() > 1);

        // Verificar que los índices son consecutivos
        for (i, chunk) in chunks.iter().enumerate() {
            assert_eq!(chunk.index, i);
        }

        // Verificar que ningún chunk es demasiado grande
        for chunk in &chunks {
            assert!(chunk.token_count <= chunker.config.max_tokens);
        }

        // Verificar overlap: el final de un chunk debería estar cerca del inicio del siguiente
        for i in 0..chunks.len() - 1 {
            let current = &chunks[i];
            let next = &chunks[i + 1];

            // Debería haber overlap
            assert!(current.end_pos > next.start_pos);
        }
    }

    #[test]
    fn test_paragraph_chunking() {
        let chunker = TextChunker::new();

        let text = "Párrafo uno.\n\nPárrafo dos.\n\nPárrafo tres.";

        let chunks = chunker.chunk_by_paragraphs(text).unwrap();

        // Texto pequeño debería resultar en un solo chunk
        assert_eq!(chunks.len(), 1);
    }

    #[test]
    fn test_large_paragraph_chunking() {
        let chunker = TextChunker::new();

        // Crear múltiples párrafos largos
        let para1 = "palabra ".repeat(300); // ~300 tokens
        let para2 = "texto ".repeat(300); // ~300 tokens
        let para3 = "contenido ".repeat(300); // ~300 tokens

        let text = format!("{}\n\n{}\n\n{}", para1, para2, para3);

        let chunks = chunker.chunk_by_paragraphs(&text).unwrap();

        // Debería haber múltiples chunks
        assert!(chunks.len() >= 2);

        // Verificar índices consecutivos
        for (i, chunk) in chunks.iter().enumerate() {
            assert_eq!(chunk.index, i);
        }
    }

    #[test]
    fn test_custom_config() {
        let config = ChunkConfig {
            max_tokens: 100,
            overlap_tokens: 10,
            chars_per_token: 5.0,
        };

        let chunker = TextChunker::with_config(config);

        assert_eq!(chunker.config.max_tokens, 100);
        assert_eq!(chunker.config.overlap_tokens, 10);
        assert_eq!(chunker.config.chars_per_token, 5.0);
    }

    #[test]
    fn test_token_estimation() {
        let config = ChunkConfig::default();

        let text = "Hola mundo"; // 10 caracteres
        let estimated = config.estimate_tokens(text);

        // 10 chars / 4 chars_per_token = 2.5 -> ceil = 3 tokens
        assert_eq!(estimated, 3);
    }

    #[test]
    fn test_word_boundary() {
        let chunker = TextChunker::new();

        let text = "Este es un texto largo con muchas palabras para probar el límite de palabras";

        let chunks = chunker.chunk_text(text).unwrap();

        // Si hay múltiples chunks, verificar que no rompen palabras
        if chunks.len() > 1 {
            for chunk in &chunks {
                // El chunk no debería terminar en medio de una palabra
                // (excepto el último)
                if chunk.index < chunks.len() - 1 {
                    let last_char = chunk.text.chars().last();
                    assert!(
                        last_char.map_or(false, |c| c.is_whitespace() || c.is_ascii_punctuation())
                    );
                }
            }
        }
    }

    #[test]
    fn test_chunk_coverage() {
        let chunker = TextChunker::new();

        let text = "palabra ".repeat(3000);

        let chunks = chunker.chunk_text(&text).unwrap();

        // El primer chunk debería empezar al inicio
        assert_eq!(chunks[0].start_pos, 0);

        // El último chunk debería terminar al final
        assert_eq!(chunks.last().unwrap().end_pos, text.len());

        // Verificar que hay overlap entre chunks consecutivos
        for i in 0..chunks.len() - 1 {
            let current = &chunks[i];
            let next = &chunks[i + 1];

            // El siguiente chunk debería empezar antes de que termine el actual
            assert!(
                next.start_pos < current.end_pos,
                "Chunk {} termina en {} pero chunk {} empieza en {}",
                i,
                current.end_pos,
                i + 1,
                next.start_pos
            );
        }
    }
}
