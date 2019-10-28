use std::time::{Duration, Instant};

pub struct TrackerConfig {
    pub work_duration: Duration,
    pub short_break_duration: Duration,
    pub long_break_duration: Duration,
}

impl Default for TrackerConfig {
    fn default() -> Self {
        Self {
            work_duration: Duration::from_secs(20 * 60),
            short_break_duration: Duration::from_secs(5 * 60),
            long_break_duration: Duration::from_secs(15 * 60),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum State {
    PendingWork,
    Working,
    PendingShortBreak,
    ShortBreak,
    PendingLongBreak,
    LongBreak,
}

pub struct Tracker {
    pub state: State,
    entered_state: Instant,
    intervals: u64,
    config: TrackerConfig,
}

impl Tracker {
    pub fn new(config: TrackerConfig) -> Self {
        return Self {
            state: State::PendingWork,
            entered_state: Instant::now(),
            intervals: 0,
            config,
        };
    }

    pub fn next(&mut self) {
        match self.state {
            State::PendingWork => {
                self.enter_state(State::Working);
            }
            State::Working => {
                self.intervals += 1;
                if self.intervals % 4 == 0 {
                    self.enter_state(State::LongBreak);
                } else {
                    self.enter_state(State::ShortBreak);
                }
            }
            State::PendingShortBreak => {
                self.enter_state(State::ShortBreak);
            }
            State::PendingLongBreak => {
                self.enter_state(State::LongBreak);
            }
            State::ShortBreak | State::LongBreak => {
                self.enter_state(State::Working);
            }
        }
    }

    pub fn tick(&mut self, now: Instant) {
        match self.state {
            State::PendingWork | State::PendingShortBreak | State::PendingLongBreak => {}
            State::Working => {
                if now.duration_since(self.entered_state) >= self.config.work_duration {
                    self.intervals += 1;
                    if self.intervals % 4 == 0 {
                        self.enter_state(State::PendingLongBreak);
                    } else {
                        self.enter_state(State::PendingShortBreak);
                    }
                }
            }
            State::ShortBreak => {
                if now.duration_since(self.entered_state) >= self.config.short_break_duration {
                    self.enter_state(State::PendingWork);
                }
            }
            State::LongBreak => {
                if now.duration_since(self.entered_state) >= self.config.long_break_duration {
                    self.enter_state(State::PendingWork);
                }
            }
        }
    }

    pub fn time_remaining(&self, now: Instant) -> Option<Duration> {
        let time_since_entered_state = now - self.entered_state;
        match self.state {
            State::PendingWork | State::PendingShortBreak | State::PendingLongBreak => None,
            State::Working => Some(self.config.work_duration - time_since_entered_state),
            State::ShortBreak => Some(self.config.short_break_duration - time_since_entered_state),
            State::LongBreak => Some(self.config.long_break_duration - time_since_entered_state),
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

    const WORK_DURATION: Duration = Duration::from_secs(20 * 60);
    const SHORT_BREAK_DURATION: Duration = Duration::from_secs(5 * 60);
    const LONG_BREAK_DURATION: Duration = Duration::from_secs(15 * 60);

    fn create_tracker() -> Tracker {
        Tracker::new(Default::default())
    }

    fn tracker_at_short_break() -> Tracker {
        let mut tracker = create_tracker();
        tracker.next(); // -> working
        tracker.tick(Instant::now() + WORK_DURATION); // -> pending short break
        tracker.next(); // -> short break
        tracker
    }

    fn tracker_at_pending_long_break() -> Tracker {
        let mut tracker = create_tracker();
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

    macro_rules! assert_approx_eq {
        ($value:expr, $target:expr) => {{
            assert!(
                $value < $target + Duration::from_secs(1)
                    && $value > $target - Duration::from_secs(1),
                format!("expected {:?} to be approximately {:?}", $value, $target,)
            );
        }};
    }

    #[test]
    fn initial_state_is_PendingWork() {
        let tracker = create_tracker();
        assert_eq!(tracker.state, State::PendingWork);
    }

    #[test]
    fn calling_next_transitions_from_PendingWork_to_Working() {
        let mut tracker = create_tracker();
        tracker.next();
        assert_eq!(tracker.state, State::Working);
    }

    #[test]
    fn calling_next_transitions_from_Working_to_ShortBreak() {
        let mut tracker = create_tracker();
        tracker.next();
        tracker.next();
        assert_eq!(tracker.state, State::ShortBreak);
    }

    #[test]
    fn calling_tick_within_20_mins_of_working_doesnt_transition() {
        let mut tracker = create_tracker();
        tracker.next();
        tracker.tick(Instant::now() + Duration::from_secs(19 * 60));
        assert_eq!(tracker.state, State::Working);
    }

    #[test]
    fn calling_tick_after_20_mins_of_working_transitions_to_PendingShortBreak() {
        let mut tracker = create_tracker();
        tracker.next();
        tracker.tick(Instant::now() + WORK_DURATION);
        assert_eq!(tracker.state, State::PendingShortBreak);
    }

    #[test]
    fn calling_tick_while_in_PendingWork_does_nothing() {
        let mut tracker = create_tracker();
        assert_eq!(tracker.state, State::PendingWork);
        tracker.tick(Instant::now() + WORK_DURATION);
        assert_eq!(tracker.state, State::PendingWork);
    }

    #[test]
    fn calling_next_from_PendingShortBreak_transitions_to_ShortBreak() {
        let mut tracker = create_tracker();
        tracker.next();
        tracker.tick(Instant::now() + WORK_DURATION);
        assert_eq!(tracker.state, State::PendingShortBreak);
        tracker.next();
        assert_eq!(tracker.state, State::ShortBreak);
    }

    #[test]
    fn calling_next_from_ShortBreak_transitions_to_Working() {
        let mut tracker = tracker_at_short_break();
        tracker.next();
        assert_eq!(tracker.state, State::Working);
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
        let mut tracker = create_tracker();
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
    fn calling_next_from_LongBreak_transitions_to_Working() {
        let mut tracker = tracker_at_pending_long_break();
        tracker.next(); // -> LongBreak
        tracker.next();
        assert_eq!(tracker.state, State::Working);
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
        let mut tracker = create_tracker();
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

    #[test]
    fn exposes_time_remaining_in_state() {
        let five_secs = Duration::from_secs(5);
        let mut tracker = create_tracker();

        // PendingWork
        assert_eq!(tracker.time_remaining(Instant::now()), None);

        // Working
        tracker.next();
        assert_approx_eq!(
            tracker.time_remaining(Instant::now() + five_secs).unwrap(),
            WORK_DURATION - five_secs
        );

        // PendingShortBreak
        tracker.tick(Instant::now() + WORK_DURATION);
        assert_eq!(tracker.time_remaining(Instant::now()), None);

        // ShortBreak
        tracker.next();
        assert_approx_eq!(
            tracker.time_remaining(Instant::now() + five_secs).unwrap(),
            SHORT_BREAK_DURATION - five_secs
        );

        // PendingLongBreak
        tracker.tick(Instant::now() + SHORT_BREAK_DURATION);
        work(&mut tracker);
        short_break(&mut tracker);
        work(&mut tracker);
        short_break(&mut tracker);
        work(&mut tracker);
        assert_eq!(tracker.time_remaining(Instant::now()), None);

        // LongBreak
        tracker.next();
        assert_approx_eq!(
            tracker.time_remaining(Instant::now() + five_secs).unwrap(),
            LONG_BREAK_DURATION - five_secs
        );
    }

    #[test]
    fn calling_next_repeatedly_cycles_correctly() {
        let mut tracker = create_tracker();
        // 1st interval
        tracker.next();
        assert_eq!(tracker.state, State::Working);
        tracker.next();
        assert_eq!(tracker.state, State::ShortBreak);
        // 2nd interval
        tracker.next();
        assert_eq!(tracker.state, State::Working);
        tracker.next();
        assert_eq!(tracker.state, State::ShortBreak);
        // 3rd interval
        tracker.next();
        assert_eq!(tracker.state, State::Working);
        tracker.next();
        assert_eq!(tracker.state, State::ShortBreak);
        // 4th interval
        tracker.next();
        assert_eq!(tracker.state, State::Working);
        tracker.next();
        assert_eq!(tracker.state, State::LongBreak);
    }
}
