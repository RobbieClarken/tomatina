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
        let loop_interval = Duration::from_millis(50);
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
            if let Some(t) = tracker.time_remaining(Instant::now()) {
                if let Some(t) = loggable_time_remaining(t, loop_interval) {
                    println!(
                        "Time remaining in state {:?}: {}:{:02}",
                        tracker.state,
                        t.as_secs() / 60,
                        t.as_secs() % 60
                    );
                }
            }
            sleep(loop_interval);
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

fn loggable_time_remaining(time_remaining: Duration, loop_interval: Duration) -> Option<Duration> {
    let half_interval = loop_interval / 2;
    if (time_remaining + half_interval).as_millis() % 60_000 <= loop_interval.as_millis() {
        Some(Duration::from_secs(
            (time_remaining + half_interval).as_secs(),
        ))
    } else {
        None
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn test_loggable_time_remaining_returns_None_if_not_near_a_multiple_of_1_minute() {
        assert_eq!(
            loggable_time_remaining(Duration::from_millis(90_000), Duration::from_millis(50)),
            None
        );
        assert_eq!(
            loggable_time_remaining(Duration::from_millis(60_026), Duration::from_millis(50)),
            None
        );
        assert_eq!(
            loggable_time_remaining(Duration::from_millis(59_974), Duration::from_millis(50)),
            None
        );
    }

    #[test]
    fn test_loggable_time_remaining_returns_Some_if_near_a_multiple_of_1_minute() {
        assert_eq!(
            loggable_time_remaining(Duration::from_millis(120_025), Duration::from_millis(50)),
            Some(Duration::from_millis(120_000)),
        );
        assert_eq!(
            loggable_time_remaining(Duration::from_millis(119_975), Duration::from_millis(50)),
            Some(Duration::from_millis(120_000)),
        );
    }
}
