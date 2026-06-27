use crate::animation::Animation;
use crate::display::{ObegraensadDisplay, BIT_COUNT, DISPLAY_SIZE};

use fugit::MicrosDurationU32;
use rand::{RngCore, SeedableRng};
use rand_xoshiro::Xoshiro128StarStar;

#[derive(Clone, Copy, Eq, PartialEq)]
enum GameState {
    Running,
    End,
    DeathAnimation,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum Direction {
    Unset,
    Up,
    Right,
    Down,
    Left,
}

pub struct Snake {
    rng: Xoshiro128StarStar,
    game_state: GameState,
    last_direction: Direction,
    position: [u8; BIT_COUNT],
    length: usize,
    dot: u8,
    animation_step: u8,
}

impl Snake {
    const SNAKE_DELAY_MS: u32 = 100;
    const BLINK_SHORT_MS: u32 = 200;
    const BLINK_LONG_MS: u32 = 500;

    pub fn new() -> Self {
        let mut this = Self {
            rng: Xoshiro128StarStar::seed_from_u64(0x5A9E_51A6_D07F_EED5),
            game_state: GameState::End,
            last_direction: Direction::Unset,
            position: [0; BIT_COUNT],
            length: 0,
            dot: 0,
            animation_step: 0,
        };
        this.init_game();
        this
    }

    fn random_pixel(&mut self) -> u8 {
        (self.rng.next_u32() % BIT_COUNT as u32) as u8
    }

    fn init_game(&mut self) {
        self.position[0] = (BIT_COUNT - DISPLAY_SIZE) as u8;
        self.position[1] = (BIT_COUNT - DISPLAY_SIZE + 1) as u8;
        self.position[2] = (BIT_COUNT - DISPLAY_SIZE + 2) as u8;
        self.length = 3;
        self.last_direction = Direction::Unset;
        self.new_dot();
    }

    fn new_dot(&mut self) {
        loop {
            let dot = self.random_pixel();
            if !self.position[..self.length].contains(&dot) {
                self.dot = dot;
                self.game_state = GameState::Running;
                return;
            }
        }
    }

    fn is_snake_position(&self, index: u8) -> bool {
        self.position[..self.length].contains(&index)
    }

    fn index_to_xy(index: u8) -> (u8, u8) {
        (index % DISPLAY_SIZE as u8, index / DISPLAY_SIZE as u8)
    }

    fn draw_index(display: &mut ObegraensadDisplay, index: u8) {
        let (x, y) = Self::index_to_xy(index);
        display.set_pixel(x, y);
    }

    fn draw_game(&self, display: &mut ObegraensadDisplay, draw_snake: bool, draw_dot: bool) {
        display.clear();

        if draw_snake {
            for index in self.position[..self.length].iter() {
                Self::draw_index(display, *index);
            }
        }

        if draw_dot {
            Self::draw_index(display, self.dot);
        }
    }

    fn move_snake(&mut self, new_position: u8) {
        if self.length < BIT_COUNT {
            self.position[self.length] = new_position;
            self.length += 1;
        }

        if new_position == self.dot {
            self.new_dot();
        } else if self.length > 0 {
            self.position.copy_within(1..self.length, 0);
            self.length -= 1;
        }
    }

    fn end(&mut self) {
        self.game_state = GameState::DeathAnimation;
        self.animation_step = 0;
    }

    fn find_direction(&mut self) {
        let snake_head = self.position[self.length - 1];
        let snake_col = snake_head % DISPLAY_SIZE as u8;
        let snake_row = snake_head / DISPLAY_SIZE as u8;
        let dot_col = self.dot % DISPLAY_SIZE as u8;
        let dot_row = self.dot / DISPLAY_SIZE as u8;

        let mut up = snake_row > 0;
        let mut down = snake_row < DISPLAY_SIZE as u8 - 1;
        let mut left = snake_col > 0;
        let mut right = snake_col < DISPLAY_SIZE as u8 - 1;

        let up_pos = snake_head.wrapping_sub(DISPLAY_SIZE as u8);
        let down_pos = snake_head.wrapping_add(DISPLAY_SIZE as u8);
        let left_pos = snake_head.wrapping_sub(1);
        let right_pos = snake_head.wrapping_add(1);

        if up && self.is_snake_position(up_pos) {
            up = false;
        }
        if down && self.is_snake_position(down_pos) {
            down = false;
        }
        if left && self.is_snake_position(left_pos) {
            left = false;
        }
        if right && self.is_snake_position(right_pos) {
            right = false;
        }

        let mut best_up = if up { 1 } else { 0 };
        let mut best_down = if down { 1 } else { 0 };
        let mut best_left = if left { 1 } else { 0 };
        let mut best_right = if right { 1 } else { 0 };

        if snake_col == dot_col {
            best_left = 0;
            best_right = 0;
        } else if snake_col > dot_col {
            if best_left != 0 {
                best_left = snake_col - dot_col;
            }
            best_right = 0;
        } else {
            if best_right != 0 {
                best_right = dot_col - snake_col;
            }
            best_left = 0;
        }

        if snake_row == dot_row {
            best_up = 0;
            best_down = 0;
        } else if snake_row > dot_row {
            if best_up != 0 {
                best_up = snake_row - dot_row;
            }
            best_down = 0;
        } else {
            if best_down != 0 {
                best_down = dot_row - snake_row;
            }
            best_up = 0;
        }

        if self.last_direction == Direction::Up && best_up != 0 {
            self.move_with_direction(up_pos, Direction::Up);
        } else if self.last_direction == Direction::Right && best_right != 0 {
            self.move_with_direction(right_pos, Direction::Right);
        } else if self.last_direction == Direction::Down && best_down != 0 {
            self.move_with_direction(down_pos, Direction::Down);
        } else if self.last_direction == Direction::Left && best_left != 0 {
            self.move_with_direction(left_pos, Direction::Left);
        } else if best_up == 0 && best_right == 0 && best_down == 0 && best_left == 0 {
            if up {
                self.move_with_direction(up_pos, Direction::Up);
            } else if down {
                self.move_with_direction(down_pos, Direction::Down);
            } else if left {
                self.move_with_direction(left_pos, Direction::Left);
            } else if right {
                self.move_with_direction(right_pos, Direction::Right);
            } else {
                self.end();
            }
        } else if best_up > best_right && best_up > best_down && best_up > best_left {
            self.move_with_direction(up_pos, Direction::Up);
        } else if best_right > best_down && best_right > best_left && best_right > best_up {
            self.move_with_direction(right_pos, Direction::Right);
        } else if best_down > best_left && best_down > best_up && best_down > best_right {
            self.move_with_direction(down_pos, Direction::Down);
        } else if best_left > best_up && best_left > best_right && best_left > best_down {
            self.move_with_direction(left_pos, Direction::Left);
        } else if best_up != 0 {
            self.move_with_direction(up_pos, Direction::Up);
        } else if best_down != 0 {
            self.move_with_direction(down_pos, Direction::Down);
        } else if best_left != 0 {
            self.move_with_direction(left_pos, Direction::Left);
        } else {
            self.move_with_direction(right_pos, Direction::Right);
        }
    }

    fn move_with_direction(&mut self, new_position: u8, direction: Direction) {
        self.move_snake(new_position);
        self.last_direction = direction;
    }

    fn render_death_animation(&mut self, display: &mut ObegraensadDisplay) -> MicrosDurationU32 {
        match self.animation_step {
            0 | 2 | 4 => {
                self.draw_game(display, false, true);
                self.animation_step += 1;
                MicrosDurationU32::millis(Self::BLINK_SHORT_MS)
            }
            1 | 3 | 5 => {
                self.draw_game(display, true, true);
                self.animation_step += 1;
                MicrosDurationU32::millis(Self::BLINK_SHORT_MS)
            }
            6 => {
                self.draw_game(display, true, true);
                self.animation_step += 1;
                MicrosDurationU32::millis(Self::BLINK_LONG_MS)
            }
            7 => {
                if self.length > 0 {
                    self.position.copy_within(1..self.length, 0);
                    self.length -= 1;
                    self.draw_game(display, true, true);
                    MicrosDurationU32::millis(Self::BLINK_SHORT_MS)
                } else {
                    self.animation_step += 1;
                    self.draw_game(display, false, true);
                    MicrosDurationU32::millis(Self::BLINK_SHORT_MS)
                }
            }
            8 => {
                self.draw_game(display, false, false);
                self.animation_step += 1;
                MicrosDurationU32::millis(Self::BLINK_SHORT_MS)
            }
            _ => {
                self.draw_game(display, false, false);
                self.game_state = GameState::End;
                self.animation_step = 0;
                MicrosDurationU32::millis(Self::BLINK_LONG_MS)
            }
        }
    }
}

impl Default for Snake {
    fn default() -> Self {
        Self::new()
    }
}

impl Animation for Snake {
    fn render_frame(&mut self, display: &mut ObegraensadDisplay) -> MicrosDurationU32 {
        match self.game_state {
            GameState::Running => {
                self.find_direction();
                self.draw_game(display, true, true);
                MicrosDurationU32::millis(Self::SNAKE_DELAY_MS)
            }
            GameState::DeathAnimation => self.render_death_animation(display),
            GameState::End => {
                self.init_game();
                self.draw_game(display, true, true);
                MicrosDurationU32::millis(Self::SNAKE_DELAY_MS)
            }
        }
    }
}
