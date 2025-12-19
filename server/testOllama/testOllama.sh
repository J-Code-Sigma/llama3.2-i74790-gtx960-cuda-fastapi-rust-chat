#!/bin/bash
for i in {1..10}
do
  echo "Request $i"
  curl -s http://localhost:11434/api/generate \
    -H "Content-Type: application/json" \
    -d '{
      "model": "tinyllama",
      "prompt": "Summarize this text: The quick brown fox jumps over the lazy dog.",
      "stream": false
    }'
  echo ""
done
