use crate::{error, str::ABCString, uint32_t};
use getset::Getters;
use scroll::ctx;
use scroll::Pread;
use scroll::Uleb128;

#[derive(Debug, Getters)]
#[get = "pub"]
pub struct ForeignClass {
    name: ABCString,
}

impl<'a> ctx::TryFromCtx<'a, scroll::Endian> for ForeignClass {
    type Error = error::Error;
    fn try_from_ctx(source: &'a [u8], _: scroll::Endian) -> Result<(Self, usize), Self::Error> {
        let name = source.pread::<ABCString>(0).unwrap();

        Ok((ForeignClass { name }, source.len()))
    }
}

#[derive(Debug, Getters)]
#[get = "pub"]
pub struct Class {
    /// 类名
    #[get = "pub"]
    name: ABCString,
    supper_class: String,
    access_flags: Vec<String>,
    num_fields: u64,
    num_methods: u64,
    // TODO: ClassData
    //class_data: Vec<TaggedValue>,

    // NOTE: 下面这2个在这里无法获得，只能从索引区域获得。
    //fields: Vec<Field>,
    //methods: Vec<Method>,
}

impl<'a> ctx::TryFromCtx<'a, scroll::Endian> for Class {
    type Error = error::Error;
    fn try_from_ctx(source: &'a [u8], _: scroll::Endian) -> Result<(Self, usize), Self::Error> {
        let mut off = 0;
        let name = source.pread::<ABCString>(0).unwrap();
        off += name.length();

        let supper_class_off = source.pread::<uint32_t>(off).unwrap();
        let mut supper_class = String::new();
        if supper_class_off != 0 {
            let str = source
                .pread::<ABCString>(supper_class_off as usize)
                .unwrap();
            supper_class = str.str();
        }
        off += 4;

        let off = &mut off;
        let access_flags = Uleb128::read(source, off).unwrap();
        let access_flags = ClassAccessFlags::parse(access_flags);
        let num_fields = Uleb128::read(source, off).unwrap();
        let num_methods = Uleb128::read(source, off).unwrap();

        // TODO: 需要解析剩余的数据。
        // TODO: ClassData
        // TODO: Fields
        // TODO: Methods

        Ok((
            Class {
                name,
                supper_class,
                access_flags,
                num_fields,
                num_methods,
            },
            source.len(),
        ))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClassAccessFlags {
    Public = 0x0001,
    Final = 0x0010,
    Super = 0x0020,
    Interface = 0x0200,
    Abstract = 0x0400,
    Synthetic = 0x1000,
    Annotation = 0x2000,
    Enum = 0x4000,
}

impl ClassAccessFlags {
    pub fn parse(value: u64) -> Vec<String> {
        let mut access_flags: Vec<String> = Vec::new();

        let flags = [
            ClassAccessFlags::Public,
            ClassAccessFlags::Final,
            ClassAccessFlags::Super,
            ClassAccessFlags::Interface,
            ClassAccessFlags::Abstract,
            ClassAccessFlags::Synthetic,
            ClassAccessFlags::Annotation,
            ClassAccessFlags::Enum,
        ]
        .to_vec();

        for flag in flags {
            let x = flag as u64;
            if value & x != 0 {
                //println!("{:?}", flag);
                access_flags.push(format!("{:?}", flag));
            }
        }

        return access_flags;
    }
}

/// ClassIndex 结构旨在允许运行时按名称快速查找类型定义。
#[derive(Debug, Getters)]
#[get = "pub"]
pub struct ClassIndex {
    /// 数组长度; 由 `Header` 的 num_classes 指定。
    num: u8,
    offsets: Vec<uint32_t>,
}
