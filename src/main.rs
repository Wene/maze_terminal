#![no_std]
#![no_main]

use panic_halt as _;

use avr_device::interrupt;
use core::cell::RefCell;

use crate::ws2812::Ws2812;
use arduino_hal::{spi, port::Pin, port::mode::{Input, PullUp}};
use smart_leds::{
    brightness,
    colors::{BLUE, CYAN, GREEN, MAGENTA, RED, YELLOW},
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

    let north = pins.d6.into_pull_up_input().downgrade();
    let east = pins.d7.into_pull_up_input().downgrade();
    let south = pins.d8.into_pull_up_input().downgrade();
    let west = pins.d9.into_pull_up_input().downgrade();

    const NUM_LEDS: usize = 87;
    let mut data: [RGB8; NUM_LEDS] = [RGB8::default(); NUM_LEDS];
    let mut ws = Ws2812::new(spi);

    let colors = [RED, YELLOW, GREEN, CYAN, BLUE, MAGENTA];

    println!("Hello serial console!");

    let mut pos: u8 = 0;

    loop {
        for i in 0_u8..(NUM_LEDS as u8) {
            let color_index = (pos + i) as usize % colors.len();
            data[i as usize] = colors[color_index];
        }

        pos += 1;
        if pos >= NUM_LEDS as u8 {
            pos = 0;
        }
        println!("{:?}", pos);

        off_if_low(&north, &mut data, 0, 23);
        off_if_low(&east, &mut data, 23, 46);
        off_if_low(&south, &mut data, 46, 69);
        off_if_low(&west, &mut data, 69, 87);

        ws.write(brightness(data.iter().cloned(), 25)).unwrap();
        arduino_hal::delay_ms(500);
    }
}

fn off_if_low(button: &Pin<Input<PullUp>>, leds: &mut [RGB8], from: usize, to: usize) {
    if button.is_low() {
        for i in from..to {
            leds[i] = RGB8::default();
        }
    }
}
