#![no_std]
#![no_main]

use panic_halt as _;

use avr_device::interrupt;
use core::cell::RefCell;

use crate::ws2812::Ws2812;
use arduino_hal::spi;
use smart_leds::{
    brightness,
    colors::GREEN,
    SmartLedsWrite, RGB8,
};
use ws2812_spi as ws2812;

type Console = arduino_hal::hal::usart::Usart0<arduino_hal::DefaultClock>;
static CONSOLE: interrupt::Mutex<RefCell<Option<Console>>> =
    interrupt::Mutex::new(RefCell::new(None));

/*
macro_rules! print {
    ($($t:tt)*) => {
        interrupt::free(
            |cs| {
                if let Some(console) = CONSOLE.borrow(cs).borrow_mut().as_mut() {
                    let _ = ufmt::uwrite!(console, $($t)*);
                }
            },
        )
    };
}
*/

macro_rules! println {
    ($($t:tt)*) => {
        interrupt::free(
            |cs| {
                if let Some(console) = CONSOLE.borrow(cs).borrow_mut().as_mut() {
                    let _ = ufmt::uwriteln!(console, $($t)*);
                }
            },
        )
    };
}

fn put_console(console: Console) {
    interrupt::free(|cs| {
        *CONSOLE.borrow(cs).borrow_mut() = Some(console);
    })
}

const NUM_LEDS: usize = 87;
const PIXEL_N: (usize, usize) = (0, 18);
const PIXEL_W: (usize, usize) = (23, 41);
const PIXEL_S: (usize, usize) = (46, 64);
const PIXEL_E: (usize, usize) = (69, 87);

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let serial = arduino_hal::default_serial!(dp, pins, 57600);
    put_console(serial);

    let sck = pins.d13.into_output();
    let mosi = pins.d11.into_output();
    let miso = pins.d12.into_pull_up_input();
    let cs = pins.d10.into_output();
    let settings = spi::Settings::default();
    let (spi, _) = spi::Spi::new(dp.SPI, sck, mosi, miso, cs, settings);


    // let mut led_n = pins.d7.into_output().downgrade(); -> 5
    // let mut led_w = pins.d6.into_output().downgrade(); -> 4
    // let mut led_e = pins.d9.into_output().downgrade(); -> 2
    // let mut led_s = pins.d8.into_output().downgrade(); -> 3

    let north = pins.d5.into_floating_input().downgrade();
    let east = pins.d4.into_floating_input().downgrade();
    let south = pins.d2.into_floating_input().downgrade();
    let west = pins.d3.into_floating_input().downgrade();

    let mut ws = Ws2812::new(spi);

    println!("Hello serial console!");

    loop {
        let mut data: [RGB8; NUM_LEDS] = [RGB8::default(); NUM_LEDS];

        if north.is_high() {
            for i in PIXEL_N.0..PIXEL_N.1 {
                data[i] = GREEN;
            }
        }
        if east.is_high() {
            for i in PIXEL_E.0..PIXEL_E.1 {
                data[i] = GREEN;
            }
        }
        if south.is_high() {
            for i in PIXEL_S.0..PIXEL_S.1 {
                data[i] = GREEN;
            }
        }
        if west.is_high() {
            for i in PIXEL_W.0..PIXEL_W.1 {
                data[i] = GREEN;
            }
        }

        ws.write(brightness(data.iter().cloned(), 25)).unwrap();
        arduino_hal::delay_ms(50);
    }
}
