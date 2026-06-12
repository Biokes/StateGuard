use super::errors::{DomainError, DomainResult};

/// Specific reasons for job failure
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FailureReason {
    TxBadSeq,
    InsufficientFee,
    NetworkTimeout,
    SimulationFailed { reason: String },
    InvalidContract,
    Other { description: String },
}

impl std::fmt::Display for FailureReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FailureReason::TxBadSeq => write!(f, "Transaction bad sequence"),
            FailureReason::InsufficientFee => write!(f, "Insufficient fee"),
            FailureReason::NetworkTimeout => write!(f, "Network timeout"),
            FailureReason::SimulationFailed { reason } => write!(f, "Simulation failed: {}", reason),
            FailureReason::InvalidContract => write!(f, "Invalid contract"),
            FailureReason::Other { description } => write!(f, "{}", description),
        }
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobStatus {
    Pending,
    Signing,
    Simulating,
    Submitted,
    AwaitingConsensus,
    Confirmed,
    Failed(FailureReason),
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobStatus::Pending => write!(f, "Pending"),
            JobStatus::Signing => write!(f, "Signing"),
            JobStatus::Simulating => write!(f, "Simulating"),
            JobStatus::Submitted => write!(f, "Submitted"),
            JobStatus::AwaitingConsensus => write!(f, "Awaiting Consensus"),
            JobStatus::Confirmed => write!(f, "Confirmed"),
            JobStatus::Failed(reason) => write!(f, "Failed: {}", reason),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionJob {
    pub job_id: String,
    pub target_contract_id: String,
    pub payload: Vec<u8>,
    pub max_gas_price: u64,
    pub current_gas_price: u64,
    pub status: JobStatus,
    pub retries: u32,
    pub max_retries: u32,
    pub transaction_hash: Option<String>,
}

impl ExecutionJob {
    pub fn new(
        job_id: String,
        target_contract_id: String,
        payload: Vec<u8>,
        max_gas_price: u64,
    ) -> DomainResult<Self> {
        if max_gas_price == 0 {
            return Err(DomainError::InvalidGasConfiguration {
                reason: "max_gas_price must be greater than zero".to_string(),
            });
        }

        Ok(Self {
            job_id,
            target_contract_id,
            payload,
            max_gas_price,
            current_gas_price: max_gas_price / 2, // Start at 50% of max
            status: JobStatus::Pending,
            retries: 0,
            max_retries: 3,
            transaction_hash: None,
        })
    }

    /// Transition to Signing state
    pub fn mark_signing(&mut self) -> DomainResult<()> {
        match self.status {
            JobStatus::Pending => {
                self.status = JobStatus::Signing;
                Ok(())
            }
            _ => Err(DomainError::InvalidStateTransition {
                from: format!("{}", self.status),
                to: "Signing".to_string(),
                reason: "can only sign from Pending state".to_string(),
            }),
        }
    }

    /// Transition to Simulating state
    pub fn mark_simulating(&mut self) -> DomainResult<()> {
        match self.status {
            JobStatus::Signing => {
                self.status = JobStatus::Simulating;
                Ok(())
            }
            _ => Err(DomainError::InvalidStateTransition {
                from: format!("{}", self.status),
                to: "Simulating".to_string(),
                reason: "can only simulate from Signing state".to_string(),
            }),
        }
    }

    /// Transition to Submitted state
    pub fn mark_submitted(&mut self, tx_hash: String) -> DomainResult<()> {
        match self.status {
            JobStatus::Simulating => {
                self.status = JobStatus::Submitted;
                self.transaction_hash = Some(tx_hash);
                Ok(())
            }
            _ => Err(DomainError::InvalidStateTransition {
                from: format!("{}", self.status),
                to: "Submitted".to_string(),
                reason: "can only submit from Simulating state".to_string(),
            }),
        }
    }

    /// Transition to AwaitingConsensus state
    pub fn mark_awaiting_consensus(&mut self) -> DomainResult<()> {
        match self.status {
            JobStatus::Submitted => {
                self.status = JobStatus::AwaitingConsensus;
                Ok(())
            }
            _ => Err(DomainError::InvalidStateTransition {
                from: format!("{}", self.status),
                to: "AwaitingConsensus".to_string(),
                reason: "can only await consensus from Submitted state".to_string(),
            }),
        }
    }

    /// Transition to Confirmed state
    pub fn mark_confirmed(&mut self) -> DomainResult<()> {
        match self.status {
            JobStatus::AwaitingConsensus => {
                self.status = JobStatus::Confirmed;
                Ok(())
            }
            _ => Err(DomainError::InvalidStateTransition {
                from: format!("{}", self.status),
                to: "Confirmed".to_string(),
                reason: "can only confirm from AwaitingConsensus state".to_string(),
            }),
        }
    }

    /// Transition to Failed state
    pub fn mark_failed(&mut self, reason: FailureReason) {
        self.status = JobStatus::Failed(reason);
    }

    /// Check if job can be retried
    pub fn can_retry(&self) -> bool {
        matches!(self.status, JobStatus::Failed(_)) && self.retries < self.max_retries
    }

    /// Increment retry counter and reset to Pending
    pub fn increment_retry(&mut self) -> DomainResult<()> {
        if !self.can_retry() {
            return Err(DomainError::InvalidStateTransition {
                from: format!("{}", self.status),
                to: "Pending (retry)".to_string(),
                reason: format!("max retries ({}) exceeded", self.max_retries),
            });
        }

        self.retries = self.retries
            .checked_add(1)
            .ok_or_else(|| DomainError::OverflowDetected {
                operation: "incrementing retry counter".to_string(),
            })?;

        self.status = JobStatus::Pending;
        self.transaction_hash = None;

        Ok(())
    }

    /// Bump fee by a percentage (for InsufficientFee failures)
    pub fn bump_fee(&mut self, increment_percent: u8) -> DomainResult<()> {
        if increment_percent == 0 || increment_percent > 100 {
            return Err(DomainError::InvalidGasConfiguration {
                reason: format!("increment_percent must be between 1 and 100, got {}", increment_percent),
            });
        }

        // Calculate new fee
        let increment = self.current_gas_price
            .checked_mul(increment_percent as u64)
            .and_then(|v| v.checked_div(100))
            .ok_or_else(|| DomainError::OverflowDetected {
                operation: "calculating fee increment".to_string(),
            })?;

        let new_fee = self.current_gas_price
            .checked_add(increment)
            .ok_or_else(|| DomainError::OverflowDetected {
                operation: "adding fee increment".to_string(),
            })?;

        // Cap at max_gas_price
        if new_fee > self.max_gas_price {
            return Err(DomainError::InvalidGasConfiguration {
                reason: format!(
                    "fee bump would exceed max_gas_price: {} > {}",
                    new_fee, self.max_gas_price
                ),
            });
        }

        self.current_gas_price = new_fee;
        Ok(())
    }

    /// Handle TxBadSeq failure - immediate retry after sequence sync
    pub fn handle_bad_sequence(&mut self) -> DomainResult<()> {
        match &self.status {
            JobStatus::Failed(FailureReason::TxBadSeq) => {
                // Reset to Pending without incrementing retry counter
                // The adapter should re-sync sequence number before retrying
                self.status = JobStatus::Pending;
                self.transaction_hash = None;
                Ok(())
            }
            _ => Err(DomainError::InvalidStateTransition {
                from: format!("{}", self.status),
                to: "Pending (sequence fix)".to_string(),
                reason: "can only handle bad sequence from Failed(TxBadSeq) state".to_string(),
            }),
        }
    }

    /// Handle InsufficientFee failure - bump fee and retry
    pub fn handle_insufficient_fee(&mut self, bump_percent: u8) -> DomainResult<()> {
        match &self.status {
            JobStatus::Failed(FailureReason::InsufficientFee) => {
                self.bump_fee(bump_percent)?;
                self.increment_retry()?;
                Ok(())
            }
            _ => Err(DomainError::InvalidStateTransition {
                from: format!("{}", self.status),
                to: "Pending (fee bump)".to_string(),
                reason: "can only handle insufficient fee from Failed(InsufficientFee) state".to_string(),
            }),
        }
    }

    /// Check if job is in terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(self.status, JobStatus::Confirmed | JobStatus::Failed(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_job() {
        let job = ExecutionJob::new(
            "job1".to_string(),
            "contract1".to_string(),
            vec![1, 2, 3],
            1_000_000,
        );
        assert!(job.is_ok());
        let j = job.unwrap();
        assert_eq!(j.status, JobStatus::Pending);
        assert_eq!(j.current_gas_price, 500_000);
    }

    #[test]
    fn test_zero_gas_rejected() {
        let job = ExecutionJob::new(
            "job1".to_string(),
            "contract1".to_string(),
            vec![1, 2, 3],
            0,
        );
        assert!(matches!(job, Err(DomainError::InvalidGasConfiguration { .. })));
    }

    #[test]
    fn test_state_transitions() {
        let mut job = ExecutionJob::new(
            "job1".to_string(),
            "contract1".to_string(),
            vec![1, 2, 3],
            1_000_000,
        ).unwrap();

        assert!(job.mark_signing().is_ok());
        assert_eq!(job.status, JobStatus::Signing);

        assert!(job.mark_simulating().is_ok());
        assert_eq!(job.status, JobStatus::Simulating);

        assert!(job.mark_submitted("tx_hash_123".to_string()).is_ok());
        assert_eq!(job.status, JobStatus::Submitted);
        assert_eq!(job.transaction_hash, Some("tx_hash_123".to_string()));

        assert!(job.mark_awaiting_consensus().is_ok());
        assert_eq!(job.status, JobStatus::AwaitingConsensus);

        assert!(job.mark_confirmed().is_ok());
        assert_eq!(job.status, JobStatus::Confirmed);
    }

    #[test]
    fn test_invalid_state_transition() {
        let mut job = ExecutionJob::new(
            "job1".to_string(),
            "contract1".to_string(),
            vec![1, 2, 3],
            1_000_000,
        ).unwrap();

        // Try to simulate before signing
        let result = job.mark_simulating();
        assert!(matches!(result, Err(DomainError::InvalidStateTransition { .. })));
    }

    #[test]
    fn test_retry_logic() {
        let mut job = ExecutionJob::new(
            "job1".to_string(),
            "contract1".to_string(),
            vec![1, 2, 3],
            1_000_000,
        ).unwrap();

        job.mark_failed(FailureReason::NetworkTimeout);
        assert!(job.can_retry());

        assert!(job.increment_retry().is_ok());
        assert_eq!(job.retries, 1);
        assert_eq!(job.status, JobStatus::Pending);

        // Exhaust retries
        job.mark_failed(FailureReason::NetworkTimeout);
        job.increment_retry().unwrap();
        job.mark_failed(FailureReason::NetworkTimeout);
        job.increment_retry().unwrap();
        job.mark_failed(FailureReason::NetworkTimeout);

        assert!(!job.can_retry());
    }

    #[test]
    fn test_fee_bump() {
        let mut job = ExecutionJob::new(
            "job1".to_string(),
            "contract1".to_string(),
            vec![1, 2, 3],
            1_000_000,
        ).unwrap();

        job.current_gas_price = 100_000;
        assert!(job.bump_fee(10).is_ok()); // 10% increase
        assert_eq!(job.current_gas_price, 110_000);
    }

    #[test]
    fn test_fee_bump_exceeds_max() {
        let mut job = ExecutionJob::new(
            "job1".to_string(),
            "contract1".to_string(),
            vec![1, 2, 3],
            1_000_000,
        ).unwrap();

        job.current_gas_price = 950_000;
        let result = job.bump_fee(10); // Would exceed max
        assert!(matches!(result, Err(DomainError::InvalidGasConfiguration { .. })));
    }

    #[test]
    fn test_handle_bad_sequence() {
        let mut job = ExecutionJob::new(
            "job1".to_string(),
            "contract1".to_string(),
            vec![1, 2, 3],
            1_000_000,
        ).unwrap();

        job.mark_failed(FailureReason::TxBadSeq);
        assert!(job.handle_bad_sequence().is_ok());
        assert_eq!(job.status, JobStatus::Pending);
        assert_eq!(job.retries, 0); // Should not increment retries
    }

    #[test]
    fn test_handle_insufficient_fee() {
        let mut job = ExecutionJob::new(
            "job1".to_string(),
            "contract1".to_string(),
            vec![1, 2, 3],
            1_000_000,
        ).unwrap();

        job.current_gas_price = 100_000;
        job.mark_failed(FailureReason::InsufficientFee);
        
        assert!(job.handle_insufficient_fee(10).is_ok());
        assert_eq!(job.status, JobStatus::Pending);
        assert_eq!(job.current_gas_price, 110_000);
        assert_eq!(job.retries, 1);
    }
}
