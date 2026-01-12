# GPU-Accelerated LLM Chat Stack (FastAPI + Rust + llama.cpp)

A high-performance, containerized chat application optimized for **x86_64 Linux** with **NVIDIA GPU Acceleration**.

This project orchestrates a secure, filtered chat interface using a microservices architecture. It is tuned specifically for older consumer GPUs (e.g., **GTX 960 2GB**) by offloading safety scanners to the CPU while dedicating VRAM to the chat model.

## key Features

-   **Hybrid Compute**: Runs the LLM on GPU (CUDA) and safety scanners on CPU to maximize limited VRAM.
-   **Security**: Multi-layer content filtering (Profanity filter, Prompt Injection detection, Toxicity analysis).
-   **Performance**: Native `llama.cpp` server compiled with CUDA support.
-   **Control**: Soft guardrails via a Rust middleware that injects system prompts from a `topics.txt` allowlist.

## Architecture

1.  **FastAPI (Python)**: Public Gateway. Handles auth, CORS, and runs CPU-based safety scanners (`llm-guard`).
2.  **Rust API (Actix-Web)**: Middleware. Manages request limits and system prompt injection.
3.  **llama.cpp (C++)**: Inference Engine. Runs the GGUF model on the GPU.

```mermaid
graph LR
    Client["React Client"] -->|"POST /chat"| FastAPI["FastAPI :8000"]
    FastAPI -->|"Scan (CPU)"| Scanners["LLM Guard"]
    
    Scanners -->|"Unsafe"| Block["Return refusal message (from .env)"]
    Block -->|"Return Message"
    
    Scanners -->|"Safe"| RustAPI["Rust API :8081"]
    RustAPI -->|"Inject System Prompt"| LlamaCPP["llama.cpp :11434"]
    LlamaCPP -->|"Inference (GPU)"| Model["Llama 3.2 1B"]
```

## Hardware Requirements

-   **OS**: Linux (x86_64) with NVIDIA Drivers installed.
-   **GPU**: NVIDIA GPU (Maxwell or newer supported). Tested on **GTX 960 (2GB VRAM)**.
-   **RAM**: 8GB+ System RAM recommended.
-   **Storage**: ~2GB for Docker images and models.

## Getting Started

### 1. Prerequisites
-   [Docker](https://www.docker.com/)
-   [NVIDIA Container Toolkit](https://docs.nvidia.com/datacenter/cloud-native/container-toolkit/latest/install-guide.html) (Crucial for GPU access)

### 2. Run with Docker Compose
```bash
# Start all services
# The first run will download the Llama 3.2 1B model (~700MB) automatically.
sudo docker-compose up -d --build
```

### 3. Verify GPU Usage
Check the `llama-cpp` logs to ensure layer offloading is working:
```bash
sudo docker-compose logs -f llama-cpp
# Look for: "llm_load_tensors: offloading 123 layers to GPU"
```

## Configuration

### Environment Variables (`docker-compose.yml`)

| Variable | Service | Default | Description |
| :--- | :--- | :--- | :--- |
| `ENABLE_SCANNERS` | `fastapi` | `true` | Enable/Disable CPU-heavy safety scanners. Set to `false` for instant startup. |
| `LLAMACPP_HOST` | `fastapi` | `http://rust-api:8080` | Internal URL for the Rust middleware. |

### Topic Control
Edit `server/RUST/topics.txt` to change the allowed conversation topics dynamically. The Rust middleware reads this file on every request, so updates are instant (no restart required).

## Troubleshooting

-   **"CUDA error: the resource allocation failed"**: You are running out of VRAM. Ensure `ENABLE_SCANNERS` is `false` (to save system RAM/VRAM overhead) or enable scanners but ensure they run on CPU (default config).
-   **Model Download Failed**: If logs show garbage output, the download might have been corrupted. Delete `models/*.gguf` and restart.

## License
[MIT](LICENSE)
