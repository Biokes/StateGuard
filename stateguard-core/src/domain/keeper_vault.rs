#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeeperVault {
    pub client_account: String,
    pub vault_balance: u64,
    pub min_reserve: u64,
    pub registered_contracts: Vec<RegisteredContract>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisteredContract {
    pub contract_id: String,
    pub safe_reserve: u64,
    pub assigned_gas_limit: u64,
}

impl KeeperVault {
    pub fn new(client_account: String, vault_balance: u64, min_reserve: u64) -> Self {
        Self {
            client_account,
            vault_balance,
            min_reserve,
            registered_contracts: Vec::new(),
        }
    }

    /// Protects the invariant: Vault Balance >= Min Reserve + Sum(Target Contract Safe Reserves)
    pub fn can_afford_new_contract(&self, contract_safe_reserve: u64) -> bool {
        let total_reserves: u64 = self.registered_contracts.iter().map(|c| c.safe_reserve).sum();
        self.vault_balance >= self.min_reserve + total_reserves + contract_safe_reserve
    }

    pub fn register_contract(&mut self, contract: RegisteredContract) -> Result<(), String> {
        if !self.can_afford_new_contract(contract.safe_reserve) {
            return Err("Insufficient vault balance to register contract".to_string());
        }
        self.registered_contracts.push(contract);
        Ok(())
    }
}
