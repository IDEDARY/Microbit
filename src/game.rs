use crate::*;
 
 
pub const OBSTACLES: [[u8; 5]; 11] = [
    [1,0,1,0,1],
    [0,1,1,1,0],
    [1,1,1,0,0],
    [0,0,1,1,1],
    [1,1,0,1,1],
 
    [1,1,1,1,0],
    [1,1,1,0,1],
    [1,1,0,1,1],
    [1,0,1,1,1],
    [0,1,1,1,1],
    [0,1,0,1,0],
];
 
pub struct Game {
    game_over: bool,
 
}