use std::fmt;

macro_rules! define_id {
    ($name:ident, $prefix:literal) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(pub u32);

        impl $name {
            pub const fn from_raw(val: u32) -> Self {
                Self(val)
            }

            pub const fn raw(self) -> u32 {
                self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}{}", $prefix, self.0)
            }
        }
    };
}

define_id!(SymbolId, "sym#");
define_id!(TheoremId, "thm#");
define_id!(FactId, "fact#");
define_id!(ProofId, "pf#");
define_id!(PointId, "pt#");
define_id!(SegmentId, "seg#");
define_id!(LineId, "line#");
define_id!(CircleId, "circ#");
define_id!(TriangleId, "tri#");
define_id!(ConstructionId, "cst#");
define_id!(Revision, "rev#");

#[derive(Debug, Default)]
pub struct IdGen {
    next: u32,
}

impl IdGen {
    pub const fn new() -> Self {
        Self { next: 1 }
    }

    pub fn next_u32(&mut self) -> u32 {
        let id = self.next;
        self.next += 1;
        id
    }

    pub fn next_symbol(&mut self) -> SymbolId {
        SymbolId(self.next_u32())
    }

    pub fn next_theorem(&mut self) -> TheoremId {
        TheoremId(self.next_u32())
    }

    pub fn next_fact(&mut self) -> FactId {
        FactId(self.next_u32())
    }

    pub fn next_proof(&mut self) -> ProofId {
        ProofId(self.next_u32())
    }

    pub fn next_point(&mut self) -> PointId {
        PointId(self.next_u32())
    }
}
