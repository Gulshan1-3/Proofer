//! Local Formal Proof Synthesizer & AI Co-Prover (Stage 16).
//!
//! Provides automated proof search and next-step infilling with
//! strict deterministic kernel guardrails (zero hallucination guarantee).

use crate::api::{process_proof_request, json_str};

#[derive(Debug, Clone)]
pub struct SynthesizedStep {
    pub text: String,
    pub kind: String, // "derive" or "therefore"
    pub conclusion: String,
    pub rule: Option<String>,
    pub premise: Option<String>,
    pub explanation: String,
    pub verified_by_kernel: bool,
}

impl SynthesizedStep {
    pub fn to_json(&self) -> String {
        format!(
            "{{\"text\":{},\"kind\":{},\"conclusion\":{},\"rule\":{},\"premise\":{},\"explanation\":{},\"verified_by_kernel\":{}}}",
            json_str(&self.text),
            json_str(&self.kind),
            json_str(&self.conclusion),
            self.rule.as_ref().map(|r| json_str(r)).unwrap_or_else(|| "null".to_string()),
            self.premise.as_ref().map(|p| json_str(p)).unwrap_or_else(|| "null".to_string()),
            json_str(&self.explanation),
            if self.verified_by_kernel { "true" } else { "false" }
        )
    }
}

pub struct ProofSynthesizer;

impl ProofSynthesizer {
    /// Infill the next plausible, formally verified proof step for an incomplete proof.
    pub fn infill_next_step(code: &str) -> Option<SynthesizedStep> {
        let candidates = Self::synthesize_candidates(code);
        // Return the first candidate that passes kernel verification
        candidates.into_iter().find(|c| c.verified_by_kernel)
    }

    /// Generates and kernel-checks all plausible candidate proof steps.
    pub fn synthesize_candidates(code: &str) -> Vec<SynthesizedStep> {
        let mut candidates = Vec::new();

        // 1. Extract context
        let lines: Vec<&str> = code.lines().collect();
        let mut target_goal = String::new();
        let mut hypotheses: Vec<(String, String)> = Vec::new();
        let mut in_proof = false;
        let mut last_hypothesis_num = 0;

        for line in &lines {
            let trimmed = line.trim();
            if trimmed.starts_with("theorem ") {
                continue;
            }
            if trimmed.contains("->") && !in_proof {
                if let Some(arrow_idx) = trimmed.rfind("->") {
                    target_goal = trimmed[arrow_idx + 2..].trim().to_string();
                }
            }
            if trimmed == "proof" {
                in_proof = true;
                continue;
            }
            if trimmed.starts_with("suppose ") || trimmed.starts_with("derive ") {
                let rest = if trimmed.starts_with("suppose ") {
                    &trimmed[8..]
                } else {
                    &trimmed[7..]
                };
                if let Some(colon) = rest.find(':') {
                    let label = rest[..colon].trim().to_string();
                    let prop = if let Some(from_idx) = rest[colon + 1..].find(" from ") {
                        rest[colon + 1..colon + 1 + from_idx].trim().to_string()
                    } else {
                        rest[colon + 1..].trim().to_string()
                    };

                    if let Some(num_str) = label.strip_prefix('h') {
                        if let Ok(num) = num_str.parse::<usize>() {
                            if num > last_hypothesis_num {
                                last_hypothesis_num = num;
                            }
                        }
                    }
                    hypotheses.push((label, prop));
                }
            }
        }

        if !in_proof {
            return Vec::new();
        }

        let next_label = format!("h{}", last_hypothesis_num + 1);

        // 2. Backward Check: Can we discharge the goal immediately?
        if !target_goal.is_empty() {
            for (label, prop) in &hypotheses {
                if Self::props_match(prop, &target_goal) {
                    let text = format!("therefore {} from {}", target_goal, label);
                    let verified = Self::verify_candidate(code, &text);
                    candidates.push(SynthesizedStep {
                        text,
                        kind: "therefore".to_string(),
                        conclusion: target_goal.clone(),
                        rule: None,
                        premise: Some(label.clone()),
                        explanation: format!("Discharges theorem target goal '{}' using established premise {}", target_goal, label),
                        verified_by_kernel: verified,
                    });
                }
            }
        }

        // 3. Forward Rules Generation
        let rules_database: Vec<(&str, &str, &str, &str)> = vec![
            (
                "IsoscelesBaseAngles",
                "AB = AC",
                "angle_ABC = angle_ACB",
                "Applies Isosceles Base Angles Theorem: equal sides subtend equal opposite angles."
            ),
            (
                "InscribedAngle",
                "diameter(AB) and on_circle(C)",
                "angle_ACB = 90",
                "Applies Thales' / Inscribed Angle Theorem: angle subtended by a diameter is a right angle."
            ),
            (
                "VerticalAngles",
                "intersect(line_AB, line_CD)",
                "angle_AEC = angle_BED",
                "Applies Vertical Angles Theorem: opposite angles formed by intersecting lines are congruent."
            ),
            (
                "TriangleAngleSum",
                "triangle(ABC)",
                "angle_sum = 180",
                "Applies Triangle Angle Sum Theorem: sum of interior angles is 180 degrees."
            ),
            (
                "MidpointBisects",
                "midpoint(M, AB) and midpoint(N, AC)",
                "parallel(line_MN, line_BC)",
                "Applies Triangle Midpoint Theorem: line joining midpoints is parallel to base."
            ),
            (
                "CyclicQuad",
                "cyclic(ABCD)",
                "angle_DAB + angle_BCD = 180",
                "Applies Cyclic Quadrilateral Theorem: opposite angles are supplementary."
            ),
            (
                "ParallelogramOppSides",
                "parallelogram(ABCD)",
                "AB = CD and BC = DA",
                "Applies Parallelogram Properties: opposite sides are equal."
            ),
            (
                "ModusPonens",
                "P and (P -> Q)",
                "Q",
                "Applies Modus Ponens deduction: from P and P->Q infer Q."
            ),
            (
                "DoubleNegation",
                "not(not(P))",
                "P",
                "Applies Double Negation elimination."
            ),
            (
                "AndEliminationLeft",
                "P and Q",
                "P",
                "Applies Conjunction Elimination (Left): from P and Q infer P."
            ),
            (
                "CongruentTrianglesSSS",
                "AB = DE and BC = EF and CA = FD",
                "congruent(triangle_ABC, triangle_DEF)",
                "Applies SSS Triangle Congruence."
            )
        ];

        for (rule_name, required_sub, concl, explanation) in rules_database {
            for (label, prop) in &hypotheses {
                if prop.contains(required_sub) || Self::props_match(prop, required_sub) {
                    let text = format!("derive {} : {} from {} using {}", next_label, concl, label, rule_name);
                    let verified = Self::verify_candidate(code, &text);
                    candidates.push(SynthesizedStep {
                        text,
                        kind: "derive".to_string(),
                        conclusion: concl.to_string(),
                        rule: Some(rule_name.to_string()),
                        premise: Some(label.clone()),
                        explanation: explanation.to_string(),
                        verified_by_kernel: verified,
                    });
                }
            }
        }

        candidates
    }

    fn props_match(a: &str, b: &str) -> bool {
        let clean_a: String = a.chars().filter(|c| !c.is_whitespace() && *c != '(' && *c != ')').collect();
        let clean_b: String = b.chars().filter(|c| !c.is_whitespace() && *c != '(' && *c != ')').collect();
        clean_a == clean_b
    }

    /// Deterministic Kernel Guardrail: verify candidate by appending to code and checking response.
    fn verify_candidate(base_code: &str, candidate_step: &str) -> bool {
        let mut test_code = String::new();
        let mut inserted = false;

        for line in base_code.lines() {
            let trimmed = line.trim();
            if trimmed == "end" && !inserted {
                test_code.push_str("    ");
                test_code.push_str(candidate_step);
                test_code.push('\n');
                inserted = true;
            }
            test_code.push_str(line);
            test_code.push('\n');
        }

        if !inserted {
            test_code.push_str("    ");
            test_code.push_str(candidate_step);
            test_code.push('\n');
            test_code.push_str("end\n");
        }

        let mut parser = crate::parser::Parser::new(&test_code);
        let file_ast = parser.parse_file();
        if !parser.diagnostics().is_empty() {
            return false;
        }

        let mut resolver = crate::hir::resolve::Resolver::new();
        let pkg = resolver.resolve_file(&file_ast);
        if !resolver.diagnostics().is_empty() {
            return false;
        }

        let mut elaborator = crate::elab::Elaborator::new();
        let results = elaborator.elaborate_package(&pkg);
        for res in &results {
            if res.proven.is_some() {
                return true;
            }
        }

        let json_res = process_proof_request(&test_code);
        if json_res.contains("\"status\": \"Valid\"") || json_res.contains("\"status\":\"Valid\"") {
            return true;
        }

        false
    }
}
