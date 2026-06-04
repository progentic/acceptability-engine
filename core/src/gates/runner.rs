use crate::error::GateError;
use crate::gates::result::GateOutput;
use crate::orchestrator::state_machine::Run;
use crate::progress::ProgressPublisher;

pub async fn run_gates_sequential_with_progress(
    run: &Run,
    progress: &ProgressPublisher,
) -> Result<Vec<GateOutput>, GateError> {
    let mut results = Vec::with_capacity(8);

    progress.gate_started(1);
    let res1 = super::gate01_contract::run(run).await?;
    let output1 = GateOutput::Simple(res1);
    progress.gate_finished(&output1);
    if push_and_should_stop(&mut results, output1) {
        return Ok(results);
    }

    progress.gate_started(2);
    let res2 = super::gate02_workspace::run(run).await?;
    let output2 = GateOutput::Simple(res2);
    progress.gate_finished(&output2);
    if push_and_should_stop(&mut results, output2) {
        return Ok(results);
    }

    progress.gate_started(3);
    let res3 = super::gate03_boundary::run(run).await?;
    let output3 = GateOutput::Simple(res3);
    progress.gate_finished(&output3);
    if push_and_should_stop(&mut results, output3) {
        return Ok(results);
    }

    progress.gate_started(4);
    let res4 = super::gate04_formatting::run(run).await?;
    let output4 = GateOutput::Execution(res4);
    progress.gate_finished(&output4);
    if push_and_should_stop(&mut results, output4) {
        return Ok(results);
    }

    progress.gate_started(5);
    let res5 = super::gate05_static_checks::run(run).await?;
    let output5 = GateOutput::Execution(res5);
    progress.gate_finished(&output5);
    if push_and_should_stop(&mut results, output5) {
        return Ok(results);
    }

    progress.gate_started(6);
    let res6 = super::gate06_build::run(run).await?;
    let output6 = GateOutput::Execution(res6);
    progress.gate_finished(&output6);
    if push_and_should_stop(&mut results, output6) {
        return Ok(results);
    }

    progress.gate_started(7);
    let res7 = super::gate07_tests::run(run).await?;
    let output7 = GateOutput::Execution(res7);
    progress.gate_finished(&output7);
    if push_and_should_stop(&mut results, output7) {
        return Ok(results);
    }

    progress.gate_started(8);
    let res8 = super::gate08_supply_chain::run(run).await?;
    let output8 = GateOutput::Execution(res8);
    progress.gate_finished(&output8);
    if push_and_should_stop(&mut results, output8) {
        return Ok(results);
    }

    Ok(results)
}

fn push_and_should_stop(results: &mut Vec<GateOutput>, output: GateOutput) -> bool {
    let should_stop = !output.passed();
    results.push(output);
    should_stop
}
