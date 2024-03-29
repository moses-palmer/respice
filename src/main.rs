extern crate xcb;

mod monitor;

fn main() {
    let (conn, screen_num) = xcb::Connection::connect(None).unwrap();
    let screen_num = screen_num as usize;

    for change in monitor::Changes::listen(&conn, screen_num).unwrap() {
        let window = change.data;
        let output_modes = match monitor::output_modes(&conn, window) {
            Ok(modes) => modes,
            Err(e) => {
                eprintln!("Failed to list modes: {:?}", e);
                continue;
            }
        };
        let preferred_modes = monitor::preferred_modes(&conn, &output_modes);
        for (output, modes) in output_modes {
            println!("Modes for output {}:", output);
            for mode in modes.iter() {
                let is_preferred = preferred_modes
                    .get(&output)
                    .map(|preferred_mode| mode == preferred_mode)
                    .unwrap_or(false);
                println!(
                    " {} {}",
                    if is_preferred { "•" } else { "◦" },
                    mode
                );
            }
        }
        let bounds = preferred_modes
            .iter()
            .filter_map(|(&output, &mode)| {
                monitor::bounds(&conn, output, mode).ok()
            })
            .flat_map(monitor::Bounds::corners)
            .collect::<monitor::Bounds>();
        match monitor::assert_screen_size(&conn, window, bounds) {
            Ok(_) => {
                println!("Resized screen to {}", bounds);
                for (output, mode) in preferred_modes {
                    match monitor::apply_mode(&conn, output, mode) {
                        Err(e) => {
                            eprintln!("Failed to apply mode {}: {:?}", mode, e)
                        }
                        Ok(_) => {
                            println!("Applied mode {}", mode);
                            if let Err(e) = monitor::assert_screen_size(
                                &conn, window, bounds,
                            ) {
                                eprintln!(
                                    "Failed to resize screen to {}: {:?}",
                                    bounds, e,
                                );
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to assert screen size {}: {:?}", bounds, e)
            }
        }
    }
}
