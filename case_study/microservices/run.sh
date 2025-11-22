#!/bin/bash

# Quick launcher - choose how to run services

echo "╔══════════════════════════════════════════════════════════╗"
echo "║         Rust Logic Graph - Microservices Launcher        ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo ""
echo "How do you want to run the services?"
echo ""
echo "  1) Background mode (recommended) - Services run in background"
echo "     → Good for: Testing, leaving services running"
echo "     → Stop with: ./scripts/stop-all-services.sh"
echo ""
echo "  2) Interactive mode - See all logs in terminal"
echo "     → Good for: Debugging, watching logs"
echo "     → Stop with: Ctrl+C"
echo ""
echo "  3) tmux mode - Each service in separate window"
echo "     → Good for: Development, restart individual services"
echo "     → Requires: tmux (brew install tmux)"
echo ""
echo "  4) Check status of services"
echo "  5) Stop all services"
echo "  q) Quit"
echo ""
read -p "Select option [1-5, q]: " choice

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

case $choice in
    1)
        echo ""
        echo "🚀 Starting services in background mode..."
        ./scripts/run-services-background.sh
        ;;
    2)
        echo ""
        echo "🚀 Starting services in interactive mode..."
        echo "   Press Ctrl+C to stop all services"
        ./scripts/run-services-interactive.sh
        ;;
    3)
        if ! command -v tmux &> /dev/null; then
            echo ""
            echo "❌ tmux is not installed"
            echo "   Install with: brew install tmux"
            exit 1
        fi
        echo ""
        echo "🚀 Starting services in tmux mode..."
        ./scripts/run-all-tmux.sh
        ;;
    4)
        echo ""
        ./scripts/status-services.sh
        ;;
    5)
        echo ""
        ./scripts/stop-all-services.sh
        ;;
    q|Q)
        echo "Bye! 👋"
        exit 0
        ;;
    *)
        echo ""
        echo "❌ Invalid option"
        exit 1
        ;;
esac
