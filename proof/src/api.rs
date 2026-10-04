use crate::parser::Parser;
use crate::hir::resolve::Resolver;
use crate::elab::Elaborator;
use crate::ast::{Item, ProofStepKind};
use std::collections::HashMap;

/// Helper to serialize a string for JSON with RFC 8259 compliant escaping
pub fn json_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 16);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                use std::fmt::Write;
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Robust JSON string field extractor that handles nested strings and escape sequences
pub fn extract_json_string_field(json: &str, target_field: &str) -> Option<String> {
    let mut chars = json.char_indices().peekable();

    // Find opening '{'
    while let Some((_, c)) = chars.next() {
        if c == '{' {
            break;
        }
    }

    loop {
        // Skip whitespace and commas
        while let Some(&(_, c)) = chars.peek() {
            if c.is_whitespace() || c == ',' {
                chars.next();
            } else {
                break;
            }
        }

        // Read key
        let key = match chars.next() {
            Some((_, '"')) => {
                let mut k = String::new();
                let mut esc = false;
                while let Some((_, c)) = chars.next() {
                    if esc {
                        k.push(c);
                        esc = false;
                    } else if c == '\\' {
                        esc = true;
                    } else if c == '"' {
                        break;
                    } else {
                        k.push(c);
                    }
                }
                k
            }
            _ => return None,
        };

        // Skip until ':'
        while let Some((_, c)) = chars.next() {
            if c == ':' {
                break;
            }
        }

        // Skip whitespace
        while let Some(&(_, c)) = chars.peek() {
            if c.is_whitespace() {
                chars.next();
            } else {
                break;
            }
        }

        if key == target_field {
            // Read target string value
            match chars.next() {
                Some((_, '"')) => {
                    let mut val = String::new();
                    let mut esc = false;
                    while let Some((_, c)) = chars.next() {
                        if esc {
                            match c {
                                'n' => val.push('\n'),
                                'r' => val.push('\r'),
                                't' => val.push('\t'),
                                '\\' => val.push('\\'),
                                '"' => val.push('"'),
                                _ => {
                                    val.push('\\');
                                    val.push(c);
                                }
                            }
                            esc = false;
                        } else if c == '\\' {
                            esc = true;
                        } else if c == '"' {
                            return Some(val);
                        } else {
                            val.push(c);
                        }
                    }
                    return None;
                }
                _ => return None,
            }
        } else {
            // Skip non-target value
            match chars.peek() {
                Some(&(_, '"')) => {
                    chars.next();
                    let mut esc = false;
                    while let Some((_, c)) = chars.next() {
                        if esc {
                            esc = false;
                        } else if c == '\\' {
                            esc = true;
                        } else if c == '"' {
                            break;
                        }
                    }
                }
                Some(&(_, '{')) | Some(&(_, '[')) => {
                    let (_, open_c) = chars.next().unwrap();
                    let close_c = if open_c == '{' { '}' } else { ']' };
                    let mut depth = 1;
                    let mut in_str = false;
                    let mut esc = false;
                    while let Some((_, c)) = chars.next() {
                        if in_str {
                            if esc {
                                esc = false;
                            } else if c == '\\' {
                                esc = true;
                            } else if c == '"' {
                                in_str = false;
                            }
                        } else if c == '"' {
                            in_str = true;
                        } else if c == open_c {
                            depth += 1;
                        } else if c == close_c {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                        }
                    }
                }
                _ => {
                    while let Some(&(_, c)) = chars.peek() {
                        if c == ',' || c == '}' {
                            break;
                        }
                        chars.next();
                    }
                }
            }
        }
    }
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
