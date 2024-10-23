pub mod abc;
pub mod bytecode;
pub mod class;
pub mod code;
mod error;
pub mod field;
pub mod header;
pub mod literal;
pub mod lnp;
pub mod method;
pub mod region;
pub mod source;
pub mod string;

use scroll::{Sleb128, Uleb128};

/// 8-bit 无符号整数
#[allow(non_camel_case_types)]
pub type uint8_t = u8;
/// 16-bit无符号整数，采用小端字节序。
#[allow(non_camel_case_types)]
pub type uint16_t = u16;
/// 32-bit无符号整数，采用小端字节序。
#[allow(non_camel_case_types)]
pub type uint32_t = u32;
/// leb128编码的无符号整数
#[allow(non_camel_case_types)]
pub type uleb128_t = Uleb128;
/// leb128编码的有符号整数。
#[allow(non_camel_case_types)]
pub type sleb128_t = Sleb128;

// A `Result` of `T` or an error of `error::Error`
//pub type Result<T> = std::result::Result<T, error::Error>;
