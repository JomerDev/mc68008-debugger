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
use {defmt_rtt as _, panic_probe as _};

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
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let r = split_resources!(p);
    let speed: u64 = 4;
    spawner.spawn(pio_task(r.pio, speed)).unwrap();
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
