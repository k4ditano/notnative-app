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

## Herramientas Disponibles

El MCP Server de NotNative expone las siguientes herramientas:

### 1. create_note
Crea una nueva nota en el workspace.

**Parámetros:**
- `name` (string, requerido): Nombre de la nota
- `content` (string, requerido): Contenido en formato Markdown
- `folder` (string, opcional): Carpeta donde crear la nota (ej: "Proyectos/Web")

**Ejemplo:**
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
        "name": "Mi Nota",
        "content": "# Título\n\nContenido...",
        "folder": "Proyectos"
      }
    }
  }'
```

### 2. read_note
Lee el contenido de una nota existente.

**Parámetros:**
- `name` (string, requerido): Nombre de la nota a leer

**Ejemplo:**
```bash
curl -X POST http://localhost:8788/mcp/call_tool \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "call_tool",
    "params": {
      "tool": "read_note",
      "args": {
        "name": "Mi Nota"
      }
    }
  }'
```

**Respuesta:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "success": true,
    "data": {
      "name": "Mi Nota",
      "content": "# Título\n\nContenido...",
      "path": "/home/user/.local/share/notnative/notes/Mi Nota.md"
    }
  }
}
```

### 3. update_note
Actualiza el contenido de una nota existente.

**Parámetros:**
- `name` (string, requerido): Nombre de la nota
- `content` (string, requerido): Nuevo contenido

**Ejemplo:**
```bash
curl -X POST http://localhost:8788/mcp/call_tool \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "call_tool",
    "params": {
      "tool": "update_note",
      "args": {
        "name": "Mi Nota",
        "content": "# Título Actualizado\n\nNuevo contenido..."
      }
    }
  }'
```

### 4. delete_note
Elimina una nota del workspace.

**Parámetros:**
- `name` (string, requerido): Nombre de la nota a eliminar

**Ejemplo:**
```bash
curl -X POST http://localhost:8788/mcp/call_tool \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "call_tool",
    "params": {
      "tool": "delete_note",
      "args": {
        "name": "Nota a Eliminar"
      }
    }
  }'
```

### 5. list_notes
Lista todas las notas disponibles en el workspace.

**Parámetros:**
- `folder` (string, opcional): Filtrar por carpeta específica

**Ejemplo:**
```bash
curl -X POST http://localhost:8788/mcp/call_tool \
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
```

**Respuesta:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "success": true,
    "data": {
      "notes": [
        "Mi Nota",
        "Proyectos/Web/Frontend",
        "Ideas/Producto"
      ],
      "count": 3
    }
  }
}
```

### 6. search_notes
Busca notas por contenido o título.

**Parámetros:**
- `query` (string, requerido): Término de búsqueda
- `case_sensitive` (boolean, opcional): Búsqueda sensible a mayúsculas (default: false)

**Ejemplo:**
```bash
curl -X POST http://localhost:8788/mcp/call_tool \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "call_tool",
    "params": {
      "tool": "search_notes",
      "args": {
        "query": "python",
        "case_sensitive": false
      }
    }
  }'
```

**Respuesta:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "success": true,
    "data": {
      "results": [
        {
          "note": "Tutorial Python",
          "matches": 5,
          "preview": "...aprender Python desde cero..."
        }
      ],
      "total": 1
    }
  }
}
```

### 7. add_tags
Agrega tags a una nota (modifica el frontmatter YAML).

**Parámetros:**
- `name` (string, requerido): Nombre de la nota
- `tags` (array<string>, requerido): Tags a agregar

**Ejemplo:**
```bash
curl -X POST http://localhost:8788/mcp/call_tool \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "call_tool",
    "params": {
      "tool": "add_tags",
      "args": {
        "name": "Mi Nota",
        "tags": ["importante", "trabajo", "2025"]
      }
    }
  }'
```

### 8. append_to_note
Añade contenido al final de una nota existente.

**Parámetros:**
- `name` (string, requerido): Nombre de la nota
- `content` (string, requerido): Contenido a añadir

**Ejemplo:**
```bash
curl -X POST http://localhost:8788/mcp/call_tool \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "call_tool",
    "params": {
      "tool": "append_to_note",
      "args": {
        "name": "Daily Log",
        "content": "\n\n## 2025-11-07\n- Reunión con equipo\n- Código revisado"
      }
    }
  }'
```

## Acceso Remoto mediante Túneles

Para acceder al MCP Server desde fuera de tu red local, puedes usar diferentes soluciones de túnel:

### Opción 1: Cloudflare Tunnel (Recomendado)

**Ventajas:**
- ✅ Gratis
- ✅ HTTPS automático
- ✅ No requiere abrir puertos
- ✅ Protección DDoS incluida

**Instalación:**
```bash
# Instalar cloudflared
yay -S cloudflared

# Autenticar
cloudflared tunnel login

# Crear túnel
cloudflared tunnel create notnative

# Configurar túnel
cat > ~/.cloudflared/config.yml << EOF
tunnel: notnative
credentials-file: /home/tu-usuario/.cloudflared/<TUNNEL-ID>.json

ingress:
  - hostname: notnative.tu-dominio.com
    service: http://localhost:8788
  - service: http_status:404
EOF

# Ejecutar túnel
cloudflared tunnel run notnative
```

**Uso:**
```bash
# Ahora puedes acceder desde cualquier lugar:
curl https://notnative.tu-dominio.com/health

# Crear nota remotamente
curl -X POST https://notnative.tu-dominio.com/mcp/call_tool \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "call_tool",
    "params": {
      "tool": "create_note",
      "args": {
        "name": "Nota Remota",
        "content": "Creada desde mi teléfono!"
      }
    }
  }'
```

### Opción 2: Tailscale (Red Privada)

**Ventajas:**
- ✅ Red privada segura
- ✅ Zero-config
- ✅ Funciona detrás de NAT/firewalls
- ✅ No expone públicamente

**Instalación:**
```bash
# Instalar Tailscale
yay -S tailscale

# Iniciar servicio
sudo systemctl enable --now tailscaled

# Conectar
sudo tailscale up
```

**Uso:**
```bash
# Obtener IP de Tailscale
tailscale ip -4

# Acceder desde cualquier dispositivo en tu red Tailscale:
curl http://100.x.x.x:8788/health

# Desde tu teléfono (con Tailscale instalado):
curl http://100.x.x.x:8788/mcp/call_tool \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"call_tool","params":{"tool":"create_note","args":{"name":"Nota Móvil","content":"Desde el móvil"}}}'
```

### Opción 3: ngrok (Desarrollo/Testing)

**Ventajas:**
- ✅ Setup instantáneo
- ✅ Útil para demos/testing
- ⚠️ URL cambia en cada sesión (plan free)

**Instalación:**
```bash
# Instalar ngrok
yay -S ngrok

# Crear túnel
ngrok http 8788
```

**Salida:**
```
Forwarding   https://abc123.ngrok.io -> http://localhost:8788
```

**Uso:**
```bash
# Acceder desde cualquier lugar:
curl https://abc123.ngrok.io/health

# Crear nota
curl -X POST https://abc123.ngrok.io/mcp/call_tool \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "call_tool",
    "params": {
      "tool": "create_note",
      "args": {
        "name": "Test ngrok",
        "content": "Funcionando!"
      }
    }
  }'
```

### Opción 4: SSH Tunnel (Método Clásico)

**Ventajas:**
- ✅ Seguro (requiere SSH)
- ✅ No requiere software adicional
- ⚠️ Más complejo de configurar

**En el servidor (tu PC con NotNative):**
```bash
# Asegurarse de que SSH esté habilitado
sudo systemctl enable --now sshd
```

**En el cliente (laptop, teléfono con Termux, etc.):**
```bash
# Crear túnel SSH
ssh -L 8788:localhost:8788 usuario@tu-ip-publica

# En otra terminal, usar el API localmente:
curl http://localhost:8788/health
```

### Opción 5: WireGuard VPN (Máxima Seguridad)

**Ventajas:**
- ✅ Máxima seguridad
- ✅ Rendimiento excelente
- ✅ Control total
- ⚠️ Requiere configuración

**Configuración básica:**
```bash
# Instalar WireGuard
yay -S wireguard-tools

# Generar claves
wg genkey | tee privatekey | wg pubkey > publickey

# Configurar interfaz
sudo nano /etc/wireguard/wg0.conf
```

**Archivo de configuración:**
```ini
[Interface]
Address = 10.0.0.1/24
ListenPort = 51820
PrivateKey = <tu-clave-privada>

[Peer]
PublicKey = <clave-publica-del-cliente>
AllowedIPs = 10.0.0.2/32
```

**Iniciar VPN:**
```bash
sudo wg-quick up wg0
sudo systemctl enable wg-quick@wg0
```

**Uso:**
```bash
# Desde el cliente conectado a la VPN:
curl http://10.0.0.1:8788/health
```

## Seguridad y Mejores Prácticas

### 🔒 Recomendaciones Generales

⚠️ **IMPORTANTE**: El servidor MCP actualmente escucha solo en `localhost:8788` para evitar exposición externa.

Para producción/acceso remoto, **DEBES** implementar:

1. **Autenticación con API Key**
   ```bash
   # Agregar header X-API-Key en todas las peticiones
   curl -X POST https://notnative.example.com/mcp/call_tool \
     -H "Content-Type: application/json" \
     -H "X-API-Key: tu-api-key-secreta-aqui" \
     -d '...'
   ```

2. **HTTPS/TLS Obligatorio**
   - Usar Cloudflare Tunnel (HTTPS automático)
   - O configurar certificados Let's Encrypt
   - **NUNCA** exponer HTTP sin cifrar públicamente

3. **Rate Limiting**
   - Limitar peticiones por IP/API key
   - Prevenir ataques de fuerza bruta
   - Ejemplo: máximo 100 requests/minuto

4. **Whitelist de IPs (opcional)**
   ```bash
   # Solo permitir acceso desde IPs conocidas
   # Configurar en Cloudflare Access o firewall
   ```

5. **Validación Estricta**
   - El servidor ya valida inputs
   - Revisar logs regularmente
   - Monitorear peticiones sospechosas

### 🛡️ Configuración de Cloudflare Access

Para proteger tu túnel con autenticación:

```yaml
# ~/.cloudflared/config.yml
tunnel: notnative
credentials-file: /home/usuario/.cloudflared/<ID>.json

ingress:
  - hostname: notnative.example.com
    service: http://localhost:8788
    originRequest:
      # Agregar headers de seguridad
      httpHostHeader: notnative.example.com
  - service: http_status:404
```

**Configurar en Cloudflare Dashboard:**
1. Access > Applications > Add application
2. Application type: Self-hosted
3. Subdomain: notnative
4. Policies: Email verification, Google OAuth, etc.

### 📊 Monitoreo y Logs

```bash
# Ver requests en tiempo real
journalctl -f | grep notnative

# Analizar logs
tail -f ~/.local/share/notnative/mcp-server.log

# Alertas (ejemplo con systemd)
sudo journalctl -u notnative -f --grep "ERROR"
```

### 🚨 En Caso de Compromiso

Si sospechas que tu API key fue comprometida:

1. **Rotar la clave inmediatamente**
2. **Revisar logs de acceso**
3. **Verificar notas no autorizadas**
4. **Cambiar URLs de túnel**

Para producción, considera:
1. Agregar autenticación (API keys)
2. HTTPS/TLS
3. Rate limiting
4. Validación estricta de inputs

## Ejemplos Prácticos de Automatización

### 📱 1. Captura de Ideas desde el Móvil (Shortcuts iOS)

Crea un Shortcut en iOS para guardar ideas rápidamente:

**Configuración del Shortcut:**
1. Acción: "Ask for Input"
   - Prompt: "¿Qué idea quieres guardar?"
   
2. Acción: "Get Contents of URL"
   - URL: `https://notnative.tu-dominio.com/mcp/call_tool`
   - Method: POST
   - Headers:
     - Content-Type: `application/json`
     - X-API-Key: `tu-api-key`
   - Request Body (JSON):
   ```json
   {
     "jsonrpc": "2.0",
     "id": 1,
     "method": "call_tool",
     "params": {
       "tool": "append_to_note",
       "args": {
         "name": "Ideas Rápidas",
         "content": "\n- [Shortcut Input]\n  *Capturado: [Current Date]*"
       }
     }
   }
   ```

3. Acción: "Show Notification"
   - Message: "✅ Idea guardada en NotNative"

**Uso:**
- Activar Siri: "Hey Siri, captura idea"
- Dictar la idea
- Se guarda automáticamente en "Ideas Rápidas"

### 🤖 2. Bot de Telegram para Notas

Script Python para crear un bot de Telegram que guarde mensajes en NotNative:

```python
#!/usr/bin/env python3
import os
import requests
from telegram import Update
from telegram.ext import Application, CommandHandler, MessageHandler, filters

NOTNATIVE_API = os.getenv("NOTNATIVE_API", "http://localhost:8788/mcp/call_tool")
TELEGRAM_TOKEN = os.getenv("TELEGRAM_BOT_TOKEN")

def call_notnative(tool, args):
    """Llama a una herramienta del MCP Server"""
    payload = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "call_tool",
        "params": {
            "tool": tool,
            "args": args
        }
    }
    response = requests.post(NOTNATIVE_API, json=payload)
    return response.json()

async def start(update: Update, context):
    """Handler para /start"""
    await update.message.reply_text(
        "¡Hola! Envíame cualquier mensaje y lo guardaré en NotNative.\n\n"
        "Comandos:\n"
        "/save <texto> - Guardar nota\n"
        "/search <query> - Buscar notas\n"
        "/list - Listar notas"
    )

async def save_message(update: Update, context):
    """Guarda el mensaje en una nota"""
    text = update.message.text
    user = update.effective_user.first_name
    
    # Guardar en nota "Telegram Log"
    result = call_notnative("append_to_note", {
        "name": "Telegram Log",
        "content": f"\n\n## {user} - {update.message.date}\n{text}"
    })
    
    if result.get("result", {}).get("success"):
        await update.message.reply_text("✅ Guardado en NotNative")
    else:
        await update.message.reply_text("❌ Error al guardar")

async def search_notes(update: Update, context):
    """Busca en las notas"""
    query = " ".join(context.args)
    
    result = call_notnative("search_notes", {
        "query": query
    })
    
    if result.get("result", {}).get("success"):
        results = result["result"]["data"]["results"]
        if results:
            response = "🔍 Resultados:\n\n"
            for r in results[:5]:  # Máximo 5 resultados
                response += f"📝 {r['note']}\n{r['preview']}\n\n"
            await update.message.reply_text(response)
        else:
            await update.message.reply_text("No se encontraron resultados")
    else:
        await update.message.reply_text("❌ Error en la búsqueda")

async def list_notes(update: Update, context):
    """Lista todas las notas"""
    result = call_notnative("list_notes", {})
    
    if result.get("result", {}).get("success"):
        notes = result["result"]["data"]["notes"]
        response = f"📚 Tienes {len(notes)} notas:\n\n"
        response += "\n".join([f"• {note}" for note in notes[:20]])
        if len(notes) > 20:
            response += f"\n\n...y {len(notes) - 20} más"
        await update.message.reply_text(response)
    else:
        await update.message.reply_text("❌ Error al listar notas")

def main():
    """Inicia el bot"""
    app = Application.builder().token(TELEGRAM_TOKEN).build()
    
    app.add_handler(CommandHandler("start", start))
    app.add_handler(CommandHandler("search", search_notes))
    app.add_handler(CommandHandler("list", list_notes))
    app.add_handler(MessageHandler(filters.TEXT & ~filters.COMMAND, save_message))
    
    print("🤖 Bot de Telegram iniciado")
    app.run_polling()

if __name__ == "__main__":
    main()
```

**Configuración:**
```bash
# Instalar dependencias
pip install python-telegram-bot requests

# Configurar variables de entorno
export TELEGRAM_BOT_TOKEN="tu-token-de-telegram"
export NOTNATIVE_API="https://notnative.tu-dominio.com/mcp/call_tool"

# Ejecutar bot
python telegram_notnative_bot.py
```

### 📧 3. Guardar Emails Importantes (Gmail + Apps Script)

Script de Google Apps Script para guardar emails etiquetados:

```javascript
/**
 * Sincroniza emails de Gmail a NotNative
 * Ejecutar: Disparadores > Agregar > Ejecutar cada hora
 */

const NOTNATIVE_API = 'https://notnative.tu-dominio.com/mcp/call_tool';
const API_KEY = 'tu-api-key';
const LABEL_NAME = 'NotNative'; // Crear esta etiqueta en Gmail

function callNotNative(tool, args) {
  const payload = {
    jsonrpc: '2.0',
    id: 1,
    method: 'call_tool',
    params: {
      tool: tool,
      args: args
    }
  };
  
  const options = {
    method: 'post',
    contentType: 'application/json',
    headers: {
      'X-API-Key': API_KEY
    },
    payload: JSON.stringify(payload)
  };
  
  const response = UrlFetchApp.fetch(NOTNATIVE_API, options);
  return JSON.parse(response.getContentText());
}

function syncGmailToNotNative() {
  const label = GmailApp.getUserLabelByName(LABEL_NAME);
  
  if (!label) {
    Logger.log('Etiqueta "' + LABEL_NAME + '" no encontrada');
    return;
  }
  
  const threads = label.getThreads(0, 10); // Últimos 10 hilos
  
  threads.forEach(thread => {
    const messages = thread.getMessages();
    
    messages.forEach(message => {
      if (message.isUnread()) {
        const subject = message.getSubject();
        const from = message.getFrom();
        const date = message.getDate();
        const body = message.getPlainBody();
        
        const noteContent = `# ${subject}\n\n` +
                          `**De:** ${from}\n` +
                          `**Fecha:** ${date}\n\n` +
                          `---\n\n${body}`;
        
        const result = callNotNative('create_note', {
          name: `Email - ${subject}`,
          content: noteContent,
          folder: 'Emails'
        });
        
        if (result.result && result.result.success) {
          message.markRead();
          Logger.log('✅ Email guardado: ' + subject);
        } else {
          Logger.log('❌ Error: ' + subject);
        }
      }
    });
  });
}
```

### 🔔 4. Recordatorios Automáticos (cron + NotNative)

Script bash para crear recordatorios diarios:

```bash
#!/bin/bash
# Guardar como: ~/bin/daily-reminder.sh
# Agregar a crontab: 0 9 * * * ~/bin/daily-reminder.sh

NOTNATIVE_API="http://localhost:8788/mcp/call_tool"
DATE=$(date +%Y-%m-%d)
WEEKDAY=$(date +%A)

# Contenido del recordatorio
REMINDER=$(cat << EOF
# Daily Reminder - $DATE ($WEEKDAY)

## ✅ Tareas del Día
- [ ] Revisar emails
- [ ] Standup meeting
- [ ] Código review
- [ ] Actualizar documentación

## 📅 Recordatorios
- Deadline proyecto X: 2025-11-15
- Reunión importante: Viernes 10:00

## 💡 Objetivos de la Semana
- Completar feature Y
- Escribir tests
- Deploy a staging

---
*Generado automáticamente*
EOF
)

# Crear nota de recordatorio
curl -s -X POST "$NOTNATIVE_API" \
  -H "Content-Type: application/json" \
  -d "{
    \"jsonrpc\": \"2.0\",
    \"id\": 1,
    \"method\": \"call_tool\",
    \"params\": {
      \"tool\": \"create_note\",
      \"args\": {
        \"name\": \"Daily - $DATE\",
        \"content\": $(echo "$REMINDER" | jq -Rs .),
        \"folder\": \"Daily Logs\"
      }
    }
  }" | jq -r '.result.data.message'

# Notificación de escritorio
notify-send "NotNative" "Recordatorio diario creado para $DATE"
```

**Configurar cron:**
```bash
# Editar crontab
crontab -e

# Agregar línea (ejecutar cada día a las 9:00 AM)
0 9 * * * /home/usuario/bin/daily-reminder.sh
```

### 🌐 5. Sincronización con Notion (Bidireccional)

Script Python para sincronizar entre Notion y NotNative:

```python
#!/usr/bin/env python3
"""
Sincronización bidireccional Notion <-> NotNative
Ejecutar periódicamente con cron
"""
import os
import requests
from datetime import datetime
from notion_client import Client

NOTION_TOKEN = os.getenv("NOTION_TOKEN")
NOTION_DATABASE_ID = os.getenv("NOTION_DATABASE_ID")
NOTNATIVE_API = os.getenv("NOTNATIVE_API", "http://localhost:8788/mcp/call_tool")

notion = Client(auth=NOTION_TOKEN)

def get_notion_pages():
    """Obtiene páginas de Notion"""
    response = notion.databases.query(database_id=NOTION_DATABASE_ID)
    return response["results"]

def get_notnative_notes():
    """Obtiene notas de NotNative"""
    payload = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "call_tool",
        "params": {
            "tool": "list_notes",
            "args": {}
        }
    }
    response = requests.post(NOTNATIVE_API, json=payload)
    return response.json()["result"]["data"]["notes"]

def sync_notion_to_notnative():
    """Sincroniza de Notion a NotNative"""
    pages = get_notion_pages()
    
    for page in pages:
        title = page["properties"]["Name"]["title"][0]["plain_text"]
        page_id = page["id"]
        
        # Obtener contenido de la página
        blocks = notion.blocks.children.list(block_id=page_id)
        content = "\n".join([
            block["paragraph"]["rich_text"][0]["plain_text"]
            for block in blocks["results"]
            if block["type"] == "paragraph" and block["paragraph"]["rich_text"]
        ])
        
        # Crear/actualizar en NotNative
        payload = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "call_tool",
            "params": {
                "tool": "create_note",
                "args": {
                    "name": f"Notion - {title}",
                    "content": f"# {title}\n\n{content}\n\n---\n*Sincronizado desde Notion*"
                }
            }
        }
        
        response = requests.post(NOTNATIVE_API, json=payload)
        if response.json().get("result", {}).get("success"):
            print(f"✅ Sincronizado: {title}")
        else:
            print(f"❌ Error: {title}")

def sync_notnative_to_notion():
    """Sincroniza de NotNative a Notion"""
    notes = get_notnative_notes()
    
    for note in notes:
        # Leer nota de NotNative
        payload = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "call_tool",
            "params": {
                "tool": "read_note",
                "args": {"name": note}
            }
        }
        
        response = requests.post(NOTNATIVE_API, json=payload)
        content = response.json()["result"]["data"]["content"]
        
        # Crear página en Notion
        notion.pages.create(
            parent={"database_id": NOTION_DATABASE_ID},
            properties={
                "Name": {"title": [{"text": {"content": note}}]}
            },
            children=[
                {
                    "object": "block",
                    "type": "paragraph",
                    "paragraph": {
                        "rich_text": [{"type": "text", "text": {"content": content}}]
                    }
                }
            ]
        )
        print(f"✅ Creado en Notion: {note}")

if __name__ == "__main__":
    print("🔄 Iniciando sincronización...")
    sync_notion_to_notnative()
    sync_notnative_to_notion()
    print("✅ Sincronización completada")
```

### 📸 6. Captura de Screenshots con Contexto

Script para capturar screenshots y crear notas automáticamente:

```bash
#!/bin/bash
# Captura screenshot y crea nota con contexto
# Guardar como: ~/bin/screenshot-to-note.sh
# Bind a tecla: Super+Shift+S

NOTNATIVE_API="http://localhost:8788/mcp/call_tool"
SCREENSHOT_DIR="$HOME/Pictures/Screenshots"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
FILENAME="screenshot_$TIMESTAMP.png"

# Crear directorio si no existe
mkdir -p "$SCREENSHOT_DIR"

# Capturar screenshot (usando grim en Wayland)
grim -g "$(slurp)" "$SCREENSHOT_DIR/$FILENAME"

# Obtener información del contexto
WINDOW_TITLE=$(hyprctl activewindow | grep "title:" | cut -d: -f2 | xargs)
APP_NAME=$(hyprctl activewindow | grep "class:" | cut -d: -f2 | xargs)

# Crear nota con contexto
NOTE_CONTENT="# Screenshot - $TIMESTAMP

![Screenshot]($SCREENSHOT_DIR/$FILENAME)

## Contexto
- **Aplicación**: $APP_NAME
- **Ventana**: $WINDOW_TITLE
- **Fecha**: $(date '+%Y-%m-%d %H:%M:%S')

## Descripción
<!-- Agregar descripción aquí -->

---
*Capturado automáticamente*
"

# Guardar en NotNative
curl -s -X POST "$NOTNATIVE_API" \
  -H "Content-Type: application/json" \
  -d "{
    \"jsonrpc\": \"2.0\",
    \"id\": 1,
    \"method\": \"call_tool\",
    \"params\": {
      \"tool\": \"create_note\",
      \"args\": {
        \"name\": \"Screenshot $TIMESTAMP\",
        \"content\": $(echo "$NOTE_CONTENT" | jq -Rs .),
        \"folder\": \"Screenshots\"
      }
    }
  }"

notify-send "Screenshot guardado" "Nota creada en NotNative"
```

**Configurar en Hyprland:**
```
# ~/.config/hyprland/hyprland.conf
bind = SUPER_SHIFT, S, exec, ~/bin/screenshot-to-note.sh
```

### 🎯 7. Integración con Habitica (Gamificación)

Sincroniza tareas completadas en NotNative con Habitica:

```python
#!/usr/bin/env python3
"""
Sincroniza tareas de NotNative con Habitica
Convierte checkboxes completados en XP de Habitica
"""
import os
import re
import requests

HABITICA_USER_ID = os.getenv("HABITICA_USER_ID")
HABITICA_API_TOKEN = os.getenv("HABITICA_API_TOKEN")
NOTNATIVE_API = os.getenv("NOTNATIVE_API", "http://localhost:8788/mcp/call_tool")

habitica_headers = {
    "x-api-user": HABITICA_USER_ID,
    "x-api-key": HABITICA_API_TOKEN,
    "Content-Type": "application/json"
}

def get_tasks_from_note(note_name):
    """Extrae tareas de una nota"""
    payload = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "call_tool",
        "params": {
            "tool": "read_note",
            "args": {"name": note_name}
        }
    }
    
    response = requests.post(NOTNATIVE_API, json=payload)
    content = response.json()["result"]["data"]["content"]
    
    # Extraer checkboxes completados
    completed = re.findall(r'- \[x\] (.+)', content, re.IGNORECASE)
    # Extraer checkboxes pendientes
    pending = re.findall(r'- \[ \] (.+)', content)
    
    return completed, pending

def complete_habitica_task(task_text):
    """Marca tarea como completada en Habitica"""
    # Buscar o crear tarea
    url = "https://habitica.com/api/v3/tasks/user"
    response = requests.get(url, headers=habitica_headers)
    tasks = response.json()["data"]
    
    task = next((t for t in tasks if t["text"] == task_text), None)
    
    if not task:
        # Crear nueva tarea
        response = requests.post(
            url,
            headers=habitica_headers,
            json={"text": task_text, "type": "todo"}
        )
        task = response.json()["data"]
    
    # Completar tarea
    complete_url = f"https://habitica.com/api/v3/tasks/{task['id']}/score/up"
    response = requests.post(complete_url, headers=habitica_headers)
    
    if response.status_code == 200:
        xp_gained = response.json()["data"]["_tmp"]["quest"]["progressDelta"]
        print(f"✅ {task_text} (+{xp_gained} XP)")

def sync_to_habitica(note_name="TODO"):
    """Sincroniza tareas completadas a Habitica"""
    completed, pending = get_tasks_from_note(note_name)
    
    print(f"📋 Encontradas {len(completed)} tareas completadas")
    
    for task in completed:
        complete_habitica_task(task)

if __name__ == "__main__":
    sync_to_habitica()
```

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
