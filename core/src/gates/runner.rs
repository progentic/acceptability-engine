use crate::error::GateError;
use crate::gates::result::GateOutput;
use crate::orchestrator::state_machine::Run;

pub async fn run_gates_sequential(run: &Run) -> Result<Vec<GateOutput>, GateError> {
    let mut results = Vec::with_capacity(7);

    let res1 = super::gate01_contract::run(run).await?;
    let passed1 = res1.passed;
    results.push(GateOutput::Simple(res1));
    if !passed1 {
        return Ok(results);
    }

    let res2 = super::gate02_workspace::run(run).await?;
    let passed2 = res2.passed;
    results.push(GateOutput::Simple(res2));
    if !passed2 {
        return Ok(results);
    }

    let res3 = super::gate03_boundary::run(run).await?;
    let passed3 = res3.passed;
    results.push(GateOutput::Simple(res3));
    if !passed3 {
        return Ok(results);
    }

    let res4 = super::gate04_formatting::run(run).await?;
    let passed4 = res4.base.passed;
    results.push(GateOutput::Execution(res4));
    if !passed4 {
        return Ok(results);
    }

    let res5 = super::gate05_static_checks::run(run).await?;
    let passed5 = res5.base.passed;
    results.push(GateOutput::Execution(res5));
    if !passed5 {
        return Ok(results);
    }

    let res6 = super::gate06_build::run(run).await?;
    let passed6 = res6.base.passed;
    results.push(GateOutput::Execution(res6));
    if !passed6 {
        return Ok(results);
    }

    let res7 = super::gate07_tests::run(run).await?;
    let passed7 = res7.base.passed;
    results.push(GateOutput::Execution(res7));
    if !passed7 {
        return Ok(results);
    }

    Ok(results)
}
