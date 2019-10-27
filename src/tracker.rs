use std::time::{Duration, Instant};

const WORK_DURATION: Duration = Duration::from_secs(20 * 60);
const SHORT_BREAK_DURATION: Duration = Duration::from_secs(5 * 60);
const LONG_BREAK_DURATION: Duration = Duration::from_secs(15 * 60);

#[derive(Debug, PartialEq)]
enum State {
    PendingWork,
    Working,
    PendingShortBreak,
    ShortBreak,
    PendingLongBreak,
    LongBreak,
}

pub struct Tracker {
    state: State,
    entered_state: Instant,
    intervals: u64,
}

impl Tracker {
    pub fn new() -> Self {
        return Self {
            state: State::PendingWork,
            entered_state: Instant::now(),
            intervals: 0,
        };
    }

    pub fn next(&mut self) {
        match self.state {
            State::PendingWork => {
                self.enter_state(State::Working);
            }
            State::PendingShortBreak => {
                self.enter_state(State::ShortBreak);
            }
            State::PendingLongBreak => {
                self.enter_state(State::LongBreak);
            }
            State::Working | State::ShortBreak | State::LongBreak => {}
        }
    }

    pub fn tick(&mut self, now: Instant) {
        match self.state {
            State::PendingWork | State::PendingShortBreak | State::PendingLongBreak => {}
            State::Working => {
                if now.duration_since(self.entered_state) >= WORK_DURATION {
                    self.intervals += 1;
                    if self.intervals % 4 != 0 {
                        self.enter_state(State::PendingShortBreak);
                    } else {
                        self.enter_state(State::PendingLongBreak);
                    }
                }
            }
            State::ShortBreak => {
                if now.duration_since(self.entered_state) >= SHORT_BREAK_DURATION {
                    self.enter_state(State::PendingWork);
                }
            }
            State::LongBreak => {
                if now.duration_since(self.entered_state) >= LONG_BREAK_DURATION {
                    self.enter_state(State::PendingWork);
                }
            }
        }
    }

    fn enter_state(&mut self, state: State) {
        self.state = state;
        self.entered_state = Instant::now();
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    fn tracker_at_short_break() -> Tracker {
        let mut tracker = Tracker::new();
        tracker.next(); // -> working
        tracker.tick(Instant::now() + WORK_DURATION); // -> pending short break
        tracker.next(); // -> short break
        tracker
    }

    fn tracker_at_pending_long_break() -> Tracker {
        let mut tracker = Tracker::new();
        work(&mut tracker);
        short_break(&mut tracker);
        work(&mut tracker);
        short_break(&mut tracker);
        work(&mut tracker);
        short_break(&mut tracker);
        work(&mut tracker);
        tracker
    }

    fn work(tracker: &mut Tracker) {
        // tracker must start at PendingWork
        tracker.next(); // -> working
        tracker.tick(Instant::now() + WORK_DURATION); // -> pending short/long break
    }

    fn short_break(tracker: &mut Tracker) {
        // tracker must start at PendingShortBreak
        tracker.next(); // -> short break
        tracker.tick(Instant::now() + SHORT_BREAK_DURATION); // -> pending work
    }

    #[test]
    fn initial_state_is_PendingWork() {
        let tracker = Tracker::new();
        assert_eq!(tracker.state, State::PendingWork);
    }

    #[test]
    fn calling_next_transitions_from_PendingWork_to_Working() {
        let mut tracker = Tracker::new();
        tracker.next();
        assert_eq!(tracker.state, State::Working);
    }

    #[test]
    fn calling_next_transitions_from_Working_doesnt_transition() {
        let mut tracker = Tracker::new();
        tracker.next();
        tracker.next();
        assert_eq!(tracker.state, State::Working);
    }

    #[test]
    fn calling_tick_within_20_mins_of_working_doesnt_transition() {
        let mut tracker = Tracker::new();
        tracker.next();
        tracker.tick(Instant::now() + Duration::from_secs(19 * 60));
        assert_eq!(tracker.state, State::Working);
    }

    #[test]
    fn calling_tick_after_20_mins_of_working_transitions_to_PendingShortBreak() {
        let mut tracker = Tracker::new();
        tracker.next();
        tracker.tick(Instant::now() + WORK_DURATION);
        assert_eq!(tracker.state, State::PendingShortBreak);
    }

    #[test]
    fn calling_tick_while_in_PendingWork_does_nothing() {
        let mut tracker = Tracker::new();
        assert_eq!(tracker.state, State::PendingWork);
        tracker.tick(Instant::now() + WORK_DURATION);
        assert_eq!(tracker.state, State::PendingWork);
    }

    #[test]
    fn calling_next_from_PendingShortBreak_transitions_to_short_break() {
        let mut tracker = Tracker::new();
        tracker.next();
        tracker.tick(Instant::now() + WORK_DURATION);
        assert_eq!(tracker.state, State::PendingShortBreak);
        tracker.next();
        assert_eq!(tracker.state, State::ShortBreak);
    }

    #[test]
    fn calling_tick_within_5_mins_of_short_break_doesnt_transition() {
        let mut tracker = tracker_at_short_break();
        tracker.tick(Instant::now() + Duration::from_secs(4 * 60));
        assert_eq!(tracker.state, State::ShortBreak);
    }

    #[test]
    fn calling_tick_after_5_mins_of_short_break_transitions_to_PendingWorking() {
        let mut tracker = tracker_at_short_break();
        tracker.tick(Instant::now() + SHORT_BREAK_DURATION);
        assert_eq!(tracker.state, State::PendingWork);
    }

    #[test]
    fn after_four_intervals_transition_to_PendingLongerBreak() {
        let mut tracker = Tracker::new();
        // 1
        work(&mut tracker);
        assert_eq!(tracker.state, State::PendingShortBreak);
        short_break(&mut tracker);
        // 2
        work(&mut tracker);
        assert_eq!(tracker.state, State::PendingShortBreak);
        short_break(&mut tracker);
        // 3
        work(&mut tracker);
        assert_eq!(tracker.state, State::PendingShortBreak);
        short_break(&mut tracker);
        // 4
        work(&mut tracker);
        assert_eq!(tracker.state, State::PendingLongBreak);
    }

    #[test]
    fn calling_next_from_PendingLongBreak_transitions_to_LongBreak() {
        let mut tracker = tracker_at_pending_long_break();
        tracker.next();
        assert_eq!(tracker.state, State::LongBreak);
    }

    #[test]
    fn calling_tick_within_15_minutes_of_a_long_break_does_nothing() {
        let mut tracker = tracker_at_pending_long_break();
        tracker.next();
        tracker.tick(Instant::now() + Duration::from_secs(14 * 60));
        assert_eq!(tracker.state, State::LongBreak);
    }

    #[test]
    fn calling_tick_after_15_minutes_of_a_long_break_transitions_to_PendingWork() {
        let mut tracker = tracker_at_pending_long_break();
        tracker.next();
        tracker.tick(Instant::now() + LONG_BREAK_DURATION);
        assert_eq!(tracker.state, State::PendingWork);
    }

    #[test]
    fn after_eight_intervals_transition_to_PendingLongerBreak() {
        let mut tracker = Tracker::new();
        // 1
        work(&mut tracker);
        assert_eq!(tracker.state, State::PendingShortBreak);
        short_break(&mut tracker);
        // 2
        work(&mut tracker);
        assert_eq!(tracker.state, State::PendingShortBreak);
        short_break(&mut tracker);
        // 3
        work(&mut tracker);
        assert_eq!(tracker.state, State::PendingShortBreak);
        short_break(&mut tracker);
        // 4
        work(&mut tracker);
        assert_eq!(tracker.state, State::PendingLongBreak);
        tracker.next();
        tracker.tick(Instant::now() + LONG_BREAK_DURATION);
        // 5
        work(&mut tracker);
        assert_eq!(tracker.state, State::PendingShortBreak);
        short_break(&mut tracker);
        // 6
        work(&mut tracker);
        assert_eq!(tracker.state, State::PendingShortBreak);
        short_break(&mut tracker);
        // 7
        work(&mut tracker);
        assert_eq!(tracker.state, State::PendingShortBreak);
        short_break(&mut tracker);
        // 8
        work(&mut tracker);
        assert_eq!(tracker.state, State::PendingLongBreak);
    }
}
