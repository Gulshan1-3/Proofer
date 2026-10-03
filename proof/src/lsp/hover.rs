use crate::api::json_str;

/// Provides markdown hover documentation for symbols under the cursor.
pub fn get_hover_markdown(doc: &str, line_idx: usize, char_idx: usize) -> Option<String> {
    let lines: Vec<&str> = doc.lines().collect();
    if line_idx >= lines.len() {
        return None;
    }

    let line = lines[line_idx];
    let (word, start_col, end_col) = find_word_at_pos(line, char_idx)?;

    let hover_text = match word {
        // Geometric & Formal Rules
        "IsoscelesBaseAngles" => Some(
            "### Rule: Isosceles Base Angles\n\n$$\\triangle ABC, AB = AC \\implies \\angle ABC = \\angle ACB$$\n\nIn an isosceles triangle, the angles opposite to the equal sides are congruent.\n\n**Premises**: Equality of two triangle sides (`AB = AC`).\n**Conclusion**: Equality of the corresponding base angles."
        ),
        "VerticalAngles" => Some(
            "### Rule: Vertical Angles\n\n$$\\text{intersect}(line\\_AB, line\\_CD) \\implies \\angle AEC = \\angle BED$$\n\nVertical angles formed by two intersecting lines are equal."
        ),
        "Thales" => Some(
            "### Rule: Thales' Theorem\n\n$$\\text{diameter}(AB) \\land \\text{on\\_circle}(C) \\implies \\angle ACB = 90^\\circ$$\n\nAn angle inscribed in a semicircle is a right angle ($90^\\circ$)."
        ),
        "InscribedAngle" => Some(
            "### Rule: Inscribed Angle Theorem\n\n$$\\text{diameter}(AB) \\land \\text{on\\_circle}(C) \\implies \\angle ACB = 90^\\circ$$\n\nAn angle inscribed in a circle with a diameter subtending the arc equals $90^\\circ$."
        ),
        "PythagoreanTheorem" => Some(
            "### Rule: Pythagorean Theorem\n\n$$\\angle C = 90^\\circ \\implies a^2 + b^2 = c^2$$\n\nIn a right triangle, the square of the hypotenuse is equal to the sum of the squares of the other two sides."
        ),
        "TriangleAngleSum" => Some(
            "### Rule: Triangle Angle Sum\n\n$$\\angle A + \\angle B + \\angle C = 180^\\circ$$\n\nThe sum of interior angles in any Euclidean triangle equals $180^\\circ$."
        ),
        "AlternateInteriorAngles" => Some(
            "### Rule: Alternate Interior Angles\n\n$$l_1 \\parallel l_2 \\implies \\angle 1 = \\angle 2$$\n\nParallel lines intersected by a transversal produce equal alternate interior angles."
        ),
        "CongruentTrianglesSSS" => Some(
            "### Rule: SSS Triangle Congruence\n\n$$AB = DE \\land BC = EF \\land AC = DF \\implies \\triangle ABC \\cong \\triangle DEF$$"
        ),

        // Proof Keywords & Verbs
        "theorem" => Some("**Keyword `theorem`**\n\nDeclares a named mathematical theorem with a proposition."),
        "proof" => Some("**Keyword `proof`**\n\nBegins a formal deduction proof block discharged by the kernel."),
        "suppose" => Some("**Proof Verb `suppose`**\n\nIntroduces a local premise hypothesis: `suppose <name> : <proposition>`."),
        "derive" => Some("**Proof Verb `derive`**\n\nInfers a new statement from previous hypotheses using an inference rule: `derive <name> : <prop> from <h> using <Rule>`."),
        "therefore" => Some("**Proof Verb `therefore`**\n\nDischarges the target theorem goal: `therefore <prop> from <h>`."),
        "construct" => Some("**Proof Verb `construct`**\n\nIntroduces an auxiliary geometric entity (e.g. `midpoint`, `altitude`, `bisector`)."),
        "end" => Some("**Keyword `end`**\n\nCloses the proof block and triggers final kernel verification."),

        // Geometric Predicates
        "diameter" => Some("**Predicate `diameter(AB)`**\n\nAsserts that the segment $AB$ is a diameter passing through the center of the circle."),
        "on_circle" => Some("**Predicate `on_circle(C)`**\n\nAsserts that point $C$ lies on the boundary circumference of the circle."),
        "intersect" => Some("**Predicate `intersect(line_1, line_2)`**\n\nAsserts that two lines intersect at a common point."),
        "midpoint" => Some("**Construction `midpoint(AB)`**\n\nConstructs point $M$ such that $AM = MB$ on segment $AB$."),
        "altitude" => Some("**Construction `altitude(AH)`**\n\nConstructs an altitude segment perpendicular to the opposing triangle side."),

        _ if word.starts_with('h') && word[1..].chars().all(|c| c.is_ascii_digit()) => {
            return find_hypothesis_definition(doc, word).map(|content| {
                format!(
                    "{{\"contents\": {{\"kind\": \"markdown\", \"value\": {}}}, \"range\": {{\"start\": {{\"line\": {}, \"character\": {}}}, \"end\": {{\"line\": {}, \"character\": {}}}}}}}",
                    json_str(&content),
                    line_idx,
                    start_col,
                    line_idx,
                    end_col
                )
            });
        }

        _ => None,
    };

    hover_text.map(|content| {
        format!(
            "{{\"contents\": {{\"kind\": \"markdown\", \"value\": {}}}, \"range\": {{\"start\": {{\"line\": {}, \"character\": {}}}, \"end\": {{\"line\": {}, \"character\": {}}}}}}}",
            json_str(content),
            line_idx,
            start_col,
            line_idx,
            end_col
        )
    })
}

fn find_hypothesis_definition(doc: &str, hyp: &str) -> Option<String> {
    for (idx, line) in doc.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("suppose ") && trimmed.contains(hyp) {
            let prop = trimmed.replace("suppose ", "");
            return Some(format!("**Hypothesis `{}`** (Assumed)\n\n`{}`\n\n*Declared on line {}*", hyp, prop, idx + 1));
        } else if trimmed.starts_with("derive ") && trimmed.contains(hyp) {
            let prop = trimmed.replace("derive ", "");
            return Some(format!("**Derived Hypothesis `{}`**\n\n`{}`\n\n*Derived on line {}*", hyp, prop, idx + 1));
        } else if trimmed.starts_with("have ") && trimmed.contains(hyp) {
            let prop = trimmed.replace("have ", "");
            return Some(format!("**Hypothesis `{}`**\n\n`{}`\n\n*Introduced on line {}*", hyp, prop, idx + 1));
        }
    }
    None
}

fn find_word_at_pos(line: &str, char_idx: usize) -> Option<(&str, usize, usize)> {
    if line.is_empty() {
        return None;
    }

    let clamped = char_idx.min(line.len().saturating_sub(1));
    let bytes = line.as_bytes();

    let is_word_char = |b: u8| b.is_ascii_alphanumeric() || b == b'_';

    if !is_word_char(bytes[clamped]) {
        // Check if previous char is a word char
        if clamped > 0 && is_word_char(bytes[clamped - 1]) {
            return find_word_around(line, clamped - 1);
        }
        return None;
    }

    find_word_around(line, clamped)
}

fn find_word_around(line: &str, idx: usize) -> Option<(&str, usize, usize)> {
    let bytes = line.as_bytes();
    let is_word_char = |b: u8| b.is_ascii_alphanumeric() || b == b'_';

    let mut start = idx;
    while start > 0 && is_word_char(bytes[start - 1]) {
        start -= 1;
    }

    let mut end = idx;
    while end < bytes.len() && is_word_char(bytes[end]) {
        end += 1;
    }

    if start < end {
        Some((&line[start..end], start, end))
    } else {
        None
    }
}
