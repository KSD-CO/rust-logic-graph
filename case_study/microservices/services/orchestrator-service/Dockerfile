FROM rustlang/rust:nightly as builder

WORKDIR /app

COPY case_study/shared/models ./case_study/shared/models
COPY case_study/services/orchestrator-service ./case_study/services/orchestrator-service

WORKDIR /app/case_study/services/orchestrator-service

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/case_study/services/orchestrator-service/target/release/orchestrator-service /usr/local/bin/

EXPOSE 8080

CMD ["orchestrator-service"]
