use crate::{tag_value::TaggedValue, uint16_t, uint32_t, uleb128_t};

pub struct Field {
    class_idx: uint16_t,
    /// ClassRegionIndex 的一个索引
    type_idx: uint16_t,
    /// 名字的偏移量，指向一个 String
    name_off: uint32_t,
    /// 它的值必须是 AccessFlag 的组合。
    access_flags: uleb128_t,
    field_data: Vec<TaggedValue>,
}

#[allow(non_camel_case_types)]
enum AccessFlag {
    ACC_PUBLIC = 0x0001,
    ACC_PRIVATE = 0x0002,
    ACC_PROTECTED = 0x0004,
    ACC_STATIC = 0x0008,
    ACC_FINAL = 0x0010,
    ACC_VOLATILE = 0x0040,
    ACC_TRANSIENT = 0x0080,
    ACC_SYNTHETIC = 0x1000,
    ACC_ENUM = 0x4000,
}
