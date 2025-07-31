# HUB75 Driver

[![Crates.io](https://img.shields.io/crates/v/hub75.svg)](https://crates.io/crates/hub75)
[![Documentation](https://docs.rs/hub75/badge.svg)](https://docs.rs/hub75)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](https://github.com/yourusername/hub75)

A high-performance, generic async driver for HUB75 RGB LED matrix displays with embedded-graphics support.

## Features

- 🚀 **Full HUB75 Protocol Implementation** - Complete support for the HUB75 interface
- ⚡ **Generic Async Support** - Works with Embassy, RTIC, tokio, and any async runtime
- 🎨 **Embedded-graphics Support** - DrawTarget implementation for easy graphics
- 🌈 **Binary Code Modulation (BCM)** - High color depth with efficient timing
- 📐 **Flexible Panel Sizes** - Support for 32x16, 64x32, 64x64, and custom sizes
- 🎬 **Animation Framework** - Built-in animations with multiple effects
- 🔄 **Double Buffering** - Smooth updates without flicker
- 💾 **Memory Efficient** - Optimized for embedded systems
- 🔧 **Configurable** - Adjustable refresh rates, brightness, and color depth
- 🔌 **Runtime Agnostic** - No Embassy dependencies, works with any DelayNs provider

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
hub75 = "0.1"
embassy-executor = "0.6"
embassy-time = "0.3"
embedded-graphics = "0.8"
```

### Basic Usage

```rust
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer, Delay};
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Rectangle, PrimitiveStyleBuilder},
};
use hub75::{Hub75Display, Hub75Pins};

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
    let mut delay = Delay; // Embassy delay provider
    loop {
        if let Err(e) = display.render_frame(&mut delay).await {
            // Handle error
        }
    }
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
| ---------- | ------------------------------ | -------- |
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
use hub75::animation::{Animation, AnimationData, AnimationEffect};
use embassy_time::Delay;

// Create frames for animation
let frames = [frame1, frame2, frame3];

// Create sliding animation
let mut animation = Animation::new(
    AnimationData::Frames(&frames),
    AnimationEffect::Slide,
    Duration::from_secs(2),
).unwrap();

// Run animation loop with DelayNs provider
let mut delay = Delay;
loop {
    match animation.next(Instant::now()) {
        AnimationState::Apply(frame) => {
            display.back_buffer().copy_from(&frame);
            display.swap_buffers();
        }
        AnimationState::Wait => animation.wait(&mut delay).await,
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
use hub75::Brightness;

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

## Task Management

### Recommended Task Structure

```rust
use embassy_time::Delay;

#[embassy_executor::task]
async fn display_refresh_task(display: &'static SharedDisplay) {
    let mut delay = Delay;
    // High priority - runs continuously for flicker-free display
    loop {
        let mut display = display.lock().await;
        display.render_frame(&mut delay).await.ok();
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

### Alternative: Combined Task Pattern

For simpler applications, combine display and graphics in one task:

```rust
#[embassy_executor::task]
async fn combined_display_task(mut display: Hub75Display</* ... */>) {
    let mut delay = Delay;
    display.set_double_buffering(true);

    loop {
        // Update graphics
        display.clear();
        // ... draw operations ...
        display.swap_buffers();

        // Render frame
        display.render_frame(&mut delay).await.ok();

        Timer::after(Duration::from_millis(16)).await;
    }
}
```

## Architecture

This driver uses a **generic async pattern** that works with any runtime providing `DelayNs`:

### Runtime Compatibility

- **Embassy** - Use `embassy_time::Delay`
- **RTIC** - Use any DelayNs-compatible timer
- **Tokio** - Use `tokio::time::sleep` with DelayNs wrapper
- **Custom runtimes** - Implement DelayNs trait for your timer

### DelayNs Pattern

All display methods that require timing accept a `&mut impl DelayNs` parameter:

```rust
use embedded_hal_async::delay::DelayNs;

// Render a frame with your delay provider
let mut delay = embassy_time::Delay;
display.render_frame(&mut delay).await?;

// Animation updates
animation.update(&mut delay).await?;

// Custom DelayNs implementation
struct MyDelay;
impl DelayNs for MyDelay {
    async fn delay_ns(&mut self, ns: u32) {
        // Your timing implementation
    }
}
```

### Key Architectural Changes

- **No Embassy Dependencies** - Removed `embassy-time` and `embassy-sync` dependencies
- **Generic Async Support** - Works with any async runtime via DelayNs trait
- **Flexible Timing** - Bring your own delay provider for maximum compatibility
- **Maintained API** - Same high-level API with DelayNs parameter additions

### Migration from Embassy-Specific Version

If you're upgrading from an earlier Embassy-specific version:

**Before (Embassy-specific):**

```rust
// Old API - no DelayNs parameter
display.render_frame().await?;
display.refresh_task().await; // Built-in task
```

**After (Generic async):**

```rust
// New API - requires DelayNs provider
let mut delay = embassy_time::Delay;
display.render_frame(&mut delay).await?;

// Manual refresh loop instead of built-in task
loop {
    display.render_frame(&mut delay).await?;
}
```

**Dependencies Update:**

```toml
# Remove from Cargo.toml (now optional)
# embassy-time = "0.3"
# embassy-sync = "0.7"

# Add if using Embassy
embassy-time = "0.3"  # For Delay implementation
```

## Platform Support

This driver works with any embedded platform that provides GPIO and DelayNs:

- **RP2040** (Raspberry Pi Pico/Pico W)
- **STM32** (All families)
- **nRF52/nRF53** (Nordic)
- **ESP32** (via esp-hal)
- **And more...**

## Examples

The examples are organized by platform in the `examples/` directory:

### RP2040 Examples (`examples/rp/`)

- [`basic_display.rs`](examples/rp/src/bin/basic_display.rs) - Simple shapes and text
- [`animated_effects.rs`](examples/rp/src/bin/animated_effects.rs) - Animation demonstrations
- [`comprehensive_demo.rs`](examples/rp/src/bin/comprehensive_demo.rs) - Full feature showcase
- [`performance_test.rs`](examples/rp/src/bin/performance_test.rs) - Performance benchmarking
- [`pico_w_demo.rs`](examples/rp/src/bin/pico_w_demo.rs) - Pico W specific features

### nRF Examples (`examples/nrf/`)

- [`basic_display.rs`](examples/nrf/src/bin/basic_display.rs) - Simple graphics demo
- [`graphics_demo.rs`](examples/nrf/src/bin/graphics_demo.rs) - Advanced graphics
- [`text_display.rs`](examples/nrf/src/bin/text_display.rs) - Text rendering
- [`animated_patterns.rs`](examples/nrf/src/bin/animated_patterns.rs) - Pattern animations

### Running Examples

```bash
# RP2040 examples
cd examples/rp
cargo run --bin basic_display

# nRF examples
cd examples/nrf
cargo run --bin basic_display
```

## Performance Characteristics

### Refresh Rates

| Panel Size | Color Depth | Typical Refresh Rate | CPU Usage |
| ---------- | ----------- | -------------------- | --------- |
| 32x16 | 4-bit | 1000+ Hz | Low |
| 32x16 | 6-bit | 400+ Hz | Medium |
| 64x32 | 4-bit | 500+ Hz | Medium |
| 64x32 | 6-bit | 200+ Hz | High |
| 64x64 | 4-bit | 250+ Hz | High |

_Actual performance depends on MCU speed and other system load_

### Performance Optimization Tips

Based on insights from [hub75-rs](https://github.com/david-sawatzke/hub75-rs):

**For Higher Color Depth (6-8 bits):**

- Use faster MCUs (>100MHz recommended for 8-bit color)
- Implement hardware timer-controlled OE timing
- Consider DMA-based data output (platform-specific)
- Pre-render frame data when possible

**For Higher Refresh Rates:**

- Reduce color depth (4-bit allows >1kHz refresh)
- Use single buffering to reduce memory bandwidth
- Optimize the refresh task priority
- Consider using dedicated hardware peripherals (PIO, I2S, etc.)

**Memory vs Performance Trade-offs:**

- **Single buffering**: Lower memory, potential tearing
- **Double buffering**: Smooth updates, 2x memory usage
- **Pre-rendered buffers**: Highest performance, highest memory usage

### Memory Usage

| Panel Size | Color Depth | Frame Buffer Size |
| ---------- | ----------- | ----------------- |
| 32x16 | 6-bit | ~1.5 KB |
| 64x32 | 6-bit | ~6 KB |
| 64x64 | 6-bit | ~24 KB |

_Double buffering doubles memory usage_

## Troubleshooting

### Common Issues

**Display flickers:**

- Increase refresh rate by calling `render_frame()` more frequently
- Reduce color depth to 4-bit
- Ensure refresh task runs at high priority
- Use a faster DelayNs implementation
- Use a faster microcontroller

**Colors look wrong or missing:**

- Check RGB pin connections (R1/R2, G1/G2, B1/B2)
- Verify color bit depth matches your needs
- **Low brightness colors may not display**: With 3 color bits, values less than 124
  (after gamma correction ~31) won't show as they're below the 1\<<5 threshold
- Try different gamma correction values
- Increase overall brightness to make dim colors visible

**Display shows garbage:**

- Verify address pin connections (A, B, C, D, E)
- Check that panel size matches configuration
- Ensure proper ground connections
- Verify power supply can handle the current draw
- Check for loose connections on the HUB75 connector

**Performance issues:**

- Reduce color depth (6-bit → 4-bit can double refresh rate)
- Lower refresh rate if flicker isn't an issue
- Use single buffering to save memory
- Optimize graphics drawing code
- Consider using DMA for data output (future feature)

**Power-related issues:**

- Ensure adequate power supply (5V, sufficient amperage)
- Check voltage drops across long cables
- Verify ground connections between MCU and display
- Consider separate power supplies for MCU and display

### Debug Features

Enable debug logging with the `defmt` feature:

```toml
[dependencies]
hub75 = { version = "0.1", features = ["defmt"] }
```

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Setup

```bash
git clone https://github.com/yourusername/hub75
cd hub75
cargo test
cargo doc --open

# Test platform-specific examples
cd examples/rp && cargo check --bins
cd ../nrf && cargo check --bins
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Acknowledgments

This driver was inspired by:

- [hub75-rs](https://github.com/david-sawatzke/hub75-rs) - Original HUB75 driver
- [hub75-remastered](https://github.com/adinack/hub75-remastered) - Improved architecture
- [microbit-bsp](https://github.com/lulf/microbit-bsp) - LED matrix patterns
- [embassy-rs](https://github.com/embassy-rs/embassy) - Async embedded framework
- [embedded-graphics](https://github.com/embedded-graphics/embedded-graphics) - Graphics library

## Future Roadmap

### Modular Architecture Vision

Following the successful pattern of the [smart-leds](https://github.com/smart-leds-rs) project, we plan to evolve into a modular ecosystem:

- **`hub75-traits`** - Core traits and abstractions for HUB75 displays
- **`hub75`** - This crate, providing generic async runtime support (current)
- **`hub75-bitbang`** - Generic bit-banging implementation for any GPIO
- **`hub75-stm32`** - STM32-specific optimizations using timers and DMA
- **`hub75-rp2040`** - RP2040 PIO-based high-performance driver
- **`hub75-esp32`** - ESP32 I2S/DMA implementation
- **`hub75-nrf`** - Nordic-specific optimizations

This approach will allow:

- **Shared abstractions** across all implementations
- **Platform-specific optimizations** where beneficial
- **Consistent APIs** regardless of backend choice
- **Easy migration** between implementations

### Advanced Performance Optimizations

Based on research from [hub75-rs](https://github.com/david-sawatzke/hub75-rs) and [advanced implementations](https://github.com/david-sawatzke/36c3_led_stuff/blob/b687925f00670082cba8eab4e593b8e0da07592b/c3_display/src/hub75dma.rs), future versions will implement:

#### 1. **Pre-rendered Frame Buffers**

```rust
// Future API concept
let mut display = Hub75Display::with_prerendered_buffers(pins);
display.enable_dma_output(); // Platform-specific optimization
```

**Benefits:**

- GPIO state pre-computed for entire frames
- DMA-driven output with minimal CPU usage
- Refresh rates >1kHz possible on fast MCUs
- Requires RGB pins on same GPIO port

#### 2. **Advanced Binary Code Modulation**

```rust
// Future timing control
display.set_bcm_timing(BcmTiming::OneShot(timer)); // Hardware timer integration
display.set_color_depth(ColorDepth::Bits8); // Full 8-bit color
```

**Benefits:**

- True 8-bit color depth (16.7M colors)
- Hardware timer-controlled OE timing
- Flicker-free high refresh rates
- Gamma correction support

#### 3. **Multi-Panel Chaining**

```rust
// Future multi-panel support
let chain = Hub75Chain::new()
    .add_panel(panel1, Position::new(0, 0))
    .add_panel(panel2, Position::new(64, 0))
    .build();
```

**Benefits:**

- Large display walls
- Synchronized refresh across panels
- Efficient data distribution

#### 4. **Platform-Specific Backends**

**STM32 with DMA:**

```rust
// STM32-specific optimizations
let display = Hub75Stm32::new(pins)
    .with_dma(dma_channel)
    .with_timer(tim2)
    .build();
```

**RP2040 with PIO:**

```rust
// RP2040 PIO state machine
let display = Hub75Rp2040::new(pins)
    .with_pio(pio0)
    .with_state_machine(sm0)
    .build();
```

**ESP32 with I2S:**

```rust
// ESP32 I2S parallel output
let display = Hub75Esp32::new(pins)
    .with_i2s(i2s0)
    .with_dma_buffer_size(4096)
    .build();
```

### Implementation Timeline

1. **Phase 1** (Current) - Generic async runtime implementation with DelayNs pattern
1. **Phase 2** - Extract core traits into `hub75-traits`
1. **Phase 3** - Platform-specific backend crates
1. **Phase 4** - Advanced features (chaining, 8-bit color, DMA)

### Contributing to the Ecosystem

We welcome contributions to any part of the future ecosystem:

- **Core traits design** - Help define the abstraction layer
- **Platform backends** - Implement optimized drivers for specific MCUs
- **Advanced features** - DMA integration, multi-panel support
- **Documentation** - Performance guides, migration tutorials

## Related Projects

- [smart-leds-matrix](https://github.com/smart-leds-rs/smart-leds-matrix) - For WS2812-based LED matrices
- [embedded-graphics](https://github.com/embedded-graphics/embedded-graphics) - 2D graphics library
- [embassy-rs](https://github.com/embassy-rs/embassy) - Async embedded framework
- [hub75-rs](https://github.com/david-sawatzke/hub75-rs) - Original HUB75 implementation
- [rpi-rgb-led-matrix](https://github.com/hzeller/rpi-rgb-led-matrix) - C++ reference implementation
