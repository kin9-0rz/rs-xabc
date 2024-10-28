use getset::Getters;
use scroll::{ctx, Uleb128};

use crate::error;

// TODO: 解析TryBlock
#[derive(Debug, Getters, Default)]
#[get = "pub"]
struct TryBlock {
    /// TryBlock的第一条指令距离其所在Code的instructions的起始位置的偏移量。
    start_pc: u64,
    /// TryBlock的大小，以字节为单位。
    length: u64,
    /// 与TryBlock关联的CatchBlock的数量，值为1。
    num_catches: u64,
    /// 与TryBlock关联的CatchBlock的数组，数组中有且仅有一个可以捕获所有类型的异常的CatchBlock。
    catch_blocks: Vec<(CatchBlock, usize)>,
}

impl<'a> ctx::TryFromCtx<'a, scroll::Endian> for TryBlock {
    type Error = error::Error;
    fn try_from_ctx(source: &'a [u8], _: scroll::Endian) -> Result<(Self, usize), Self::Error> {
        let off = &mut 0;
        let start_pc = Uleb128::read(source, off).unwrap();
        let length = Uleb128::read(source, off).unwrap();
        let num_catches = Uleb128::read(source, off).unwrap();

        let catch_blocks = (0..num_catches)
            .map(|_| CatchBlock::try_from_ctx(source, scroll::Endian::Little))
            .collect::<Result<Vec<_>, _>>()?;

        Ok((
            TryBlock {
                start_pc,
                length,
                num_catches,
                catch_blocks,
            },
            source.len(),
        ))
    }
}

// TODO: 解析CatchBlock
#[derive(Debug, Getters, Default)]
#[get = "pub"]
struct CatchBlock {
    /// 值是0，表示此CatchBlock块捕获了所有类型的异常。
    type_idx: u64,
    /// 异常处理逻辑的第一条指令的程序计数器。
    handler_pc: u64,
    /// 此CatchBlock的大小，以字节为单位。
    catch_type: u64,
}

impl<'a> ctx::TryFromCtx<'a, scroll::Endian> for CatchBlock {
    type Error = error::Error;
    fn try_from_ctx(source: &'a [u8], _: scroll::Endian) -> Result<(Self, usize), Self::Error> {
        let off = &mut 0;
        let type_idx = Uleb128::read(source, off).unwrap();
        let handler_pc = Uleb128::read(source, off).unwrap();
        let catch_type = Uleb128::read(source, off).unwrap();

        Ok((
            CatchBlock {
                type_idx,
                handler_pc,
                catch_type,
            },
            source.len(),
        ))
    }
}

#[derive(Debug, Getters, Default)]
#[get = "pub"]
pub struct Code {
    /// 寄存器的数量，存放入参和默认参数的寄存器不计算在内。
    num_regs: u64,
    /// 入参和默认参数的总数量。
    num_args: u64,
    /// 所有指令的总大小，以字节为单位。
    code_size: u64,
    /// try_blocks数组的长度，即TryBlock的数量。
    tries_size: u64,
    /// 所有指令的数组。
    instructions: Vec<u8>,
    /// 一个数组，数组中每一个元素都是TryBlock类型。
    try_blocks: Vec<(TryBlock, usize)>,
}

impl<'a> ctx::TryFromCtx<'a, scroll::Endian> for Code {
    type Error = error::Error;
    fn try_from_ctx(source: &'a [u8], _: scroll::Endian) -> Result<(Self, usize), Self::Error> {
        let off = &mut 0;

        let num_regs = Uleb128::read(source, off).unwrap();
        let num_args = Uleb128::read(source, off).unwrap();
        let code_size = Uleb128::read(source, off).unwrap();
        let tries_size = Uleb128::read(source, off).unwrap();
        println!(
            "num_regs: {}, num_args: {}, code_size: {}, tries_size: {}",
            num_regs, num_args, code_size, tries_size
        );

        let instructions = source[*off..*off + code_size as usize].to_vec();
        *off += code_size as usize;

        let try_blocks = (0..tries_size)
            .map(|_| TryBlock::try_from_ctx(source, scroll::Endian::Little))
            .collect::<Result<Vec<_>, _>>()?;

        Ok((
            Code {
                num_regs,
                num_args,
                code_size,
                tries_size,
                instructions,
                try_blocks,
            },
            source.len(),
        ))
    }
}
