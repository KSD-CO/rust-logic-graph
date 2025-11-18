# Purchasing Flow Microservices - Deployment Guide

This guide explains how to deploy the Purchasing Flow application as microservices on Kubernetes.

## Architecture Overview

The application has been redesigned from a monolithic structure to a microservices architecture with the following services:

```
┌─────────────────────────────────────────────────────────────┐
│                    Orchestrator Service                      │
│                    (API Gateway - Port 8080)                 │
└──────────────┬──────────────────────────────────────────────┘
               │
       ┌───────┴────────┐
       │                │
   ┌───▼───┐        ┌──▼────┐
   │  OMS  │        │ Inv.  │
   │ :8081 │        │ :8082 │
   └───┬───┘        └───┬───┘
       │                │
   ┌───▼────┐      ┌───▼────┐
   │ Supp.  │      │  UOM   │
   │ :8083  │      │ :8084  │
   └────────┘      └────────┘
       │
   ┌───▼──────────┐
   │ Rule Engine  │
   │    :8085     │
   └───┬──────────┘
       │
   ┌───▼──────┐
   │ PO Svc   │
   │  :8086   │
   └──────────┘
```

### Services:

1. **Orchestrator Service** (Port 8080) - API Gateway that orchestrates the purchasing workflow
2. **OMS Service** (Port 8081) - Order Management System data
3. **Inventory Service** (Port 8082) - Inventory levels data
4. **Supplier Service** (Port 8083) - Supplier information
5. **UOM Service** (Port 8084) - Unit of Measurement conversions
6. **Rule Engine Service** (Port 8085) - Business rules processing (GRL)
7. **PO Service** (Port 8086) - Purchase Order creation and sending

## Prerequisites

### For Local Development (Docker Compose)
- Docker & Docker Compose
- At least 4GB RAM available for containers

### For Kubernetes Deployment
- Kubernetes cluster (minikube, kind, or cloud provider)
- kubectl configured
- Helm (optional, for advanced deployments)
- Docker registry access (for pushing images)

## Directory Structure

```
case_study/
├── services/
│   ├── oms-service/          # OMS microservice
│   ├── inventory-service/    # Inventory microservice
│   ├── supplier-service/     # Supplier microservice
│   ├── uom-service/          # UOM microservice
│   ├── rule-engine-service/  # Rule Engine microservice
│   ├── po-service/           # Purchase Order microservice
│   └── orchestrator-service/ # API Gateway orchestrator
├── shared/
│   └── models/               # Shared data models library
├── k8s/
│   ├── namespace.yaml
│   ├── configmaps/           # Configuration files
│   ├── secrets/              # Secrets (DB credentials)
│   ├── deployments/          # Kubernetes deployments
│   └── services/             # Kubernetes services
├── docker-compose.yml        # Local development
└── MICROSERVICES_DEPLOYMENT.md  # This file
```

## Quick Start - Local Development

### 1. Build and Run with Docker Compose

```bash
cd case_study

# Build all services
docker-compose build

# Start all services
docker-compose up -d

# Check service health
docker-compose ps

# View logs
docker-compose logs -f orchestrator-service
```

### 2. Test the API

```bash
# Health check all services
curl http://localhost:8080/health
curl http://localhost:8081/health
curl http://localhost:8082/health
curl http://localhost:8083/health
curl http://localhost:8084/health
curl http://localhost:8085/health
curl http://localhost:8086/health

# Execute purchasing flow
curl -X POST http://localhost:8080/purchasing/flow \
  -H "Content-Type: application/json" \
  -d '{"product_id": "PROD-001"}'
```

### 3. Stop Services

```bash
docker-compose down

# With volume cleanup
docker-compose down -v
```

## Kubernetes Deployment

### Step 1: Build and Push Docker Images

First, build all service images and push to your registry:

```bash
cd case_study

# Set your Docker registry
export DOCKER_REGISTRY="your-registry.io/purchasing-flow"

# Build all images
docker build -f services/oms-service/Dockerfile -t $DOCKER_REGISTRY/oms-service:latest .
docker build -f services/inventory-service/Dockerfile -t $DOCKER_REGISTRY/inventory-service:latest .
docker build -f services/supplier-service/Dockerfile -t $DOCKER_REGISTRY/supplier-service:latest .
docker build -f services/uom-service/Dockerfile -t $DOCKER_REGISTRY/uom-service:latest .
docker build -f services/rule-engine-service/Dockerfile -t $DOCKER_REGISTRY/rule-engine-service:latest .
docker build -f services/po-service/Dockerfile -t $DOCKER_REGISTRY/po-service:latest .
docker build -f services/orchestrator-service/Dockerfile -t $DOCKER_REGISTRY/orchestrator-service:latest .

# Push to registry
docker push $DOCKER_REGISTRY/oms-service:latest
docker push $DOCKER_REGISTRY/inventory-service:latest
docker push $DOCKER_REGISTRY/supplier-service:latest
docker push $DOCKER_REGISTRY/uom-service:latest
docker push $DOCKER_REGISTRY/rule-engine-service:latest
docker push $DOCKER_REGISTRY/po-service:latest
docker push $DOCKER_REGISTRY/orchestrator-service:latest
```

Or use the build script:

```bash
# Create build script
cat > build-and-push.sh <<'EOF'
#!/bin/bash
set -e

REGISTRY=${DOCKER_REGISTRY:-"purchasing-flow"}

services=(
  "oms-service"
  "inventory-service"
  "supplier-service"
  "uom-service"
  "rule-engine-service"
  "po-service"
  "orchestrator-service"
)

for service in "${services[@]}"; do
  echo "Building $service..."
  docker build -f services/$service/Dockerfile -t $REGISTRY/$service:latest .

  if [ "$PUSH" = "true" ]; then
    echo "Pushing $service..."
    docker push $REGISTRY/$service:latest
  fi
done

echo "Done!"
EOF

chmod +x build-and-push.sh

# Build only
./build-and-push.sh

# Build and push
PUSH=true ./build-and-push.sh
```

### Step 2: Update Image References

Update the image references in the deployment files to match your registry:

```bash
# Update all deployment files
find k8s/deployments -name "*.yaml" -exec sed -i '' \
  's|purchasing-flow/|your-registry.io/purchasing-flow/|g' {} \;
```

### Step 3: Deploy to Kubernetes

```bash
# Create namespace
kubectl apply -f k8s/namespace.yaml

# Deploy ConfigMaps and Secrets
kubectl apply -f k8s/configmaps/
kubectl apply -f k8s/secrets/

# Deploy MySQL database
kubectl apply -f k8s/deployments/mysql-deployment.yaml
kubectl apply -f k8s/services/mysql-service.yaml

# Wait for MySQL to be ready
kubectl wait --for=condition=ready pod -l app=mysql -n purchasing-flow --timeout=300s

# Deploy all microservices
kubectl apply -f k8s/deployments/
kubectl apply -f k8s/services/

# Check deployment status
kubectl get all -n purchasing-flow
```

### Step 4: Verify Deployment

```bash
# Check pods status
kubectl get pods -n purchasing-flow

# Check services
kubectl get services -n purchasing-flow

# View logs
kubectl logs -n purchasing-flow -l app=orchestrator-service --tail=50

# Get orchestrator service URL
kubectl get service orchestrator-service -n purchasing-flow
```

### Step 5: Test the Deployment

```bash
# Port-forward for testing (if LoadBalancer not available)
kubectl port-forward -n purchasing-flow service/orchestrator-service 8080:8080

# In another terminal, test the API
curl -X POST http://localhost:8080/purchasing/flow \
  -H "Content-Type: application/json" \
  -d '{"product_id": "PROD-001"}'
```

## Database Initialization

Before first use, initialize the MySQL databases:

```bash
# Connect to MySQL pod
kubectl exec -it -n purchasing-flow $(kubectl get pod -n purchasing-flow -l app=mysql -o jsonpath='{.items[0].metadata.name}') -- mysql -uroot -ppassword

# Run initialization scripts from your original setup
# Or copy the init script:
kubectl cp scripts/init_databases.sql purchasing-flow/mysql-pod:/tmp/init.sql
kubectl exec -n purchasing-flow mysql-pod -- mysql -uroot -ppassword < /tmp/init.sql
```

## Scaling

Scale individual services based on load:

```bash
# Scale rule engine (CPU intensive)
kubectl scale deployment rule-engine-service -n purchasing-flow --replicas=5

# Scale orchestrator (high traffic)
kubectl scale deployment orchestrator-service -n purchasing-flow --replicas=5

# Scale data services
kubectl scale deployment oms-service -n purchasing-flow --replicas=3
kubectl scale deployment inventory-service -n purchasing-flow --replicas=3
```

## Monitoring

### View Logs

```bash
# All services
kubectl logs -n purchasing-flow -l app=orchestrator-service --tail=100 -f

# Specific service
kubectl logs -n purchasing-flow -l app=rule-engine-service --tail=100 -f
```

### Resource Usage

```bash
# Pod resources
kubectl top pods -n purchasing-flow

# Node resources
kubectl top nodes
```

## Troubleshooting

### Service Not Starting

```bash
# Check pod events
kubectl describe pod -n purchasing-flow <pod-name>

# Check logs
kubectl logs -n purchasing-flow <pod-name>

# Check service endpoints
kubectl get endpoints -n purchasing-flow
```

### Database Connection Issues

```bash
# Verify MySQL is running
kubectl get pods -n purchasing-flow -l app=mysql

# Test database connection
kubectl exec -it -n purchasing-flow <oms-service-pod> -- sh
# Inside pod:
# env | grep DB
```

### Service Communication Issues

```bash
# Check service DNS
kubectl run -it --rm debug --image=busybox --restart=Never -n purchasing-flow -- nslookup oms-service

# Check connectivity
kubectl run -it --rm debug --image=curlimages/curl --restart=Never -n purchasing-flow -- curl http://oms-service:8081/health
```

## Configuration

### Environment Variables

Edit ConfigMaps to change service URLs or database names:

```bash
kubectl edit configmap db-config -n purchasing-flow
kubectl edit configmap service-urls -n purchasing-flow
```

### Secrets

Update database credentials:

```bash
kubectl edit secret db-secret -n purchasing-flow
```

After updating, restart affected pods:

```bash
kubectl rollout restart deployment -n purchasing-flow
```

## Cleanup

### Remove All Resources

```bash
# Delete namespace (removes everything)
kubectl delete namespace purchasing-flow

# Or delete selectively
kubectl delete -f k8s/deployments/
kubectl delete -f k8s/services/
kubectl delete -f k8s/configmaps/
kubectl delete -f k8s/secrets/
kubectl delete -f k8s/namespace.yaml
```

### Docker Compose Cleanup

```bash
docker-compose down -v
docker system prune -a
```

## Production Considerations

### Security
1. Change default MySQL password in `k8s/secrets/db-secret.yaml`
2. Use TLS/SSL for inter-service communication
3. Implement API authentication/authorization
4. Use network policies to restrict pod-to-pod traffic
5. Scan container images for vulnerabilities

### High Availability
1. Run multiple replicas of each service
2. Use pod anti-affinity rules
3. Configure horizontal pod autoscaling (HPA)
4. Set up proper resource limits and requests
5. Use readiness and liveness probes

### Persistence
1. Use StatefulSet for MySQL in production
2. Configure proper backup strategies
3. Use persistent volume with appropriate storage class
4. Consider managed database services (RDS, Cloud SQL)

### Observability
1. Implement distributed tracing (Jaeger, Zipkin)
2. Set up metrics collection (Prometheus)
3. Configure log aggregation (ELK, Loki)
4. Create dashboards (Grafana)
5. Set up alerting rules

### Performance
1. Configure connection pooling for databases
2. Implement caching layer (Redis)
3. Use async/non-blocking operations
4. Optimize Docker image sizes
5. Enable compression for API responses

## API Documentation

### Orchestrator Service (Main Entry Point)

**Execute Purchasing Flow**
```http
POST /purchasing/flow
Content-Type: application/json

{
  "product_id": "PROD-001"
}

Response:
{
  "success": true,
  "po": {
    "po_id": "PO-1234567890",
    "product_id": "PROD-001",
    "supplier_id": "SUP-001",
    "qty": 100,
    "unit_price": 12.50,
    "total_amount": 1250.00,
    "status": "sent",
    "created_at": "2025-01-18T10:00:00Z",
    "sent_at": "2025-01-18T10:00:05Z"
  },
  "calculation": {
    "need_reorder": true,
    "shortage": 75.0,
    "order_qty": 100,
    "total_amount": 1250.00,
    "requires_approval": false,
    "approval_status": "auto_approved"
  },
  "message": "Purchasing flow completed successfully"
}
```

### Individual Services

Each service exposes:
- `GET /health` - Health check endpoint

Data services also expose:
- OMS: `GET /oms/history/{product_id}`
- Inventory: `GET /inventory/levels/{product_id}`
- Supplier: `GET /supplier/info/{product_id}`
- UOM: `GET /uom/conversion/{product_id}`

## Support

For issues or questions:
1. Check logs: `kubectl logs -n purchasing-flow <pod-name>`
2. Verify service health: `curl http://<service>:port/health`
3. Check resource usage: `kubectl top pods -n purchasing-flow`
4. Review events: `kubectl get events -n purchasing-flow`

## License

MIT License - See main project README for details.
