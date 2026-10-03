use crate::api::json_str;

/// Provides Go to Definition (F12) for hypothesis references.
pub fn get_definition(doc: &str, line_idx: usize, char_idx: usize, uri: &str) -> Option<String> {
    let lines: Vec<&str> = doc.lines().collect();
    if line_idx >= lines.len() {
        return None;
    }

    let line = lines[line_idx];
    let (word, _, _) = find_word_at_pos(line, char_idx)?;

    // If word is a hypothesis identifier (e.g. h1, h2)
    if word.starts_with('h') && word[1..].chars().all(|c| c.is_ascii_digit()) {
        for (def_line_idx, l) in lines.iter().enumerate() {
            let trimmed = l.trim();
            if (trimmed.starts_with("suppose ") || trimmed.starts_with("derive ") || trimmed.starts_with("have "))
                && l.contains(word)
            {
                if let Some(col) = l.find(word) {
                    return Some(format!(
                        "{{\"uri\": {}, \"range\": {{\"start\": {{\"line\": {}, \"character\": {}}}, \"end\": {{\"line\": {}, \"character\": {}}}}}}}",
                        json_str(uri),
                        def_line_idx,
                        col,
                        def_line_idx,
                        col + word.len()
                    ));
                }
            }
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
