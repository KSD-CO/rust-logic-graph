//! Saga Pattern Implementation
//! Transaction coordinator, compensation, state persistence, timeout/deadline

use std::collections::HashMap;
use std::time::{Duration, Instant};
use anyhow::Result;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SagaStepStatus {
    Pending,
    Completed,
    Failed,
    Compensated,
    Aborted,
}

pub struct SagaStep {
    pub id: String,
    pub action: Box<dyn Fn(&mut SagaContext) -> Result<()>>,
    pub compensation: Option<Box<dyn Fn(&mut SagaContext) -> Result<()>>>,
    pub status: SagaStepStatus,
    pub timeout: Option<Duration>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct SagaContext {
    pub data: HashMap<String, serde_json::Value>,
}

#[derive(Debug)]
pub struct SagaState {
    pub steps: Vec<SagaStepStatus>,
    pub started_at: Instant,
    pub finished_at: Option<Instant>,
    pub aborted: bool,
}

pub struct SagaCoordinator {
    pub steps: Vec<SagaStep>,
    pub context: SagaContext,
    pub state: SagaState,
    pub deadline: Option<Instant>,
}

impl SagaCoordinator {
    pub fn new(deadline: Option<Duration>) -> Self {
        let now = Instant::now();
        Self {
            steps: Vec::new(),
            context: SagaContext::default(),
            state: SagaState {
                steps: Vec::new(),
                started_at: now,
                finished_at: None,
                aborted: false,
            },
            deadline: deadline.map(|d| now + d),
        }
    }

    pub fn add_step(&mut self, step: SagaStep) {
        self.steps.push(step);
        self.state.steps.push(SagaStepStatus::Pending);
    }

    pub fn execute(&mut self) -> Result<()> {
        for (i, step) in self.steps.iter_mut().enumerate() {
            if let Some(deadline) = self.deadline {
                if Instant::now() > deadline {
                    self.state.aborted = true;
                    self.state.finished_at = Some(Instant::now());
                    self.compensate(i)?;
                    return Err(anyhow::anyhow!("Saga deadline exceeded"));
                }
            }
            let step_start = Instant::now();
            let timeout = step.timeout;
            let result = (step.action)(&mut self.context);
            if let Err(e) = result {
                self.state.steps[i] = SagaStepStatus::Failed;
                self.compensate(i)?;
                self.state.aborted = true;
                self.state.finished_at = Some(Instant::now());
                return Err(e);
            }
            if let Some(t) = timeout {
                if step_start.elapsed() > t {
                    self.state.steps[i] = SagaStepStatus::Aborted;
                    self.compensate(i)?;
                    self.state.aborted = true;
                    self.state.finished_at = Some(Instant::now());
                    return Err(anyhow::anyhow!("Step timeout exceeded"));
                }
            }
            self.state.steps[i] = SagaStepStatus::Completed;
        }
        self.state.finished_at = Some(Instant::now());
        Ok(())
    }

    fn compensate(&mut self, failed_index: usize) -> Result<()> {
        for (i, step) in self.steps[..=failed_index].iter_mut().rev().enumerate() {
            if let Some(comp) = &step.compensation {
                let result = (comp)(&mut self.context);
                if result.is_ok() {
                    self.state.steps[failed_index - i] = SagaStepStatus::Compensated;
                }
            }
        }
        Ok(())
    }
}
