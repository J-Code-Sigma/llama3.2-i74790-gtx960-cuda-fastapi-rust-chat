# Build stage
FROM nvidia/cuda:12.2.2-devel-ubuntu22.04 AS builder

RUN apt-get update && apt-get install -y \
    build-essential \
    git \
    cmake \
    curl \
    libcurl4-openssl-dev \
    && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Clone specific tag or just main
RUN git clone --depth 1 https://github.com/ggml-org/llama.cpp.git .

# Build the server and install to a temporary directory
# -DGGML_CUDA=ON enables CUDA support
# -DGGML_NATIVE=OFF (or omitted) to avoid -march=native on build host if different from runtime
# But we can try to keep default or specify logic. 
# Safe bet for generic container: let it autodect or specify arch.
# For simplicity and robustness on this machine, we trust cmake's detection or fallback.
# Link CUDA stubs for build time
RUN ln -s /usr/local/cuda/lib64/stubs/libcuda.so /usr/local/cuda/lib64/stubs/libcuda.so.1
ENV LD_LIBRARY_PATH=/usr/local/cuda/lib64/stubs:${LD_LIBRARY_PATH}

RUN cmake -B build -DGGML_CUDA=ON -DCMAKE_CUDA_ARCHITECTURES=52 -DCMAKE_INSTALL_PREFIX=/install \
    -DCMAKE_BUILD_TYPE=Release -DLLAMA_BUILD_SERVER=ON -DLLAMA_CURL=ON && \
    cmake --build build --config Release -j1 && \
    cmake --install build

# Runtime stage
FROM nvidia/cuda:12.2.2-runtime-ubuntu22.04

RUN apt-get update && apt-get install -y \
    curl \
    ca-certificates \
    libcurl4 \
    libgomp1 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /

# Copy the entire installation (bin and lib)
COPY --from=builder /install /usr/local

# Update linker cache to find the new .so files
RUN ldconfig

# The startup script will be mounted by docker-compose
ENTRYPOINT [ "/bin/sh", "/start-llama.sh" ]
