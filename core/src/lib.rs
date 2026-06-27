#![no_std]

pub mod animation;
pub mod animation_empty;
pub mod animation_firework;
pub mod animation_leaves;
pub mod animation_matrix_rain;
pub mod animation_snake;
pub mod display;
pub mod hardware;

pub use animation::Animation;
pub use animation_empty::EmptyAnimation;
pub use animation_firework::Firework;
pub use animation_leaves::FallingLeaves;
pub use animation_matrix_rain::MatrixRain;
pub use animation_snake::Snake;
pub use display::{ObegraensadDisplay, BYTE_COUNT, DISPLAY_SIZE};
