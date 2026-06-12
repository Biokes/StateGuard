use crate::domain::contract_monitor::{ContractMonitor, StorageType};
use crate::ports::ITTLQueryPort;
use std::future::Future;

pub struct ZephyrIngestionAdapter {
    pub endpoint: String,
}

impl ZephyrIngestionAdapter {
    pub fn new(endpoint: String) -> Self {
        Self { endpoint }
    }
}

impl ITTLQueryPort for ZephyrIngestionAdapter {
    fn get_contract_ttl(
        &self,
        contract_id: &str,
    ) -> impl Future<Output = Result<ContractMonitor, String>> + Send {
        let contract_id = contract_id.to_string();
        async move {
            ContractMonitor::new(
                contract_id,
                StorageType::Persistent,
                1000,
                5000,
            )
            .map_err(|e| e.to_string())
        }
    }

    fn get_current_ledger(&self) -> impl Future<Output = Result<u32, String>> + Send {
        async move {
            // Mock implementation
            Ok(4000)
        }
    }
}
