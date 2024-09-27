#![no_std]
#![no_main]

use panic_halt as _;

use avr_device::interrupt;
use core::cell::RefCell;

use crate::ws2812::Ws2812;
use arduino_hal::{port::mode::{Input, PullUp, Output}, spi, port::Pin};
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

    let btn_north = pins.a1.into_pull_up_input().downgrade();
    let btn_east = pins.a2.into_pull_up_input().downgrade();
    let btn_south = pins.a3.into_pull_up_input().downgrade();
    let btn_west = pins.a4.into_pull_up_input().downgrade();

    let mut ws = Ws2812::new(spi);

    println!("Greetings from the MazeTerminal");

    loop {
        let mut data: [RGB8; NUM_LEDS] = [RGB8::default(); NUM_LEDS];

        light_pixel_if_high(&mut data, &in_north, PIXEL_N);
        light_pixel_if_high(&mut data, &in_east, PIXEL_E);
        light_pixel_if_high(&mut data, &in_south, PIXEL_S);
        light_pixel_if_high(&mut data, &in_west, PIXEL_W);

        forward_btn_to_out(&btn_north, &mut out_north);
        forward_btn_to_out(&btn_east, &mut out_east);
        forward_btn_to_out(&btn_south, &mut out_south);
        forward_btn_to_out(&btn_west, &mut out_west);

        ws.write(brightness(data.iter().cloned(), 25)).unwrap();
        arduino_hal::delay_ms(10);
    }
}

fn forward_btn_to_out(btn: &Pin<Input<PullUp>>, out: &mut Pin<Output>) {
    if btn.is_low() {
        out.set_low();
    }
    else {
        out.set_high();
    }
}

fn light_pixel_if_high(data: &mut [RGB8], input: &Pin<Input<PullUp>>, (from, to): (usize, usize)) {
    if input.is_high() {
        for i in from..to {
            data[i] = GREEN;
        }
    }
}