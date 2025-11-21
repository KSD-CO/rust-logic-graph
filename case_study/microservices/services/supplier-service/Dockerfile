FROM rustlang/rust:nightly-slim AS builder

WORKDIR /build

# Copy workspace structure
COPY shared/models /build/shared/models
COPY proto /build/proto

# Copy service
COPY services/supplier-service /build/services/supplier-service

# Build from service directory
WORKDIR /build/services/supplier-service
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/services/supplier-service/target/release/supplier-service /usr/local/bin/

EXPOSE 8083 50053

CMD ["supplier-service"]
