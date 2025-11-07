# Background Control System

NotNative can now run in the background and be controlled from external scripts, waybar, or hyprland.

## Implemented Features

### 1. Single Instance Detection
- Only allows one instance of the app running at the same time
- Lock file at `/tmp/notnative.lock` with the process PID
- PID validation before rejecting (detects dead processes)
- Automatic cleanup on exit

```bash
# If you try to open another instance:
$ notnative-app
❌ NotNative is already running (PID: 123456)
💡 If you think this is an error, remove: /tmp/notnative.lock
```

### 2. Window Hide/Show
- When closing the window (X or Ctrl+Q), the app minimizes to background
- The app keeps running (MCP Server active, music playing)
- Window can be shown/hidden on demand

### 3. File-based Control System
Since GTK4/Wayland doesn't support traditional system tray, we use a file-based control system:

```bash
# Included helper script
./notnative-control.sh show    # Show window
./notnative-control.sh hide    # Hide window  
./notnative-control.sh toggle  # Toggle visibility
./notnative-control.sh quit    # Quit completely
```

Or directly:
```bash
echo "show" > /tmp/notnative.control
echo "hide" > /tmp/notnative.control
echo "quit" > /tmp/notnative.control
```

The app monitors `/tmp/notnative.control` every 500ms and executes commands automatically.

### 4. Waybar Integration

Add to your waybar config (`~/.config/waybar/config`):

```json
{
  "modules-right": ["custom/notnative", "..."],
  
  "custom/notnative": {
    "format": "📝 NotNative",
    "on-click": "/path/to/notnative-control.sh toggle",
    "on-click-right": "/path/to/notnative-control.sh quit",
    "tooltip": true,
    "tooltip-format": "Click: Show/Hide\nRight click: Quit"
  }
}
```

### 5. Hyprland Integration

Add keyboard shortcuts in `~/.config/hypr/hyprland.conf`:

```conf
# Show/hide NotNative
bind = SUPER, N, exec, /path/to/notnative-control.sh toggle

# Quit NotNative completely
bind = SUPER_SHIFT, N, exec, /path/to/notnative-control.sh quit
```

## Use Cases

### MCP Server Always Available
```bash
# Start NotNative in background at login
notnative-app &

# Hide window if visible
./notnative-control.sh hide

# Now the MCP Server is available 24/7 at http://localhost:8788
# You can create notes from n8n, scripts, etc. without having the window visible
```

### Workflow with Waybar
1. Click on waybar icon → window appears
2. Work on your notes
3. Close the window (X) → minimizes to background
4. MCP Server stays active
5. Right click on waybar → app closes completely

### Control from Scripts
```bash
#!/bin/bash
# Create note from external script

# Ensure NotNative is running
if [ ! -f /tmp/notnative.lock ]; then
    notnative-app &
    sleep 2
fi

# Create note via MCP
curl -X POST http://localhost:8788/mcp/call_tool \
  -H "Content-Type: application/json" \
  -d '{
    "tool": "CreateNote",
    "name": "Script Note",
    "content": "Created from script"
  }'

# Show window to view the note
./notnative-control.sh show
```

## Control Files

- `/tmp/notnative.lock` - Lock file with PID (prevents multiple instances)
- `/tmp/notnative.control` - Commands to control the app (show/hide/quit)
- `/tmp/notnative_mcp_update.signal` - MCP changes signal (auto-refresh)

All are automatically cleaned up when closing the app.

## Limitations

- **No visual system tray icon**: GTK4 + Wayland don't support traditional libappindicator
- **Alternative solution**: Use waybar custom module or control scripts
- **Real system tray**: Would require implementing D-Bus StatusNotifierItem (complex)

## Advantages of Current System

✅ Works perfectly on Wayland/Hyprland  
✅ Integrable with waybar, rofi, any script  
✅ No extra dependencies required (D-Bus, etc)  
✅ Simple and reliable  
✅ MCP Server always available in background  
