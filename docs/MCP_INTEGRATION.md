# Integración MCP (Model Context Protocol)

NotNative implementa un sistema híbrido MCP que permite:
1. **MCP Server**: Exponer las funcionalidades de NotNative para que aplicaciones externas puedan interactuar
2. **MCP Client**: Conectar a servidores MCP externos para extender las capacidades

## Arquitectura

```
┌─────────────────────────────────────────────────────────────┐
│                      NotNative App                          │
│                                                             │
│  ┌───────────┐        ┌────────────┐                       │
│  │ MCP Server│◄───────┤ Tool       │                       │
│  │ :8788     │        │ Executor   │                       │
│  └─────┬─────┘        └────────────┘                       │
│        │                                                    │
│        │              ┌────────────┐                       │
│        │              │ MCP Client │                       │
│        │              │ Manager    │                       │
│        │              └──────┬─────┘                       │
│        │                     │                             │
└────────┼─────────────────────┼─────────────────────────────┘
         │                     │
         │                     │
    HTTP │                     │ HTTP
  (inbound)                    │ (outbound)
         │                     │
         ▼                     ▼
    ┌─────────┐           ┌─────────┐
    │ n8n     │           │ External│
    │ Workflow│           │ MCP     │
    │         │           │ Server  │
    └─────────┘           └─────────┘
```

## MCP Server - API REST

El servidor MCP de NotNative escucha en `http://localhost:8788` y expone tres endpoints:

### GET /health

Health check del servidor.

**Ejemplo con curl:**
```bash
curl http://localhost:8788/health
```

**Respuesta:**
```json
{
  "status": "ok",
  "service": "NotNative MCP Server",
  "version": "1.0.0"
}
```

### POST /mcp/list_tools

Lista todas las herramientas disponibles.

**Request (JSON-RPC 2.0):**
```bash
curl -X POST http://localhost:8788/mcp/list_tools \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "list_tools"
  }'
```

**Respuesta:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "tools": [
      {
        "name": "create_note",
        "description": "USA SOLO cuando el usuario diga: 'crea', 'crear una nota'...",
        "parameters": {
          "type": "object",
          "properties": {
            "name": {"type": "string"},
            "content": {"type": "string"},
            "folder": {"type": "string"}
          },
          "required": ["name", "content"]
        }
      },
      ...
    ]
  }
}
```

### POST /mcp/call_tool

Ejecuta una herramienta específica.

**Ejemplo - Crear nota:**
```bash
curl -X POST http://localhost:8788/mcp/call_tool \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "call_tool",
    "params": {
      "tool": "create_note",
      "args": {
        "name": "Mi Nota desde API",
        "content": "# Título\n\nContenido de la nota creada externamente"
      }
    }
  }'
```

**Respuesta exitosa:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "success": true,
    "data": {
      "note_name": "Mi Nota desde API",
      "message": "✓ Nota 'Mi Nota desde API' creada exitosamente"
    }
  }
}
```

**Respuesta con error:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32602,
    "message": "Parámetros inválidos: missing field `name`"
  }
}
```

## Ejemplos de Integración

### 1. n8n Workflow

Crea un workflow en n8n para sincronizar emails importantes a NotNative:

```json
{
  "nodes": [
    {
      "name": "Gmail Trigger",
      "type": "n8n-nodes-base.gmailTrigger",
      "parameters": {
        "labelIds": ["IMPORTANT"]
      }
    },
    {
      "name": "Create Note",
      "type": "n8n-nodes-base.httpRequest",
      "parameters": {
        "url": "http://localhost:8788/mcp/call_tool",
        "method": "POST",
        "bodyParameters": {
          "parameters": [
            {
              "name": "jsonrpc",
              "value": "2.0"
            },
            {
              "name": "id",
              "value": 1
            },
            {
              "name": "method",
              "value": "call_tool"
            },
            {
              "name": "params",
              "value": {
                "tool": "create_note",
                "args": {
                  "name": "={{$node[\"Gmail Trigger\"].json[\"subject\"]}}",
                  "content": "# {{$node[\"Gmail Trigger\"].json[\"subject\"]}}\n\n{{$node[\"Gmail Trigger\"].json[\"textPlain\"]}}\n\n---\nFrom: {{$node[\"Gmail Trigger\"].json[\"from\"]}}\nDate: {{$node[\"Gmail Trigger\"].json[\"date\"]}}"
                }
              }
            }
          ]
        }
      }
    }
  ]
}
```

### 2. Script Python - Backup de GitHub Issues

```python
#!/usr/bin/env python3
import requests
import json

# Configuración
NOTNATIVE_API = "http://localhost:8788/mcp/call_tool"
GITHUB_TOKEN = "tu_token_aqui"
GITHUB_REPO = "usuario/repo"

def create_note(name, content):
    """Crea una nota en NotNative"""
    payload = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "call_tool",
        "params": {
            "tool": "create_note",
            "args": {
                "name": name,
                "content": content
            }
        }
    }
    
    response = requests.post(NOTNATIVE_API, json=payload)
    return response.json()

def fetch_github_issues():
    """Obtiene issues de GitHub"""
    headers = {"Authorization": f"token {GITHUB_TOKEN}"}
    url = f"https://api.github.com/repos/{GITHUB_REPO}/issues"
    
    response = requests.get(url, headers=headers)
    return response.json()

def sync_issues():
    """Sincroniza issues de GitHub a NotNative"""
    issues = fetch_github_issues()
    
    for issue in issues:
        note_name = f"GitHub Issue #{issue['number']} - {issue['title']}"
        note_content = f"""# {issue['title']}

**Status**: {issue['state']}
**Created**: {issue['created_at']}
**URL**: {issue['html_url']}

## Description

{issue['body']}

---
Labels: {', '.join([label['name'] for label in issue['labels']])}
"""
        
        result = create_note(note_name, note_content)
        
        if result.get('result', {}).get('success'):
            print(f"✓ Sincronizado: {note_name}")
        else:
            print(f"✗ Error: {result.get('error', {}).get('message')}")

if __name__ == "__main__":
    sync_issues()
```

### 3. JavaScript/Node.js - Obsidian Sync

```javascript
const axios = require('axios');
const fs = require('fs').promises;
const path = require('path');

const NOTNATIVE_API = 'http://localhost:8788/mcp/call_tool';
const OBSIDIAN_VAULT = '/ruta/a/tu/vault';

async function createNote(name, content) {
  try {
    const response = await axios.post(NOTNATIVE_API, {
      jsonrpc: '2.0',
      id: 1,
      method: 'call_tool',
      params: {
        tool: 'create_note',
        args: { name, content }
      }
    });
    
    return response.data.result;
  } catch (error) {
    console.error('Error creando nota:', error.response?.data || error.message);
    return null;
  }
}

async function syncFromObsidian() {
  const files = await fs.readdir(OBSIDIAN_VAULT);
  
  for (const file of files) {
    if (!file.endsWith('.md')) continue;
    
    const filePath = path.join(OBSIDIAN_VAULT, file);
    const content = await fs.readFile(filePath, 'utf-8');
    const name = file.replace('.md', '');
    
    const result = await createNote(name, content);
    
    if (result?.success) {
      console.log(`✓ Sincronizado: ${name}`);
    }
  }
}

syncFromObsidian();
```

### 4. Bash Script - Daily Backup

```bash
#!/bin/bash

NOTNATIVE_API="http://localhost:8788/mcp/call_tool"
BACKUP_DIR="/backup/notes"

# Función para crear nota
create_note() {
    local name="$1"
    local content="$2"
    
    curl -s -X POST "$NOTNATIVE_API" \
        -H "Content-Type: application/json" \
        -d "{
            \"jsonrpc\": \"2.0\",
            \"id\": 1,
            \"method\": \"call_tool\",
            \"params\": {
                \"tool\": \"create_note\",
                \"args\": {
                    \"name\": \"$name\",
                    \"content\": \"$content\"
                }
            }
        }"
}

# Listar todas las notas
list_notes() {
    curl -s -X POST "$NOTNATIVE_API" \
        -H "Content-Type: application/json" \
        -d '{
            "jsonrpc": "2.0",
            "id": 1,
            "method": "call_tool",
            "params": {
                "tool": "list_notes",
                "args": {}
            }
        }'
}

# Crear backup diario
DATE=$(date +%Y-%m-%d)
notes=$(list_notes | jq -r '.result.data.notes[]')

mkdir -p "$BACKUP_DIR/$DATE"

for note in $notes; do
    echo "Respaldando: $note"
    # Aquí podrías usar read_note para obtener el contenido
done
```

## MCP Client - Conectar a Servidores Externos

NotNative puede actuar como cliente y conectar a servidores MCP externos:

```rust
use notnative::mcp::{MCPClient, MCPClientManager};

// Crear manager de clientes
let mut manager = MCPClientManager::new();

// Agregar servidor externo
manager.add_server(
    "notion".to_string(),
    "http://localhost:9000".to_string()
);

// Descubrir herramientas
let tools = manager.discover_all_tools().await?;

// Llamar a herramienta externa
if let Some(client) = manager.get_client("notion") {
    let result = client.call_tool(
        "create_page",
        serde_json::json!({
            "title": "Nueva página",
            "content": "Creada desde NotNative"
        })
    ).await?;
}
```

## Casos de Uso Reales

### 1. Automatización de Reuniones

**Problema**: Quieres que las transcripciones de Zoom se conviertan automáticamente en notas.

**Solución**:
- Zoom webhook → n8n → NotNative MCP
- n8n recibe la transcripción
- Llama a `create_note` con la transcripción formateada

### 2. Investigación Web

**Problema**: Guardar artículos interesantes mientras navegas.

**Solución**:
- Extensión de navegador → NotNative MCP
- Botón "Guardar en NotNative" en el navegador
- Llama a `create_note` con el contenido del artículo

### 3. Sync Multi-plataforma

**Problema**: Trabajas en Notion pero quieres backup local.

**Solución**:
- Script Python periódico
- Lee de Notion API
- Llama a `create_note` para cada página

### 4. Integración con IDEs

**Problema**: Documentar código directamente desde VS Code.

**Solución**:
- Extensión VS Code → NotNative MCP
- Comando "Crear nota técnica"
- Extrae comentarios y estructura del código

## Seguridad

⚠️ **IMPORTANTE**: El servidor MCP actualmente escucha solo en `localhost:8788` para evitar exposición externa.

Para producción, considera:
1. Agregar autenticación (API keys)
2. HTTPS/TLS
3. Rate limiting
4. Validación estricta de inputs

## Troubleshooting

### El servidor no inicia
```bash
# Verificar que el puerto 8788 no esté ocupado
lsof -i :8788

# Ver logs de NotNative
journalctl -f | grep notnative
```

### Error de conexión desde cliente externo
```bash
# Verificar que el servidor esté corriendo
curl http://localhost:8788/health

# Debería responder:
# {"status":"ok","service":"NotNative MCP Server","version":"1.0.0"}
```

### Error "Parámetros inválidos"
- Verificar que el JSON tenga la estructura correcta
- Revisar los tipos de datos (string, number, etc.)
- Usar el endpoint `/mcp/list_tools` para ver la estructura esperada

## Contribuir

¿Tienes ideas para nuevas integraciones? ¡Abre un issue en GitHub!

- Proponer nuevas herramientas MCP
- Compartir workflows de n8n
- Crear wrappers para otros lenguajes
- Mejorar la documentación

## Licencia

Ver LICENSE en el repositorio principal.
