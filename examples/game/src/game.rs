//! The debris-dodging mini-game, written entirely with Bevy primitives.
//!
//! There is not a single Microbit type in this file: it imports only the
//! generic interfaces exposed by `bevy_microbit` (`ButtonInput`, `FrameBuffer`,
//! `Entropy` plus the official Bevy types). The same modules would run on a
//! desktop mock by swapping the platform plugin.

use bevy_microbit::prelude::*;

/// Logical width of the playfield (matches the LED matrix columns).
const WIDTH: usize = 5;
/// Height of the playfield (matches the LED matrix rows).
const HEIGHT: usize = 5;
/// Minimum time between two player moves, in seconds.
const MOVE_COOLDOWN_SECS: f32 = 0.05;
/// How often a new row of debris is spawned, in seconds.
const SPAWN_INTERVAL_SECS: f32 = 0.5;
/// How often debris falls one row, in seconds.
const FALL_INTERVAL_SECS: f32 = 0.1;

/// The set of falling silhouette rows that can spawn, one per entry.
///
/// `true` marks a column occupied by debris in that pattern.
const OBSTACLES: [[bool; WIDTH]; 11] = [
    [true, false, true, false, true],
    [false, true, true, true, false],
    [true, true, true, false, false],
    [false, false, true, true, true],
    [true, true, false, true, true],
    [true, true, true, true, false],
    [true, true, true, false, true],
    [true, true, false, true, true],
    [true, false, true, true, true],
    [false, true, true, true, true],
    [false, true, false, true, false],
];

/// The player's paddle, represented as a single entity.
#[derive(Component)]
struct Player {
    /// Column the player currently occupies (0-4).
    x: usize,
}

/// A single falling piece of debris.
#[derive(Component)]
struct Debris {
    /// Column the debris currently occupies (0-4).
    x: usize,
    /// Row the debris currently occupies (0-4).
    y: usize,
}

/// Length of time since the player last moved.
#[derive(Component)]
struct MoveCooldown(Timer);

/// The player's running score.
#[derive(Resource, Default)]
struct Score(usize);

/// Top-level game state shared across systems.
#[derive(Resource)]
struct GameState {
    /// Whether the last round has ended (awaiting a reset).
    game_over: bool,
}
impl Default for GameState {
    /// Starts a fresh round.
    fn default() -> Self {
        Self { game_over: false }
    }
}

/// The spawn and fall cadences, ticked every frame.
#[derive(Resource)]
struct GameTimers {
    /// Drives new row spawning.
    spawn: Timer,
    /// Drives the falling movement.
    fall: Timer,
}
impl GameTimers {
    /// Creates the timers with the configured cadences.
    fn new() -> Self {
        Self {
            spawn: Timer::from_seconds(SPAWN_INTERVAL_SECS, TimerMode::Repeating),
            fall: Timer::from_seconds(FALL_INTERVAL_SECS, TimerMode::Repeating),
        }
    }

    /// Restarts both timers from the beginning (used on reset).
    fn reset(&mut self) {
        self.spawn.reset();
        self.fall.reset();
    }
}

/// Registers the game's resources, entities, and systems.
pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        /* app.add_systems(Startup, setup);
        app.add_systems(Update, (
            player_input,
            spawn_debris,
            fall_debris,
            collision,
            reset,
            draw,
        ).chain()); */
        app.add_systems(Update, |mut frame: ResMut<FrameBuffer>| {
            frame.set(0, 0, true);
        });
    }
}

/// Spawns the player and inserts the round-level resources.
fn setup(mut commands: Commands) {
    commands.insert_resource(Score(0));
    commands.insert_resource(GameState::default());
    commands.insert_resource(GameTimers::new());
    commands.spawn((
        Player { x: 2 },
        MoveCooldown(Timer::from_seconds(MOVE_COOLDOWN_SECS, TimerMode::Once)),
    ));
}

/// Moves the player left/right on button input, gated by a short cooldown.
fn player_input(
    input: Res<ButtonInput<GameButton>>,
    time: Res<Time>,
    state: Res<GameState>,
    mut player: Query<(&mut Player, &mut MoveCooldown)>,
) {
    if state.game_over {
        return;
    }
    let Ok((mut player, mut cooldown)) = player.single_mut() else {
        return;
    };

    cooldown.0.tick(time.delta());
    if !cooldown.0.is_finished() {
        return;
    }

    if input.just_pressed(GameButton::A) && player.x > 0 {
        player.x -= 1;
        cooldown.0.reset();
    } else if input.just_pressed(GameButton::B) && player.x < WIDTH - 1 {
        player.x += 1;
        cooldown.0.reset();
    }
}

/// Periodically spawns a random obstacle pattern into free columns.
fn spawn_debris(
    time: Res<Time>,
    state: Res<GameState>,
    mut timers: ResMut<GameTimers>,
    mut entropy: ResMut<Entropy>,
    mut commands: Commands,
    debris: Query<&Debris>,
) {
    timers.spawn.tick(time.delta());

    if state.game_over || !timers.spawn.just_finished() {
        return;
    }

    let pattern = OBSTACLES[entropy.next_below(OBSTACLES.len())];
    for (x, blocked) in pattern.into_iter().enumerate() {
        // Skip columns that already hold debris so pieces never stack.
        let occupied = debris.iter().any(|piece| piece.x == x);
        if blocked && !occupied {
            commands.spawn(Debris { x, y: 0 });
        }
    }
}

/// Advances every piece of debris one row, scoring when one exits the display.
fn fall_debris(
    time: Res<Time>,
    state: Res<GameState>,
    mut timers: ResMut<GameTimers>,
    mut score: ResMut<Score>,
    mut commands: Commands,
    mut debris: Query<(Entity, &mut Debris)>,
) {
    timers.fall.tick(time.delta());

    if state.game_over || !timers.fall.just_finished() {
        return;
    }

    for (entity, mut piece) in &mut debris {
        if piece.y >= HEIGHT - 1 {
            // The piece reached the bottom and is removed for a point.
            commands.entity(entity).despawn();
            score.0 += 1;
        } else {
            piece.y += 1;
        }
    }
}

/// Ends the round when the falling debris reaches the player.
fn collision(
    mut state: ResMut<GameState>,
    player: Query<&Player>,
    debris: Query<&Debris>,
) {
    if state.game_over {
        return;
    }
    let Ok(player) = player.single() else {
        return;
    };
    let hit = debris.iter().any(|piece| piece.x == player.x && piece.y == HEIGHT - 1);
    state.game_over = hit;
}

/// Starts a fresh round when A is pressed after a game over.
fn reset(
    input: Res<ButtonInput<GameButton>>,
    mut state: ResMut<GameState>,
    mut score: ResMut<Score>,
    mut timers: ResMut<GameTimers>,
    mut commands: Commands,
    mut player: Query<(&mut Player, &mut MoveCooldown)>,
    debris: Query<(Entity, &Debris)>,
) {
    if !state.game_over || !input.just_pressed(GameButton::A) {
        return;
    }

    state.game_over = false;
    score.0 = 0;
    timers.reset();
    for (entity, _) in &debris {
        commands.entity(entity).despawn();
    }
    if let Ok((mut player, mut cooldown)) = player.single_mut() {
        player.x = 2;
        cooldown.0.reset();
    }
}

/// Renders the player and all debris into the shared frame buffer.
fn draw(mut frame: ResMut<FrameBuffer>, player: Query<&Player>, debris: Query<&Debris>) {
    frame.clear();
    if let Ok(player) = player.single() {
        frame.set(player.x, HEIGHT - 1, true);
    }
    for piece in &debris {
        frame.set(piece.x, piece.y, true);
    }
}
