use std::time;

/// A screen change notification.
#[derive(Clone, Copy, Debug)]
pub struct Change<T> {
    /// The instant this event was generated, relative to some point in time.
    pub timestamp: time::Duration,

    /// The new width.
    pub width: usize,

    /// The new height.
    pub height: usize,

    /// Associated data..
    pub data: T,
}

impl<T> Change<T> {
    /// Determines whether a change event occurred later than another one.
    ///
    /// # Arguments
    /// *  `other` - The other change event.
    pub fn later_than(&self, other: &Self) -> bool {
        self.timestamp > other.timestamp
    }
}
