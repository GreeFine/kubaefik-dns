FROM rust:1.65 as builder

WORKDIR /app

# Create fake file to pre-load dependencies
COPY Cargo.lock Cargo.toml /app/
RUN mkdir -p ./src && \
    echo 'fn main(){}' > ./src/main.rs && \
    cargo build && \
    rm ./src/main.rs

COPY . .
RUN cargo build --release

FROM debian:stable-slim AS runtime
WORKDIR /app

RUN apt update && apt install -y ca-certificates libssl1.1 && rm -rf /var/lib/apt/lists/* 
COPY --from=builder /app/target/release/kubaefik-dns /app/kubaefik-dns
ENTRYPOINT ["/app/kubaefik-dns"]
