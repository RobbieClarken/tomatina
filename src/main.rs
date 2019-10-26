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

fn open_device(context: &Context) -> Option<DeviceHandle> {
    context.open_device_with_vid_pid(VENDOR_ID, PRODUCT_ID)
}

fn set_color(handle: &DeviceHandle, red: u8, green: u8, blue: u8) {
    handle
        .write_control(
            UM_REQUEST_TYPE,
            UM_REQUEST,
            USBBTN_VALUE,
            USBBTN_INTERFACE,
            &[1, red, green, blue],
            UM_TIMEOUT,
        )
        .expect("transfer failed");
}

fn main() {
    let mut context = libusb::Context::new().unwrap();
    context.set_log_level(libusb::LogLevel::Debug);
    let handle = open_device(&context).expect("failed to find USBButton");
    set_color(&handle, 255, 0, 255);
}
