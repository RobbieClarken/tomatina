#![allow(dead_code)]
extern crate libusb;
use libc::mkfifo;
use libusb::{Context, DeviceHandle};
use std::ffi::CString;
use std::fs;
use std::io::Read;
use std::io::Result;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

mod tracker;

const USBBTN_VENDOR_ID: u16 = 0xD209;
const USBBTN_PRODUCT_ID: u16 = 0x1200;
const USBBTN_INTERFACE: u16 = 0;
const USBBTN_VALUE: u16 = 0x0200;
const UM_REQUEST_TYPE: u8 = 0x21;
const UM_REQUEST: u8 = 9;
const TIMEOUT: Duration = Duration::from_millis(2000);

const WORKING: Color = Color {
    red: 0,
    green: 0,
    blue: 255,
};
const BREAK: Color = Color {
    red: 0,
    green: 255,
    blue: 0,
};
const CHANGE: Color = Color {
    red: 255,
    green: 0,
    blue: 0,
};

enum ButtonAction {
    Alternate = 0,
    Extended = 1,
    Both = 2,
}

enum Key {
    T = 0x17,
    U = 0x18,
    CtrlLeft = 0x70,
    AltLeft = 0x72,
    CmdLeft = 0x73,
}

struct Color {
    red: u8,
    green: u8,
    blue: u8,
}

fn open_device(context: &Context) -> Option<DeviceHandle> {
    context.open_device_with_vid_pid(USBBTN_VENDOR_ID, USBBTN_PRODUCT_ID)
}

fn set_button_data(handle: &DeviceHandle, released_color: &Color, pressed_color: &Color) {
    let mut buf: [u8; 64] = [0; 64];

    // header
    buf[0] = 0x50;
    buf[1] = 0xdd;
    buf[2] = ButtonAction::Both as u8;

    buf[4] = released_color.red;
    buf[5] = released_color.green;
    buf[6] = released_color.blue;

    buf[7] = pressed_color.red;
    buf[8] = pressed_color.green;
    buf[9] = pressed_color.blue;

    // Primary key sequence
    buf[10] = Key::CtrlLeft as u8;
    buf[11] = Key::AltLeft as u8;
    buf[12] = Key::CmdLeft as u8;
    buf[13] = Key::T as u8;

    // Secondary key sequence
    buf[34] = Key::CtrlLeft as u8;
    buf[35] = Key::AltLeft as u8;
    buf[36] = Key::CmdLeft as u8;
    buf[37] = Key::U as u8;

    for offset in (0..buf.len()).step_by(4) {
        handle
            .write_control(
                UM_REQUEST_TYPE,
                UM_REQUEST,
                USBBTN_VALUE,
                USBBTN_INTERFACE,
                &buf[offset..offset + 4],
                TIMEOUT,
            )
            .expect("transfer failed");
    }
}

fn main() -> Result<()> {
    let mut context = libusb::Context::new().unwrap();
    context.set_log_level(libusb::LogLevel::Debug);
    let handle = open_device(&context).expect("failed to find USBButton");
    set_button_data(
        &handle,
        &WORKING,
        &Color {
            red: 255,
            green: 255,
            blue: 255,
        },
    );
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
