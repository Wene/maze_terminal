#![no_std]
#![no_main]

use panic_halt as _;

use avr_device::interrupt;
use core::cell::RefCell;

use crate::ws2812::Ws2812;
use arduino_hal::{
    port::mode::{Input, Output, PullUp},
    port::Pin,
    spi,
};
use smart_leds::{brightness, colors::BLUE, SmartLedsWrite, RGB8};
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

struct Btn {
    north: Pin<Input<PullUp>>,
    east: Pin<Input<PullUp>>,
    south: Pin<Input<PullUp>>,
    west: Pin<Input<PullUp>>,
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

    let in_north = pins.d4.into_pull_up_input().downgrade();
    let in_east = pins.d2.into_pull_up_input().downgrade();
    let in_south = pins.d3.into_pull_up_input().downgrade();
    let in_west = pins.d5.into_pull_up_input().downgrade();

    let mut out_north = pins.d9.into_output().downgrade();
    out_north.set_high();
    let mut out_east = pins.d7.into_output().downgrade();
    out_east.set_high();
    let mut out_south = pins.d6.into_output().downgrade();
    out_south.set_high();
    let mut out_west = pins.d8.into_output().downgrade();
    out_west.set_high();

    let btn = Btn {
        north: pins.a1.into_pull_up_input().downgrade(),
        east: pins.a2.into_pull_up_input().downgrade(),
        south: pins.a3.into_pull_up_input().downgrade(),
        west: pins.a4.into_pull_up_input().downgrade(),
    };

    let mut ws = Ws2812::new(spi);

    println!("Greetings from the MazeTerminal");

    if btn.north.is_low() || btn.east.is_low() || btn.south.is_low() || btn.west.is_low() {
        loop {
            let mut data: [RGB8; NUM_LEDS] = [BLUE; NUM_LEDS];

            dark_pixel_if_low(&mut data, &btn.north, PIXEL_N);
            dark_pixel_if_low(&mut data, &btn.east, PIXEL_E);
            dark_pixel_if_low(&mut data, &btn.south, PIXEL_S);
            dark_pixel_if_low(&mut data, &btn.west, PIXEL_W);

            ws.write(brightness(data.iter().cloned(), 25)).unwrap();
            arduino_hal::delay_ms(10);
        }
    }

    let mut pos = 0;
    loop {
        let mut data: [RGB8; NUM_LEDS] = [RGB8::default(); NUM_LEDS];

        const STEP: u8 = 255_u8 / NUM_LEDS as u8 * 6_u8;
        const SEGMENT: u8 = NUM_LEDS as u8 / 6;

        let mut led_index = pos;

        for i in 0..SEGMENT {
            data[advance_led_index(&mut led_index)] = RGB8 {
                r: 255,
                g: i * STEP,
                b: 0,
            };
        }
        for i in 0..SEGMENT {
            data[advance_led_index(&mut led_index)] = RGB8 {
                r: 255 - i * STEP,
                g: 255,
                b: 0,
            };
        }
        for i in 0..SEGMENT {
            data[advance_led_index(&mut led_index)] = RGB8 {
                r: 0,
                g: 255,
                b: i * STEP,
            };
        }
        for i in 0..SEGMENT {
            data[advance_led_index(&mut led_index)] = RGB8 {
                r: 0,
                g: 255 - i * STEP,
                b: 255,
            };
        }
        for i in 0..SEGMENT {
            data[advance_led_index(&mut led_index)] = RGB8 {
                r: i * STEP,
                g: 0,
                b: 255,
            };
        }
        for i in 0..SEGMENT + (NUM_LEDS as u8 % 6) {
            data[advance_led_index(&mut led_index)] = RGB8 {
                r: 255,
                g: 0,
                b: 255 - i * STEP,
            };
        }

        pos += 1;
        if NUM_LEDS <= pos {
            pos = 0;
        }

        dark_pixel_if_low(&mut data, &in_north, PIXEL_N);
        dark_pixel_if_low(&mut data, &in_east, PIXEL_E);
        dark_pixel_if_low(&mut data, &in_south, PIXEL_S);
        dark_pixel_if_low(&mut data, &in_west, PIXEL_W);

        forward_btn_to_out(&btn.north, &mut out_north);
        forward_btn_to_out(&btn.east, &mut out_east);
        forward_btn_to_out(&btn.south, &mut out_south);
        forward_btn_to_out(&btn.west, &mut out_west);

        ws.write(brightness(data.iter().cloned(), 25)).unwrap();
        arduino_hal::delay_ms(10);
    }
}

fn advance_led_index(index: &mut usize) -> usize {
    *index += 1;
    if NUM_LEDS <= *index {
        *index = 0;
    }

    *index
}

fn forward_btn_to_out(btn: &Pin<Input<PullUp>>, out: &mut Pin<Output>) {
    if btn.is_low() {
        out.set_low();
    } else {
        out.set_high();
    }
}

fn dark_pixel_if_low(data: &mut [RGB8], input: &Pin<Input<PullUp>>, (from, to): (usize, usize)) {
    if input.is_low() {
        for i in from..to {
            data[i] = RGB8::default();
        }
    }
}
