//! The embedded application runner and its custom schedule labels.
//!
//! [`microbit_runner`] is installed as Bevy's [`App`] runner, so the game entry
//! point looks identical to a desktop Bevy app (`App::new().add_plugins(..)`
//! `.run()`). It drives the two software cadences of a micro:bit display:
//!
//! * a 1 ms **tick** that scans one row of the LED matrix out to the pins, and
//! * a **frame** (every 3 ticks) in which the game `Update` systems run.
//!
//! Game code only ever sees the `Startup`, `PreFrame` and `Update` schedules;
//! [`Tick`] is a backend detail owned by the rendering plugin.

use bevy_app::{App, AppExit, PostUpdate, PreUpdate, Startup, Update};
use bevy_ecs::schedule::ScheduleLabel;
use rtt_target::rprintln;

use crate::render::RenderState;

/// Duration (ms) of a single frame of game logic. One frame spans the 3 rows
/// of the scanned display.
pub const FRAME_MILLIS: u32 = 3;

/// The application runner used by [`microbit_runner`] to drive the loop.
///
/// This schedule owns the per-1 ms LED row refresh and is normally not used by
/// game code.
#[derive(Clone, Debug, PartialEq, Eq, Hash, ScheduleLabel)]
pub struct Tick;

/// The embedded runner installed as Bevy's `App` runner.
///
/// Runs the one-time `Startup` schedule, then loops forever refreshing the
/// display: it executes `PreFrame` + `Update` whenever a new frame begins and
/// `Tick` on every 1 ms tick. Never returns.
pub fn microbit_runner(mut app: App) -> AppExit {

    // Run startup schedule
    app.world_mut().run_schedule(Startup);

    loop {
        // A new frame begins whenever the scan pointer wraps back to row 0.
        let new_frame = app.world_mut().get_resource::<RenderState>().expect("MicrobitPlugins not added").row == 0;

        // Refresh inputs and the clock, then run the game logic.
        if new_frame {
            app.world_mut().run_schedule(PreUpdate);
            app.world_mut().run_schedule(Update);
            app.world_mut().run_schedule(PostUpdate);
        }

        // Always refresh exactly one matrix row this tick.
        app.world_mut().run_schedule(Tick);
    }
}

/// Entry point recommended for the embedded target.
///
/// Unlike [`App::run`], this drives the schedules **in place** on the given
/// `&mut App` and never copies the (comparatively large) `World` off the stack,
/// so it is safe on the micro:bit's limited RAM. Runs `Startup` once, then
/// loops forever running `PreUpdate`/`Update`/`PostUpdate` each frame and
/// `Tick` every 1 ms. Never returns.
pub fn start(app: &mut App) -> ! {
    rprintln!("runner: enter");

    // Run the one-time setup schedule.
    app.world_mut().run_schedule(Startup);
    rprintln!("runner: startup done");

    let mut frames = 0usize;
    loop {
        // A new frame begins whenever the scan pointer wraps back to row 0.
        let new_frame = app
            .world_mut()
            .get_resource::<RenderState>()
            .expect("MicrobitPlugins not added")
            .row
            == 0;

        if new_frame {
            // Refresh inputs and the clock, then run the game logic.
            app.world_mut().run_schedule(PreUpdate);
            app.world_mut().run_schedule(Update);
            app.world_mut().run_schedule(PostUpdate);

            frames += 1;
            rprintln!("runner: frame {}", frames);
        }

        // Always refresh exactly one matrix row this tick.
        app.world_mut().run_schedule(Tick);
    }
}
