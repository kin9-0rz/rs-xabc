use crate::field::Field;
use crate::uint8_t;
use crate::{error, string::ABCString, uint32_t};
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
    /// 类的访问标志
    access_flags: Vec<String>,
    num_fields: u64,
    num_methods: u64,
    // class_data: Vec<TaggedValue>,
    fields: Vec<Field>,
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

        // let mut offset = *off;

        // TODO: ClassData
        'l: loop {
            print!("{} ", *off);
            let tag_value = source.pread::<uint8_t>(*off).unwrap();
            *off += 1;
            println!(" -> {} ", *off);
            match tag_value {
                0x00 => {
                    println!("NOTHING: exit\n");
                    break 'l;
                }
                0x01 => {
                    println!("INTERFACES");
                }
                0x02 => {
                    let data = source.pread::<uint8_t>(*off).unwrap();
                    *off += 1;
                    print!("SOURCE_LANG -> {}", data);
                }
                0x03 => {
                    println!("RUNTIME_ANNOTATION");
                }
                0x04 => {
                    println!("ANNOTATION");
                }
                0x05 => {
                    println!("RUNTIME_TYPE_ANNOTATION");
                }
                0x06 => {
                    println!("TYPE_ANNOTATION");
                }
                0x07 => {
                    println!("SOURCE_FILE");
                }
                _ => {
                    println!("Error! -> UNKNOWN: {}", tag_value);
                    break 'l;
                }
            }
        }

        let mut offset = *off;
        let mut fields = Vec::new();
        for _ in 0..num_fields {
            let field = source.pread::<Field>(offset).unwrap();
            println!("{:?}", field);
            let size = *field.size();
            offset += size;
            fields.push(field);
        }

        // TODO: Methods

        Ok((
            Class {
                name,
                supper_class,
                access_flags,
                num_fields,
                num_methods,
                fields,
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

        access_flags
    }
}

/// ClassIndex 结构旨在允许运行时按名称快速查找类型定义。
#[derive(Debug, Getters)]
#[get = "pub"]
pub struct ClassIndex {
    /// 指向 Class 或 ForeignClass
    offsets: Vec<uint32_t>,
}

impl ClassIndex {
    pub fn push(&mut self, offset: uint32_t) {
        self.offsets.push(offset);
    }
}
