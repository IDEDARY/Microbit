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

/// Registrar for every platform plugin required by a micro:bit app.
///
/// Adding this plugin (e.g. `app.add_plugin(MicrobitPlugins)`) installs device
/// discovery, time, input, and LED rendering in one go, mirroring Bevy's plugin
/// groups.
pub struct MicrobitPlugins;
impl<W: WorldApi> Plugin<W> for MicrobitPlugins where
    W: HasResource<crate::device::Device>
        + HasResource<crate::device::Entropy>
        + HasResource<tiny_ecs::Time>
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