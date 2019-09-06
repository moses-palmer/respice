use std::collections::hash_map;

use xcb::{randr, xproto};

mod change;
pub use self::change::*;
mod err;
pub use self::err::*;
mod mode;
pub use self::mode::*;
mod transform;
pub use self::transform::*;

pub mod xcb_interface;
pub use self::xcb_interface::*;

/// The name of the XRANDR extension.
const EXTENSION: &str = &"RANDR";

/// An iterator over screen change notifications.
pub struct Changes<'a> {
    /// The _XCB_ connection.
    conn: &'a xcb::Connection,

    /// The event number for which to listen.
    event: u8,

    /// The previous emitted notification.
    previous: Option<Change<xproto::Window>>,
}

impl<'a> Changes<'a> {
    /// Creates a new change notification iterator.
    pub fn new(conn: &'a xcb::Connection, event: u8) -> Self {
        Self {
            conn,
            event,
            previous: None,
        }
    }

    /// Creates an iterator for screen change events.
    ///
    /// # Arguments
    /// *  `conn` - The _XCB_ connection.
    /// *  `screen_num` - The screen whose change events to list.
    pub fn listen(
        conn: &'a xcb::Connection,
        screen_num: usize,
    ) -> Result<impl Iterator<Item = Change<xproto::Window>> + 'a, Error> {
        // Get the root window
        let root = conn
            .get_setup()
            .roots()
            .nth(screen_num)
            .ok_or(Error::InvalidScreen)?
            .root();

        // Enable notifications about screen change
        randr::select_input(
            &conn,
            root,
            (randr::NOTIFY_MASK_SCREEN_CHANGE
                | randr::NOTIFY_MASK_OUTPUT_CHANGE) as u16,
        );

        // Get information about the XRANDR extension in order to determine the
        // actual event to which to listen to
        xcb::query_extension(conn, EXTENSION)
            .get_reply()
            .map(|reply| {
                Changes::new(
                    conn,
                    reply.first_event() + randr::SCREEN_CHANGE_NOTIFY,
                )
            })
            .map_err(|_| Error::UnknownExtension)
    }
}

impl<'a> Iterator for Changes<'a> {
    type Item = Change<xproto::Window>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(event) = self.conn.wait_for_event() {
            if event.response_type() == self.event {
                // Convert the event to our internal representation
                let event: Change<_> = unsafe {
                    xcb::cast_event::<randr::ScreenChangeNotifyEvent>(&event)
                }
                .into();

                // If the event is an update, yield it
                if self.previous.map_or_else(|| true, |e| event.later_than(&e))
                {
                    self.previous = Some(event);
                    return Some(event);
                }
            }
        }

        None
    }
}

/// Lists all modes for all outputs.
///
/// # Arguments
/// *  `conn` - The _XCB_ connection.
/// *  `window` - The root window.
pub fn output_modes(
    conn: &xcb::Connection,
    window: xproto::Window,
) -> Result<hash_map::HashMap<randr::Output, Vec<Mode>>, Error> {
    let resources = randr::get_screen_resources(conn, window).get_reply()?;
    let outputs = resources
        .outputs()
        .iter()
        .map(|&output| {
            randr::get_output_info(conn, output, 0)
                .get_reply()
                .map(|output_info| (output, output_info))
        })
        .collect::<Result<Vec<_>, _>>()?;

    outputs
        .iter()
        .map(|(output, output_info)| {
            Ok((
                *output,
                output_info
                    .modes()
                    .iter()
                    .filter_map(|&mode_id| {
                        resources
                            .modes()
                            .find(|mode_info| mode_info.id() == mode_id)
                    })
                    .map(Mode::from)
                    .collect::<Vec<_>>(),
            ))
        })
        .collect()
}

/// Finds the preferred mode for a collection of outputs.
///
/// # Arguments
/// *  `conn` - The _XCB_ connection.
/// *  `output_modes` - A mapping from outputs to all associated modes.
pub fn preferred_modes(
    conn: &xcb::Connection,
    output_modes: &hash_map::HashMap<randr::Output, Vec<Mode>>,
) -> hash_map::HashMap<randr::Output, Mode> {
    output_modes
        .iter()
        .enumerate()
        .filter_map(|(i, (output, modes))| {
            randr::get_output_info(conn, *output, 0)
                .get_reply()
                .ok()
                .map(|output_info| (i, output, output_info, modes))
        })
        .filter_map(|(_, output, output_info, modes)| {
            let n_preferred = output_info.num_preferred();
            if n_preferred > 0 {
                modes.iter().next().map(|mode| (*output, mode.clone()))
            } else {
                None
            }
        })
        .collect()
}

/// Calculates the bounds for an output.
///
/// # Arguments
/// *  `conn` - The _XCB_ connection.
/// *  `output` - The output to modify.
/// *  `mode` - The mode to set.
pub fn bounds(
    conn: &xcb::Connection,
    output: randr::Output,
    mode: Mode,
) -> Result<Bounds, Error> {
    let output_info = randr::get_output_info(conn, output, 0)
        .get_reply()
        .map_err(|_| Error::UnknownOutput(output))?;
    let crtc_info =
        randr::get_crtc_info(conn, output_info.crtc(), 0).get_reply()?;
    let transform: Transform =
        randr::get_crtc_transform(conn, output_info.crtc())
            .get_reply()?
            .current_transform()
            .into();
    let (x, y) = (f32::from(crtc_info.x()), f32::from(crtc_info.y()));
    let (width, height) = match u32::from(crtc_info.rotation() & 0x0F) {
        randr::ROTATION_ROTATE_0 | randr::ROTATION_ROTATE_180 => {
            (mode.width as f32, mode.height as f32)
        }
        randr::ROTATION_ROTATE_90 | randr::ROTATION_ROTATE_270 => {
            (mode.height as f32, mode.width as f32)
        }
        _ => (0.0, 0.0),
    };

    Ok([
        Point { x, y },
        Point { x: x + width, y },
        Point { x, y: y + height },
        Point {
            x: x + width,
            y: y + height,
        },
    ]
    .iter()
    .map(|&point| transform.apply(point))
    .collect())
}

/// Asserts that a bounded rectangle fits on screen.
///
/// # Arguments
/// *  `conn` - The _XCB_ connection.
/// *  `output` - The output to modify.
/// *  `bounds` - The rectangle.
pub fn assert_screen_size(
    conn: &xcb::Connection,
    window: xproto::Window,
    bounds: Bounds,
) -> Result<(), Error> {
    // Verify that the requested size is valid
    let screen_size_range =
        randr::get_screen_size_range(conn, window).get_reply()?;
    if (bounds.width as u16) > screen_size_range.max_width()
        || (bounds.width as u16) < screen_size_range.min_width()
        || (bounds.height as u16) > screen_size_range.max_height()
        || (bounds.height as u16) < screen_size_range.min_height()
    {
        return Err(Error::UnsupportedScreenSize(bounds.width, bounds.height));
    }

    randr::set_screen_size(
        conn,
        window,
        bounds.width as u16,
        bounds.height as u16,
        bounds.width as u32,
        bounds.height as u32,
    )
    .request_check()
    .map(|_| ())
    .map_err(Error::from)
}

/// Attempts to apply a mode to an output.
///
/// # Arguments
/// *  `conn` - The _XCB_ connection.
/// *  `output` - The output to modify.
/// *  `mode` - The mode to set.
pub fn apply_mode(
    conn: &xcb::Connection,
    output: randr::Output,
    mode: Mode,
) -> Result<(), Error> {
    let output_info = randr::get_output_info(conn, output, 0)
        .get_reply()
        .map_err(|_| Error::UnknownOutput(output))?;
    let crtc_info =
        randr::get_crtc_info(conn, output_info.crtc(), 0).get_reply()?;
    Ok(randr::set_crtc_config(
        conn,
        output_info.crtc(),
        0,
        0,
        crtc_info.x(),
        crtc_info.y(),
        mode.id,
        crtc_info.rotation(),
        crtc_info.outputs(),
    )
    .get_reply()
    .map(|_| ())?)
}
