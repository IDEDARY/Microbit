//! A minimal Bevy-idiomatic application shell built directly on `bevy_ecs`.
//!
//! The full `bevy_app` crate is far too heavy for the micro:bit's 16 KiB of
//! RAM (its `App`/sub-app/schedule-manager pushes heap *and* stack past the
//! limit). This module re-implements just the small surface the game uses —
//! [`App`], [`Plugin`], the `Startup`/`PreUpdate`/`Update`/`PostUpdate`
//! schedules, and [`App::run`] — on top of [`bevy_ecs::World`] so game code
//! reads exactly like a desktop Bevy app without the memory cost.
//!
//! [`App::run`] drives two software cadences of a micro:bit display:
//!
//! * a 1 ms **tick** that scans one row of the LED matrix out to the pins, and
//! * a **frame** (every 3 ticks) in which the game `Update` systems run.

use bevy_ecs::prelude::{FromWorld, Resource, World};
use bevy_ecs::schedule::{IntoScheduleConfigs, Schedule, ScheduleLabel, Schedules};
use bevy_ecs::system::ScheduleSystem;
use rtt_target::rprintln;

use crate::render::RenderState;

/// Duration (ms) of a single frame of game logic. One frame spans the 3 rows
/// of the scanned display.
pub const FRAME_MILLIS: u32 = 3;

/// Runs once at application startup (device discovery, spawning).
#[derive(Clone, Debug, PartialEq, Eq, Hash, ScheduleLabel)]
pub struct Startup;

/// Runs at the start of each frame, before [`Update`].
///
/// Platform plugins (input sampling, time advance) hook in here so game systems
/// observe a consistent world state every frame.
#[derive(Clone, Debug, PartialEq, Eq, Hash, ScheduleLabel)]
pub struct PreUpdate;

/// Runs once per frame and holds the game's gameplay systems.
#[derive(Clone, Debug, PartialEq, Eq, Hash, ScheduleLabel)]
pub struct Update;

/// Runs at the end of each frame, after [`Update`].
#[derive(Clone, Debug, PartialEq, Eq, Hash, ScheduleLabel)]
pub struct PostUpdate;

/// Runs every ~1 ms to refresh one row of the LED matrix.
///
/// Owned by the backend rendering plugin; normally not used by game code.
#[derive(Clone, Debug, PartialEq, Eq, Hash, ScheduleLabel)]
pub struct Tick;

/// Extension point mirroring Bevy's `Plugin` trait.
pub trait Plugin {
    /// Registers systems, resources, and startup logic with the [`App`].
    fn build(&self, app: &mut App);
}

/// Sealed tuple marker used by [`App::add_plugins`] to register a bundle of
/// plugins in order.
pub trait Plugins {
    /// Adds every contained plugin to the app.
    fn add(self, app: &mut App);
}

impl Plugins for () {
    fn add(self, _app: &mut App) {}
}

impl<P: Plugin> Plugins for P {
    fn add(self, app: &mut App) {
        app.add_plugin(self);
    }
}

/// Implements `Plugins` for tuples of plugins, mirroring `bevy`'s plugin groups.
macro_rules! impl_plugins {
    ($($p:ident),+) => {
        impl<$($p: Plugin),+> Plugins for ($($p,)+) {
            #[allow(non_snake_case)]
            fn add(self, app: &mut App) {
                let ($($p,)+) = self;
                $(app.add_plugin($p);)+
            }
        }
    };
}

impl_plugins!(A, B);
impl_plugins!(A, B, C);
impl_plugins!(A, B, C, D);
impl_plugins!(A, B, C, D, E);
impl_plugins!(A, B, C, D, E, F);
impl_plugins!(A, B, C, D, E, F, G);
impl_plugins!(A, B, C, D, E, F, G, H);

/// The embedded application container, wrapping a single [`World`].
pub struct App {
    /// The underlying ECS world holding all resources, entities, and schedules.
    world: World,
}

impl Default for App {
    /// Creates an app with the schedules the runner drives pre-registered.
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    /// Creates a new application with the relevant schedules registered.
    pub fn new() -> Self {
        let mut app = Self {
            world: World::new(),
        };
        let mut schedules = app.world.get_resource_or_init::<Schedules>();
        schedules.insert(Schedule::new(Startup));
        schedules.insert(Schedule::new(PreUpdate));
        schedules.insert(Schedule::new(Update));
        schedules.insert(Schedule::new(PostUpdate));
        schedules.insert(Schedule::new(Tick));
        app
    }

    /// Returns an immutable borrow of the underlying world.
    pub fn world(&self) -> &World {
        &self.world
    }

    /// Returns a mutable borrow of the underlying world.
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    /// Inserts a resource into the world.
    pub fn insert_resource<R: Resource>(&mut self, resource: R) -> &mut Self {
        self.world.insert_resource(resource);
        self
    }

    /// Inserts a resource built from the world, if it is not already present.
    pub fn init_resource<R: Resource + FromWorld>(&mut self) -> &mut Self {
        self.world.init_resource::<R>();
        self
    }

    /// Adds systems to a schedule.
    pub fn add_systems<M>(
        &mut self,
        schedule: impl ScheduleLabel,
        systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> &mut Self {
        self.world
            .get_resource_or_init::<Schedules>()
            .add_systems(schedule, systems);
        self
    }

    /// Registers a single plugin.
    pub fn add_plugin(&mut self, plugin: impl Plugin) -> &mut Self {
        plugin.build(self);
        self
    }

    /// Registers a group of plugins (or a single plugin), consuming the app so
    /// it can be chained into [`App::run`].
    pub fn add_plugins(mut self, plugins: impl Plugins) -> Self {
        plugins.add(&mut self);
        self
    }

    /// Runs the application forever: `Startup` once, then per frame
    /// `PreUpdate`/`Update`/`PostUpdate` and `Tick` every 1 ms.
    ///
    /// Drives the world in place (never copying it off the stack) and never
    /// returns.
    pub fn run(mut self) -> ! {
        rprintln!("runner: enter");

        // Run the one-time setup schedule.
        self.world.run_schedule(Startup);
        rprintln!("runner: startup done");

        let mut frames = 0usize;
        loop {
            // A new frame begins whenever the scan pointer wraps back to row 0.
            let new_frame = self
                .world
                .get_resource::<RenderState>()
                .expect("MicrobitPlugins not added")
                .row
                == 0;

            if new_frame {
                // Refresh inputs and the clock, then run the game logic.
                self.world.run_schedule(PreUpdate);
                self.world.run_schedule(Update);
                self.world.run_schedule(PostUpdate);

                frames += 1;
                rprintln!("runner: frame {}", frames);
            }

            // Always refresh exactly one matrix row this tick.
            self.world.run_schedule(Tick);
        }
    }
}
