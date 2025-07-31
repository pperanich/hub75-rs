# HUB75 Embassy Examples

This directory contains examples demonstrating how to use the `hub75-embassy` driver with different microcontrollers and scenarios.

## Structure

The examples are organized following the pattern used by other Embassy projects:

- **Root level examples** (`*.rs`): Generic examples that work with any Embassy-supported microcontroller
- **Platform-specific directories**: Examples tailored for specific microcontroller families

## Generic Examples

These examples work with any Embassy-supported microcontroller by adapting the pin configuration:

### `basic_usage.rs`

Basic setup and usage patterns for the HUB75 driver. Demonstrates:

- Display initialization
- Basic shapes and text
- Double buffering
- Task spawning

### `animations.rs`

Various animation effects and patterns. Shows:

- Color cycling
- Moving patterns
- Smooth transitions
- Performance optimization

### `animations.rs`

Various animation effects and patterns. Shows:

- Color cycling
- Moving patterns
- Smooth transitions
- Performance optimization

## Platform-Specific Examples

### `nrf/` - nRF52 Series Examples

Complete examples for nRF52832, nRF52833, and nRF52840 microcontrollers:

- `basic_display.rs` - Simple shapes and text display
- `animated_patterns.rs` - Rainbow bars, bouncing balls, plasma effects
- `text_display.rs` - Scrolling text and real-time information
- `graphics_demo.rs` - Advanced embedded-graphics features

See [`nrf/README.md`](nrf/README.md) for detailed setup instructions.

### `rp/` - RP2040/RP2350 Examples

Complete examples for Raspberry Pi Pico, Pico W, and compatible RP2040/RP2350 boards:

- `basic_display.rs` - Simple HUB75 setup with basic graphics
- `comprehensive_demo.rs` - Full-featured example with advanced task management
- `animated_effects.rs` - Visual effects including fire, plasma, matrix rain
- `performance_test.rs` - Performance measurement and optimization testing
- `pico_w_demo.rs` - WiFi integration example for Pico W boards

See [`rp/README.md`](rp/README.md) for detailed setup instructions.

## Running Examples

### Generic Examples

For generic examples, you'll need to adapt the pin configuration for your specific microcontroller:

```bash
# Build only (since pins need to be configured for your specific board)
cargo check --example animations --features embedded-graphics
```

### Platform-Specific Examples

Platform-specific examples are ready to run:

```bash
# nRF52840 example
cd examples/nrf
cargo run --release --features nrf52840 --bin basic_display

# nRF52833 (e.g., micro:bit v2)
cd examples/nrf  
cargo run --release --features nrf52833 --target thumbv7em-none-eabihf --bin basic_display

# RP2040 Pico example
cd examples/rp
cargo run --release --bin basic_display

# Pico W with WiFi
cd examples/rp
cargo run --release --features pico-w --bin pico_w_demo
```

## Hardware Requirements

All examples assume:

- A HUB75 RGB LED matrix panel (tested with 64x32 and 32x16)
- Appropriate power supply (5V, sufficient current capacity)
- Proper wiring between microcontroller and panel

## Adding New Platform Examples

To add examples for a new microcontroller family:

1. Create a new directory (e.g., `stm32/`, `esp32/`)
1. Add a platform-specific `Cargo.toml` with appropriate dependencies
1. Create `src/bin/` directory with example binaries
1. Add `.cargo/config.toml` and memory layout files
1. Include a comprehensive README with setup instructions

Follow the pattern established in the `nrf/` and `rp/` directories for consistency.

## Features

Examples demonstrate various driver features:

- **Color Depths**: 4-bit, 6-bit, and 8-bit color support
- **Panel Sizes**: 32x16, 64x32, 64x64, 128x64 configurations
- **Graphics Integration**: Full embedded-graphics compatibility
- **Performance**: Binary Code Modulation, double buffering, async refresh
- **Animations**: Smooth transitions, color effects, pattern generation

## Troubleshooting

Common issues and solutions:

1. **Compilation errors**: Ensure correct target and dependencies
1. **Display flickering**: Check power supply and refresh rate
1. **Wrong colors**: Verify RGB pin connections
1. **Performance issues**: Use release builds and appropriate color depth

For platform-specific troubleshooting, see the respective README files.
