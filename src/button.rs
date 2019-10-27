use libusb::{Context, DeviceHandle};
use std::time::Duration;

const USBBTN_VENDOR_ID: u16 = 0xD209;
const USBBTN_PRODUCT_ID: u16 = 0x1200;
const USBBTN_INTERFACE: u16 = 0;
const USBBTN_VALUE: u16 = 0x0200;
const UM_REQUEST_TYPE: u8 = 0x21;
const UM_REQUEST: u8 = 9;
const TIMEOUT: Duration = Duration::from_millis(2000);

#[allow(dead_code)]
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

#[derive(Clone)]
pub struct Color(pub u8, pub u8, pub u8);

pub struct Button<'a> {
    device: DeviceHandle<'a>,
}

impl<'a> Button<'a> {
    pub fn connect(context: &'a Context) -> Self {
        let device = context
            .open_device_with_vid_pid(USBBTN_VENDOR_ID, USBBTN_PRODUCT_ID)
            .expect("failed to find USBButton");
        Self { device }
    }

    pub fn configure(&self, color: &Color) {
        self.set_button_data(color, &Color(255, 255, 255));
    }

    fn set_button_data(&self, released_color: &Color, pressed_color: &Color) {
        let mut buf: [u8; 64] = [0; 64];

        // header
        buf[0] = 0x50;
        buf[1] = 0xdd;
        buf[2] = ButtonAction::Both as u8;

        buf[4] = released_color.0;
        buf[5] = released_color.1;
        buf[6] = released_color.2;

        buf[7] = pressed_color.0;
        buf[8] = pressed_color.1;
        buf[9] = pressed_color.2;

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
            self.device
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

    pub fn set_color(&self, color: &Color) {
        self.device
            .write_control(
                UM_REQUEST_TYPE,
                UM_REQUEST,
                USBBTN_VALUE,
                USBBTN_INTERFACE,
                &[1, color.0 as u8, color.1 as u8, color.2 as u8],
                TIMEOUT,
            )
            .expect("transfer failed");
    }
}
