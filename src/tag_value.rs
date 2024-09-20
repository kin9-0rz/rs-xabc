use crate::uint8_t;

enum ClassTag {
    /// 拥有此标记的TaggedValue，是其所在class_data的最后一项。
    Nothing = 0,
    /// 拥有此标记的TaggedValue的data是0，表示源码语言是ArkTS/TS/JS。
    SourceLang = 0x02,
    /// 拥有此标记的TaggedValue的data是一个偏移量，指向字符串，表示源文件的名称。
    SourceFile = 0x07,
}

/// https://gitee.com/openharmony/arkcompiler_runtime_core/blob/master/docs/file_format.md#methodtag
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

/// Field, Method, Class，都会包含一个 data 字段，数据类型是 TaggedValue[]。
pub struct TaggedValue {
    tag_value: uint8_t,
    /// 数据的长度，取决于 tag_value 的值。
    data: Vec<uint8_t>,
}

// TODO: 根据 Tag 类型，读取数据。
