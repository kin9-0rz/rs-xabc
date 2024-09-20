use crate::uint32_t;

pub struct Class {}

/// ClassIndex 结构旨在允许运行时按名称快速查找类型定义。
pub struct ClassIndex {
    /// 数组长度; 由 `Header` 的 num_classes 指定。
    num: u8,
    offsets: Vec<uint32_t>,
}
