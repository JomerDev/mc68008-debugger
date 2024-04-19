#![no_std]
#![no_main]

use assign_resources::assign_resources;
use defmt::unwrap;
use embassy_executor::{Executor, Spawner};

use embassy_rp::clocks::{clk_sys_freq, pll_sys_freq};
use embassy_rp::config::Config;
use embassy_rp::multicore::{spawn_core1, Stack};
use embassy_rp::peripherals::PIO1;
// use embassy_rp::gpio::{AnyPin, Level, Output};
use embassy_rp::pio::{
    Config as PioConfig, Direction, FifoJoin, InterruptHandler as InterruptHandlerPio, Pio,
    ShiftConfig, ShiftDirection, StateMachine,
};

use embassy_rp::{pac, Peripheral};
use embassy_rp::{
    bind_interrupts,
    peripherals,
    peripherals::PIO0,
    // pio::Direction,
};

use embassy_time::Timer;
use fixed::traits::ToFixed;
use fixed_macro::types::U56F8;
use {defmt_rtt as _, panic_probe as _};
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandlerPio<PIO0>;
    PIO1_IRQ_0 => InterruptHandlerPio<PIO1>;
});

assign_resources! {
    pins: Pins {
        pin0: PIN_0,
        pin1: PIN_1,
        pin2: PIN_2,
        pin3: PIN_3,
        pin4: PIN_4,
        pin5: PIN_5,
        pin6: PIN_6,
        pin7: PIN_7,
        pin8: PIN_8,
        pin9: PIN_9,
        pin10: PIN_10,
        pin11: PIN_11,
        pin12: PIN_12,
        pin13: PIN_13,
        pin14: PIN_14,
        pin15: PIN_15,
        pin16: PIN_16,
        pin17: PIN_17,
        pin18: PIN_18,
        pin19: PIN_19,
        pin20: PIN_20,
        pin21: PIN_21,
        pin22: PIN_22,
        pin24: PIN_24,
        pin26: PIN_26,
        pio: PIO0,
    }
    pio: PioResources {
        dma1: DMA_CH0,
        dma2: DMA_CH1,
    },
    pio2: PioResources2 {
        dma1: DMA_CH2,
        dma2: DMA_CH3,
    }
}

static mut CORE1_STACK: Stack<1024> = Stack::new();
static EXECUTOR1: StaticCell<Executor> = StaticCell::new();

trait Overclock<T> {
    fn overclock() -> T;
}

impl Overclock<embassy_rp::config::Config> for embassy_rp::config::Config {
    fn overclock() -> Self {
        let mut config = Self::default();
        if let Some(xosc) = config.clocks.xosc.as_mut() {
            if let Some(sys_pll) = xosc.sys_pll.as_mut() {
                sys_pll.post_div2 = 1;
            }
        }
        config
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Config::default());
    let r = split_resources!(p);

    defmt::info!("Clock speed {} {}", clk_sys_freq(), pll_sys_freq() );

    pac::BUSCTRL.bus_priority().modify(|b| {
        b.set_proc1(true);
    });

    let Pio {
        mut common,
        sm0: mut sm,
        mut sm1,
        ..
    } = Pio::new(r.pins.pio, Irqs);

    let pin0 = common.make_pio_pin(r.pins.pin0);
    let pin1 = common.make_pio_pin(r.pins.pin1);
    let pin2 = common.make_pio_pin(r.pins.pin2);
    let pin3 = common.make_pio_pin(r.pins.pin3);
    let pin4 = common.make_pio_pin(r.pins.pin4);
    let pin5 = common.make_pio_pin(r.pins.pin5);
    let pin6 = common.make_pio_pin(r.pins.pin6);
    let pin7 = common.make_pio_pin(r.pins.pin7);
    let pin8 = common.make_pio_pin(r.pins.pin8);
    let pin9 = common.make_pio_pin(r.pins.pin9);
    let pin10 = common.make_pio_pin(r.pins.pin10);
    let pin11 = common.make_pio_pin(r.pins.pin11);
    let pin12 = common.make_pio_pin(r.pins.pin12);
    let pin13 = common.make_pio_pin(r.pins.pin13);
    let pin14 = common.make_pio_pin(r.pins.pin14);

    let pin15 = common.make_pio_pin(r.pins.pin15);
    let pin16 = common.make_pio_pin(r.pins.pin16);
    let pin17 = common.make_pio_pin(r.pins.pin17);
    let pin18 = common.make_pio_pin(r.pins.pin18);
    let pin19 = common.make_pio_pin(r.pins.pin19);
    let pin20 = common.make_pio_pin(r.pins.pin20);
    let pin21 = common.make_pio_pin(r.pins.pin21);
    let pin22 = common.make_pio_pin(r.pins.pin22);
    let pin26 = common.make_pio_pin(r.pins.pin26);

    let prg = pio_proc::pio_asm!(
        r#"
            loop:
                in null, 17         ; Read 17 null bits
                wait 1 gpio 26       ; Wait for OE to be negated
                in pins, 15         ; Read address
                out pins, 32         ; Write data
                wait 0 gpio 26
                jmp loop
        "#
    );

    let mut config = PioConfig::default();
    config.use_program(&common.load_program(&prg.program), &[]);
    config.clock_divider = (1).to_fixed();
    config.shift_in = ShiftConfig {
        auto_fill: true,
        threshold: 32,
        direction: ShiftDirection::Left,
    };
    config.shift_out = ShiftConfig {
        auto_fill: true,
        threshold: 32,
        direction: ShiftDirection::Right,
    };
    config.out_sticky = true;
    sm.set_pin_dirs(
        Direction::Out,
        &[
            &pin15, &pin16, &pin17, &pin18, &pin19, &pin20, &pin21, &pin22,
        ],
    );
    sm.set_pin_dirs(
        Direction::In,
        &[
            &pin0, &pin1, &pin2, &pin3, &pin4, &pin5, &pin6, &pin7, &pin8, &pin9, &pin10, &pin11, &pin12,
            &pin13, &pin14, &pin26, &pin15, &pin16, &pin17, &pin18, &pin19, &pin20, &pin21, &pin22,
        ],
    );
    config.set_in_pins(&[
        &pin0, &pin1, &pin2, &pin3, &pin4, &pin5, &pin6, &pin7, &pin8, &pin9, &pin10, &pin11,
        &pin12, &pin13, &pin14, 
    ]);
    config.set_out_pins(&[
        &pin15, &pin16, &pin17, &pin18, &pin19, &pin20, &pin21, &pin22,
    ]);
    config.fifo_join = FifoJoin::Duplex;

    sm.set_config(&config);

    let prg2 = pio_proc::pio_asm!(
        r#"
            .side_set 1

            loop:
                out pins, 32        side 0b1    ; Write data
                ;nop                 side 0b1
                ;nop                 side 0b1
                ;nop                 side 0b1
                in pins, 8          side 0b1    ; Read address
                in null, 24         side 0b0    ; Read 24 null bits
                ;nop                 side 0b0
                ;nop                 side 0b0
                jmp loop            side 0b0
        "#
    );

    let mut config2 = PioConfig::default();
    config2.use_program(&common.load_program(&prg2.program), &[&pin26]);
    config2.clock_divider = (U56F8!(125_000_000) / (4*2*1_000_000)).to_fixed();
    config2.shift_in = ShiftConfig {
        auto_fill: true,
        threshold: 32,
        direction: ShiftDirection::Left,
    };
    config2.shift_out = ShiftConfig {
        auto_fill: true,
        threshold: 32,
        direction: ShiftDirection::Right,
    };
    config2.out_sticky = true;
    sm1.set_pin_dirs(
        Direction::In,
        &[
            &pin15, &pin16, &pin17, &pin18, &pin19, &pin20, &pin21, &pin22,
        ],
    );
    sm1.set_pin_dirs(
        Direction::Out,
        &[
            &pin0, &pin1, &pin2, &pin3, &pin4, &pin5, &pin6, &pin7, &pin8, &pin9, &pin10, &pin11, &pin12,
            &pin13, &pin14, &pin26, 
            &pin15, &pin16, &pin17, &pin18, &pin19, &pin20, &pin21, &pin22,
        ],
    );
    config2.set_out_pins(&[
        &pin0, &pin1, &pin2, &pin3, &pin4, &pin5, &pin6, &pin7, &pin8, &pin9, &pin10, &pin11,
        &pin12, &pin13, &pin14,
    ]);
    config2.set_in_pins(&[
        &pin15, &pin16, &pin17, &pin18, &pin19, &pin20, &pin21, &pin22,
    ]);
    config2.fifo_join = FifoJoin::Duplex;

    sm1.set_config(&config2);

    Timer::after_secs(2).await;
    
    defmt::info!("Startup");

    // let con = ShiftConfig::default();

    // defmt::info!("ShiftConfig: {}, {}, {}", con.threshold, con.direction, con.auto_fill);
    spawner.spawn(eeprom_test(r.pio2, sm1)).unwrap();

    spawn_core1(
        p.CORE1,
        unsafe { &mut *core::ptr::addr_of_mut!(CORE1_STACK) },
        move || {
            let executor1 = EXECUTOR1.init(Executor::new());
            executor1.run(|spawner| unwrap!(spawner.spawn(eeprom(r.pio, sm))));
        },
    );

}

#[embassy_executor::task]
async fn eeprom(res: PioResources, mut sm: StateMachine<'static, PIO0, 0>) {

    defmt::info!("Core 2: Startup");

    sm.set_enable(true);

    let mut din = [0u32; 1];
    let mut dout: u32 = 0; //[0u32; 1];

    let mut dma_out_ref = res.dma1.into_ref();
    let mut dma_in_ref = res.dma2.into_ref();

    loop {
        sm.rx().dma_pull(dma_in_ref.reborrow(), &mut din).await;
        // din = sm.rx().wait_pull().await;
        if din[0] < 0x00010000 {
            dout = din[0] & 0x000000FF;
            // sm.tx().wait_push(dout).await;
            sm.tx().dma_push(dma_out_ref.reborrow(), &[dout]).await;
        }
    }
}

#[embassy_executor::task]
async fn eeprom_test(res: PioResources2,  mut sm: StateMachine<'static, PIO0, 1>) {    

    sm.set_enable(true);

    // let mut dma_in_ref = res.dma1.into_ref();
    // let mut dma_out_ref = res.dma2.into_ref();
    let mut din: u32 = 0;
    let dma_fut = async {
        for t in 0u8..255 {
            Timer::after_millis(50).await;
            sm.tx().wait_push(u32::from_be_bytes([0,0,0,t])).await;
            din = sm.rx().wait_pull().await;
            defmt::info!("{} {}", t, u32::from_be(din));
        }
        sm.set_enable(false);
    };
    dma_fut.await;
}
