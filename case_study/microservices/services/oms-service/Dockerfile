FROM rustlang/rust:nightly-slim AS builder

WORKDIR /build

# Copy workspace structure
COPY shared/models /build/shared/models
COPY proto /build/proto

# Copy service
COPY services/oms-service /build/services/oms-service

# Build from service directory
WORKDIR /build/services/oms-service
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/services/oms-service/target/release/oms-service /usr/local/bin/

EXPOSE 8081 50051

CMD ["oms-service"]
