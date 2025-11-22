# 🔌 Integration Guide

Complete guide to database and AI/LLM integrations in Rust Logic Graph.

---

## 📋 Table of Contents

- [Database Integrations](#database-integrations)
  - [PostgreSQL](#postgresql)
  - [MySQL](#mysql)
  - [Redis](#redis)
  - [MongoDB](#mongodb)
- [AI/LLM Integrations](#aillm-integrations)
  - [OpenAI](#openai)
  - [Anthropic Claude](#anthropic-claude)
  - [Ollama](#ollama)
- [Configuration](#configuration)
- [Best Practices](#best-practices)

---

## 🗄️ Database Integrations

### PostgreSQL

PostgreSQL integration with connection pooling and async queries.

#### Features
- ✅ Connection pooling with sqlx
- ✅ Async query execution
- ✅ Transaction support
- ✅ Parameterized queries
- ✅ JSON result conversion

#### Installation

```toml
[dependencies]
rust-logic-graph = { version = "0.2.0", features = ["postgres"] }
```

#### Basic Usage

```rust
use rust_logic_graph::integrations::PostgresNode;

// Create node with query
let node = PostgresNode::new(
    "fetch_users",
    "SELECT id, name, email FROM users WHERE status = 'active'"
);

// Initialize with connection pool
let node = node.with_pool("postgres://user:pass@localhost/mydb").await?;

// Execute
let result = node.run(&mut context).await?;
```

#### Parameterized Queries

```rust
// Use {{variable}} syntax for parameters
let node = PostgresNode::new(
    "fetch_user",
    "SELECT * FROM users WHERE id = {{user_id}}"
);

// Set parameter in context
context.data.insert("user_id", json!(42));

// Execute - SQL will be: SELECT * FROM users WHERE id = 42
let result = node.run(&mut context).await?;
```

#### Result Format

```json
{
  "fetch_users_result": [
    {"id": 1, "name": "Alice", "email": "alice@example.com"},
    {"id": 2, "name": "Bob", "email": "bob@example.com"}
  ],
  "fetch_users_count": 2
}
```

---

### MySQL

MySQL integration with connection pooling.

#### Installation

```toml
[dependencies]
rust-logic-graph = { version = "0.2.0", features = ["mysql"] }
```

#### Basic Usage

```rust
use rust_logic_graph::integrations::MySqlNode;

let node = MySqlNode::new(
    "fetch_orders",
    "SELECT * FROM orders WHERE customer_id = {{customer_id}}"
)
.with_pool("mysql://user:pass@localhost/mydb").await?;

context.data.insert("customer_id", json!(123));
let result = node.run(&mut context).await?;
```

---

### Redis

Redis integration for caching and pub/sub.

#### Features
- ✅ GET/SET/DELETE operations
- ✅ TTL support
- ✅ Async operations
- ✅ JSON value support

#### Installation

```toml
[dependencies]
rust-logic-graph = { version = "0.2.0", features = ["redis-cache"] }
```

#### Basic Usage

```rust
use rust_logic_graph::integrations::RedisNode;

// GET operation
let get_node = RedisNode::get("cache_check", "user:{{user_id}}")
    .with_client("redis://localhost:6379")?;

// SET operation with TTL
let set_node = RedisNode::set(
    "cache_store",
    "user:{{user_id}}",
    "{{user_data}}"
)
.with_ttl(3600) // 1 hour
.with_client("redis://localhost:6379")?;

// Execute
context.data.insert("user_id", json!(42));
let result = get_node.run(&mut context).await?;
```

#### Operations

```rust
// GET - retrieve value
RedisNode::get(id, key)

// SET - store value
RedisNode::set(id, key, value)

// EXISTS - check if key exists
RedisNode::exists(id, key)

// DELETE - remove key
RedisNode::delete(id, key)
```

---

### MongoDB

MongoDB integration for document operations.

#### Features
- ✅ Find/FindOne operations
- ✅ Insert/Update/Delete
- ✅ Aggregation support
- ✅ JSON/BSON conversion

#### Installation

```toml
[dependencies]
rust-logic-graph = { version = "0.2.0", features = ["mongodb-db"] }
```

#### Basic Usage

```rust
use rust_logic_graph::integrations::MongoNode;

// Find documents
let find_node = MongoNode::find(
    "fetch_users",
    "mydb",
    "users",
    r#"{"status": "active"}"#
)
.with_client("mongodb://localhost:27017").await?;

// Find one document
let find_one = MongoNode::find_one(
    "fetch_user",
    "mydb",
    "users",
    r#"{"_id": {"$oid": "{{user_id}}"}}"#
)
.with_client("mongodb://localhost:27017").await?;

// Insert document
let insert_node = MongoNode::insert(
    "create_user",
    "mydb",
    "users",
    r#"{"name": "{{name}}", "email": "{{email}}"}"#
)
.with_client("mongodb://localhost:27017").await?;
```

#### Query Examples

```rust
// Find with filter
r#"{"age": {"$gte": 18}}"#

// Find with multiple conditions
r#"{"status": "active", "role": "admin"}"#

// Update operation
MongoNode::update(
    "update_user",
    "mydb",
    "users",
    r#"{"_id": "{{id}}"}"#,  // filter
    r#"{"$set": {"status": "inactive"}}"#  // update
)
```

---

## 🤖 AI/LLM Integrations

### OpenAI

OpenAI GPT integration with streaming support.

#### Features
- ✅ GPT-4, GPT-3.5 Turbo support
- ✅ System prompts
- ✅ Temperature control
- ✅ Token usage tracking
- ✅ Streaming (future)

#### Installation

```toml
[dependencies]
rust-logic-graph = { version = "0.2.0", features = ["openai"] }
```

#### Basic Usage

```rust
use rust_logic_graph::integrations::OpenAINode;

// GPT-4
let node = OpenAINode::gpt4(
    "analyzer",
    "Analyze the sentiment of: {{text}}"
)
.with_temperature(0.7)
.with_max_tokens(100)
.with_api_key(std::env::var("OPENAI_API_KEY")?);

context.data.insert("text", json!("I love this!"));
let result = node.run(&mut context).await?;
```

#### Available Models

```rust
// GPT-4 (most capable)
OpenAINode::gpt4(id, prompt)

// GPT-4 Turbo (faster)
OpenAINode::gpt4_turbo(id, prompt)

// GPT-3.5 Turbo (cost-effective)
OpenAINode::gpt35_turbo(id, prompt)

// Custom model
OpenAINode::new(id, "model-name", prompt)
```

#### System Prompts

```rust
let node = OpenAINode::gpt4("assistant", "{{question}}")
    .with_system_prompt("You are a Rust programming expert.");
```

#### Result Format

```json
{
  "analyzer_result": {
    "content": "The sentiment is positive...",
    "finish_reason": "stop",
    "usage": {
      "prompt_tokens": 15,
      "completion_tokens": 25,
      "total_tokens": 40
    }
  },
  "analyzer_content": "The sentiment is positive..."
}
```

---

### Anthropic Claude

Anthropic Claude integration for advanced AI tasks.

#### Features
- ✅ Claude 3.5 Sonnet, Opus, Haiku
- ✅ System prompts
- ✅ Temperature control
- ✅ Token usage tracking

#### Installation

```toml
[dependencies]
rust-logic-graph = { version = "0.2.0", features = ["claude"] }
```

#### Basic Usage

```rust
use rust_logic_graph::integrations::ClaudeNode;

// Claude 3.5 Sonnet
let node = ClaudeNode::sonnet_35(
    "writer",
    "Write a short story about {{topic}}"
)
.with_system_prompt("You are a creative writer.")
.with_temperature(0.8)
.with_max_tokens(1000)
.with_api_key(std::env::var("ANTHROPIC_API_KEY")?);

context.data.insert("topic", json!("time travel"));
let result = node.run(&mut context).await?;
```

#### Available Models

```rust
// Claude 3.5 Sonnet (recommended)
ClaudeNode::sonnet_35(id, prompt)

// Claude 3 Opus (most capable)
ClaudeNode::opus(id, prompt)

// Claude 3 Sonnet
ClaudeNode::sonnet(id, prompt)

// Claude 3 Haiku (fastest)
ClaudeNode::haiku(id, prompt)
```

#### Result Format

```json
{
  "writer_result": {
    "content": "In the year 2157...",
    "stop_reason": "end_turn",
    "model": "claude-3-5-sonnet-20241022",
    "usage": {
      "input_tokens": 20,
      "output_tokens": 150
    }
  },
  "writer_content": "In the year 2157..."
}
```

---

### Ollama

Local LLM integration with Ollama.

#### Features
- ✅ Local model execution
- ✅ Multiple model support
- ✅ No API costs
- ✅ Privacy-focused

#### Installation

```toml
[dependencies]
rust-logic-graph = { version = "0.2.0", features = ["ollama"] }
```

#### Basic Usage

```rust
use rust_logic_graph::integrations::OllamaNode;

// Llama 3.1
let node = OllamaNode::llama31(
    "summarizer",
    "Summarize this text: {{text}}"
)
.with_temperature(0.5)
.with_base_url("http://localhost:11434");

context.data.insert("text", json!("Long article..."));
let result = node.run(&mut context).await?;
```

#### Available Models

```rust
// Llama 3.1
OllamaNode::llama31(id, prompt)

// Llama 2
OllamaNode::llama2(id, prompt)

// Mistral
OllamaNode::mistral(id, prompt)

// CodeLlama
OllamaNode::codellama(id, prompt)

// Custom model
OllamaNode::new(id, "model-name", prompt)
```

#### Prerequisites

1. Install Ollama: https://ollama.ai
2. Pull a model: `ollama pull llama3.1`
3. Start Ollama: `ollama serve`

#### Result Format

```json
{
  "summarizer_result": {
    "content": "This text discusses...",
    "model": "llama3.1",
    "done": true,
    "stats": {
      "duration_ms": 1234,
      "eval_count": 50,
      "prompt_eval_count": 100
    }
  },
  "summarizer_content": "This text discusses..."
}
```

---

## ⚙️ Configuration

### Environment Variables

```bash
# Database connections
export DATABASE_URL="postgres://user:pass@localhost/mydb"
export MYSQL_URL="mysql://user:pass@localhost/mydb"
export REDIS_URL="redis://localhost:6379"
export MONGODB_URL="mongodb://localhost:27017"

# AI API keys
export OPENAI_API_KEY="sk-..."
export ANTHROPIC_API_KEY="sk-ant-..."
```

### Feature Flags

Enable only the integrations you need:

```toml
# Single integration
[dependencies]
rust-logic-graph = { version = "0.2.0", features = ["postgres"] }

# Multiple integrations
[dependencies]
rust-logic-graph = { version = "0.2.0", features = ["postgres", "redis-cache", "openai"] }

# All integrations
[dependencies]
rust-logic-graph = { version = "0.2.0", features = ["all-integrations"] }
```

Available features:
- `postgres` - PostgreSQL support
- `mysql` - MySQL support
- `redis-cache` - Redis support
- `mongodb-db` - MongoDB support
- `openai` - OpenAI GPT support
- `claude` - Anthropic Claude support
- `ollama` - Ollama local LLM support
- `all-integrations` - All of the above

---

## 🎯 Best Practices

### Connection Pooling

Always use connection pools for databases:

```rust
// ✅ Good - reuse pool
let pool = PgPool::connect(url).await?;
let node1 = PostgresNode::new("q1", "...").with_pool_ref(&pool)?;
let node2 = PostgresNode::new("q2", "...").with_pool_ref(&pool)?;

// ❌ Bad - creates new connection each time
let node = PostgresNode::new("q1", "...").with_pool(url).await?;
```

### Error Handling

Handle errors gracefully:

```rust
match node.run(&mut context).await {
    Ok(result) => {
        // Process result
    }
    Err(RuleError::Eval(msg)) => {
        // Database/API error
        eprintln!("Operation failed: {}", msg);
    }
}
```

### API Keys

Never hardcode API keys:

```rust
// ✅ Good - from environment
let api_key = std::env::var("OPENAI_API_KEY")?;
let node = OpenAINode::gpt4("ai", "...").with_api_key(api_key);

// ❌ Bad - hardcoded
let node = OpenAINode::gpt4("ai", "...").with_api_key("sk-...");
```

### Context Variables

Use descriptive variable names:

```rust
// ✅ Good
context.data.insert("user_id", json!(42));
context.data.insert("order_total", json!(99.99));

// ❌ Bad
context.data.insert("id", json!(42));
context.data.insert("total", json!(99.99));
```

### Temperature Settings

Choose appropriate temperature values:

```rust
// Factual/deterministic tasks (0.0 - 0.3)
.with_temperature(0.1)  // Data extraction, classification

// Balanced tasks (0.4 - 0.7)
.with_temperature(0.7)  // Q&A, summarization

// Creative tasks (0.8 - 1.0)
.with_temperature(0.9)  // Story writing, brainstorming
```

### Query Optimization

Optimize database queries:

```rust
// ✅ Good - specific columns
"SELECT id, name FROM users LIMIT 100"

// ❌ Bad - all columns
"SELECT * FROM users"

// ✅ Good - indexed columns
"WHERE user_id = {{id}}"

// ❌ Bad - function on column
"WHERE LOWER(email) = {{email}}"
```

---

## 📖 Examples

See the `examples/` directory for complete examples:

- `examples/postgres_flow.rs` - PostgreSQL integration
- `examples/openai_flow.rs` - OpenAI GPT integration
- More examples coming in v0.2.0

---

## 🚀 Next Steps

1. **Read the Use Cases** - See [USE_CASES.md](USE_CASES.md) for real-world examples
2. **Check the API Docs** - Run `cargo doc --open --features all-integrations`
3. **Try Examples** - Run examples with `cargo run --example <name> --features <feature>`
4. **Join Community** - GitHub Discussions for help

---

## 📊 Integration Matrix

| Integration | Status | Version | Documentation |
|-------------|--------|---------|---------------|
| PostgreSQL  | ✅ Complete | 0.2.0 | [Docs](#postgresql) |
| MySQL       | ✅ Complete | 0.2.0 | [Docs](#mysql) |
| Redis       | ✅ Complete | 0.2.0 | [Docs](#redis) |
| MongoDB     | ✅ Complete | 0.2.0 | [Docs](#mongodb) |
| OpenAI      | ✅ Complete | 0.2.0 | [Docs](#openai) |
| Claude      | ✅ Complete | 0.2.0 | [Docs](#claude) |
| Ollama      | ✅ Complete | 0.2.0 | [Docs](#ollama) |

---

<div align="center">

**Need help?** [Open an issue](https://github.com/KSD-CO/rust-logic-graph/issues)

[Back to Main README](../README.md) • [Documentation Index](README.md)

</div>
