# Rusty OBEGRÄNSAD
Display custom animations on IKEA's OBEGRÄNSAD using Rust on the Raspberry Pi Pico or ESP32-S3-DevKitC-1.

## Crates
This repository is split into a Cargo workspace with separate portable and board-specific crates:

- `core`: `no_std` display buffer, OBEGRÄNSAD pixel mapping, animation trait, built-in animations, and hardware boundary traits.
- `boards/pico`: Raspberry Pi Pico firmware. This crate owns RP2040 startup, GPIO, SPI, DMA, timer interrupts, and the concrete animation loop.
- `boards/esp32-s3-devkit-c-1`: ESP32-S3-DevKitC-1 firmware using `esp-hal`. This crate owns ESP32-S3 startup and GPIO output to the OBEGRÄNSAD LED-driver chain.

The boundary between portable code and board code is intentionally small:

- `animation::Animation` renders the next frame into `ObegraensadDisplay` and returns its frame duration.
- `hardware::DisplayDriver` describes the board-specific transport for writing, latching, and enabling the physical display.
- `hardware::AnimationSelect` describes board-specific input used to switch animations.

## Interfacing with OBEGRÄNSAD
OBEGRÄNSAD consists of 16 daisy-chained SCT2024 16 bit serial-in/parallel-out constant-current LED drivers.
After de-soldering the on-board microcontroller, the board that you choose can be connected to the Clock, Data In, Latch, and inverted Enable inputs of the SCT2024 chain.
These inputs as well as +5V and GND can readily be accessed at the bottom of the OBEGRÄNSAD PCB that contained the original microcontroller.

### Raspberry Pi specific
For the Rasppery Pi Pico you need to convert the 3.3V outputs to 5V outputs.
In order to interface with the 5V CMOS inputs of the SCT2024, a level shifter is required to translate the 3.3V outputs of the Pico to 5V.
I assembled a helper board for level shifting using an SN74AHCT125, wired up as shown in the following schematic:
![Schematic of level-shifter board](schematic/level-shifter.png)

### Other remarks
The timings for the SCT2024 are such that the Clock and Data In lines can be driven by SPI.
To latch the transmitted data to the LEDs, a short positive pulse on the Latch line is required.

Note that the 16 LEDs driven by one SCT2024 are laid out in a circular pattern around each chip such that it is non-trivial to index the LEDs of the display.
While one can derive an algorithm to compute the index of an LED, I found the algorithm to be not very readable and only used it to compute a look-up table to index the LEDs/pixels.

## Implementing custom animations
A custom animation for the display should implement the `obegraensad_core::Animation` trait with its only method `render_frame`.
The return value of the `render_frame` method indicates for how long this frame should be displayed.
When implementing this method, you typically want to use `display.clear()` to erase the current contents of the display and then draw your frame using `display.set_pixel(x, y)` or `display.set_pixel_brightness(x, y, brightness)`.
Brightness is represented as `0..=255` in `ObegraensadDisplay`.
The current board crates render brightness on the binary SCT2024 LED-driver chain using 16-phase temporal PWM.

The core crate currently includes `FallingLeaves`, `Firework`, `MatrixRain`, `Snake`, and `EmptyAnimation`.
To show your custom animation on the display, add it to `core` or another crate and add a mutable reference to an instance of the animation to the `animations` array in `boards/pico/src/main.rs`.
The button of the display can be used to cycle through the different animations.

## Generating a UF2 binary
### Raspberry Pi Pico
Ensure that Rust is up-to-date, target support for `thumbv6m-none-eabi` is provided, and elf2uf2-rs is installed:
```
rustup self update
rustup update stable
rustup target add thumbv6m-none-eabi
cargo install elf2uf2-rs
```

Execute `cargo run -p obegraensad-pico --release` to generate the UF2 binary at `target/thumbv6m-none-eabi/release/obegraensad-pico.uf2`.

### ESP32-S3-DevKitC-1
The ESP32-S3 crate targets `xtensa-esp32s3-none-elf` and is intended to be built with the `esp` Rust toolchain:
```
cd boards/esp32-s3-devkit-c-1
cargo +esp check
cargo +esp run --release
```
For release builds and flashing, ensure the ESP Xtensa GCC toolchain is on `PATH`; with `espup`, source the generated export file before running Cargo.

The ESP32-S3 implementation bit-bangs the display using this pin mapping:

- Latch: `GPIO10` on esp32 -> **CLA** on OBEGRÄNSAD PCB
- Clock: `GPIO12` on esp32 -> **CLK** on OBEGRÄNSAD PCB
- Data: `GPIO11` on esp32 -> **IN** on OBEGRÄNSAD PCB
- Inverted enable: `GPIO9` on esp32 ->  **EN** on OBEGRÄNSAD PCB
- Animation-select button: `GPIO0` on esp32 -> button in OBEGRÄNSAD case

The crate-local Cargo config uses `espflash flash --monitor` as its runner.

## git setup

### GitHub noreply email
```
git config user.name "a-johanson"
git config user.email "a-johanson@users.noreply.github.com"
```

### GitHub tokens
```
git remote add origin https://a-johanson:<TOKEN>@github.com/a-johanson/rusty-obegraensad.git
git push -u origin master
```
