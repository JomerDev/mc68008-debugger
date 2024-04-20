#![no_std]
#![no_main]

use assign_resources::assign_resources;
use embassy_executor::Spawner;

use embassy_rp::clocks::{clk_sys_freq, pll_sys_freq};
use embassy_rp::config::Config;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::PIO1;
use embassy_rp::pio::{
    Config as PioConfig, Direction, FifoJoin, InterruptHandler as InterruptHandlerPio, Pio,
    ShiftConfig, ShiftDirection, StateMachine,
};

use embassy_rp::{
    bind_interrupts,
    peripherals,
    peripherals::PIO0,
};

use embassy_time::Timer;
use fixed::traits::ToFixed;
use fixed_macro::types::U56F8;
use {defmt_rtt as _, panic_probe as _};

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
    },
    pio3: PioResources3 {
        pin25: PIN_25
    }
}

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
    // unsafe {
    //     // cortex_m::Peripherals::take().unwrap().SCB.vtor.write(0x20000000+4);
    // }
    let p = embassy_rp::init(Config::overclock());
    let r = split_resources!(p);

    defmt::info!("Clock speed {} {}", clk_sys_freq(), pll_sys_freq() );

    // pac::BUSCTRL.bus_priority().modify(|b| {
    //     b.set_proc1(true);
    // });

    let Pio {
        mut common,
        // sm0: mut sm,
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

    let prg2 = pio_proc::pio_asm!(
        r#"
            .side_set 1

            loop:
                pull block          side 0b1
                out pins, 32        side 0b0    ; Write address
                ;nop                 side 0b1
                ;nop                 side 0b1
                nop                 side 0b0
                in pins, 8          side 0b0    ; Read data
                in null, 24         side 0b1    ; Read 24 null bits
                push block          side 0b1
                ;nop                 side 0b0
                ;nop                 side 0b0
                jmp loop            side 0b1
        "#
    );

    let mut config2 = PioConfig::default();
    config2.use_program(&common.load_program(&prg2.program), &[&pin26]);
    config2.clock_divider = (U56F8!(125_000_000) / (1*1*1_000_000)).to_fixed();
    config2.shift_in = ShiftConfig {
        auto_fill: false,
        threshold: 32,
        direction: ShiftDirection::Left,
    };
    config2.shift_out = ShiftConfig {
        auto_fill: false,
        threshold: 32,
        direction: ShiftDirection::Right,
    };
    config2.out_sticky = false;
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
    spawner.spawn(eeprom_test(r.pio2, sm1)).unwrap();
    spawner.spawn(led_test(r.pio3)).unwrap();

}


#[embassy_executor::task]
async fn eeprom_test(_res: PioResources2,  mut sm: StateMachine<'static, PIO0, 1>) {    

    sm.set_enable(true);

    // let mut dma_in_ref = res.dma1.into_ref();
    // let mut dma_out_ref = res.dma2.into_ref();

    // sm.rx().wait_pull().await;
    let mut din: u32 = 99;
    let dma_fut = async {
        for t in 0u8..33 {
            Timer::after_millis(50).await;
            sm.tx().wait_push(u32::from_be_bytes([0,0,0,t])).await;
            din = sm.rx().wait_pull().await;
            defmt::info!("{} {}", t, u32::from_be(din));
        }
        // sm.set_enable(false);
        din = sm.rx().wait_pull().await;
        defmt::info!("Last value: {}", u32::from_be(din));
    };
    dma_fut.await;
}

#[embassy_executor::task]
async fn led_test(res: PioResources3) {    
    let mut out = Output::new(res.pin25, Level::Low);
    loop {
        out.set_high();
        Timer::after_secs(1).await;
        out.set_low();
        Timer::after_secs(1).await;
    };
}
