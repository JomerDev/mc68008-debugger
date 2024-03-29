#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_rp::pio::{
    Common, Config, FifoJoin, Instance, InterruptHandler, Pio, PioPin, ShiftDirection, StateMachine,
};
use embassy_rp::{
    bind_interrupts, gpio,
    peripherals::PIO0,
    pio::{Direction, ShiftConfig},
    Peripheral,
};
use embassy_time::{Duration, Timer};
use fixed::traits::ToFixed;
use fixed_macro::types::U56F8;
use gpio::{Level, Output};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let pio = p.PIO0;
    let Pio {
        mut common,
        sm0: mut sm,
        ..
    } = Pio::new(pio, Irqs);

    let prg = pio_proc::pio_asm!(
        "set pins 0",
        "nop",
        "nop",
        "nop",
        "nop",
        ".wrap_target",
        "wait 1 gpio 0  ; Wait for analyzer-b to be ready",
        "set pins 3     ; Unset HALT"
        "wait 1 gpio 1  ; Wait for AS",
        "set pins, 2    ; Set HALT"
        "in pins, 22    ; Read address + IPL1 + IPL0",
        "in null, 10    ; Fill up the rest with zero",
        "push block     ; Push data and wait for it to be accepted"
        ".wrap",
    );

    let mut config = Config::default();
    config.use_program(&common.load_program(&prg.program), &[]);
    config.clock_divider = (U56F8!(125_000_000) / U56F8!(16_000)).to_fixed();
    config.shift_in = ShiftConfig {
        auto_fill: false,
        threshold: 32,
        direction: ShiftDirection::Left,
    };
    config.shift_out = ShiftConfig {
        auto_fill: false,
        threshold: 32,
        direction: ShiftDirection::Right,
    };
    config.out_sticky = true;
    let pin0 = common.make_pio_pin(p.PIN_0);
    let pin1 = common.make_pio_pin(p.PIN_1);
    let pin2 = common.make_pio_pin(p.PIN_2);
    let pin3 = common.make_pio_pin(p.PIN_3);
    let pin4 = common.make_pio_pin(p.PIN_4);
    let pin5 = common.make_pio_pin(p.PIN_5);
    let pin6 = common.make_pio_pin(p.PIN_6);
    let pin7 = common.make_pio_pin(p.PIN_7);
    let pin8 = common.make_pio_pin(p.PIN_8);
    let pin9 = common.make_pio_pin(p.PIN_9);
    let pin10 = common.make_pio_pin(p.PIN_10);
    let pin11 = common.make_pio_pin(p.PIN_11);
    let pin12 = common.make_pio_pin(p.PIN_12);
    let pin13 = common.make_pio_pin(p.PIN_13);
    let pin14 = common.make_pio_pin(p.PIN_14);
    let pin15 = common.make_pio_pin(p.PIN_15);
    let pin16 = common.make_pio_pin(p.PIN_16);
    let pin17 = common.make_pio_pin(p.PIN_17);
    let pin18 = common.make_pio_pin(p.PIN_18);
    let pin19 = common.make_pio_pin(p.PIN_19);
    let pin20 = common.make_pio_pin(p.PIN_20);
    let pin21 = common.make_pio_pin(p.PIN_21);
    let pin22 = common.make_pio_pin(p.PIN_22);
    let pin23 = common.make_pio_pin(p.PIN_23);
    let pin26 = common.make_pio_pin(p.PIN_26);
    let pin27 = common.make_pio_pin(p.PIN_27);
    let pin28 = common.make_pio_pin(p.PIN_28);
    sm.set_pin_dirs(Direction::Out, &[&pin27, &pin28]);
    sm.set_pin_dirs(
        Direction::In,
        &[
            &pin0, &pin1, &pin2, &pin3, &pin4, &pin5, &pin6, &pin7, &pin8, &pin9, &pin10, &pin11,
            &pin12, &pin13, &pin14, &pin15, &pin16, &pin17, &pin18, &pin19, &pin20, &pin21, &pin22,
            &pin23, &pin26,
        ],
    );
    config.set_set_pins(&[&pin27, &pin28]);
    config.set_in_pins(&[
        &pin1, &pin2, &pin3, &pin4, &pin5, &pin6, &pin7, &pin8, &pin9, &pin10, &pin11, &pin12,
        &pin13, &pin14, &pin15, &pin16, &pin17, &pin18, &pin19, &pin20, &pin21, &pin22, &pin23,
    ]);
    config.fifo_join = FifoJoin::RxOnly;

    sm.set_config(&config);
    sm.set_enable(true);

    let mut dma_in_ref = p.DMA_CH1.into_ref();
    let mut din = [0u32; 1];
    loop {
        let rx = sm.rx();

        rx.dma_pull(dma_in_ref.reborrow(), &mut din).await;

        info!("Swapped {} words", din.len());
    }
}

fn create_pio() -> {
    
}