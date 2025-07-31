# HUB75 Embassy Driver

[![Crates.io](https://img.shields.io/crates/v/hub75-embassy.svg)](https://crates.io/crates/hub75-embassy)
[![Documentation](https://docs.rs/hub75-embassy/badge.svg)](https://docs.rs/hub75-embassy)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](https://github.com/yourusername/hub75-embassy)

A high-performance, embassy-compatible driver for HUB75 RGB LED matrix displays with embedded-graphics support.

## Features

- 🚀 **Full HUB75 Protocol Implementation** - Complete support for the HUB75 interface
- ⚡ **Embassy-rs Integration** - Native async/await support with embassy-time
- 🎨 **Embedded-graphics Support** - DrawTarget implementation for easy graphics
- 🌈 **Binary Code Modulation (BCM)** - High color depth with efficient timing
- 📐 **Flexible Panel Sizes** - Support for 32x16, 64x32, 64x64, and custom sizes
- 🎬 **Animation Framework** - Built-in animations with multiple effects
- 🔄 **Double Buffering** - Smooth updates without flicker
- 💾 **Memory Efficient** - Optimized for embedded systems
- 🔧 **Configurable** - Adjustable refresh rates, brightness, and color depth

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
hub75-embassy = "0.1"
embassy-executor = "0.6"
embassy-time = "0.3"
embedded-graphics = "0.8"
```

### Basic Usage

```rust
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Rectangle, PrimitiveStyleBuilder},
};
use hub75_embassy::{Hub75Display, Hub75Pins};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Configure your pins (example for RP2040)
    let pins = Hub75Pins::new_64x32(
        r1_pin, g1_pin, b1_pin,  // Upper RGB
        r2_pin, g2_pin, b2_pin,  // Lower RGB  
        a_pin, b_pin, c_pin, d_pin,  // Address
        clk_pin, lat_pin, oe_pin,    // Control
    );
    
    let mut display = Hub75Display::<_, 64, 32, 6>::new(pins).unwrap();
    display.set_double_buffering(true);
    
    // Draw a red rectangle
    Rectangle::new(Point::new(10, 10), Size::new(20, 12))
        .into_styled(PrimitiveStyleBuilder::new()
            .fill_color(Rgb565::RED)
            .build())
        .draw(&mut display)
        .unwrap();
    
    // Start the refresh task
    spawner.spawn(refresh_task(display)).unwrap();
}

#[embassy_executor::task]
async fn refresh_task(mut display: Hub75Display</* ... */>) {
    display.refresh_task().await;
}
```

## Supported Panel Sizes

The driver supports common HUB75 panel sizes with convenient type aliases:

```rust
// 32x16 panels (3 address pins)
type Display32x16 = Hub75_32x16<YourPinType, 6>;

// 64x32 panels (4 address pins) 
type Display64x32 = Hub75_64x32<YourPinType, 6>;

// 64x64 panels (5 address pins)
type Display64x64 = Hub75_64x64<YourPinType, 6>;

// Custom sizes
type CustomDisplay = Hub75Display<YourPinType, WIDTH, HEIGHT, COLOR_BITS>;
```

## Pin Configuration

### Standard HUB75 Pinout

| Pin | Description | Required |
|-----|-------------|----------|
| R1, G1, B1 | Upper half RGB data | Yes |
| R2, G2, B2 | Lower half RGB data | Yes |
| A, B, C | Address pins (3 bits = 8 rows) | Yes |
| D | Address pin (4 bits = 16 rows) | 64x32+ |
| E | Address pin (5 bits = 32 rows) | 64x64+ |
| CLK | Clock signal | Yes |
| LAT | Latch signal | Yes |
| OE | Output Enable (active low) | Yes |

### Pin Configuration Examples

```rust
// For 32x16 displays (3 address pins)
let pins = Hub75Pins::new_32x16(
    r1, g1, b1, r2, g2, b2,  // RGB pins
    a, b, c,                  // Address pins
    clk, lat, oe             // Control pins
);

// For 64x32 displays (4 address pins)  
let pins = Hub75Pins::new_64x32(
    r1, g1, b1, r2, g2, b2,  // RGB pins
    a, b, c, d,              // Address pins
    clk, lat, oe             // Control pins
);

// For 64x64 displays (5 address pins)
let pins = Hub75Pins::new_64x64(
    r1, g1, b1, r2, g2, b2,  // RGB pins
    a, b, c, d, e,           // Address pins
    clk, lat, oe             // Control pins
);
```

## Advanced Features

### Animation System

The driver includes a powerful animation system inspired by the microbit patterns:

```rust
use hub75_embassy::animation::{Animation, AnimationData, AnimationEffect};

// Create frames for animation
let frames = [frame1, frame2, frame3];

// Create sliding animation
let mut animation = Animation::new(
    AnimationData::Frames(&frames),
    AnimationEffect::Slide,
    Duration::from_secs(2),
).unwrap();

// Run animation loop
loop {
    match animation.next(Instant::now()) {
        AnimationState::Apply(frame) => {
            display.back_buffer().copy_from(&frame);
            display.swap_buffers();
        }
        AnimationState::Wait => Timer::after(Duration::from_millis(10)).await,
        AnimationState::Done => break,
    }
}
```

### Available Animation Effects

- **None** - Direct frame display
- **Slide** - Frames slide in from the right
- **Fade** - Frames fade in and out
- **Wipe** - Frames are revealed column by column

### Double Buffering

Enable smooth updates without flicker:

```rust
display.set_double_buffering(true);

// Draw to back buffer
Rectangle::new(Point::new(0, 0), Size::new(32, 16))
    .into_styled(PrimitiveStyleBuilder::new().fill_color(Rgb565::RED).build())
    .draw(display.back_buffer())
    .unwrap();

// Atomically swap buffers
display.swap_buffers();
```

### Brightness Control

```rust
use hub75_embassy::Brightness;

// Set brightness (0-255)
display.set_brightness(Brightness::new(128)); // 50%

// Adjust brightness
let mut brightness = display.brightness();
brightness = brightness + 10;  // Increase
brightness = brightness - 5;   // Decrease (with saturation)
display.set_brightness(brightness);
```

### Performance Tuning

```rust
// Adjust refresh rate (higher = smoother, more CPU usage)
display.set_refresh_interval(Duration::from_micros(50)); // 20kHz

// Color depth affects quality vs performance
// Higher bits = better colors, more CPU usage
type HighQuality = Hub75Display<Pin, 64, 32, 8>;  // 8-bit color
type Balanced = Hub75Display<Pin, 64, 32, 6>;     // 6-bit color  
type Performance = Hub75Display<Pin, 64, 32, 4>;  // 4-bit color
```

## Embassy Task Management

### Recommended Task Structure

```rust
#[embassy_executor::task]
async fn display_refresh_task(display: &'static SharedDisplay) {
    // High priority - runs continuously for flicker-free display
    loop {
        let mut display = display.lock().await;
        display.render_frame().await.ok();
        embassy_futures::yield_now().await;
    }
}

#[embassy_executor::task] 
async fn graphics_task(display: &'static SharedDisplay) {
    // Medium priority - updates graphics periodically
    loop {
        {
            let mut display = display.lock().await;
            // Update graphics...
            display.swap_buffers();
        }
        Timer::after(Duration::from_millis(16)).await; // ~60 FPS
    }
}
```

### Sharing Display Between Tasks

Use `embassy-sync` for safe sharing:

```rust
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, mutex::Mutex};

type SharedDisplay = Mutex<ThreadModeRawMutex, Hub75Display</* ... */>>;
static DISPLAY: SharedDisplay = Mutex::new(/* ... */);
```

## Platform Support

This driver works with any platform supported by embassy-rs:

- **RP2040** (Raspberry Pi Pico)
- **STM32** (All families)
- **nRF52/nRF53** (Nordic)
- **ESP32** (via esp-hal)
- **And more...**

## Examples

See the `examples/` directory for complete working examples:

- [`basic_usage.rs`](examples/basic_usage.rs) - Simple graphics with embedded-graphics
- [`animations.rs`](examples/animations.rs) - Animation effects demonstration
- [`rp2040_example.rs`](examples/rp2040_example.rs) - Complete RP2040 implementation

## Performance Characteristics

### Refresh Rates

| Panel Size | Color Depth | Typical Refresh Rate | CPU Usage |
|------------|-------------|---------------------|-----------|
| 32x16 | 4-bit | 1000+ Hz | Low |
| 32x16 | 6-bit | 400+ Hz | Medium |
| 64x32 | 4-bit | 500+ Hz | Medium |
| 64x32 | 6-bit | 200+ Hz | High |
| 64x64 | 4-bit | 250+ Hz | High |

*Actual performance depends on MCU speed and other system load*

### Memory Usage

| Panel Size | Color Depth | Frame Buffer Size |
|------------|-------------|-------------------|
| 32x16 | 6-bit | ~1.5 KB |
| 64x32 | 6-bit | ~6 KB |
| 64x64 | 6-bit | ~24 KB |

*Double buffering doubles memory usage*

## Troubleshooting

### Common Issues

**Display flickers:**

- Increase refresh rate: `display.set_refresh_interval(Duration::from_micros(50))`
- Reduce color depth to 4-bit
- Ensure refresh task runs at high priority

**Colors look wrong:**

- Check RGB pin connections (R1/R2, G1/G2, B1/B2)
- Verify color bit depth matches your needs
- Try different gamma correction values

**Display shows garbage:**

- Verify address pin connections (A, B, C, D, E)
- Check that panel size matches configuration
- Ensure proper ground connections

**Performance issues:**

- Reduce color depth
- Lower refresh rate
- Use single buffering
- Optimize graphics drawing code

### Debug Features

Enable debug logging with the `defmt` feature:

```toml
[dependencies]
hub75-embassy = { version = "0.1", features = ["defmt"] }
```

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Setup

```bash
git clone https://github.com/yourusername/hub75-embassy
cd hub75-embassy
cargo test
cargo doc --open
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Acknowledgments

This driver was inspired by:

- [hub75-rs](https://github.com/david-sawatzke/hub75-rs) - Original HUB75 driver
- [hub75-remastered](https://github.com/david-sawatzke/hub75-remastered) - Improved architecture
- [microbit-bsp](https://github.com/nrf-rs/microbit) - LED matrix patterns
- [embassy-rs](https://github.com/embassy-rs/embassy) - Async embedded framework
- [embedded-graphics](https://github.com/embedded-graphics/embedded-graphics) - Graphics library

## Related Projects

- [smart-leds-matrix](https://github.com/smart-leds-rs/smart-leds-matrix) - For WS2812-based LED matrices
- [embedded-graphics](https://github.com/embedded-graphics/embedded-graphics) - 2D graphics library
- [embassy-rs](https://github.com/embassy-rs/embassy) - Async embedded framework
