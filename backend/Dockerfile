FROM rust:1.85-slim AS builder
WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/* \
    && useradd -r -s /bin/false appuser
COPY --from=builder /app/target/release/allmaptout-backend /usr/local/bin/
USER appuser
EXPOSE 3001
CMD ["allmaptout-backend"]
