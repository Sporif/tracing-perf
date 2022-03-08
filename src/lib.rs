#![doc = include_str!("../README.md")]
#![cfg_attr(
    feature = "docsrs",
    cfg_attr(doc, doc = ::document_features::document_features!())
)]
#![forbid(unsafe_code)]
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]

use std::fmt;

#[cfg(feature = "start-print-order")]
use indexmap::IndexMap as HashMap;
#[cfg(not(feature = "start-print-order"))]
use std::collections::HashMap;

#[cfg(not(feature = "minstant"))]
use std::time::{Duration, Instant};
#[cfg(feature = "minstant")]
use ::{minstant_crate::Instant, std::time::Duration};

use tracing::Level;

/// Enum containing possible printing orders of total times.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum PrintOrder {
    /// Starting order.
    #[cfg(feature = "start-print-order")]
    Start,
    /// Reverse starting order.
    #[cfg(feature = "start-print-order")]
    RevStart,
    /// Key name.
    Key,
    /// Reverse key name.
    RevKey,
    /// Increasing duration.
    IncDuration,
    /// Decreasing duration.
    DecDuration,
}

impl Default for PrintOrder {
    #[cfg(feature = "start-print-order")]
    fn default() -> Self {
        Self::Start
    }
    #[cfg(not(feature = "start-print-order"))]
    fn default() -> Self {
        Self::DecDuration
    }
}

/// A configurable builder for a `TimeReporter`
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimeReporterBuilder {
    name: String,
    level: Level,
    print_order: PrintOrder,
    width: usize,
    precision: usize,
}

impl TimeReporterBuilder {
    /// Create a new `TimeReporter` builder with the given name
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            level: Level::INFO,
            print_order: PrintOrder::default(),
            width: 11,
            precision: 9,
        }
    }

    #[must_use]
    pub fn build(&self) -> TimeReporter {
        TimeReporter {
            times: HashMap::new(),
            name: self.name.clone(),
            cur_state_time: None,
            level: self.level,
            print_order: self.print_order,
            width: self.width,
            precision: self.precision,
        }
    }

    /// Set the logging level.
    pub fn level(&mut self, level: Level) -> &mut Self {
        self.level = level;
        self
    }

    /// Set the printing order of the total times.
    pub fn print_order(&mut self, print_order: PrintOrder) -> &mut Self {
        self.print_order = print_order;
        self
    }

    /// Set the minimum formatting width of the total times.
    ///
    /// Note: Should be at least `precision + 2`  (i.e at least one leading digit + decimal point + precision)
    /// or this option will have no effect. Alternatively precision should be at most `width - 2`.
    ///
    /// Fill character and alignment are hardcoded to space and left-align.
    pub fn width(&mut self, width: usize) -> &mut Self {
        self.width = width;
        self
    }

    /// Set the number of digits after the decimal point
    /// that should be printed for the total times.
    pub fn precision(&mut self, precision: usize) -> &mut Self {
        self.precision = precision;
        self
    }
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
    print_order: PrintOrder,
    width: usize,
    precision: usize,
}

impl TimeReporter {
    /// Create a new `TimeReporter`.
    pub fn new<S: Into<String>>(name: S) -> Self {
        TimeReporterBuilder::new(name).build()
    }

    /// Create a new `TimeReporter` with a specified level.
    pub fn new_with_level<S: Into<String>>(name: S, level: Level) -> Self {
        TimeReporterBuilder::new(name).level(level).build()
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

fn get_times(
    times: &HashMap<&'static str, Duration>,
    print_order: PrintOrder,
) -> Vec<(&'static str, Duration)> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "start-print-order")] {
            if print_order == PrintOrder::RevStart {
                times.iter().rev().map(|(&k, &v)| (k, v)).collect()
            } else {
                times.iter().map(|(&k, &v)| (k, v)).collect()
            }
        } else {
            let _ = print_order;
            times.iter().map(|(&k, &v)| (k, v)).collect()
        }
    }
}

impl<'a> fmt::Display for TimeReporter {
    #[allow(clippy::cast_precision_loss)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut stats: Vec<(&'static str, Duration)> = get_times(&self.times, self.print_order);
        match self.print_order {
            #[cfg(feature = "start-print-order")]
            PrintOrder::Start | PrintOrder::RevStart => {}
            PrintOrder::Key => stats.sort_by_key(|s| s.0),
            PrintOrder::RevKey => stats.sort_by(|a, b| b.0.cmp(a.0)),
            PrintOrder::IncDuration => stats.sort_by_key(|s| s.1),
            PrintOrder::DecDuration => stats.sort_by(|a, b| b.1.cmp(&a.1)),
        }

        write!(f, "name: {}", self.name)?;
        let precision = self.precision;
        let width = self.width;
        for &(state, dur) in &stats {
            let dur = dur.as_secs() as f64 + f64::from(dur.subsec_nanos()) / 1_000_000_000_f64;
            write!(f, ", {}: {:<width$.precision$}", state, dur)?;
        }

        Ok(())
    }
}

macro_rules! _span {
    ($lvl:expr, $($args:tt)*) => {{
        match $lvl {
            Level::ERROR => ::tracing::span!(Level::ERROR, $($args)*),
            Level::WARN  => ::tracing::span!(Level::WARN,  $($args)*),
            Level::INFO  => ::tracing::span!(Level::INFO,  $($args)*),
            Level::DEBUG => ::tracing::span!(Level::DEBUG, $($args)*),
            Level::TRACE => ::tracing::span!(Level::TRACE, $($args)*),
        }
    }};
}

macro_rules! _event {
    (target: $target:expr, $lvl:expr, $($args:tt)*) => {{
        match $lvl {
            Level::ERROR => ::tracing::event!(target: $target, Level::ERROR, $($args)*),
            Level::WARN  => ::tracing::event!(target: $target, Level::WARN,  $($args)*),
            Level::INFO  => ::tracing::event!(target: $target, Level::INFO,  $($args)*),
            Level::DEBUG => ::tracing::event!(target: $target, Level::DEBUG, $($args)*),
            Level::TRACE => ::tracing::event!(target: $target, Level::TRACE, $($args)*),
        }
    }};
}

impl Drop for TimeReporter {
    fn drop(&mut self) {
        let _span = _span!(self.level, "time-report").entered();
        _event!(target: "tracing-perf", self.level, "{}", self);
    }
}
