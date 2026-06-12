#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageType {
    Persistent,
    Instance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractMonitor {
    pub contract_id: String,
    pub storage_type: StorageType,
    pub target_threshold: u32,
    pub live_ledger_expiration_bound: u32,
}

impl ContractMonitor {
    pub fn new(
        contract_id: String,
        storage_type: StorageType,
        target_threshold: u32,
        live_ledger_expiration_bound: u32,
    ) -> Self {
        Self {
            contract_id,
            storage_type,
            target_threshold,
            live_ledger_expiration_bound,
        }
    }

    pub fn remaining_life(&self, current_ledger: u32) -> Option<u32> {
        self.live_ledger_expiration_bound.checked_sub(current_ledger)
    }

    pub fn requires_extension(&self, current_ledger: u32) -> bool {
        match self.remaining_life(current_ledger) {
            Some(rem) => rem <= self.target_threshold,
            None => true,
        }
    }
}
