# 🚀 Kubernetes Deployment Guide

Complete guide for deploying rust-logic-graph on Kubernetes with production-ready architecture.

---

## 📋 Table of Contents

- [Architecture Overview](#architecture-overview)
- [Deployment Strategies](#deployment-strategies)
- [Recommended Architecture](#recommended-architecture)
- [Configuration](#configuration)
- [Scaling Strategies](#scaling-strategies)
- [Monitoring & Observability](#monitoring--observability)
- [State Management](#state-management)
- [Best Practices](#best-practices)

---

## 🏗️ Architecture Overview

### Three Deployment Approaches

#### 1. Monolithic Approach (Simple)

**When to use:** Small to medium workloads, simple deployment, getting started

```
┌─────────────────────────────────┐
│   K8s Pod (rust-logic-graph)    │
│   ┌─────────────────────────┐   │
│   │  REST/gRPC API Server   │   │
│   │         ↓               │   │
│   │   Graph Executor        │   │
│   │   (All node types)      │   │
│   └─────────────────────────┘   │
└─────────────────────────────────┘
```

**Pros:**
- Simple deployment
- Easy to debug
- Low operational overhead

**Cons:**
- Cannot scale node types independently
- Resource inefficient
- Limited flexibility

---

#### 2. Microservices Approach (Recommended)

**When to use:** Production workloads, need independent scaling, large deployments

```
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│ API Gateway  │───▶│ Orchestrator │───▶│ Node Workers │
│   Service    │    │   Service    │    │  (DB/AI/Rule)│
└──────────────┘    └──────────────┘    └──────────────┘
      ▲                    │                    │
      │                    ▼                    ▼
      │              ┌──────────┐        ┌──────────┐
      └──────────────│  Redis   │◀───────│  Queue   │
                     │ (State)  │        │ (RabbitMQ)│
                     └──────────┘        └──────────┘
```

**Pros:**
- Independent scaling per node type
- Resource optimization
- Fault isolation
- Flexible deployment

**Cons:**
- More complex setup
- Requires message queue
- Higher operational overhead

---

#### 3. Serverless Approach (Cost-Optimized)

**When to use:** Bursty workloads, cost optimization, variable traffic

```
┌─────────────────────────────────────┐
│   Knative Serving / KEDA            │
│   ┌──────────────────────────────┐  │
│   │  logic-graph-executor        │  │
│   │  Scale to zero when idle     │  │
│   │  Scale up on demand          │  │
│   └──────────────────────────────┘  │
└─────────────────────────────────────┘
```

**Pros:**
- Cost efficient (scale to zero)
- Auto-scaling based on demand
- No idle resource waste

**Cons:**
- Cold start latency
- Requires Knative/KEDA setup
- Complex networking

---

## 🎯 Recommended Architecture

The **Microservices Approach** is recommended for production deployments.

### Components

#### 1. API Gateway Service

**Responsibility:** Accept graph definitions and execution requests

**Resources:**
- CPU: 500m (request) - 2 cores (limit)
- Memory: 512Mi (request) - 2Gi (limit)
- Replicas: 3-5

**Endpoints:**
- `POST /api/v1/graphs` - Submit graph definition
- `POST /api/v1/graphs/{id}/execute` - Execute graph
- `GET /api/v1/graphs/{id}/status` - Get execution status
- `GET /api/v1/graphs/{id}/results` - Get execution results

---

#### 2. Orchestrator Service

**Responsibility:** Manage graph execution, topological sorting, task distribution

**Resources:**
- CPU: 1 core (request) - 4 cores (limit)
- Memory: 1Gi (request) - 4Gi (limit)
- Replicas: 2-3

**Functions:**
- Parse graph definition
- Perform topological sort
- Identify parallel execution opportunities
- Distribute tasks to worker queues
- Aggregate results
- Handle failures and retries

---

#### 3. Worker Services

##### DB Worker Pool

**Responsibility:** Execute database nodes (PostgreSQL, MySQL, Redis, MongoDB)

**Resources:**
- CPU: 200m (request) - 1 core (limit)
- Memory: 256Mi (request) - 1Gi (limit)
- Replicas: 5-20 (auto-scale)

**Node Types:** `PostgresNode`, `MySQLNode`, `RedisNode`, `MongoNode`

##### AI Worker Pool

**Responsibility:** Execute AI/LLM nodes (OpenAI, Claude, Ollama)

**Resources:**
- CPU: 1 core (request) - 4 cores (limit)
- Memory: 1Gi (request) - 4Gi (limit)
- Replicas: 2-10 (auto-scale, expensive!)

**Node Types:** `OpenAINode`, `ClaudeNode`, `OllamaNode`

##### Rule Worker Pool

**Responsibility:** Execute rule evaluation nodes

**Resources:**
- CPU: 500m (request) - 2 cores (limit)
- Memory: 512Mi (request) - 2Gi (limit)
- Replicas: 3-15 (auto-scale)

**Node Types:** `RuleNode`, `GrlNode`

---

#### 4. State Management

##### Redis (In-Memory State)

**Purpose:** Fast access to execution state, node results, cache

**Resources:**
- CPU: 1 core (request) - 2 cores (limit)
- Memory: 2Gi (request) - 8Gi (limit)
- Replicas: 3 (Redis cluster)

**Data:**
- Execution status
- Node results (temporary)
- Cache layer
- Rate limiting

##### PostgreSQL (Persistent State)

**Purpose:** Durable storage for graph definitions, execution history

**Resources:**
- CPU: 2 cores (request) - 4 cores (limit)
- Memory: 4Gi (request) - 16Gi (limit)
- Storage: 100Gi PVC

**Schema:**
```sql
CREATE TABLE graph_definitions (
  id UUID PRIMARY KEY,
  name VARCHAR(255),
  definition JSONB NOT NULL,
  created_at TIMESTAMP DEFAULT NOW(),
  updated_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE graph_executions (
  id UUID PRIMARY KEY,
  graph_id UUID REFERENCES graph_definitions(id),
  status VARCHAR(20) NOT NULL, -- pending, running, completed, failed
  input_context JSONB,
  results JSONB,
  error TEXT,
  started_at TIMESTAMP,
  completed_at TIMESTAMP,
  duration_ms INTEGER
);

CREATE TABLE node_executions (
  id UUID PRIMARY KEY,
  execution_id UUID REFERENCES graph_executions(id),
  node_id VARCHAR(255) NOT NULL,
  node_type VARCHAR(50) NOT NULL,
  status VARCHAR(20) NOT NULL,
  input JSONB,
  output JSONB,
  error TEXT,
  started_at TIMESTAMP,
  completed_at TIMESTAMP,
  duration_ms INTEGER
);

CREATE INDEX idx_executions_graph_id ON graph_executions(graph_id);
CREATE INDEX idx_executions_status ON graph_executions(status);
CREATE INDEX idx_node_executions_execution_id ON node_executions(execution_id);
```

---

#### 5. Message Queue

##### RabbitMQ

**Purpose:** Task distribution, work queue, event streaming

**Resources:**
- CPU: 500m (request) - 2 cores (limit)
- Memory: 1Gi (request) - 4Gi (limit)
- Replicas: 3 (cluster)

**Queues:**
- `graph.orchestration` - Graph execution requests
- `node.db.tasks` - Database node tasks
- `node.ai.tasks` - AI node tasks
- `node.rule.tasks` - Rule node tasks
- `results.aggregation` - Result collection

---

## 📦 Configuration

### Kubernetes Manifests

#### Namespace

```yaml
# namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: logic-graph
  labels:
    app: logic-graph
```

---

#### API Gateway Deployment

```yaml
# api-gateway-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: logic-graph-api
  namespace: logic-graph
  labels:
    app: logic-graph
    component: api-gateway
spec:
  replicas: 3
  selector:
    matchLabels:
      app: logic-graph
      component: api-gateway
  template:
    metadata:
      labels:
        app: logic-graph
        component: api-gateway
    spec:
      containers:
      - name: api-gateway
        image: logic-graph:0.3.0
        imagePullPolicy: Always
        ports:
        - containerPort: 8080
          name: http
          protocol: TCP
        - containerPort: 9090
          name: metrics
          protocol: TCP
        env:
        - name: MODE
          value: "api-gateway"
        - name: RUST_LOG
          value: "info"
        - name: REDIS_URL
          valueFrom:
            secretKeyRef:
              name: redis-secret
              key: url
        - name: POSTGRES_URL
          valueFrom:
            secretKeyRef:
              name: postgres-secret
              key: url
        - name: RABBITMQ_URL
          valueFrom:
            secretKeyRef:
              name: rabbitmq-secret
              key: url
        resources:
          requests:
            cpu: 500m
            memory: 512Mi
          limits:
            cpu: 2000m
            memory: 2Gi
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 5

---
apiVersion: v1
kind: Service
metadata:
  name: logic-graph-api
  namespace: logic-graph
  labels:
    app: logic-graph
    component: api-gateway
spec:
  type: LoadBalancer
  ports:
  - port: 80
    targetPort: 8080
    protocol: TCP
    name: http
  - port: 9090
    targetPort: 9090
    protocol: TCP
    name: metrics
  selector:
    app: logic-graph
    component: api-gateway
```

---

#### Orchestrator Deployment

```yaml
# orchestrator-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: logic-graph-orchestrator
  namespace: logic-graph
  labels:
    app: logic-graph
    component: orchestrator
spec:
  replicas: 2
  selector:
    matchLabels:
      app: logic-graph
      component: orchestrator
  template:
    metadata:
      labels:
        app: logic-graph
        component: orchestrator
    spec:
      containers:
      - name: orchestrator
        image: logic-graph:0.3.0
        imagePullPolicy: Always
        ports:
        - containerPort: 9090
          name: metrics
          protocol: TCP
        env:
        - name: MODE
          value: "orchestrator"
        - name: RUST_LOG
          value: "info,rust_logic_graph=debug"
        - name: REDIS_URL
          valueFrom:
            secretKeyRef:
              name: redis-secret
              key: url
        - name: POSTGRES_URL
          valueFrom:
            secretKeyRef:
              name: postgres-secret
              key: url
        - name: RABBITMQ_URL
          valueFrom:
            secretKeyRef:
              name: rabbitmq-secret
              key: url
        resources:
          requests:
            cpu: 1000m
            memory: 1Gi
          limits:
            cpu: 4000m
            memory: 4Gi
        livenessProbe:
          httpGet:
            path: /health
            port: 9090
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 9090
          initialDelaySeconds: 10
          periodSeconds: 5

---
apiVersion: v1
kind: Service
metadata:
  name: logic-graph-orchestrator
  namespace: logic-graph
  labels:
    app: logic-graph
    component: orchestrator
spec:
  type: ClusterIP
  ports:
  - port: 9090
    targetPort: 9090
    protocol: TCP
    name: metrics
  selector:
    app: logic-graph
    component: orchestrator
```

---

#### DB Worker Deployment

```yaml
# db-worker-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: logic-graph-db-worker
  namespace: logic-graph
  labels:
    app: logic-graph
    component: db-worker
spec:
  replicas: 5
  selector:
    matchLabels:
      app: logic-graph
      component: db-worker
  template:
    metadata:
      labels:
        app: logic-graph
        component: db-worker
    spec:
      containers:
      - name: db-worker
        image: logic-graph:0.3.0
        imagePullPolicy: Always
        ports:
        - containerPort: 9090
          name: metrics
          protocol: TCP
        env:
        - name: MODE
          value: "worker"
        - name: WORKER_TYPE
          value: "database"
        - name: RUST_LOG
          value: "info"
        - name: REDIS_URL
          valueFrom:
            secretKeyRef:
              name: redis-secret
              key: url
        - name: POSTGRES_URL
          valueFrom:
            secretKeyRef:
              name: postgres-secret
              key: url
        - name: RABBITMQ_URL
          valueFrom:
            secretKeyRef:
              name: rabbitmq-secret
              key: url
        resources:
          requests:
            cpu: 200m
            memory: 256Mi
          limits:
            cpu: 1000m
            memory: 1Gi
        livenessProbe:
          httpGet:
            path: /health
            port: 9090
          initialDelaySeconds: 30
          periodSeconds: 10
```

---

#### AI Worker Deployment

```yaml
# ai-worker-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: logic-graph-ai-worker
  namespace: logic-graph
  labels:
    app: logic-graph
    component: ai-worker
spec:
  replicas: 2
  selector:
    matchLabels:
      app: logic-graph
      component: ai-worker
  template:
    metadata:
      labels:
        app: logic-graph
        component: ai-worker
    spec:
      containers:
      - name: ai-worker
        image: logic-graph:0.3.0
        imagePullPolicy: Always
        ports:
        - containerPort: 9090
          name: metrics
          protocol: TCP
        env:
        - name: MODE
          value: "worker"
        - name: WORKER_TYPE
          value: "ai"
        - name: RUST_LOG
          value: "info"
        - name: REDIS_URL
          valueFrom:
            secretKeyRef:
              name: redis-secret
              key: url
        - name: RABBITMQ_URL
          valueFrom:
            secretKeyRef:
              name: rabbitmq-secret
              key: url
        - name: OPENAI_API_KEY
          valueFrom:
            secretKeyRef:
              name: openai-secret
              key: api-key
        - name: ANTHROPIC_API_KEY
          valueFrom:
            secretKeyRef:
              name: anthropic-secret
              key: api-key
        resources:
          requests:
            cpu: 1000m
            memory: 1Gi
          limits:
            cpu: 4000m
            memory: 4Gi
        livenessProbe:
          httpGet:
            path: /health
            port: 9090
          initialDelaySeconds: 30
          periodSeconds: 10
```

---

#### Rule Worker Deployment

```yaml
# rule-worker-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: logic-graph-rule-worker
  namespace: logic-graph
  labels:
    app: logic-graph
    component: rule-worker
spec:
  replicas: 3
  selector:
    matchLabels:
      app: logic-graph
      component: rule-worker
  template:
    metadata:
      labels:
        app: logic-graph
        component: rule-worker
    spec:
      containers:
      - name: rule-worker
        image: logic-graph:0.3.0
        imagePullPolicy: Always
        ports:
        - containerPort: 9090
          name: metrics
          protocol: TCP
        env:
        - name: MODE
          value: "worker"
        - name: WORKER_TYPE
          value: "rule"
        - name: RUST_LOG
          value: "info"
        - name: REDIS_URL
          valueFrom:
            secretKeyRef:
              name: redis-secret
              key: url
        - name: RABBITMQ_URL
          valueFrom:
            secretKeyRef:
              name: rabbitmq-secret
              key: url
        resources:
          requests:
            cpu: 500m
            memory: 512Mi
          limits:
            cpu: 2000m
            memory: 2Gi
        livenessProbe:
          httpGet:
            path: /health
            port: 9090
          initialDelaySeconds: 30
          periodSeconds: 10
```

---

## ⚡ Scaling Strategies

### Horizontal Pod Autoscaler (HPA)

#### API Gateway HPA

```yaml
# api-gateway-hpa.yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: logic-graph-api-hpa
  namespace: logic-graph
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: logic-graph-api
  minReplicas: 3
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
  behavior:
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
      - type: Percent
        value: 50
        periodSeconds: 60
    scaleUp:
      stabilizationWindowSeconds: 0
      policies:
      - type: Percent
        value: 100
        periodSeconds: 30
      - type: Pods
        value: 2
        periodSeconds: 30
      selectPolicy: Max
```

---

#### DB Worker HPA

```yaml
# db-worker-hpa.yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: logic-graph-db-worker-hpa
  namespace: logic-graph
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: logic-graph-db-worker
  minReplicas: 5
  maxReplicas: 20
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Pods
    pods:
      metric:
        name: rabbitmq_queue_messages
      target:
        type: AverageValue
        averageValue: "10"
```

---

#### AI Worker HPA

```yaml
# ai-worker-hpa.yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: logic-graph-ai-worker-hpa
  namespace: logic-graph
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: logic-graph-ai-worker
  minReplicas: 2
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 60
  - type: Pods
    pods:
      metric:
        name: rabbitmq_queue_messages
      target:
        type: AverageValue
        averageValue: "5"
```

---

### KEDA ScaledObject (Advanced)

For more sophisticated scaling based on queue depth:

```yaml
# keda-scaledobject.yaml
apiVersion: keda.sh/v1alpha1
kind: ScaledObject
metadata:
  name: logic-graph-db-worker-scaler
  namespace: logic-graph
spec:
  scaleTargetRef:
    name: logic-graph-db-worker
  minReplicaCount: 5
  maxReplicaCount: 30
  pollingInterval: 15
  cooldownPeriod: 300
  triggers:
  - type: rabbitmq
    metadata:
      protocol: auto
      queueName: node.db.tasks
      mode: QueueLength
      value: "10"
      activationValue: "5"
      hostFromEnv: RABBITMQ_URL
```

---

## 📊 Monitoring & Observability

### Prometheus ServiceMonitor

```yaml
# servicemonitor.yaml
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: logic-graph-metrics
  namespace: logic-graph
  labels:
    app: logic-graph
spec:
  selector:
    matchLabels:
      app: logic-graph
  endpoints:
  - port: metrics
    interval: 30s
    path: /metrics
```

---

### Key Metrics to Monitor

#### Application Metrics

```rust
// Metrics to expose via /metrics endpoint

// Graph execution metrics
graph_executions_total{status="completed|failed|timeout"}
graph_execution_duration_seconds{status="completed|failed"}
graph_nodes_executed_total{node_type="rule|db|ai"}

// Node execution metrics
node_execution_duration_seconds{node_type, status}
node_execution_errors_total{node_type, error_type}

// Queue metrics
queue_depth{queue_name}
queue_processing_rate{queue_name}
queue_age_seconds{queue_name}

// Worker metrics
worker_tasks_processed_total{worker_type, status}
worker_active_tasks{worker_type}
worker_idle_time_seconds{worker_type}

// Cache metrics
cache_hits_total{cache_type="redis"}
cache_misses_total{cache_type="redis"}
cache_size_bytes{cache_type="redis"}
```

---

### Grafana Dashboard

Key panels to include:

1. **Graph Execution Overview**
   - Total executions (counter)
   - Success rate (gauge)
   - Average execution time (graph)
   - Concurrent executions (gauge)

2. **Node Performance**
   - Execution time by node type (heatmap)
   - Error rate by node type (graph)
   - Throughput by node type (graph)

3. **Worker Health**
   - Active workers by type (gauge)
   - Task processing rate (graph)
   - Queue depth (graph)
   - Worker CPU/Memory (graph)

4. **Infrastructure**
   - Pod CPU/Memory usage (graph)
   - Redis connections/operations (graph)
   - Database connections/queries (graph)
   - RabbitMQ queue depth (graph)

---

### Logging with Loki

```yaml
# promtail-configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: promtail-config
  namespace: logic-graph
data:
  promtail.yaml: |
    server:
      http_listen_port: 3101

    clients:
      - url: http://loki:3100/loki/api/v1/push

    scrape_configs:
      - job_name: kubernetes-pods
        kubernetes_sd_configs:
          - role: pod
            namespaces:
              names:
                - logic-graph
        relabel_configs:
          - source_labels: [__meta_kubernetes_pod_label_app]
            target_label: app
          - source_labels: [__meta_kubernetes_pod_label_component]
            target_label: component
          - source_labels: [__meta_kubernetes_pod_name]
            target_label: pod
```

---

## 🗄️ State Management

### Redis Configuration

```yaml
# redis-deployment.yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: redis
  namespace: logic-graph
spec:
  serviceName: redis
  replicas: 3
  selector:
    matchLabels:
      app: redis
  template:
    metadata:
      labels:
        app: redis
    spec:
      containers:
      - name: redis
        image: redis:7-alpine
        ports:
        - containerPort: 6379
          name: redis
        resources:
          requests:
            cpu: 1000m
            memory: 2Gi
          limits:
            cpu: 2000m
            memory: 8Gi
        volumeMounts:
        - name: redis-data
          mountPath: /data
  volumeClaimTemplates:
  - metadata:
      name: redis-data
    spec:
      accessModes: [ "ReadWriteOnce" ]
      resources:
        requests:
          storage: 20Gi
```

---

### PostgreSQL Configuration

```yaml
# postgres-deployment.yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: postgres
  namespace: logic-graph
spec:
  serviceName: postgres
  replicas: 1
  selector:
    matchLabels:
      app: postgres
  template:
    metadata:
      labels:
        app: postgres
    spec:
      containers:
      - name: postgres
        image: postgres:15-alpine
        ports:
        - containerPort: 5432
          name: postgres
        env:
        - name: POSTGRES_DB
          value: logic_graph
        - name: POSTGRES_USER
          valueFrom:
            secretKeyRef:
              name: postgres-secret
              key: username
        - name: POSTGRES_PASSWORD
          valueFrom:
            secretKeyRef:
              name: postgres-secret
              key: password
        resources:
          requests:
            cpu: 2000m
            memory: 4Gi
          limits:
            cpu: 4000m
            memory: 16Gi
        volumeMounts:
        - name: postgres-data
          mountPath: /var/lib/postgresql/data
  volumeClaimTemplates:
  - metadata:
      name: postgres-data
    spec:
      accessModes: [ "ReadWriteOnce" ]
      resources:
        requests:
          storage: 100Gi
```

---

## 🔐 Secrets Management

### Create Secrets

```bash
# Redis secret
kubectl create secret generic redis-secret \
  --from-literal=url='redis://redis:6379' \
  -n logic-graph

# PostgreSQL secret
kubectl create secret generic postgres-secret \
  --from-literal=url='postgresql://user:pass@postgres:5432/logic_graph' \
  --from-literal=username='user' \
  --from-literal=password='pass' \
  -n logic-graph

# RabbitMQ secret
kubectl create secret generic rabbitmq-secret \
  --from-literal=url='amqp://user:pass@rabbitmq:5672' \
  -n logic-graph

# OpenAI secret
kubectl create secret generic openai-secret \
  --from-literal=api-key='sk-...' \
  -n logic-graph

# Anthropic secret
kubectl create secret generic anthropic-secret \
  --from-literal=api-key='sk-ant-...' \
  -n logic-graph
```

---

## 🚀 Deployment Steps

### 1. Create Namespace

```bash
kubectl apply -f namespace.yaml
```

### 2. Deploy State Management

```bash
# Deploy Redis
kubectl apply -f redis-deployment.yaml
kubectl apply -f redis-service.yaml

# Deploy PostgreSQL
kubectl apply -f postgres-deployment.yaml
kubectl apply -f postgres-service.yaml

# Deploy RabbitMQ
kubectl apply -f rabbitmq-deployment.yaml
kubectl apply -f rabbitmq-service.yaml

# Wait for ready
kubectl wait --for=condition=ready pod -l app=redis -n logic-graph --timeout=300s
kubectl wait --for=condition=ready pod -l app=postgres -n logic-graph --timeout=300s
kubectl wait --for=condition=ready pod -l app=rabbitmq -n logic-graph --timeout=300s
```

### 3. Create Secrets

```bash
kubectl create secret generic redis-secret --from-literal=url='redis://redis:6379' -n logic-graph
kubectl create secret generic postgres-secret --from-literal=url='postgresql://user:pass@postgres:5432/logic_graph' -n logic-graph
kubectl create secret generic rabbitmq-secret --from-literal=url='amqp://user:pass@rabbitmq:5672' -n logic-graph
kubectl create secret generic openai-secret --from-literal=api-key='your-key' -n logic-graph
kubectl create secret generic anthropic-secret --from-literal=api-key='your-key' -n logic-graph
```

### 4. Initialize Database

```bash
# Port-forward to PostgreSQL
kubectl port-forward svc/postgres 5432:5432 -n logic-graph &

# Run migrations
psql postgresql://user:pass@localhost:5432/logic_graph < migrations/init.sql
```

### 5. Deploy Services

```bash
# Deploy Orchestrator
kubectl apply -f orchestrator-deployment.yaml

# Deploy Workers
kubectl apply -f db-worker-deployment.yaml
kubectl apply -f ai-worker-deployment.yaml
kubectl apply -f rule-worker-deployment.yaml

# Deploy API Gateway
kubectl apply -f api-gateway-deployment.yaml

# Wait for ready
kubectl wait --for=condition=ready pod -l component=orchestrator -n logic-graph --timeout=300s
kubectl wait --for=condition=ready pod -l component=api-gateway -n logic-graph --timeout=300s
```

### 6. Deploy Autoscaling

```bash
kubectl apply -f api-gateway-hpa.yaml
kubectl apply -f db-worker-hpa.yaml
kubectl apply -f ai-worker-hpa.yaml
```

### 7. Verify Deployment

```bash
# Check all pods
kubectl get pods -n logic-graph

# Check services
kubectl get svc -n logic-graph

# Check HPA
kubectl get hpa -n logic-graph

# Check logs
kubectl logs -f deployment/logic-graph-api -n logic-graph
```

---

## 🎯 Best Practices

### 1. Resource Limits

Always set both requests and limits:
- **Requests:** Guaranteed resources
- **Limits:** Maximum resources (prevents noisy neighbor)

### 2. Probes

Configure health checks:
- **Liveness:** Restart unhealthy pods
- **Readiness:** Remove from service until ready
- **Startup:** Allow slow-starting pods

### 3. Pod Disruption Budgets

Ensure availability during updates:

```yaml
apiVersion: policy/v1
kind: PodDisruptionBudget
metadata:
  name: logic-graph-api-pdb
  namespace: logic-graph
spec:
  minAvailable: 2
  selector:
    matchLabels:
      app: logic-graph
      component: api-gateway
```

### 4. Network Policies

Restrict traffic between pods:

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: logic-graph-network-policy
  namespace: logic-graph
spec:
  podSelector:
    matchLabels:
      app: logic-graph
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - podSelector:
        matchLabels:
          app: logic-graph
  egress:
  - to:
    - podSelector:
        matchLabels:
          app: logic-graph
  - to:
    - podSelector:
        matchLabels:
          app: redis
  - to:
    - podSelector:
        matchLabels:
          app: postgres
```

### 5. Monitoring

- Monitor all components
- Set up alerts for critical metrics
- Use distributed tracing (Jaeger/Zipkin)
- Aggregate logs centrally (Loki/ELK)

### 6. Security

- Use RBAC for service accounts
- Encrypt secrets (use Sealed Secrets or external secret managers)
- Scan images for vulnerabilities
- Use network policies
- Enable audit logging

### 7. Cost Optimization

- Use HPA to scale down during low traffic
- Use node affinity to optimize instance types
- Consider spot instances for non-critical workers
- Use KEDA for scale-to-zero capabilities

---

## 📚 Related Documentation

- [Architecture Overview](../README.md)
- [ROADMAP](../ROADMAP.md)
- [Integrations Guide](INTEGRATIONS.md)
- [Extending Guide](EXTENDING.md)

---

## 🔗 External Resources

- [Kubernetes Documentation](https://kubernetes.io/docs/)
- [KEDA Documentation](https://keda.sh/)
- [Prometheus Operator](https://prometheus-operator.dev/)
- [Grafana Documentation](https://grafana.com/docs/)

---

<div align="center">

**Ready for production deployment!**

[Back to README](../README.md) • [View Examples](../examples/) • [API Documentation](API.md)

</div>
