/// 索引结构，用于存放索引信息，根据索引信息可以快速定位到对应的数据。
use getset::{CopyGetters, Getters};
use scroll::Pread;
use std::fmt;

use crate::uint32_t;

#[derive(Debug, Pread, CopyGetters)]
#[get_copy = "pub"]
pub struct RegionHeader {
    /// 该区域的起始偏移量。
    start_off: uint32_t,
    /// 该区域的结束偏移量。
    end_off: uint32_t,
    /// 该区域的 ClassRegionIndex 中元素的数量，最大值为65536。
    class_idx_size: uint32_t,
    /// 一个偏移量，指向 ClassRegionIndex。
    class_idx_off: uint32_t,
    /// MethodStringLiteralRegionIndex 的数量
    method_string_literal_region_idx_size: uint32_t,
    /// 指向方法、字符串或者字面量数组。
    method_string_literal_region_idx_off: uint32_t,
    // 0XFFFFFFFF 表示无
    /// FieldRegionIndex
    field_idx_size: uint32_t,
    field_idx_off: uint32_t,
    /// ProtoRegionIndex
    proto_idx_size: uint32_t,
    proto_idx_off: uint32_t,
}

/// 存放类型，如果不是基础类型，那么就是一个指向Class的偏移量
//pub type FieldType = uint32_t;

#[derive(Debug, CopyGetters, Getters, Default)]
#[get = "pub"]
pub struct FieldType {
    // TODO: 如果类的数量特别多的话，会不会重复占用内存？
    // TODO: 也许可以增加一个 benchmark 来验证一下
    pub name: String,
}

impl fmt::Display for FieldType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Default)]
pub struct ClassRegionIndex {
    // 一个数组，数组中每个元素都是Type类型。
    offsets: Vec<FieldType>,
}

impl ClassRegionIndex {
    pub fn push(&mut self, field_type: FieldType) {
        self.offsets.push(field_type);
    }

    pub fn get(&self, idx: &usize) -> &FieldType {
        &self.offsets[*idx]
    }
}

#[derive(Debug, Getters, Default)]
#[get = "pub"]
pub struct MethodStringLiteralRegionIndex {
    offsets: Vec<uint32_t>,
}

impl MethodStringLiteralRegionIndex {
    pub fn push(&mut self, offset: uint32_t) {
        self.offsets.push(offset);
    }

    pub fn get(&self, idx: &usize) -> &uint32_t {
        &self.offsets[*idx]
    }
}

#[derive(Debug, Getters, Default)]
#[get = "pub"]
pub struct FieldRegionIndex {
    offsets: Vec<uint32_t>,
}

impl FieldRegionIndex {
    pub fn push(&mut self, offset: uint32_t) {
        self.offsets.push(offset);
    }
}

#[derive(Debug, Getters, Default)]
#[get = "pub"]
pub struct ProtoRegionIndex {
    offsets: Vec<uint32_t>,
}

impl ProtoRegionIndex {
    pub fn push(&mut self, offset: uint32_t) {
        self.offsets.push(offset);
    }
}

#[derive(Debug, Getters)]
#[get = "pub"]
pub struct Region {
    header: RegionHeader,
    /// 通过索引找到类型 FieldType
    class_region_idx: ClassRegionIndex,
    /// 找到对应的方法、字符串或者字面量数组。
    method_string_literal_region_idx: MethodStringLiteralRegionIndex,
    field_region_idx: FieldRegionIndex,
    proto_region_idx: ProtoRegionIndex,
}

impl Region {
    pub fn new(
        header: RegionHeader,
        cri: ClassRegionIndex,
        mslri: MethodStringLiteralRegionIndex,
        field_region_idx: FieldRegionIndex,
        proto_region_idx: ProtoRegionIndex,
    ) -> Self {
        Self {
            header,
            class_region_idx: cri,
            method_string_literal_region_idx: mslri,
            field_region_idx,
            proto_region_idx,
        }
    }

    /// 数据是否在这个区域内
    pub fn is_here(&self, off: usize) -> bool {
        self.header.start_off() as usize <= off && off < self.header.end_off() as usize
    }

    /// 根据索引获取它的类型
    // 只有一个地方用到，解析方法的时候，有一个。
    pub fn get_class_name(&self, idx: usize) -> &FieldType {
        self.class_region_idx.get(&idx)
    }

    /// 根据索引获取它的偏移量
    pub fn get_msl_offset(&self, idx: usize) -> &uint32_t {
        self.method_string_literal_region_idx.get(&idx)
    }
}
