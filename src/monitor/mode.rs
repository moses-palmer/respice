use std::fmt;

/// A description of a mode.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mode {
    /// The output width.
    pub width: usize,

    /// The output height.
    pub height: usize,

    /// The mode identifier.
    pub id: u32,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}x{} (0x{:x?})", self.width, self.height, self.id)
    }
}
