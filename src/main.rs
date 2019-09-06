extern crate xcb;

mod monitor;

fn main() {
    let (conn, screen_num) = xcb::Connection::connect(None).unwrap();
    let screen_num = screen_num as usize;

    for change in monitor::Changes::listen(&conn, screen_num).unwrap() {
        // TODO: Implement
    }
}
