use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::embedding_config::EmbeddingConfig;

/// Trait para proveedores de embeddings
#[async_trait::async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// Generar embedding para un texto
    async fn embed_text(&self, text: &str) -> Result<Vec<f32>>;

    /// Generar embeddings para múltiples textos (batch)
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>>;

    /// Obtener dimensión de los embeddings
    fn dimension(&self) -> usize;

    /// Nombre del proveedor
    fn provider_name(&self) -> &str;
}

/// Cliente de embeddings que gestiona diferentes proveedores
pub struct EmbeddingClient {
    provider: Box<dyn EmbeddingProvider>,
    config: EmbeddingConfig,
}

impl EmbeddingClient {
    /// Crear un nuevo cliente basado en la configuración
    pub fn new(config: EmbeddingConfig) -> Result<Self> {
        println!(
            "🔍 DEBUG: Creando EmbeddingClient con config: api_key={:?}, provider={}, model={}",
            config.api_key.as_ref().map(|k| if k.len() > 10 {
                format!("{}...", &k[..10])
            } else {
                k.clone()
            }),
            config.provider,
            config.model
        );

        if !config.is_valid() {
            return Err(anyhow!("Configuración de embeddings inválida"));
        }

        let provider: Box<dyn EmbeddingProvider> = match config.provider.as_str() {
            "openrouter" => Box::new(OpenRouterProvider::new(
                config.api_key.clone().unwrap_or_default(),
                config.model.clone(),
                config.dimension,
                Some(config.api_url.clone()),
            )?),
            "ollama" => Box::new(OllamaProvider::new(
                config.model.clone(),
                config.dimension,
                Some(config.api_url.clone()),
            )?),
            _ => return Err(anyhow!("Proveedor desconocido: {}", config.provider)),
        };

        Ok(Self { provider, config })
    }

    /// Generar embedding para un texto
    pub async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        self.provider.embed_text(text).await
    }

    /// Generar embeddings para múltiples textos
    pub async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        self.provider.embed_batch(texts).await
    }

    /// Obtener la configuración actual
    pub fn config(&self) -> &EmbeddingConfig {
        &self.config
    }

    /// Obtener dimensión de embeddings
    pub fn dimension(&self) -> usize {
        self.provider.dimension()
    }
}

// ============================================================================
// OpenRouter Provider
// ============================================================================

/// Proveedor de embeddings usando OpenRouter API
struct OpenRouterProvider {
    api_key: String,
    model: String,
    dimension: usize,
    api_url: String,
    client: reqwest::Client,
}

#[derive(Serialize)]
struct OpenRouterEmbeddingRequest {
    model: String,
    input: EmbeddingInput,
}

#[derive(Serialize)]
#[serde(untagged)]
enum EmbeddingInput {
    Single(String),
    Batch(Vec<String>),
}

#[derive(Deserialize)]
struct OpenRouterEmbeddingResponse {
    data: Vec<EmbeddingData>,
}

#[derive(Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
    index: usize,
}

#[derive(Deserialize)]
struct OpenRouterErrorResponse {
    error: OpenRouterError,
}

#[derive(Deserialize)]
struct OpenRouterError {
    message: String,
    #[serde(rename = "type")]
    error_type: Option<String>,
    code: Option<String>,
}

impl OpenRouterProvider {
    fn new(
        api_key: String,
        model: String,
        dimension: usize,
        api_url: Option<String>,
    ) -> Result<Self> {
        if api_key.is_empty() {
            return Err(anyhow!("API key de OpenRouter requerida"));
        }

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .context("Error creando cliente HTTP")?;

        let api_url = api_url.unwrap_or_else(|| "https://openrouter.ai/api/v1".to_string());

        Ok(Self {
            api_key,
            model,
            dimension,
            api_url,
            client,
        })
    }

    /// Realizar petición con retry y backoff exponencial
    async fn request_with_retry<T: serde::de::DeserializeOwned>(
        &self,
        input: EmbeddingInput,
        max_retries: u32,
    ) -> Result<T> {
        let mut retries = 0;
        let mut delay = Duration::from_millis(500);

        loop {
            let request = OpenRouterEmbeddingRequest {
                model: self.model.clone(),
                input: match &input {
                    EmbeddingInput::Single(s) => EmbeddingInput::Single(s.clone()),
                    EmbeddingInput::Batch(b) => EmbeddingInput::Batch(b.clone()),
                },
            };

            eprintln!(
                "🔍 DEBUG: Enviando request a {}/embeddings con modelo {}",
                self.api_url, self.model
            );

            let response = self
                .client
                .post(format!("{}/embeddings", self.api_url))
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("Content-Type", "application/json")
                .json(&request)
                .send()
                .await;

            eprintln!(
                "🔍 DEBUG: Respuesta recibida: {:?}",
                response.as_ref().map(|r| r.status())
            );

            match response {
                Ok(resp) => {
                    let status = resp.status();

                    if status.is_success() {
                        let json_result = resp.json::<T>().await;
                        if let Err(ref e) = json_result {
                            eprintln!("🔍 DEBUG: Error deserializando respuesta exitosa: {}", e);
                        }
                        return json_result.context("Error deserializando respuesta");
                    }

                    // Intentar obtener el mensaje de error
                    let error_body = resp
                        .text()
                        .await
                        .unwrap_or_else(|_| String::from("(no se pudo leer el cuerpo)"));
                    eprintln!("🔍 DEBUG: Error HTTP {}: {}", status, error_body);

                    let error_msg = format!("OpenRouter error: HTTP {} - {}", status, error_body);

                    // Rate limit o errores de servidor: reintentar
                    if status.as_u16() == 429 || status.is_server_error() {
                        if retries < max_retries {
                            retries += 1;
                            eprintln!(
                                "⚠️  Error {}: Reintentando en {:?} ({}/{})",
                                status, delay, retries, max_retries
                            );
                            tokio::time::sleep(delay).await;
                            delay *= 2; // Backoff exponencial
                            continue;
                        }
                    }

                    return Err(anyhow!(error_msg));
                }
                Err(e) => {
                    if retries < max_retries {
                        retries += 1;
                        eprintln!(
                            "⚠️  Error de conexión: {}. Reintentando en {:?} ({}/{})",
                            e, delay, retries, max_retries
                        );
                        tokio::time::sleep(delay).await;
                        delay *= 2;
                        continue;
                    }
                    return Err(anyhow!("Error de conexión: {}", e));
                }
            }
        }
    }
}

#[async_trait::async_trait]
impl EmbeddingProvider for OpenRouterProvider {
    async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        let response: OpenRouterEmbeddingResponse = self
            .request_with_retry(EmbeddingInput::Single(text.to_string()), 3)
            .await
            .context("Error obteniendo embedding de OpenRouter")?;

        if response.data.is_empty() {
            return Err(anyhow!("OpenRouter devolvió respuesta vacía"));
        }

        let embedding = &response.data[0].embedding;

        if embedding.len() != self.dimension {
            return Err(anyhow!(
                "Dimensión incorrecta: esperaba {}, obtuvo {}",
                self.dimension,
                embedding.len()
            ));
        }

        Ok(embedding.clone())
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        eprintln!("🔍 DEBUG: embed_batch llamado con {} textos", texts.len());
        eprintln!(
            "🔍 DEBUG: Primer texto (primeros 50 chars): {}",
            texts.first().map(|s| &s[..s.len().min(50)]).unwrap_or("")
        );

        // OpenRouter recomienda máximo 100 textos por batch
        const MAX_BATCH_SIZE: usize = 100;

        if texts.len() > MAX_BATCH_SIZE {
            eprintln!("🔍 DEBUG: Batch grande, dividiendo en chunks");
            // Dividir en chunks y procesar recursivamente
            let mut all_embeddings = Vec::with_capacity(texts.len());

            for chunk in texts.chunks(MAX_BATCH_SIZE) {
                let chunk_embeddings = self.embed_batch(chunk).await?;
                all_embeddings.extend(chunk_embeddings);
            }

            return Ok(all_embeddings);
        }

        eprintln!("🔍 DEBUG: Llamando request_with_retry...");
        let response: OpenRouterEmbeddingResponse = self
            .request_with_retry(EmbeddingInput::Batch(texts.to_vec()), 3)
            .await
            .map_err(|e| {
                eprintln!("🔍 DEBUG: Error en request_with_retry: {}", e);
                e
            })
            .context("Error obteniendo embeddings batch de OpenRouter")?;

        eprintln!(
            "🔍 DEBUG: Respuesta recibida con {} embeddings",
            response.data.len()
        );

        if response.data.len() != texts.len() {
            eprintln!(
                "❌ ERROR: Respuesta batch incompleta: esperaba {}, obtuvo {}",
                texts.len(),
                response.data.len()
            );
            return Err(anyhow!(
                "Respuesta batch incompleta: esperaba {}, obtuvo {}",
                texts.len(),
                response.data.len()
            ));
        }

        eprintln!("🔍 DEBUG: Ordenando por índice...");
        // Ordenar por índice para asegurar el orden correcto
        let mut data = response.data;
        data.sort_by_key(|d| d.index);

        eprintln!(
            "🔍 DEBUG: Validando dimensiones (esperado: {})...",
            self.dimension
        );
        // Validar dimensiones
        for (i, item) in data.iter().enumerate() {
            if item.embedding.len() != self.dimension {
                eprintln!(
                    "❌ ERROR: Dimensión incorrecta en índice {}: esperada {}, obtenida {}",
                    i,
                    self.dimension,
                    item.embedding.len()
                );
                return Err(anyhow!(
                    "Dimensión incorrecta en índice {}: esperada {}, obtenida {}",
                    i,
                    self.dimension,
                    item.embedding.len()
                ));
            }
        }

        eprintln!(
            "✅ DEBUG: Validación OK, extrayendo {} embeddings",
            data.len()
        );
        Ok(data.into_iter().map(|d| d.embedding).collect())
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn provider_name(&self) -> &str {
        "openrouter"
    }
}

// ============================================================================
// Ollama Provider (Stub para futuro)
// ============================================================================

/// Proveedor de embeddings usando Ollama local
struct OllamaProvider {
    model: String,
    dimension: usize,
    #[allow(dead_code)]
    api_url: String,
}

impl OllamaProvider {
    fn new(model: String, dimension: usize, api_url: Option<String>) -> Result<Self> {
        let api_url = api_url.unwrap_or_else(|| "http://localhost:11434".to_string());

        Ok(Self {
            model,
            dimension,
            api_url,
        })
    }
}

#[async_trait::async_trait]
impl EmbeddingProvider for OllamaProvider {
    async fn embed_text(&self, _text: &str) -> Result<Vec<f32>> {
        Err(anyhow!(
            "Ollama provider no implementado aún. Use 'openrouter' como proveedor."
        ))
    }

    async fn embed_batch(&self, _texts: &[String]) -> Result<Vec<Vec<f32>>> {
        Err(anyhow!(
            "Ollama provider no implementado aún. Use 'openrouter' como proveedor."
        ))
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn provider_name(&self) -> &str {
        "ollama"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let config = EmbeddingConfig::default();

        // Sin API key debería fallar
        let result = EmbeddingClient::new(config);
        assert!(result.is_err());

        // Con API key debería funcionar
        let mut config = EmbeddingConfig::default();
        config.enabled = true;
        config.api_key = Some("test-key".to_string());
        let result = EmbeddingClient::new(config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_provider_creation() {
        // OpenRouter sin API key debería fallar
        let result = OpenRouterProvider::new(
            "".to_string(),
            "qwen/qwen3-embedding-8b".to_string(),
            1024,
            None,
        );
        assert!(result.is_err());

        // OpenRouter con API key debería funcionar
        let result = OpenRouterProvider::new(
            "test-key".to_string(),
            "qwen/qwen3-embedding-8b".to_string(),
            1024,
            None,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_ollama_not_implemented() {
        let provider = OllamaProvider::new("nomic-embed-text".to_string(), 768, None).unwrap();

        // Debería devolver error indicando que no está implementado
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(provider.embed_text("test"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no implementado"));
    }
}
