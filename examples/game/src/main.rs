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

    // A frame is exactly three display row scans (3 ms).
    const ROWS_PER_FRAME: usize = 3;

    // Assemble the world and install every platform plugin plus the game.
    let world = World::new();
    let mut app = App::new(world);
    app.add_plugin(MicrobitPlugins);
    app.add_plugin(game::GamePlugin);

    let mut world = app.into_world();

    // Run one-time setup explicitly (player spawn, round resources).
    game::setup(&mut world);
    world.run_schedule(Startup);
    world.flush_commands();
    rprintln!("microbit: startup done");

    // The runner loop: drive the schedules at the micro:bit's cadence. Each
    // iteration is one 1 ms display row scan; every third scan is a new frame.
    let mut tick = 0usize;
    let mut frames = 0usize;
    loop {
        // Refresh exactly one matrix row this tick.
        world.run_schedule(Tick);
        world.flush_commands();

        tick = tick.wrapping_add(1);
        if tick.is_multiple_of(ROWS_PER_FRAME) {
            // Refresh inputs and the clock, then run the game logic.
            world.run_schedule(PreUpdate);
            world.run_schedule(Update);
            world.run_schedule(PostUpdate);
            world.flush_commands();

            frames += 1;
            rprintln!("runner: frame {}", frames);
        }
    }
}