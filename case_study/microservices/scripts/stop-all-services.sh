#!/bin/bash

# Script to stop all running services

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/.."

echo "⏹️  Stopping all microservices..."
echo ""

# First, try to stop by PID files
stopped_any=false

# Kill all services by PID files
for pidfile in logs/*.pid; do
    if [ -f "$pidfile" ]; then
        pid=$(cat "$pidfile")
        service=$(basename "$pidfile" .pid)
        
        if kill -0 $pid 2>/dev/null; then
            echo "  Stopping $service (PID: $pid)"
            kill $pid 2>/dev/null
            stopped_any=true
            
            # Wait a bit for graceful shutdown
            sleep 1
            
            # Force kill if still running
            if kill -0 $pid 2>/dev/null; then
                echo "  Force stopping $service"
                kill -9 $pid 2>/dev/null
            fi
        fi
        
        rm "$pidfile"
    fi
done

# If no PIDs were found, try killing by ports
if [ "$stopped_any" = false ]; then
    echo "  No PID files found, checking ports..."
    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    "$SCRIPT_DIR/kill-all-ports.sh"
else
    echo ""
    echo "✅ All services stopped"
fi

