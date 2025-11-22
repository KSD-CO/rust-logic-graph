# 🏗️ Architecture Patterns

> Real-world patterns for building distributed reasoning systems with Rust Logic Graph

---

## 📋 Pattern Catalog

| Pattern | Use Case | Complexity |
|---------|----------|------------|
| [Multi-Database Reasoning](#pattern-1-multi-database-reasoning) | Aggregate data from multiple sources | ⭐⭐ |
| [AI Agent with Tools](#pattern-2-ai-agent-with-tool-calling) | LLM orchestration with external tools | ⭐⭐⭐ |
| [Saga Pattern](#pattern-3-saga-pattern-for-distributed-transactions) | Distributed transactions with compensation | ⭐⭐⭐⭐ |
| [RAG Pipeline](#pattern-4-rag-retrieval-augmented-generation) | Vector search + LLM generation | ⭐⭐⭐ |
| [Event-Driven Reasoning](#pattern-5-event-driven-reasoning) | React to events with business rules | ⭐⭐⭐ |
| [Multi-Agent Coordination](#pattern-6-multi-agent-coordination) | Coordinate multiple AI agents | ⭐⭐⭐⭐ |

---

## Pattern 1: Multi-Database Reasoning

### Problem
Your data lives in multiple databases (PostgreSQL, MongoDB, Redis). You need to query all of them, aggregate the data, apply business rules, and make a decision.

### Solution
```rust
use rust_logic_graph::{Graph, NodeConfig, NodeType, Edge};

let graph = Graph::new()
    // Data collection from multiple sources
    .add_node("user_profile", NodeConfig {
        node_type: NodeType::DBNode {
            database: "postgres",
            query: "SELECT * FROM users WHERE id = $1".to_string(),
        },
        ..Default::default()
    })
    .add_node("analytics", NodeConfig {
        node_type: NodeType::DBNode {
            database: "mongodb",
            query: r#"{ "collection": "user_analytics", "filter": { "user_id": "$user_id" } }"#.to_string(),
        },
        ..Default::default()
    })
    .add_node("cache_check", NodeConfig {
        node_type: NodeType::DBNode {
            database: "redis",
            query: "GET recommendation:$user_id".to_string(),
        },
        ..Default::default()
    })
    
    // Apply business rules
    .add_node("recommendation_engine", NodeConfig {
        node_type: NodeType::RuleNode {
            rules_file: "recommendation_rules.grl".to_string(),
        },
        dependencies: vec!["user_profile", "analytics", "cache_check"],
        ..Default::default()
    })
    
    // Decision node
    .add_node("make_decision", NodeConfig {
        node_type: NodeType::ConditionalNode {
            condition: "confidence_score > 0.8".to_string(),
            true_branch: "send_recommendation".to_string(),
            false_branch: "fallback_logic".to_string(),
        },
        dependencies: vec!["recommendation_engine"],
        ..Default::default()
    });
```

### Architecture Diagram
```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│  PostgreSQL │    │   MongoDB   │    │    Redis    │
│  (Users)    │    │ (Analytics) │    │   (Cache)   │
└──────┬──────┘    └──────┬──────┘    └──────┬──────┘
       │                  │                   │
       └──────────────────┴───────────────────┘
                          │
                  ┌───────▼───────┐
                  │  Rule Engine  │◀─── GRL Business Rules
                  │  (Reasoning)  │
                  └───────┬───────┘
                          │
                  ┌───────▼───────┐
                  │ Decision Node │
                  └───────┬───────┘
                          │
            ┌─────────────┴─────────────┐
            │                           │
    ┌───────▼───────┐         ┌────────▼────────┐
    │ Send Recommend│         │ Fallback Logic  │
    └───────────────┘         └─────────────────┘
```

### Benefits
- ✅ Query multiple databases in parallel
- ✅ Apply complex business logic with GRL
- ✅ Type-safe data aggregation
- ✅ Automatic error handling

---

## Pattern 2: AI Agent with Tool Calling

### Problem
Build an AI agent that can:
1. Understand user queries
2. Call external tools (search, database, APIs)
3. Reason over the results
4. Validate with business rules

### Solution
```rust
// Define tool subgraphs
let search_tool = GraphDef::from_yaml("tools/search.yaml")?;
let database_tool = GraphDef::from_yaml("tools/database.yaml")?;
let calculator_tool = GraphDef::from_yaml("tools/calculator.yaml")?;

let agent_graph = Graph::new()
    // Step 1: Understand the query
    .add_node("understand", NodeConfig {
        node_type: NodeType::AINode {
            provider: "claude",
            model: "claude-3-5-sonnet-20241022".to_string(),
            system_prompt: "You are a helpful assistant. Analyze the query and decide which tools to call.".to_string(),
        },
        ..Default::default()
    })
    
    // Step 2: Call tools in parallel
    .add_node("search_knowledge", NodeConfig {
        node_type: NodeType::SubgraphNode {
            graph_def: search_tool,
            input_mapping: vec![("query", "search_query")],
            output_key: "search_results".to_string(),
        },
        dependencies: vec!["understand"],
        ..Default::default()
    })
    .add_node("query_database", NodeConfig {
        node_type: NodeType::SubgraphNode {
            graph_def: database_tool,
            input_mapping: vec![("sql", "db_query")],
            output_key: "db_results".to_string(),
        },
        dependencies: vec!["understand"],
        ..Default::default()
    })
    
    // Step 3: Reason over results
    .add_node("reason", NodeConfig {
        node_type: NodeType::AINode {
            provider: "openai",
            model: "gpt-4".to_string(),
            system_prompt: "Synthesize information from tools and provide answer.".to_string(),
        },
        dependencies: vec!["search_knowledge", "query_database"],
        ..Default::default()
    })
    
    // Step 4: Validate with business rules
    .add_node("validate", NodeConfig {
        node_type: NodeType::RuleNode {
            rules_file: "validation_rules.grl".to_string(),
        },
        dependencies: vec!["reason"],
        ..Default::default()
    })
    
    // Step 5: Retry on validation failure
    .add_retry("reason", RetryConfig {
        max_attempts: 3,
        backoff_ms: 1000,
        exponential: true,
    });
```

### Architecture Diagram
```
┌─────────────┐
│ User Query  │
└──────┬──────┘
       │
┌──────▼────────────┐
│ LLM (Claude)      │
│ Understand Intent │
└──────┬────────────┘
       │
       ├───────────────────────┬───────────────────┐
       │                       │                   │
┌──────▼──────────┐  ┌────────▼────────┐  ┌──────▼──────┐
│ Search Tool     │  │ Database Tool   │  │ Calculator  │
│ (Subgraph)      │  │ (Subgraph)      │  │ (Subgraph)  │
└──────┬──────────┘  └────────┬────────┘  └──────┬──────┘
       │                       │                   │
       └───────────────────────┴───────────────────┘
                               │
                     ┌─────────▼──────────┐
                     │ LLM (GPT-4)        │
                     │ Reason & Synthesize│
                     └─────────┬──────────┘
                               │
                     ┌─────────▼──────────┐
                     │ Validate (GRL)     │
                     │ Business Rules     │
                     └─────────┬──────────┘
                               │
                      ┌────────▼────────┐
                      │ Valid?          │
                      └────────┬────────┘
                               │
                    ┌──────────┴──────────┐
                    │                     │
              ┌─────▼─────┐        ┌─────▼─────┐
              │  Success  │        │  Retry    │
              └───────────┘        └───────────┘
```

### Benefits
- ✅ Multi-step AI reasoning
- ✅ Tool calling with validation
- ✅ Automatic retry logic
- ✅ Business rule compliance

---

## Pattern 3: Saga Pattern for Distributed Transactions

### Problem
Coordinate a transaction across multiple microservices. If any step fails, roll back previous steps with compensation logic.

### Solution
```rust
let order_saga = Graph::new()
    // Step 1: Reserve inventory
    .add_node("reserve_inventory", NodeConfig {
        node_type: NodeType::GrpcNode {
            endpoint: "inventory-service:50051".to_string(),
            method: "ReserveItems".to_string(),
        },
        ..Default::default()
    })
    .add_compensation("reserve_inventory", NodeConfig {
        node_type: NodeType::GrpcNode {
            endpoint: "inventory-service:50051".to_string(),
            method: "ReleaseItems".to_string(),
        },
        ..Default::default()
    })
    
    // Step 2: Charge payment
    .add_node("charge_payment", NodeConfig {
        node_type: NodeType::GrpcNode {
            endpoint: "payment-service:50052".to_string(),
            method: "ChargeCard".to_string(),
        },
        dependencies: vec!["reserve_inventory"],
        ..Default::default()
    })
    .add_compensation("charge_payment", NodeConfig {
        node_type: NodeType::GrpcNode {
            endpoint: "payment-service:50052".to_string(),
            method: "RefundPayment".to_string(),
        },
        ..Default::default()
    })
    
    // Step 3: Create shipment
    .add_node("create_shipment", NodeConfig {
        node_type: NodeType::GrpcNode {
            endpoint: "shipping-service:50053".to_string(),
            method: "CreateShipment".to_string(),
        },
        dependencies: vec!["charge_payment"],
        ..Default::default()
    })
    .add_compensation("create_shipment", NodeConfig {
        node_type: NodeType::GrpcNode {
            endpoint: "shipping-service:50053".to_string(),
            method: "CancelShipment".to_string(),
        },
        ..Default::default()
    })
    
    // Error handling with saga rollback
    .add_saga_error_handler(|error, completed_steps| {
        // Automatically call compensation for completed steps
        for step in completed_steps.iter().rev() {
            step.compensate()?;
        }
        Ok(())
    });
```

### Architecture Diagram
```
┌─────────────┐
│ Order       │
│ Request     │
└──────┬──────┘
       │
┌──────▼────────────┐
│ Reserve Inventory │ ◀── Compensation: Release Inventory
└──────┬────────────┘
       │ Success
┌──────▼────────────┐
│ Charge Payment    │ ◀── Compensation: Refund Payment
└──────┬────────────┘
       │ Success
┌──────▼────────────┐
│ Create Shipment   │ ◀── Compensation: Cancel Shipment
└──────┬────────────┘
       │ Success
┌──────▼────────────┐
│ Complete Order    │
└───────────────────┘

If any step fails:
       │ Failure
┌──────▼────────────┐
│ Execute           │
│ Compensations     │◀── Roll back in reverse order
└───────────────────┘
```

### Benefits
- ✅ Distributed transaction coordination
- ✅ Automatic compensation on failure
- ✅ Fault-tolerant microservices
- ✅ ACID-like guarantees across services

---

## Pattern 4: RAG (Retrieval-Augmented Generation)

### Problem
Build an AI system that:
1. Searches a vector database for relevant documents
2. Generates embeddings for queries
3. Ranks and filters results
4. Uses LLM to generate answers based on retrieved context

### Solution
```rust
let rag_pipeline = Graph::new()
    // Step 1: Generate query embedding
    .add_node("embed_query", NodeConfig {
        node_type: NodeType::AINode {
            provider: "openai",
            model: "text-embedding-ada-002".to_string(),
            ..Default::default()
        },
        ..Default::default()
    })
    
    // Step 2: Vector search
    .add_node("vector_search", NodeConfig {
        node_type: NodeType::DBNode {
            database: "pinecone",
            query: r#"{"top_k": 5, "include_metadata": true}"#.to_string(),
        },
        dependencies: vec!["embed_query"],
        ..Default::default()
    })
    
    // Step 3: Rerank results with business rules
    .add_node("rerank", NodeConfig {
        node_type: NodeType::RuleNode {
            rules_file: "reranking_rules.grl".to_string(),
        },
        dependencies: vec!["vector_search"],
        ..Default::default()
    })
    
    // Step 4: Generate answer with context
    .add_node("generate", NodeConfig {
        node_type: NodeType::AINode {
            provider: "openai",
            model: "gpt-4".to_string(),
            system_prompt: "Use the provided context to answer the question.".to_string(),
        },
        dependencies: vec!["rerank"],
        ..Default::default()
    })
    
    // Step 5: Validate answer
    .add_node("validate_answer", NodeConfig {
        node_type: NodeType::RuleNode {
            rules_file: "answer_validation.grl".to_string(),
        },
        dependencies: vec!["generate"],
        ..Default::default()
    });
```

### Benefits
- ✅ Accurate, grounded AI responses
- ✅ Custom reranking logic
- ✅ Answer validation
- ✅ Modular RAG pipeline

---

## Pattern 5: Event-Driven Reasoning

### Problem
React to events from Kafka/RabbitMQ, apply business rules in real-time, and trigger actions.

### Solution
```rust
let event_processor = Graph::new()
    // Step 1: Consume event
    .add_trigger("kafka_consumer", TriggerConfig {
        source: "kafka://events-topic".to_string(),
        ..Default::default()
    })
    
    // Step 2: Enrich with context
    .add_node("enrich", NodeConfig {
        node_type: NodeType::DBNode {
            database: "postgres",
            query: "SELECT * FROM context WHERE event_id = $1".to_string(),
        },
        ..Default::default()
    })
    
    // Step 3: Apply rules
    .add_node("evaluate_rules", NodeConfig {
        node_type: NodeType::RuleNode {
            rules_file: "event_rules.grl".to_string(),
        },
        dependencies: vec!["enrich"],
        ..Default::default()
    })
    
    // Step 4: Route based on decision
    .add_node("route", NodeConfig {
        node_type: NodeType::ConditionalNode {
            condition: "action_required == true".to_string(),
            true_branch: "trigger_action".to_string(),
            false_branch: "log_event".to_string(),
        },
        dependencies: vec!["evaluate_rules"],
        ..Default::default()
    })
    
    // Circuit breaker for external actions
    .add_circuit_breaker("trigger_action", CircuitBreakerConfig {
        failure_threshold: 5,
        timeout_ms: 60000,
    });
```

### Benefits
- ✅ Real-time event processing
- ✅ Business rule evaluation
- ✅ Fault tolerance with circuit breaker
- ✅ Conditional routing

---

## Pattern 6: Multi-Agent Coordination

### Problem
Coordinate multiple AI agents that:
1. Have specialized roles
2. Share context
3. Make decisions collaboratively
4. Validate each other's outputs

### Solution
```rust
let multi_agent_system = Graph::new()
    // Agent 1: Data Analyst
    .add_node("analyst_agent", NodeConfig {
        node_type: NodeType::AINode {
            provider: "openai",
            model: "gpt-4".to_string(),
            system_prompt: "You are a data analyst. Analyze the data and provide insights.".to_string(),
        },
        ..Default::default()
    })
    
    // Agent 2: Business Strategist
    .add_node("strategist_agent", NodeConfig {
        node_type: NodeType::AINode {
            provider: "claude",
            model: "claude-3-5-sonnet-20241022".to_string(),
            system_prompt: "You are a business strategist. Recommend actions based on analysis.".to_string(),
        },
        dependencies: vec!["analyst_agent"],
        ..Default::default()
    })
    
    // Agent 3: Risk Assessor
    .add_node("risk_agent", NodeConfig {
        node_type: NodeType::AINode {
            provider: "openai",
            model: "gpt-4".to_string(),
            system_prompt: "You are a risk assessor. Evaluate risks of proposed actions.".to_string(),
        },
        dependencies: vec!["strategist_agent"],
        ..Default::default()
    })
    
    // Coordination with business rules
    .add_node("coordination", NodeConfig {
        node_type: NodeType::RuleNode {
            rules_file: "agent_coordination.grl".to_string(),
        },
        dependencies: vec!["analyst_agent", "strategist_agent", "risk_agent"],
        ..Default::default()
    })
    
    // Final decision
    .add_node("final_decision", NodeConfig {
        node_type: NodeType::ConditionalNode {
            condition: "consensus_reached == true && risk_acceptable == true".to_string(),
            true_branch: "execute_action".to_string(),
            false_branch: "escalate_to_human".to_string(),
        },
        dependencies: vec!["coordination"],
        ..Default::default()
    });
```

### Benefits
- ✅ Specialized AI agents
- ✅ Collaborative decision making
- ✅ Risk assessment
- ✅ Human escalation path

---

## 📚 Related Documents

- [Use Cases](USE_CASES.md) - Real-world applications
- [GRL Guide](GRL.md) - Business rule syntax
- [Integrations](INTEGRATIONS.md) - Database & AI integrations
- [Extending Guide](EXTENDING.md) - Custom nodes

---

<div align="center">

**Build distributed reasoning systems with proven patterns**

[Main README](../README.md) • [Documentation](README.md) • [Examples](../examples/)

</div>
