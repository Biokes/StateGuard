use std::fmt;

/// Domain-specific errors that enforce business invariants
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    /// Vault does not have sufficient funds to complete operation
    InsufficientFunds {
        required: u64,
        available: u64,
    },
    
    /// Integer overflow detected in arithmetic operation
    OverflowDetected {
        operation: String,
    },
    
    /// Invalid threshold configuration
    InvalidThreshold {
        threshold: u32,
        reason: String,
    },
    
    /// Contract not found in vault's registered contracts
    ContractNotFound {
        contract_id: String,
    },
    
    /// Invalid reserve configuration
    InvalidReserve {
        reason: String,
    },
    
    /// State transition not allowed
    InvalidStateTransition {
        from: String,
        to: String,
        reason: String,
    },
    
    /// Invalid gas configuration
    InvalidGasConfiguration {
        reason: String,
    },
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DomainError::InsufficientFunds { required, available } => {
                write!(
                    f,
                    "Insufficient funds: required {} stroops, available {} stroops",
                    required, available
                )
            }
            DomainError::OverflowDetected { operation } => {
                write!(f, "Integer overflow detected during: {}", operation)
            }
            DomainError::InvalidThreshold { threshold, reason } => {
                write!(f, "Invalid threshold {}: {}", threshold, reason)
            }
            DomainError::ContractNotFound { contract_id } => {
                write!(f, "Contract not found: {}", contract_id)
            }
            DomainError::InvalidReserve { reason } => {
                write!(f, "Invalid reserve configuration: {}", reason)
            }
            DomainError::InvalidStateTransition { from, to, reason } => {
                write!(f, "Invalid state transition from {} to {}: {}", from, to, reason)
            }
            DomainError::InvalidGasConfiguration { reason } => {
                write!(f, "Invalid gas configuration: {}", reason)
            }
        }
    }
}

impl std::error::Error for DomainError {}

/// Convenience Result type for domain operations
pub type DomainResult<T> = Result<T, DomainError>;
