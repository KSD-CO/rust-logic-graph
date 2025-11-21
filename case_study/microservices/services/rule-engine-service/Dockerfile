FROM rustlang/rust:nightly-slim AS builder

WORKDIR /build

# Copy workspace structure
COPY shared/models /build/shared/models
COPY proto /build/proto

# Copy service
COPY services/rule-engine-service /build/services/rule-engine-service

# Build from service directory
WORKDIR /build/services/rule-engine-service
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/services/rule-engine-service/target/release/rule-engine-service /usr/local/bin/

EXPOSE 8085 50055

CMD ["rule-engine-service"]
