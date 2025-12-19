seq 1 5 | parallel -j2 'curl -s http://localhost:11434/api/generate \
  -H "Content-Type: application/json" \
  -d "{\"model\": \"tinyllama\", \"prompt\": \"Hello Pi!\", \"stream\": false}"'
