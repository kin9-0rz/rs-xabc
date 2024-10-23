use std::collections::HashMap;

use scroll::Pread;

use crate::{method, region::Region, string::ABCString, uint32_t};

// https://gitee.com/openharmony/arkcompiler_runtime_core/blob/master/libpandafile/literal_data_accessor.h#L32
#[allow(non_camel_case_types)]
pub enum LiteralTag {
    TAG_VALUE = 0x00,
    BOOL = 0x01,
    // int_8 and tagvalue have the same range for data representation.
    // INTEGER_8 = TAGVALUE,
    INTEGER = 0x02,
    FLOAT = 0x03,
    DOUBLE = 0x04,
    STRING = 0x05,
    METHOD = 0x06,
    GENERATORMETHOD = 0x07,
    ACCESSOR = 0x08,
    METHODAFFILIATE = 0x09,
    ARRAY_U1 = 0x0a,
    ARRAY_U8 = 0x0b,
    ARRAY_I8 = 0x0c,
    ARRAY_U16 = 0x0d,
    ARRAY_I16 = 0x0e,
    ARRAY_U32 = 0x0f,
    ARRAY_I32 = 0x10,
    ARRAY_U64 = 0x11,
    ARRAY_I64 = 0x12,
    ARRAY_F32 = 0x13,
    ARRAY_F64 = 0x14,
    ARRAY_STRING = 0x15,
    ASYNC_GENERATOR_METHOD = 0x16,
    LITERAL_BUFFER_INDEX = 0x17,
    LITERAL_ARRAY = 0x18,
    BUILTIN_TYPE_INDEX = 0x19,
    GETTER = 0x1a,
    SETTER = 0x1b,
    NULL_VALUE = 0xff,
    UNKNOWN = 0xee,
}

impl LiteralTag {
    pub fn from_u8(value: u8) -> Self {
        match value {
            0x00 => LiteralTag::TAG_VALUE,
            0x01 => LiteralTag::BOOL,
            0x02 => LiteralTag::INTEGER,
            0x03 => LiteralTag::FLOAT,
            0x04 => LiteralTag::DOUBLE,
            0x05 => LiteralTag::STRING,
            0x06 => LiteralTag::METHOD,
            0x07 => LiteralTag::GENERATORMETHOD,
            0x08 => LiteralTag::ACCESSOR,
            0x09 => LiteralTag::METHODAFFILIATE,
            0x0a => LiteralTag::ARRAY_U1,
            0x0b => LiteralTag::ARRAY_U8,
            0x0c => LiteralTag::ARRAY_I8,
            0x0d => LiteralTag::ARRAY_U16,
            0x0e => LiteralTag::ARRAY_I16,
            0x0f => LiteralTag::ARRAY_U32,
            0x10 => LiteralTag::ARRAY_I32,
            0x11 => LiteralTag::ARRAY_U64,
            0x12 => LiteralTag::ARRAY_I64,
            0x13 => LiteralTag::ARRAY_F32,
            0x14 => LiteralTag::ARRAY_F64,
            0x15 => LiteralTag::ARRAY_STRING,
            0x16 => LiteralTag::ASYNC_GENERATOR_METHOD,
            0x17 => LiteralTag::LITERAL_BUFFER_INDEX,
            0x18 => LiteralTag::LITERAL_ARRAY,
            0x19 => LiteralTag::BUILTIN_TYPE_INDEX,
            0x1a => LiteralTag::GETTER,
            0x1b => LiteralTag::SETTER,
            0xff => LiteralTag::NULL_VALUE,
            _ => LiteralTag::UNKNOWN,
        }
    }
}

// https://developer.huawei.com/consumer/cn/doc/harmonyos-guides-V5/arkts-bytecode-file-format-V5#literalarray
fn parse_literal_array(source: &[u8], offset: usize, region: &Region) -> String {
    // println!("parse_literal_array");
    let num_literals = source.pread_with::<uint32_t>(offset, scroll::LE).unwrap();

    let mut off = offset;
    off += 4;

    let mut result = String::new();

    let mut counter = 0;
    loop {
        // println!("num_literals: {}", num_literals);
        // println!("counter: {}", counter);
        if counter >= num_literals {
            // println!("num_literals: {}", num_literals);
            // println!("counter: {}", counter);
            break;
        }

        counter += 1;

        let tag_value = source.pread::<u8>(off).unwrap();
        off += 1;
        match LiteralTag::from_u8(tag_value) {
            LiteralTag::TAG_VALUE => {
                // let data = source.pread::<u8>(off).unwrap();
                // println!("TaggleValue: {}", data);
                off += 1;
            }
            LiteralTag::BOOL => {
                let data = source.pread::<u8>(off).unwrap();
                off += 1;
                // println!("Bool: {}", data);
                let s = format!("bool: {}, ", data);
                result += &s;
            }
            LiteralTag::INTEGER => {
                let data = source.pread::<u32>(off).unwrap();
                // let data = source.pread_with::<u32>(off, scroll::BE).unwrap();
                let s = format!("i32: 0x{:X}, ", data);
                result += &s;
                off += 4;
            }
            LiteralTag::FLOAT => {
                let data = source.pread_with::<u32>(off, scroll::BE).unwrap();
                let s = format!("f32: {}, ", f32::from_bits(data));
                result += &s;
                off += 4;
            }
            LiteralTag::DOUBLE => {
                let data = source.pread_with::<u64>(off, scroll::BE).unwrap();
                println!("f64: 0x{:X} ", data);
                let s = format!("f64: {}, ", f64::from_bits(data));
                result += &s;

                off += 8;
            }
            LiteralTag::STRING => {
                // println!("-> Parse String");
                let string_off = source.pread::<u32>(off).unwrap();
                let str = source.pread::<ABCString>(string_off as usize).unwrap();
                // println!("Str: {}", str.str());
                let s = format!("str: \"{}\", ", str.str());
                result += &s;

                off += 4;
            }
            LiteralTag::METHOD => {
                let method_off = source.pread::<uint32_t>(off).unwrap();
                let method = method::get_method_sign(source, method_off as usize, region);
                // println!("Method: {}", method);
                let s = format!("Method: {}, ", method);
                result += &s;

                off += 4;
            }
            LiteralTag::GENERATORMETHOD => {
                let _off = source.pread::<uint32_t>(off).unwrap();
                let s = format!("GeneratorMethod: {}, ", _off);
                result += &s;
                off += 4;
            }
            LiteralTag::ACCESSOR => {
                let data = source.pread::<u8>(off).unwrap();
                // println!("Accessor: {}", data);
                // println!("Accessor: {}", data);
                let s = format!("Accessor: {}, ", data);
                result += &s;
                off += 1;
            }
            LiteralTag::METHODAFFILIATE => {
                let data = source.pread::<u16>(off).unwrap();
                let s = format!("MethodAffiliate: {}, ", data);
                result += &s;
                off += 2;
            }
            LiteralTag::ARRAY_U1 => {
                off += 4;
                // println!("ArrayU1: {}", tag_value);
            }
            LiteralTag::ARRAY_U8 => {
                // println!("ArrayU8: {}", tag_value);
                off += 4;
            }
            LiteralTag::ARRAY_I8 => {
                // println!("ArrayI8: {}", tag_value);
                off += 4;
            }
            LiteralTag::ARRAY_U16 => {
                // println!("ArrayU16: {}", tag_value);
                off += 4;
            }
            LiteralTag::ARRAY_I16 => {
                // println!("ArrayI16: {}", tag_value);
                off += 4;
            }
            LiteralTag::ARRAY_U32 => {
                // println!("ArrayU32: {}", tag_value);
                off += 4;
            }
            LiteralTag::ARRAY_I32 => {
                // println!("ArrayI32: {}", tag_value);
                off += 4;
            }
            LiteralTag::ARRAY_U64 => {
                // println!("ArrayU64: {}", tag_value);
                off += 4;
            }
            LiteralTag::ARRAY_I64 => {
                // println!("ArrayI64: {}", tag_value);
                off += 4;
            }
            LiteralTag::ARRAY_F32 => {
                // println!("ArrayF32: {}", tag_value);
                off += 4;
            }
            LiteralTag::ARRAY_F64 => {
                // println!("ArrayF64: {}", tag_value);
                off += 8;
            }
            LiteralTag::ARRAY_STRING => {
                // TODO: 一个字符串数组
                off += 4;
            }
            LiteralTag::ASYNC_GENERATOR_METHOD => {
                println!("ArrayGeneratorMethod: {}", tag_value);
                off += 4;
            }
            LiteralTag::LITERAL_BUFFER_INDEX => {
                println!("LiteralBufferIndex: {}", tag_value);
                off += 4;
            }
            LiteralTag::LITERAL_ARRAY => {
                println!("LiteralArr: {}", tag_value);
                off += 4;
            }
            LiteralTag::BUILTIN_TYPE_INDEX => {
                println!("BuiltinTypeIndex: {}", tag_value);
                off += 1;
            }
            LiteralTag::GETTER => {
                let data = source.pread::<uint32_t>(off).unwrap();
                let s = format!("Getter: 0x{:X}, ", data);
                result += &s;
                off += 4;
            }
            LiteralTag::SETTER => {
                println!("Setter: {}", tag_value);
                off += 4;
            }
            LiteralTag::NULL_VALUE => {
                println!("NullValue: {}", tag_value);
                off += 1;
            }
            LiteralTag::UNKNOWN => {
                // println!("未知的Tag: 0x{:X}", tag_value);
                break;
            }
        }
    }

    result
}

pub fn parse_literal_array_index(
    source: &[u8],
    offset: uint32_t,
    num_literals: uint32_t,
    regions: &[Region],
) -> HashMap<usize, String> {
    let mut off = offset as usize;
    let mut literal_array_map: HashMap<usize, String> = HashMap::new();

    for _ in 0..num_literals {
        let array_off = source.pread::<uint32_t>(off).unwrap();
        off += 4;

        let mut region: Option<&Region> = None;
        for item in regions.iter() {
            if item.is_here(array_off as usize) {
                region = Some(item);
                break;
            }
        }

        if region.is_none() {
            println!("region not found");
            continue;
        }

        let region = region.unwrap();

        let literal = parse_literal_array(source, array_off as usize, region);

        println!("{} -> {}", array_off, literal);

        literal_array_map.insert(array_off as usize, literal);
    }

    literal_array_map
}
