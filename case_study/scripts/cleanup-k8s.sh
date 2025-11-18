#!/bin/bash
set -e

# Clean up all Kubernetes resources
# Usage: ./scripts/cleanup-k8s.sh

echo "Cleaning up Purchasing Flow from Kubernetes..."

read -p "This will delete all resources in the 'purchasing-flow' namespace. Continue? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
  echo "Cancelled."
  exit 1
fi

cd "$(dirname "$0")/.."

# Delete all resources
echo "Deleting all deployments and services..."
kubectl delete -f k8s/deployments/ --ignore-not-found=true
kubectl delete -f k8s/services/ --ignore-not-found=true

echo "Deleting ConfigMaps and Secrets..."
kubectl delete -f k8s/configmaps/ --ignore-not-found=true
kubectl delete -f k8s/secrets/ --ignore-not-found=true

echo "Deleting namespace..."
kubectl delete -f k8s/namespace.yaml --ignore-not-found=true

echo ""
echo "==================================="
echo "Cleanup complete!"
echo "==================================="
