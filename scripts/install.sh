#!/bin/bash
set -e

# Kilok (Claude Time Tracker) - Installation Script
# Installs binaries to ~/.local/bin and configures Claude Code

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
INSTALL_DIR="${HOME}/.local/bin"

echo "=== Kilok Installation ==="
echo "Automatic time tracking for Claude Code"
echo ""
echo "Project root: $PROJECT_ROOT"
echo "Install dir:  $INSTALL_DIR"
echo ""

# Ensure install directory exists
mkdir -p "$INSTALL_DIR"

# Build release binaries
echo "Building release binaries..."
cd "$PROJECT_ROOT"
cargo build --release -p claude-time-tracker

# Install binaries
echo "Installing binaries to $INSTALL_DIR..."
cp "$PROJECT_ROOT/target/release/claude-time-tracker" "$INSTALL_DIR/"
cp "$PROJECT_ROOT/target/release/claude-time-tracker-mcp" "$INSTALL_DIR/"
chmod +x "$INSTALL_DIR/claude-time-tracker"
chmod +x "$INSTALL_DIR/claude-time-tracker-mcp"

# Setup skill symlink
SKILL_DIR="${HOME}/.claude/skills/kilok"
OLD_SKILL_DIR="${HOME}/.claude/skills/claude-time-tracker"

# Remove old skill symlink if exists
if [ -L "$OLD_SKILL_DIR" ]; then
    echo "Removing old skill symlink at $OLD_SKILL_DIR..."
    rm "$OLD_SKILL_DIR"
fi

if [ -L "$SKILL_DIR" ]; then
    echo "Skill symlink already exists."
elif [ -d "$SKILL_DIR" ]; then
    echo "Warning: $SKILL_DIR exists but is not a symlink."
    echo "Please manually remove it and run this script again."
else
    echo "Creating skill symlink..."
    mkdir -p "${HOME}/.claude/skills"
    ln -s "$PROJECT_ROOT/skill" "$SKILL_DIR"
fi

# Check if ~/.local/bin is in PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo ""
    echo "WARNING: $INSTALL_DIR is not in your PATH."
    echo "Add this to your ~/.zshrc or ~/.bashrc:"
    echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
fi

echo ""
echo "=== Kilok Installation Complete ==="
echo ""
echo "Binaries installed:"
echo "  - $INSTALL_DIR/claude-time-tracker"
echo "  - $INSTALL_DIR/claude-time-tracker-mcp"
echo ""
echo "Skill installed:"
echo "  - /kilok command is now available in Claude Code"
echo ""
echo "Next steps:"
echo ""
echo "1. Copy the statusline script:"
echo "   cp $PROJECT_ROOT/scripts/statusline-example.sh $INSTALL_DIR/kilok-statusline.sh"
echo ""
echo "2. Add to ~/.claude/settings.json:"
echo "   { \"statusline_script\": \"$INSTALL_DIR/kilok-statusline.sh\" }"
echo ""
echo "3. (Optional) Add MCP server to ~/.mcp.json:"
echo "   { \"mcpServers\": { \"time-tracker\": { \"command\": \"$INSTALL_DIR/claude-time-tracker-mcp\" } } }"
echo ""
