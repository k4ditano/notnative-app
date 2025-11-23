//! Adapter para RIG Framework que mantiene compatibilidad con el trait AIClient existente
//!
//! Este módulo permite usar RIG como backend alternativo sin modificar el código existente.

use anyhow::Result;
use async_trait::async_trait;
use rig::client::CompletionClient;
use rig::completion::{Completion, Prompt};
use rig::providers::openai::{Client as OpenAIClient, CompletionModel};
use rig::providers::openrouter;

use crate::ai_chat::{AIModelConfig, ChatMessage, MessageRole};
use crate::ai_client::{AIClient, AIResponse};
use crate::mcp::MCPToolRegistry;

/// Cliente RIG que puede usar OpenAI u OpenRouter
pub enum RigClientBackend {
    OpenAI(OpenAIClient),
    OpenRouter(openrouter::Client),
}

/// Cliente RIG que implementa el trait AIClient
pub struct RigClient {
    pub backend: RigClientBackend,
    pub model: String,
    pub temperature: f32,
}

impl RigClient {
    /// Crea un nuevo cliente RIG desde la configuración (OpenAI nativo)
    pub fn new(config: &AIModelConfig, api_key: &str) -> Result<Self> {
        let client = OpenAIClient::new(api_key);

        Ok(Self {
            backend: RigClientBackend::OpenAI(client),
            model: config.model.clone(),
            temperature: config.temperature,
        })
    }

    /// Crea un nuevo cliente RIG usando OpenRouter
    pub fn new_openrouter(config: &AIModelConfig, api_key: &str) -> Result<Self> {
        let client = openrouter::Client::new(api_key);

        Ok(Self {
            backend: RigClientBackend::OpenRouter(client),
            model: config.model.clone(),
            temperature: config.temperature,
        })
    }

    /// Crea un cliente OpenAI para embeddings usando URL de OpenRouter
    /// OpenRouter expone un endpoint compatible con OpenAI en /api/v1
    pub fn create_openrouter_embedding_client(api_key: &str) -> OpenAIClient {
        use rig::providers::openai::ClientBuilder;

        eprintln!("🔧 Creando cliente de embeddings para OpenRouter...");
        eprintln!("   API Key: {}...", &api_key[..15]);
        eprintln!("   URL: https://openrouter.ai/api/v1");

        // Crear cliente simple sin headers personalizados - dejar que RIG maneje todo
        let client = ClientBuilder::new(api_key)
            .base_url("https://openrouter.ai/api/v1")
            .build();

        eprintln!("✅ Cliente creado");
        client
    }
}

#[async_trait]
impl AIClient for RigClient {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    async fn send_message_with_tools(
        &self,
        messages: &[ChatMessage],
        context: &str,
        _tools: Option<&MCPToolRegistry>,
    ) -> Result<AIResponse> {
        // Convertir mensajes al formato de RIG
        let mut prompt = if !context.is_empty() {
            format!("System: {}\n\n", context)
        } else {
            String::new()
        };

        for m in messages {
            match m.role {
                MessageRole::User => prompt.push_str(&format!("User: {}\n", m.content)),
                MessageRole::Assistant => prompt.push_str(&format!("Assistant: {}\n", m.content)),
                MessageRole::System => prompt.push_str(&format!("System: {}\n", m.content)),
            }
        }

        // Crear agente según el backend
        let response = match &self.backend {
            RigClientBackend::OpenAI(client) => {
                let agent = client.agent(&self.model).build();
                agent.prompt(&prompt).await?
            }
            RigClientBackend::OpenRouter(client) => {
                let agent = client.agent(&self.model).build();
                agent.prompt(&prompt).await?
            }
        };

        Ok(AIResponse {
            content: Some(response),
            tool_calls: Vec::new(),
        })
    }

    async fn send_message_streaming(
        &self,
        messages: &[ChatMessage],
        context: &str,
    ) -> Result<tokio::sync::mpsc::UnboundedReceiver<String>> {
        // Streaming simulado (devuelve todo de una vez)
        let response = self
            .send_message_with_tools(messages, context, None)
            .await?;
        let content = response.content.unwrap_or_default();

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let _ = tx.send(content);

        Ok(rx)
    }
}
