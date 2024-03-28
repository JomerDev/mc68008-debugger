#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::gpio;
use embassy_time::{Duration, Timer};
use gpio::{Level, Output};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let pio = p.PIO0;
    let Pio {
        mut common,
        sm0: mut sm,
        ..
    } = Pio::new(pio, Irqs);

    // let prg = get_proc();

    let mut cfg = Config::default();
    // cfg.use_program(&common.load_program(&prg.program), &[]);
    cfg.clock_divider = (U56F8!(125_000_000) / U56F8!(10_000)).to_fixed();
    cfg.shift_in = ShiftConfig {
        auto_fill: true,
        threshold: 32,
        direction: ShiftDirection::Left,
    };
    cfg.shift_out = ShiftConfig {
        auto_fill: true,
        threshold: 32,
        direction: ShiftDirection::Right,
    };
    // cfg

    sm.set_config(&cfg);
    sm.set_enable(true);

    let mut dma_out_ref = p.DMA_CH0.into_ref();
    let mut dma_in_ref = p.DMA_CH1.into_ref();
    let mut dout = [0x12345678u32; 29];
    for i in 1..dout.len() {
        dout[i] = (dout[i - 1] & 0x0fff_ffff) * 13 + 7;
    }
    let mut din = [0u32; 29];
    loop {
        let (rx, tx) = sm.rx_tx();
        join(
            tx.dma_push(dma_out_ref.reborrow(), &dout),
            rx.dma_pull(dma_in_ref.reborrow(), &mut din),
        )
        .await;
        for i in 0..din.len() {
            assert_eq!(din[i], swap_nibbles(dout[i]));
        }
        info!("Swapped {} words", dout.len());
    }
}


pub fn get_proc() -> ProgramWithDefines<ExpandedDefines, _> {
    // pio_proc::pio_file!(
    //     "set pindirs,1",
    //     "set y,7",
    //     ".wrap_target",
    //     "wait 1 gpio 0  ; Wait for analyzer-b to be ready",
    //     "set pins 3     ; Unset HALT"
    //     "wait 1 gpio 1  ; Wait for AS",
    //     "set pins, 2    ; Set HALT"
    //     "in pins, 22    ; Read address + IPL1 + IPL0",
    //     "in null, 10    ; Fill up the rest with zero",
    //     "push block     ; Push data and wait for it to be accepted"
    //     ".wrap",
    // )

    pio_proc::pio_file!("test.pio")
}