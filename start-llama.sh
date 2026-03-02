#!/bin/sh
set -e

# --- MODEL SELECTION ---
# Default for GTX 960 (2GB VRAM): Llama 3.2 1B (Tiny but fast)
MODEL_URL="https://huggingface.co/bartowski/Llama-3.2-1B-Instruct-GGUF/resolve/main/Llama-3.2-1B-Instruct-Q4_K_M.gguf?download=true"
MODEL_PATH="/models/Llama-3.2-1B-Instruct-Q4_K_M.gguf"

# Better GPU option (e.g. RTX 3060 12GB+): Llama 3.2 3B (Better reasoning)
# MODEL_URL="https://huggingface.co/bartowski/Llama-3.2-3B-Instruct-GGUF/resolve/main/Llama-3.2-3B-Instruct-Q8_0.gguf"
# MODEL_PATH="/models/Llama-3.2-3B-Instruct-Q8_0.gguf"

# High-End GPU option (e.g. RTX 3090/4090): Llama 3 8B (Much more capable)
# MODEL_URL="https://huggingface.co/bartowski/Meta-Llama-3-8B-Instruct-GGUF/resolve/main/Meta-Llama-3-8B-Instruct-Q5_K_M.gguf"
# MODEL_PATH="/models/Meta-Llama-3-8B-Instruct-Q5_K_M.gguf"


if [ -f "$MODEL_PATH" ] && [ $(stat -c%s "$MODEL_PATH") -lt 600000000 ]; then
  echo "Existing model file is too small, likely a failed download. Content start:"
  head -n 20 "$MODEL_PATH"
  echo "Removing..."
  rm "$MODEL_PATH"
fi

if [ ! -f "$MODEL_PATH" ]; then
  echo "Downloading model file..."
  if ! curl -f -L "$MODEL_URL" -o "$MODEL_PATH"; then
    echo "Download failed!"
    exit 1
  fi
else
  echo "Model already exists and seems valid. Skipping download."
fi

echo "Starting llama.cpp server..."
# --host 0.0.0.0 to allow connections from other containers
# --port 11434 to keep existing mapping or change as needed
# -m for the model path
# -c: Context size (memory). 2048 is safe for low RAM; 8192+ for better cards.
# -ngl: Number of GPU layers. 0 for CPU, 99 for everything to GPU.

# Default (Optimized for GTX 960 2GB)
llama-server --host 0.0.0.0 --port 11434 -m "$MODEL_PATH" -c 2048 --cache-ram 0

# High-Performance (Optimized for RTX 3060+ 12GB+)
# llama-server --host 0.0.0.0 --port 11434 -m "$MODEL_PATH" -c 8192 -ngl 99 -b 512 -ub 512 --cache-ram 0
