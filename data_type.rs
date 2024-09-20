//! 字节码文件数据类型
//!
//! <https://developer.huawei.com/consumer/cn/doc/harmonyos-guides-V5/arkts-bytecode-file-format-V5#%E5%AD%97%E8%8A%82%E7%A0%81%E6%96%87%E4%BB%B6%E6%95%B0%E6%8D%AE%E7%B1%BB%E5%9E%8B>

use getset::CopyGetters;
use scroll::{Sleb128, Uleb128};

/// 8-bit 无符号整数
#[allow(non_camel_case_types)]
pub type uint8_t = u8;
/// 16-bit无符号整数，采用小端字节序。
#[allow(non_camel_case_types)]
pub type uint16_t = u16;
/// 32-bit无符号整数，采用小端字节序。
#[allow(non_camel_case_types)]
pub type uint32_t = u32;
/// leb128编码的无符号整数
#[allow(non_camel_case_types)]
pub type uleb128_t = Uleb128;
/// leb128编码的有符号整数。
#[allow(non_camel_case_types)]
pub type sleb128_t = Sleb128;

/// TODO 如何读取一个 ubleb128
/// 字符串，对齐方式：单字节对齐。
struct ABCString {
    /// 值为len << 1 | is_ascii，其中len是字符串在UTF-16编码中的大小，is_ascii标记该字符串是否仅包含ASCII字符，可能的值是0或1。
    utf16_length: uleb128_t,
    /// 以'\0'结尾的MUTF-8编码字符序列。
    data: [uint8_t],
}

enum ClassTag {
    /// 拥有此标记的TaggedValue，是其所在class_data的最后一项。
    Nothing = 0,
    /// 拥有此标记的TaggedValue的data是0，表示源码语言是ArkTS/TS/JS。
    SourceLang = 0x02,
    /// 拥有此标记的TaggedValue的data是一个偏移量，指向字符串，表示源文件的名称。
    SourceFile = 0x07,
}

enum MethodTag {
    /// 拥有此标记的TaggedValue，是其所在method_data的最后一项。
    Nothing = 0,
    /// 拥有此标记的TaggedValue的data是一个偏移量，指向Code，表示方法的代码段。
    Code = 0x01,
    /// 拥有此标记的TaggedValue的data是0，表示源码语言是ArkTS/TS/JS。
    SourceLang = 0x02,
    DebugInfo = 0x05,
    /// 拥有此标记的TaggedValue的data是一个偏移量，指向Annotation， 表示方法的注解。
    Annotation = 0x06,
}

enum FieldTag {
    /// 拥有此标记的TaggedValue，是其所在field_data的最后一项。
    Nothing = 0,
    /// 拥有此标记的TaggedValue的data的类型为boolean、byte、char、short 或 int。
    IntValue = 0x01,
    /// 拥有此标记的TaggedValue的data的类型为Value formats中的FLOAT或ID。
    Value = 0x02,
}

#[derive(Debug)]
pub struct TaggedValue {
    /// 它有可能是ClassTag或者MethodTag或者FieldTag，表示数据种类的标记。
    tag: uint8_t,
    /// 根据不同的标记，data是不同类型的数据或者为空。
    data: Vec<uint8_t>,
}

/// TypeDescriptor是类(Class) 名称的格式，由'L'、'_'、ClassName和';'组成：L_ClassName;。其中，ClassName是类的全名，名字中的'.'会被替换为'/'。
#[derive(Debug)]
pub struct TypeDescriptor {
    name: String,
}

impl TypeDescriptor {
    pub fn new(name: &str) -> TypeDescriptor {
        TypeDescriptor {
            name: name.to_string(),
        }
    }
}
