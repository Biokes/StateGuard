use super::errors::{DomainError, DomainResult};
use std::collections::VecDeque;

const GAS_HISTORY_CAPACITY: usize = 100;
const LEDGERS_PER_DAY: u64 = 17280;
const EMA_SMOOTHING_FACTOR: f64 = 0.1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeeperVault {
    pub client_account: String,
    pub vault_balance: u64,
    pub min_reserve: u64,
    pub registered_contracts: Vec<RegisteredContract>,
    pub gas_burn_history: VecDeque<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisteredContract {
    pub contract_id: String,
    pub safe_reserve: u64,
    pub assigned_gas_limit: u64,
}

impl KeeperVault {
    pub fn new(client_account: String, vault_balance: u64, min_reserve: u64) -> DomainResult<Self> {
        if min_reserve > vault_balance {
            return Err(DomainError::InvalidReserve {
                reason: format!(
                    "min_reserve ({}) cannot exceed vault_balance ({})",
                    min_reserve, vault_balance
                ),
            });
        }

        Ok(Self {
            client_account,
            vault_balance,
            min_reserve,
            registered_contracts: Vec::new(),
            gas_burn_history: VecDeque::with_capacity(GAS_HISTORY_CAPACITY),
        })
    }

    pub fn can_afford_new_contract(&self, contract_safe_reserve: u64) -> DomainResult<bool> {
        let total_reserves = self.calculate_total_reserves()?;
        
        let required_reserve = self.min_reserve
            .checked_add(total_reserves)
            .and_then(|sum| sum.checked_add(contract_safe_reserve))
            .ok_or_else(|| DomainError::OverflowDetected {
                operation: "calculating required reserve for new contract".to_string(),
            })?;

        Ok(self.vault_balance >= required_reserve)
    }

    pub fn register_contract(&mut self, contract: RegisteredContract) -> DomainResult<()> {
        if !self.can_afford_new_contract(contract.safe_reserve)? {
            let total_reserves = self.calculate_total_reserves()?;
            let required = self.min_reserve
                .checked_add(total_reserves)
                .and_then(|sum| sum.checked_add(contract.safe_reserve))
                .ok_or_else(|| DomainError::OverflowDetected {
                    operation: "calculating required funds".to_string(),
                })?;

            return Err(DomainError::InsufficientFunds {
                required,
                available: self.vault_balance,
            });
        }

        if self.registered_contracts.iter().any(|c| c.contract_id == contract.contract_id) {
            return Err(DomainError::InvalidReserve {
                reason: format!("Contract {} already registered", contract.contract_id),
            });
        }

        self.registered_contracts.push(contract);
        Ok(())
    }

    pub fn record_gas_burn(&mut self, amount: u64) -> DomainResult<()> {
        self.vault_balance = self.vault_balance
            .checked_sub(amount)
            .ok_or_else(|| DomainError::InsufficientFunds {
                required: amount,
                available: self.vault_balance,
            })?;

        if self.gas_burn_history.len() >= GAS_HISTORY_CAPACITY {
            self.gas_burn_history.pop_front();
        }
        self.gas_burn_history.push_back(amount);

        Ok(())
    }

    pub fn calculate_burn_rate_ema(&self) -> u64 {
        if self.gas_burn_history.is_empty() {
            return 0;
        }

        if self.gas_burn_history.len() == 1 {
            return self.gas_burn_history[0];
        }

        let mut ema = self.gas_burn_history[0] as f64;
        
        for &value in self.gas_burn_history.iter().skip(1) {
            ema = (EMA_SMOOTHING_FACTOR * value as f64) + ((1.0 - EMA_SMOOTHING_FACTOR) * ema);
        }

        ema as u64
    }

    pub fn will_run_out_in_24h(&self) -> bool {
        if self.gas_burn_history.is_empty() {
            return false;
        }

        let burn_rate_per_ledger = self.calculate_burn_rate_ema();
        
        let projected_burn = match burn_rate_per_ledger.checked_mul(LEDGERS_PER_DAY) {
            Some(burn) => burn,
            None => return true,
        };

        let available = self.vault_balance.saturating_sub(self.min_reserve);
        available < projected_burn
    }

    pub fn available_balance(&self) -> u64 {
        self.vault_balance.saturating_sub(self.min_reserve)
    }

    pub fn deposit(&mut self, amount: u64) -> DomainResult<()> {
        self.vault_balance = self.vault_balance
            .checked_add(amount)
            .ok_or_else(|| DomainError::OverflowDetected {
                operation: "depositing funds".to_string(),
            })?;
        Ok(())
    }

    fn calculate_total_reserves(&self) -> DomainResult<u64> {
        let mut total: u64 = 0;
        for contract in &self.registered_contracts {
            total = total
                .checked_add(contract.safe_reserve)
                .ok_or_else(|| DomainError::OverflowDetected {
                    operation: "summing contract reserves".to_string(),
                })?;
        }
        Ok(total)
    }

    pub fn unregister_contract(&mut self, contract_id: &str) -> DomainResult<RegisteredContract> {
        let position = self.registered_contracts
            .iter()
            .position(|c| c.contract_id == contract_id)
            .ok_or_else(|| DomainError::ContractNotFound {
                contract_id: contract_id.to_string(),
            })?;

        Ok(self.registered_contracts.remove(position))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_vault_with_valid_params() {
        let vault = KeeperVault::new(
            "GABC123".to_string(),
            1_000_000_000,
            100_000_000,
        );
        assert!(vault.is_ok());
    }

    #[test]
    fn test_new_vault_with_invalid_reserve() {
        let vault = KeeperVault::new(
            "GABC123".to_string(),
            100_000_000,
            1_000_000_000,
        );
        assert!(matches!(vault, Err(DomainError::InvalidReserve { .. })));
    }

    #[test]
    fn test_register_contract_success() {
        let mut vault = KeeperVault::new(
            "GABC123".to_string(),
            1_000_000_000,
            100_000_000,
        ).unwrap();

        let contract = RegisteredContract {
            contract_id: "CONTRACT1".to_string(),
            safe_reserve: 50_000_000,
            assigned_gas_limit: 1_000_000,
        };

        assert!(vault.register_contract(contract).is_ok());
        assert_eq!(vault.registered_contracts.len(), 1);
    }

    #[test]
    fn test_register_contract_insufficient_funds() {
        let mut vault = KeeperVault::new(
            "GABC123".to_string(),
            200_000_000,
            100_000_000,
        ).unwrap();

        let contract = RegisteredContract {
            contract_id: "CONTRACT1".to_string(),
            safe_reserve: 150_000_000,
            assigned_gas_limit: 1_000_000,
        };

        let result = vault.register_contract(contract);
        assert!(matches!(result, Err(DomainError::InsufficientFunds { .. })));
    }

    #[test]
    fn test_gas_burn_tracking() {
        let mut vault = KeeperVault::new(
            "GABC123".to_string(),
            1_000_000_000,
            100_000_000,
        ).unwrap();

        vault.record_gas_burn(1_000_000).unwrap();
        vault.record_gas_burn(2_000_000).unwrap();
        vault.record_gas_burn(1_500_000).unwrap();

        assert_eq!(vault.gas_burn_history.len(), 3);
        assert_eq!(vault.vault_balance, 995_500_000);
    }

    #[test]
    fn test_ema_calculation() {
        let mut vault = KeeperVault::new(
            "GABC123".to_string(),
            1_000_000_000,
            100_000_000,
        ).unwrap();

        vault.record_gas_burn(1_000_000).unwrap();
        vault.record_gas_burn(2_000_000).unwrap();
        vault.record_gas_burn(1_500_000).unwrap();

        let ema = vault.calculate_burn_rate_ema();
        assert!(ema > 0);
        assert!(ema <= 2_000_000);
    }

    #[test]
    fn test_overflow_protection() {
        let mut vault = KeeperVault::new(
            "GABC123".to_string(),
            u64::MAX,
            0,
        ).unwrap();

        let result = vault.deposit(1);
        assert!(matches!(result, Err(DomainError::OverflowDetected { .. })));
    }
}
