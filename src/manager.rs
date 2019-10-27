use crate::button::{Button, Color};
use crate::tracker::{State, Tracker};

use libc::mkfifo;
use std::collections::HashMap;
use std::ffi::CString;
use std::fs;
use std::io::Read;
use std::io::Result;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::thread::sleep;
use std::time::{Duration, Instant};

const RED: Color = Color(255, 0, 0);
const GREEN: Color = Color(0, 255, 0);
const BLUE: Color = Color(0, 0, 255);
const PURPLE: Color = Color(255, 0, 255);

pub struct Manager {}

impl Manager {
    pub fn run(&self) {
        let colors: HashMap<State, Color> = [
            (State::PendingWork, RED),
            (State::Working, GREEN),
            (State::PendingShortBreak, RED),
            (State::ShortBreak, BLUE),
            (State::PendingLongBreak, RED),
            (State::LongBreak, PURPLE),
        ]
        .iter()
        .cloned()
        .collect();

        let context = libusb::Context::new().unwrap();
        let button = Button::connect(&context);
        let mut tracker = Tracker::new();
        let mut signal = ButtonSignal::create().expect("failed to create pipe to button");
        button.configure(colors.get(&tracker.state).unwrap());
        loop {
            let init_state = tracker.state;
            match signal.poll() {
                Some(ButtonPress::Primary) => {
                    println!("Detected button press");
                    tracker.next();
                }
                _ => {}
            }
            tracker.tick(Instant::now());
            if tracker.state != init_state {
                println!("State changed from {:?} to {:?}", init_state, tracker.state);
                button.set_color(colors.get(&tracker.state).unwrap());
            }
            sleep(Duration::from_millis(50));
        }
    }
}

enum ButtonPress {
    Primary,
    Secondary,
}

struct ButtonSignal {
    file: fs::File,
}

impl ButtonSignal {
    fn create() -> Result<Self> {
        let path = Path::new("/tmp/tomatina.fifo");
        if path.exists() {
            fs::remove_file(path)?;
        }
        let path_str = CString::new(path.to_str().unwrap())?;
        let result: i32 = unsafe { mkfifo(path_str.as_ptr(), 0o600) }.into();
        assert_eq!(result, 0);
        let file = fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(path)?;
        Ok(Self { file })
    }

    fn poll(&mut self) -> Option<ButtonPress> {
        let mut buf: [u8; 1] = [0; 1];
        let read = self.file.read(&mut buf).expect("failed to read from pipe");
        if read == 0 {
            return None;
        }
        match buf[0] as char {
            '1' => Some(ButtonPress::Primary),
            '2' => Some(ButtonPress::Secondary),
            c => {
                eprintln!("unexpected button code: {}", c);
                None
            }
        }
    }
}
