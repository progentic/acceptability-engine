use crate::contract::Contract;
use crate::error::validation::ValidationError;
use crate::gates::result::GateOutput;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

const STRICT_POLICY_ID: &str = "strict-v1";
const STRICT_POLICY_VERSION: u32 = 1;
const MANDATORY_GATES: [u8; 8] = [1, 2, 3, 4, 5, 6, 7, 8];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdmissionPolicy {
    #[serde(default = "default_policy_id")]
    pub id: String,
    #[serde(default = "default_policy_version")]
    pub version: u32,
    #[serde(default)]
    pub rules: AdmissionPolicyRules,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdmissionPolicyRules {
    #[serde(default = "default_true")]
    pub require_all_gates_pass: bool,
    #[serde(default = "default_required_gates")]
    pub required_gates: Vec<u8>,
    #[serde(default = "default_max_test_parse_errors")]
    pub max_test_parse_errors: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PolicyEvaluation {
    pub policy_id: String,
    pub policy_version: u32,
    pub passed: bool,
    pub reason: String,
    pub trace_json: String,
}

#[derive(Serialize)]
struct PolicyTrace {
    candidate: CandidateTrace,
    policy_id: String,
    policy_version: u32,
    required_gates: Vec<u8>,
    gate_results: Vec<GateTrace>,
}

#[derive(Serialize)]
struct CandidateTrace {
    repo_url: String,
    base_sha: String,
    candidate_sha: String,
    candidate_ref: Option<String>,
}

#[derive(Serialize)]
struct GateTrace {
    gate_num: u8,
    passed: bool,
    message: String,
    parse_errors: Option<u32>,
}

impl Default for AdmissionPolicy {
    fn default() -> Self {
        Self {
            id: default_policy_id(),
            version: default_policy_version(),
            rules: AdmissionPolicyRules::default(),
        }
    }
}

impl Default for AdmissionPolicyRules {
    fn default() -> Self {
        Self {
            require_all_gates_pass: default_true(),
            required_gates: default_required_gates(),
            max_test_parse_errors: default_max_test_parse_errors(),
        }
    }
}

impl AdmissionPolicy {
    pub fn validate(&self) -> Result<(), ValidationError> {
        validate_policy_id(&self.id)?;
        validate_policy_version(self.version)?;
        validate_policy_rules(&self.rules)
    }
}

pub fn evaluate_policy(
    gate_outputs: &[GateOutput],
    policy: &AdmissionPolicy,
    contract: &Contract,
) -> PolicyEvaluation {
    let trace = policy_trace(gate_outputs, policy, contract);
    let reason = policy_rejection_reason(gate_outputs, &policy.rules);
    PolicyEvaluation {
        policy_id: policy.id.clone(),
        policy_version: policy.version,
        passed: reason.is_none(),
        reason: reason.unwrap_or_else(|| "admission policy passed".to_string()),
        trace_json: serde_json::to_string(&trace).unwrap_or_else(|_| "{}".to_string()),
    }
}

fn validate_policy_id(policy_id: &str) -> Result<(), ValidationError> {
    if policy_id.trim().is_empty() {
        return Err(ValidationError::InvalidPolicy(
            "policy id is empty".to_string(),
        ));
    }
    if !policy_id.chars().all(is_policy_id_character) {
        return Err(ValidationError::InvalidPolicy(format!(
            "policy id '{policy_id}' contains unsupported characters"
        )));
    }
    if policy_id != STRICT_POLICY_ID {
        return Err(ValidationError::InvalidPolicy(format!(
            "policy id '{policy_id}' is not supported"
        )));
    }
    Ok(())
}

fn validate_policy_version(version: u32) -> Result<(), ValidationError> {
    if version == 0 {
        return Err(ValidationError::InvalidPolicy(
            "policy version must be greater than zero".to_string(),
        ));
    }
    if version != STRICT_POLICY_VERSION {
        return Err(ValidationError::InvalidPolicy(format!(
            "policy version '{version}' is not supported"
        )));
    }
    Ok(())
}

fn validate_policy_rules(rules: &AdmissionPolicyRules) -> Result<(), ValidationError> {
    validate_gate_pass_rule(rules)?;
    validate_required_gates(&rules.required_gates)
}

fn validate_gate_pass_rule(rules: &AdmissionPolicyRules) -> Result<(), ValidationError> {
    if rules.require_all_gates_pass {
        return Ok(());
    }
    Err(ValidationError::InvalidPolicy(
        "policy cannot disable mandatory gate pass requirements".to_string(),
    ))
}

fn validate_required_gates(required_gates: &[u8]) -> Result<(), ValidationError> {
    let unique_gates: BTreeSet<u8> = required_gates.iter().copied().collect();
    if unique_gates.len() != required_gates.len() {
        return Err(ValidationError::InvalidPolicy(
            "policy required gates must be unique".to_string(),
        ));
    }
    if required_gates != MANDATORY_GATES {
        return Err(ValidationError::InvalidPolicy(
            "policy required gates must be ordered 1 through 8".to_string(),
        ));
    }
    if unique_gates != mandatory_gate_set() {
        return Err(ValidationError::InvalidPolicy(
            "policy must require gates 1 through 8".to_string(),
        ));
    }
    Ok(())
}

fn policy_rejection_reason(
    gate_outputs: &[GateOutput],
    rules: &AdmissionPolicyRules,
) -> Option<String> {
    failed_gate_reason(gate_outputs, rules)
        .or_else(|| missing_required_gate(gate_outputs, &rules.required_gates))
        .or_else(|| test_parse_error_reason(gate_outputs, rules))
}

fn missing_required_gate(gate_outputs: &[GateOutput], required_gates: &[u8]) -> Option<String> {
    let executed_gates: BTreeSet<u8> = gate_outputs.iter().map(GateOutput::gate_num).collect();
    required_gates
        .iter()
        .find(|gate_num| !executed_gates.contains(gate_num))
        .map(|gate_num| format!("Admission policy missing required gate {gate_num}."))
}

fn failed_gate_reason(gate_outputs: &[GateOutput], rules: &AdmissionPolicyRules) -> Option<String> {
    if !rules.require_all_gates_pass {
        return None;
    }
    gate_outputs
        .iter()
        .find(|output| !output.passed())
        .map(|output| {
            format!(
                "Admission policy rejected gate {}: {}",
                output.gate_num(),
                output.message()
            )
        })
}

fn test_parse_error_reason(
    gate_outputs: &[GateOutput],
    rules: &AdmissionPolicyRules,
) -> Option<String> {
    let max_parse_errors = rules.max_test_parse_errors?;
    gate_outputs
        .iter()
        .filter_map(test_parse_errors)
        .find(|parse_errors| *parse_errors > max_parse_errors)
        .map(|parse_errors| {
            format!(
                "Admission policy rejected test parse errors: {parse_errors} exceeds {max_parse_errors}."
            )
        })
}

fn policy_trace(
    gate_outputs: &[GateOutput],
    policy: &AdmissionPolicy,
    contract: &Contract,
) -> PolicyTrace {
    PolicyTrace {
        candidate: candidate_trace(contract),
        policy_id: policy.id.clone(),
        policy_version: policy.version,
        required_gates: policy.rules.required_gates.clone(),
        gate_results: gate_outputs.iter().map(gate_trace).collect(),
    }
}

fn candidate_trace(contract: &Contract) -> CandidateTrace {
    CandidateTrace {
        repo_url: contract.repo_url.clone(),
        base_sha: contract.base_sha.clone(),
        candidate_sha: contract.candidate_sha.clone(),
        candidate_ref: contract.candidate_ref.clone(),
    }
}

fn gate_trace(output: &GateOutput) -> GateTrace {
    GateTrace {
        gate_num: output.gate_num(),
        passed: output.passed(),
        message: output.message().to_string(),
        parse_errors: test_parse_errors(output),
    }
}

fn test_parse_errors(output: &GateOutput) -> Option<u32> {
    match output {
        GateOutput::Simple(_) => None,
        GateOutput::Execution(result) => result
            .test_metrics
            .as_ref()
            .map(|metrics| metrics.parse_errors),
    }
}

fn mandatory_gate_set() -> BTreeSet<u8> {
    MANDATORY_GATES.into_iter().collect()
}

fn is_policy_id_character(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
}

fn default_policy_id() -> String {
    STRICT_POLICY_ID.to_string()
}

fn default_policy_version() -> u32 {
    STRICT_POLICY_VERSION
}

fn default_true() -> bool {
    true
}

fn default_required_gates() -> Vec<u8> {
    MANDATORY_GATES.to_vec()
}

fn default_max_test_parse_errors() -> Option<u32> {
    Some(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gates::result::{ExecutionResult, GateOutput, GateResult, TestMetrics};

    #[test]
    fn default_policy_passes_all_required_gates() {
        let outputs = passing_gates();

        let evaluation = evaluate_policy(&outputs, &AdmissionPolicy::default(), &test_contract());

        assert!(evaluation.passed);
    }

    #[test]
    fn policy_rejects_missing_required_gate() {
        let outputs = vec![GateOutput::Simple(GateResult::pass(1, "ok"))];

        let evaluation = evaluate_policy(&outputs, &AdmissionPolicy::default(), &test_contract());

        assert!(!evaluation.passed);
        assert!(evaluation.reason.contains("missing required gate 2"));
    }

    #[test]
    fn policy_rejects_test_parse_errors() {
        let mut outputs = passing_gates();
        let GateOutput::Execution(test_result) = &mut outputs[6] else {
            panic!("gate 7 must be execution output");
        };
        test_result.test_metrics = Some(TestMetrics {
            parse_errors: 1,
            ..TestMetrics::default()
        });

        let evaluation = evaluate_policy(&outputs, &AdmissionPolicy::default(), &test_contract());

        assert!(!evaluation.passed);
        assert!(evaluation.reason.contains("parse errors"));
    }

    #[test]
    fn policy_trace_records_candidate_identity() {
        let outputs = passing_gates();

        let evaluation = evaluate_policy(&outputs, &AdmissionPolicy::default(), &test_contract());

        assert!(evaluation.trace_json.contains("\"candidate_sha\""));
        assert!(evaluation
            .trace_json
            .contains("b9993e364706816aba3e25717850c26c9cd0d89d"));
    }

    #[test]
    fn policy_cannot_disable_mandatory_gate_passes() {
        let policy = AdmissionPolicy {
            rules: AdmissionPolicyRules {
                require_all_gates_pass: false,
                ..AdmissionPolicyRules::default()
            },
            ..AdmissionPolicy::default()
        };

        assert!(policy.validate().is_err());
    }

    #[test]
    fn unsupported_policy_id_fails_closed() {
        let policy = AdmissionPolicy {
            id: "relaxed-v1".to_string(),
            ..AdmissionPolicy::default()
        };

        assert!(policy.validate().is_err());
    }

    #[test]
    fn unsupported_policy_version_fails_closed() {
        let policy = AdmissionPolicy {
            version: 2,
            ..AdmissionPolicy::default()
        };

        assert!(policy.validate().is_err());
    }

    #[test]
    fn unordered_required_gates_fail_closed() {
        let policy = AdmissionPolicy {
            rules: AdmissionPolicyRules {
                required_gates: vec![2, 1, 3, 4, 5, 6, 7, 8],
                ..AdmissionPolicyRules::default()
            },
            ..AdmissionPolicy::default()
        };

        assert!(policy.validate().is_err());
    }

    fn passing_gates() -> Vec<GateOutput> {
        vec![
            simple_pass(1),
            simple_pass(2),
            simple_pass(3),
            execution_pass(4),
            execution_pass(5),
            execution_pass(6),
            execution_pass(7),
            execution_pass(8),
        ]
    }

    fn test_contract() -> Contract {
        Contract {
            id: "run-001".to_string(),
            repo_url: "https://github.com/progentic/acceptability-engine.git".to_string(),
            base_sha: "a9993e364706816aba3e25717850c26c9cd0d89d".to_string(),
            candidate_sha: "b9993e364706816aba3e25717850c26c9cd0d89d".to_string(),
            candidate_ref: Some("refs/pull/1/head".to_string()),
            scopes: vec!["core/src".to_string()],
            requires_human_review: false,
            admission_policy: AdmissionPolicy::default(),
        }
    }

    fn simple_pass(gate_num: u8) -> GateOutput {
        GateOutput::Simple(GateResult::pass(gate_num, "ok"))
    }

    fn execution_pass(gate_num: u8) -> GateOutput {
        GateOutput::Execution(ExecutionResult::pass(
            gate_num,
            "ok",
            0,
            1,
            Vec::new(),
            Vec::new(),
        ))
    }
}
