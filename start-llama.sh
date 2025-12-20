#!/bin/sh
set -e

MODEL_URL="https://huggingface.co/bartowski/Llama-3.2-1B-Instruct-GGUF/resolve/main/Llama-3.2-1B-Instruct-Q4_K_M.gguf?download=true"
MODEL_PATH="/models/Llama-3.2-1B-Instruct-Q4_K_M.gguf"

if [ -f "$MODEL_PATH" ] && [ $(stat -c%s "$MODEL_PATH") -lt 1000000 ]; then
  echo "Existing model file is too small, likely a failed download. Removing..."
  rm "$MODEL_PATH"
fi

if [ ! -f "$MODEL_PATH" ]; then
  echo "Downloading Llama 3.2 1B GGUF model (~700MB)..."
  curl -L "$MODEL_URL" -o "$MODEL_PATH"
else
  echo "Model already exists and seems valid. Skipping download."
fi

echo "Starting llama.cpp server..."
# --host 0.0.0.0 to allow connections from other containers
# --port 11434 to keep existing mapping or change as needed
# -m for the model path
# -c 2048 Caps the conversation memory (Context) to save RAM (Crucial for Pi!)
# --cache-ram 0 Disables the massive prompt cache to save RAM
llama-server --host 0.0.0.0 --port 11434 -m "$MODEL_PATH" -c 2048 --cache-ram 0
