#!/bin/bash

echo "🔄 Restarting rule-engine-service..."
pkill -f 'rule-engine.*cargo' 2>/dev/null || true
sleep 1

cd /Users/jamesvu/Documents/Personals/rust-logic-graph/case_study/microservices/services/rule-engine-service
PORT=8085 GRPC_PORT=50055 cargo run > /tmp/rule-engine-service.log 2>&1 &
PID=$!

echo "✅ Started rule-engine-service (PID: $PID)"
echo "⏳ Waiting for service to be ready..."
sleep 4

echo ""
echo "📋 Test Results:"
echo "================"

echo ""
echo "🧪 Test 1: PROD-001 (No reorder needed)"
curl -s -X POST http://localhost:8080/purchasing/flow \
  -H "Content-Type: application/json" \
  -d '{"product_id": "PROD-001"}' | jq '{need_reorder: .calculation.need_reorder, shortage: .calculation.shortage, order_qty: .calculation.order_qty, total: .calculation.total_amount}'

echo ""
echo "🧪 Test 2: PROD-002 (Standard reorder)"
curl -s -X POST http://localhost:8080/purchasing/flow \
  -H "Content-Type: application/json" \
  -d '{"product_id": "PROD-002"}' | jq '{need_reorder: .calculation.need_reorder, shortage: .calculation.shortage, order_qty: .calculation.order_qty, total: .calculation.total_amount, approval: .calculation.requires_approval}'

echo ""
echo "🧪 Test 3: PROD-003 (High value - requires approval)"
curl -s -X POST http://localhost:8080/purchasing/flow \
  -H "Content-Type: application/json" \
  -d '{"product_id": "PROD-003"}' | jq '{need_reorder: .calculation.need_reorder, order_qty: .calculation.order_qty, total: .calculation.total_amount, approval: .calculation.requires_approval, status: .calculation.approval_status}'

echo ""
echo "🧪 Test 4: PROD-004 (Increasing trend)"
curl -s -X POST http://localhost:8080/purchasing/flow \
  -H "Content-Type: application/json" \
  -d '{"product_id": "PROD-004"}' | jq '{need_reorder: .calculation.need_reorder, shortage: .calculation.shortage, order_qty: .calculation.order_qty, total: .calculation.total_amount}'

echo ""
echo "📊 View logs: tail -f /tmp/rule-engine-service.log"
