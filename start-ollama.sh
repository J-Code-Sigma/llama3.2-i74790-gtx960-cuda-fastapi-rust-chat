#!/bin/sh
set -e

echo "Starting Ollama server..."
ollama serve &

echo "Waiting for Ollama to be ready..."
until ollama list >/dev/null 2>&1; do
  sleep 1
done

if ollama list | grep -q "tinyllama"; then
  echo "Model 'tinyllama' already exists. Skipping pull."
else
  echo "Pulling TinyLlama model..."
  ollama pull tinyllama
fi

echo "Ollama is ready."
wait
