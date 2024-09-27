use getset::Getters;
use scroll::Uleb128;

use crate::error;
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

#[derive(Debug, Getters, Default)]
#[get = "pub"]
pub struct Method {
    class_idx: uint16_t,
    proto_idx: uint16_t,
    /// 名字的偏移量，指向一个 String
    name_off: uint32_t,
    /// 它的值必须是 AccessFlag 的组合。
    access_flags: Vec<String>,
    // method_data: Vec,
    size: usize,
}

impl<'a> ctx::TryFromCtx<'a, scroll::Endian> for Method {
    type Error = error::Error;
    fn try_from_ctx(source: &'a [u8], _: scroll::Endian) -> Result<(Self, usize), Self::Error> {
        let class_idx = source.pread::<uint16_t>(0).unwrap();
        let proto_idx = source.pread::<uint16_t>(2).unwrap();
        let name_off = source.pread::<uint32_t>(4).unwrap();

        let off = &mut 8;
        let access_flags = Uleb128::read(source, off).unwrap();
        let access_flags = MethodAccessFlags::parse(access_flags);

        // 解析 method_data
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
                    let code_off = source.pread::<uint32_t>(*off).unwrap();
                    *off += 4;
                    // TODO: 解析代码？
                    println!("CODE {:?}", code_off);
                }
                0x02 => {
                    let data = source.pread::<uint8_t>(*off).unwrap();
                    *off += 1;
                    println!("SOURCE_LANG {:?}", data);
                }
                0x03 => {
                    let data = source.pread::<uint32_t>(*off).unwrap();
                    *off += 4;
                    println!("RUNTIME_ANNOTATION {:?}", data);
                }
                0x04 => {
                    let data = source.pread::<uint32_t>(*off).unwrap();
                    *off += 4;
                    println!("RUNTIME_ANNOTATION_ANNOTATION {:?}", data);
                }
                0x05 => {
                    let data = source.pread::<uint32_t>(*off).unwrap();
                    *off += 4;
                    println!("DEBUG_INFO {:?}", data);
                }
                0x06 => {
                    let data = source.pread::<uint32_t>(*off).unwrap();
                    *off += 4;
                    println!("ANNOTATION {:?}", data);
                }
                0x07 => {
                    let data = source.pread::<uint32_t>(*off).unwrap();
                    *off += 4;
                    println!("PARAM_ANNOTATION {:?}", data);
                }
                0x08 => {
                    let data = source.pread::<uint32_t>(*off).unwrap();
                    *off += 4;
                    println!("TYPE_ANNOTATION {:?}", data);
                }
                0x09 => {
                    let data = source.pread::<uint32_t>(*off).unwrap();
                    *off += 4;
                    println!("RUNTIME_TYPE_ANNOTATION {:?}", data);
                }
                _ => {
                    println!("UNKNOWN");
                }
            }
        }

        let size = *off;

        Ok((
            Method {
                class_idx,
                proto_idx,
                name_off,
                access_flags,
                // method_data: Vec::new(),
                size,
            },
            source.len(),
        ))
    }
}
