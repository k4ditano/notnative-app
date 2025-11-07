# MCP Integration (Model Context Protocol)

NotNative implements a hybrid MCP system that enables:
1. **MCP Server**: Expose NotNative functionalities so external applications can interact
2. **MCP Client**: Connect to external MCP servers to extend capabilities

## Architecture

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

## MCP Server - REST API

The NotNative MCP server listens on `http://localhost:8788` and exposes three endpoints:

### GET /health

Server health check.

**Example with curl:**
```bash
curl http://localhost:8788/health
```

**Response:**
```json
{
  "status": "ok",
  "service": "NotNative MCP Server",
  "version": "1.0.0"
}
```

### POST /mcp/list_tools

Lists all available tools.

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

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "tools": [
      {
        "name": "create_note",
        "description": "Creates a new note in the workspace",
        "parameters": {
          "type": "object",
          "properties": {
            "name": {"type": "string"},
            "content": {"type": "string"},
            "folder": {"type": "string"}
          },
          "required": ["name", "content"]
        }
      }
    ]
  }
}
```

### POST /mcp/call_tool

Executes a specific tool.

**Example - Create note:**
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
        "name": "My Note from API",
        "content": "# Title\n\nContent of externally created note"
      }
    }
  }'
```

**Success response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "success": true,
    "data": {
      "note_name": "My Note from API",
      "message": "✓ Note 'My Note from API' created successfully"
    }
  }
}
```

**Error response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32602,
    "message": "Invalid parameters: missing field `name`"
  }
}
```

## Available Tools

The NotNative MCP Server exposes the following tools:

### 1. create_note
Creates a new note in the workspace.

**Parameters:**
- `name` (string, required): Note name
- `content` (string, required): Content in Markdown format
- `folder` (string, optional): Folder to create the note in (e.g., "Projects/Web")

### 2. read_note
Reads the content of an existing note.

**Parameters:**
- `name` (string, required): Name of the note to read

### 3. update_note
Updates the content of an existing note.

**Parameters:**
- `name` (string, required): Note name
- `content` (string, required): New content

### 4. delete_note
Deletes a note from the workspace.

**Parameters:**
- `name` (string, required): Name of the note to delete

### 5. list_notes
Lists all available notes in the workspace.

**Parameters:**
- `folder` (string, optional): Filter by specific folder

### 6. search_notes
Searches notes by content or title.

**Parameters:**
- `query` (string, required): Search term
- `case_sensitive` (boolean, optional): Case-sensitive search (default: false)

### 7. add_tags
Adds tags to a note (modifies YAML frontmatter).

**Parameters:**
- `name` (string, required): Note name
- `tags` (array<string>, required): Tags to add

### 8. append_to_note
Appends content to the end of an existing note.

**Parameters:**
- `name` (string, required): Note name
- `content` (string, required): Content to append

## Integration Examples

### 1. Python Script - GitHub Issues Backup

```python
#!/usr/bin/env python3
import requests
import json

NOTNATIVE_API = "http://localhost:8788/mcp/call_tool"
GITHUB_TOKEN = "your_token_here"
GITHUB_REPO = "username/repo"

def create_note(name, content):
    """Creates a note in NotNative"""
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
    """Fetches GitHub issues"""
    headers = {"Authorization": f"token {GITHUB_TOKEN}"}
    url = f"https://api.github.com/repos/{GITHUB_REPO}/issues"
    
    response = requests.get(url, headers=headers)
    return response.json()

def sync_issues():
    """Syncs GitHub issues to NotNative"""
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
            print(f"✓ Synced: {note_name}")
        else:
            print(f"✗ Error: {result.get('error', {}).get('message')}")

if __name__ == "__main__":
    sync_issues()
```

### 2. Bash Script - Daily Backup

```bash
#!/bin/bash

NOTNATIVE_API="http://localhost:8788/mcp/call_tool"

# Function to create note
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

# List all notes
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
```

## Remote Access via Tunnels

To access the MCP Server from outside your local network, you can use different tunnel solutions:

### Option 1: Cloudflare Tunnel (Recommended)

**Advantages:**
- ✅ Free
- ✅ Automatic HTTPS
- ✅ No port forwarding required
- ✅ DDoS protection included

**Installation:**
```bash
# Install cloudflared
yay -S cloudflared

# Authenticate
cloudflared tunnel login

# Create tunnel
cloudflared tunnel create notnative

# Configure tunnel
cat > ~/.cloudflared/config.yml << EOF
tunnel: notnative
credentials-file: /home/your-user/.cloudflared/<TUNNEL-ID>.json

ingress:
  - hostname: notnative.your-domain.com
    service: http://localhost:8788
  - service: http_status:404
EOF

# Run tunnel
cloudflared tunnel run notnative
```

### Option 2: Tailscale (Private Network)

**Advantages:**
- ✅ Secure private network
- ✅ Zero-config
- ✅ Works behind NAT/firewalls
- ✅ Not publicly exposed

**Installation:**
```bash
# Install Tailscale
yay -S tailscale

# Start service
sudo systemctl enable --now tailscaled

# Connect
sudo tailscale up
```

### Option 3: ngrok (Development/Testing)

**Advantages:**
- ✅ Instant setup
- ✅ Useful for demos/testing
- ⚠️ URL changes each session (free plan)

**Installation:**
```bash
# Install ngrok
yay -S ngrok

# Create tunnel
ngrok http 8788
```

## Security and Best Practices

### �� General Recommendations

⚠️ **IMPORTANT**: The MCP server currently only listens on `localhost:8788` to avoid external exposure.

For production/remote access, you **MUST** implement:

1. **API Key Authentication**
   ```bash
   # Add X-API-Key header in all requests
   curl -X POST https://notnative.example.com/mcp/call_tool \
     -H "Content-Type: application/json" \
     -H "X-API-Key: your-secret-api-key-here" \
     -d '...'
   ```

2. **Mandatory HTTPS/TLS**
   - Use Cloudflare Tunnel (automatic HTTPS)
   - Or configure Let's Encrypt certificates
   - **NEVER** expose unencrypted HTTP publicly

3. **Rate Limiting**
   - Limit requests per IP/API key
   - Prevent brute force attacks
   - Example: maximum 100 requests/minute

4. **IP Whitelist (optional)**
   ```bash
   # Only allow access from known IPs
   # Configure in Cloudflare Access or firewall
   ```

5. **Strict Validation**
   - Server already validates inputs
   - Review logs regularly
   - Monitor suspicious requests

## Troubleshooting

### Server doesn't start
```bash
# Check if port 8788 is occupied
lsof -i :8788

# View NotNative logs
journalctl -f | grep notnative
```

### Connection error from external client
```bash
# Verify server is running
curl http://localhost:8788/health

# Should respond:
# {"status":"ok","service":"NotNative MCP Server","version":"1.0.0"}
```

### "Invalid parameters" error
- Verify JSON has correct structure
- Check data types (string, number, etc.)
- Use `/mcp/list_tools` endpoint to see expected structure

## Contributing

Have ideas for new integrations? Open an issue on GitHub!

- Propose new MCP tools
- Share n8n workflows
- Create wrappers for other languages
- Improve documentation

## License

See LICENSE in the main repository.
