# ----------------------
# Builder Stage
# ----------------------
FROM rust:latest AS builder

WORKDIR /usr/src/app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

# ----------------------
# Runtime Stage
# ----------------------
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /usr/src/app/target/release/llammacpp_server ./llammacpp_server

EXPOSE 8080
CMD ["./llammacpp_server"]
