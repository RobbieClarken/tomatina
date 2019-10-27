#![allow(dead_code)]
extern crate libusb;

mod button;
mod tracker;

use button::Button;
use libc::mkfifo;
use std::ffi::CString;
use std::fs;
use std::io::Read;
use std::io::Result;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

fn main() -> Result<()> {
    let mut context = libusb::Context::new().unwrap();
    context.set_log_level(libusb::LogLevel::Debug);

    let button = Button::connect(&context);
    button.configure();

    let p = Path::new("/tmp/tomatina.fifo");
    create_named_pipe(p)?;
    let mut file = fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NONBLOCK)
        .open(p)?;
    let mut buf: [u8; 1] = [0; 1];
    loop {
        let read = file.read(&mut buf)?;
        if read > 0 {
            if buf[0] as char == '1' {
                println!("transitioning...");
            }
        }
        sleep(Duration::from_millis(50));
    }
}

fn create_named_pipe(p: &Path) -> Result<()> {
    if p.exists() {
        fs::remove_file(p)?;
    }
    let path = CString::new(p.to_str().unwrap())?;
    let result: i32 = unsafe { mkfifo(path.as_ptr(), 0o600) }.into();
    assert_eq!(result, 0);
    Ok(())
}
