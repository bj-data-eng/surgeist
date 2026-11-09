//! Geometry-independent participation shared by final readback and windows.
#[derive(Clone, Copy)]
pub(crate) enum Content {
    Text {
        retained: bool,
        hard_separator: bool,
    },
    Cell,
    Atomic {
        in_flow: bool,
    },
    Boundary {
        participates: bool,
    },
}

impl Content {
    pub(crate) const fn in_flow(self) -> bool {
        match self {
            Self::Text {
                retained,
                hard_separator,
            } => retained && !hard_separator,
            Self::Cell => true,
            Self::Atomic { in_flow } => in_flow,
            Self::Boundary { participates } => participates,
        }
    }
}
