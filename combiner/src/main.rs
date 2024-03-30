use std::{fmt::Display, io::Read, time::Duration};

use packed_struct::prelude::*;
use serialport5::SerialPort;

#[derive(PrimitiveEnum_u8, Debug, Clone, Copy)]
pub enum RW {
    Read = 0,
    Write = 1,
}

impl Display for RW {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            RW::Read => write!(f, "READ "),
            RW::Write => write!(f, "WRITE"),
        }
    }
}

#[derive(PrimitiveEnum_u8, Debug, Clone, Copy)]
pub enum InterruptLevel {
    A = 0,
    B = 1,
    C = 2,
    D = 3,
}

#[derive(PackedStruct, Debug)]
#[packed_struct(bit_numbering = "msb0", endian = "msb")]
pub struct AnalyzerA {
    #[packed_field(endian = "msb", bits = "0..=31")]
    num: u32,
    _reserved: ReservedZero<packed_bits::Bits::<11>>,
    
    // #[packed_field(bits = "41..=42", ty = "enum")]
    // interrupt_level: InterruptLevel,
    #[packed_field(bits = "43..63", endian = "msb")]
    address: Integer<u32, packed_bits::Bits<20>>,
    #[packed_field(bits = "63", ty = "enum")]
    rw: RW,
}

#[derive(PackedStruct)]
#[packed_struct(bit_numbering = "msb0")]
pub struct AnalyzerB {
    #[packed_field(endian = "msb")]
    num: u32,
    #[packed_field(endian = "msb")]
    data: u8,
    // #[packed_field(bits = "32", ty = "enum")]
    // rw: RW,
    // #[packed_field(bits = "33..=34", ty = "enum")]
    // interrupt_level: InterruptLevel,
    // #[packed_field(bits = "35..=55", endian = "msb")]
    // address: Integer<u32, packed_bits::Bits<20>>,
}

// pub fn get_address_area(address: u32) -> String {
//     match address {
//         0..=0x1FFFF => "ROM".to_owned(),
//         0x20000..=0x3FFFF => "RAM".to_owned(),
//         0x40000..=0x7FFFF => "MC68681".to_owned(),
//         0x80000..=0x83FFF => "LCD".to_owned(),
//         0x84000..=0x87FFF => "574_DA".to_owned(),
//         _ => "???".to_owned(),
//     }
// }

pub fn get_address_area(address: u32) -> String {
    match address {
        0..=0x0FFFF => "ROM".to_owned(),
        0x10000..=0x1FFFF => "RAM".to_owned(),
        0x20000..=0x2FFFF => "MC68681".to_owned(),
        0x30000..=0x3FFFF => "LCD".to_owned(),
        0x40000..=0x4FFFF => "LED".to_owned(),
        _ => "???".to_owned(),
    }
}

pub fn format_data(a: AnalyzerA) -> String {
    format!(
        "{} | {:07x} | {: >10} |",
        a.rw,
        a.address.to_le(),
        get_address_area(a.address.to_le())
    )
}

fn main() {
    // let a = AnalyzerA {
    //     num: 0,
    //     rw: RW::Read,
    //     interrupt_level: InterruptLevel::A,
    //     address: 0x60000.into(),
    // };

    // println!("{}", format_data(a));

    let ports = serialport5::available_ports().expect("No ports found!");
    for p in ports {
        println!("{}", p.port_name);
    }

    let mut port = SerialPort::builder()
        .baud_rate(921_600)
        .read_timeout(None)
        .open("COM8")
        .expect("Failed to open port");

    let mut serial_buf: [u8; 8] = [0; 8];
    loop {
        let res = port.read(&mut serial_buf);
        match res {
            Err(e) => match e {
                _ => continue,

            },
            Ok(_) => ()
        }

        println!("{:?}", serial_buf);


        let data = AnalyzerA::unpack(&serial_buf);

        if let Ok(d) = data {
            println!("{}", format_data(d));
        } else {
            println!("{}", data.expect_err(""));
        }

    }
}
