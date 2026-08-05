//! Minimal `no_std` replacements for the `Time`/`Timer` types the game used to
//! import from `bevy_time`.
//!
//! The surface mirrors what `bevy_time` exposes (`Time::delta`, `Timer::tick`,
//! `just_finished`, `reset`, `TimerMode::Once/Repeating`) so existing game
//! systems keep reading the same without the `bevy_time` dependency.

use core::time::Duration;

/// A wall-clock stand-in advanced once per frame by the platform plugin.
///
/// Stores the delta since the last frame and the total elapsed time since the
/// app started; both expressed as `core::time::Duration` (no `std` needed).
#[derive(Debug, Clone)]
pub struct Time {
    /// Wall-clock duration since the previous frame.
    delta: Duration,
    /// Total elapsed wall-clock time since the world was created.
    elapsed: Duration,
}

// `Time` is inserted as a Bevy-style resource; the marker trait is implemented
// directly (rather than via the derive macro) because the derive references
// `::tiny_ecs`, which does not resolve from inside the crate itself.
impl super::Resource for Time {}

impl Default for Time {
    fn default() -> Self {
        Self {
            delta: Duration::ZERO,
            elapsed: Duration::ZERO,
        }
    }
}

impl Time {
    /// Creates a `Time` with zero delta and elapsed time.
    pub const fn new() -> Self {
        Self {
            delta: Duration::ZERO,
            elapsed: Duration::ZERO,
        }
    }

    /// Advances the clock by `delta`, adding to both delta and elapsed.
    pub fn advance_by(&mut self, delta: Duration) {
        self.delta = delta;
        self.elapsed += delta;
    }

    /// Returns the duration elapsed since the previous frame.
    pub const fn delta(&self) -> Duration {
        self.delta
    }

    /// Returns the total elapsed time since the world was created.
    pub const fn elapsed(&self) -> Duration {
        self.elapsed
    }
}

/// Whether a [`Timer`] repeats after finishing or stops for good.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TimerMode {
    /// Run once, then remain finished.
    #[default]
    Once,
    /// Restart automatically after each completion.
    Repeating,
}

/// A countdown timer, advanced by [`Timer::tick`].
///
/// Mirrors the small surface `bevy_time::Timer` the game relies on
/// (`from_seconds`, `tick`, `just_finished`, `is_finished`, `reset`).
#[derive(Debug, Clone)]
pub struct Timer {
    /// Total countdown length.
    duration: Duration,
    /// Time elapsed so far.
    elapsed: Duration,
    /// Whether the timer loops.
    mode: TimerMode,
    /// Set `true` on the tick the countdown completes; cleared on the next tick.
    just_finished: bool,
    /// Latched `true` after completion for `TimerMode::Once`.
    finished: bool,
}

impl Timer {
    /// Creates a timer that counts down `seconds` with the given mode.
    pub fn from_seconds(seconds: f32, mode: TimerMode) -> Self {
        Self {
            duration: Duration::from_secs_f32(seconds),
            elapsed: Duration::ZERO,
            mode,
            just_finished: false,
            finished: false,
        }
    }

    /// Builds a timer directly from a `Duration`.
    pub fn from_duration(duration: Duration, mode: TimerMode) -> Self {
        Self {
            duration,
            elapsed: Duration::ZERO,
            mode,
            just_finished: false,
            finished: false,
        }
    }

    /// Advances the timer by `delta`, recomputing the edge flags.
    pub fn tick(&mut self, delta: Duration) -> &Self {
        // Carrying a `just_finished` flag across ticks would let it linger; a
        // fresh tick clears it unless this tick itself reaches the end.
        self.just_finished = false;

        // An already-finished one-shot timer stays parked at the end and never
        // fires "just finished" again.
        if self.mode == TimerMode::Once && self.finished {
            return self;
        }

        self.elapsed += delta;
        if self.elapsed >= self.duration {
            self.just_finished = true;
            match self.mode {
                TimerMode::Repeating => {
                    // Carry the overshoot into the next cycle.
                    self.elapsed -= self.duration;
                }
                TimerMode::Once => {
                    self.elapsed = self.duration;
                    self.finished = true;
                }
            }
        }
        self
    }

    /// Returns `true` for exactly one tick after the timer completes.
    pub const fn just_finished(&self) -> bool {
        self.just_finished
    }

    /// Returns `true` while the timer is considered finished.
    ///
    /// For `TimerMode::Once` this latches once completed; for `Repeating` it is
    /// only true on the tick where the cycle wrapped.
    pub fn is_finished(&self) -> bool {
        match self.mode {
            TimerMode::Once => self.finished,
            TimerMode::Repeating => self.just_finished,
        }
    }

    /// Resets the timer to its starting state without changing its duration.
    pub fn reset(&mut self) {
        self.elapsed = Duration::ZERO;
        self.just_finished = false;
        self.finished = false;
    }

    /// Returns the configured countdown length.
    pub const fn duration(&self) -> Duration {
        self.duration
    }

    /// Returns the time elapsed in the current cycle.
    pub const fn elapsed(&self) -> Duration {
        self.elapsed
    }

    /// Returns the timer's mode.
    pub const fn mode(&self) -> TimerMode {
        self.mode
    }
}