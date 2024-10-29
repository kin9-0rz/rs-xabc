/// 解析字节码
use std::collections::HashMap;

use getset::Getters;
use scroll::Pread;

use crate::{
    code::Code,
    method::{self},
    region::Region,
    string::ABCString,
};

// https://developer.huawei.com/consumer/cn/doc/harmonyos-guides-V5/arkts-bytecode-fundamentals-V5#字节码格式说明
/// 格式组成的基本单元
#[derive(Debug, Clone)]
pub enum FormatUnit {
    Opcode,
    PrefixOpcode,
    /// 方舟运行时内部使用的8位保留数字，此处提及仅为完整展示指令格式，开发者无需关注。
    RR,
    RRRR,
    /// 特殊的4位寄存器，2个一起出现
    V4V4,
    V8,
    V16,
    /// 4位立即数，2个一起出现
    Imm4Imm4,
    IMM8,
    IMM16,
    IMM32,
    IMM64,
    /// 16位ID, ID16,
    LiteralID,
    StringID,
    MethodID,
}

#[derive(Debug, Clone, Getters)]
#[get = "pub"]
pub struct ByteCodeFormat {
    name: String,
    formats: Vec<FormatUnit>,
    instruction: Vec<u8>,
}

impl ByteCodeFormat {
    pub fn new(name: String, formats: Vec<FormatUnit>) -> Self {
        Self {
            name,
            formats,
            instruction: vec![],
        }
    }

    pub fn set_instruction(&mut self, instruction: Vec<u8>) {
        self.instruction = instruction;
    }

    pub fn get_size(&self) -> usize {
        let mut size = 0;
        for unit in &self.formats {
            match unit {
                FormatUnit::Opcode => size += 1,
                FormatUnit::PrefixOpcode => size += 2,
                FormatUnit::V4V4 => size += 1,
                FormatUnit::V8 => size += 1,
                FormatUnit::V16 => size += 2,
                FormatUnit::LiteralID => size += 2,
                FormatUnit::StringID => size += 2,
                FormatUnit::MethodID => size += 2,
                FormatUnit::Imm4Imm4 => size += 1,
                FormatUnit::IMM8 => size += 1,
                FormatUnit::IMM16 => size += 2,
                FormatUnit::IMM32 => size += 4,
                FormatUnit::IMM64 => size += 8,
                FormatUnit::RR => size += 1,
                FormatUnit::RRRR => size += 2,
            }
        }

        size
    }

    pub fn parse(
        &self,
        instructions: &Vec<u8>,
        offset: usize,
        region: &Region,
        source: &[u8],
        literal_array_map: &HashMap<usize, String>,
    ) -> usize {
        let mut offset = offset;
        let opcode_name = self.name.split_whitespace().collect::<Vec<&str>>()[0];
        let mut strx = format!("{} ", opcode_name);
        let mut raw = String::from("0x");

        for unit in &self.formats {
            match unit {
                FormatUnit::Opcode => {
                    let data = instructions.pread::<u8>(offset).unwrap();
                    offset += 1;
                    raw += &format!("{:02X}", data);
                }
                FormatUnit::PrefixOpcode => {
                    // 大小端互换
                    let data = instructions.pread_with::<u16>(offset, scroll::BE).unwrap();
                    offset += 2;
                    raw += &format!("{:04X}", data);
                }

                FormatUnit::V4V4 => {
                    let data = instructions.pread::<u8>(offset).unwrap();
                    raw += &format!("{:02X}", data);
                    offset += 1;
                    let high_nibble = (data >> 4) & 0b1111;
                    let low_nibble = data & 0b1111;
                    strx += &format!("v{} v{}", low_nibble, high_nibble);
                }
                FormatUnit::V8 => {
                    let data = instructions.pread::<u8>(offset).unwrap();
                    raw += &format!("{:02X}", data);
                    offset += 1;
                    strx += &format!("v{} ", data);
                }
                FormatUnit::V16 => {
                    let data = instructions.pread::<u16>(offset).unwrap();
                    raw += &format!("{:04X}", data);
                    offset += 2;
                    strx += &format!("v{} ", data);
                }
                // NOTE: 这个是索引，不是偏移
                FormatUnit::LiteralID => {
                    let data = instructions.pread_with::<u16>(offset, scroll::LE).unwrap();
                    raw += &format!("{:04X}", data);
                    offset += 2;

                    let array_off = *region.get_msl_offset(data as usize);
                    let array_off = array_off as usize;
                    let x = literal_array_map.get(&array_off).unwrap();
                    strx += &format!("{{ {} }}", x);
                }
                FormatUnit::StringID => {
                    let data = instructions.pread_with::<u16>(offset, scroll::LE).unwrap();
                    raw += &format!("{:04X}", data);
                    offset += 2;

                    let string_offset = region.get_msl_offset(data as usize);
                    let x = source
                        .as_ref()
                        .pread::<ABCString>(*string_offset as usize)
                        .unwrap()
                        .str();
                    strx += &format!("\"{}\" ", x);
                }
                FormatUnit::MethodID => {
                    let data = instructions.pread_with::<u16>(offset, scroll::LE).unwrap();
                    raw += &format!("{:04X}", data);
                    offset += 2;
                    let method_offset = region.get_msl_offset(data as usize);

                    let method_sign =
                        method::get_method_sign(source, *method_offset as usize, region);

                    strx += &method_sign.to_string();
                    strx += " ";
                }
                FormatUnit::Imm4Imm4 => {
                    let data = instructions.pread::<u8>(offset).unwrap();
                    raw += &format!("{:02X}", data);
                    offset += 1;
                    strx += "Imm4Imm4";
                    strx += &format!("+{} ", data);
                }
                FormatUnit::IMM8 => {
                    let data = instructions.pread::<u8>(offset).unwrap();
                    raw += &format!("{:02X}", data);
                    offset += 1;
                    strx += &format!("+{} ", data);
                }
                FormatUnit::IMM16 => {
                    let data = instructions.pread::<u16>(offset).unwrap();
                    raw += &format!("{:04X}", data);
                    offset += 2;
                    strx += &format!("+{} ", data);
                }
                FormatUnit::IMM32 => {
                    let data = instructions.pread::<u32>(offset).unwrap();
                    raw += &format!("{:08X}", data);
                    offset += 4;
                    strx += "IMM32";
                    strx += &format!("+{} ", data);
                }
                FormatUnit::IMM64 => {
                    let data = instructions.pread::<u64>(offset).unwrap();
                    raw += &format!("{:16X}", data);
                    offset += 8;
                    strx += "IMM64";
                    strx += &format!("+{} ", data);
                }
                FormatUnit::RR => {
                    let _ = instructions.pread::<u8>(offset).unwrap();
                    offset += 1;
                }
                FormatUnit::RRRR => {
                    let _ = instructions.pread::<u16>(offset).unwrap();
                    offset += 2;
                }
            }
        }

        // println!("{} : {} : {}", raw, self.name, strx);
        println!("{} : {}", raw, strx);

        offset
    }
}

/// 字节码解析器
pub struct BytecodeParser {
    // 存放字节码的字节码表
    pub opcode_table: HashMap<u16, ByteCodeFormat>,
    // 存放前缀字节码的字节码表
    pub prefix_opcode_table: HashMap<u16, ByteCodeFormat>,
}

fn init_opcode_map() -> HashMap<u16, ByteCodeFormat> {
    let opcode_vec = vec![
        // 0x00 	NONE 	ldundefined
        (
            0x0,
            ByteCodeFormat::new("ldundefined".to_owned(), vec![FormatUnit::Opcode]),
        ),
        // 0x01 	NONE 	ldnull
        (
            0x1,
            ByteCodeFormat::new("ldnull".to_owned(), vec![FormatUnit::Opcode]),
        ),
        // 0x02 	NONE 	ldtrue
        (
            0x2,
            ByteCodeFormat::new("ldtrue".to_owned(), vec![FormatUnit::Opcode]),
        ),
        // 0x03 	NONE 	ldfalse
        (
            0x3,
            ByteCodeFormat::new("ldfalse".to_owned(), vec![FormatUnit::Opcode]),
        ),
        // 0x04 	NONE 	createemptyobject
        (
            0x4,
            ByteCodeFormat::new("createemptyobject".to_owned(), vec![FormatUnit::Opcode]),
        ),
        // 0x05 	IMM8 	createemptyarray RR
        (
            0x5,
            ByteCodeFormat::new(
                "createemptyarray".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RR],
            ),
        ),
        // 0x06 	IMM8_ID16 	createarraywithbuffer RR, @AAAA
        (
            0x6,
            ByteCodeFormat::new(
                "createarraywithbuffer".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RR, FormatUnit::LiteralID],
            ),
        ),
        // 0x07 	IMM8_ID16 	createobjectwithbuffer RR, @AAAA
        (
            0x07,
            ByteCodeFormat::new(
                "createobjectwithbuffer".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RR, FormatUnit::LiteralID],
            ),
        ),
        // 0x08 	IMM8_IMM8_V8 	newobjrange RR, +AA, vBB
        (
            0x08,
            ByteCodeFormat::new(
                "newobjrange RR, +AA, vBB".to_owned(),
                vec![
                    FormatUnit::Opcode,
                    FormatUnit::RR,
                    FormatUnit::IMM8,
                    FormatUnit::V8,
                ],
            ),
        ),
        // 0x09 	IMM8 	newlexenv +AA
        (
            0x09,
            ByteCodeFormat::new(
                "newlexenv".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::IMM8],
            ),
        ),
        // 0x0a 	IMM8_V8 	add2 RR, vAA
        (
            0x0a,
            ByteCodeFormat::new(
                "add2 RR, vAA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RR, FormatUnit::V8],
            ),
        ),
        // 0x0b 	IMM8_V8 	sub2 RR, vAA
        (
            0x0b,
            ByteCodeFormat::new(
                "sub2 RR, vAA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RR, FormatUnit::V8],
            ),
        ),
        // 0x0c 	IMM8_V8 	mul2 RR, vAA
        (
            0x0c,
            ByteCodeFormat::new(
                "mul2 RR, vAA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RR, FormatUnit::V8],
            ),
        ),
        // 0x0d 	IMM8_V8 	div2 RR, vAA
        (
            0x0d,
            ByteCodeFormat::new(
                "div2 RR, vAA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RR, FormatUnit::V8],
            ),
        ),
        // 0x0e 	IMM8_V8 	mod2 RR, vAA
        (
            0x0e,
            ByteCodeFormat::new(
                "mod2 RR, vAA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RR, FormatUnit::V8],
            ),
        ),
        // 0x0f 	IMM8_V8 	eq RR, vAA
        (
            0x0f,
            ByteCodeFormat::new(
                "eq RR, vAA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RR, FormatUnit::V8],
            ),
        ),
        // 0x10 	IMM8_V8 	noteq RR, vAA
        (
            0x10,
            ByteCodeFormat::new(
                "noteq RR, vAA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RR, FormatUnit::V8],
            ),
        ),
        // 0x11 	IMM8_V8 	less RR, vAA
        (
            0x11,
            ByteCodeFormat::new(
                "less RR, vAA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RR, FormatUnit::V8],
            ),
        ),
        // 0x12 	IMM8_V8 	lesseq RR, vAA
        (
            0x12,
            ByteCodeFormat::new(
                "lesseq RR, vAA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RR, FormatUnit::V8],
            ),
        ),
        // 0x13 	IMM8_V8 	greater RR, vAA
        (
            0x13,
            ByteCodeFormat::new(
                "greater RR, vAA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RR, FormatUnit::V8],
            ),
        ),
        // 0x14 	IMM8_V8 	greatereq RR, vAA
        (
            0x13,
            ByteCodeFormat::new(
                "greatereq RR, vAA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RR, FormatUnit::V8],
            ),
        ),
        // 0x15 	IMM8_V8 	shl2 RR, vAA
        (
            0x15,
            ByteCodeFormat::new(
                "shl2 RR, vAA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RR, FormatUnit::V8],
            ),
        ),
        // 0x16 	IMM8_V8 	shr2 RR, vAA
        (
            0x16,
            ByteCodeFormat::new(
                "shr2 RR, vAA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RR, FormatUnit::V8],
            ),
        ),
        // 0x17 	IMM8_V8 	ashr2 RR, vAA
        (
            0x17,
            ByteCodeFormat::new(
                "ashr2 RR, vAA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RR, FormatUnit::V8],
            ),
        ),
        // 0x18 	IMM8_V8 	and2 RR, vAA
        (
            0x18,
            ByteCodeFormat::new(
                "and2 RR, vAA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RR, FormatUnit::V8],
            ),
        ),
        // 0x19 	IMM8_V8 	or2 RR, vAA
        (
            0x19,
            ByteCodeFormat::new(
                "or2 RR, vAA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RR, FormatUnit::V8],
            ),
        ),
        // 0x1a 	IMM8_V8 	xor2 RR, vAA
        (
            0x1a,
            ByteCodeFormat::new(
                "xor2 RR, vAA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RR, FormatUnit::V8],
            ),
        ),
        // 0x1b 	IMM8_V8 	exp RR, vAA
        (
            0x1b,
            ByteCodeFormat::new(
                "exp RR, vAA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RR, FormatUnit::V8],
            ),
        ),
        // 0x1c 	IMM8 	typeof RR
        (
            0x1c,
            ByteCodeFormat::new(
                "typeof RR".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RR],
            ),
        ),
        // 0x1d 	IMM8 	tonumber RR
        (
            0x1d,
            ByteCodeFormat::new(
                "tonumber RR".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RR],
            ),
        ),
        // 0x1e 	IMM8 	tonumeric RR
        (
            0x1e,
            ByteCodeFormat::new(
                "tonumeric RR".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RR],
            ),
        ),
        // 0x1f 	IMM8 	neg RR
        (
            0x1f,
            ByteCodeFormat::new(
                "neg RR".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RR],
            ),
        ),
        // 0x20 	IMM8 	not RR
        (
            0x20,
            ByteCodeFormat::new(
                "not RR".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RR],
            ),
        ),
        // 0x21 	IMM8 	inc RR
        (
            0x21,
            ByteCodeFormat::new(
                "inc RR".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RR],
            ),
        ),
        // 0x22 	IMM8 	dec RR
        (
            0x22,
            ByteCodeFormat::new(
                "dec RR".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RR],
            ),
        ),
        // 0x23 	NONE 	istrue
        (
            0x23,
            ByteCodeFormat::new(
                "istrue".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RR],
            ),
        ),
        // 0x24 	NONE 	isfalse
        (
            0x24,
            ByteCodeFormat::new(
                "isfalse".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RR],
            ),
        ),
        // 0x25 	IMM8_V8 	isin RR, vAA
        (
            0x25,
            ByteCodeFormat::new(
                "isin RR, vAA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RR, FormatUnit::V8],
            ),
        ),
        // 0x26 	IMM8_V8 	instanceof RR, vAA
        (
            0x26,
            ByteCodeFormat::new(
                "instanceof RR, vAA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RR, FormatUnit::V8],
            ),
        ),
        // 0x27 	IMM8_V8 	strictnoteq RR, vAA
        (
            0x27,
            ByteCodeFormat::new(
                "strictnoteq RR, vAA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RR, FormatUnit::V8],
            ),
        ),
        // 0x28 	IMM8_V8 	stricteq RR, vAA
        (
            0x28,
            ByteCodeFormat::new(
                "stricteq RR, vAA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RR, FormatUnit::V8],
            ),
        ),
        // 0x29 	IMM8 	callarg0 RR
        (
            0x29,
            ByteCodeFormat::new(
                "callarg0 RR".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RR],
            ),
        ),
        // 0x2a 	IMM8_V8 	callarg1 RR, vAA
        (
            0x2a,
            ByteCodeFormat::new(
                "callarg1 RR, vAA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RR, FormatUnit::V8],
            ),
        ),
        // 0x2b 	IMM8_V8_V8 	callargs2 RR, vAA, vBB
        (
            0x2b,
            ByteCodeFormat::new(
                "callargs2 RR, vAA, vBB".to_owned(),
                vec![
                    FormatUnit::Opcode,
                    FormatUnit::RR,
                    FormatUnit::V8,
                    FormatUnit::V8,
                ],
            ),
        ),
        // 0x2c 	IMM8_V8_V8_V8 	callargs3 RR, vAA, vBB, vCC
        (
            0x2c,
            ByteCodeFormat::new(
                "callargs3 RR, vAA, vBB, vCC".to_owned(),
                vec![
                    FormatUnit::Opcode,
                    FormatUnit::RR,
                    FormatUnit::V8,
                    FormatUnit::V8,
                    FormatUnit::V8,
                ],
            ),
        ),
        // 0x2d 	IMM8_V8 	callthis0 RR, vAA
        (
            0x2d,
            ByteCodeFormat::new(
                "callthis0 RR, vAA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RR, FormatUnit::V8],
            ),
        ),
        // 0x2e 	IMM8_V8_V8 	callthis1 RR, vAA, vBB
        (
            0x2e,
            ByteCodeFormat::new(
                "callthis1 RR, vAA, vBB".to_owned(),
                vec![
                    FormatUnit::Opcode,
                    FormatUnit::RR,
                    FormatUnit::V8,
                    FormatUnit::V8,
                ],
            ),
        ),
        // 0x2f 	IMM8_V8_V8_V8 	callthis2 RR, vAA, vBB, vCC
        (
            0x2f,
            ByteCodeFormat::new(
                "callthis2 RR, vAA, vBB, vCC".to_owned(),
                vec![
                    FormatUnit::Opcode,
                    FormatUnit::RR,
                    FormatUnit::V8,
                    FormatUnit::V8,
                    FormatUnit::V8,
                ],
            ),
        ),
        // 0x30 	IMM8_V8_V8_V8_V8 	callthis3 RR, vAA, vBB, vCC, vDD
        (
            0x30,
            ByteCodeFormat::new(
                "callthis3 RR, vAA, vBB, vCC, vDD".to_owned(),
                vec![
                    FormatUnit::Opcode,
                    FormatUnit::RR,
                    FormatUnit::V8,
                    FormatUnit::V8,
                    FormatUnit::V8,
                    FormatUnit::V8,
                ],
            ),
        ),
        // 0x31 	IMM8_IMM8_V8 	callthisrange RR, +AA, vBB
        (
            0x31,
            ByteCodeFormat::new(
                "callthisrange RR, +AA, vBB".to_owned(),
                vec![
                    FormatUnit::Opcode,
                    FormatUnit::RR,
                    FormatUnit::IMM8,
                    FormatUnit::V8,
                ],
            ),
        ),
        // 0x32 	IMM8_IMM8_V8 	supercallthisrange RR, +AA, vBB
        (
            0x32,
            ByteCodeFormat::new(
                "supercallthisrange RR, +AA, vBB".to_owned(),
                vec![
                    FormatUnit::Opcode,
                    FormatUnit::RR,
                    FormatUnit::IMM8,
                    FormatUnit::V8,
                ],
            ),
        ),
        // 0x33 	IMM8_ID16_IMM8 	definefunc RR, @AAAA, +BB
        (
            0x33,
            ByteCodeFormat::new(
                "definefunc RR, @AAAA, +BB".to_owned(),
                vec![
                    FormatUnit::Opcode,
                    FormatUnit::RR,
                    FormatUnit::MethodID,
                    FormatUnit::IMM8,
                ],
            ),
        ),
        // 0x34 	IMM8_ID16_IMM8 	definemethod RR, @AAAA, +BB
        (
            0x34,
            ByteCodeFormat::new(
                "definemethod RR, @AAAA, +BB".to_owned(),
                vec![
                    FormatUnit::Opcode,
                    FormatUnit::RR,
                    FormatUnit::MethodID,
                    FormatUnit::IMM8,
                ],
            ),
        ),
        // 0x35 	IMM8_ID16_ID16_IMM16_V8 	defineclasswithbuffer RR, @AAAA, @BBBB, +CCCC, vDD
        (
            0x35,
            ByteCodeFormat::new(
                "defineclasswithbuffer RR, @AAAA, @BBBB, +CCCC, vDD".to_owned(),
                vec![
                    FormatUnit::Opcode,
                    FormatUnit::RR,
                    FormatUnit::MethodID,
                    FormatUnit::LiteralID,
                    FormatUnit::IMM16,
                    FormatUnit::V8,
                ],
            ),
        ),
        // 0x36 	V8 	getnextpropname vAA 	A：迭代器 	执行for-in迭代器A的next方法，并将结果存放到acc中。
        (
            0x36,
            ByteCodeFormat::new(
                "getnextpropname".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::V8],
            ),
        ),
        // 0x37 	IMM8_V8 	ldobjbyvalue RR, vAA
        (
            0x37,
            ByteCodeFormat::new(
                "ldobjbyvalue RR, vAA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RR, FormatUnit::V8],
            ),
        ),
        // 0x38 	IMM8_V8_V8 	stobjbyvalue RR, vAA, vBB
        (
            0x38,
            ByteCodeFormat::new(
                "stobjbyvalue RR, vAA, vBB".to_owned(),
                vec![
                    FormatUnit::Opcode,
                    FormatUnit::RR,
                    FormatUnit::V8,
                    FormatUnit::V8,
                ],
            ),
        ),
        // 0x39 	IMM8_V8 	ldsuperbyvalue RR, vAA
        (
            0x39,
            ByteCodeFormat::new(
                "ldsuperbyvalue RR, vAA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RR, FormatUnit::V8],
            ),
        ),
        // 0x3a 	IMM8_IMM16 	ldobjbyindex RR, +AAAA
        (
            0x3a,
            ByteCodeFormat::new(
                "ldobjbyindex RR, +AAAA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RR, FormatUnit::IMM16],
            ),
        ),
        // 0x3b 	IMM8_V8_IMM16 	stobjbyindex RR, vAA, +BBBB
        (
            0x3b,
            ByteCodeFormat::new(
                "stobjbyindex RR, vAA, +BBBB".to_owned(),
                vec![
                    FormatUnit::Opcode,
                    FormatUnit::RR,
                    FormatUnit::V8,
                    FormatUnit::IMM16,
                ],
            ),
        ),
        // 0x3c 	IMM4_IMM4 	ldlexvar +A, +B
        (
            0x3c,
            ByteCodeFormat::new(
                "ldlexvar +A, +B".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::Imm4Imm4],
            ),
        ),
        // 0x3d 	IMM4_IMM4 	stlexvar +A, +B
        (
            0x3d,
            ByteCodeFormat::new(
                "stlexvar +A, +B".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::Imm4Imm4],
            ),
        ),
        // 0x3e 	ID16 	lda.str @AAAA 	A：string id 	将索引A对应的字符串存放到acc中。
        (
            0x3e,
            ByteCodeFormat::new(
                "lda.str @AAAA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::StringID],
            ),
        ),
        // 0x3f 	IMM8_ID16 	tryldglobalbyname RR, @AAAA
        (
            0x3f,
            ByteCodeFormat::new(
                "tryldglobalbyname RR, @AAAA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RR, FormatUnit::StringID],
            ),
        ),
        // 0x40 	IMM8_ID16 	trystglobalbyname RR, @AAAA
        (
            0x40,
            ByteCodeFormat::new(
                "trystglobalbyname RR, @AAAA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RR, FormatUnit::StringID],
            ),
        ),
        // 0x41 	IMM16_ID16 	ldglobalvar RRRR, @AAAA
        (
            0x41,
            ByteCodeFormat::new(
                "ldglobalvar RRRR, @AAAA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RRRR, FormatUnit::StringID],
            ),
        ),
        // 0x42 	IMM8_ID16 	ldobjbyname RR, @AAAA
        (
            0x42,
            ByteCodeFormat::new(
                "ldobjbyname RR, @AAAA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::IMM8, FormatUnit::StringID],
            ),
        ),
        // 0x43 	IMM8_ID16_V8 	stobjbyname RR, @AAAA, vBB
        (
            0x43,
            ByteCodeFormat::new(
                "stobjbyname RR, @AAAA, vBB".to_owned(),
                vec![
                    FormatUnit::Opcode,
                    FormatUnit::RR,
                    FormatUnit::StringID,
                    FormatUnit::V8,
                ],
            ),
        ),
        // 0x44 	V4_V4 	mov vA, vB 	A, B：寄存器索引 	将寄存器B中的内容复制到寄存器A中。
        (
            0x44,
            ByteCodeFormat::new(
                "mov vA, vB".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::V4V4],
            ),
        ),
        // 0x45 	V8_V8 	mov vAA, vBB
        (
            0x45,
            ByteCodeFormat::new(
                "mov vAA, vBB".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::V8, FormatUnit::V8],
            ),
        ),
        // 0x46 	IMM8_ID16 	ldsuperbyname RR, @AAAA
        (
            0x46,
            ByteCodeFormat::new(
                "ldsuperbyname RR, @AAAA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RR, FormatUnit::StringID],
            ),
        ),
        // 0x47 	IMM16_ID16 	stconsttoglobalrecord RRRR, @AAAA
        (
            0x47,
            ByteCodeFormat::new(
                "stconsttoglobalrecord RRRR, @AAAA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RRRR, FormatUnit::StringID],
            ),
        ),
        // 0x48 	IMM16_ID16 	sttoglobalrecord RRRR, @AAAA
        (
            0x48,
            ByteCodeFormat::new(
                "stconsttoglobalrecord RRRR, @AAAA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RRRR, FormatUnit::StringID],
            ),
        ),
        // 0x49 	IMM8_ID16 	ldthisbyname RR, @AAAA
        (
            0x49,
            ByteCodeFormat::new(
                "ldthisbyname RR, @AAAA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RR, FormatUnit::StringID],
            ),
        ),
        // 0x4a 	IMM8_ID16 	stthisbyname RR, @AAAA
        (
            0x4a,
            ByteCodeFormat::new(
                "stthisbyname RR, @AAAA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RR, FormatUnit::StringID],
            ),
        ),
        // 0x4b 	IMM8 	ldthisbyvalue RR
        (
            0x4b,
            ByteCodeFormat::new(
                "ldthisbyvalue RR".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RR],
            ),
        ),
        // 0x4c 	IMM8_V8 	stthisbyvalue RR, vAA
        (
            0x4c,
            ByteCodeFormat::new(
                "stthisbyvalue RR, vAA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RR, FormatUnit::V8],
            ),
        ),
        // 0x4d 	IMM8 	jmp +AA 	A：有符号的分支偏移量 	无条件跳转到分支A。
        (
            0x4d,
            ByteCodeFormat::new(
                "jmp +AA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::IMM8],
            ),
        ),
        // 0x4e 	IMM16 	jmp +AAAA 	A：有符号的分支偏移量 	无条件跳转到分支A。
        (
            0x4e,
            ByteCodeFormat::new(
                "jmp +AAAA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::IMM16],
            ),
        ),
        // 0x4f 	IMM8 	jeqz +AA
        (
            0x4f,
            ByteCodeFormat::new(
                "jeqz +AA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::IMM8],
            ),
        ),
        // 0x50 	IMM16 	jeqz +AAAA
        (
            0x50,
            ByteCodeFormat::new(
                "jeqz +AAAA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::IMM16],
            ),
        ),
        // 0x51 	IMM8 	jnez +AA
        (
            0x51,
            ByteCodeFormat::new(
                "jnez +AA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::IMM8],
            ),
        ),
        // 0x52 	IMM8 	jstricteqz +AA
        (
            0x52,
            ByteCodeFormat::new(
                "jstricteqz +AA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::IMM8],
            ),
        ),
        // 0x53 	IMM8 	jnstricteqz +AA
        (
            0x53,
            ByteCodeFormat::new(
                "jnstricteqz +AA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::IMM8],
            ),
        ),
        // 0x54 	IMM8 	jeqnull +AA
        (
            0x54,
            ByteCodeFormat::new(
                "jeqnull +AA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::IMM8],
            ),
        ),
        // 0x55 	IMM8 	jnenull +AA
        (
            0x55,
            ByteCodeFormat::new(
                "jnenull +AA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::IMM8],
            ),
        ),
        // 0x56 	IMM8 	jstricteqnull +AA
        (
            0x56,
            ByteCodeFormat::new(
                "jstricteqnull +AA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::IMM8],
            ),
        ),
        // 0x57 	IMM8 	jnstricteqnull +AA
        (
            0x57,
            ByteCodeFormat::new(
                "jnstricteqnull +AA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::IMM8],
            ),
        ),
        // 0x58 	IMM8 	jequndefined +AA
        (
            0x58,
            ByteCodeFormat::new(
                "jequndefined +AA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::IMM8],
            ),
        ),
        // 0x59 	IMM8 	jneundefined +AA
        (
            0x59,
            ByteCodeFormat::new(
                "jneundefined +AA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::IMM8],
            ),
        ),
        // 0x5a 	IMM8 	jstrictequndefined +AA
        (
            0x5a,
            ByteCodeFormat::new(
                "jstrictequndefined +AA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::IMM8],
            ),
        ),
        // 0x5b 	IMM8 	jnstrictequndefined +AA
        (
            0x5b,
            ByteCodeFormat::new(
                "jnstrictequndefined +AA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::IMM8],
            ),
        ),
        // 0x5c 	V8_IMM8 	jeq vAA, +BB
        (
            0x5c,
            ByteCodeFormat::new(
                "jeq vAA, +BB".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::V8, FormatUnit::IMM8],
            ),
        ),
        // 0x5d 	V8_IMM8 	jne vAA, +BB
        (
            0x5d,
            ByteCodeFormat::new(
                "jne vAA, +BB".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::V8, FormatUnit::IMM8],
            ),
        ),
        // 0x5e 	V8_IMM8 	jstricteq vAA, +BB
        (
            0x5e,
            ByteCodeFormat::new(
                "jstricteq vAA, +BB".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::V8, FormatUnit::IMM8],
            ),
        ),
        // 0x5f 	V8_IMM8 	jnstricteq vAA, +BB
        (
            0x5f,
            ByteCodeFormat::new(
                "jnstricteq vAA, +BB".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::V8, FormatUnit::IMM8],
            ),
        ),
        // 0x60 	V8 	lda vAA 	A：寄存器索引 	将寄存器A中的内容存放到acc中。
        (
            0x60,
            ByteCodeFormat::new(
                "lda vAA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::V8],
            ),
        ),
        // 0x61 	V8 	sta vAA
        (
            0x61,
            ByteCodeFormat::new(
                "sta vAA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::V8],
            ),
        ),
        // 0x62 	IMM32 	ldai +AAAAAAAA 	A：常量字面量 	将整型字面量A存放到acc中。
        (
            0x62,
            ByteCodeFormat::new(
                "ldai".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::IMM32],
            ),
        ),
        // 0x63 	IMM64 	fldai +AAAAAAAAAAAAAAAA 	A：常量字面量 	将双精度浮点型字面量A存放到acc中。
        (
            0x63,
            ByteCodeFormat::new(
                "fldai".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::IMM64],
            ),
        ),
        // 0x64 	NONE 	return 	默认入参：acc：值 	返回acc中的值。
        (
            0x64,
            ByteCodeFormat::new("return".to_owned(), vec![FormatUnit::Opcode]),
        ),
        // 0x65 	NONE 	returnundefined 		返回undefined。
        (
            0x65,
            ByteCodeFormat::new("returnundefined".to_owned(), vec![FormatUnit::Opcode]),
        ),
        // 0x66 	NONE 	getpropiterator 	默认入参：acc：对象 	将acc中所存的对象的for-in迭代器存放到acc中。
        (
            0x66,
            ByteCodeFormat::new("getpropiterator".to_owned(), vec![FormatUnit::Opcode]),
        ),
        // 0x67 	IMM8 	getiterator RR
        (
            0x67,
            ByteCodeFormat::new(
                "getiterator".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RR],
            ),
        ),
        // 0x68 	IMM8_V8 	closeiterator RR, vAA
        (
            0x68,
            ByteCodeFormat::new(
                "closeiterator".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RR, FormatUnit::V8],
            ),
        ),
        // 0x69 	NONE 	poplexenv 		跳出当前的词法环境，进入外面一层词法环境。
        (
            0x69,
            ByteCodeFormat::new("poplexenv".to_owned(), vec![FormatUnit::Opcode]),
        ),
        // 0x6a 	NONE 	ldnan 		将nan存放到acc中。
        (
            0x6a,
            ByteCodeFormat::new("ldnan".to_owned(), vec![FormatUnit::Opcode]),
        ),
        // 0x6b 	NONE 	ldinfinity 		将infinity存放到acc中。
        (
            0x6b,
            ByteCodeFormat::new("ldinfinity".to_owned(), vec![FormatUnit::Opcode]),
        ),
        // 0x6c 	NONE 	getunmappedargs 		将当前函数的arguments存放到acc中。
        (
            0x6c,
            ByteCodeFormat::new("getunmappedargs".to_owned(), vec![FormatUnit::Opcode]),
        ),
        // 0x6d 	NONE 	ldglobal 		将global对象存放到acc中。
        (
            0x6d,
            ByteCodeFormat::new("ldglobal".to_owned(), vec![FormatUnit::Opcode]),
        ),
        // 0x6e 	NONE 	ldnewtarget
        (
            0x6e,
            ByteCodeFormat::new("ldnewtarget".to_owned(), vec![FormatUnit::Opcode]),
        ),
        // 0x6f 	NONE 	ldthis 		将this存放到acc中。
        (
            0x6f,
            ByteCodeFormat::new("ldthis".to_owned(), vec![FormatUnit::Opcode]),
        ),
        // 0x70 	NONE 	ldhole 		将hole存放到acc中。
        (
            0x70,
            ByteCodeFormat::new("ldhole".to_owned(), vec![FormatUnit::Opcode]),
        ),
        // 0x71 	IMM8_ID16_IMM8 	createregexpwithliteral RR, @AAAA, +BB
        (
            0x71,
            ByteCodeFormat::new(
                "createregexpwithliteral".to_owned(),
                vec![
                    FormatUnit::Opcode,
                    FormatUnit::RR,
                    FormatUnit::MethodID,
                    FormatUnit::IMM8,
                ],
            ),
        ),
        // 0x72 	IMM16_ID16_IMM8 	createregexpwithliteral RRRR, @AAAA, +BB
        (
            0x72,
            ByteCodeFormat::new(
                "createregexpwithliteral".to_owned(),
                vec![
                    FormatUnit::Opcode,
                    FormatUnit::RRRR,
                    FormatUnit::StringID,
                    FormatUnit::IMM8,
                ],
            ),
        ),
        // 0x73 	IMM8_IMM8_V8 	callrange RR, +AA, vBB
        (
            0x73,
            ByteCodeFormat::new(
                "callrange".to_owned(),
                vec![
                    FormatUnit::Opcode,
                    FormatUnit::RR,
                    FormatUnit::IMM8,
                    FormatUnit::V8,
                ],
            ),
        ),
        // 0x74 	IMM16_ID16_IMM8 	definefunc RRRR, @AAAA, +BB
        (
            0x74,
            ByteCodeFormat::new(
                "definefunc".to_owned(),
                vec![
                    FormatUnit::Opcode,
                    FormatUnit::RRRR,
                    FormatUnit::MethodID,
                    FormatUnit::IMM8,
                ],
            ),
        ),
        // 0x75 	IMM16_ID16_ID16_IMM16_V8 	defineclasswithbuffer RRRR, @AAAA, @BBBB, +CCCC, vDD
        (
            0x75,
            ByteCodeFormat::new(
                "defineclasswithbuffer".to_owned(),
                vec![
                    FormatUnit::Opcode,
                    FormatUnit::RRRR,
                    FormatUnit::MethodID,
                    FormatUnit::LiteralID,
                    FormatUnit::IMM16,
                    FormatUnit::V8,
                ],
            ),
        ),
        // 0x76 	IMM8 	gettemplateobject RR
        (
            0x76,
            ByteCodeFormat::new(
                "gettemplateobject".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RR],
            ),
        ),
        // 0x77 	IMM8_V8 	setobjectwithproto RR, vAA
        (
            0x77,
            ByteCodeFormat::new(
                "setobjectwithproto".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RR, FormatUnit::V8],
            ),
        ),
        // 0x78 	IMM8_V8_V8 	stownbyvalue RR, vAA, vBB
        (
            0x78,
            ByteCodeFormat::new(
                "stownbyvalue".to_owned(),
                vec![
                    FormatUnit::Opcode,
                    FormatUnit::RR,
                    FormatUnit::V8,
                    FormatUnit::V8,
                ],
            ),
        ),
        // 0x79 	IMM8_V8_IMM16 	stownbyindex RR, vAA, +BBBB
        (
            0x79,
            ByteCodeFormat::new(
                "stownbyindex".to_owned(),
                vec![
                    FormatUnit::Opcode,
                    FormatUnit::RR,
                    FormatUnit::V8,
                    FormatUnit::IMM16,
                ],
            ),
        ),
        // 0x7a 	IMM8_ID16_V8 	stownbyname RR, @AAAA, vBB
        (
            0x7a,
            ByteCodeFormat::new(
                "stownbyname".to_owned(),
                vec![
                    FormatUnit::Opcode,
                    FormatUnit::RR,
                    FormatUnit::StringID,
                    FormatUnit::V8,
                ],
            ),
        ),
        // 0x7b 	IMM8 	getmodulenamespace +AA 	A：模块索引 	对第A个模块，执行GetModuleNamespace，并将结果存放到acc中。
        (
            0x7b,
            ByteCodeFormat::new(
                "getmodulenamespace".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::IMM8],
            ),
        ),
        // 0x7c 	IMM8 	stmodulevar +AA
        (
            0x7c,
            ByteCodeFormat::new(
                "stmodulevar".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::IMM8],
            ),
        ),
        // 0x7d 	IMM8 	ldlocalmodulevar +AA 	A：槽位号 	将槽位号为A的局部模块变量存放到acc中。
        (
            0x7d,
            ByteCodeFormat::new(
                "ldlocalmodulevar".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::IMM8],
            ),
        ),
        // 0x7e 	IMM8 	ldexternalmodulevar +AA 	A：槽位号 	将槽位号为A的外部模块变量存放到acc中。
        (
            0x7e,
            ByteCodeFormat::new(
                "ldexternalmodulevar +AA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::IMM8],
            ),
        ),
        // 0x7f 	IMM16_ID16 	stglobalvar RRRR, @AAAA
        (
            0x7f,
            ByteCodeFormat::new(
                "stglobalvar".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RRRR, FormatUnit::StringID],
            ),
        ),
        // 0x80 	IMM16 	createemptyarray RRRR 	R：方舟运行时内部使用的16位保留数字 	创建一个空数组，并将其存放到acc中。
        (
            0x80,
            ByteCodeFormat::new(
                "createemptyarray".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RRRR],
            ),
        ),
        // 0x81 	IMM16_ID16 	createarraywithbuffer RRRR, @AAAA
        (
            0x81,
            ByteCodeFormat::new(
                "createarraywithbuffer".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RRRR, FormatUnit::LiteralID],
            ),
        ),
        // 0x82 	IMM16_ID16 	createobjectwithbuffer RRRR, @AAAA
        (
            0x82,
            ByteCodeFormat::new(
                "createobjectwithbuffer".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RRRR, FormatUnit::LiteralID],
            ),
        ),
        // 0x83 	IMM16_IMM8_V8 	newobjrange RRRR, +AA, vBB
        (
            0x83,
            ByteCodeFormat::new(
                "newobjrange".to_owned(),
                vec![
                    FormatUnit::Opcode,
                    FormatUnit::RRRR,
                    FormatUnit::IMM8,
                    FormatUnit::V8,
                ],
            ),
        ),
        // 0x84 	IMM16 	typeof RRRR
        (
            0x84,
            ByteCodeFormat::new(
                "typeof".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RRRR],
            ),
        ),
        // 0x85 	IMM16_V8 	ldobjbyvalue RRRR, vAA
        (
            0x85,
            ByteCodeFormat::new(
                "ldobjbyvalue".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RRRR, FormatUnit::V8],
            ),
        ),
        // 0x86 	IMM16_V8_V8 	stobjbyvalue RRRR, vAA, vBB
        (
            0x86,
            ByteCodeFormat::new(
                "stobjbyvalue".to_owned(),
                vec![
                    FormatUnit::Opcode,
                    FormatUnit::RRRR,
                    FormatUnit::V8,
                    FormatUnit::V8,
                ],
            ),
        ),
        // 0x87 	IMM16_V8 	ldsuperbyvalue RRRR, vAA
        (
            0x87,
            ByteCodeFormat::new(
                "ldsuperbyvalue".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RRRR, FormatUnit::V8],
            ),
        ),
        // 0x88 	IMM16_IMM16 	ldobjbyindex RRRR, +AAAA

        // 0x89 	IMM16_V8_IMM16 	stobjbyindex RRRR, vAA, +BBBB
        // 0x8a 	IMM8_IMM8 	ldlexvar +AA, +BB
        // 0x8b 	IMM8_IMM8 	stlexvar +AA, +BB
        // 0x8c 	IMM16_ID16 	tryldglobalbyname RRRR, @AAAA
        // 0x8d 	IMM16_ID16 	trystglobalbyname RRRR, @AAAA
        // 0x8e 	IMM8_ID16_V8 	stownbynamewithnameset RR, @AAAA, vBB
        // 0x8f 	V16_V16 	mov vAAAA, vBBBB 	A, B：寄存器索引 	将寄存器B中的内容复制到寄存器A中。
        // 0x90 	IMM16_ID16 	ldobjbyname RRRR, @AAAA
        // 0x91 	IMM16_ID16_V8 	stobjbyname RRRR, @AAAA, vBB
        // 0x92 	IMM16_ID16 	ldsuperbyname RRRR, @AAAA
        // 0x93 	IMM16_ID16 	ldthisbyname RRRR, @AAAA
        // 0x94 	IMM16_ID16 	stthisbyname RRRR, @AAAA
        // 0x95 	IMM16 	ldthisbyvalue RRRR
        // 0x96 	IMM16_V8 	stthisbyvalue RRRR, vAA
        // 0x97 	V8 	asyncgeneratorreject vAA
        // 0x98 	IMM32 	jmp +AAAAAAAA 	A：有符号的分支偏移量 	无条件跳转到分支A。
        // 0x99 	IMM8_V8_V8 	stownbyvaluewithnameset RR, vAA, vBB
        // 0x9a 	IMM32 	jeqz +AAAAAAAA
        // 0x9b 	IMM16 	jnez +AAAA
        // 0x9c 	IMM32 	jnez +AAAAAAAA
        // 0x9d 	IMM16 	jstricteqz +AAAA
        // 0x9e 	IMM16 	jnstricteqz +AAAA
        // 0x9f 	IMM16 	jeqnull +AAAA
        // 0xa0 	IMM16 	jnenull +AAAA
        // 0xa1 	IMM16 	jstricteqnull +AAAA
        // 0xa2 	IMM16 	jnstricteqnull +AAAA
        // 0xa3 	IMM16 	jequndefined +AAAA
        // 0xa4 	IMM16 	jneundefined +AAAA
        // 0xa5 	IMM16 	jstrictequndefined +AAAA
        // 0xa6 	IMM16 	jnstrictequndefined +AAAA
        // 0xa7 	V8_IMM16 	jeq vAA, +BBBB
        // 0xa8 	V8_IMM16 	jne vAA, +BBBB
        // 0xa9 	V8_IMM16 	jstricteq vAA, +BBBB
        // 0xaa 	V8_IMM16 	jnstricteq vAA, +BBBB
        // 0xab 	IMM16 	getiterator RRRR
        // 0xac 	IMM16_V8 	closeiterator RRRR, vAA
        // 0xad 	NONE 	ldsymbol 		加载Symbol对象到acc中。
        (
            0xad,
            ByteCodeFormat::new("ldsymbol".to_owned(), vec![FormatUnit::Opcode]),
        ),
        // 0xae 	NONE 	asyncfunctionenter 		创建一个异步函数对象，并将这个对象存放到acc中。
        (
            0xae,
            ByteCodeFormat::new("asyncfunctionenter".to_owned(), vec![FormatUnit::Opcode]),
        ),
        // 0xaf 	NONE 	ldfunction 		将当前的函数对象加载到acc中。
        (
            0xaf,
            ByteCodeFormat::new("ldfunction".to_owned(), vec![FormatUnit::Opcode]),
        ),
        // 0xb0 	NONE 	debugger 		调试时用于暂停执行。
        (
            0xb0,
            ByteCodeFormat::new("debugger".to_owned(), vec![FormatUnit::Opcode]),
        ),
        // 0xb1 	V8 	creategeneratorobj vAA
        (
            0xb1,
            ByteCodeFormat::new(
                "creategeneratorobj vAA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::V8],
            ),
        ),
        // 0xb2 	V8_V8 	createiterresultobj vAA, vBB
        (
            0xb2,
            ByteCodeFormat::new(
                "createiterresultobj vAA, vBB".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::V8, FormatUnit::V8],
            ),
        ),
        // 0xb3 	IMM8_V8_V8 	createobjectwithexcludedkeys +AA, vBB, vCC
        (
            0xb3,
            ByteCodeFormat::new(
                "createobjectwithexcludedkeys +AA, vBB, vCC".to_owned(),
                vec![
                    FormatUnit::Opcode,
                    FormatUnit::IMM8,
                    FormatUnit::V8,
                    FormatUnit::V8,
                ],
            ),
        ),
        // 0xb4 	IMM8_V8 	newobjapply RR, vAA
        (
            0xb4,
            ByteCodeFormat::new(
                "newobjapply RR, vAA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RR, FormatUnit::V8],
            ),
        ),
        // 0xb5 	IMM16_V8 	newobjapply RRRR, vAA
        (
            0xb5,
            ByteCodeFormat::new(
                "newobjapply RRRR, vAA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RRRR, FormatUnit::V8],
            ),
        ),
        // 0xb6 	IMM8_ID16 	newlexenvwithname +AA, @BBBB
        (
            0xb6,
            ByteCodeFormat::new(
                "newlexenvwithname".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::IMM8, FormatUnit::LiteralID],
            ),
        ),
        // 0xb7 	V8 	createasyncgeneratorobj vAA
        (
            0xb7,
            ByteCodeFormat::new(
                "createasyncgeneratorobj vAA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::V8],
            ),
        ),
        // 0xb8 	V8_V8_V8 	asyncgeneratorresolve vAA, vBB, vCC
        (
            0xb8,
            ByteCodeFormat::new(
                "asyncgeneratorresolve".to_owned(),
                vec![
                    FormatUnit::Opcode,
                    FormatUnit::V8,
                    FormatUnit::V8,
                    FormatUnit::V8,
                ],
            ),
        ),
        // 0xb9 	IMM8_V8 	supercallspread RR, vAA
        (
            0xb9,
            ByteCodeFormat::new(
                "supercallspread RR, vAA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RR, FormatUnit::V8],
            ),
        ),
        // 0xba 	IMM8_V8_V8 	applyspread RR, vAA, vBB
        (
            0xba,
            ByteCodeFormat::new(
                "applyspread RR, vAA, vBB".to_owned(),
                vec![
                    FormatUnit::Opcode,
                    FormatUnit::RR,
                    FormatUnit::V8,
                    FormatUnit::V8,
                ],
            ),
        ),
        // 0xba 	IMM8_V8_V8 	apply RR, vAA, vBB
        (
            0xba,
            ByteCodeFormat::new(
                "apply RR, vAA, vBB".to_owned(),
                vec![
                    FormatUnit::Opcode,
                    FormatUnit::RR,
                    FormatUnit::V8,
                    FormatUnit::V8,
                ],
            ),
        ),
        // 0xbb 	IMM8_IMM8_V8 	supercallarrowrange RR, +AA, vBB
        (
            0xbb,
            ByteCodeFormat::new(
                "supercallarrowrange".to_owned(),
                vec![
                    FormatUnit::Opcode,
                    FormatUnit::RR,
                    FormatUnit::IMM8,
                    FormatUnit::V8,
                ],
            ),
        ),
        // 0xbc 	V8_V8_V8_V8 	definegettersetterbyvalue vAA, vBB, vCC, vDD
        (
            0xbc,
            ByteCodeFormat::new(
                "definegettersetterbyvalue".to_owned(),
                vec![
                    FormatUnit::Opcode,
                    FormatUnit::V8,
                    FormatUnit::V8,
                    FormatUnit::V8,
                    FormatUnit::V8,
                ],
            ),
        ),
        // 0xbd 	NONE 	dynamicimport 	默认入参：acc：值 	使用acc中的值作为参数，执行ImportCalls，并把结果存放到acc中。
        (
            0xbd,
            ByteCodeFormat::new("dynamicimport".to_owned(), vec![FormatUnit::Opcode]),
        ),
        // 0xbe 	IMM16_ID16_IMM8 	definemethod RRRR, @AAAA, +BB
        (
            0xbe,
            ByteCodeFormat::new(
                "definemethod RRRR, @AAAA, +BB".to_owned(),
                vec![
                    FormatUnit::Opcode,
                    FormatUnit::RRRR,
                    FormatUnit::MethodID,
                    FormatUnit::IMM8,
                ],
            ),
        ),
        // 0xbf 	NONE 	resumegenerator
        (
            0xbf,
            ByteCodeFormat::new("resumegenerator".to_owned(), vec![FormatUnit::Opcode]),
        ),
        // 0xc0 	NONE 	getresumemode
        (
            0xc0,
            ByteCodeFormat::new("getresumemode".to_owned(), vec![FormatUnit::Opcode]),
        ),
        // 0xc1 	IMM16 	gettemplateobject RRRR
        (
            0xc1,
            ByteCodeFormat::new(
                "gettemplateobject RRRR".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RRRR],
            ),
        ),
        // 0xc2 	V8 	delobjprop vAA
        (
            0xc2,
            ByteCodeFormat::new(
                "delobjprop vAA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::V8],
            ),
        ),
        // 0xc3 	V8 	suspendgenerator vAA
        (
            0xc3,
            ByteCodeFormat::new(
                "suspendgenerator vAA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::V8],
            ),
        ),
        // 0xc4 	V8 	asyncfunctionawaituncaught vAA
        (
            0xc4,
            ByteCodeFormat::new(
                "asyncfunctionawaituncaught vAA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::V8],
            ),
        ),
        // 0xc5 	V8 	copydataproperties vAA
        (
            0xc5,
            ByteCodeFormat::new(
                "copydataproperties vAA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::V8],
            ),
        ),
        // 0xc6 	V8_V8 	starrayspread vAA, vBB
        (
            0xc6,
            ByteCodeFormat::new(
                "starrayspread vAA, vBB".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::V8, FormatUnit::V8],
            ),
        ),
        // 0xc7 	IMM16_V8 	setobjectwithproto RRRR, vAA
        (
            0xc7,
            ByteCodeFormat::new(
                "setobjectwithproto RRRR, vAA".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RRRR, FormatUnit::V8],
            ),
        ),
        // 0xc8 	IMM16_V8_V8 	stownbyvalue RRRR, vAA, vBB
        (
            0xc8,
            ByteCodeFormat::new(
                "stownbyvalue RRRR, vAA, vBB".to_owned(),
                vec![
                    FormatUnit::Opcode,
                    FormatUnit::RRRR,
                    FormatUnit::V8,
                    FormatUnit::V8,
                ],
            ),
        ),
        // 0xc9 	IMM8_V8_V8 	stsuperbyvalue RR, vAA, vBB
        (
            0xc9,
            ByteCodeFormat::new(
                "stsuperbyvalue RR, vAA, vBB".to_owned(),
                vec![
                    FormatUnit::Opcode,
                    FormatUnit::IMM8,
                    FormatUnit::V8,
                    FormatUnit::V8,
                ],
            ),
        ),
        // 0xca 	IMM16_V8_V8 	stsuperbyvalue RRRR, vAA, vBB
        (
            0xca,
            ByteCodeFormat::new(
                "stsuperbyvalue RRRR, vAA, vBB".to_owned(),
                vec![
                    FormatUnit::Opcode,
                    FormatUnit::RRRR,
                    FormatUnit::V8,
                    FormatUnit::V8,
                ],
            ),
        ),
        // 0xcb 	IMM16_V8_IMM16 	stownbyindex RRRR, vAA, +BBBB
        (
            0xcb,
            ByteCodeFormat::new(
                "stownbyindex RRRR, vAA, +BBBB".to_owned(),
                vec![
                    FormatUnit::Opcode,
                    FormatUnit::RRRR,
                    FormatUnit::V8,
                    FormatUnit::IMM16,
                ],
            ),
        ),
        // 0xcc 	IMM16_ID16_V8 	stownbyname RRRR, @AAAA, vBB
        (
            0xcc,
            ByteCodeFormat::new(
                "stownbyname RRRR, @AAAA, vBB".to_owned(),
                vec![
                    FormatUnit::Opcode,
                    FormatUnit::RRRR,
                    FormatUnit::StringID,
                    FormatUnit::V8,
                ],
            ),
        ),
        // 0xcd 	V8 	asyncfunctionresolve vAA
        (
            0xcd,
            ByteCodeFormat::new("asyncfunctionresolve".to_owned(), vec![FormatUnit::Opcode]),
        ),
        // 0xce 	V8 	asyncfunctionreject vAA
        (
            0xce,
            ByteCodeFormat::new("asyncfunctionreject".to_owned(), vec![FormatUnit::Opcode]),
        ),
        // 0xcf 	IMM8 	copyrestargs +AA 	A：形参列表中剩余参数所在的位次 	复制剩余参数，并将复制出的参数数组副本存放到acc中。
        (
            0xcf,
            ByteCodeFormat::new(
                "copyrestargs".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::IMM8],
            ),
        ),
        // 0xd0 	IMM8_ID16_V8 	stsuperbyname RR, @AAAA, vBB
        (
            0xd0,
            ByteCodeFormat::new(
                "stsuperbyname".to_owned(),
                vec![
                    FormatUnit::Opcode,
                    FormatUnit::RR,
                    FormatUnit::StringID,
                    FormatUnit::V8,
                ],
            ),
        ),
        // 0xd1 	IMM16_ID16_V8 	stsuperbyname RRRR, @AAAA, vBB
        (
            0xd1,
            ByteCodeFormat::new(
                "stsuperbyname".to_owned(),
                vec![
                    FormatUnit::Opcode,
                    FormatUnit::RRRR,
                    FormatUnit::StringID,
                    FormatUnit::V8,
                ],
            ),
        ),
        // 0xd2 	IMM16_V8_V8 	stownbyvaluewithnameset RRRR, vAA, vBB
        (
            0xd2,
            ByteCodeFormat::new(
                "stownbyvaluewithnameset".to_owned(),
                vec![
                    FormatUnit::Opcode,
                    FormatUnit::RRRR,
                    FormatUnit::V8,
                    FormatUnit::V8,
                ],
            ),
        ),
        // 0xd3 	ID16 	ldbigint @AAAA 	A：string id 	基于索引A对应的字符串，创建BigInt类型的值，并将其存放到acc中。
        (
            0xd3,
            ByteCodeFormat::new(
                "ldbigint".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::StringID],
            ),
        ),
        // 0xd4 	IMM16_ID16_V8 	stownbynamewithnameset RRRR, @AAAA, vBB
        (
            0xd4,
            ByteCodeFormat::new(
                "stownbynamewithnameset".to_owned(),
                vec![
                    FormatUnit::Opcode,
                    FormatUnit::RRRR,
                    FormatUnit::StringID,
                    FormatUnit::V8,
                ],
            ),
        ),
        // 0xd5 	NONE 	nop 		无操作。
        (
            0xd5,
            ByteCodeFormat::new("nop".to_owned(), vec![FormatUnit::Opcode]),
        ),
        // 0xd6 	IMM8 	setgeneratorstate +AA
        (
            0xd6,
            ByteCodeFormat::new(
                "setgeneratorstate".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::IMM8],
            ),
        ),
        // 0xd7 	IMM8 	getasynciterator RR
        (
            0xd7,
            ByteCodeFormat::new(
                "getasynciterator".to_owned(),
                vec![FormatUnit::Opcode, FormatUnit::RR],
            ),
        ),
        // 0xd8 	IMM8_IMM16_IMM16 	ldprivateproperty RR, +AAAA, +BBBB
        (
            0xd8,
            ByteCodeFormat::new(
                "ldprivateproperty".to_owned(),
                vec![
                    FormatUnit::Opcode,
                    FormatUnit::RR,
                    FormatUnit::IMM16,
                    FormatUnit::IMM16,
                ],
            ),
        ),
        // 0xd9 	IMM8_IMM16_IMM16_V8 	stprivateproperty RR, +AAAA, +BBBB, vCC
        (
            0xd9,
            ByteCodeFormat::new(
                "stprivateproperty".to_owned(),
                vec![
                    FormatUnit::Opcode,
                    FormatUnit::RR,
                    FormatUnit::IMM16,
                    FormatUnit::IMM16,
                    FormatUnit::V8,
                ],
            ),
        ),
        // 0xda 	IMM8_IMM16_IMM16 	testin RR, +AAAA, +BBBB
        (
            0xda,
            ByteCodeFormat::new(
                "testin".to_owned(),
                vec![
                    FormatUnit::Opcode,
                    FormatUnit::RR,
                    FormatUnit::IMM16,
                    FormatUnit::IMM16,
                ],
            ),
        ),
        // 0xdb 	IMM8_ID16_V8 	definefieldbyname RR, @AAAA, vBB
        (
            0xdb,
            ByteCodeFormat::new(
                "definefieldbyname".to_owned(),
                vec![
                    FormatUnit::Opcode,
                    FormatUnit::RR,
                    FormatUnit::StringID,
                    FormatUnit::V8,
                ],
            ),
        ),
        // 0xfb 	PREF_NONE 	callruntime.notifyconcurrentresult 	默认入参：acc：并发函数的返回值
        (
            0xfb,
            ByteCodeFormat::new(
                "callruntime.notifyconcurrentresult".to_owned(),
                vec![FormatUnit::PrefixOpcode],
            ),
        ),
        // 0xfd 	PREF_IMM16_V8_V8 	wide.createobjectwithexcludedkeys +AAAA, vBB, vCC
        (
            0xfd,
            ByteCodeFormat::new(
                "wide.createobjectwithexcludedkeys".to_owned(),
                vec![
                    FormatUnit::PrefixOpcode,
                    FormatUnit::IMM16,
                    FormatUnit::V8,
                    FormatUnit::V8,
                ],
            ),
        ),
        // 0xfe 	PREF_NONE 	throw 	默认入参：acc：异常 	抛出acc中存放的异
        (
            0xfe,
            ByteCodeFormat::new("thrown".to_owned(), vec![FormatUnit::PrefixOpcode]),
        ),
    ];

    let mut map = HashMap::new();
    for team in &opcode_vec {
        map.insert(team.0, team.1.clone());
    }
    map
}

fn init_prefix_opcode_map() -> HashMap<u16, ByteCodeFormat> {
    let prefix_opcode_vec = vec![
        (
            0x01fb,
            ByteCodeFormat::new(
                "callruntime.definefieldbyvalue RR, vAA, vBB".to_owned(),
                vec![
                    FormatUnit::PrefixOpcode,
                    FormatUnit::RR,
                    FormatUnit::V8,
                    FormatUnit::V8,
                ],
            ),
        ),
        (
            0x01fd,
            ByteCodeFormat::new(
                "wide.newobjrange +AAAA, vBB".to_owned(),
                vec![FormatUnit::PrefixOpcode, FormatUnit::IMM16, FormatUnit::V8],
            ),
        ),
        (
            0x01fe,
            ByteCodeFormat::new("throw.notexists".to_owned(), vec![FormatUnit::PrefixOpcode]),
        ),
        (
            0x02fb,
            ByteCodeFormat::new(
                "callruntime.definefieldbyindex RR, +AAAAAAAA, vBB".to_owned(),
                vec![
                    FormatUnit::PrefixOpcode,
                    FormatUnit::RR,
                    FormatUnit::IMM32,
                    FormatUnit::V8,
                ],
            ),
        ),
        (
            0x02fd,
            ByteCodeFormat::new(
                "wide.newlexenv +AAAA".to_owned(),
                vec![FormatUnit::PrefixOpcode, FormatUnit::IMM16],
            ),
        ),
        (
            0x02fe,
            ByteCodeFormat::new(
                "throw.patternnoncoercible".to_owned(),
                vec![FormatUnit::PrefixOpcode],
            ),
        ),
        (
            0x03fb,
            ByteCodeFormat::new(
                "wide.newlexenvwithname +AAAA, @BBBB".to_owned(),
                vec![
                    FormatUnit::PrefixOpcode,
                    FormatUnit::IMM16,
                    FormatUnit::LiteralID,
                ],
            ),
        ),
        (
            0x03fe,
            ByteCodeFormat::new(
                "throw.deletesuperproperty".to_owned(),
                vec![FormatUnit::PrefixOpcode],
            ),
        ),
        (
            0x04fb,
            ByteCodeFormat::new(
                "callruntime.createprivateproperty +AAAA, @BBBB".to_owned(),
                vec![
                    FormatUnit::PrefixOpcode,
                    FormatUnit::IMM16,
                    FormatUnit::LiteralID,
                ],
            ),
        ),
        (
            0x04fd,
            ByteCodeFormat::new(
                "wide.callrange +AAAA, vBB".to_owned(),
                vec![FormatUnit::PrefixOpcode, FormatUnit::IMM16, FormatUnit::V8],
            ),
        ),
        // 0x04fe 	PREF_V8 	throw.constassignment vAA 	A：常量变量的名称 	抛出异常：对常量变量进行赋值。
        (
            0x04fe,
            ByteCodeFormat::new(
                "throw.constassignment".to_owned(),
                vec![FormatUnit::PrefixOpcode, FormatUnit::V8],
            ),
        ),
        // 0x05fb 	PREF_IMM8_IMM_16_IMM_16_V8 	callruntime.defineprivateproperty RR, +AAAA, +BBBB, vCC
        (
            0x05fb,
            ByteCodeFormat::new(
                "callruntime.defineprivateproperty".to_owned(),
                vec![
                    FormatUnit::PrefixOpcode,
                    FormatUnit::RR,
                    FormatUnit::IMM16,
                    FormatUnit::IMM16,
                    FormatUnit::V8,
                ],
            ),
        ),
        // 0x05fd 	PREF_IMM16_V8 	wide.callthisrange +AAAA, vBB
        (
            0x05fd,
            ByteCodeFormat::new(
                "wide.callthisrange".to_owned(),
                vec![FormatUnit::PrefixOpcode, FormatUnit::IMM16, FormatUnit::V8],
            ),
        ),
        // 0x05fb 	PREF_IMM8_IMM_16_IMM_16_V8 	callruntime.defineprivateproperty RR, +AAAA, +BBBB, vCC
        (
            0x05fe,
            ByteCodeFormat::new(
                "throw.ifnotobject".to_owned(),
                vec![
                    FormatUnit::PrefixOpcode,
                    FormatUnit::RR,
                    FormatUnit::IMM16,
                    FormatUnit::IMM16,
                    FormatUnit::V8,
                ],
            ),
        ),
        // 0x05fd 	PREF_IMM16_V8 	wide.callthisrange +AAAA, vBB
        (
            0x05fd,
            ByteCodeFormat::new(
                "wide.callthisrange".to_owned(),
                vec![FormatUnit::PrefixOpcode, FormatUnit::IMM16, FormatUnit::V8],
            ),
        ),
        // 0x05fe 	PREF_V8 	throw.ifnotobject vAA 	A：对象 	如果A不是一个对象，抛出异常。
        (
            0x05fe,
            ByteCodeFormat::new(
                "throw.ifnotobject".to_owned(),
                vec![FormatUnit::PrefixOpcode, FormatUnit::V8],
            ),
        ),
        // 0x06fb 	PREF_IMM8_V8 	callruntime.callinit +RR, vAA
        (
            0x06fb,
            ByteCodeFormat::new(
                "callruntime.callinit".to_owned(),
                vec![FormatUnit::PrefixOpcode, FormatUnit::RR, FormatUnit::V8],
            ),
        ),
        // 0x06fd 	PREF_IMM16_V8 	wide.supercallthisrange +AAAA, vBB
        (
            0x06fd,
            ByteCodeFormat::new(
                "wide.supercallthisrange".to_owned(),
                vec![FormatUnit::PrefixOpcode, FormatUnit::IMM16, FormatUnit::V8],
            ),
        ),
        // 0x06fd 	PREF_IMM16_V8 	wide.supercallthisrange +AAAA, vBB
        (
            0x06fe,
            ByteCodeFormat::new(
                "throw.undefinedifhole".to_owned(),
                vec![FormatUnit::PrefixOpcode, FormatUnit::IMM16, FormatUnit::V8],
            ),
        ),
        // 0x06fe 	PREF_V8_V8 	throw.undefinedifhole vAA, vBB
        (
            0x06fe,
            ByteCodeFormat::new(
                "throw.undefinedifhole".to_owned(),
                vec![FormatUnit::PrefixOpcode, FormatUnit::V8, FormatUnit::V8],
            ),
        ),
        // 0x07fb 	PREF_IMM16_ID16_ID16_IMM16_V8 	callruntime.definesendableclass RRRR, @AAAA, @BBBB, +CCCC, vDD
        (
            0x07fb,
            ByteCodeFormat::new(
                "callruntime.definesendableclass".to_owned(),
                vec![
                    FormatUnit::PrefixOpcode,
                    FormatUnit::RRRR,
                    FormatUnit::MethodID,
                    FormatUnit::LiteralID,
                    FormatUnit::IMM16,
                    FormatUnit::V8,
                ],
            ),
        ),
        // 0x07fd 	PREF_IMM16_V8 	wide.supercallarrowrange +AAAA, vBB
        (
            0x07fd,
            ByteCodeFormat::new(
                "wide.supercallarrowrange".to_owned(),
                vec![FormatUnit::PrefixOpcode, FormatUnit::IMM16, FormatUnit::V8],
            ),
        ),
        // 0x07fe 	PREF_IMM8 	throw.ifsupernotcorrectcall +AA
        (
            0x07fe,
            ByteCodeFormat::new(
                "throw.ifsupernotcorrectcall".to_owned(),
                vec![FormatUnit::PrefixOpcode, FormatUnit::IMM8],
            ),
        ),
        // 0x08fb 	PREF_IMM16 	callruntime.ldsendableclass +AAAA
        (
            0x08fb,
            ByteCodeFormat::new(
                "callruntime.ldsendableclass".to_owned(),
                vec![FormatUnit::PrefixOpcode, FormatUnit::IMM16],
            ),
        ),
        // 0x08fd 	PREF_IMM32 	wide.ldobjbyindex +AAAAAAAA
        (
            0x08fd,
            ByteCodeFormat::new(
                "wide.ldobjbyindex".to_owned(),
                vec![FormatUnit::PrefixOpcode, FormatUnit::IMM32],
            ),
        ),
        // 0x08fe 	PREF_IMM16 	throw.ifsupernotcorrectcall +AAAA
        (
            0x08fe,
            ByteCodeFormat::new(
                "throw.ifsupernotcorrectcall".to_owned(),
                vec![FormatUnit::PrefixOpcode, FormatUnit::IMM16],
            ),
        ),
        // 0x09fd 	PREF_V8_IMM32 	wide.stobjbyindex vAA, +BBBBBBBB
        (
            0x09fd,
            ByteCodeFormat::new(
                "wide.stobjbyindex".to_owned(),
                vec![FormatUnit::PrefixOpcode, FormatUnit::V8, FormatUnit::IMM32],
            ),
        ),
        // 0x09fe 	PREF_ID16 	throw.undefinedifholewithname @AAAA
        (
            0x09fe,
            ByteCodeFormat::new(
                "throw.undefinedifholewithname".to_owned(),
                vec![FormatUnit::PrefixOpcode, FormatUnit::StringID],
            ),
        ),
        // 0x0afd 	PREF_V8_IMM32 	wide.stownbyindex vAA, +BBBBBBBB
        (
            0x0afd,
            ByteCodeFormat::new(
                "wide.stownbyindex vAA, +BBBBBBBB".to_owned(),
                vec![FormatUnit::PrefixOpcode, FormatUnit::V8, FormatUnit::IMM32],
            ),
        ),
        // 0x0bfd 	PREF_IMM16 	wide.copyrestargs +AAAA
        (
            0x0bfd,
            ByteCodeFormat::new(
                "wide.copyrestargs +AAAA".to_owned(),
                vec![FormatUnit::PrefixOpcode, FormatUnit::IMM16],
            ),
        ),
        // 0x0cfd 	PREF_IMM16_IMM16 	wide.ldlexvar +AAAA, +BBBB
        (
            0x0cfd,
            ByteCodeFormat::new(
                "wide.ldlexvar +AAAA, +BBBB".to_owned(),
                vec![
                    FormatUnit::PrefixOpcode,
                    FormatUnit::IMM16,
                    FormatUnit::IMM16,
                ],
            ),
        ),
        // 0x0dfd 	PREF_IMM16_IMM16 	wide.stlexvar +AAAA, +BBBB
        (
            0x0dfd,
            ByteCodeFormat::new(
                "wide.stlexvar +AAAA, +BBBB".to_owned(),
                vec![
                    FormatUnit::PrefixOpcode,
                    FormatUnit::IMM16,
                    FormatUnit::IMM16,
                ],
            ),
        ),
        // 0x0efd 	PREF_IMM16 	wide.getmodulenamespace +AAAA
        (
            0x0efd,
            ByteCodeFormat::new(
                "wide.getmodulenamespace +AAAA".to_owned(),
                vec![FormatUnit::PrefixOpcode, FormatUnit::IMM16],
            ),
        ),
        // 0x0ffd 	PREF_IMM16 	wide.stmodulevar +AAAA
        (
            0x0ffd,
            ByteCodeFormat::new(
                "wide.stmodulevar +AAAA".to_owned(),
                vec![FormatUnit::PrefixOpcode, FormatUnit::IMM16],
            ),
        ),
        // 0x10fd 	PREF_IMM16 	wide.ldlocalmodulevar +AAAA
        (
            0x10fd,
            ByteCodeFormat::new(
                "wide.ldlocalmodulevar +AAAA".to_owned(),
                vec![FormatUnit::PrefixOpcode, FormatUnit::IMM16],
            ),
        ),
        // 0x11fd 	PREF_IMM16 	wide.ldexternalmodulevar +AAAA
        (
            0x11fd,
            ByteCodeFormat::new(
                "wide.ldexternalmodulevar +AAAA".to_owned(),
                vec![FormatUnit::PrefixOpcode, FormatUnit::IMM16],
            ),
        ),
        // 0x12fd 	PREF_IMM16 	wide.ldpatchvar +AAAA
        (
            0x12fd,
            ByteCodeFormat::new(
                "wide.ldpatchvar +AAAA".to_owned(),
                vec![FormatUnit::PrefixOpcode, FormatUnit::IMM16],
            ),
        ),
        // 0x13fd 	PREF_IMM16 	wide.stpatchvar +AAAA
        (
            0x13fd,
            ByteCodeFormat::new(
                "wide.stpatchvar +AAAA".to_owned(),
                vec![FormatUnit::PrefixOpcode, FormatUnit::IMM16],
            ),
        ),
    ];

    let mut ok = HashMap::new();
    for team in &prefix_opcode_vec {
        ok.insert(team.0, team.1.clone());
    }
    ok
}

impl BytecodeParser {
    pub fn new() -> Self {
        let opcode_table = init_opcode_map();
        let prefix_opcode_table = init_prefix_opcode_map();

        Self {
            opcode_table,
            prefix_opcode_table,
        }
    }

    fn get_opcode(&self, opcode: u16) -> &ByteCodeFormat {
        self.opcode_table.get(&opcode).unwrap()
    }

    fn get_prefix_opcode(&self, opcode: u16) -> Option<&ByteCodeFormat> {
        self.prefix_opcode_table.get(&opcode)
    }

    pub fn parse(
        &self,
        code: &Code,
        region: &Region,
        source: &[u8],
        literal_array_map: &HashMap<usize, String>,
    ) {
        let instructions = code.instructions();
        let mut offset = 0;
        let size = instructions.len();
        loop {
            let pref_opcode = instructions.pread::<u16>(offset).unwrap();
            let bcf = self.get_prefix_opcode(pref_opcode);

            if bcf.is_none() {
                let opcode = instructions.pread::<u8>(offset).unwrap();
                let bcf = self.get_opcode(opcode as u16);
                offset = bcf.parse(instructions, offset, region, source, literal_array_map);
            } else {
                offset =
                    bcf.unwrap()
                        .parse(instructions, offset, region, source, literal_array_map);
            }

            if offset >= size {
                break;
            }

            // 剩余最后一个
            if offset + 1 == size {
                let opcode = instructions.pread::<u8>(offset).unwrap();
                let bcf = self.get_opcode(opcode as u16);
                bcf.parse(instructions, offset, region, source, literal_array_map);
                break;
            }
        }
    }
}

impl Default for BytecodeParser {
    fn default() -> Self {
        Self::new()
    }
}
