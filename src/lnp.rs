use getset::Getters;

use crate::uint32_t;

/// 根据索引访问行号程序
#[derive(Debug, Getters, Default)]
#[get = "pub"]
pub struct LineNumberProgramIndex {
    /// 一个数组，数组中每个元素的值是一个偏移量，指向一个行号程序。
    offsets: Vec<uint32_t>,
}

impl LineNumberProgramIndex {
    pub fn push(&mut self, offset: uint32_t) {
        self.offsets.push(offset);
    }
}

/// 行号程序由指令组成。每条指令都有一个字节的操作码和可选参数。
/// 根据 opcode 参数的值可能被编码到指令中，或者指令需要从常量池中读取值。
// NOTE: 不知道有啥作用，先不处理。
#[allow(dead_code)]
struct LineNumberProgram {}
