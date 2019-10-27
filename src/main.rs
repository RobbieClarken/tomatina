#![allow(dead_code)]
extern crate libusb;
use libusb::{Context, DeviceHandle};
use std::time::Duration;

const VENDOR_ID: u16 = 0xD209;
const PRODUCT_ID: u16 = 0x1200;
const USBBTN_INTERFACE: u16 = 0;
const USBBTN_VALUE: u16 = 0x0200;
// const USBBTN_MESG_LENGTH = 4;
// const USBBTN_SIZE = 64;
const UM_REQUEST_TYPE: u8 = 0x21;
const UM_REQUEST: u8 = 9;
const UM_TIMEOUT: Duration = Duration::from_millis(2000);

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
    context.open_device_with_vid_pid(VENDOR_ID, PRODUCT_ID)
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
                UM_TIMEOUT,
            )
            .expect("transfer failed");
    }
}

fn main() {
    let mut context = libusb::Context::new().unwrap();
    context.set_log_level(libusb::LogLevel::Debug);
    let handle = open_device(&context).expect("failed to find USBButton");
    set_button_data(
        &handle,
        &Color {
            red: 255,
            green: 0,
            blue: 255,
        },
        &Color {
            red: 255,
            green: 0,
            blue: 255,
        },
    );
}
