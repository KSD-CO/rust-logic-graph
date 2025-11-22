use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn, error};

use crate::error::{RustLogicGraphError, ErrorContext};

/// Transaction state for distributed transactions
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionState {
    /// Transaction initiated, waiting for prepare phase
    Initiated,
    /// All participants prepared, ready to commit
    Prepared,
    /// Transaction committed successfully
    Committed,
    /// Transaction aborted/rolled back
    Aborted,
}

/// Participant in a distributed transaction
#[derive(Debug, Clone)]
pub struct TransactionParticipant {
    pub id: String,
    pub database: String,
    pub state: TransactionState,
}

/// Distributed transaction using Two-Phase Commit (2PC) protocol
/// 
/// Coordinates transactions across multiple databases to ensure atomicity.
/// 
/// # Example
/// ```no_run
/// use rust_logic_graph::multi_db::DistributedTransaction;
/// 
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let mut txn = DistributedTransaction::new("order_txn_123");
///     
///     // Register participants
///     txn.add_participant("orders_db", "Insert order");
///     txn.add_participant("inventory_db", "Decrement stock");
///     txn.add_participant("payments_db", "Charge customer");
///     
///     // Phase 1: Prepare
///     txn.prepare().await?;
///     
///     // Phase 2: Commit (or abort if any participant fails)
///     if txn.can_commit() {
///         txn.commit().await?;
///     } else {
///         txn.abort().await?;
///     }
///     
///     Ok(())
/// }
/// ```
#[derive(Clone)]
pub struct DistributedTransaction {
    pub id: String,
    pub participants: Vec<TransactionParticipant>,
    pub state: TransactionState,
    pub metadata: HashMap<String, String>,
}

impl DistributedTransaction {
    /// Create a new distributed transaction
    pub fn new(id: impl Into<String>) -> Self {
        let txn_id = id.into();
        info!("🔄 Distributed Transaction: Creating transaction '{}'", txn_id);
        
        Self {
            id: txn_id,
            participants: Vec::new(),
            state: TransactionState::Initiated,
            metadata: HashMap::new(),
        }
    }
    
    /// Add a participant to the transaction
    pub fn add_participant(&mut self, database: impl Into<String>, id: impl Into<String>) -> &mut Self {
        let participant = TransactionParticipant {
            id: id.into(),
            database: database.into(),
            state: TransactionState::Initiated,
        };
        
        info!("  ➕ Adding participant: {} ({})", participant.id, participant.database);
        self.participants.push(participant);
        self
    }
    
    /// Add metadata to the transaction
    pub fn add_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
    
    /// Phase 1: Prepare - Ask all participants if they can commit
    /// 
    /// In a real implementation, this would:
    /// 1. Lock resources in each database
    /// 2. Validate constraints
    /// 3. Write to transaction log
    /// 4. Return ready/abort status
    pub async fn prepare(&mut self) -> Result<bool, RustLogicGraphError> {
        info!("🔄 Transaction '{}': PREPARE phase starting ({} participants)", 
            self.id, self.participants.len());
        
        if self.state != TransactionState::Initiated {
            return Err(RustLogicGraphError::configuration_error(
                format!("Cannot prepare transaction in state {:?}", self.state)
            ));
        }
        
        let mut all_prepared = true;
        
        // Ask each participant to prepare
        let participant_count = self.participants.len();
        for i in 0..participant_count {
            info!("  🔍 Preparing participant: {} ({})", 
                self.participants[i].id, self.participants[i].database);
            
            // TODO: In real implementation, call database-specific prepare
            // For now, simulate success
            let participant = &self.participants[i];
            let prepared = self.simulate_prepare(participant).await?;
            
            if prepared {
                self.participants[i].state = TransactionState::Prepared;
                info!("  ✅ Participant {} prepared successfully", self.participants[i].id);
            } else {
                self.participants[i].state = TransactionState::Aborted;
                warn!("  ❌ Participant {} failed to prepare", self.participants[i].id);
                all_prepared = false;
                break;
            }
        }
        
        if all_prepared {
            self.state = TransactionState::Prepared;
            info!("✅ Transaction '{}': All participants prepared", self.id);
        } else {
            self.state = TransactionState::Aborted;
            warn!("⚠️  Transaction '{}': Prepare phase failed", self.id);
        }
        
        Ok(all_prepared)
    }
    
    /// Check if transaction can commit (all participants prepared)
    pub fn can_commit(&self) -> bool {
        self.state == TransactionState::Prepared &&
        self.participants.iter().all(|p| p.state == TransactionState::Prepared)
    }
    
    /// Phase 2: Commit - Instruct all participants to commit
    pub async fn commit(&mut self) -> Result<(), RustLogicGraphError> {
        info!("🔄 Transaction '{}': COMMIT phase starting", self.id);
        
        if !self.can_commit() {
            return Err(RustLogicGraphError::configuration_error(
                format!("Cannot commit transaction in state {:?}", self.state)
            ));
        }
        
        // Commit all participants
        let participant_count = self.participants.len();
        for i in 0..participant_count {
            info!("  💾 Committing participant: {} ({})", 
                self.participants[i].id, self.participants[i].database);
            
            // TODO: In real implementation, call database-specific commit
            let participant = &self.participants[i];
            self.simulate_commit(participant).await?;
            
            self.participants[i].state = TransactionState::Committed;
            info!("  ✅ Participant {} committed", self.participants[i].id);
        }
        
        self.state = TransactionState::Committed;
        info!("✅ Transaction '{}': Successfully committed", self.id);
        
        Ok(())
    }
    
    /// Abort/rollback the transaction
    pub async fn abort(&mut self) -> Result<(), RustLogicGraphError> {
        warn!("🔄 Transaction '{}': ABORT phase starting", self.id);
        
        // Rollback all participants
        let participant_count = self.participants.len();
        for i in 0..participant_count {
            if self.participants[i].state == TransactionState::Prepared {
                warn!("  ↩️  Rolling back participant: {} ({})", 
                    self.participants[i].id, self.participants[i].database);
                
                // TODO: In real implementation, call database-specific rollback
                let participant = &self.participants[i];
                self.simulate_rollback(participant).await?;
                
                self.participants[i].state = TransactionState::Aborted;
                warn!("  ✅ Participant {} rolled back", self.participants[i].id);
            }
        }
        
        self.state = TransactionState::Aborted;
        warn!("⚠️  Transaction '{}': Aborted and rolled back", self.id);
        
        Ok(())
    }
    
    // Simulation methods (to be replaced with real database calls)
    
    async fn simulate_prepare(&self, _participant: &TransactionParticipant) -> Result<bool, RustLogicGraphError> {
        // Simulate network delay
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        Ok(true) // Always succeed for now
    }
    
    async fn simulate_commit(&self, _participant: &TransactionParticipant) -> Result<(), RustLogicGraphError> {
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        Ok(())
    }
    
    async fn simulate_rollback(&self, _participant: &TransactionParticipant) -> Result<(), RustLogicGraphError> {
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        Ok(())
    }
}

/// Transaction coordinator managing multiple distributed transactions
pub struct TransactionCoordinator {
    transactions: Arc<Mutex<HashMap<String, DistributedTransaction>>>,
}

impl TransactionCoordinator {
    /// Create a new transaction coordinator
    pub fn new() -> Self {
        Self {
            transactions: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Begin a new distributed transaction
    pub async fn begin(&self, txn_id: impl Into<String>) -> Result<String, RustLogicGraphError> {
        let id = txn_id.into();
        let txn = DistributedTransaction::new(id.clone());
        
        let mut txns = self.transactions.lock().await;
        if txns.contains_key(&id) {
            return Err(RustLogicGraphError::configuration_error(
                format!("Transaction '{}' already exists", id)
            ));
        }
        
        txns.insert(id.clone(), txn);
        Ok(id)
    }
    
    /// Get a transaction by ID
    pub async fn get(&self, txn_id: &str) -> Result<DistributedTransaction, RustLogicGraphError> {
        let txns = self.transactions.lock().await;
        txns.get(txn_id)
            .cloned()
            .ok_or_else(|| RustLogicGraphError::configuration_error(
                format!("Transaction '{}' not found", txn_id)
            ))
    }
    
    /// Update a transaction
    pub async fn update(&self, txn: DistributedTransaction) -> Result<(), RustLogicGraphError> {
        let mut txns = self.transactions.lock().await;
        txns.insert(txn.id.clone(), txn);
        Ok(())
    }
    
    /// Remove a completed transaction
    pub async fn remove(&self, txn_id: &str) -> Result<(), RustLogicGraphError> {
        let mut txns = self.transactions.lock().await;
        txns.remove(txn_id);
        Ok(())
    }
    
    /// Get all active transactions
    pub async fn active_transactions(&self) -> Vec<String> {
        let txns = self.transactions.lock().await;
        txns.keys().cloned().collect()
    }
}

impl Default for TransactionCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_transaction_lifecycle() {
        let mut txn = DistributedTransaction::new("test_txn");
        
        txn.add_participant("db1", "op1");
        txn.add_participant("db2", "op2");
        
        // Prepare
        let prepared = txn.prepare().await.unwrap();
        assert!(prepared);
        assert_eq!(txn.state, TransactionState::Prepared);
        
        // Commit
        txn.commit().await.unwrap();
        assert_eq!(txn.state, TransactionState::Committed);
    }
    
    #[tokio::test]
    async fn test_transaction_abort() {
        let mut txn = DistributedTransaction::new("test_txn");
        
        txn.add_participant("db1", "op1");
        
        // Prepare
        txn.prepare().await.unwrap();
        
        // Abort instead of commit
        txn.abort().await.unwrap();
        assert_eq!(txn.state, TransactionState::Aborted);
    }
    
    #[tokio::test]
    async fn test_coordinator() {
        let coordinator = TransactionCoordinator::new();
        
        let txn_id = coordinator.begin("txn1").await.unwrap();
        assert_eq!(txn_id, "txn1");
        
        let txn = coordinator.get(&txn_id).await.unwrap();
        assert_eq!(txn.id, "txn1");
        
        let active = coordinator.active_transactions().await;
        assert_eq!(active.len(), 1);
    }
}
