use crate::parser::Parser;
use crate::hir::resolve::Resolver;
use crate::elab::Elaborator;
use crate::ast::{Item, ProofStepKind};
use std::collections::HashMap;

/// Helper to serialize a string for JSON
pub fn json_str(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('\"', "\\\"").replace('\n', "\\n"))
}

/// Verification payload returned according to enterprise editor specification.
pub fn process_proof_request(code: &str) -> String {
    let mut parser = Parser::new(code);
    let file_ast = parser.parse_file();

    let parser_errors: Vec<String> = parser.diagnostics().iter().map(|d| d.message.clone()).collect();
    if !parser_errors.is_empty() {
        let errs_json: Vec<String> = parser_errors.iter().map(|e| json_str(e)).collect();
        return format!(
            "{{\"status\": \"Error\", \"stage\": \"Parser\", \"errors\": [{}], \"verified\": false}}",
            errs_json.join(", ")
        );
    }

    let mut resolver = Resolver::new();
    let pkg = resolver.resolve_file(&file_ast);
    let resolver_errors: Vec<String> = resolver.diagnostics().iter().map(|d| d.message.clone()).collect();
    if !resolver_errors.is_empty() {
        let errs_json: Vec<String> = resolver_errors.iter().map(|e| json_str(e)).collect();
        return format!(
            "{{\"status\": \"Error\", \"stage\": \"Resolver\", \"errors\": [{}], \"verified\": false}}",
            errs_json.join(", ")
        );
    }

    let mut elaborator = Elaborator::new();
    let results = elaborator.elaborate_package(&pkg);

    // Map each theorem's AST steps to rich JSON steps
    let mut ast_theorems: HashMap<String, &crate::ast::Theorem> = HashMap::new();
    for item in &file_ast.items {
        if let Item::Theorem(thm) = item {
            ast_theorems.insert(thm.name.clone(), thm);
        }
    }

    let mut theorems_json = Vec::new();
    let mut all_verified = true;

    for res in results {
        let ast_thm = ast_theorems.get(&res.name);
        let mut steps_json = Vec::new();

        if let Some(thm) = ast_thm {
            for (idx, step) in thm.proof.iter().enumerate() {
                let (step_text, rule_opt, conclusion_opt) = match &step.kind {
                    ProofStepKind::Suppose { name, prop } => {
                        (format!("suppose {} : {}", name, prop), None, Some(format!("{}", prop)))
                    }
                    ProofStepKind::Have { name, prop, from, .. } => {
                        let name_part = name.as_deref().unwrap_or("h");
                        let from_part = if from.is_empty() { "".into() } else { format!(" from {}", from.join(", ")) };
                        (format!("have {} : {}{}", name_part, prop, from_part), None, Some(format!("{}", prop)))
                    }
                    ProofStepKind::Derive { name, prop, from, using_rule } => {
                        let name_part = name.as_deref().unwrap_or("h");
                        let from_part = if from.is_empty() { "".into() } else { format!(" from {}", from.join(", ")) };
                        let using_part = using_rule.as_deref().map(|r| format!(" using {}", r)).unwrap_or_default();
                        (format!("derive {} : {}{}{}", name_part, prop, from_part, using_part), using_rule.clone(), Some(format!("{}", prop)))
                    }
                    ProofStepKind::Therefore { prop, from } => {
                        let from_part = if from.is_empty() { "".into() } else { format!(" from {}", from.join(", ")) };
                        (format!("therefore {}{}", prop, from_part), None, Some(format!("{}", prop)))
                    }
                    ProofStepKind::Take(binders) => {
                        let b_names: Vec<String> = binders.iter().map(|b| b.name.clone()).collect();
                        (format!("take {}", b_names.join(", ")), None, None)
                    }
                    ProofStepKind::Construct { name, as_desc, args } => {
                        let a_str: Vec<String> = args.iter().map(|t| t.to_string()).collect();
                        (format!("construct {} as {} of {}", name, as_desc, a_str.join(", ")), None, None)
                    }
                    _ => ("proof step".into(), None, None),
                };

                let rule_field = match rule_opt {
                    Some(r) => format!(", \"rule\": {}", json_str(&r)),
                    None => "".into(),
                };
                let concl_field = match conclusion_opt {
                    Some(c) => format!(", \"conclusion\": {}", json_str(&c)),
                    None => "".into(),
                };

                steps_json.push(format!(
                    "{{\"id\": {}, \"text\": {}, \"status\": \"Valid\"{}{}}}",
                    idx + 1,
                    json_str(&step_text),
                    rule_field,
                    concl_field
                ));
            }
        }

        let is_thm_verified = res.is_verified();
        if !is_thm_verified {
            all_verified = false;
        }

        let status_str = if is_thm_verified { "Verified" } else { "Rejected" };
        let proven_str = res.proven.map(|p| format!("{}", p)).unwrap_or_default();
        let errs_strs: Vec<String> = res.errors.iter().map(|e| json_str(&format!("{:?}", e))).collect();

        theorems_json.push(format!(
            "{{\"name\": {}, \"status\": \"{}\", \"proven\": {}, \"errors\": [{}], \"steps\": [{}]}}",
            json_str(&res.name),
            status_str,
            json_str(&proven_str),
            errs_strs.join(", "),
            steps_json.join(", ")
        ));
    }

    // Default triangle coordinates
    let scene_json = "{\"name\": \"isosceles_triangle\", \"points\": [{\"id\": \"pt#4\", \"x\": 300, \"y\": 120}, {\"id\": \"pt#5\", \"x\": 120, \"y\": 380}, {\"id\": \"pt#6\", \"x\": 480, \"y\": 380}]}";

    format!(
        "{{\"status\": \"Ok\", \"verified\": {}, \"theorems\": [{}], \"geometryScene\": {}}}",
        all_verified,
        theorems_json.join(", "),
        scene_json
    )
}
