use std::ops::RangeFrom;

/// 索引结构，用于存放索引信息，根据索引信息可以快速定位到对应的数据。
use getset::CopyGetters;
use scroll::Pread;

use crate::{source::Source, uint32_t};

pub struct MethodStringLiteralRegionIndex {
    // 一个数组，数组中每个元素的值是一个偏移量，指向方法、字符串或者字面量数组。
    /// 数组长度由IndexHeader中的method_string_literal_region_idx_size指定。
    offsets: Vec<uint32_t>,
}

/// 基础类型
/// 表示一个基本类型编码或一个指向Class的偏移量，是一个32位的值。
#[derive(Debug)]
enum FieldType {
    u1 = 0x00,
    i8 = 0x01,
    u8 = 0x02,
    i16 = 0x03,
    u16 = 0x04,
    i32 = 0x05,
    u32 = 0x06,
    f32 = 0x07,
    f64 = 0x08,
    i64 = 0x09,
    u64 = 0x0a,
    any = 0x0b,
}

// https://gitee.com/openharmony/arkcompiler_runtime_core/blob/master/docs/file_format.md#classregionindex
pub struct ClassRegionIndex {
    /// 数组长度，由IndexHeader中的class_region_idx_size指定。
    num: u8,
    /// 一个数组，数组中每个元素都是Type类型。
    /// TODO: uint32_t, 如果不是FieldType类型的一个值，那么就是一个指向Class的偏移量
    offsets: Vec<FieldType>,
}

pub struct MethodRegionIndex {
    num: u8,
    /// 指向 Method 或 ForeignMethod 的偏移量
    offsets: Vec<uint32_t>,
}

/// 每个 IndexHeader 结构描述一个索引区域。
/// 每个索引区域都有两类索引：指向Type的索引和指向方法、字符串或者字面量数组的索引。
/// 65536 = 2^16
/// NOTE: 为什么不使用一个索引？
/// 1. 从安全上考虑，如果类、字符串的数量非常大，怎么办？如何索引。
/// 2. 如果数量大，大于 uint32 呢？
/// 既然这样，直接将这些需要索引的数据，分成一小块。
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
    /// 该区域的 MethodRegionIndex 中元素的数量，最大值为65536。
    method_idx_size: uint32_t,
    /// 一个偏移量，指向 `MethodRegionIndex`
    method_idx_off: uint32_t,
    field_idx_size: uint32_t,
    field_idx_off: uint32_t,
    proto_idx_size: uint32_t,
    proto_idx_off: uint32_t,
}

impl RegionHeader {
    // 获取类
    pub fn getClass(&self) {}

    // 获取方法、字符串或者字面量
    pub fn getString(&self) {}
}

/// 用于查找 RegionHeader 结构
pub struct RegionIndex<T> {
    source: Source<T>,
    /// IndexHeader 的数量。
    num: uint32_t,
    /// 一个偏移量，指向 IndexSection。
    offset: uint32_t,
    /// 一个数组，数组中每个元素是 IndexHeader 类型。
    /// 数组中的元素根据区域的起始偏移量进行排序。
    headers: Vec<RegionHeader>,
}

impl<T> RegionIndex<T>
where
    T: AsRef<[u8]>,
{
    /// 创建一个新的 `IndexSection`
    pub fn new(source: Source<T>, num: uint32_t, off: uint32_t) -> Self {
        let s = &source;
        //s.pread_with(0, ()).unwrap();
        //s.pread(0);

        Self {
            source,
            num,
            offset: off,
            headers: Vec::new(),
        }
    }

    /// 解析 IndexHeader
    pub fn parse(&self) {
        let source = &self.source;
        let a = source[0];
        println!("a: {}", a);
        let b = &source[0..10];
        println!("b: {:?}", b);
        //let x = [1, 2];
        //x.pread(0);
        //source.pread(0);
        //let string_data_off = source.pread_with(self.offset, ())?;
        //source.pread::<RegionHeader>(self.offset).unwrap();
        // TODO: 从 source 中读取 num 个 IndexHeader

        for i in 0..self.num {
            println!("i: {}", i);

            //source.pread::<RegionHeader>(self.offset).unwrap();
            //source.pread
            //let header = source.::<IndexHeader>(self.offset).unwrap();
            //self.headers.push(header);
        }

        // TODO: 将 IndexHeader 里面的数据对应的类和方法解析出来？这个放在
    }
}
