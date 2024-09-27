use std::{
    fmt,
    fs::File,
    io::{self, Cursor, Seek},
    path::Path,
};

use anyhow::Result;

use binrw::BinRead;
use getset::{CopyGetters, Getters};

use crate::data_type::{uint32_t, uint8_t, uleb128_t, TaggedValue};

/// ABC file header
/// 12*4 + 8 + 4 = 60
/// 字节码文件中结构的引用方式包括偏移量和索引。
/// 偏移量是一个32位长度的值，表示当前结构的起始位置在字节码文件中相对于文件头的距离，从0开始计算。
/// 索引是一个16位长度的值，表示当前结构在索引区域中的位置，此机制将在IndexSection章节描述。
#[derive(BinRead, Debug, CopyGetters)]
#[br(little)]
#[get_copy = "pub"]
pub struct Header {
    /// 文件头魔数，值必须是'P' 'A' 'N' 'D' 'A' '\0' '\0' '\0'。
    magic: [uint8_t; 8],
    /// 字节码文件除文件头魔数和本校验字段之外的内容的adler32校验和。
    checksum: uint32_t,
    /// 字节码文件的版本号 (Version) 。
    version: [uint8_t; 4],
    /// 字节码文件的大小，以字节为单位。
    file_size: uint32_t,
    /// 一个偏移量，指向外部区域。外部区域中仅包含类型为ForeignClass或ForeignMethod的元素。foreign_off指向该区域的第一个元素。
    foreign_off: uint32_t,
    /// 外部区域的大小，以字节为单位。
    foreign_size: uint32_t,
    /// ClassIndex结构中元素的数量，即文件中定义的Class的数量。
    num_classes: uint32_t,
    /// 一个偏移量，指向ClassIndex。
    class_idx_off: uint32_t,
    /// LineNumberProgramIndex结构中元素的数量，即文件中定义的Line number program的数量。
    num_lnps: uint32_t,
    /// 一个偏移量，指向LineNumberProgramIndex。
    lnp_idx_off: uint32_t,
    /// 方舟字节码文件内部使用的保留字段。
    reserved: uint32_t,
    reserved2: uint32_t,
    /// IndexSection 结构中元素的数量，即文件中 IndexHeader 的数量。
    num_index_regions: uint32_t,
    /// 一个偏移量，指向 IndexSection。
    index_section_off: uint32_t,
}

impl Header {
    //pub fn parse(bytes: &[u8]) -> error::Result<Self> {
    //pub fn parse(bytes: &[u8]) -> Result<()> {
    //let mut cursor = Cursor::new(bytes);
    //let header = Header::read(&mut cursor)?;
    //Ok(header)
    //Ok(())
    //}
}

impl fmt::Display for Header {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let magic = String::from_utf8_lossy(&self.magic);
        let version = self
            .version
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(".");

        write!(
            f,
            "
magic: {}
checksum: {}
版本: {}
文件大小: {}
外部区域偏移: {}
外部区域大小: {}
类的数量: {}
类索引的偏移: {}
行号索引数量: {}
行号索引偏移: {}
索引头的数量: {}
索引头的偏移: {}
",
            magic,
            self.checksum,
            version,
            self.file_size,
            self.foreign_off,
            self.foreign_size,
            self.num_classes,
            self.class_idx_off,
            self.num_lnps,
            self.lnp_idx_off,
            self.num_index_regions,
            self.index_section_off,
        )
    }
}

#[derive(Debug, Getters, CopyGetters)]
pub struct ABCFile {
    #[get = "pub"]
    header: Header,
    //classes: Vec<ABCClass>,
}

//TypeDescriptor是类(Class) 名称的格式，由'L'、'_'、ClassName和';'组成：L_ClassName;。其中，ClassName是类的全名，名字中的'.'会被替换为'/'。
#[derive(Debug)]
pub struct TypeDescriptor {}

#[derive(Debug)]
pub struct Class {
    /// String
    /// Class的名称，命名遵循TypeDescriptor语法。
    name: TypeDescriptor,
    reserved: uint32_t,
    /// Class的访问标志，是ClassAccessFlag的组合。
    /// TODO: 什么是组合？不是其中一个么？
    access_flags: uleb128_t,
    /// Class的字段的数量。
    num_fields: uleb128_t,
    /// Class的方法的数量。
    num_methods: uleb128_t,
    /// 不定长度的数组，数组中每个元素都是TaggedValue类型，元素的标记是ClassTag类型，数组中的元素按照标记递增排序（0x00标记除外）。
    class_data: Vec<TaggedValue>,
}

pub struct ABCReader;

#[derive(Debug, CopyGetters)]
//#[br(little)]
pub struct ABCString {
    /// 值为len << 1 | is_ascii，其中len是字符串在UTF-16编码中的大小，is_ascii标记该字符串是否仅包含ASCII字符，可能的值是0或1。
    utf16_length: uleb128_t,
    // 以'\0'结尾的MUTF-8编码字符序列。
    //data: Vec<uint8_t>,
}

#[derive(BinRead, Debug, CopyGetters)]
#[br(little)]
pub struct Offset {
    pub value: uint32_t,
}

#[derive(Debug, CopyGetters)]
pub struct ClassIndex {
    pub num_classes: uint32_t,
    /// 一个数组，数组中每个元素的值是一个指向 Class 的偏移量。
    /// 数组中的元素根据类的名称进行排序，名称遵循 TypeDescriptor 语法。
    /// 数组长度由 Header 中的 num_classes 指定。
    pub offsets: Vec<Offset>,
}

impl ClassIndex {
    pub fn parse(self, file: &mut File) -> Result<Vec<Offset>> {
        let mut offsets = Vec::with_capacity(self.num_classes as usize);
        for _ in 0..self.num_classes {
            let offset = Offset::read(file)?;
            offsets.push(offset);
        }
        Ok(offsets)
    }
}

impl ABCReader {
    /// TODO: read 返回一个ABC对象，这个对象可以读取方法和数据。
    pub fn from_file<P: AsRef<Path>>(file: P) -> Result<ABCFile> {
        let mut file = File::open(file)?;
        println!("Current read position: {}", file.stream_position()?);
        let header = Header::read(&mut file)?;
        println!("Current read position: {}", file.stream_position()?);

        let class_index = ClassIndex {
            num_classes: header.num_classes,
            offsets: vec![],
        };
        let offsets = class_index.parse(&mut file)?;
        println!("offsets: {:?}", offsets);
        println!("当前偏移: {}", file.stream_position()?);

        for offset in offsets.iter() {
            println!("Offset: {:?}", offset);
        }

        //file.seek(io::SeekFrom::Start(offsets[0].value as u64))?;
        //println!("当前偏移: {}", file.stream_position()?);
        // TODO 读取一个类；
        // TODO 读取一个uleb128_t类型的数据
        //uleb128_t::read(&mut file).unwrap();

        //file.seek(io::SeekFrom::Start(offsets[0].value as u64));
        // 打印当前文件的位置
        Ok(ABCFile { header })
    }

    /// Loads a `Dex` from a `Vec<u8>`
    pub fn from_vec<B: AsRef<[u8]>>(buf: B) -> Result<Header> {
        let mut cursor = Cursor::new(buf.as_ref());
        let header = Header::read(&mut cursor).unwrap();
        Ok(header)
    }
}
