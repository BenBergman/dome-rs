#![no_std]
#![no_main]
#![feature(extern_crate_item_prelude)]

extern crate embedded_hal as hal;
extern crate itsybitsy_m0 as itsy;
extern crate libm;
extern crate panic_abort;

use hal::digital::OutputPin;
use itsy::clock::GenericClockController;
use itsy::delay::Delay;
use itsy::prelude::*;
use itsy::time::MegaHertz;
use itsy::{entry, CorePeripherals, Peripherals};
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
    let mut pins = itsy::Pins::new(peripherals.PORT);
    let mut _red_led = pins.d13.into_open_drain_output(&mut pins.port);
    let mut delay = Delay::new(core.SYST, &mut clocks);

    let mut latch = pins.d2.into_open_drain_output(&mut pins.port);

    let mut spi = itsy::spi_master(
        &mut clocks,
        MegaHertz(10),
        peripherals.SERCOM4,
        &mut peripherals.PM,
        pins.sck,
        pins.mosi,
        pins.miso,
        &mut pins.port,
    );

    let mut lights = Lights::new(spi, latch);
    let mut counter = 0;
    loop {
        /*
        for &frame in DATA.iter() {
            for &val in frame.iter() {
                let _ = spi.send(val);
                latch.set_low();
                latch.set_high();
            }
            delay.delay_ms(200u8);
        }
        */
        /*
        let base = CARTESIAN_MAP[0];
        for (_i, &point) in CARTESIAN_MAP.iter().enumerate() {
            // if distance from base is about 0.350, light it up
            let direction = [base[0] + point[0], base[1] + base[1], base[2] + base[2]];
            let distance = (direction[0] * direction[0]
                + direction[1] * direction[1]
                + direction[2] * direction[2])
                .sqrt();
            if distance < 0.4 {
                // light up point i
            }
        }
        */

        lights.clear();
        lights.set_light(counter);
        lights.draw();

        counter = (counter + 1) % 60;
        delay.delay_ms(100u8);
    }
}

struct Lights<SPI: hal::blocking::spi::Write<u8>, LATCH: OutputPin> {
    // TODO: generalize spi and latch
    buffer: [u8; 8],
    spi: SPI,
    latch: LATCH,
    // TODO: Move coordinate mapping into here? It is pretty interdependant...
}

impl<SPI, LATCH> Lights<SPI, LATCH>
where
    SPI: hal::blocking::spi::Write<u8>,
    LATCH: OutputPin,
{
    // TODO: generalize spi params to take anything that implements SPI trait
    fn new(spi: SPI, latch: LATCH) -> Self {
        Self {
            buffer: [0; 8],
            spi: spi,
            latch: latch,
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
        self.latch.set_low();
        self.latch.set_high();
    }
}

const DATA: [[u8; 8]; 1] = [
    // Cycle through banks
    //   6     5     4     3     2     1     0
    [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01],
    /*
       [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF],
       [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0x00],
       [0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0x00, 0x00],
       [0x00, 0x00, 0x00, 0x00, 0xFF, 0x00, 0x00, 0x00],
       [0x00, 0x00, 0x00, 0xFF, 0x00, 0x00, 0x00, 0x00],
       [0x00, 0x00, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00],
       [0x00, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
       [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF],
       [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0x00],
       [0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0x00, 0x00],
       [0x00, 0x00, 0x00, 0x00, 0xFF, 0x00, 0x00, 0x00],
       [0x00, 0x00, 0x00, 0xFF, 0x00, 0x00, 0x00, 0x00],
       [0x00, 0x00, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00],
       [0x00, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
       [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF],
       [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0x00],
       [0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0x00, 0x00],
       [0x00, 0x00, 0x00, 0x00, 0xFF, 0x00, 0x00, 0x00],
       [0x00, 0x00, 0x00, 0xFF, 0x00, 0x00, 0x00, 0x00],
       [0x00, 0x00, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00],
       [0x00, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
    // Cycle down rings
    [0x00, 0x08, 0x08, 0x00, 0x08, 0x08, 0x00, 0x08],
    [0x00, 0x10, 0x10, 0x11, 0x10, 0x10, 0x11, 0x10],
    [0x00, 0x20, 0x21, 0x22, 0x01, 0x21, 0x22, 0x21],
    [0x00, 0x42, 0x00, 0x44, 0x42, 0x42, 0x44, 0x42],
    [0x00, 0x84, 0x84, 0x88, 0x84, 0x84, 0x88, 0x84],
    [0x00, 0x08, 0x08, 0x00, 0x08, 0x08, 0x00, 0x08],
    [0x00, 0x10, 0x10, 0x11, 0x10, 0x10, 0x11, 0x10],
    [0x00, 0x20, 0x21, 0x22, 0x01, 0x21, 0x22, 0x21],
    [0x00, 0x42, 0x00, 0x44, 0x42, 0x42, 0x44, 0x42],
    [0x00, 0x84, 0x84, 0x88, 0x84, 0x84, 0x88, 0x84],
    [0x00, 0x08, 0x08, 0x00, 0x08, 0x08, 0x00, 0x08],
    [0x00, 0x10, 0x10, 0x11, 0x10, 0x10, 0x11, 0x10],
    [0x00, 0x20, 0x21, 0x22, 0x01, 0x21, 0x22, 0x21],
    [0x00, 0x42, 0x00, 0x44, 0x42, 0x42, 0x44, 0x42],
    [0x00, 0x84, 0x84, 0x88, 0x84, 0x84, 0x88, 0x84],
    [0x00, 0x08, 0x08, 0x00, 0x08, 0x08, 0x00, 0x08],
    [0x00, 0x10, 0x10, 0x11, 0x10, 0x10, 0x11, 0x10],
    [0x00, 0x20, 0x21, 0x22, 0x01, 0x21, 0x22, 0x21],
    [0x00, 0x42, 0x00, 0x44, 0x42, 0x42, 0x44, 0x42],
    [0x00, 0x84, 0x84, 0x88, 0x84, 0x84, 0x88, 0x84],
    */
    /*
    [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
    */
];

const CARTESIAN_MAP: [[f32; 3]; 61] = [
    [-0.89438856, 0.0, 0.44729087],
    [-0.8374509, -0.17523615, 0.14909694],
    [-0.7805132, -0.3504722, -0.149097],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
];

/*
[0.7805132, 0, 0.14909694],
[0.48238364, 0, 0.63152725],
[0, 0, 1],
[-0.29812953, 0, 0.81576365],
[-0.59625906, 0, 0.63152725],
[-0.7805132, 0, -0.149097],
[-0.57451075, 0.63401026, -0.149097],
[0.056937594, -0.74231213, 0.44729087],
[-0.09212711, -0.850614, 0.14909694],
[0.42544606, -0.74231213, -0.149097],
[-0.63144845, 0.4587741, 0.14909694],
[-0.63144845, -0.4587741, 0.14909694],
[-0.27638134, -0.850614, 0.44729087],
[0.72357553, 0.5257084, 0.44729087],
[0.48238364, -0.3504723, 0.63152725],
[0.24119185, 0.17523612, 0.81576365],
[-0.39025664, -0.283538, 0.63152725],
[0.72357553, 0.1752361, 0.44729087],
[-0.42544606, -0.74231213, 0.14909694],
[-0.24119185, 0.74231213, -0.149097],
[0.7805131, -0.3504723, 0.14909694],
[-0.24119182, -0.74231213, -0.149097],
[0.24119182, -0.17523615, 0.81576365],
[0.72357553, -0.17523617, 0.44729087],
[0.63144845, 0.45877412, -0.149097],
[-0.48238364, 0.567076, 0.44729087],
[0.63144845, -0.45877412, -0.149097],
[-0.092127115, -0.283538, 0.81576365],
[0.57451075, 0.63401026, 0.14909694],
[0.4823837, 0.35047224, 0.63152725],
[-0.8374509, 0.17523605, 0.14909694],
[-0.3902566, 0.283538, 0.63152725],
[0.056937695, 0.7423122, 0.44729087],
[-0.68838614, -0.28353807, 0.44729087],
[-0.18425415, 0.567076, 0.63152725],
[0.24119182, 0.74231213, 0.14909694],
[-0.7805132, 0.35047218, -0.149097],
[0.7805132, 0.35047224, 0.14909694],
[0.09212714, -0.850614, -0.149097],
[-0.09212708, 0.283538, 0.81576365],
[0.425446, 0.74231213, -0.149097],
[-0.68838614, 0.28353795, 0.44729087],
[-0.5745108, -0.63401014, -0.149097],
[0.57451075, -0.63401026, 0.14909694],
[-0.425446, 0.74231213, 0.14909694],
[0.7235755, -0.52570844, 0.44729087],
[0.24119182, -0.74231213, 0.14909694],
[-0.27638122, 0.8506141, 0.44729087],
[0.8374508, 0.17523612, -0.149097],
[0.8374508, -0.17523615, -0.149097],
[-0.48238373, -0.567076, 0.44729087],
[0.39025652, -0.6340103, 0.44729087],
[0.09212708, 0.8506141, -0.149097],
[0.3902566, 0.63401026, 0.44729087],
[0.1490647, -0.45877418, 0.63152725],
[-0.18425423, -0.567076, 0.63152725],
[-0.09212708, 0.8506141, 0.14909694],
[0.14906476, 0.45877418, 0.63152725],
*/
