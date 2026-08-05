//! Button input, modelled after Bevy's `bevy_input::ButtonInput`.
//!
//! The game reads [`ButtonInput<GameButton>`] as a resource; the plugin samples
//! the physical pins and performs edge detection so that `just_pressed` /
//! `just_released` behave exactly like on a desktop.

use alloc::collections::BTreeSet;

use bevy_app::{App, Plugin, PreUpdate};
use bevy_ecs::prelude::{ResMut, Resource};

use crate::device::{Device, read_button_pin};

/// The physical buttons exposed to the game.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum GameButton {
    /// Left button (button A).
    A,
    /// Right button (button B).
    B,
}

/// Tracks the current and just-pressed/just-released state of every button.
///
/// Mirrors the method surface of `bevy_input::ButtonInput`.
#[derive(Debug, Clone, Resource)]
pub struct ButtonInput<T> {
    /// Buttons currently held down.
    pressed: BTreeSet<T>,
    /// Buttons that transitioned to pressed since the last frame.
    just_pressed: BTreeSet<T>,
    /// Buttons that transitioned to released since the last frame.
    just_released: BTreeSet<T>,
}
impl<T: Ord> Default for ButtonInput<T> {
    /// Creates an empty input state.
    fn default() -> Self {
        Self {
            pressed: BTreeSet::new(),
            just_pressed: BTreeSet::new(),
            just_released: BTreeSet::new(),
        }
    }
}
impl<T: Ord + Copy> ButtonInput<T> {
    /// Records a button as newly pressed.
    pub fn press(&mut self, input: T) {
        self.just_pressed.insert(input);
        self.pressed.insert(input);
    }

    /// Records a button as released.
    pub fn release(&mut self, input: T) {
        self.just_released.insert(input);
        self.pressed.remove(&input);
    }

    /// Resets the per-frame edge sets, ready for the next poll of hardware.
    pub fn clear(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
    }

    /// Returns `true` while the button is held down.
    pub fn pressed(&self, input: T) -> bool {
        self.pressed.contains(&input)
    }

    /// Returns `true` on the exact frame the button first became pressed.
    pub fn just_pressed(&self, input: T) -> bool {
        self.just_pressed.contains(&input)
    }

    /// Returns `true` on the exact frame the button was released.
    pub fn just_released(&self, input: T) -> bool {
        self.just_released.contains(&input)
    }
}

/// Polls the physical buttons and updates [`ButtonInput`] every frame.
pub struct MicrobitInputPlugin;
impl Plugin for MicrobitInputPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ButtonInput::<GameButton>::default());
        app.add_systems(PreUpdate, read_buttons);
    }
}

/// Samples both pins, performs edge detection, and refreshes the input state.
fn read_buttons(
    mut device: ResMut<Device>,
    mut input: ResMut<ButtonInput<GameButton>>,
) {
    // Forget last frame's edges, then re-derive them from the current pins.
    input.clear();

    let a = read_button_pin(&mut device.buttons.button_a);
    let b = read_button_pin(&mut device.buttons.button_b);

    if a {
        input.press(GameButton::A);
    } else {
        input.release(GameButton::A);
    }

    if b {
        input.press(GameButton::B);
    } else {
        input.release(GameButton::B);
    }
}
