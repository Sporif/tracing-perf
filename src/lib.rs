//! This crate provides very useful tools for reporting performance metrics
//! through `tracing`.

use std::{collections::HashMap, fmt, time};
#[cfg(not(feature = "minstant"))]
use time::Instant;

#[cfg(feature = "minstant")]
use minstant::Instant;
use tracing::{span, Level};

macro_rules! event {
    (target: $target:expr, $lvl:expr, $($args:tt)*) => {{
        match $lvl {
            Level::ERROR => ::tracing::event!(target: $target, Level::ERROR, $($args)*),
            Level::WARN => ::tracing::event!(target: $target, Level::WARN, $($args)*),
            Level::INFO => ::tracing::event!(target: $target, Level::INFO, $($args)*),
            Level::DEBUG => ::tracing::event!(target: $target, Level::DEBUG, $($args)*),
            Level::TRACE => ::tracing::event!(target: $target, Level::TRACE, $($args)*),
        }
    }};
}

/// Collect and report total time spent on set of activities
///
/// `TimeReporter` is useful for generating and reporting
/// time reports: how much time was spend on a given activity.
///
/// On `drop` or on call to `finish` it will report total times
/// gathered as a `tracing` event.
pub struct TimeReporter {
    times: HashMap<&'static str, time::Duration>,
    cur_state_time: Option<(&'static str, Instant)>,
    name: String,
    level: Level,
}

impl TimeReporter {
    /// Create new `TimeReporter`
    pub fn new<S: Into<String>>(name: S) -> TimeReporter {
        TimeReporter {
            times: HashMap::new(),
            name: name.into(),
            cur_state_time: None,
            level: Level::INFO,
        }
    }

    /// Create new `TimeReporter`
    pub fn new_with_level<S: Into<String>>(name: S, level: Level) -> TimeReporter {
        TimeReporter {
            times: HashMap::new(),
            name: name.into(),
            cur_state_time: None,
            level,
        }
    }

    /// Start counting time for a state named "key"
    ///
    /// If this the `TimeReporter` was already counting time
    /// for another state, it will end counting time for it
    /// before starting new one.
    pub fn start(&mut self, key: &'static str) {
        let now = Instant::now();

        self.save_current(now);

        self.cur_state_time = Some((key, now))
    }

    /// Start counting time and execute a function `f`
    ///
    /// This is handy syntax for `if let` or `while let` expressions
    /// where it would be inconvenient to add standalone `start` call.
    pub fn start_with<F, R>(&mut self, key: &'static str, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        self.start(key);

        f()
    }

    fn save_current(&mut self, now: Instant) {
        if let Some((key, prev)) = self.cur_state_time.take() {
            *self
                .times
                .entry(key)
                .or_insert_with(|| time::Duration::new(0, 0)) += now - prev;
        }
    }

    pub fn stop(&mut self) {
        let now = Instant::now();
        self.save_current(now);
    }

    /// Finish counting time and report results
    pub fn finish(self) {}
}

impl<'a> fmt::Display for TimeReporter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut stats: Vec<(&'static str, time::Duration)> =
            self.times.iter().map(|(&k, &v)| (k, v)).collect();
        stats.sort_by_key(|s| s.1);

        write!(f, "name: {}", self.name)?;
        for &(state, dur) in stats.iter().rev() {
            write!(
                f,
                ", {}: {}",
                state,
                dur.as_secs() as f64 + dur.subsec_nanos() as f64 / 1000000000f64
            )?;
        }

        Ok(())
    }
}

impl Drop for TimeReporter {
    fn drop(&mut self) {
        let _span = span!(Level::INFO, "time-report").entered();
        event!(target: "tracing-perf", self.level, "{}", self)
    }
}
