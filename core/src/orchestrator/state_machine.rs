use std::path::PathBuf;
use crate::contract::Contract;

pub enum FinalDecision {
    Approve,
    Reject { reason: String },
}

pub struct Run {
    pub run_id: i64,
    pub contract: Contract,
    pub workspace: PathBuf,
}
