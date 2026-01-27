#!/bin/bash
set -e

VERSION="${1:-1.1.0}"
REPO="samvallad33/vestige"

echo "Building Vestige MCPB v${VERSION}..."

# Create server directory
mkdir -p server

# Download macOS ARM64
echo "Downloading macOS ARM64 binary..."
curl -sL "https://github.com/${REPO}/releases/download/v${VERSION}/vestige-mcp-aarch64-apple-darwin.tar.gz" | tar -xz -C server
mv server/vestige-mcp server/vestige-mcp-darwin-arm64
mv server/vestige server/vestige-darwin-arm64

# Download Linux x64
echo "Downloading Linux x64 binary..."
curl -sL "https://github.com/${REPO}/releases/download/v${VERSION}/vestige-mcp-x86_64-unknown-linux-gnu.tar.gz" | tar -xz -C server
mv server/vestige-mcp server/vestige-mcp-linux-x64
mv server/vestige server/vestige-linux-x64

# Download Windows x64
echo "Downloading Windows x64 binary..."
curl -sL "https://github.com/${REPO}/releases/download/v${VERSION}/vestige-mcp-x86_64-pc-windows-msvc.zip" -o /tmp/win.zip
unzip -q /tmp/win.zip -d server
mv server/vestige-mcp.exe server/vestige-mcp-win32-x64.exe
mv server/vestige.exe server/vestige-win32-x64.exe
rm /tmp/win.zip

# Make executable
chmod +x server/*

echo "Binaries downloaded. Run 'mcpb pack' to create bundle."
