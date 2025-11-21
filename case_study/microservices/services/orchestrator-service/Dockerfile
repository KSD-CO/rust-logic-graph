FROM rustlang/rust:nightly-slim AS builder

WORKDIR /build

# Copy workspace structure
COPY shared/models /build/shared/models
COPY proto /build/proto

# Copy service
COPY services/orchestrator-service /build/services/orchestrator-service

# Build from service directory
WORKDIR /build/services/orchestrator-service
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/services/orchestrator-service/target/release/orchestrator-service /usr/local/bin/

EXPOSE 8080

CMD ["orchestrator-service"]
