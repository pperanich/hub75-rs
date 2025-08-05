//! Basic HUB75 display example for nRF52 series microcontrollers
//!
//! This example demonstrates:
//! - Setting up a HUB75 display with embassy-nrf
//! - Drawing basic shapes and text
//! - Running the display refresh task
//!
//! Hardware connections (example for 64x32 panel):
//! - R1, G1, B1: P0.02, P0.03, P0.04
//! - R2, G2, B2: P0.05, P0.06, P0.07
//! - A, B, C, D: P0.08, P0.28, P0.29, P0.30
//! - CLK: P0.12
//! - LAT: P0.13
//! - OE: P0.14

#![no_std]
#![no_main]

use core::ops::DerefMut;
use embassy_executor::Spawner;
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::{Duration, Timer};
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Circle, PrimitiveStyleBuilder, Rectangle},
    text::Text,
};
use hub75::{display, Hub75AddressPins, Hub75ControlPins, Hub75Display, Hub75Pins, Hub75RgbPins};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

type Display = Hub75Display<Output<'static>, 32, 32, 2>;

static DISPLAY: StaticCell<Mutex<NoopRawMutex, Display>> = StaticCell::new();

#[embassy_executor::task]
pub async fn refresh_task(display_handle: &'static Mutex<NoopRawMutex, Display>) -> ! {
    let mut delay = embassy_time::Delay;

    loop {
        {
            let mut display = display_handle.lock().await;
            let _ = display.render_frame(&mut delay).await;
        }
        Timer::after(Duration::from_millis(1)).await;
    }
}

#[embassy_executor::task]
async fn combined_display_task(display_handle: &'static Mutex<NoopRawMutex, Display>) {
    defmt::info!("Starting combined display and graphics task");

    let mut counter = 0u32;

    // Create a delay provider using embassy-time
    let mut delay = embassy_time::Delay;

    loop {
        // Clear the back buffer
        defmt::info!("LOOP");
        {
            let mut display_guard = display_handle.lock().await;
            let mut display = display_guard.deref_mut();
            display.clear();

            Rectangle::new(Point::new(0, 0), Size::new(32, 32))
                .into_styled(
                    PrimitiveStyleBuilder::new()
                        .fill_color(Rgb565::WHITE)
                        .build(),
                )
                .draw(display)
                .unwrap();

            // Draw a red rectangle
            Rectangle::new(Point::new(5, 16), Size::new(2, 2))
                .into_styled(PrimitiveStyleBuilder::new().fill_color(Rgb565::RED).build())
                .draw(display)
                .unwrap();

            // Draw a green circle
            Circle::new(Point::new(16, 16), 6)
                .into_styled(
                    PrimitiveStyleBuilder::new()
                        .fill_color(Rgb565::GREEN)
                        .build(),
                )
                .draw(display)
                .unwrap();

            // // Draw a blue rectangle
            // Rectangle::new(Point::new(45, 2), Size::new(15, 12))
            //     .into_styled(PrimitiveStyleBuilder::new().fill_color(Rgb565::BLUE).build())
            //     .draw(&mut display)
            //     .unwrap();

            // // Draw counter text
            // let mut text_buffer = heapless::String::<32>::new();
            // core::fmt::write(&mut text_buffer, format_args!("Count: {}", counter)).unwrap();

            // Text::new(
            //     &text_buffer,
            //     Point::new(2, 25),
            //     MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE),
            // )
            // .draw(&mut display)
            // .unwrap();

            // Show nRF52 info
            Text::new(
                "nRF52",
                Point::new(2, 10),
                MonoTextStyle::new(&FONT_6X10, Rgb565::CYAN),
            )
            .draw(display)
            .unwrap();

            // Swap buffers to display the new frame
            display.swap_buffers();
        }

        counter = counter.wrapping_add(1);
        Timer::after(Duration::from_millis(100)).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    defmt::info!("nRF HUB75 Basic Display Example");

    // Configure HUB75 pins for nRF52 using commonly available pins
    let drive = OutputDrive::Standard;

    let pins = Hub75Pins {
        rgb: Hub75RgbPins {
            r1: Output::new(p.P0_07, Level::Low, drive), // R1 - GPIO6 - D6 - P0.07
            g1: Output::new(p.P0_03, Level::Low, drive), // G1 - GPIOA5 - A5 - P0.03
            b1: Output::new(p.P0_05, Level::Low, drive), // B1 - GPIOA1 - A1 - P0.05
            r2: Output::new(p.P0_04, Level::Low, drive), // R2 - GPIOA0 - A0 - P0.04
            g2: Output::new(p.P0_02, Level::Low, drive), // G2 - GPIOA4 - A4 - P0.02
            b2: Output::new(p.P0_06, Level::Low, drive), // B2 - GPIO11 - D11 - P0.06
        },
        address: Hub75AddressPins {
            a: Output::new(p.P0_27, Level::Low, drive), // ADDR_A - GPIO10 - D10 - P0.27
            b: Output::new(p.P1_08, Level::Low, drive), // ADDR_B - GPIO5 - D5 - P1.08
            c: Output::new(p.P1_09, Level::Low, drive), // ADDR_C - GPIO13 - D13 - P1.09
            d: Some(Output::new(p.P0_26, Level::Low, drive)), // ADDR_D - GPIO9 - D9 - P0.26
            e: None,
        },
        control: Hub75ControlPins {
            clk: Output::new(p.P0_08, Level::Low, drive), // CLK - GPIO12 - D12 - P0.08
            lat: Output::new(p.P0_24, Level::Low, drive), // LAT - GPIORX - RXD - P0.24
            oe: Output::new(p.P0_25, Level::High, drive), // OE - GPIOTX - TXD - P0.25
        },
    };

    // Create the display
    let mut display = match Hub75Display::new(pins) {
        Ok(display) => display,
        Err(e) => {
            defmt::error!("Failed to create display: {:?}", e);
            return;
        }
    };
    defmt::info!("Display initialized");

    // Enable double buffering for smooth updates
    display.set_double_buffering(true);

    let display = DISPLAY.init(Mutex::new(display));

    // Since the display can't be cloned, we need to use a different approach
    // For now, let's combine both tasks into one
    spawner.spawn(combined_display_task(display)).unwrap();
    spawner.spawn(refresh_task(display)).unwrap();

    defmt::info!("Tasks spawned, entering main loop");

    // Main task can do other work or just sleep
    loop {
        Timer::after(Duration::from_secs(1)).await;
        defmt::info!("Main loop tick");
    }
}

