#![allow(unused_imports)]
#![allow(dead_code)]
#![no_main]
#![no_std]
 
pub use heapless::*;
use core::cell::RefCell;
use core::fmt::Write;
 
pub use cortex_m_rt::entry;
pub use embedded_hal::digital::{InputPin, OutputPin};
pub use microbit::{gpio::DisplayPins, hal::gpio::Level};
pub use rtt_target::{rtt_init_print, rprintln};
pub use panic_rtt_target as _;
pub use microbit::board::Board;
pub use microbit::hal::timer::Timer;
pub use embedded_hal::delay::DelayNs;
pub use microbit::hal::gpio::{Output, PushPull, Pin};
use tinyrand::Seeded;
pub use tinyrand::{Rand, RandRange, StdRand};
use microbit::hal::{pac, rng::Rng};
 
mod device;
mod game;
use {device::*, game::*};
 
#[entry]
fn main() -> ! {
    rtt_init_print!();
    let mut device = Device::new();
 
    let mut game_over = false;
 
    let mut score: usize = 0;
 
    let mut player_x = 2;
    let mut player_move_cooldown = 0;
 
    let mut debris: [Option<(usize, usize)>; 25] = [None; 25];
 
    let debris_spawn_alarm = RefCell::new(Alarm::new(0));
    let debris_fall_alarm = RefCell::new(Alarm::new(0));
 
    let _ = device.alarms.push(&debris_spawn_alarm);
    let _ = device.alarms.push(&debris_fall_alarm);
 
    loop {
 
        // START NEW FRAME
        if device.new_frame() {
 
            // GAME RESET
            if device.button_a.just_pressed() && game_over {
                game_over = false;
                device.text = None;
                score = 0;
 
                player_x = 2;
                player_move_cooldown = 0;
 
                debris = [None; 25];
                debris_spawn_alarm.borrow_mut().set(0);
                debris_fall_alarm.borrow_mut().set(0);
            }
 
            // GAME END
            if game_over {
                if let None = device.text {
                    let mut t = String::new();
                    write!(t, "{}", score).unwrap();
                    device.text = Some(Text::from(t))
                }
            }
 
            // GAME
            if !game_over {
 
                // DRAW PLAYER
                device.canvas.matrix[4][player_x] = true;
 
                // DRAW DEBRIS
                for rock in &mut debris {
                    if let Some((x, y)) = rock {
                        device.canvas.matrix[*y][*x] = true;
                    }
                }
 
                // DEBRIS LOGIC
                if debris_fall_alarm.borrow_mut().wait_for(100) {
                    let mut was_despawned = false;
                    for rock in &mut debris {
                        if let Some((x, y)) = rock {
                            if *y == 4 {
                                *rock = None;
                                was_despawned = true;
                                continue;
                            }
                            *rock = Some((*x, *y + 1));
                        }
                    }
 
                    if was_despawned {
                        score += 1;
                    }
                }
 
                // DEBRIS SPAWN
                if debris_spawn_alarm.borrow_mut().wait_for(500) {
                    let mut obstacle = OBSTACLES[device.rand.next_range(0..OBSTACLES.len())];
                    for rock in &mut debris {
                        if let None = rock {
                            for (i, obs) in &mut obstacle.iter_mut().enumerate() {
                                if *obs == 1 {
                                    *obs = 0;
                                    *rock = Some((i, 0));
                                    break;
                                }
                            }
                        }
                    }
                }
 
 
                // PLAYER LOGIC
                if player_move_cooldown != 0 { player_move_cooldown -= 1 }
 
                if device.button_a.just_pressed() {
                    if player_x != 0 && player_move_cooldown == 0 { player_x -= 1; player_move_cooldown = 5; }
                }
 
                if device.button_b.just_pressed() {
                    if player_x != 4 && player_move_cooldown == 0 { player_x += 1; player_move_cooldown = 5; }
                }
                
                // COLLISION CHECK
                for rock in &mut debris {
                    if let Some((x, y)) = rock {
                        if *x == player_x && *y == 4 {
                            game_over = true;
                        }
                    }
                }
            }
 
            // END FRAME
            device.end_frame();
        }
 
        // END TICK
        device.end_tick();
    }
}