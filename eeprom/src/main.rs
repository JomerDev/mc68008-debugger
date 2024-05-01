#![no_std]
#![no_main]

use assign_resources::assign_resources;
use defmt::unwrap;
use embassy_executor::{Executor, Spawner};

use embassy_rp::clocks::{clk_sys_freq, pll_sys_freq};
use embassy_rp::config::Config;
use embassy_rp::gpio::{Drive, Level, Output, SlewRate};
use embassy_rp::multicore::{spawn_core1, Stack};
use embassy_rp::pac;
use embassy_rp::peripherals::PIO1;

use embassy_rp::pio::{
    Common, Config as PioConfig, Direction, FifoJoin, InterruptHandler as InterruptHandlerPio, Pin, Pio, ShiftConfig, ShiftDirection, StateMachine
};

use embassy_rp::{
    bind_interrupts,
    peripherals,
    peripherals::PIO0,
};

use embassy_time::Timer;
use fixed::traits::ToFixed;
use static_cell::StaticCell;
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
    // unsafe {
    //     cortex_m::Peripherals::take().unwrap().SCB.vtor.write(0x20000000+4);
    // }
    let p = embassy_rp::init(Config::overclock());
    let r = split_resources!(p);

    defmt::info!("Clock speed {} {}", clk_sys_freq(), pll_sys_freq() );

    pac::BUSCTRL.bus_priority().modify(|b| {
        b.set_proc1(true);
    });

    let Pio {
        mut common,
        sm0: mut read_sm,
        sm1: mut write_sm,
        ..
    } = Pio::new(r.pins.pio, Irqs);

    let mut pin0 = common.make_pio_pin(r.pins.pin0);
    let mut pin1 = common.make_pio_pin(r.pins.pin1);
    let mut pin2 = common.make_pio_pin(r.pins.pin2);
    let mut pin3 = common.make_pio_pin(r.pins.pin3);
    let mut pin4 = common.make_pio_pin(r.pins.pin4);
    let mut pin5 = common.make_pio_pin(r.pins.pin5);
    let mut pin6 = common.make_pio_pin(r.pins.pin6);
    let mut pin7 = common.make_pio_pin(r.pins.pin7);
    let mut pin8 = common.make_pio_pin(r.pins.pin8);
    let mut pin9 = common.make_pio_pin(r.pins.pin9);
    let mut pin10 = common.make_pio_pin(r.pins.pin10);
    let mut pin11 = common.make_pio_pin(r.pins.pin11);
    let mut pin12 = common.make_pio_pin(r.pins.pin12);
    let mut pin13 = common.make_pio_pin(r.pins.pin13);
    let mut pin14 = common.make_pio_pin(r.pins.pin14);

    let mut pin15 = common.make_pio_pin(r.pins.pin15);
    let mut pin16 = common.make_pio_pin(r.pins.pin16);
    let mut pin17 = common.make_pio_pin(r.pins.pin17);
    let mut pin18 = common.make_pio_pin(r.pins.pin18);
    let mut pin19 = common.make_pio_pin(r.pins.pin19);
    let mut pin20 = common.make_pio_pin(r.pins.pin20);
    let mut pin21 = common.make_pio_pin(r.pins.pin21);
    let mut pin22 = common.make_pio_pin(r.pins.pin22);

    let mut pin26 = common.make_pio_pin(r.pins.pin26);

    setup_pins(&mut [
        &mut pin0, &mut pin1, &mut pin2, &mut pin3, &mut pin4, &mut pin5, &mut pin6, &mut pin7, &mut pin8, &mut pin9, &mut pin10, &mut pin11, &mut pin12,
        &mut pin13, &mut pin14, &mut pin15, &mut pin16, &mut pin17, &mut pin18, &mut pin19, &mut pin20, &mut pin21, &mut pin22, &mut pin26,  
    ]);

    let mut read_config = create_read_pio_config(&mut common);
    read_sm.set_pin_dirs(
        Direction::In,
        &[
            &pin0, &pin1, &pin2, &pin3, &pin4, &pin5, &pin6, &pin7, &pin8, &pin9, &pin10, &pin11, &pin12,
            &pin13, &pin14, &pin26, 
        ],
    );
    read_config.set_in_pins(&[
        &pin0, &pin1, &pin2, &pin3, &pin4, &pin5, &pin6, &pin7, &pin8, &pin9, &pin10, &pin11,
        &pin12, &pin13, &pin14, 
    ]);

    read_sm.set_config(&read_config);

    let mut write_config = create_write_pio_config(&mut common);
    write_sm.set_pin_dirs(
        Direction::Out,
        &[
            &pin15, &pin16, &pin17, &pin18, &pin19, &pin20, &pin21, &pin22,
        ],
    );
    write_sm.set_pin_dirs(
        Direction::In,
        &[
            &pin0, &pin1, &pin2, &pin3, &pin4, &pin5, &pin6, &pin7, &pin8, &pin9, &pin10, &pin11, &pin12,
            &pin13, &pin14, &pin26, 
        ],
    );
    write_config.set_in_pins(&[
        &pin0, &pin1, &pin2, &pin3, &pin4, &pin5, &pin6, &pin7, &pin8, &pin9, &pin10, &pin11,
        &pin12, &pin13, &pin14, 
    ]);
    write_config.set_out_pins(&[
        &pin15, &pin16, &pin17, &pin18, &pin19, &pin20, &pin21, &pin22,
    ]);

    write_sm.set_config(&write_config);

    Timer::after_secs(2).await;
    
    defmt::info!("Startup");

    spawner.spawn(led_test(r.pio3)).unwrap();
    // spawner.spawn(eeprom_run(r.pio, sm)).unwrap();

    spawn_core1(
        p.CORE1,
        unsafe { &mut *core::ptr::addr_of_mut!(CORE1_STACK) },
        move || {
            let executor1 = EXECUTOR1.init(Executor::new());
            executor1.run(|spawner| unwrap!(spawner.spawn(eeprom_run(r.pio, read_sm, write_sm))));
        },
    );

}

#[embassy_executor::task]
async fn eeprom_run(_res: PioResources, mut read_sm: StateMachine<'static, PIO0, 0>, mut write_sm: StateMachine<'static, PIO0, 1>) {

    read_sm.set_enable(true);
    write_sm.set_enable(true);

    defmt::info!("EEPROM core startup");

    eeprom_loop(read_sm, write_sm);
}

fn eeprom_loop(mut read_sm: StateMachine<'static, PIO0, 0>, mut write_sm: StateMachine<'static, PIO0, 1>) -> ! {
    let mut din: u32;
    let mut dout: u32;

    //test
    loop {
        if read_sm.rx().empty() {
            continue;
        }
        din = read_sm.rx().pull();
        if din < 0x00010000 {
            dout = din & 0x000000FF; //u32::from_be_bytes([0,255,(din & 0x000000FF) as u8,0]);
            write_sm.tx().push(dout);
            // defmt::info!("Value: {}", dout);
        }
    }
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


fn create_read_pio_config<'a, PIO: embassy_rp::pio::Instance>(common: &mut Common<'a, PIO>) -> PioConfig<'a, PIO> {
    let prg = pio_proc::pio_asm!(
        r#"
            loop:
                in null, 17         ; Read 17 null bits
                wait 1 gpio 26       ; Wait for OE to be negated
                in pins, 15         ; Read address
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
    config.out_sticky = false;
    config.fifo_join = FifoJoin::RxOnly;

    config
}

fn create_write_pio_config<'a, PIO: embassy_rp::pio::Instance>(common: &mut Common<'a, PIO>) -> PioConfig<'a, PIO> {
    let prg = pio_proc::pio_asm!(
        r#"
            set y, 0b00111

            loop:    
                wait 1 gpio 26       ; Wait for OE to be negated
                mov osr, pins
                out  null, 14
                jmp x!=y, output
                jmp no_output
            output:
                mov osr, ~null
                out pindirs, 8
                out pins, 32
                wait 0 gpio 26
                mov osr, null
                out pindirs, 8
                jmp loop
            no_output:
                wait 0 gpio 26
                jmp loop
        "#
    );

    // TODO: Move output enable/disable into separate state machine

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
    config.out_sticky = false;
    config.fifo_join = FifoJoin::Duplex;

    config
}

fn setup_pins<'d, PIO: embassy_rp::pio::Instance>(pins: &mut [&mut Pin<'d, PIO>]) {
    pins.iter_mut().for_each(|pin| {
        pin.set_drive_strength(Drive::_4mA);
        pin.set_slew_rate(SlewRate::Slow);
    });
}