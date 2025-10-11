#!/bin/bash
# Helper script to run the editor with proper environment variables

# Disable DMA-BUF renderer to avoid GBM buffer errors
export WEBKIT_DISABLE_DMABUF_RENDERER=1

# Use X11 backend instead of Wayland
export GDK_BACKEND=x11

# Run the dev server
npm run tauri:dev
