/// An error occurring when starting to listen for screen changes.
#[derive(Debug)]
pub enum Error {
    /// The screen does not exist.
    InvalidScreen,

    /// The _XRANDR_ extension is not enabled.
    UnknownExtension,

    /// The output is is unknown.
    UnknownOutput(u32),

    /// The requested screen size is unsupported.
    UnsupportedScreenSize(u32, u32),

    /// A function returned an _XCB_ error.
    XCBError(u8),
}
