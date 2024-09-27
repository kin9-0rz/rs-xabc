use getset::Getters;
use scroll::Uleb128;

use crate::error;
use crate::str::ABCString;
use crate::tag_value::TaggedValue;
use crate::uint16_t;
use crate::uint32_t;
use crate::uint8_t;
use scroll::ctx;
use scroll::Pread;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MethodAccessFlags {
    Public = 0x0001,
    Private = 0x0002,
    Protected = 0x0004,
    Static = 0x0008,
    Final = 0x0010,
    Synchronized = 0x0020,
    Bridge = 0x0040,
    Varargs = 0x0080,
    Native = 0x0100,
    Abstract = 0x0400,
    Strict = 0x0800,
    Synthetic = 0x1000,
}

impl MethodAccessFlags {
    pub fn parse(value: u64) -> Vec<String> {
        let mut access_flags: Vec<String> = Vec::new();

        let flags = [
            MethodAccessFlags::Public,
            MethodAccessFlags::Private,
            MethodAccessFlags::Protected,
            MethodAccessFlags::Static,
            MethodAccessFlags::Final,
            MethodAccessFlags::Synchronized,
            MethodAccessFlags::Bridge,
            MethodAccessFlags::Varargs,
            MethodAccessFlags::Native,
            MethodAccessFlags::Abstract,
            MethodAccessFlags::Strict,
            MethodAccessFlags::Synthetic,
        ]
        .to_vec();

        for flag in flags {
            let x = flag as u64;
            if value & x != 0 {
                //println!("{:?}", flag);
                access_flags.push(format!("{:?}", flag));
            }
        }

        access_flags
    }
}

#[derive(Debug, Getters)]
#[get = "pub"]
pub struct Method {
    name: String,
    access_flags: Vec<String>,
}

impl<'a> ctx::TryFromCtx<'a, scroll::Endian> for Method {
    type Error = error::Error;
    fn try_from_ctx(source: &'a [u8], _: scroll::Endian) -> Result<(Self, usize), Self::Error> {
        println!(" {:?}", &source[0..2]);
        let mut off = 0;
        let class_idx = source.pread::<uint16_t>(0).unwrap();
        println!("Method -> class_idx: {}", class_idx);
        //let name = source.pread::<ABCString>(class_idx as usize).unwrap();
        //println!("Method -> name: {}", name);

        Ok((
            Method {
                name: String::new(),
                access_flags: Vec::new(),
            },
            source.len(),
        ))
    }
}
