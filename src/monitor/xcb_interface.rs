use std::time;

use xcb::{randr, render, xproto};

use super::change::Change;
use super::err::Error;
use super::mode::Mode;
use super::transform::Transform;

/// A constant used to transform `u16` fixed decimal values to floating point
/// values.
const FIXED_MULTIPLIER: f32 = 1.0 / (1 << 16) as f32;

impl<'a> From<&'a randr::ScreenChangeNotifyEvent> for Change<xproto::Window> {
    /// Constructs a screen change struct from an _XRANDR_ screen change
    /// notification.
    ///
    /// # Arguments
    /// *  `source` - The source notification.
    fn from(source: &'a randr::ScreenChangeNotifyEvent) -> Self {
        let timestamp =
            time::Duration::from_millis(source.config_timestamp().into());
        let width = source.width() as usize;
        let height = source.height() as usize;
        let data = source.root();
        Change {
            timestamp,
            width,
            height,
            data,
        }
    }
}

impl From<xcb::GenericError> for Error {
    /// Constructs an error from an _XCB generic error_.
    ///
    /// This will create an error with an error code from the source.
    ///
    /// # Arguments
    /// *  `source` - The source error.
    fn from(source: xcb::GenericError) -> Self {
        Error::XCBError(source.error_code())
    }
}

impl From<randr::ModeInfo> for Mode {
    /// Constructs a mode struct from an _XRANDR_ mode information object.
    ///
    /// # Arguments
    /// *  `source` - The source mode information.
    fn from(source: randr::ModeInfo) -> Self {
        let width = source.width() as usize;
        let height = source.height() as usize;
        let id = source.id();
        Mode { width, height, id }
    }
}

impl From<render::Transform> for Transform {
    /// Constructs a transform from an _XCB transform_.
    ///
    /// # Arguments
    /// *  `source` - The source transform.
    fn from(source: render::Transform) -> Self {
        Transform {
            matrix: [
                [
                    float(source.matrix11()),
                    float(source.matrix12()),
                    float(source.matrix13()),
                ],
                [
                    float(source.matrix21()),
                    float(source.matrix22()),
                    float(source.matrix23()),
                ],
                [
                    float(source.matrix31()),
                    float(source.matrix32()),
                    float(source.matrix33()),
                ],
            ],
        }
    }
}

/// Converts a floating point value from a _fixed X value_.
///
/// # Arguments
/// *  `v` - The source fixed value.
fn float(v: render::Fixed) -> f32 {
    v as f32 * FIXED_MULTIPLIER
}
