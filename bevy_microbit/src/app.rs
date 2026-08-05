use tiny_ecs::prelude::*;

/// Duration (ms) of a single row scan; three rows make one frame.
pub const FRAME_MILLIS: u32 = 3;

/// The number of LED rows the display multiplexes across.
pub const ROW_COUNT: usize = 3;

/// High-cadence schedule (~1 ms) driving the LED matrix row scan.
///
/// Defined here rather than in `tiny_ecs` to demonstrate that any crate can
/// mint its own schedule label via `#[derive(ScheduleLabel)]` without a central
/// enum.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, tiny_ecs_macros::ScheduleLabel)]
pub struct Tick;

/// The default micro:bit runner: runs `Startup` once, then loops driving the
/// `Tick` schedule every iteration (1 ms LED row scan) and the
/// `PreUpdate`/`Update`/`PostUpdate` schedules once every [`ROW_COUNT`] ticks
/// (one frame).
///
/// Pass this to [`App::run`] after all plugins are registered:
///
/// ```ignore
/// app.run(microbit_runner);
/// ```
pub fn microbit_runner<W: WorldApi>(world: &mut W) -> AppExit {
    // One-time setup.
    world.run_schedule(Startup);
    world.flush_commands();

    let mut tick: usize = 0;
    loop {
        // Refresh exactly one matrix row this tick (~1 ms).
        world.run_schedule(Tick);
        world.flush_commands();

        tick = tick.wrapping_add(1);
        if tick.is_multiple_of(ROW_COUNT) {
            // Refresh inputs and the clock, then run the game logic.
            world.run_schedule(PreUpdate);
            world.run_schedule(Update);
            world.run_schedule(PostUpdate);
            world.flush_commands();
        }
    }
}

/// Registrar for every platform plugin required by a micro:bit app.
///
/// Adding this plugin (e.g. `app.add_plugin(MicrobitPlugins)`) installs device
/// discovery, time, input, and LED rendering in one go, mirroring Bevy's plugin
/// groups.
pub struct MicrobitPlugins;
impl<W: WorldApi> Plugin<W> for MicrobitPlugins where
    W: HasResource<tiny_ecs::Time>
        + HasResource<crate::device::Entropy>
        + HasResource<crate::device::Device>
        + HasResource<crate::input::ButtonInput<crate::input::GameButton>>
        + HasResource<crate::framebuffer::FrameBuffer>
        + HasResource<crate::render::RenderState>
{
    fn build(&self, app: &mut tiny_ecs::app::App<W>) {
        app.add_plugin(crate::device::MicrobitDevicePlugin)
            .add_plugin(crate::time::MicrobitTimePlugin)
            .add_plugin(crate::input::MicrobitInputPlugin)
            .add_plugin(crate::render::MicrobitRenderingPlugin);
    }
}