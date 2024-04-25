#![no_std]
#![no_main]

use assign_resources::assign_resources;
use embassy_executor::Spawner;

use embassy_rp::clocks::{clk_sys_freq, pll_sys_freq};
use embassy_rp::config::Config;
use embassy_rp::flash::{Blocking, Flash};
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{FLASH, PIO1};
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
use rand_chacha::rand_core::RngCore;
use {defmt_rtt as _, panic_probe as _};

use rand_chacha::{rand_core::SeedableRng, ChaCha8Rng};

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
        flash: FLASH
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

pub fn get_unique_id(flash: &mut FLASH) -> Option<u64> {
    let mut flash: Flash<'_, FLASH, Blocking, { 2 * 1024 * 1024 }> = Flash::new_blocking(flash);

    // TODO: For different flash chips, we want to handle things
    // differently based on their jedec? That being said: I control
    // the hardware for this project, and both chips (Pico and XIAO)
    // support unique ID, so oh well.
    //
    // let jedec = flash.blocking_jedec_id().unwrap();

    let mut id = [0u8; core::mem::size_of::<u64>()];
    flash.blocking_unique_id(&mut id).unwrap();
    Some(u64::from_be_bytes(id))
}

pub fn get_rand(unique_id: u64) -> ChaCha8Rng {
    // TODO: Get some real entropy
    let mut seed = [0u8; 32];
    let uid = unique_id.to_le_bytes();
    seed.chunks_exact_mut(8).for_each(|c| {
        c.copy_from_slice(&uid);
    });
    ChaCha8Rng::from_seed(seed)
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // unsafe {
    //     // cortex_m::Peripherals::take().unwrap().SCB.vtor.write(0x20000000+4);
    // }
    let p = embassy_rp::init(Config::default());
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
                pull block          side 0b0
                out pins, 32        side 0b1    ; Write address
                in pins, 8          side 0b1    ; Read data
                in null, 24         side 0b0    ; Read 24 null bits
                push block          side 0b0
                jmp loop            side 0b0
        "#
    );

    let mut config2 = PioConfig::default();
    config2.use_program(&common.load_program(&prg2.program), &[&pin26]);
    config2.clock_divider = (U56F8!(125_000_000) / (4*2*1_000_000)).to_fixed();
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
async fn eeprom_test(mut res: PioResources2,  mut sm: StateMachine<'static, PIO0, 1>) {    

    sm.set_enable(true);

    // let mut dma_in_ref = res.dma1.into_ref();
    // let mut dma_out_ref = res.dma2.into_ref();

    // sm.rx().wait_pull().await;

    let mut rand = get_rand(get_unique_id(&mut res.flash).unwrap_or(21365213));

    let mut din: u32 = 99;
    let mut wrong: u32 = 0;
    let mut last_wrong: u32 = 0;
    let dma_fut = async {
        for i in 0u32..65_537 {
            let t = rand.next_u32() & 0x0000FFFF;
            Timer::after_nanos(125*4).await;
            sm.tx().wait_push(u32::from_le(t)).await;
            din = u32::from_be(sm.rx().wait_pull().await);
            if din != (t & 0x000000FF) {
                wrong += 1;
                defmt::info!("{} {} {} {}", i, din, t & 0x000000FF, i - last_wrong);
                last_wrong = i;
            }
        }
        sm.set_enable(false);
        // din = sm.rx().wait_pull().await;
        defmt::info!("Wrong values: {}", wrong);
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