FROM rustlang/rust:nightly-slim AS builder

WORKDIR /build

# Copy workspace structure
COPY shared/models /build/shared/models
COPY proto /build/proto

# Copy service
COPY services/inventory-service /build/services/inventory-service

# Build from service directory
WORKDIR /build/services/inventory-service
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/services/inventory-service/target/release/inventory-service /usr/local/bin/

EXPOSE 8082 50052

CMD ["inventory-service"]
