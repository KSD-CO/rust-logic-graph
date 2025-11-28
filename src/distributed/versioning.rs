//! Context Versioning and Conflict Resolution
//!
//! Provides version tracking and conflict resolution strategies for distributed contexts.

use crate::distributed::context::{DistributedContext, ContextSnapshot};
use serde::{Serialize, Deserialize};
use anyhow::{Result, bail};

/// Version information for a context
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContextVersion {
    /// Version number
    pub version: u64,
    
    /// Timestamp of this version
    pub timestamp: u64,
    
    /// Service that created this version
    pub created_by: Option<String>,
    
    /// Parent version (for tracking lineage)
    pub parent_version: Option<u64>,
}

impl ContextVersion {
    /// Create a new version
    pub fn new(version: u64) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        Self {
            version,
            timestamp,
            created_by: None,
            parent_version: None,
        }
    }
    
    /// Create a new version with parent
    pub fn with_parent(version: u64, parent: u64) -> Self {
        let mut v = Self::new(version);
        v.parent_version = Some(parent);
        v
    }
}

/// Conflict resolution strategies
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictResolution {
    /// Last write wins (based on timestamp)
    LastWriteWins,
    
    /// Higher version wins
    HigherVersionWins,
    
    /// Fail on conflict (require manual resolution)
    FailOnConflict,
    
    /// Merge all changes (may cause data loss)
    MergeAll,
}

/// A versioned context with history tracking
#[derive(Debug, Clone)]
pub struct VersionedContext {
    /// Current context
    pub current: DistributedContext,
    
    /// Version history (limited to last N versions)
    pub history: Vec<ContextSnapshot>,
    
    /// Maximum history size
    pub max_history: usize,
    
    /// Conflict resolution strategy
    pub resolution_strategy: ConflictResolution,
}

impl VersionedContext {
    /// Create a new versioned context
    pub fn new(session_id: impl Into<String>) -> Self {
        Self {
            current: DistributedContext::new(session_id),
            history: Vec::new(),
            max_history: 10,
            resolution_strategy: ConflictResolution::LastWriteWins,
        }
    }
    
    /// Create with custom settings
    pub fn with_config(
        session_id: impl Into<String>,
        max_history: usize,
        strategy: ConflictResolution,
    ) -> Self {
        Self {
            current: DistributedContext::new(session_id),
            history: Vec::new(),
            max_history,
            resolution_strategy: strategy,
        }
    }
    
    /// Update the context and save to history
    pub fn update(&mut self, new_context: DistributedContext) -> Result<()> {
        // Save current state to history
        let snapshot = self.current.snapshot();
        self.history.push(snapshot);
        
        // Trim history if needed
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }
        
        self.current = new_context;
        Ok(())
    }
    
    /// Merge another context with conflict resolution
    pub fn merge_with_resolution(&mut self, other: &DistributedContext) -> Result<()> {
        match self.resolution_strategy {
            ConflictResolution::LastWriteWins => {
                self.merge_last_write_wins(other)
            }
            ConflictResolution::HigherVersionWins => {
                self.merge_higher_version_wins(other)
            }
            ConflictResolution::FailOnConflict => {
                self.merge_fail_on_conflict(other)
            }
            ConflictResolution::MergeAll => {
                self.merge_all(other)
            }
        }
    }
    
    fn merge_last_write_wins(&mut self, other: &DistributedContext) -> Result<()> {
        // Compare timestamps
        if other.metadata.updated_at > self.current.metadata.updated_at {
            self.update(other.clone())?;
        }
        Ok(())
    }
    
    fn merge_higher_version_wins(&mut self, other: &DistributedContext) -> Result<()> {
        if other.metadata.version > self.current.metadata.version {
            self.update(other.clone())?;
        }
        Ok(())
    }
    
    fn merge_fail_on_conflict(&mut self, other: &DistributedContext) -> Result<()> {
        // Check if versions diverged
        if self.current.metadata.version != other.metadata.version {
            bail!(
                "Version conflict: current={}, other={}. Manual resolution required.",
                self.current.metadata.version,
                other.metadata.version
            );
        }
        
        self.update(other.clone())?;
        Ok(())
    }
    
    fn merge_all(&mut self, other: &DistributedContext) -> Result<()> {
        // Merge all fields from other into current
        self.current.merge(other);
        
        // Save snapshot
        let snapshot = self.current.snapshot();
        self.history.push(snapshot);
        
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }
        
        Ok(())
    }
    
    /// Get a specific version from history
    pub fn get_version(&self, version: u64) -> Option<&ContextSnapshot> {
        self.history.iter().find(|s| s.version == version)
    }
    
    /// Rollback to a previous version
    pub fn rollback_to(&mut self, version: u64) -> Result<()> {
        let snapshot = self.get_version(version)
            .ok_or_else(|| anyhow::anyhow!("Version {} not found in history", version))?;
        
        // Reconstruct context from snapshot
        let mut new_context = DistributedContext::new(&snapshot.session_id);
        new_context.data = snapshot.data.clone();
        new_context.metadata.version = snapshot.version + 1; // Increment version
        
        self.update(new_context)?;
        Ok(())
    }
    
    /// Get version history
    pub fn get_history(&self) -> &[ContextSnapshot] {
        &self.history
    }
    
    /// Clear history
    pub fn clear_history(&mut self) {
        self.history.clear();
    }
}

/// Three-way merge for complex conflict resolution
pub struct ThreeWayMerge {
    /// Base version (common ancestor)
    pub base: ContextSnapshot,
    
    /// Local changes
    pub local: ContextSnapshot,
    
    /// Remote changes
    pub remote: ContextSnapshot,
}

impl ThreeWayMerge {
    /// Create a new three-way merge
    pub fn new(
        base: ContextSnapshot,
        local: ContextSnapshot,
        remote: ContextSnapshot,
    ) -> Self {
        Self { base, local, remote }
    }
    
    /// Perform three-way merge
    pub fn merge(&self) -> Result<DistributedContext> {
        use serde_json::Value;
        
        let mut merged = DistributedContext::new(&self.local.session_id);
        
        // Collect all keys
        let mut all_keys: std::collections::HashSet<String> = std::collections::HashSet::new();
        all_keys.extend(self.base.data.keys().cloned());
        all_keys.extend(self.local.data.keys().cloned());
        all_keys.extend(self.remote.data.keys().cloned());
        
        // Merge each key
        for key in all_keys {
            let base_val = self.base.data.get(&key);
            let local_val = self.local.data.get(&key);
            let remote_val = self.remote.data.get(&key);
            
            let merged_val = match (base_val, local_val, remote_val) {
                // Both sides deleted
                (Some(_), None, None) => None,
                
                // Local deleted, remote unchanged
                (Some(b), None, Some(r)) if b == r => None,
                
                // Remote deleted, local unchanged
                (Some(b), Some(l), None) if b == l => None,
                
                // Both sides modified to same value
                (Some(_), Some(l), Some(r)) if l == r => Some(l.clone()),
                
                // Local modified, remote unchanged
                (Some(b), Some(l), Some(r)) if b == r => Some(l.clone()),
                
                // Remote modified, local unchanged
                (Some(b), Some(l), Some(r)) if b == l => Some(r.clone()),
                
                // Conflict: both modified differently
                (Some(_), Some(l), Some(r)) if l != r => {
                    // Last write wins (prefer remote in case of conflict)
                    Some(r.clone())
                }
                
                // New key on both sides with same value
                (None, Some(l), Some(r)) if l == r => Some(l.clone()),
                
                // New key on local only
                (None, Some(l), None) => Some(l.clone()),
                
                // New key on remote only
                (None, None, Some(r)) => Some(r.clone()),
                
                // Conflict: new key on both sides with different values
                (None, Some(_), Some(r)) => Some(r.clone()), // Prefer remote
                
                _ => None,
            };
            
            if let Some(val) = merged_val {
                merged.set(key, val);
            }
        }
        
        Ok(merged)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_context_version() {
        let v1 = ContextVersion::new(1);
        assert_eq!(v1.version, 1);
        assert!(v1.timestamp > 0);
        
        let v2 = ContextVersion::with_parent(2, 1);
        assert_eq!(v2.version, 2);
        assert_eq!(v2.parent_version, Some(1));
    }
    
    #[test]
    fn test_versioned_context() {
        let mut vctx = VersionedContext::new("test");
        
        // Initial version
        assert_eq!(vctx.current.metadata.version, 1);
        
        // Update
        let mut new_ctx = DistributedContext::new("test");
        new_ctx.set("key1", json!("value1"));
        vctx.update(new_ctx).unwrap();
        
        // Check history
        assert_eq!(vctx.history.len(), 1);
    }
    
    #[test]
    fn test_last_write_wins() {
        let mut vctx = VersionedContext::new("test");
        vctx.resolution_strategy = ConflictResolution::LastWriteWins;
        
        // First update
        let mut ctx1 = DistributedContext::new("test");
        ctx1.set("key1", json!("value1"));
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        vctx.update(ctx1).unwrap();
        
        // Second update (newer timestamp)
        let mut ctx2 = DistributedContext::new("test");
        ctx2.set("key1", json!("value2"));
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        vctx.merge_with_resolution(&ctx2).unwrap();
        
        // Should have newer value
        assert_eq!(vctx.current.get("key1"), Some(&json!("value2")));
    }
    
    #[test]
    fn test_fail_on_conflict() {
        let mut vctx = VersionedContext::new("test");
        vctx.resolution_strategy = ConflictResolution::FailOnConflict;
        
        // Update to version 2
        let mut ctx1 = DistributedContext::new("test");
        ctx1.set("key1", json!("value1"));
        vctx.update(ctx1).unwrap();
        
        // Try to merge version 1 (conflict)
        let ctx2 = DistributedContext::new("test");
        let result = vctx.merge_with_resolution(&ctx2);
        
        assert!(result.is_err());
    }
    
    #[test]
    fn test_rollback() {
        let mut vctx = VersionedContext::new("test");
        
        // Version 1
        let mut ctx1 = DistributedContext::new("test");
        ctx1.set("key1", json!("v1"));
        vctx.update(ctx1).unwrap();
        let v1 = vctx.current.metadata.version;
        
        // Version 2
        let mut ctx2 = DistributedContext::new("test");
        ctx2.set("key1", json!("v2"));
        vctx.update(ctx2).unwrap();
        
        // Rollback to v1
        vctx.rollback_to(v1).unwrap();
        assert_eq!(vctx.current.get("key1"), Some(&json!("v1")));
    }
    
    #[test]
    fn test_three_way_merge() {
        // Base version
        let mut base = DistributedContext::new("test");
        base.set("key1", json!("base"));
        let base_snapshot = base.snapshot();
        
        // Local changes
        let mut local = base.clone();
        local.set("key1", json!("local"));
        local.set("key2", json!("local-only"));
        let local_snapshot = local.snapshot();
        
        // Remote changes
        let mut remote = base.clone();
        remote.set("key1", json!("remote"));
        remote.set("key3", json!("remote-only"));
        let remote_snapshot = remote.snapshot();
        
        // Perform merge
        let merger = ThreeWayMerge::new(base_snapshot, local_snapshot, remote_snapshot);
        let merged = merger.merge().unwrap();
        
        // Remote wins on conflict (key1)
        assert_eq!(merged.get("key1"), Some(&json!("remote")));
        // Local-only key preserved
        assert_eq!(merged.get("key2"), Some(&json!("local-only")));
        // Remote-only key preserved
        assert_eq!(merged.get("key3"), Some(&json!("remote-only")));
    }
}
