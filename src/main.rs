#![no_std]
#![no_main]

use panic_halt as _;

use avr_device::interrupt;
use core::cell::RefCell;

use arduino_hal::spi;
use ws2812_spi as ws2812;
use crate::ws2812::Ws2812;
use smart_leds::{brightness, colors::{BLUE, CYAN, GREEN, MAGENTA, RED, YELLOW}, SmartLedsWrite, RGB8};

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
    let settings = spi::Settings::default();
    let (spi, _) = spi::Spi::new(dp.SPI, sck, mosi, miso, cs, settings);

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

        ws.write(brightness(data.iter().cloned(), 50)).unwrap();
        arduino_hal::delay_ms(1000);
    }
  
}
