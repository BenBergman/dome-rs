#![no_std]
#![no_main]
#![feature(extern_crate_item_prelude)]

extern crate embedded_hal as hal;
extern crate itsybitsy_m0 as itsy;
extern crate libm;
extern crate panic_abort;

use hal::blocking::spi::{Transfer, Write};
use hal::digital::OutputPin;
use itsy::clock::GenericClockController;
use itsy::delay::Delay;
use itsy::prelude::*;
use itsy::time::MegaHertz;
use itsy::{entry, spi_master, CorePeripherals, Peripherals, Pins};
use libm::F32Ext;

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
    let mut pins = Pins::new(peripherals.PORT);
    let mut delay = Delay::new(core.SYST, &mut clocks);

    let led_latch = pins.d2.into_open_drain_output(&mut pins.port);
    let button_latch = pins.d7.into_open_drain_output(&mut pins.port);

    let spi = spi_master(
        &mut clocks,
        MegaHertz(10),
        peripherals.SERCOM4,
        &mut peripherals.PM,
        pins.sck,
        pins.mosi,
        pins.miso,
        &mut pins.port,
    );

    let mut lights = Lights::new(spi, led_latch, button_latch);
    let mut counter = 0;
    loop {
        let base = CARTESIAN_MAP[counter];
        let buttons = lights.read_buttons();
        if buttons & 1u8 == 1u8 {
            for (i, &point) in CARTESIAN_MAP.iter().enumerate() {
                // if distance from base is about 0.350, light it up
                let direction = [base[0] - point[0], base[1] - point[1], base[2] - point[2]];
                let distance = (direction[0] * direction[0]
                    + direction[1] * direction[1]
                    + direction[2] * direction[2])
                    .sqrt();
                if distance < 0.4 {
                    lights.set_light(i);
                }
            }
        } else {
            lights.set_light(counter);
        }
        lights.draw();

        lights.clear();
        counter = (counter + 1) % 61;
        delay.delay_ms(200u8);
    }
}

struct Lights<
    SPI: Transfer<u8, Error = E> + Write<u8>,
    LED_LATCH: OutputPin,
    BUTTON_LATCH: OutputPin,
    E,
> {
    // TODO: generalize spi and latch
    buffer: [u8; 8],
    spi: SPI,
    led_latch: LED_LATCH,
    button_latch: BUTTON_LATCH,
    // TODO: Move coordinate mapping into here? It is pretty interdependant...
}

impl<SPI, LED_LATCH, BUTTON_LATCH, E> Lights<SPI, LED_LATCH, BUTTON_LATCH, E>
where
    SPI: Transfer<u8, Error = E> + Write<u8>,
    LED_LATCH: OutputPin,
    BUTTON_LATCH: OutputPin,
{
    // TODO: generalize spi params to take anything that implements SPI trait
    fn new(spi: SPI, led_latch: LED_LATCH, button_latch: BUTTON_LATCH) -> Self {
        Self {
            buffer: [0; 8],
            spi: spi,
            led_latch: led_latch,
            button_latch: button_latch,
        }
    }

    fn set_light(&mut self, i: usize) {
        let bank = 7 - (i / 8);
        let bit = i % 8;
        self.buffer[bank] = self.buffer[bank] | (1 << bit);
    }

    fn clear(&mut self) {
        for bank in self.buffer.iter_mut() {
            *bank = 0x00;
        }
    }

    fn draw(&mut self) {
        let _ = self.spi.write(&self.buffer);
        self.led_latch.set_low();
        self.led_latch.set_high();
    }

    fn read_buttons(&mut self) -> u8 {
        self.button_latch.set_low();
        let mut buffer = [0u8; 1];
        let _ = self.spi.transfer(&mut buffer);
        self.button_latch.set_high();
        buffer[0]
    }
}

const CARTESIAN_MAP: [[f32; 3]; 61] = [
    // Bank 0
    [-0.89438856, 0.0, 0.44729087],
    [-0.8374509, -0.17523615, 0.14909694],
    [-0.7805132, -0.3504722, -0.149097],
    [-0.29812953, 0.0, 0.81576365],
    [-0.39025664, -0.283538, 0.63152725],
    [-0.48238373, -0.567076, 0.44729087],
    [-0.42544606, -0.74231213, 0.14909694],
    [-0.24119182, -0.74231213, -0.149097],
    // Bank 1
    [-0.59625906, 0.0, 0.63152725],
    [-0.68838614, -0.28353807, 0.44729087],
    [-0.63144845, -0.4587741, 0.14909694],
    [-0.5745108, -0.63401014, -0.149097],
    [-0.18425423, -0.567076, 0.63152725],
    [0.056937594, -0.74231213, 0.44729087],
    [0.24119182, -0.74231213, 0.14909694],
    [0.42544606, -0.74231213, -0.149097],
    // Bank 2
    [-0.27638134, -0.850614, 0.44729087],
    [-0.09212711, -0.850614, 0.14909694],
    [0.09212714, -0.850614, -0.149097],
    [-0.092127115, -0.283538, 0.81576365],
    [0.1490647, -0.45877418, 0.63152725],
    [0.39025652, -0.6340103, 0.44729087],
    [0.57451075, -0.63401026, 0.14909694],
    [0.63144845, -0.45877412, -0.149097],
    // Bank 3
    [0.7235755, -0.52570844, 0.44729087],
    [0.7805131, -0.3504723, 0.14909694],
    [0.8374508, -0.17523615, -0.149097],
    [0.24119182, -0.17523615, 0.81576365],
    [0.48238364, 0.0, 0.63152725],
    [0.72357553, 0.1752361, 0.44729087],
    [0.7805132, 0.35047224, 0.14909694],
    [0.63144845, 0.45877412, -0.149097],
    // Bank 4
    [0.4823837, 0.35047224, 0.63152725],
    [0.3902566, 0.63401026, 0.44729087],
    [0.24119182, 0.74231213, 0.14909694],
    [0.09212708, 0.8506141, -0.149097],
    [-0.18425415, 0.567076, 0.63152725],
    [-0.48238364, 0.567076, 0.44729087],
    [-0.63144845, 0.4587741, 0.14909694],
    [-0.7805132, 0.35047218, -0.149097],
    // Bank 5
    [0.72357553, 0.5257084, 0.44729087],
    [0.57451075, 0.63401026, 0.14909694],
    [0.425446, 0.74231213, -0.149097],
    [0.24119185, 0.17523612, 0.81576365],
    [0.14906476, 0.45877418, 0.63152725],
    [0.056937695, 0.7423122, 0.44729087],
    [-0.09212708, 0.8506141, 0.14909694],
    [-0.24119185, 0.74231213, -0.149097],
    // Bank 6
    [-0.27638122, 0.8506141, 0.44729087],
    [-0.425446, 0.74231213, 0.14909694],
    [-0.57451075, 0.63401026, -0.149097],
    [-0.09212708, 0.283538, 0.81576365],
    [-0.3902566, 0.283538, 0.63152725],
    [-0.68838614, 0.28353795, 0.44729087],
    [-0.8374509, 0.17523605, 0.14909694],
    [-0.7805132, 0.0, -0.149097],
    // Bank 7
    [0.48238364, -0.3504723, 0.63152725],
    [0.72357553, -0.17523617, 0.44729087],
    [0.7805132, 0.0, 0.14909694],
    [0.8374508, 0.17523612, -0.149097],
    [0.0, 0.0, 1.0],
];
