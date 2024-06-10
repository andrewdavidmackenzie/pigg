#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    BoardLayout,
    BCMLayout,
}

impl Layout {
    pub const ALL: [Layout; 2] = [Layout::BoardLayout, Layout::BCMLayout];
}

// Implementing format for Layout
// TODO could maybe put the Name as a &str inside the enum elements above?
impl std::fmt::Display for Layout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Layout::BoardLayout => "Board Pin Layout",
                Layout::BCMLayout => "BCM Pin Layout",
            }
        )
    }
}
