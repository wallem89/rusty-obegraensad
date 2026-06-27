#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp-hal types"
)]
#![deny(clippy::large_stack_frames)]

use core::convert::Infallible;

use esp_hal::{
    clock::CpuClock,
    gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull},
    main,
    time::{Duration, Instant},
};
use fugit::MicrosDurationU32;
use obegraensad_core::{
    hardware::{AnimationSelect, DisplayDriver},
    Animation, EmptyAnimation, FallingLeaves, Firework, MatrixRain, ObegraensadDisplay, Snake,
    BYTE_COUNT, PWM_PHASE_COUNT,
};

esp_bootloader_esp_idf::esp_app_desc!();

const FRAME_AFTER_ANIMATION_SWITCH: MicrosDurationU32 = MicrosDurationU32::millis(30);
const PWM_PHASE_DURATION_US: u32 = 1_000;

struct Esp32S3Display<'d> {
    clock: Output<'d>,
    data: Output<'d>,
    latch: Output<'d>,
    not_enable: Output<'d>,
}

impl<'d> Esp32S3Display<'d> {
    fn new(clock: Output<'d>, data: Output<'d>, latch: Output<'d>, not_enable: Output<'d>) -> Self {
        Self {
            clock,
            data,
            latch,
            not_enable,
        }
    }

    fn write_byte(&mut self, byte: u8) -> Result<(), Infallible> {
        for bit in (0..8).rev() {
            if (byte & (1 << bit)) == 0 {
                self.data.set_low();
            } else {
                self.data.set_high();
            }

            self.clock.set_high();
            self.clock.set_low();
        }

        Ok(())
    }
}

impl DisplayDriver for Esp32S3Display<'_> {
    type Error = Infallible;

    fn write_frame(
        &mut self,
        display: &ObegraensadDisplay,
        pwm_phase: u8,
    ) -> Result<(), Self::Error> {
        let mut buffer = [0; BYTE_COUNT];
        display.to_output_buffer_for_pwm_phase(&mut buffer, pwm_phase);

        for byte in buffer {
            self.write_byte(byte)?;
        }

        Ok(())
    }

    fn latch(&mut self) -> Result<(), Self::Error> {
        self.latch.set_high();
        self.latch.set_low();
        Ok(())
    }

    fn set_enabled(&mut self, enabled: bool) -> Result<(), Self::Error> {
        if enabled {
            self.not_enable.set_low();
        } else {
            self.not_enable.set_high();
        }

        Ok(())
    }
}

struct ActiveLowButton<'d> {
    input: Input<'d>,
}

impl<'d> ActiveLowButton<'d> {
    fn new(input: Input<'d>) -> Self {
        Self { input }
    }
}

impl AnimationSelect for ActiveLowButton<'_> {
    type Error = Infallible;

    fn is_selected(&mut self) -> Result<bool, Self::Error> {
        Ok(self.input.is_low())
    }
}

fn delay_micros(us: u32) {
    let delay_start = Instant::now();
    while delay_start.elapsed() < Duration::from_micros(us as u64) {}
}

fn frame_duration_us(duration: MicrosDurationU32) -> u32 {
    duration.to_micros().max(1)
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[allow(
    clippy::large_stack_frames,
    reason = "the firmware keeps the display buffer and animation state on the stack"
)]
#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let output_config = OutputConfig::default();
    let mut display_driver = Esp32S3Display::new(
        Output::new(peripherals.GPIO12, Level::Low, output_config),
        Output::new(peripherals.GPIO11, Level::Low, output_config),
        Output::new(peripherals.GPIO10, Level::Low, output_config),
        Output::new(peripherals.GPIO9, Level::High, output_config),
    );
    let mut animation_select = ActiveLowButton::new(Input::new(
        peripherals.GPIO0,
        InputConfig::default().with_pull(Pull::Up),
    ));

    let mut display = ObegraensadDisplay::new();
    let mut animation_leaves = FallingLeaves::new();
    let mut animation_firework = Firework::new();
    let mut animation_matrix_rain = MatrixRain::new();
    let mut animation_snake = Snake::new();
    let mut animation_empty = EmptyAnimation::new();
    const ANIMATION_COUNT: usize = 5;
    let animations: [&mut dyn Animation; ANIMATION_COUNT] = [
        &mut animation_leaves,
        &mut animation_firework,
        &mut animation_matrix_rain,
        &mut animation_snake,
        &mut animation_empty,
    ];
    let mut current_animation_index = 0;
    let mut current_frame_remaining_us =
        frame_duration_us(animations[current_animation_index].render_frame(&mut display));
    let mut pwm_phase = 0;

    display_driver.set_enabled(true).unwrap();

    loop {
        if animation_select.is_selected().unwrap() {
            while animation_select.is_selected().unwrap() {
                delay_micros(20_000);
            }

            current_animation_index += 1;
            if current_animation_index >= ANIMATION_COUNT {
                current_animation_index = 0;
            }

            current_frame_remaining_us = frame_duration_us(FRAME_AFTER_ANIMATION_SWITCH);
            animations[current_animation_index].render_frame(&mut display);
            pwm_phase = 0;
        }

        display_driver.write_frame(&display, pwm_phase).unwrap();
        display_driver.latch().unwrap();

        let phase_duration_us = current_frame_remaining_us.min(PWM_PHASE_DURATION_US);
        delay_micros(phase_duration_us);
        current_frame_remaining_us -= phase_duration_us;

        pwm_phase += 1;
        if pwm_phase >= PWM_PHASE_COUNT {
            pwm_phase = 0;
        }

        if current_frame_remaining_us == 0 {
            current_frame_remaining_us =
                frame_duration_us(animations[current_animation_index].render_frame(&mut display));
        }
    }
}
