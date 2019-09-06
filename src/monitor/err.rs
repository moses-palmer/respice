/// An error occurring when starting to listen for screen changes.
#[derive(Debug)]
pub enum Error {
    /// A function returned an _XCB_ error.
    XCBError(u8),
}
