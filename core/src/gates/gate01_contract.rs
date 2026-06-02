use crate::error::GateError;
use crate::gates::result::GateResult;
use crate::orchestrator::state_machine::Run;

// Required for compatibility with validation sequence loop interfaces.
// This step evaluates static types and triggers zero asynchronous I/O.
pub async fn run(run: &Run) -> Result<GateResult, GateError> {
    run.contract.validate()?;
    Ok(GateResult::pass(1, "Contract schema validation successful"))
}
