#![deny(unsafe_code)]
#![no_main]
#![no_std]

use cortex_m_rt::entry;
use embedded_hal::digital::{InputPin, OutputPin};
use microbit::hal::gpio::Level;
use rtt_target::{rtt_init_print, rprintln};
use panic_rtt_target as _;
use microbit::board::Board;
use microbit::hal::timer::Timer;
use embedded_hal::delay::DelayNs;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let mut board = Board::take().unwrap();
    let mut timer = Timer::new(board.TIMER0);
    
    board.display_pins.row1.set_high().unwrap();

    let mut row1 = board.display_pins.row1.into_push_pull_output(Level::Low);
    let mut row2 = board.display_pins.row2.into_push_pull_output(Level::Low);
    let mut row3 = board.display_pins.row3.into_push_pull_output(Level::Low);

    // Configure columns as output pins
    let mut col1 = board.display_pins.col1.into_push_pull_output(Level::High);
    let mut col2 = board.display_pins.col2.into_push_pull_output(Level::High);
    let mut col3 = board.display_pins.col3.into_push_pull_output(Level::High);
    let mut col4 = board.display_pins.col4.into_push_pull_output(Level::High);
    let mut col5 = board.display_pins.col5.into_push_pull_output(Level::High);
    let mut col6 = board.display_pins.col6.into_push_pull_output(Level::High);
    let mut col7 = board.display_pins.col7.into_push_pull_output(Level::High);
    let mut col8 = board.display_pins.col8.into_push_pull_output(Level::High);
    let mut col9 = board.display_pins.col9.into_push_pull_output(Level::High);

    loop {
        //timer.delay_ms(1000u32);
        //rprintln!("1000 ms passed - Hello EdHouse");
        
        if let Ok(false) = board.buttons.button_a.is_high() {
            row1.set_high().unwrap();
            row2.set_high().unwrap();
            row3.set_high().unwrap();
            
            // Set all columns low to enable all LEDs
            col1.set_low().unwrap();
            col2.set_low().unwrap();
            col3.set_low().unwrap();
            col4.set_low().unwrap();
            col5.set_low().unwrap();
            col6.set_low().unwrap();
            col7.set_low().unwrap();
            col8.set_low().unwrap();
            col9.set_low().unwrap();
        } else {
            row1.set_low().unwrap();
            row2.set_low().unwrap();
            row3.set_low().unwrap();
            
            col1.set_high().unwrap();
            col2.set_high().unwrap();
            col3.set_high().unwrap();
            col4.set_high().unwrap();
            col5.set_high().unwrap();
            col6.set_high().unwrap();
            col7.set_high().unwrap();
            col8.set_high().unwrap();
            col9.set_high().unwrap();
        }

        if let Ok(false) = board.buttons.button_b.is_high() {
            row2.set_high().unwrap();
            col5.set_low().unwrap();
        } else {
            row2.set_low().unwrap();
            col5.set_high().unwrap();
        }
    }
}