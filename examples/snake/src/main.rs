#![no_std]
#![no_main]

use cortex_m as _;
use cortex_m_rt::entry;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use bevy_microbit::prelude::*;

mod snake;

// The concrete `tiny_ecs` `World` for snake: no component columns (the snake
// body and food live in resources), just the platform resources the micro:bit
// plugins install plus the game's own resources.
define_world! {
    pub struct World {
        entities: 4,
        schedules: 8,
        resources {
            device: bevy_microbit::device::Device,
            entropy: bevy_microbit::device::Entropy,
            buttons: bevy_microbit::input::ButtonInput<bevy_microbit::input::GameButton>,
            frame: bevy_microbit::framebuffer::FrameBuffer,
            render: bevy_microbit::render::RenderState,
            time: tiny_ecs::time::Time,
            snake: snake::Snake,
            food: snake::Food,
            game_state: snake::GameState,
            move_timer: snake::MoveTimer,
        }
    }
}

/// Device entry point.
#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("microbit: snake boot");

    let world = World::new();

    App::new(world)
        .add_plugin(MicrobitPlugins)
        .add_plugin(snake::SnakePlugin)
        .run(microbit_runner);

    unreachable!("microbit_runner should never return")
}
