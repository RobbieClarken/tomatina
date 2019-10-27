#![allow(dead_code)]
extern crate libusb;

mod manager;
mod button;
mod tracker;

use std::io::Result;

fn main() -> Result<()> {
    let mgr = manager::Manager {};
    mgr.run();
    Ok(())
}
