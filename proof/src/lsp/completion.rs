use crate::api::json_str;

/// Provides context-aware completion items for the Proofer language.
pub fn get_completions(doc: &str, line_idx: usize, char_idx: usize) -> String {
    let lines: Vec<&str> = doc.lines().collect();
    let line_prefix = if line_idx < lines.len() {
        let l = lines[line_idx];
        let safe_end = char_idx.min(l.len());
        &l[..safe_end]
    } else {
        ""
    };

    let trimmed = line_prefix.trim_start();
    let mut items = Vec::new();

    // 1. Completion after "using " -> suggest inference rules
    if line_prefix.contains("using ") || trimmed.ends_with("using") {
        let rules = [
            ("IsoscelesBaseAngles", "Rule: Base angles of an isosceles triangle are congruent"),
            ("VerticalAngles", "Rule: Vertically opposite angles are equal"),
            ("Thales", "Rule: Diameter inscribed triangle has right angle (90°)"),
            ("InscribedAngle", "Rule: Inscribed angle subtended by diameter is 90°"),
            ("PythagoreanTheorem", "Rule: a² + b² = c² in right triangles"),
            ("TriangleAngleSum", "Rule: Sum of angles in a triangle is 180°"),
            ("AlternateInteriorAngles", "Rule: Alternate interior angles of parallel lines are equal"),
            ("CongruentTrianglesSSS", "Rule: SSS triangle congruence"),
            ("CongruentTrianglesSAS", "Rule: SAS triangle congruence"),
            ("CongruentTrianglesASA", "Rule: ASA triangle congruence"),
        ];

        for (rule, doc) in rules {
            items.push(format!(
                "{{\"label\": {}, \"kind\": 7, \"detail\": \"Inference Rule\", \"documentation\": {}}}",
                json_str(rule),
                json_str(doc)
            ));
        }
        return format!("{{\"isIncomplete\": false, \"items\": [{}]}}", items.join(", "));
    }

    // 2. Completion after "from " -> suggest hypotheses in the current proof
    if line_prefix.contains("from ") || trimmed.ends_with("from") {
        let hypotheses = extract_hypotheses(doc);
        for (h, prop) in hypotheses {
            items.push(format!(
                "{{\"label\": {}, \"kind\": 6, \"detail\": \"Hypothesis\", \"documentation\": {}}}",
                json_str(&h),
                json_str(&format!("`{}`: {}", h, prop))
            ));
        }
        return format!("{{\"isIncomplete\": false, \"items\": [{}]}}", items.join(", "));
    }

    // 3. General keywords, proof verbs, and predicates
    let keywords = [
        ("theorem", "Declares a mathematical theorem", 14),
        ("proof", "Starts formal deduction proof block", 14),
        ("suppose", "Introduce a premise hypothesis", 3),
        ("derive", "Derive a proposition from hypotheses", 3),
        ("therefore", "Discharge the target theorem goal", 3),
        ("construct", "Construct a geometric auxiliary entity", 3),
        ("have", "Assert an intermediate fact", 3),
        ("end", "Conclude proof block", 14),
        ("diameter", "Geometric predicate for circle diameter", 2),
        ("on_circle", "Geometric predicate for boundary circle points", 2),
        ("intersect", "Geometric predicate for line intersection", 2),
        ("midpoint", "Constructs midpoint of a line segment", 2),
        ("altitude", "Constructs triangle altitude line", 2),
    ];

    for (kw, detail, kind) in keywords {
        items.push(format!(
            "{{\"label\": {}, \"kind\": {}, \"detail\": {}, \"documentation\": {}}}",
            json_str(kw),
            kind,
            json_str(detail),
            json_str(detail)
        ));
    }

    return format!("{{\"isIncomplete\": false, \"items\": [{}]}}", items.join(", "));
}

fn extract_hypotheses(doc: &str) -> Vec<(String, String)> {
    let mut hyps = Vec::new();
    for line in doc.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("suppose ") {
            let rest = &trimmed[8..];
            if let Some(pos) = rest.find(':') {
                let name = rest[..pos].trim().to_string();
                let prop = rest[pos + 1..].trim().to_string();
                hyps.push((name, prop));
            }
        } else if trimmed.starts_with("derive ") {
            let rest = &trimmed[7..];
            if let Some(pos) = rest.find(':') {
                let name = rest[..pos].trim().to_string();
                let after_colon = rest[pos + 1..].trim();
                let prop = after_colon.split(" from ").next().unwrap_or(after_colon).trim().to_string();
                hyps.push((name, prop));
            }
        } else if trimmed.starts_with("have ") {
            let rest = &trimmed[5..];
            if let Some(pos) = rest.find(':') {
                let name = rest[..pos].trim().to_string();
                let after_colon = rest[pos + 1..].trim();
                let prop = after_colon.split(" from ").next().unwrap_or(after_colon).trim().to_string();
                hyps.push((name, prop));
            }
        }
    }
    hyps
}
