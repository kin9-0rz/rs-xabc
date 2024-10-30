use getset::Getters;
use scroll::Uleb128;
use tracing::debug;

use crate::error;
use crate::region::Region;
use crate::string::ABCString;
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
                access_flags.push(format!("{:?}", flag));
            }
        }

        access_flags
    }
}

#[derive(Debug, Getters, Default)]
#[get = "pub"]
pub struct MethodData {
    /// 指向方法的 Code 对象的偏移量。
    code_off: uint32_t,
    source_lang: uint8_t,
    runtime_annotation_off: uint32_t,
    runtime_param_annotation_off: uint32_t,
    debug_info_off: uint32_t,
    annotation_off: uint32_t,
    param_annotation_off: uint32_t,
    type_annotation_off: uint32_t,
    runtime_type_annotation_off: uint32_t,
}

#[derive(Debug, Getters, Default)]
#[get = "pub"]
pub struct Method {
    /// 类的索引
    class_idx: uint16_t,
    proto_idx: uint16_t,
    /// 名字的偏移量，指向一个 String
    name_off: uint32_t,

    /// 它的值必须是 AccessFlag 的组合。
    access_flags: Vec<String>,
    // method_data: Vec,
    size: usize,
    method_data: MethodData,
}

impl Method {}

// TODO: 方法签名还不完整
pub fn get_method_sign(source: &[u8], offset: usize, region: &Region) -> String {
    let mut name = String::new();
    let mut off = offset;
    let class_idx = source.pread::<uint16_t>(off).unwrap();
    off += 2;
    let class_name = region.get_class_name(class_idx as usize).to_string();
    name += &class_name;
    name += "->";

    // TODO: 获取方法签名，获取参数
    let _proto_idx = source.pread::<uint16_t>(off).unwrap();
    off += 2;

    let name_idx = source.pread::<uint32_t>(off).unwrap();
    let method_name = source
        .pread::<ABCString>(name_idx as usize)
        .unwrap()
        .to_string();

    name += &method_name;

    name
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

        let mut method_data = MethodData::default();
        // NOTE: 数据保存
        'l: loop {
            let tag_value = source.pread::<u8>(*off).unwrap();
            *off += 1;

            match tag_value {
                0x00 => {
                    tracing::debug!("NOTHING");
                    break 'l;
                }
                0x01 => {
                    let code_off = source.pread::<uint32_t>(*off).unwrap();
                    *off += 4;
                    method_data.code_off = code_off;
                    debug!("CODE {:?}", code_off);
                }
                0x02 => {
                    let data = source.pread::<u8>(*off).unwrap();
                    *off += 1;
                    method_data.source_lang = data;
                    debug!("SOURCE_LANG {:?}", data);
                }
                0x03 => {
                    let data = source.pread::<uint32_t>(*off).unwrap();
                    *off += 4;
                    method_data.runtime_annotation_off = data;
                    debug!("RUNTIME_ANNOTATION {:?}", data);
                }
                0x04 => {
                    let data = source.pread::<uint32_t>(*off).unwrap();
                    *off += 4;
                    method_data.runtime_param_annotation_off = data;
                    debug!("RUNTIME_PARAM_ANNOTATION {:?}", data);
                }
                0x05 => {
                    let data = source.pread::<uint32_t>(*off).unwrap();
                    *off += 4;
                    method_data.debug_info_off = data;
                    debug!("DEBUG_INFO {:?}", data);
                }
                0x06 => {
                    let data = source.pread::<uint32_t>(*off).unwrap();
                    *off += 4;
                    method_data.annotation_off = data;
                    debug!("ANNOTATION {:?}", data);
                }
                0x07 => {
                    let data = source.pread::<uint32_t>(*off).unwrap();
                    *off += 4;
                    method_data.param_annotation_off = data;
                    debug!("PARAM_ANNOTATION {:?}", data);
                }
                0x08 => {
                    let data = source.pread::<uint32_t>(*off).unwrap();
                    *off += 4;
                    method_data.type_annotation_off = data;
                    debug!("TYPE_ANNOTATION {:?}", data);
                }
                0x09 => {
                    let data = source.pread::<uint32_t>(*off).unwrap();
                    *off += 4;
                    method_data.runtime_type_annotation_off = data;
                    debug!("RUNTIME_TYPE_ANNOTATION {:?}", data);
                }
                _ => {
                    // FIXME: 这种情况是不可能出现，一定有问题。
                    tracing::error!("Method Data: UNKNOWN 0x{:02X}", tag_value);
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
                method_data,
                size,
            },
            source.len(),
        ))
    }
}
