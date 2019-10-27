mod button;
mod manager;
mod tracker;

extern crate libusb;

fn main() {
    let mgr = manager::Manager {};
    mgr.run();
}
