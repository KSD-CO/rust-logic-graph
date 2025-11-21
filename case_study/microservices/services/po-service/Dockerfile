FROM rustlang/rust:nightly-slim AS builder

WORKDIR /build

# Copy workspace structure
COPY shared/models /build/shared/models
COPY proto /build/proto

# Copy service
COPY services/po-service /build/services/po-service

# Build from service directory
WORKDIR /build/services/po-service
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/services/po-service/target/release/po-service /usr/local/bin/

EXPOSE 8086 50056

CMD ["po-service"]
