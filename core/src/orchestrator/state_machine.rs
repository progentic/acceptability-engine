use crate::contract::Contract;
use std::path::PathBuf;

pub enum FinalDecision {
    Approve,
    Reject { reason: String },
}

pub struct Run {
    pub contract: Contract,
    pub workspace: PathBuf,
}
