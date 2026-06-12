use crate::domain::contract_monitor::ContractMonitor;
use crate::domain::execution_job::ExecutionJob;
use crate::domain::keeper_vault::KeeperVault;

pub trait ITTLQueryPort {
    fn get_contract_ttl(
        &self,
        contract_id: &str,
    ) -> impl std::future::Future<Output = Result<ContractMonitor, String>> + Send;
    
    fn get_current_ledger(
        &self,
    ) -> impl std::future::Future<Output = Result<u32, String>> + Send;
}

pub trait ITransactionSubmitPort {
    fn submit_job(
        &self,
        job: &ExecutionJob,
    ) -> impl std::future::Future<Output = Result<(), String>> + Send;
}

pub trait IVaultStoragePort {
    fn get_vault(
        &self,
        client_account: &str,
    ) -> impl std::future::Future<Output = Result<KeeperVault, String>> + Send;
    
    fn save_vault(
        &self,
        vault: &KeeperVault,
    ) -> impl std::future::Future<Output = Result<(), String>> + Send;
}
