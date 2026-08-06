//! Snake for the BBC micro:bit.
//!
//! A classic snake game on the 5x5 LED matrix. Button A turns left, button B
//! turns right (relative steering). Eat the pulsing food dot to grow; hitting
//! a wall or yourself ends the round — press A to restart.
//!
//! The snake body lives in a single [`Snake`] resource (a fixed array of cell
//! coordinates) rather than as per-segment entities, because the ordered
//! tail-to-head shift is trivial on a flat array and avoids the entity churn
//! that tiny_ecs' deferred commands would make awkward for same-tick
//! respawn. Food is likewise a singleton [`Food`] resource, relocated
//! synchronously on each eat so the renderer always sees a consistent
//! position.

use bevy_microbit::prelude::*;

/// Side length of the square LED grid.
const GRID: usize = 5;
/// Maximum snake length (fills the entire grid).
const MAX_LEN: usize = GRID * GRID;

/// Seconds between snake movement steps.
const MOVE_SECS: f32 = 0.50;

/// Brightness of the snake head.
const HEAD_BRIGHTNESS: u8 = 255;
/// Brightness of the snake body segments.
const BODY_BRIGHTNESS: u8 = 10;
/// Minimum brightness of the pulsing food.
const FOOD_MIN: u8 = 10;

/// Starting head position (centre).
const INITIAL_HEAD: (usize, usize) = (2, 2);
/// Starting food position (ahead of the head).
const INITIAL_FOOD: (usize, usize) = (4, 2);

// ---------------------------------------------------------------------
// --- Direction -------------------------------------------------------
// ---------------------------------------------------------------------

/// One of the four cardinal directions the snake can face.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    /// Returns the direction 90 degrees counter-clockwise.
    fn turn_left(self) -> Self {
        match self {
            Direction::Up => Direction::Left,
            Direction::Left => Direction::Down,
            Direction::Down => Direction::Right,
            Direction::Right => Direction::Up,
        }
    }

    /// Returns the direction 90 degrees clockwise.
    fn turn_right(self) -> Self {
        match self {
            Direction::Up => Direction::Right,
            Direction::Right => Direction::Down,
            Direction::Down => Direction::Left,
            Direction::Left => Direction::Up,
        }
    }

    /// Returns the `(dx, dy)` step for one move in this direction.
    fn delta(self) -> (i8, i8) {
        match self {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        }
    }
}

// ---------------------------------------------------------------------
// --- Resources -------------------------------------------------------
// ---------------------------------------------------------------------

/// The snake: an ordered list of body cells (`body[0]` is the head), the
/// current length, facing direction, and one buffered turn.
#[derive(Resource, Debug)]
pub struct Snake {
    body: [(usize, usize); MAX_LEN],
    len: usize,
    dir: Direction,
    /// A pending direction change queued by the player, applied on the next
    /// move tick. Only one turn may be buffered at a time, which prevents a
    /// rapid double-tap from reversing 180 degrees into the neck.
    pending: Option<Direction>,
}

impl Default for Snake {
    fn default() -> Self {
        let mut body = [(0usize, 0usize); MAX_LEN];
        body[0] = INITIAL_HEAD;
        Self {
            body,
            len: 2,
            dir: Direction::Right,
            pending: None,
        }
    }
}

/// The single food pellet's position.
#[derive(Resource, Debug, Clone, Copy)]
pub struct Food {
    x: usize,
    y: usize,
}

/// Whether the round has ended (snake hit a wall or itself, or won).
#[derive(Resource, Debug, Default)]
pub struct GameState {
    dead: bool,
}

/// Paces snake movement at a fixed interval.
#[derive(Resource, Debug)]
pub struct MoveTimer(Timer);

impl MoveTimer {
    fn new() -> Self {
        Self(Timer::from_seconds(MOVE_SECS, TimerMode::Repeating))
    }
}

// ---------------------------------------------------------------------
// --- Plugin ----------------------------------------------------------
// ---------------------------------------------------------------------

/// Registers the snake's resources and systems.
pub struct SnakePlugin;

impl Plugin<crate::World> for SnakePlugin {
    fn build(&self, app: &mut App<crate::World>) {
        app.insert_resource(Snake::default());
        app.insert_resource(Food {
            x: INITIAL_FOOD.0,
            y: INITIAL_FOOD.1,
        });
        app.insert_resource(GameState::default());
        app.insert_resource(MoveTimer::new());

        app.add_system(Update, steer);
        app.add_system(Update, advance);
        app.add_system(Update, restart);
        app.add_system(Update, draw);
    }
}

// ---------------------------------------------------------------------
// --- Systems ---------------------------------------------------------
// ---------------------------------------------------------------------

/// Reads button presses and queues a single relative turn.
#[system]
fn steer(input: Res<ButtonInput<GameButton>>, mut snake: ResMut<Snake>) {
    if snake.pending.is_some() {
        return;
    }
    if input.just_pressed(GameButton::A) {
        snake.pending = Some(snake.dir.turn_left());
    } else if input.just_pressed(GameButton::B) {
        snake.pending = Some(snake.dir.turn_right());
    }
}

/// Moves the snake forward on each timer tick, handling eating, growth,
/// collision, and food relocation.
#[system]
fn advance(
    time: Res<Time>,
    mut timer: ResMut<MoveTimer>,
    mut state: ResMut<GameState>,
    mut snake: ResMut<Snake>,
    mut food: ResMut<Food>,
    mut entropy: ResMut<Entropy>,
) {
    if state.dead {
        return;
    }

    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }

    // Apply the buffered turn (if any).
    if let Some(d) = snake.pending.take() {
        snake.dir = d;
    }

    // Compute the new head position.
    let (hx, hy) = snake.body[0];
    let (dx, dy) = snake.dir.delta();
    let nx = hx as i8 + dx;
    let ny = hy as i8 + dy;

    // Wall collision.
    if nx < 0 || ny < 0 || nx >= GRID as i8 || ny >= GRID as i8 {
        state.dead = true;
        return;
    }
    let new_head = (nx as usize, ny as usize);

    // Is the head moving onto the food?
    let eating = food.x == new_head.0 && food.y == new_head.1;

    // Self-collision: when not eating, the tail vacates this tick, so we only
    // check body[1..len-1]. body[0] (old head) can never be hit because the
    // delta is always non-zero. With fewer than 3 segments there is no cell
    // that could cause a collision.
    if !eating && snake.len > 2 {
        let limit = snake.len - 1;
        if snake.body[1..limit].contains(&new_head) {
            state.dead = true;
            return;
        }
    }

    // Shift the body forward and place the new head. `copy_within` handles
    // overlapping ranges (memmove semantics).
    if eating {
        // Growing: shift everything one step toward the tail, keeping the old
        // tail (length increases by one).
        let len = snake.len;
        snake.body.copy_within(0..len, 1);
        snake.len += 1;
        snake.body[0] = new_head;

        // Win: the snake fills the entire grid.
        if snake.len >= MAX_LEN {
            state.dead = true;
            return;
        }

        relocate_food(&snake, &mut food, &mut entropy);
    } else {
        // Not growing: shift everything except the tail (tail is overwritten).
        let len = snake.len;
        snake.body.copy_within(0..len - 1, 1);
        snake.body[0] = new_head;
    }
}

/// Restarts the round when A is pressed after death.
#[system]
fn restart(
    input: Res<ButtonInput<GameButton>>,
    mut state: ResMut<GameState>,
    mut snake: ResMut<Snake>,
    mut timer: ResMut<MoveTimer>,
    mut food: ResMut<Food>,
    mut entropy: ResMut<Entropy>,
) {
    if !state.dead || !input.just_pressed(GameButton::A) {
        return;
    }

    *snake = Snake::default();
    state.dead = false;
    timer.0.reset();
    relocate_food(&snake, &mut food, &mut entropy);
}

/// Renders the snake, food, and game-over blink into the frame buffer.
#[system]
fn draw(
    mut frame: ResMut<FrameBuffer>,
    time: Res<Time>,
    state: Res<GameState>,
    snake: Res<Snake>,
    food: Res<Food>,
) {
    frame.clear();

    if state.dead {
        // Blink the full screen on/off every ~450 ms.
        let ms = time.elapsed().as_millis() as u64;
        if (ms / 450).is_multiple_of(2) {
            frame.fill_rect(0, 0, GRID, GRID, 255);
        }
        return;
    }

    // Food: gentle brightness pulse (triangle wave, ~1 s period).
    let ms = time.elapsed().as_millis() as u64 % 1000;
    let range = 255u32 - FOOD_MIN as u32;
    let food_brightness: u8 = if ms < 500 {
        (FOOD_MIN as u32 + ms as u32 * range / 200) as u8
    } else {
        (255 - (ms as u32 - 500) * range / 200) as u8
    };
    frame.set(food.x, food.y, food_brightness);

    // Snake: bright head, dim body.
    for (i, &(x, y)) in snake.body[0..snake.len].iter().enumerate() {
        let brightness = if i == 0 {
            HEAD_BRIGHTNESS
        } else {
            BODY_BRIGHTNESS
        };
        frame.set(x, y, brightness);
    }
}

// ---------------------------------------------------------------------
// --- Helpers ---------------------------------------------------------
// ---------------------------------------------------------------------

/// Places food on a random cell not occupied by the snake.
///
/// Scans all 25 cells, collects the free ones, and picks one at random using
/// the hardware-seeded [`Entropy`] resource.
fn relocate_food(snake: &Snake, food: &mut Food, entropy: &mut Entropy) {
    let mut free: [(usize, usize); MAX_LEN] = [(0, 0); MAX_LEN];
    let mut count = 0;
    for y in 0..GRID {
        for x in 0..GRID {
            let occupied = snake.body[0..snake.len].contains(&(x, y));
            if !occupied {
                free[count] = (x, y);
                count += 1;
            }
        }
    }
    if count > 0 {
        let (fx, fy) = free[entropy.next_below(count)];
        food.x = fx;
        food.y = fy;
    }
}
