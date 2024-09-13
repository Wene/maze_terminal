#![no_std]
#![no_main]

use panic_halt as _;

use avr_device::interrupt;
use core::cell::RefCell;

// use arduino_hal::prelude::*;
use arduino_hal::spi;
use ws2812_spi as ws2812;
use crate::ws2812::prerendered::Ws2812;
use smart_leds::{SmartLedsWrite, RGB8};

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
    let  pins = arduino_hal::pins!(dp);

    let serial = arduino_hal::default_serial!(dp, pins, 57600);
    put_console(serial);


    let  sck = pins.d13.into_output();
    let  mosi = pins.d11.into_output();
    let  miso = pins.d12.into_pull_up_input();
    let  cs = pins.d10.into_output();
    let settings = spi::Settings {clock: spi::SerialClockRate::OscfOver8, ..Default::default()};
    let (spi, _) = spi::Spi::new(dp.SPI, sck, mosi, miso, cs, settings);


    const NUM_LEDS: usize = 6;
    let mut output_buffer = [0; 20 + (NUM_LEDS * 12)];
    let mut data: [RGB8; NUM_LEDS] = [RGB8::default(); NUM_LEDS];
    // let empty: [RGB8; 3] = [RGB8::default(); 3];
    let mut ws = Ws2812::new(spi, &mut output_buffer);

    const BRIGHTNESS: u8 = 50;
    let red = RGB8 {r: BRIGHTNESS, g: 0, b: 0};
    let green = RGB8 {r: 0, g: BRIGHTNESS, b: 0};
    let blue = RGB8 {r: 0, g: 0, b: BRIGHTNESS};
    let colors = [red, green, blue];

    println!("Hello serial console!");

    let mut pos: u8 = 0;

    loop {

        for i in 0_u8..(NUM_LEDS as u8) {
            let color_index: u8 = (pos + i) % 3;
            data[i as usize] = colors[color_index as usize];
        }

        pos += 1;

        ws.write(data.iter().cloned()).unwrap();
        arduino_hal::delay_ms(1000);
    }

    /*
    let dp: arduino_hal::Peripherals = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let serial = arduino_hal::default_serial!(dp, pins, 57600);
    put_console(serial);

    let (spi, _) = spi::Spi::new(
        dp.SPI,
        pins.sck.into_output(&mut pins.ddr),
        pins.mosi.into_output(&mut pins.ddr),
        pins.miso.into_pull_up_input(&mut pins.ddr),
        spi::Settings {clock: spi::SerialClockRate::OscfOver8, ..Default::default() },
    );



    println!("Hello serial console!");

    let mut led = pins.d13.into_output();

    let mut count = 0;

    loop {
        led.toggle();
        arduino_hal::delay_ms(500);

        println!("loop {}", count);
        count += 1;
    }

    */

}
