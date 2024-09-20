use scroll::Uleb128;

use crate::{tag_value::TaggedValue, uint16_t, uint32_t};

pub struct Method {
    class_idx: uint16_t,
    proto_idx: uint16_t,
    name_off: uint32_t,
    access_flags: Uleb128,
    method_data: Vec<TaggedValue>,
}
