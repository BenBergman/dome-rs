#![no_std]
#![no_main]
#![feature(extern_crate_item_prelude)]

extern crate itsybitsy_m0 as hal;
extern crate panic_abort;

use hal::clock::GenericClockController;
use hal::delay::Delay;
use hal::prelude::*;
use hal::time::MegaHertz;
use hal::{entry, CorePeripherals, Peripherals};

#[entry]
fn main() -> ! {
    let mut peripherals = Peripherals::take().unwrap();
    let core = CorePeripherals::take().unwrap();
    let mut clocks = GenericClockController::with_internal_32kosc(
        peripherals.GCLK,
        &mut peripherals.PM,
        &mut peripherals.SYSCTRL,
        &mut peripherals.NVMCTRL,
    );
    let mut pins = hal::Pins::new(peripherals.PORT);
    let mut red_led = pins.d13.into_open_drain_output(&mut pins.port);
    let mut delay = Delay::new(core.SYST, &mut clocks);

    let mut latch = pins.d2.into_open_drain_output(&mut pins.port);

    let mut spi = hal::spi_master(
        &mut clocks,
        MegaHertz(10),
        peripherals.SERCOM4,
        &mut peripherals.PM,
        pins.sck,
        pins.mosi,
        pins.miso,
        &mut pins.port,
    );

    let mut counter = 0xFA;

    loop {
        delay.delay_ms(200u8);
        red_led.set_high();
        delay.delay_ms(200u8);
        red_led.set_low();

        let _ = spi.send(counter);
        latch.set_low();
        latch.set_high();
        counter = counter.wrapping_add(1);
    }
}
