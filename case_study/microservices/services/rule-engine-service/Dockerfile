FROM rustlang/rust:nightly as builder

WORKDIR /app

# Copy rust-logic-graph library (root project)
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY benches ./benches
COPY examples ./examples

# Copy case study files
COPY case_study/shared/models ./case_study/shared/models
COPY case_study/services/rule-engine-service ./case_study/services/rule-engine-service
COPY case_study/rules ./case_study/rules

WORKDIR /app/case_study/services/rule-engine-service

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/case_study/services/rule-engine-service/target/release/rule-engine-service /usr/local/bin/
COPY --from=builder /app/case_study/rules /app/rules

EXPOSE 8085

CMD ["rule-engine-service"]
