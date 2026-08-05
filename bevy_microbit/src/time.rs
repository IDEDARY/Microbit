//! Minimal `no_std` time resource driving and its plugin.
//!
//! This is the stand-in for Bevy's `TimePlugin`. Instead of reading an OS
//! clock, [`MicrobitTimePlugin`] advances the shared `Time` resource by a
//! fixed frame duration once per frame, so `Res<Time>` behaves like on a
//! desktop.

use tiny_ecs::prelude::*;

use crate::app::FRAME_MILLIS;

/// Advances `Time` by one frame's worth of wall-clock time every frame.
pub struct MicrobitTimePlugin;
impl<W: WorldApi> Plugin<W> for MicrobitTimePlugin where W: HasResource<tiny_ecs::Time> {
    fn build(&self, app: &mut App<W>) {
        app.insert_resource(tiny_ecs::Time::new());
        app.add_system(tiny_ecs::schedule::PreUpdate, advance_time);
    }
}

/// Moves the shared clock forward by the fixed frame period.
#[system]
fn advance_time(mut time: ResMut<Time>) {
    time.advance_by(core::time::Duration::from_millis(FRAME_MILLIS as u64));
}