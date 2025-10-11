#!/bin/bash
# Ferrite Editor - Run Script
# This script runs the editor with the correct environment variables

# Disable DMA-BUF renderer to avoid GBM buffer errors
export WEBKIT_DISABLE_DMABUF_RENDERER=1

# Use X11 backend instead of Wayland
export GDK_BACKEND=x11

echo "🚀 Starting Ferrite Scene Editor..."
echo "   Environment: WEBKIT_DISABLE_DMABUF_RENDERER=1 GDK_BACKEND=x11"
echo ""

npm run tauri:dev
