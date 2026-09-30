use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct FileId(pub u32);

impl fmt::Display for FileId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "file#{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Span {
    pub file: FileId,
    pub start: u32,
    pub end: u32,
}

impl Span {
    pub const DUMMY: Span = Span {
        file: FileId(0),
        start: 0,
        end: 0,
    };

    pub fn new(file: FileId, start: u32, end: u32) -> Self {
        assert!(start <= end, "Span start {} cannot be greater than end {}", start, end);
        Self { file, start, end }
    }

    pub fn point(file: FileId, pos: u32) -> Self {
        Self {
            file,
            start: pos,
            end: pos,
        }
    }

    pub fn len(&self) -> u32 {
        self.end.saturating_sub(self.start)
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    pub fn merge(&self, other: Span) -> Span {
        assert_eq!(self.file, other.file, "Cannot merge spans from different files");
        Span {
            file: self.file,
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }

    pub fn contains_pos(&self, pos: u32) -> bool {
        pos >= self.start && pos <= self.end
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}-{}", self.file, self.start, self.end)
    }
}

pub trait Spanned {
    fn span(&self) -> Span;
}

impl Spanned for Span {
    fn span(&self) -> Span {
        *self
    }
}
