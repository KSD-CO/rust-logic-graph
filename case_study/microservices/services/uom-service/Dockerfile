FROM rustlang/rust:nightly-slim AS builder

WORKDIR /build

# Copy workspace structure
COPY shared/models /build/shared/models
COPY proto /build/proto

# Copy service
COPY services/uom-service /build/services/uom-service

# Build from service directory
WORKDIR /build/services/uom-service
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/services/uom-service/target/release/uom-service /usr/local/bin/

EXPOSE 8084 50054

CMD ["uom-service"]
