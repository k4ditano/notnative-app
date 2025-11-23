use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;

use crate::ai::agent::{Agent, ExecutorType};
use crate::ai::executors::react::ReActStep;
use crate::ai_chat::{ChatMessage, MessageRole};
use crate::ai_client::AIClient;
use crate::mcp::MCPToolExecutor;

/// Clasificación de la intención del usuario
#[derive(Debug, Clone)]
pub struct IntentClassification {
    pub agent_type: String,
    pub confidence: f32,
}

/// Router que clasifica la intención del usuario y delega al agente apropiado
#[derive(Clone)]
pub struct RouterAgent {
    llm: Arc<dyn AIClient>,
    agents: HashMap<String, Agent>,
}

impl std::fmt::Debug for RouterAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RouterAgent")
            .field("llm", &"Arc<dyn AIClient>")
            .field("agents", &self.agents)
            .finish()
    }
}

impl RouterAgent {
    /// Crea un nuevo router con los agentes predefinidos
    pub fn new(llm: Arc<dyn AIClient>) -> Self {
        let mut agents = HashMap::new();

        {
            agents.insert("rig".to_string(), Agent::rig_agent());
        }

        agents.insert("create".to_string(), Agent::create_agent());
        agents.insert("search".to_string(), Agent::search_agent());
        agents.insert("analyze".to_string(), Agent::analyze_agent());
        agents.insert("execute".to_string(), Agent::multi_step_agent());
        agents.insert("chat".to_string(), Agent::chat_agent());

        Self { llm, agents }
    }

    /// Obtiene una referencia al cliente LLM
    pub fn get_llm(&self) -> Arc<dyn AIClient> {
        self.llm.clone()
    }

    /// Clasifica la intención y ejecuta con el agente apropiado
    pub async fn route_and_execute<F>(
        &self,
        messages: &[ChatMessage],
        context: &str,
        mcp_executor: &MCPToolExecutor,
        step_callback: F,
    ) -> Result<String>
    where
        F: FnMut(&ReActStep) + Send + 'static,
    {
        // Extraer el último mensaje del usuario como la tarea actual
        let task = messages.last().map(|m| m.content.as_str()).unwrap_or("");

        // 1. Clasificar la intención
        let classification = self.classify_intent(task).await?;

        println!(
            "🎯 Intent classified as: {} (confidence: {:.2})",
            classification.agent_type, classification.confidence
        );

        // 2. Obtener agente apropiado
        let agent = self.agents.get(&classification.agent_type).ok_or_else(|| {
            anyhow::anyhow!("Agente no encontrado: {}", classification.agent_type)
        })?;

        println!("🤖 Using agent: {}", agent.name);

        // 3. Ejecutar con el agente seleccionado (pasando historial completo y callback)
        agent
            .run(
                messages,
                context,
                self.llm.clone(),
                mcp_executor,
                step_callback,
            )
            .await
    }

    /// Clasifica la intención del usuario usando el LLM
    async fn classify_intent(&self, _task: &str) -> Result<IntentClassification> {
        // RIG es ahora el agente predeterminado para todas las tareas
        Ok(IntentClassification {
            agent_type: "rig".to_string(),
            confidence: 1.0,
        })
    }
}
