use std::time::{Duration, Instant};

#[derive(Debug, PartialEq)]
enum State {
    PendingWork,
    Working,
    PendingShortBreak,
    ShortBreak,
}

pub struct Tracker {
    state: State,
    entered_state: Instant,
}

impl Tracker {
    pub fn new() -> Self {
        return Self {
            state: State::PendingWork,
            entered_state: Instant::now(),
        };
    }

    pub fn next(&mut self) {
        match self.state {
            State::PendingWork => {
                self.enter_state(State::Working);
            }
            State::PendingShortBreak => {
                self.enter_state(State::ShortBreak);
            },
            State::Working | State::ShortBreak => {},
        }
    }

    pub fn tick(&mut self, now: Instant) {
        match self.state {
            State::PendingWork | State::PendingShortBreak => {}
            State::Working => {
                if now.duration_since(self.entered_state) >= Duration::from_secs(20 * 60) {
                    self.enter_state(State::PendingShortBreak);
                }
            },
            State::ShortBreak => {
                unimplemented!();
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
        tracker.tick(Instant::now() + Duration::from_secs(20 * 60));
        assert_eq!(tracker.state, State::PendingShortBreak);
    }

    #[test]
    fn calling_tick_while_in_PendingWork_does_nothing() {
        let mut tracker = Tracker::new();
        assert_eq!(tracker.state, State::PendingWork);
        tracker.tick(Instant::now() + Duration::from_secs(60 * 60));
        assert_eq!(tracker.state, State::PendingWork);
    }

    #[test]
    fn calling_next_from_PendingShortBreak_transitions_to_short_break() {
        let mut tracker = Tracker::new();
        tracker.next();
        tracker.tick(Instant::now() + Duration::from_secs(20 * 60));
        assert_eq!(tracker.state, State::PendingShortBreak);
        tracker.next();
        assert_eq!(tracker.state, State::ShortBreak);
    }

}
