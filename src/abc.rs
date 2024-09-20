use getset::{CopyGetters, Getters};
use log::debug;
use log::info;
use memmap2::{Mmap, MmapOptions};
use std::fmt;
use std::{fs::File, path::Path};

use crate::class::Class;
use crate::error;
use crate::index::RegionIndex;
use crate::method::Method;
use crate::source::Source;

use super::uint32_t;
use super::uint8_t;
//use super::Result;

use scroll::{ctx, Pread};

/// ABC file header
/// 12*4 + 8 + 4 = 60
#[derive(Debug, Pread, CopyGetters)]
#[get_copy = "pub"]
pub struct Header {
    /// 文件头魔数，值必须是'P' 'A' 'N' 'D' 'A' '\0' '\0' '\0'。
    magic: [uint8_t; 8],
    /// 字节码文件除文件头魔数和本校验字段之外的内容的 adler32 校验和。
    checksum: [uint8_t; 4],
    /// 字节码文件的版本号 (Version) 。
    version: [uint8_t; 4],
    /// 字节码文件的大小，以字节为单位。
    file_size: uint32_t,
    /// 一个偏移量，指向外部区域。外部区域中仅包含类型为 ForeignClass 或ForeignMethod的元素。foreign_off指向该区域的第一个元素。
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
    /// LiteralArrayIndex 的数量
    num_literalarrays: uint32_t,
    /// 指向 LiteralArrayIndex 的偏移量
    literalarray_idx_off: uint32_t,
    /// RegionIndex 的数量
    num_index_regions: uint32_t,
    /// 一个偏移量，指向IndexSection。
    index_section_off: uint32_t,
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
checksum: {:?}
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

/// TODO: 它可能不是必要的
#[derive(Debug, Getters, CopyGetters)]
pub(crate) struct AbcInner {
    /// 头
    #[get = "pub"]
    header: Header,
}

impl AbcInner {
    pub fn file_size(&self) -> uint32_t {
        self.header.file_size
    }

    pub fn foreign_off(&self) -> uint32_t {
        self.header.foreign_off
    }

    pub fn foreign_size(&self) -> uint32_t {
        self.header.foreign_size
    }

    pub fn num_classes(&self) -> uint32_t {
        self.header.num_classes
    }
}

impl<'a> ctx::TryFromCtx<'a, scroll::Endian> for AbcInner {
    type Error = error::Error;
    fn try_from_ctx(source: &'a [u8], le: scroll::Endian) -> Result<(Self, usize), Self::Error> {
        println!(" -> hello");
        info!("xxxx --->>>");
        debug!("malformed dex: size < minimum header size");
        let header = source.pread_with::<Header>(0, scroll::LE)?;
        debug!("malformed dex: size < minimum header size");
        Ok((AbcInner { header }, source.len()))
    }
}

pub struct Abc<T> {
    source: Source<T>,
    header: Header,
    region_index: RegionIndex<T>,
}

impl<T> Abc<T>
where
    T: AsRef<[u8]>,
{
    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn parse_region_index(&self) {
        self.region_index.parse();
    }

    /// 获取所有的字符串
    pub fn strings(&self) -> Vec<String> {
        todo!()
    }

    /// 获取所有的类
    pub fn classes(&self) -> Vec<Class> {
        todo!()
    }

    /// 获取所有的方法
    pub fn methods(&self) -> Vec<Method> {
        todo!()
    }
}

/// 用于读取 `Abc` 文件
pub struct AbcReader;

impl AbcReader {
    /// Try to read a `Dex` from the given path, returns error if
    /// the file is not a dex or in case of I/O errors
    pub fn from_file<P: AsRef<Path>>(file: P) -> Result<Abc<Mmap>, error::Error> {
        let map = unsafe { MmapOptions::new().map(&File::open(file.as_ref())?)? };
        let header = map.pread_with::<Header>(0, scroll::LE)?;
        let source = Source::new(map);
        /// FIXME: 它不能使用 pread 方法
        source.pread(0);
        //let x = source.as_ref();
        //x.pread_with::<AbcInner>(0, scroll::LE)?;
        let region_index = RegionIndex::new(
            source.clone(),
            header.num_index_regions,
            header.index_section_off,
        );

        Ok(Abc {
            source: source.clone(),
            header,
            region_index,
        })
    }

    // Loads a `Dex` from a `Vec<u8>`
    //pub fn from_vec<B: AsRef<[u8]>>(buf: B) -> Result<Abc<B>> {
    //    let inner: DexInner = buf.as_ref().pread(0)?;
    //    let source = Source::new(buf);
    //    let cache = Strings::new();
    //    Ok(Abc {
    //        source: source.clone(),
    //        strings: cache,
    //        inner,
    //    })
    //}
}
