FROM rustlang/rust:nightly as builder

WORKDIR /app

# Copy shared models
COPY case_study/shared/models ./case_study/shared/models

# Copy service
COPY case_study/services/oms-service ./case_study/services/oms-service

WORKDIR /app/case_study/services/oms-service

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/case_study/services/oms-service/target/release/oms-service /usr/local/bin/

EXPOSE 8081

CMD ["oms-service"]
