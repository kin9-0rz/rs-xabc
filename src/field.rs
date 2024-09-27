use getset::Getters;
use scroll::ctx;
use scroll::Pread;
use scroll::Sleb128;
use scroll::Uleb128;

use crate::uint8_t;
use crate::{error, uint16_t, uint32_t};

#[derive(Debug, Getters, Default)]
#[get = "pub"]
pub struct Field {
    class_idx: uint16_t,
    /// ClassRegionIndex 的一个索引
    type_idx: uint16_t,
    /// 名字的偏移量，指向一个 String
    name_off: uint32_t,
    /// 它的值必须是 AccessFlag 的组合。
    access_flags: Vec<String>,
    // field_data: Vec<TaggedValue>,
    size: usize,
}

impl<'a> ctx::TryFromCtx<'a, scroll::Endian> for Field {
    type Error = error::Error;
    fn try_from_ctx(source: &'a [u8], _: scroll::Endian) -> Result<(Self, usize), Self::Error> {
        let class_idx = source.pread::<uint16_t>(0).unwrap();
        let type_idx = source.pread::<uint16_t>(2).unwrap();
        let name_off = source.pread::<uint32_t>(4).unwrap();

        let off = &mut 8;
        let access_flags = Uleb128::read(source, off).unwrap();
        let access_flags = FieldAccessFlag::parse(access_flags);

        // 解析 field_data
        // NOTE: 数据保存
        'l: loop {
            let tag_value = source.pread::<uint8_t>(*off).unwrap();
            *off += 1;
            match tag_value {
                0x00 => {
                    println!("NOTHING");
                    break 'l;
                }
                0x01 => {
                    let num = Sleb128::read(source, off).unwrap();
                    println!("int off: {:?}", off);
                    println!("INT_VALUE -> {}", num);
                }
                0x02 => {
                    let data = source.pread::<uint32_t>(*off).unwrap();
                    *off += 4;
                    println!("VALUE -> {}", data);
                }
                0x03 => {
                    let data = source.pread::<uint32_t>(*off).unwrap();
                    *off += 4;
                    println!("RUNTIME_ANNOTATIONS -> {}", data);
                }
                0x04 => {
                    let data = source.pread::<uint32_t>(*off).unwrap();
                    *off += 4;
                    println!("ANNOTATIONS -> {}", data);
                }
                0x05 => {
                    let data = source.pread::<uint32_t>(*off).unwrap();
                    *off += 4;
                    println!("RUNTIME_TYPE_ANNOTATION -> {}", data);
                }
                0x06 => {
                    let data = source.pread::<uint32_t>(*off).unwrap();
                    *off += 4;
                    println!("TYPE_ANNOTATION -> {}", data);
                }
                _ => {
                    println!("UNKNOWN: {}", tag_value);
                    break 'l;
                }
            }
        }

        let size = *off;

        Ok((
            Field {
                class_idx,
                type_idx,
                name_off,
                access_flags,
                // field_data: Vec::new(),
                size,
            },
            source.len(),
        ))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldAccessFlag {
    PUBLIC = 0x0001,
    PRIVATE = 0x0002,
    PROTECTED = 0x0004,
    STATIC = 0x0008,
    FINAL = 0x0010,
    VOLATILE = 0x0040,
    TRANSIENT = 0x0080,
    SYNTHETIC = 0x1000,
    ENUM = 0x4000,
}
impl FieldAccessFlag {
    pub fn parse(value: u64) -> Vec<String> {
        let mut access_flags: Vec<String> = Vec::new();

        let flags = [
            FieldAccessFlag::PUBLIC,
            FieldAccessFlag::PRIVATE,
            FieldAccessFlag::PROTECTED,
            FieldAccessFlag::STATIC,
            FieldAccessFlag::FINAL,
            FieldAccessFlag::VOLATILE,
            FieldAccessFlag::TRANSIENT,
            FieldAccessFlag::SYNTHETIC,
            FieldAccessFlag::ENUM,
        ]
        .to_vec();

        for flag in flags {
            let x = flag as u64;
            if value & x != 0 {
                access_flags.push(format!("{:?}", flag));
            }
        }

        access_flags
    }
}
