#!/bin/bash

echo "Stopping rule-engine-service..."
pkill -f 'rule-engine.*cargo run' || true
sleep 2

echo "Starting rule-engine-service..."
cd /Users/jamesvu/Documents/Personals/rust-logic-graph/case_study/microservices/services/rule-engine-service

PORT=8085 GRPC_PORT=50055 cargo run > /tmp/rule-engine-service.log 2>&1 &

echo "Rule Engine Service started (PID: $!)"
echo "Waiting for service to be ready..."
sleep 3

echo ""
echo "Testing with PROD-002 (should trigger reorder)..."
curl -s -X POST http://localhost:8080/purchasing/flow \
  -H "Content-Type: application/json" \
  -d '{"product_id": "PROD-002"}' | jq '.'

echo ""
echo "View logs: tail -f /tmp/rule-engine-service.log"
