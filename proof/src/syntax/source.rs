use crate::syntax::span::{FileId, Span};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SourceFile {
    pub id: FileId,
    pub name: String,
    pub src: String,
    line_starts: Vec<u32>,
}

impl SourceFile {
    pub fn new(id: FileId, name: String, src: String) -> Self {
        let mut line_starts = vec![0];
        for (idx, b) in src.bytes().enumerate() {
            if b == b'\n' {
                line_starts.push((idx + 1) as u32);
            }
        }
        Self {
            id,
            name,
            src,
            line_starts,
        }
    }

    pub fn len(&self) -> usize {
        self.src.len()
    }

    pub fn is_empty(&self) -> bool {
        self.src.is_empty()
    }

    pub fn text(&self) -> &str {
        &self.src
    }

    pub fn slice(&self, span: Span) -> &str {
        if span.file != self.id {
            return "";
        }
        let start = span.start as usize;
        let end = span.end as usize;
        if start <= end && end <= self.src.len() {
            &self.src[start..end]
        } else {
            ""
        }
    }

    /// 1-based (line, column) where column is character/code-point count on that line.
    pub fn line_col(&self, byte_offset: u32) -> (usize, usize) {
        let offset = (byte_offset as usize).min(self.src.len());
        // Find line index via binary search
        let line_idx = match self.line_starts.binary_search(&(offset as u32)) {
            Ok(idx) => idx,
            Err(idx) => idx.saturating_sub(1),
        };
        let line_start_byte = self.line_starts[line_idx] as usize;
        let line_slice = &self.src[line_start_byte..offset];
        let col = line_slice.chars().count() + 1;
        (line_idx + 1, col)
    }

    pub fn line_text(&self, line_number_1_based: usize) -> Option<&str> {
        if line_number_1_based == 0 || line_number_1_based > self.line_starts.len() {
            return None;
        }
        let start = self.line_starts[line_number_1_based - 1] as usize;
        let end = if line_number_1_based < self.line_starts.len() {
            // strip possible trailing newline
            let next_start = self.line_starts[line_number_1_based] as usize;
            if next_start > 0 && self.src.as_bytes().get(next_start - 1) == Some(&b'\n') {
                if next_start > 1 && self.src.as_bytes().get(next_start - 2) == Some(&b'\r') {
                    next_start - 2
                } else {
                    next_start - 1
                }
            } else {
                next_start
            }
        } else {
            self.src.len()
        };
        Some(&self.src[start..end])
    }
}

#[derive(Debug, Default)]
pub struct SourceMap {
    files: Vec<SourceFile>,
    by_name: HashMap<String, FileId>,
}

impl SourceMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_file(&mut self, name: impl Into<String>, content: impl Into<String>) -> FileId {
        let name = name.into();
        let id = FileId(self.files.len() as u32);
        let file = SourceFile::new(id, name.clone(), content.into());
        self.by_name.insert(name, id);
        self.files.push(file);
        id
    }

    pub fn get(&self, id: FileId) -> Option<&SourceFile> {
        self.files.get(id.0 as usize)
    }

    pub fn get_by_name(&self, name: &str) -> Option<&SourceFile> {
        self.by_name.get(name).and_then(|id| self.get(*id))
    }
}
