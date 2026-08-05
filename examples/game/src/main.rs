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
        label: GameSchedule,
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

/// The app's schedule labels, mirroring Bevy's `Startup`/`PreUpdate`/
/// `Update`/`PostUpdate` plus the high-cadence `Tick` used by the LED scan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameSchedule {
    /// Default / zero-value variant (required by `SmallMap` key init).
    #[default]
    None,
    /// One-time setup.
    Startup,
    /// Per-frame, before `Update`.
    PreUpdate,
    /// Per-frame gameplay systems.
    Update,
    /// Per-frame, after `Update`.
    PostUpdate,
    /// ~1 ms display row scan.
    Tick,
}
impl StandardSchedules for GameSchedule {
    fn startup() -> Self {
        GameSchedule::Startup
    }
    fn pre_update() -> Self {
        GameSchedule::PreUpdate
    }
    fn update() -> Self {
        GameSchedule::Update
    }
    fn post_update() -> Self {
        GameSchedule::PostUpdate
    }
    fn tick() -> Self {
        GameSchedule::Tick
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
    let world = World::<GameSchedule>::new();
    let mut app = App::new(world);
    app.add_schedule(GameSchedule::Startup);
    app.add_schedule(GameSchedule::PreUpdate);
    app.add_schedule(GameSchedule::Update);
    app.add_schedule(GameSchedule::PostUpdate);
    app.add_schedule(GameSchedule::Tick);
    app.add_plugin(MicrobitPlugins);
    app.add_plugin(game::GamePlugin);

    let mut world = app.into_world();

    // Run one-time setup explicitly (player spawn, round resources).
    game::setup(&mut world);
    world.run_schedule(&GameSchedule::Startup);
    world.flush_commands();
    rprintln!("microbit: startup done");

    // The runner loop: drive the schedules at the micro:bit's cadence. Each
    // iteration is one 1 ms display row scan; every third scan is a new frame.
    let mut tick = 0usize;
    let mut frames = 0usize;
    loop {
        // Refresh exactly one matrix row this tick.
        world.run_schedule(&GameSchedule::Tick);
        world.flush_commands();

        tick = tick.wrapping_add(1);
        if tick.is_multiple_of(ROWS_PER_FRAME) {
            // Refresh inputs and the clock, then run the game logic.
            world.run_schedule(&GameSchedule::PreUpdate);
            world.run_schedule(&GameSchedule::Update);
            world.run_schedule(&GameSchedule::PostUpdate);
            world.flush_commands();

            frames += 1;
            rprintln!("runner: frame {}", frames);
        }
    }
}