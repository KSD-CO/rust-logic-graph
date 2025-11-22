#!/bin/bash

# Script to kill all services by port (useful for cleanup)

echo "🧹 Cleaning up all microservice ports..."
echo ""

PORTS=(8080 8081 8082 8083 8084 8085 8086 50050 50051 50052 50053 50054 50055 50056)

for port in "${PORTS[@]}"; do
    pids=$(lsof -ti :$port 2>/dev/null)
    
    if [ -n "$pids" ]; then
        echo "  Killing processes on port $port: $pids"
        kill -9 $pids 2>/dev/null
    fi
done

echo ""
echo "✅ All ports cleaned up"
