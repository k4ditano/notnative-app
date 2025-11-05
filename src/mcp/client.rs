use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Cliente para conectar a servidores MCP externos
pub struct MCPClient {
    base_url: String,
    client: reqwest::Client,
}

/// Request JSON-RPC genérico
#[derive(Debug, Serialize)]
struct JsonRpcRequest<T> {
    jsonrpc: String,
    id: i32,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<T>,
}

/// Response JSON-RPC genérico
#[derive(Debug, Deserialize)]
struct JsonRpcResponse<T> {
    jsonrpc: String,
    id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

/// Respuesta de list_tools
#[derive(Debug, Deserialize)]
pub struct ListToolsResponse {
    pub tools: Vec<Value>,
}

/// Parámetros para call_tool
#[derive(Debug, Serialize)]
pub struct CallToolParams {
    pub tool: String,
    pub args: Value,
}

impl MCPClient {
    /// Crea un nuevo cliente MCP
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            client: reqwest::Client::new(),
        }
    }

    /// Verifica la conexión con el servidor MCP
    pub async fn connect(&self) -> Result<()> {
        let url = format!("{}/health", self.base_url);
        let response = self.client.get(&url).send().await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(anyhow!(
                "Error conectando al servidor MCP: {}",
                response.status()
            ))
        }
    }

    /// Lista todas las herramientas disponibles en el servidor MCP
    pub async fn list_tools(&self) -> Result<Vec<Value>> {
        let url = format!("{}/mcp/list_tools", self.base_url);

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "list_tools".to_string(),
            params: None::<Value>,
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await?
            .json::<JsonRpcResponse<ListToolsResponse>>()
            .await?;

        if let Some(error) = response.error {
            return Err(anyhow!("Error del servidor: {}", error.message));
        }

        Ok(response.result.map(|r| r.tools).unwrap_or_default())
    }

    /// Llama a una herramienta específica en el servidor MCP
    pub async fn call_tool(&self, tool: &str, args: Value) -> Result<Value> {
        let url = format!("{}/mcp/call_tool", self.base_url);

        let params = CallToolParams {
            tool: tool.to_string(),
            args,
        };

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "call_tool".to_string(),
            params: Some(params),
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await?
            .json::<JsonRpcResponse<Value>>()
            .await?;

        if let Some(error) = response.error {
            return Err(anyhow!("Error del servidor: {}", error.message));
        }

        response
            .result
            .ok_or_else(|| anyhow!("No se recibió resultado del servidor"))
    }
}

/// Gestiona múltiples conexiones a servidores MCP externos
pub struct MCPClientManager {
    clients: Vec<(String, MCPClient)>,
}

impl MCPClientManager {
    pub fn new() -> Self {
        Self {
            clients: Vec::new(),
        }
    }

    /// Agrega un nuevo servidor MCP externo
    pub fn add_server(&mut self, name: String, url: String) {
        let client = MCPClient::new(url);
        self.clients.push((name, client));
    }

    /// Elimina un servidor MCP externo
    pub fn remove_server(&mut self, name: &str) {
        self.clients.retain(|(n, _)| n != name);
    }

    /// Lista todos los servidores configurados
    pub fn list_servers(&self) -> Vec<&str> {
        self.clients.iter().map(|(name, _)| name.as_str()).collect()
    }

    /// Obtiene un cliente específico por nombre
    pub fn get_client(&self, name: &str) -> Option<&MCPClient> {
        self.clients
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, client)| client)
    }

    /// Descubre todas las herramientas de todos los servidores
    pub async fn discover_all_tools(&self) -> Result<Vec<(String, Vec<Value>)>> {
        let mut all_tools = Vec::new();

        for (name, client) in &self.clients {
            match client.list_tools().await {
                Ok(tools) => {
                    all_tools.push((name.clone(), tools));
                }
                Err(e) => {
                    eprintln!("⚠️  Error descubriendo tools de '{}': {}", name, e);
                }
            }
        }

        Ok(all_tools)
    }
}
