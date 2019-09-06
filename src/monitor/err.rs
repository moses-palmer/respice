/// An error occurring when starting to listen for screen changes.
#[derive(Debug)]
pub enum Error {
    /// The screen does not exist.
    InvalidScreen,

    /// The _XRANDR_ extension is not enabled.
    UnknownExtension,

    /// A function returned an _XCB_ error.
    XCBError(u8),
}
