#![no_std]
#![no_main]

use assign_resources::assign_resources;
use embassy_executor::Spawner;




// use embassy_rp::gpio::{AnyPin, Level, Output};
use embassy_rp::pio::{
    Config as PioConfig, Direction, InterruptHandler as InterruptHandlerPio, Pio
};

use embassy_rp::{
    bind_interrupts, peripherals,
    peripherals::PIO0,
    // pio::Direction,
};


use embassy_time::{Duration, Timer};


use fixed::traits::ToFixed;
use fixed_macro::types::U56F8;
use led_settings::Ws2812;
use smart_leds::{White, RGBW};
use {defmt_rtt as _, panic_probe as _};

mod led_settings;

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandlerPio<PIO0>;
});

assign_resources! {
    pio: PioResources {
        clock_pin: PIN_26,
        led_pin_red: PIN_17,
        led_pin_green: PIN_16,
        led_pin_blue: PIN_25,
        pio: PIO0,
        dma: DMA_CH0,
    }
}

// pub struct Brightness<I> {
//     iter: I,
//     brightness: u8,
// }

// impl<I> Iterator for Brightness<I>
// where
//     I: Iterator<Item = RGBW<u8>>,
// {
//     type Item = RGBW<u8>;

//     fn next(&mut self) -> Option<RGBW<u8>> {
//         self.iter.next().map(|a| RGBW {
//             r: (a.r as u16 * (self.brightness as u16 + 1) / 256) as u8,
//             g: (a.g as u16 * (self.brightness as u16 + 1) / 256) as u8,
//             b: (a.b as u16 * (self.brightness as u16 + 1) / 256) as u8,
//             a: White((a.a.0 as u16 * (self.brightness as u16 + 1) / 256) as u8),
//         })
//     }
// }

// /// Pass your iterator into this function to get reduced brightness
// pub fn brightness<I>(iter: I, brightness: u8) -> Brightness<I>
// where
//     I: Iterator<Item = RGBW<u8>>,
// {
//     Brightness { iter, brightness }
// }

pub trait Brightness {
    fn brightness(&self, brigtness: u8) -> Self;
}

impl Brightness for RGBW<u8> {
    fn brightness(&self, brightness: u8) -> Self {
        Self {
            r: (self.r as u16 * (brightness as u16 + 1) / 256) as u8,
            g: (self.g as u16 * (brightness as u16 + 1) / 256) as u8,
            b: (self.b as u16 * (brightness as u16 + 1) / 256) as u8,
            a: White((self.a.0 as u16 * (brightness as u16 + 1) / 256) as u8),
        }
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let r = split_resources!(p);
    // let speed: u64 = 4;
    // spawner.spawn(pio_task(r.pio, speed)).unwrap();
    spawner.spawn(ws2812_task(r.pio)).unwrap();
}

#[embassy_executor::task]
async fn ws2812_task(res: PioResources) {
    let pio = res.pio;
    let Pio {
        mut common,
        sm0: mut sm,
        // mut sm1,
        ..
    } = Pio::new(pio, Irqs);
    let mut ws: Ws2812<PIO0, 0, 1> = Ws2812::new(&mut common, sm, res.dma, res.clock_pin);

    loop {
        ws.write(&[RGBW{r: 255, g: 0, b: 0, a: White(0)}.brightness(32)]).await;
        Timer::after_secs(5).await;
        ws.write(&[RGBW{r: 0, g: 255, b: 0, a: White(0)}.brightness(32)]).await;
        Timer::after_secs(5).await;
        ws.write(&[RGBW{r: 0, g: 0, b: 255, a: White(0)}.brightness(32)]).await;
        Timer::after_secs(5).await;
        ws.write(&[RGBW{r: 0, g: 0, b: 0, a: White(255)}.brightness(32)]).await;
        Timer::after_secs(5).await;
    }

}

#[embassy_executor::task]
async fn pio_task(res: PioResources, mhz: u64) {
    let pio = res.pio;
    let Pio {
        mut common,
        sm0: mut sm,
        // mut sm1,
        ..
    } = Pio::new(pio, Irqs);

    let clock_pin = common.make_pio_pin(res.clock_pin);

    let prg = pio_proc::pio_asm!(
        r#"
            .side_set 1

            loop:
                nop         side 0b1
                jmp loop    side 0b0
        "#
    );

    let mut config = PioConfig::default();
    config.use_program(&common.load_program(&prg.program), &[&clock_pin]);
    config.clock_divider = (U56F8!(125_000_000) / (mhz*2*1_000_000)).to_fixed();
    sm.set_pin_dirs(Direction::Out, &[&clock_pin]);

    sm.set_config(&config);
    sm.set_enable(true);

    loop {
        Timer::after(Duration::from_secs(10)).await;
    }
}
