use super::errors::{DomainError, DomainResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageType {
    Persistent,
    Instance,
}

/// Represents the footprint analysis for storage entries
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FootprintData {
    pub size_bytes: u64,
    pub max_cost: u64, // Maximum extension cost in stroops
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContractMonitor {
    pub contract_id: String,
    pub storage_type: StorageType,
    pub base_threshold: u32,           // Base threshold without congestion
    pub target_threshold: u32,         // Current threshold (adjusted for congestion)
    pub live_ledger_expiration_bound: u32,
    pub congestion_factor: f64,        // 1.0 = no congestion, >1.0 = congested
    pub footprint: Option<FootprintData>,
}

impl ContractMonitor {
    pub fn new(
        contract_id: String,
        storage_type: StorageType,
        base_threshold: u32,
        live_ledger_expiration_bound: u32,
    ) -> DomainResult<Self> {
        if base_threshold == 0 {
            return Err(DomainError::InvalidThreshold {
                threshold: base_threshold,
                reason: "threshold must be greater than zero".to_string(),
            });
        }

        Ok(Self {
            contract_id,
            storage_type,
            base_threshold,
            target_threshold: base_threshold,
            live_ledger_expiration_bound,
            congestion_factor: 1.0,
            footprint: None,
        })
    }

    /// Calculate remaining ledgers until expiration
    pub fn remaining_life(&self, current_ledger: u32) -> Option<u32> {
        self.live_ledger_expiration_bound.checked_sub(current_ledger)
    }

    /// Check if contract requires extension based on current threshold
    pub fn requires_extension(&self, current_ledger: u32) -> bool {
        match self.remaining_life(current_ledger) {
            Some(rem) => rem <= self.target_threshold,
            None => true, // Already expired
        }
    }

    /// Adjust threshold dynamically based on network congestion
    /// 
    /// When the network is congested (high base fee), we extend contracts earlier
    /// to avoid last-minute bidding wars and ensure inclusion.
    pub fn adjust_threshold_for_congestion(&mut self, base_fee: u64) -> DomainResult<()> {
        // Define congestion thresholds (in stroops)
        const NORMAL_BASE_FEE: u64 = 100;      // Normal network conditions
        const HIGH_BASE_FEE: u64 = 1_000;      // High congestion
        const EXTREME_BASE_FEE: u64 = 10_000;  // Extreme congestion

        // Calculate congestion factor
        self.congestion_factor = if base_fee <= NORMAL_BASE_FEE {
            1.0
        } else if base_fee <= HIGH_BASE_FEE {
            1.0 + ((base_fee - NORMAL_BASE_FEE) as f64 / NORMAL_BASE_FEE as f64) * 0.5
        } else if base_fee <= EXTREME_BASE_FEE {
            1.5 + ((base_fee - HIGH_BASE_FEE) as f64 / HIGH_BASE_FEE as f64) * 1.0
        } else {
            2.5 // Maximum 2.5x expansion
        };

        // Apply congestion factor to base threshold
        let adjusted = (self.base_threshold as f64 * self.congestion_factor) as u32;
        
        // Cap at reasonable maximum (e.g., 100,000 ledgers)
        self.target_threshold = adjusted.min(100_000);

        Ok(())
    }

    /// Set footprint analysis data
    pub fn set_footprint(&mut self, size_bytes: u64, estimated_cost_per_ledger: u64) -> DomainResult<()> {
        if size_bytes == 0 {
            return Err(DomainError::InvalidThreshold {
                threshold: 0,
                reason: "footprint size cannot be zero".to_string(),
            });
        }

        // Calculate maximum cost for extending to target threshold
        let max_cost = estimated_cost_per_ledger
            .checked_mul(self.target_threshold as u64)
            .ok_or_else(|| DomainError::OverflowDetected {
                operation: "calculating max footprint cost".to_string(),
            })?;

        self.footprint = Some(FootprintData {
            size_bytes,
            max_cost,
        });

        Ok(())
    }

    /// Calculate the estimated cost of extending this contract's storage
    pub fn calculate_extension_cost(&self) -> DomainResult<u64> {
        match &self.footprint {
            Some(footprint) => Ok(footprint.max_cost),
            None => {
                // Fallback: Use storage type to estimate
                let base_cost_per_ledger: u64 = match self.storage_type {
                    StorageType::Persistent => 1000, // Higher cost for persistent storage
                    StorageType::Instance => 500,    // Lower cost for instance storage
                };

                base_cost_per_ledger
                    .checked_mul(self.target_threshold as u64)
                    .ok_or_else(|| DomainError::OverflowDetected {
                        operation: "calculating extension cost".to_string(),
                    })
            }
        }
    }

    /// Update the live ledger expiration bound (called after successful extension)
    pub fn update_expiration(&mut self, new_expiration: u32) -> DomainResult<()> {
        if new_expiration <= self.live_ledger_expiration_bound {
            return Err(DomainError::InvalidThreshold {
                threshold: new_expiration,
                reason: format!(
                    "new expiration ({}) must be greater than current ({})",
                    new_expiration, self.live_ledger_expiration_bound
                ),
            });
        }

        self.live_ledger_expiration_bound = new_expiration;
        Ok(())
    }

    /// Get urgency level for extension (0.0 = not urgent, 1.0 = critical)
    pub fn urgency_level(&self, current_ledger: u32) -> f64 {
        match self.remaining_life(current_ledger) {
            Some(remaining) if remaining > self.target_threshold => 0.0,
            Some(remaining) => {
                1.0 - (remaining as f64 / self.target_threshold as f64)
            }
            None => 1.0, // Maximum urgency if expired
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_contract_monitor() {
        let monitor = ContractMonitor::new(
            "CONTRACT123".to_string(),
            StorageType::Persistent,
            5000,
            10000,
        );
        assert!(monitor.is_ok());
        let m = monitor.unwrap();
        assert_eq!(m.base_threshold, 5000);
        assert_eq!(m.target_threshold, 5000);
    }

    #[test]
    fn test_zero_threshold_rejected() {
        let monitor = ContractMonitor::new(
            "CONTRACT123".to_string(),
            StorageType::Persistent,
            0,
            10000,
        );
        assert!(matches!(monitor, Err(DomainError::InvalidThreshold { .. })));
    }

    #[test]
    fn test_remaining_life() {
        let monitor = ContractMonitor::new(
            "CONTRACT123".to_string(),
            StorageType::Persistent,
            5000,
            10000,
        ).unwrap();

        assert_eq!(monitor.remaining_life(8000), Some(2000));
        assert_eq!(monitor.remaining_life(10000), Some(0));
        assert_eq!(monitor.remaining_life(11000), None);
    }

    #[test]
    fn test_requires_extension() {
        let monitor = ContractMonitor::new(
            "CONTRACT123".to_string(),
            StorageType::Persistent,
            5000,
            10000,
        ).unwrap();

        assert!(!monitor.requires_extension(1000)); // 9000 remaining > 5000
        assert!(monitor.requires_extension(6000));  // 4000 remaining < 5000
        assert!(monitor.requires_extension(11000)); // Expired
    }

    #[test]
    fn test_congestion_adjustment() {
        let mut monitor = ContractMonitor::new(
            "CONTRACT123".to_string(),
            StorageType::Persistent,
            5000,
            10000,
        ).unwrap();

        // Normal conditions
        monitor.adjust_threshold_for_congestion(100).unwrap();
        assert_eq!(monitor.target_threshold, 5000);

        // High congestion
        monitor.adjust_threshold_for_congestion(1000).unwrap();
        assert!(monitor.target_threshold > 5000);
        assert!(monitor.congestion_factor > 1.0);

        // Extreme congestion
        monitor.adjust_threshold_for_congestion(20000).unwrap();
        assert!(monitor.target_threshold > 10000);
        assert_eq!(monitor.congestion_factor, 2.5);
    }

    #[test]
    fn test_footprint_analysis() {
        let mut monitor = ContractMonitor::new(
            "CONTRACT123".to_string(),
            StorageType::Persistent,
            5000,
            10000,
        ).unwrap();

        monitor.set_footprint(1024, 100).unwrap();
        
        let cost = monitor.calculate_extension_cost().unwrap();
        assert_eq!(cost, 500_000); // 100 * 5000
    }

    #[test]
    fn test_urgency_level() {
        let monitor = ContractMonitor::new(
            "CONTRACT123".to_string(),
            StorageType::Persistent,
            5000,
            10000,
        ).unwrap();

        // Not urgent
        assert_eq!(monitor.urgency_level(1000), 0.0);
        
        // Moderately urgent
        let urgency = monitor.urgency_level(7500);
        assert!(urgency > 0.0 && urgency < 1.0);
        
        // Critical
        assert_eq!(monitor.urgency_level(11000), 1.0);
    }
}
