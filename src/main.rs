mod button;
mod manager;
mod tracker;

extern crate libusb;

use structopt::StructOpt;
use std::time::Duration;

#[derive(StructOpt, Debug)]
#[structopt(about = "Use a USBButton as a pomodoro timer")]
struct Opt {
    #[structopt(long, default_value = "25")]
    work: u64,

    #[structopt(long, default_value = "5")]
    short_break: u64,

    #[structopt(long, default_value = "20")]
    long_break: u64,
}

fn main() {
    let opt = Opt::from_args();
    let config = tracker::TrackerConfig {
        work_duration: Duration::from_secs(60 * opt.work),
        short_break_duration: Duration::from_secs(60 * opt.short_break),
        long_break_duration: Duration::from_secs(60 * opt.long_break),
    };
    manager::run(config);
}
