//! This crate provides very useful tools for reporting performance metrics
//! through `tracing`.

#![forbid(unsafe_code)]
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]

use std::{collections::HashMap, fmt};

#[cfg(not(feature = "minstant"))]
use std::time::{Duration, Instant};
#[cfg(feature = "minstant")]
use ::{minstant::Instant, std::time::Duration};

use tracing::Level;

macro_rules! span {
    ($lvl:expr, $($args:tt)*) => {{
        match $lvl {
            Level::ERROR => ::tracing::span!(Level::ERROR, $($args)*),
            Level::WARN => ::tracing::span!(Level::WARN, $($args)*),
            Level::INFO => ::tracing::span!(Level::INFO, $($args)*),
            Level::DEBUG => ::tracing::span!(Level::DEBUG, $($args)*),
            Level::TRACE => ::tracing::span!(Level::TRACE, $($args)*),
        }
    }};
}

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

/// Collect and report total time spent on set of activities.
///
/// `TimeReporter` is useful for generating and reporting
/// time reports: how much time was spend on a given activity.
///
/// On `drop` or on call to `finish` it will report total times
/// gathered as a `tracing` event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimeReporter {
    times: HashMap<&'static str, Duration>,
    cur_state_time: Option<(&'static str, Instant)>,
    name: String,
    level: Level,
}

impl TimeReporter {
    /// Create new `TimeReporter`.
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            times: HashMap::new(),
            name: name.into(),
            cur_state_time: None,
            level: Level::INFO,
        }
    }

    /// Create new `TimeReporter` with a specified level.
    pub fn new_with_level<S: Into<String>>(name: S, level: Level) -> Self {
        Self {
            times: HashMap::new(),
            name: name.into(),
            cur_state_time: None,
            level,
        }
    }

    /// Start counting time for a state named "key".
    ///
    /// If this `TimeReporter` was already counting time
    /// for another state, it will end counting time for it
    /// before starting a new one.
    pub fn start(&mut self, key: &'static str) {
        let now = Instant::now();

        self.save_current(now);

        self.cur_state_time = Some((key, now));
    }

    /// Start counting time and execute a function `f`.
    ///
    /// This is handy syntax for `if let` or `while let` expressions
    /// where it would be inconvenient to add a standalone `start` call.
    pub fn start_with<F, R>(&mut self, key: &'static str, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        self.start(key);

        f()
    }

    fn save_current(&mut self, now: Instant) {
        if let Some((key, prev)) = self.cur_state_time.take() {
            *self.times.entry(key).or_insert_with(|| Duration::new(0, 0)) += now - prev;
        }
    }

    /// Stop counting time.
    pub fn stop(&mut self) {
        let now = Instant::now();
        self.save_current(now);
    }

    /// Finish counting time and report results.
    #[allow(clippy::unused_self)]
    pub fn finish(self) {}
}

impl<'a> fmt::Display for TimeReporter {
    #[allow(clippy::cast_precision_loss, clippy::cast_lossless)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut stats: Vec<(&'static str, Duration)> =
            self.times.iter().map(|(&k, &v)| (k, v)).collect();
        stats.sort_by_key(|s| s.1);

        write!(f, "name: {}", self.name)?;
        for &(state, dur) in stats.iter().rev() {
            write!(
                f,
                ", {}: {}",
                state,
                dur.as_secs() as f64 + dur.subsec_nanos() as f64 / 1_000_000_000_f64
            )?;
        }

        Ok(())
    }
}

impl Drop for TimeReporter {
    fn drop(&mut self) {
        let _span = span!(self.level, "time-report").entered();
        event!(target: "tracing-perf", self.level, "{}", self);
    }
}
