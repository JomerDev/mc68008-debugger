use std::fmt::Display;

use packed_struct::prelude::*;

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

#[derive(PackedStruct)]
#[packed_struct(bit_numbering = "msb0")]
pub struct AnalyzerA {
    #[packed_field(endian = "msb")]
    num: u32,
    #[packed_field(bits = "32", ty = "enum")]
    rw: RW,
    #[packed_field(bits = "33..=34", ty = "enum")]
    interrupt_level: InterruptLevel,
    #[packed_field(bits = "35..=55", endian = "msb")]
    address: Integer<u32, packed_bits::Bits<20>>,
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
    let a = AnalyzerA {
        num: 0,
        rw: RW::Read,
        interrupt_level: InterruptLevel::A,
        address: 0x60000.into(),
    };

    println!("{}", format_data(a));
}
