//! Interfacing the on-board L3GD20 (gyroscope)
#![deny(unsafe_code)]
// #![deny(warnings)]
#![no_main]
#![no_std]

#[macro_use(entry, exception)]
extern crate cortex_m_rt as rt;
extern crate cortex_m;
extern crate panic_semihosting;
extern crate stm32l432xx_hal as hal;
extern crate ssd1351;
extern crate embedded_graphics;
extern crate heapless;

use hal::prelude::*;
use hal::spi::Spi;
use hal::stm32l4::stm32l4x2;
use rt::ExceptionFrame;
use ssd1351::builder::Builder;
use ssd1351::mode::{GraphicsMode};
use ssd1351::prelude::*;
use hal::delay::Delay;
use hal::rtc::{Rtc, Date, Time};
use hal::pwr::Pwr;
use core::fmt::Write;
use heapless::String;
use heapless::consts::*;

use embedded_graphics::prelude::*;
use embedded_graphics::fonts::Font12x16;

/// SPI mode for


entry!(main);

fn main() -> ! {
    let p = stm32l4x2::Peripherals::take().unwrap();

    let mut flash = p.FLASH.constrain();
    let mut rcc = p.RCC.constrain();

    let clocks = rcc.cfgr.sysclk(80.mhz()).pclk1(80.mhz()).pclk2(80.mhz()).freeze(&mut flash.acr);

    let mut gpioa = p.GPIOA.split(&mut rcc.ahb2);
    let mut gpiob = p.GPIOB.split(&mut rcc.ahb2);

    let cp = cortex_m::Peripherals::take().unwrap();
    let mut delay = Delay::new(cp.SYST, clocks);

    let mut pwr = Pwr::pwr(&mut rcc.apb1r1);
    let rtc = Rtc::rtc(p.RTC, &mut rcc.apb1r1, &mut rcc.bdcr, &mut pwr.cr1);

    let mut rst = gpioa
        .pa8
        .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);

    let dc = gpiob
        .pb1
        .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);

    let sck = gpioa.pa5.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
    let miso = gpioa.pa6.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
    let mosi = gpioa.pa7.into_af5(&mut gpioa.moder, &mut gpioa.afrl);

    let spi = Spi::spi1(
        p.SPI1,
        (sck, miso, mosi),
        SSD1351_SPI_MODE,
        8.mhz(),
        clocks,
        &mut rcc.apb2,
    );

    let mut time = Time::new(21, 57, 32, false);
    let mut date = Date::new(1, 24, 4, 2018);
    
    rtc.set_time(&time);
    rtc.set_date(&date);
    
    let mut display: GraphicsMode<_> = Builder::new().connect_spi(spi, dc).into();
    display.reset(&mut rst, &mut delay);
    display.init().unwrap();

    // let mut buffer: [u8; 256] = [0; 256];
    let mut buffer: String<U16> = String::new();
    loop {
        time = rtc.get_time();
        date = rtc.get_date();
        {
            write!(buffer, "{:02}:{:02}:{:02}", time.hours, time.minutes, time.seconds).unwrap();
            display.draw(Font12x16::render_str(buffer.as_str(), 0x880B_u16.into()).into_iter());
            buffer.clear(); // reset the buffer
            write!(buffer, "{:02}:{:02}:{:04}", date.date, date.month, date.year).unwrap();
            display.draw(Font12x16::render_str(buffer.as_str(), 0xF818_u16.into()).translate(Coord::new(0, 20)).into_iter());
            buffer.clear(); // reset the buffer
            // delay.delay_ms(333_u16);
        }
    }
}

exception!(HardFault, hard_fault);

fn hard_fault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}

exception!(*, default_handler);

fn default_handler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}
