use getset::CopyGetters;
use std::fmt;

use super::uint32_t;

use scroll::Pread;

/// ABC file header
/// 12*4 + 8 + 4 = 60
#[derive(Debug, Pread, CopyGetters, Default)]
#[get_copy = "pub"]
pub struct Header {
    /// 文件头魔数，值必须是'P' 'A' 'N' 'D' 'A' '\0' '\0' '\0'。
    magic: [u8; 8],
    /// 字节码文件除文件头魔数和本校验字段之外的内容的 adler32 校验和。
    checksum: [u8; 4],
    /// 字节码文件的版本号 (Version) 。
    version: [u8; 4],
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
    region_size: uint32_t,
    /// 一个偏移量，指向第一个 RegionIndex
    region_off: uint32_t,
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
            self.region_size,
            self.region_off,
        )
    }
}
