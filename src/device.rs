use core::str::FromStr;

use microbit::board::Buttons;
use pac::TIMER0;
use tinyrand::Wyrand;

use crate::*;

pub const LED_LAYOUT: [[(usize, usize); 5]; 5] = [
    [(0, 0), (1, 3), (0, 1), (1, 4), (0, 2)],
    [(2, 3), (2, 4), (2, 5), (2, 6), (2, 7)],
    [(1, 1), (0, 8), (1, 2), (2, 8), (1, 0)],
    [(0, 7), (0, 6), (0, 5), (0, 4), (0, 3)],
    [(2, 2), (1, 6), (2, 0), (1, 5), (2, 1)],
];

pub const NUM: [[[u8; 3]; 5]; 10] = [
    [   // 0
        [0,1,0],
        [1,0,1],
        [1,0,1],
        [1,0,1],
        [0,1,0],
    ],
    [   // 1
        [0,1,0],
        [1,1,0],
        [0,1,0],
        [0,1,0],
        [0,1,0],
    ],
    [   // 2
        [1,1,0],
        [0,0,1],
        [0,1,0],
        [1,0,0],
        [1,1,1],
    ],
    [   // 3
        [1,1,0],
        [0,0,1],
        [1,1,0],
        [0,0,1],
        [1,1,0],
    ],
    [   // 4
        [1,0,0],
        [1,0,0],
        [1,1,1],
        [0,1,0],
        [0,1,0],
    ],
    [   // 5
        [1,1,1],
        [1,0,0],
        [1,1,0],
        [0,0,1],
        [1,1,0],
    ],
    [   // 6
        [0,1,1],
        [1,0,0],
        [1,1,0],
        [1,0,1],
        [0,1,0],
    ],
    [   // 7
        [1,1,1],
        [0,0,1],
        [0,1,0],
        [1,0,0],
        [1,0,0],
    ],
    [   // 8
        [0,1,0],
        [1,0,1],
        [0,1,0],
        [1,0,1],
        [0,1,0],
    ],
    [   // 9
        [0,1,0],
        [1,0,1],
        [0,1,1],
        [0,0,1],
        [0,1,0],
    ],
];


#[derive(Clone, Debug, PartialEq)]
pub struct Text<const N: usize> {
    pub alarm: RefCell<Alarm>,
    scroll: usize,
    pub string: String<N>,
}
impl <const N: usize> Text<N> {
    pub fn new(text: &str) -> Self {
        Text {
            alarm: RefCell::new(Alarm::new(0)),
            scroll: 0,
            string: String::from_str(text).unwrap_or_default(),
        }
    }
    pub fn from(text: String<N>) -> Self {
        Text {
            alarm: RefCell::new(Alarm::new(0)),
            scroll: 0,
            string: text,
        }
    }
    pub fn set(&mut self, text: &str) {
        self.string = String::from_str(text).unwrap_or_default();
    }
    pub fn draw_text(&mut self, canvas: &mut Canvas) {

        if self.alarm.borrow_mut().wait_for(100) {
            if self.scroll <= 4 * self.string.chars().count() + 3 { self.scroll += 1; } else { self.scroll = 0 }
        }

        let mut bitmap: Vec<[[u8; 3]; 5], 16> = Vec::new();
        for ch in self.string.chars().into_iter() {
            let _ = bitmap.push(match ch {
                '0' => NUM[0],
                '1' => NUM[1],
                '2' => NUM[2],
                '3' => NUM[3],
                '4' => NUM[4],
                '5' => NUM[5],
                '6' => NUM[6],
                '7' => NUM[7],
                '8' => NUM[8],
                '9' => NUM[9],
                _ => todo!(),
            });
        }

        for (i, symbol) in bitmap.iter().enumerate() {
            canvas.paste(4 -(self.scroll as isize) + 4 * i as isize, 0, *symbol);
        }
    }  
}

pub struct Device<'a> {
    raw_buttons: Buttons,
    raw_display_pins: DisplayPins,

    /// Canvas
    pub canvas: Canvas,
    /// Tick counter
    pub tick: usize,
    /// Timer
    pub timer: Timer<TIMER0>,
    /// Random
    pub rand: Wyrand,
    /// Button A
    pub button_a: Button,
    /// Button B
    pub button_b: Button,
    /// If this is Some, then the text will get rendered instead
    pub text: Option<Text<16>>,
    /// Alarm pointer stack, max 16.
    pub alarms: Vec<&'a RefCell<Alarm>, 16>,
}
impl <'a> Device<'a> {
    fn start_frame(&mut self) {
        // START FRAME
        self.canvas.set([[false;5];5]);

        // GET INPUT
        self.button_a.current_value = self.raw_buttons.button_a.is_low().unwrap_or(false);
        self.button_b.current_value = self.raw_buttons.button_b.is_low().unwrap_or(false);
    }

    /// Create new instance
    pub fn new() -> Self {
        let board = Board::take().unwrap();
        Device {
            raw_display_pins: board.display_pins,
            raw_buttons: board.buttons,

            canvas: Canvas::new(),
            tick: 0,
            timer: Timer::new(board.TIMER0),
            rand: StdRand::seed(Rng::new(board.RNG).random_u64()),
            button_a: Button::new(),
            button_b: Button::new(),
            text: None,
            alarms: Vec::new(),
        }
    }
    /// Starts and checks for a new frame
    pub fn new_frame(&mut self) -> bool {
        if self.canvas.row == 0 {
            self.start_frame();
            true
        } else {
            false
        }
    }
    /// Code that needs to run on the end of a frame
    pub fn end_frame(&mut self) {
        // RENDER TEXT IF APPLICABLE
        if let Some(text) = &mut self.text {
            text.draw_text(&mut self.canvas)
        }
        // LOG PREVIOUS INPUT
        self.button_a.previous_value = self.button_a.current_value;
        self.button_b.previous_value = self.button_b.current_value;
    }
    /// Code that needs to run on the end of a tick
    pub fn end_tick(&mut self) {
        // RENDER CANVAS
        self.canvas.clear(&mut self.raw_display_pins);
        self.canvas.partial_render(&mut self.raw_display_pins);

        // INNER CLOCK
        self.timer.delay_ms(1u32);
        self.tick = self.tick.overflowing_add(1).0;

        // Tick text scroll alarm
        if let Some(text) = &mut self.text {
            if let Ok(mut aa) = text.alarm.try_borrow_mut() { aa.tick(); }
        }
        // Tick all registered alarms
        for a in self.alarms.iter() {
            if let Ok(mut aa) = a.try_borrow_mut() { aa.tick(); }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Canvas {
    pub matrix: [[bool; 5]; 5],
    row: usize,
}
impl Canvas {
    /// Render canvas to display pins. Needs to be called 3 times in different ticks to work.
    fn partial_render(&mut self, pins: &mut DisplayPins) {
        match self.row {
            0 => { pins.row1.set_high().unwrap() },
            1 => { pins.row2.set_high().unwrap() },
            2 => { pins.row3.set_high().unwrap() },
            _ => unreachable!(),
        }
        for (y, row) in self.matrix.iter().enumerate() {
            for (x, value) in row.iter().enumerate() {
                if LED_LAYOUT[y][x].0 == self.row && *value == true {
                    match LED_LAYOUT[y][x].1 {
                        0 => { pins.col1.set_low().unwrap() },
                        1 => { pins.col2.set_low().unwrap() },
                        2 => { pins.col3.set_low().unwrap() },
                        3 => { pins.col4.set_low().unwrap() },
                        4 => { pins.col5.set_low().unwrap() },
                        5 => { pins.col6.set_low().unwrap() },
                        6 => { pins.col7.set_low().unwrap() },
                        7 => { pins.col8.set_low().unwrap() },
                        8 => { pins.col9.set_low().unwrap() },
                        _ => unreachable!(),
                    }
                }
            }
        }
        if self.row == 2 { self.row = 0 } else { self.row += 1 }
    }
    /// Clear the display pins.
    fn clear(&mut self, pins: &mut DisplayPins) {
        pins.row1.set_low().unwrap();
        pins.row2.set_low().unwrap();
        pins.row3.set_low().unwrap();
        pins.col1.set_high().unwrap();
        pins.col2.set_high().unwrap();
        pins.col3.set_high().unwrap();
        pins.col4.set_high().unwrap();
        pins.col5.set_high().unwrap();
        pins.col6.set_high().unwrap();
        pins.col7.set_high().unwrap();
        pins.col8.set_high().unwrap();
        pins.col9.set_high().unwrap();
    }
    
    /// Creates new instance
    pub fn new() -> Self {
        Canvas {
            matrix: [[false; 5]; 5],
            row: 0,
        }
    }
        /// Override the canvas with given bitmap
    pub fn set(&mut self, mat: [[bool; 5]; 5]) {
        self.matrix = mat;
    }
        /// Paste bitmap of any size. Will change only true fields.
    pub fn paste<const N: usize, const M: usize>(&mut self, x: isize, y: isize, mut mat: [[u8; N]; M]) {
        for (yy, row) in mat.iter_mut().enumerate() {
            for (xx, value) in row.iter_mut().enumerate() {
                if (xx as isize + x >= 0 && xx as isize + x <= 4) && (yy as isize + y >= 0 && yy as isize + y <= 4) && *value != 0 {
                    self.matrix[(yy as isize + y) as usize][(xx as isize + x) as usize] = true;
                }
            }
        }
    }
    /// Paste bitmap of any size. Will override all affected fields.
    pub fn set_paste<const N: usize, const M: usize>(&mut self, x: isize, y: isize, mut mat: [[u8; N]; M]) {
        for (yy, row) in mat.iter_mut().enumerate() {
            for (xx, value) in row.iter_mut().enumerate() {
                if (xx as isize + x >= 0 && xx as isize + x <= 4) && (yy as isize + y >= 0 && yy as isize + y <= 4) {
                    self.matrix[(yy as isize + y) as usize][(xx as isize + x) as usize] = *value != 0;
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Button {
    previous_value: bool,
    current_value: bool,
}
impl Button {
    /// Create new instance
    pub fn new() -> Self {
        Button {
            previous_value: false,
            current_value: false,
        }
    }
    /// Checks if the button is pressed
    pub fn pressed(&self) -> bool {
        self.current_value
    }
    /// Checks if the button was just pressed
    pub fn just_pressed(&self) -> bool {
        self.current_value && self.current_value != self.previous_value
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Alarm {
    time_remain: usize,
}
impl Alarm {
    /// The alarm tick
    fn tick(&mut self) {
        if self.time_remain != 0 { self.time_remain -= 1 }
    }

    /// Create new instance
    pub fn new(init: usize) -> Self {
        Alarm { time_remain: init }
    }
    /// Set remaining time to given value
    pub fn set(&mut self, time: usize) {
        self.time_remain = time;
    }
    /// Checks if remaining time is zero and if yes set it to value given
    pub fn wait_for(&mut self, time: usize) -> bool {
        if self.time_remain == 0 {
            self.time_remain = time;
            true
        } else {
            false
        }
    }
}
