use crate::animation::Animation;
use crate::display::{ObegraensadDisplay, BIT_COUNT, DISPLAY_SIZE};

use fugit::MicrosDurationU32;
use rand::{RngCore, SeedableRng};
use rand_xoshiro::Xoshiro128StarStar;

#[derive(Clone, Copy)]
struct Column {
    y: i8,
    speed: u8,
    length: u8,
    active: bool,
}

impl Column {
    const fn new() -> Self {
        Self {
            y: 0,
            speed: 1,
            length: 3,
            active: false,
        }
    }
}

pub struct MatrixRain {
    rng: Xoshiro128StarStar,
    pixels: [u8; BIT_COUNT],
    columns: [Column; DISPLAY_SIZE],
}

impl MatrixRain {
    const FRAME_DELAY_MS: u32 = 50;
    const MAX_TRAIL_LENGTH: u8 = 8;
    const FADE_AMOUNT: u8 = 15;

    pub fn new() -> Self {
        let mut this = Self {
            rng: Xoshiro128StarStar::seed_from_u64(0xA17A_1CE5_0B3E_903D),
            pixels: [0; BIT_COUNT],
            columns: [Column::new(); DISPLAY_SIZE],
        };
        this.reset_columns();
        this
    }

    fn random_u8(&mut self, upper_bound: u8) -> u8 {
        (self.rng.next_u32() % upper_bound as u32) as u8
    }

    fn reset_columns(&mut self) {
        for i in 0..DISPLAY_SIZE {
            self.columns[i] = Column {
                y: -(self.random_u8(DISPLAY_SIZE as u8) as i8),
                speed: 1 + self.random_u8(3),
                length: 3 + self.random_u8(Self::MAX_TRAIL_LENGTH - 3),
                active: self.random_u8(100) > 30,
            };
        }
    }

    fn set_brightness(&mut self, x: u8, y: i8, brightness: u8) {
        if x >= DISPLAY_SIZE as u8 || y < 0 || y >= DISPLAY_SIZE as i8 {
            return;
        }

        self.pixels[y as usize * DISPLAY_SIZE + x as usize] = brightness;
    }

    fn draw(&self, display: &mut ObegraensadDisplay) {
        display.clear();

        for y in 0..DISPLAY_SIZE as u8 {
            for x in 0..DISPLAY_SIZE as u8 {
                if self.pixels[y as usize * DISPLAY_SIZE + x as usize] > 0 {
                    display.set_pixel(x, y);
                }
            }
        }
    }
}

impl Default for MatrixRain {
    fn default() -> Self {
        Self::new()
    }
}

impl Animation for MatrixRain {
    fn render_frame(&mut self, display: &mut ObegraensadDisplay) -> MicrosDurationU32 {
        for pixel in self.pixels.iter_mut() {
            *pixel = pixel.saturating_sub(Self::FADE_AMOUNT);
        }

        for i in 0..DISPLAY_SIZE {
            if !self.columns[i].active {
                if self.random_u8(100) > 95 {
                    self.columns[i].active = true;
                    self.columns[i].y = -(self.random_u8(5) as i8);
                    self.columns[i].speed = 1 + self.random_u8(3);
                    self.columns[i].length = 3 + self.random_u8(Self::MAX_TRAIL_LENGTH - 3);
                }
                continue;
            }

            let column = self.columns[i];
            self.set_brightness(i as u8, column.y, 255);

            for j in 1..column.length {
                let trail_y = column.y - j as i8;
                let brightness = 255 - (j * 255 / column.length);
                self.set_brightness(i as u8, trail_y, brightness);
            }

            self.columns[i].y += column.speed as i8;

            if self.columns[i].y - self.columns[i].length as i8 >= DISPLAY_SIZE as i8 {
                self.columns[i].active = false;
            }
        }

        self.draw(display);
        MicrosDurationU32::millis(Self::FRAME_DELAY_MS)
    }
}
