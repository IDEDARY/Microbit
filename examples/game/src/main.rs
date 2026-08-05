#![no_std]
#![no_main]

use cortex_m as _;
use cortex_m_rt::entry;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use bevy_microbit::prelude::*;

mod game;

// The concrete `tiny_ecs` `World` for this game: every component column and
// resource the app uses, enumerated up front so the whole structure has a
// fixed, compile-time RAM footprint.
define_world! {
    pub struct World {
        entities: 64,
        schedules: 8,
        components {
            player: game::Player [4],
            debris: game::Debris [64],
            move_cooldown: game::MoveCooldown [4],
        }
        resources {
            device: bevy_microbit::device::Device,
            entropy: bevy_microbit::device::Entropy,
            buttons: bevy_microbit::input::ButtonInput<bevy_microbit::input::GameButton>,
            frame: bevy_microbit::framebuffer::FrameBuffer,
            render: bevy_microbit::render::RenderState,
            time: tiny_ecs::time::Time,
            score: game::Score,
            game_state: game::GameState,
            timers: game::GameTimers,
        }
    }
}

/// Device entry point.
#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("microbit: boot");

    // Create the ECS world
    let world = World::new();

    // Create the app and run it
    App::new(world)
        .add_plugin(MicrobitPlugins)
        .add_plugin(game::GamePlugin)
        .run(microbit_runner);

    unreachable!("microbit_runner should never return")
}