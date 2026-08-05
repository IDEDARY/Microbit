//! Time resource driving and its plugin.
//!
//! This is the no_std stand-in for Bevy's `TimePlugin`. Instead of reading an
//! OS clock, [`MicrobitTimePlugin`] advances the shared `Time` resource by a
//! fixed frame duration once per frame, so `Res<Time>` behaves like on desktop.

use bevy_ecs::prelude::ResMut;
use bevy_time::Time;

use crate::app::{App, Plugin, PreUpdate, FRAME_MILLIS};

/// Advances `Time` by one frame's worth of wall-clock time every frame.
pub struct MicrobitTimePlugin;
impl Plugin for MicrobitTimePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Time::<()>::default());
        app.add_systems(PreUpdate, advance_time);
    }
}

/// Moves the shared clock forward by the fixed frame period.
fn advance_time(mut time: ResMut<Time>) {
    time.advance_by(core::time::Duration::from_millis(FRAME_MILLIS as u64));
}
