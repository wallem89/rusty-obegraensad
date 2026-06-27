use crate::animation::Animation;
use crate::display::{ObegraensadDisplay, DISPLAY_SIZE};

use fugit::MicrosDurationU32;
use rand::{RngCore, SeedableRng};
use rand_xoshiro::Xoshiro128StarStar;

enum FireworkState {
    Rocket,
    ExplosionGrow,
    ExplosionFade,
    Wait,
}

pub struct Firework {
    rng: Xoshiro128StarStar,
    rocket_x: u8,
    rocket_y: i8,
    explosion_x: u8,
    explosion_y: u8,
    max_radius: u8,
    radius: u8,
    fade_step: u8,
    wait_frame_count: u8,
    state: FireworkState,
}

impl Firework {
    const EXPLOSION_DELAY_MS: u32 = 60;
    const FADE_DELAY_MS: u32 = 24;
    const ROCKET_DELAY_MS: u32 = 60;
    const EXPLOSION_DURATION_MS: u32 = 500;
    const FADE_STEPS: u8 = 32;

    pub fn new() -> Self {
        Self {
            rng: Xoshiro128StarStar::seed_from_u64(0xF17E_2A70_91D5_4C33),
            rocket_x: 0,
            rocket_y: DISPLAY_SIZE as i8,
            explosion_x: 0,
            explosion_y: 0,
            max_radius: 3,
            radius: 1,
            fade_step: 0,
            wait_frame_count: 0,
            state: FireworkState::Rocket,
        }
    }

    fn random_u8(&mut self, upper_bound: u8) -> u8 {
        (self.rng.next_u32() % upper_bound as u32) as u8
    }

    fn launch_next_rocket(&mut self) {
        self.rocket_y = DISPLAY_SIZE as i8;
        self.rocket_x = self.random_u8(DISPLAY_SIZE as u8);
        self.state = FireworkState::Rocket;
    }

    fn start_explosion(&mut self) {
        self.explosion_x = self.rocket_x;
        self.explosion_y = self.rocket_y.max(0) as u8;
        self.max_radius = 3 + self.random_u8(3);
        self.radius = 1;
        self.fade_step = 0;
        self.state = FireworkState::ExplosionGrow;
    }

    fn draw_explosion(&self, display: &mut ObegraensadDisplay, radius: u8, brightness: u8) {
        let radius_squared = (radius as i16) * (radius as i16);

        for y in 0..DISPLAY_SIZE as u8 {
            for x in 0..DISPLAY_SIZE as u8 {
                let dx = x as i16 - self.explosion_x as i16;
                let dy = y as i16 - self.explosion_y as i16;
                if dx * dx + dy * dy <= radius_squared {
                    display.set_pixel_brightness(x, y, brightness);
                }
            }
        }
    }
}

impl Default for Firework {
    fn default() -> Self {
        Self::new()
    }
}

impl Animation for Firework {
    fn render_frame(&mut self, display: &mut ObegraensadDisplay) -> MicrosDurationU32 {
        display.clear();

        match self.state {
            FireworkState::Rocket => {
                if self.rocket_y >= 0 {
                    display.set_pixel(self.rocket_x, self.rocket_y as u8);
                }

                self.rocket_y -= 1;
                if self.rocket_y < self.random_u8(8) as i8 {
                    self.start_explosion();
                }

                MicrosDurationU32::millis(Self::ROCKET_DELAY_MS)
            }
            FireworkState::ExplosionGrow => {
                self.draw_explosion(display, self.radius, u8::MAX);
                self.radius += 1;

                if self.radius > self.max_radius {
                    self.state = FireworkState::ExplosionFade;
                }

                MicrosDurationU32::millis(Self::EXPLOSION_DELAY_MS)
            }
            FireworkState::ExplosionFade => {
                let brightness = u8::MAX.saturating_sub(self.fade_step.saturating_mul(8));
                self.draw_explosion(display, self.max_radius, brightness);
                self.fade_step += 1;

                if self.fade_step >= Self::FADE_STEPS {
                    self.wait_frame_count = 0;
                    self.state = FireworkState::Wait;
                }

                MicrosDurationU32::millis(Self::FADE_DELAY_MS)
            }
            FireworkState::Wait => {
                self.wait_frame_count += 1;

                if self.wait_frame_count
                    >= (Self::EXPLOSION_DURATION_MS / Self::ROCKET_DELAY_MS) as u8
                {
                    self.launch_next_rocket();
                }

                MicrosDurationU32::millis(Self::ROCKET_DELAY_MS)
            }
        }
    }
}
