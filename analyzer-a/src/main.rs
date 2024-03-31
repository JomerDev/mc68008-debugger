#![no_std]
#![no_main]

use assign_resources::assign_resources;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_futures::select::{select, Either};
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::USB;
use embassy_rp::pio::{
    Config as PioConfig, FifoJoin, InterruptHandler as InterruptHandlerPio, Pio, ShiftDirection,
};
use embassy_rp::usb::{Driver, InterruptHandler as InterruptHandlerUsb};
use embassy_rp::{
    bind_interrupts, peripherals,
    peripherals::PIO0,
    pio::{Direction, ShiftConfig},
    Peripheral,
};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::Timer;
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::{Builder, Config};
use fixed::traits::ToFixed;
use fixed_macro::types::U56F8;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandlerPio<PIO0>;
    USBCTRL_IRQ => InterruptHandlerUsb<USB>;
});

static SHARED: Channel<ThreadModeRawMutex, u32, 4> = Channel::new();

static MHZ: u64 = 2;

assign_resources! {
    pio: PioResources {
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
        pin27: PIN_27,
        pin28: PIN_28,
        pio: PIO0,
        dma: DMA_CH0,
    }
    usb: UsbResources {
        usb: USB
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let r = split_resources!(p);
    spawner.spawn(usb_serial(r.usb)).unwrap();
    spawner.spawn(pio_task(r.pio)).unwrap();

    // loop {}
}

#[embassy_executor::task]
async fn pio_task(res: PioResources) {
    let pio = res.pio;
    let Pio {
        mut common,
        sm0: mut sm,
        mut sm1,
        mut sm2,
        ..
    } = Pio::new(pio, Irqs);

    let pin0 = common.make_pio_pin(res.pin0);
    let pin1 = common.make_pio_pin(res.pin1);
    let pin2 = common.make_pio_pin(res.pin2);
    let pin3 = common.make_pio_pin(res.pin3);
    let pin4 = common.make_pio_pin(res.pin4);
    let pin5 = common.make_pio_pin(res.pin5);
    let pin6 = common.make_pio_pin(res.pin6);
    let pin7 = common.make_pio_pin(res.pin7);
    let pin8 = common.make_pio_pin(res.pin8);
    let pin9 = common.make_pio_pin(res.pin9);
    let pin10 = common.make_pio_pin(res.pin10);
    let pin11 = common.make_pio_pin(res.pin11);
    let pin12 = common.make_pio_pin(res.pin12);
    let pin13 = common.make_pio_pin(res.pin13);
    let pin14 = common.make_pio_pin(res.pin14);
    let pin15 = common.make_pio_pin(res.pin15);
    let pin16 = common.make_pio_pin(res.pin16);
    let pin17 = common.make_pio_pin(res.pin17);
    let pin18 = common.make_pio_pin(res.pin18);
    let pin19 = common.make_pio_pin(res.pin19);
    let pin20 = common.make_pio_pin(res.pin20);
    let pin21 = common.make_pio_pin(res.pin21);
    let pin22 = common.make_pio_pin(res.pin22);
    let pin26 = common.make_pio_pin(res.pin26);
    let pin27 = common.make_pio_pin(res.pin27);
    let pin28 = common.make_pio_pin(res.pin28);

    let prg = pio_proc::pio_asm!(
        r#"
            .side_set 2 opt
            ;loop:
            ;    nop             side 0b11
            ;    jmp loop        side 0b11

            loop2:
                wait 1 gpio 0   side 0b10   ; Wait for analyzer-b to be ready
                in null, 11     side 0b11   ; Unset HALT
                wait 1 gpio 26  side 0b11   ; Wait for AS to be negated
                wait 0 gpio 26  side 0b11   ; Wait for AS
                in pins, 21     side 0b11   ; Read R/W + address
                ;in null, 11     side 0b10   ; Fill up the rest with zero
                // nop             side 0b10
                // nop             side 0b10
                // nop             side 0b10
                // nop             side 0b10
                push block      side 0b10   ; Push data and wait for it to be accepted
                jmp loop2       side 0b10
        "#
    );

    let mut config = PioConfig::default();
    config.use_program(&common.load_program(&prg.program), &[&pin27, &pin28]);
    config.clock_divider = (U56F8!(125_000_000) / (MHZ*2*1_000_000)).to_fixed();
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
    sm.set_pin_dirs(Direction::Out, &[&pin27, &pin28]);
    sm.set_pin_dirs(
        Direction::In,
        &[
            &pin0, &pin2, &pin3, &pin4, &pin5, &pin6, &pin7, &pin8, &pin9, &pin10, &pin11,
            &pin12, &pin13, &pin14, &pin15, &pin16, &pin17, &pin18, &pin19, &pin20, &pin21, &pin22,
            &pin26,
        ],
    );
    config.set_set_pins(&[&pin27, &pin28]);
    config.set_in_pins(&[
        &pin2, &pin3, &pin4, &pin5, &pin6, &pin7, &pin8, &pin9, &pin10, &pin11, &pin12,
        &pin13, &pin14, &pin15, &pin16, &pin17, &pin18, &pin19, &pin20, &pin21, &pin22,
    ]);
    config.fifo_join = FifoJoin::RxOnly;

    sm.set_config(&config);
    sm.set_enable(false);


    let prg2 = pio_proc::pio_asm!(
        r#"
            .side_set 2

            loop:
                nop         side 0b00
                jmp loop    side 0b00
        "#
    );

    let mut config2 = PioConfig::default();
    config2.use_program(&common.load_program(&prg2.program), &[&pin27, &pin28]);
    config2.clock_divider = (U56F8!(125_000_000) / (MHZ*2*1_000_000)).to_fixed();
    
    sm1.set_pin_dirs(Direction::Out, &[&pin27, &pin28]);
    config2.set_set_pins(&[&pin27, &pin28]);
    // config2.
    // config2.out_sticky = true;

    sm1.set_config(&config2);
    sm1.set_enable(true);

    let prg3 = pio_proc::pio_asm!(
        r#"
            .side_set 1

            loop:
                nop         side 0b1
                jmp loop    side 0b0
        "#
    );

    let mut config3 = PioConfig::default();
    config3.use_program(&common.load_program(&prg3.program), &[&pin1]);
    config3.clock_divider = (U56F8!(125_000_000) / (MHZ*2*1_000_000)).to_fixed();
    
    sm2.set_pin_dirs(Direction::Out, &[&pin1]);

    sm2.set_config(&config3);
    sm2.set_enable(true);


    let button = Input::new(res.pin24, Pull::Up);

    let mut count: u32 = 0;
    let mut was_pressed = true;
    let mut is_enabled = false;

    let mut dma_in_ref = res.dma.into_ref();
    let mut din = [0u32; 1];
    let dma_fut = async {
        loop {
            let res = select(sm.rx().dma_pull(dma_in_ref.reborrow(), &mut din), Timer::after_millis(1)).await;
            // defmt::info!("Pull");
            match res {
                Either::First(_) => {
                    // defmt::info!("received DMA");
                    SHARED.send(count).await;
                    SHARED.send(din[0]).await;
                    count = count.wrapping_add(1);
                }
                _ => (),
            }
            
            if button.is_low() && !was_pressed {
                was_pressed = true;
                is_enabled = !is_enabled;
                sm.set_enable(is_enabled);
                sm1.set_enable(!is_enabled);
                defmt::info!("sm {}", is_enabled);
            } else if button.is_high() && was_pressed {
                was_pressed = false;
            }
        }
    };
    dma_fut.await;

}

#[embassy_executor::task]
async fn usb_serial(usb: UsbResources) {
    let driver = Driver::new(usb.usb, Irqs);

    // Create embassy-usb Config
    let mut config = Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Embassy");
    config.product = Some("USB-serial example");
    config.serial_number = Some("12345678");
    config.max_power = 100;
    config.max_packet_size_0 = 64;

    // Required for windows compatibility.
    // https://developer.nordicsemi.com/nRF_Connect_SDK/doc/1.9.1/kconfig/CONFIG_CDC_ACM_IAD.html#help
    config.device_class = 0xEF;
    config.device_sub_class = 0x02;
    config.device_protocol = 0x01;
    config.composite_with_iads = true;

    // Create embassy-usb DeviceBuilder using the driver and config.
    // It needs some buffers for building the descriptors.
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut device_descriptor = [0; 256];
    let mut control_buf = [0; 64];

    let mut state = State::new();

    let mut builder = Builder::new(
        driver,
        config,
        &mut device_descriptor, // no msos descriptors
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut [], // no msos descriptors
        &mut control_buf,
    );

    // Create classes on the builder.
    let mut class = CdcAcmClass::new(&mut builder, &mut state, 64);

    // Build the builder.
    let mut usb = builder.build();

    // Run the USB device.
    let usb_fut = usb.run();

    // Do stuff with the class!
    let echo_fut = async {
        defmt::info!("Startup");
        loop {
            class.wait_connection().await;
            defmt::info!("Connected");
            loop {
                let byte = SHARED.receive().await;
                let bytes = byte.to_be_bytes();
                // defmt::info!("Write {}", bytes);
                let res = class.write_packet(&bytes).await;
                if let Err(embassy_usb_driver::EndpointError::Disabled) = res {
                    defmt::info!("Disconnected");
                    break;
                }
            }
        }
    };

    // Run everything concurrently.
    // If we had made everything `'static` above instead, we could do this using separate tasks instead.
    join(usb_fut, echo_fut).await;
}
