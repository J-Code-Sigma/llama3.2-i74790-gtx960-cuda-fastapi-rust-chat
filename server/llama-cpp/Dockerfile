# Build stage
FROM debian:bookworm-slim AS builder

RUN apt-get update && apt-get install -y \
    build-essential \
    git \
    cmake \
    curl \
    && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Clone specific tag or just main
RUN git clone --depth 1 https://github.com/ggml-org/llama.cpp.git .

# Build the server
# -DGGML_NATIVE=ON ensures it uses Pi's CPU instructions (NEON, etc)
RUN cmake -B build -DGGML_NATIVE=ON && \
    cmake --build build --config Release -j$(nproc) --target llama-server

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    curl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /

COPY --from=builder /app/build/bin/llama-server /usr/local/bin/llama-server

# The startup script will be mounted by docker-compose
ENTRYPOINT [ "/bin/sh", "/start-llama.sh" ]
