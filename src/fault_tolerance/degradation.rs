use serde_json::Value;

/// Fallback handler type: given node id and context, produce a best-effort result
pub type FallbackHandler = fn(node_id: &str, ctx: &mut crate::core::Context) -> Option<Value>;

/// Simple helper: attempt fallback when node execution fails
pub fn degrade_on_failure(node_id: &str, ctx: &mut crate::core::Context, handler: Option<FallbackHandler>) -> Option<Value> {
    if let Some(h) = handler {
        h(node_id, ctx)
    } else {
        // Default graceful degradation: inject a null result marker
        let key = format!("{}_result", node_id);
        ctx.data.insert(key.clone(), Value::Null);
        Some(Value::Null)
    }
}
