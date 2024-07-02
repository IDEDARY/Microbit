#![allow(unused_imports)]
#![deny(unsafe_code)]
#![no_main]
#![no_std]

use cortex_m_rt::entry;
use embedded_hal::digital::{InputPin, OutputPin};
use microbit::{gpio::DisplayPins, hal::gpio::Level};
use rtt_target::{rtt_init_print, rprintln};
use panic_rtt_target as _;
use microbit::board::Board;
use microbit::hal::timer::Timer;
use embedded_hal::delay::DelayNs;
use microbit::hal::gpio::{Output, PushPull, Pin};
use panic_rtt_target as _;


// Define the trait
pub trait LedMatrixControl {
    fn set_leds(&mut self, leds: [[bool; 5]; 5]);
}

// Implement the trait for DisplayPins
impl LedMatrixControl for DisplayPins {
    fn set_leds(&mut self, leds: [[bool; 5]; 5]) {
        // Turn off all rows before setting columns
        self.row1.set_low().unwrap();
        self.row2.set_low().unwrap();
        self.row3.set_low().unwrap();

        // Columns are active low, so setting them high turns off the LEDs
        self.col1.set_high().unwrap();
        self.col2.set_high().unwrap();
        self.col3.set_high().unwrap();
        self.col4.set_high().unwrap();
        self.col5.set_high().unwrap();
        self.col6.set_high().unwrap();
        self.col7.set_high().unwrap();
        self.col8.set_high().unwrap();
        self.col9.set_high().unwrap();

        for (row_index, row) in leds.iter().enumerate() {
            // Activate the correct row pin
            match row_index {
                0 => self.row1.set_high().unwrap(),
                1 => self.row1.set_high().unwrap(),
                2 => self.row2.set_high().unwrap(),
                3 => self.row2.set_high().unwrap(),
                4 => self.row3.set_high().unwrap(),
                _ => unreachable!(),
            }

            for (col_index, &led_on) in row.iter().enumerate() {
                match (row_index, col_index) {
                    (0 | 1, 0) => if led_on { self.col1.set_low().unwrap() } else { self.col1.set_high().unwrap() },
                    (0 | 1, 1) => if led_on { self.col2.set_low().unwrap() } else { self.col2.set_high().unwrap() },
                    (0 | 1, 2) => if led_on { self.col3.set_low().unwrap() } else { self.col3.set_high().unwrap() },
                    (0 | 1, 3) => if led_on { self.col4.set_low().unwrap() } else { self.col4.set_high().unwrap() },
                    (0 | 1, 4) => if led_on { self.col5.set_low().unwrap() } else { self.col5.set_high().unwrap() },
                    (2 | 3, 0) => if led_on { self.col6.set_low().unwrap() } else { self.col6.set_high().unwrap() },
                    (2 | 3, 1) => if led_on { self.col7.set_low().unwrap() } else { self.col7.set_high().unwrap() },
                    (2 | 3, 2) => if led_on { self.col8.set_low().unwrap() } else { self.col8.set_high().unwrap() },
                    (2 | 3, 3) => if led_on { self.col9.set_low().unwrap() } else { self.col9.set_high().unwrap() },
                    (4, 0) => if led_on { self.col1.set_low().unwrap() } else { self.col1.set_high().unwrap() },
                    (4, 1) => if led_on { self.col2.set_low().unwrap() } else { self.col2.set_high().unwrap() },
                    (4, 2) => if led_on { self.col3.set_low().unwrap() } else { self.col3.set_high().unwrap() },
                    (4, 3) => if led_on { self.col4.set_low().unwrap() } else { self.col4.set_high().unwrap() },
                    (4, 4) => if led_on { self.col5.set_low().unwrap() } else { self.col5.set_high().unwrap() },
                    _ => {},
                }
            }

            // Deactivate the row pin after setting the columns
            match row_index {
                0 => self.row1.set_low().unwrap(),
                1 => self.row1.set_low().unwrap(),
                2 => self.row2.set_low().unwrap(),
                3 => self.row2.set_low().unwrap(),
                4 => self.row3.set_low().unwrap(),
                _ => unreachable!(),
            }
        }
    }
}


#[entry]
fn main() -> ! {
    rtt_init_print!();
    let mut board = Board::take().unwrap();
    let mut timer = Timer::new(board.TIMER0);

    //let mut led_matrix = LedMatrix::new(board);
    
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

        // Example LED pattern
        /* let leds = [
            [true, false, true, false, true],
            [false, true, false, true, false],
            [true, false, true, false, true],
            [false, true, false, true, false],
            [true, false, true, false, true],
        ];

        // Update the LED matrix
        board.display_pins.set_leds(leds); */

        // Add a small delay
        //timer.delay_ms(1000u32);


        
        if let Ok(false) = board.buttons.button_a.is_high() {
            row1.set_high().unwrap();
            col1.set_high().unwrap();
        } else {
            row1.set_low().unwrap();
            col1.set_low().unwrap();

        }

        /* if let Ok(false) = board.buttons.button_b.is_high() {
            row2.set_high().unwrap();
            col2.set_high().unwrap();
        } else {
            row2.set_low().unwrap();
            col2.set_low().unwrap();
        } */
    }
}