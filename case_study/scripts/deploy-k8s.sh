#!/bin/bash
set -e

# Deploy all resources to Kubernetes
# Usage: ./scripts/deploy-k8s.sh

echo "Deploying Purchasing Flow to Kubernetes..."

cd "$(dirname "$0")/.."

# Create namespace
echo "Creating namespace..."
kubectl apply -f microservices/k8s/namespace.yaml

# Deploy ConfigMaps and Secrets
echo "Deploying ConfigMaps and Secrets..."
kubectl apply -f microservices/k8s/configmaps/
kubectl apply -f microservices/k8s/secrets/

# Deploy MySQL
echo "Deploying MySQL database..."
kubectl apply -f microservices/k8s/deployments/mysql-deployment.yaml
kubectl apply -f microservices/k8s/services/mysql-service.yaml

# Wait for MySQL to be ready
echo "Waiting for MySQL to be ready..."
kubectl wait --for=condition=ready pod -l app=mysql -n purchasing-flow --timeout=300s || true

# Deploy all microservices
echo "Deploying microservices..."
kubectl apply -f microservices/k8s/deployments/
kubectl apply -f microservices/k8s/services/

echo ""
echo "==================================="
echo "Deployment complete!"
echo "==================================="
echo ""
echo "Check deployment status:"
echo "  kubectl get all -n purchasing-flow"
echo ""
echo "View logs:"
echo "  kubectl logs -n purchasing-flow -l app=orchestrator-service --tail=50"
echo ""
echo "Port-forward orchestrator service:"
echo "  kubectl port-forward -n purchasing-flow service/orchestrator-service 8080:8080"
