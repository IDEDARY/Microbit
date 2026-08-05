//! Button input, modelled after Bevy's `bevy_input::ButtonInput`.
//!
//! The game reads [`ButtonInput<GameButton>`] as a resource; the plugin samples
//! the physical pins and performs edge detection so that `just_pressed` /
//! `just_released` behave exactly like on a desktop.

use tiny_ecs::prelude::*;

use crate::device::{read_button_pin, Device};

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
/// Mirrors the method surface of `bevy_input::ButtonInput`. Two small arrays
/// would be cheaper, but with only two buttons the representation stays tiny
/// and readable.
#[derive(Debug, Clone, Resource)]
pub struct ButtonInput<T: 'static> {
    /// Buttons currently held down.
    pressed: [bool; 2],
    /// Buttons that transitioned to pressed since the last frame.
    just_pressed: [bool; 2],
    /// Buttons that transitioned to released since the last frame.
    just_released: [bool; 2],
    _marker: core::marker::PhantomData<T>,
}

// Manual `Default` so `T` need not be `Default` (only `GameButton` ever goes
// here, and it has no `Default` variant).
impl<T: 'static> Default for ButtonInput<T> {
    fn default() -> Self {
        Self {
            pressed: [false, false],
            just_pressed: [false, false],
            just_released: [false, false],
            _marker: core::marker::PhantomData,
        }
    }
}

impl<T: ButtonKey + 'static> ButtonInput<T> {
    /// Records a button as newly pressed.
    pub fn press(&mut self, input: T) {
        let i = input.index();
        self.just_pressed[i] = true;
        self.pressed[i] = true;
    }

    /// Records a button as released.
    pub fn release(&mut self, input: T) {
        let i = input.index();
        self.just_released[i] = true;
        self.pressed[i] = false;
    }

    /// Resets the per-frame edge sets, ready for the next poll of hardware.
    pub fn clear(&mut self) {
        self.just_pressed = [false, false];
        self.just_released = [false, false];
    }

    /// Returns `true` while the button is held down.
    pub fn pressed(&self, input: T) -> bool {
        self.pressed[input.index()]
    }

    /// Returns `true` on the exact frame the button first became pressed.
    pub fn just_pressed(&self, input: T) -> bool {
        self.just_pressed[input.index()]
    }

    /// Returns `true` on the exact frame the button was released.
    pub fn just_released(&self, input: T) -> bool {
        self.just_released[input.index()]
    }
}

/// Maps a two-button enum onto a `0..2` index used by [`ButtonInput`].
pub trait ButtonKey {
    /// Returns the dense index of this button in `0..2`.
    fn index(self) -> usize;
}

impl ButtonKey for GameButton {
    fn index(self) -> usize {
        match self {
            GameButton::A => 0,
            GameButton::B => 1,
        }
    }
}

/// Polls the physical buttons and updates [`ButtonInput`] every frame.
pub struct MicrobitInputPlugin;
impl<W: WorldApi> Plugin<W> for MicrobitInputPlugin where W: HasResource<Device> + HasResource<ButtonInput<GameButton>> {
    fn build(&self, app: &mut App<W>) {
        app.insert_resource(ButtonInput::<GameButton>::default());
        app.add_system(W::Label::pre_update(), read_buttons);
    }
}

/// Samples both pins, performs edge detection, and refreshes the input state.
#[system]
fn read_buttons(mut device: ResMut<Device>, mut input: ResMut<ButtonInput<GameButton>>) {
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