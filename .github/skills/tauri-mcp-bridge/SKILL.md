---
name: tauri-mcp-bridge
description: MCP Bridge setup and tool reference for Tauri v2 apps. Use when setting up MCP Bridge plugin, troubleshooting connection issues, or looking up which MCP tool to use for UI automation tasks (screenshots, DOM inspection, element interaction, backend state queries).
---

# Tauri MCP Bridge

Reference for MCP Bridge plugin setup and tool usage in Tauri v2 apps.

## Setup Checklist

Run `scripts/check-setup.sh` to verify setup, or check manually:

| Requirement | File | What to Check |
|-------------|------|---------------|
| Dependency | `src-tauri/Cargo.toml` | `tauri-plugin-mcp-bridge = "0.8"` |
| Plugin registration | `src-tauri/src/lib.rs` | `builder.plugin(tauri_plugin_mcp_bridge::init())` in `#[cfg(debug_assertions)]` |
| Global Tauri | `src-tauri/tauri.conf.json` | `"withGlobalTauri": true` in `app` section |
| Permission | `src-tauri/capabilities/default.json` | `"mcp-bridge:default"` in permissions array |
| VS Code config | `.vscode/mcp.json` | Server configured for `@hypothesi/tauri-mcp-server` |

## Connection Workflow

```
1. Start app: npm run tauri dev
2. Wait ~10s for WebSocket server (port 9223)
3. driver_session(action="start")
4. driver_session(action="status") → verify connected=true
```

## Tool Catalog

### Session Management

| Tool | Purpose |
|------|---------|
| `driver_session` | Start/stop/status of MCP connection |

### UI Inspection

| Tool | Purpose |
|------|---------|
| `webview_screenshot` | Capture current UI as image |
| `webview_dom_snapshot` | Get full element tree |
| `webview_find_element` | Locate elements by CSS selector |
| `webview_get_styles` | Get computed styles of element |

### UI Interaction

| Tool | Purpose |
|------|---------|
| `webview_interact` | Click, focus, select, scroll |
| `webview_keyboard` | Type text, press keys |
| `webview_wait_for` | Wait for element/condition |

### Backend Integration

| Tool | Purpose |
|------|---------|
| `ipc_get_backend_state` | Query Rust state (e.g., StateMachine) |
| `ipc_execute_command` | Call Tauri commands directly |
| `ipc_emit_event` | Emit events to frontend |
| `ipc_get_captured` | Get captured IPC traffic |
| `ipc_monitor` | Monitor IPC calls in real-time |

### Window Management

| Tool | Purpose |
|------|---------|
| `manage_window` | Move, resize, focus windows |
| `list_devices` | List available input devices |

### Logs

| Tool | Purpose |
|------|---------|
| `read_logs` | Read Tauri app logs |

## Troubleshooting

### Connection Fails

1. **App not running**: Start with `npm run tauri dev`, wait for window
2. **Wrong port**: WebSocket binds to 9223 by default
3. **Missing globalTauri**: Add `"withGlobalTauri": true` to tauri.conf.json
4. **Missing permission**: Add `"mcp-bridge:default"` to capabilities
5. **Plugin not registered**: Check lib.rs has plugin init in debug block
6. **Port in use**: Kill other processes on 9223, or configure different port

### Element Not Found

1. Check selector syntax (CSS selectors)
2. Use `webview_dom_snapshot` to see actual element tree
3. Element may not be rendered yet—use `webview_wait_for`
4. Element may be in shadow DOM or iframe

### Backend State Empty

1. Verify state is exposed via Tauri command
2. Check command is registered in `generate_handler![]`
3. Use `ipc_execute_command` to call `get_state` or similar

## Common Patterns

### Verify Button Exists and Works

```
1. webview_find_element(selector="button.start-btn")
2. Verify: element found
3. webview_interact(action="click", selector="button.start-btn")
4. webview_screenshot() → capture result
5. ipc_get_backend_state() → verify state changed
```

### Wait for Async Operation

```
1. webview_interact(action="click", selector="#submit")
2. webview_wait_for(selector=".success-message", timeout=5000)
3. webview_screenshot() → capture final state
```

### Capture Full UI State

```
1. webview_screenshot() → visual state
2. webview_dom_snapshot() → element tree
3. ipc_get_backend_state() → Rust state
```
