use crate::domain::execution_job::ExecutionJob;
use crate::ports::ITransactionSubmitPort;
use std::future::Future;

pub struct StellarTxSubmitterAdapter {
    pub rpc_url: String,
}

impl StellarTxSubmitterAdapter {
    pub fn new(rpc_url: String) -> Self {
        Self { rpc_url }
    }
}

impl ITransactionSubmitPort for StellarTxSubmitterAdapter {
    fn submit_job(
        &self,
        _job: &ExecutionJob,
    ) -> impl Future<Output = Result<(), String>> + Send {
        async move {
            Ok(())
        }
    }
}
