#!/bin/bash
# Check Tauri MCP Bridge setup requirements
# Usage: ./check-setup.sh [project-dir]

set -e

PROJECT_DIR="${1:-.}"
TAURI_DIR="$PROJECT_DIR/src-tauri"

if [[ ! -d "$TAURI_DIR" ]]; then
    # Try nested structure (e.g., opensnipping/opensnipping/src-tauri)
    for dir in "$PROJECT_DIR"/*/src-tauri; do
        if [[ -d "$dir" ]]; then
            TAURI_DIR="$dir"
            PROJECT_DIR="$(dirname "$dir")"
            break
        fi
    done
fi

if [[ ! -d "$TAURI_DIR" ]]; then
    echo "❌ Not a Tauri project: src-tauri/ not found"
    exit 1
fi

echo "Checking MCP Bridge setup in: $TAURI_DIR"
echo "========================================="

ERRORS=0

# 1. Check Cargo.toml dependency
echo -n "1. Cargo.toml dependency: "
if grep -q 'tauri-plugin-mcp-bridge' "$TAURI_DIR/Cargo.toml" 2>/dev/null; then
    echo "✓"
else
    echo "❌ Missing tauri-plugin-mcp-bridge"
    ERRORS=$((ERRORS + 1))
fi

# 2. Check plugin registration in lib.rs
echo -n "2. Plugin registration (lib.rs): "
if grep -q 'tauri_plugin_mcp_bridge::init' "$TAURI_DIR/src/lib.rs" 2>/dev/null; then
    echo "✓"
else
    echo "❌ Missing plugin init in lib.rs"
    ERRORS=$((ERRORS + 1))
fi

# 3. Check withGlobalTauri in tauri.conf.json
echo -n "3. withGlobalTauri setting: "
if grep -q '"withGlobalTauri".*true' "$TAURI_DIR/tauri.conf.json" 2>/dev/null; then
    echo "✓"
else
    echo "❌ Missing withGlobalTauri: true in tauri.conf.json"
    ERRORS=$((ERRORS + 1))
fi

# 4. Check permission in capabilities
echo -n "4. MCP Bridge permission: "
CAPS_FILE="$TAURI_DIR/capabilities/default.json"
if [[ -f "$CAPS_FILE" ]] && grep -q 'mcp-bridge:default' "$CAPS_FILE" 2>/dev/null; then
    echo "✓"
else
    echo "❌ Missing mcp-bridge:default permission"
    ERRORS=$((ERRORS + 1))
fi

# 5. Check VS Code MCP config
echo -n "5. VS Code MCP config: "
MCP_CONFIG="$PROJECT_DIR/../.vscode/mcp.json"
if [[ ! -f "$MCP_CONFIG" ]]; then
    MCP_CONFIG="$PROJECT_DIR/.vscode/mcp.json"
fi
if [[ -f "$MCP_CONFIG" ]] && grep -q 'tauri-mcp-server' "$MCP_CONFIG" 2>/dev/null; then
    echo "✓"
else
    echo "⚠ Missing or incomplete .vscode/mcp.json (optional for non-VS Code usage)"
fi

echo "========================================="
if [[ $ERRORS -eq 0 ]]; then
    echo "✓ All required checks passed"
    exit 0
else
    echo "❌ $ERRORS issue(s) found"
    exit 1
fi
