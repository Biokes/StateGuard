#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobStatus {
    Pending,
    Signed,
    Submitted,
    Confirmed,
    Failed(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionJob {
    pub job_id: String,
    pub target_contract_id: String,
    pub payload: Vec<u8>,
    pub max_gas_price: u64,
    pub status: JobStatus,
    pub retries: u32,
}

impl ExecutionJob {
    pub fn new(job_id: String, target_contract_id: String, payload: Vec<u8>, max_gas_price: u64) -> Self {
        Self {
            job_id,
            target_contract_id,
            payload,
            max_gas_price,
            status: JobStatus::Pending,
            retries: 0,
        }
    }

    pub fn mark_signed(&mut self) {
        if self.status == JobStatus::Pending {
            self.status = JobStatus::Signed;
        }
    }

    pub fn mark_submitted(&mut self) {
        if self.status == JobStatus::Signed {
            self.status = JobStatus::Submitted;
        }
    }

    pub fn mark_confirmed(&mut self) {
        if self.status == JobStatus::Submitted {
            self.status = JobStatus::Confirmed;
        }
    }

    pub fn mark_failed(&mut self, reason: String) {
        self.status = JobStatus::Failed(reason);
    }

    pub fn increment_retry(&mut self) {
        self.retries += 1;
        self.status = JobStatus::Pending; // Reset status for retry
    }
}
