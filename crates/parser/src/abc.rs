use memmap2::Mmap;
use std::collections::{HashMap, HashSet};
use std::{
    fs::{self, File},
    io::Read,
    path::Path,
};

use crate::bytecode::BytecodeParser;
use crate::class::{Class, ForeignClass};
use crate::code::Code;
use crate::header::Header;
use crate::lnp::LineNumberProgramIndex;
use crate::region::{
    ClassRegionIndex, FieldRegionIndex, FieldType, MethodStringLiteralRegionIndex,
    ProtoRegionIndex, Region, RegionHeader,
};
use crate::source::Source;
use crate::string::ABCString;
use crate::{error, init_logging, literal};

use super::uint32_t;

use scroll::Pread;

/// 对外暴露的接口
pub struct AbcFile<T> {
    source: Source<T>,
    header: Header,
    classes: HashMap<uint32_t, Class>,
    foreign_classes: HashMap<uint32_t, ForeignClass>,
    regions: Vec<Region>,
    literal_array_map: HashMap<usize, String>,
}

impl<T> AbcFile<T>
where
    T: AsRef<[u8]>,
{
    pub fn header(&self) -> &Header {
        &self.header
    }

    fn parse_header(&mut self) {
        self.header = self.source.pread::<Header>(0).unwrap();
    }

    pub fn classes(&self) -> &HashMap<uint32_t, Class> {
        &self.classes
    }

    fn init(&mut self) {
        self.parse_header();
        self.parse_class_index();
        self.parse_region_index();
        self.parse_literal_array_index();
    }

    /// 解析 LiteralArray 并将数据存放起来
    fn parse_literal_array_index(&mut self) {
        self.literal_array_map = literal::parse_literal_array_index(
            self.source.as_ref(),
            self.header.literalarray_idx_off(),
            self.header.literalarrays_size(),
            &self.regions,
        );
    }

    fn get_region(&self, offset: usize) -> Option<&Region> {
        let mut result: Option<&Region> = None;
        for one in self.regions.iter() {
            let is_it = one.is_here(offset);
            if is_it {
                result = Some(one);
                break;
            }
        }

        result
    }

    pub fn parse_method(&self, name: String) {
        let arr = name.split("->").collect::<Vec<&str>>();
        let target_clazz = arr[0];
        let target_method = arr[1];

        let bytecode_map = BytecodeParser::new();
        for item in &self.classes {
            let offset = item.0;
            let region = self.get_region(*offset as usize).unwrap();
            let clazz = item.1;

            let class_name = clazz.name().str();
            if target_clazz == class_name {
                for (_offset, method) in clazz.method_map().iter() {
                    let _name = self.get_string_by_off(*method.name_off());
                    if target_method == _name {
                        let data = method.method_data();
                        let code_off = data.code_off();
                        let code = self
                            .source
                            .as_ref()
                            .pread::<Code>(*code_off as usize)
                            .unwrap();
                        bytecode_map.parse(
                            &code,
                            region,
                            self.source.as_ref(),
                            &self.literal_array_map,
                        );

                        break;
                    }
                }

                break;
            }
        }
    }
    /// 解析 Code，按需解析
    // TODO: 解析整个文件，则输出到文件中？
    // TODO: 解析指定类？

    pub fn parse_code(&mut self) {
        let bytecode_map = BytecodeParser::new();
        for item in &self.classes {
            let offset = item.0;
            let region = self.get_region(*offset as usize).unwrap();
            let clazz = item.1;

            let class_name = clazz.name().str();
            tracing::debug!("Class Name -> {}", class_name);
            for field in clazz.fields() {
                let off = field.name_off();
                let type_idx = *field.type_idx();
                let type_name = region.get_class_name(type_idx as usize);
                tracing::debug!(
                    "Field Name -> {}:{}",
                    self.get_string_by_off(*off),
                    type_name
                );
                // TODO: 解析 Field 的值
            }

            // TODO: 调整代码的输出
            for (_offset, method) in clazz.method_map().iter() {
                let name = self.get_string_by_off(*method.name_off());
                println!("\n[方法]{}->{}", class_name, name);
                let data = method.method_data();
                let code_off = data.code_off();
                let code = self
                    .source
                    .as_ref()
                    .pread::<Code>(*code_off as usize)
                    .unwrap();
                bytecode_map.parse(&code, region, self.source.as_ref(), &self.literal_array_map);
            }
        }
    }

    /// 解析 Class
    fn parse_class_index(&mut self) {
        let num_classes = self.header.classes_size() as usize;
        let class_idx_off = self.header.class_idx_off() as usize;

        // 一次性解析所有的Class
        for i in 0..num_classes {
            let off = class_idx_off + i * 4;
            let class_idx_off = self.source.pread::<uint32_t>(off).unwrap();

            let is_foreign_class = self.is_foreign_off(class_idx_off);

            if is_foreign_class {
                let class = self
                    .source
                    .as_ref()
                    .pread::<ForeignClass>(class_idx_off as usize)
                    .unwrap();
                self.foreign_classes.insert(class_idx_off, class);
            } else {
                let class = self
                    .source
                    .as_ref()
                    .pread::<Class>(class_idx_off as usize)
                    .unwrap();
                self.classes.insert(class_idx_off, class);
            }
        }
    }

    #[allow(dead_code)]
    fn parse_lnp_idx(&mut self) {
        // NOTE: 解析行号程序，未来再说。
        let mut lnp_idx = LineNumberProgramIndex::default();
        let num_lnp = self.header().num_lnps() as usize;
        let lnp_off = self.header().lnp_idx_off() as usize;
        for i in 0..num_lnp {
            let offset = self
                .source
                .as_ref()
                .pread::<uint32_t>(lnp_off + i * 4)
                .unwrap();

            lnp_idx.push(offset);
        }
    }

    fn get_class_name_by_offset(&self, idx: uint32_t) -> ABCString {
        if self.is_foreign_off(idx) {
            let v = self.foreign_classes.get(&idx).unwrap();
            return v.name().clone();
        }

        self.classes.get(&idx).unwrap().name().clone()
    }

    /// 获取基本类型
    fn get_primitive_type(&self, i: uint32_t) -> FieldType {
        let names = [
            "i8", "u8", "i16", "u16", "i32", "u32", "f32", "f64", "i64", "u64", "any",
        ];

        let n = names[i as usize];
        FieldType {
            name: n.to_string(),
        }
    }

    /// 解析字段类型
    fn parse_field_type(&mut self, idx: uint32_t) -> FieldType {
        if idx <= 0xb {
            return self.get_primitive_type(idx);
        }

        let item = self.get_class_name_by_offset(idx).to_string();
        FieldType { name: item }
    }

    fn get_string_by_off(&self, off: uint32_t) -> String {
        self.source
            .as_ref()
            .pread::<ABCString>(off as usize)
            .unwrap()
            .str()
    }

    /// 解析 RegionIndex
    fn parse_region_index(&mut self) {
        for i in 0..self.header().region_size() as usize {
            let off = self.header().region_off() as usize + i * 4;
            let region_header = self.source.pread::<RegionHeader>(off).unwrap();

            // 解析 ClassRegionIndex
            let mut class_region_idx = ClassRegionIndex::default();
            let class_idx_off = region_header.class_idx_off() as usize;
            for i in 0..region_header.class_idx_size() as usize {
                let off = class_idx_off + i * 4;

                // 一个FiedType 大小是u32
                let class_offset = self.source.pread_with::<uint32_t>(off, scroll::LE).unwrap();
                let f = self.parse_field_type(class_offset);
                // tracing::debug!("FieldType: {} -> {:?}", off, &f);
                class_region_idx.push(f);
            }

            // 解析 MethodStringLiteralRegionIndex
            let msl_off = region_header.method_string_literal_region_idx_off() as usize;
            let mut mslr_idx = MethodStringLiteralRegionIndex::default();
            for i in 0..region_header.method_string_literal_region_idx_size() as usize {
                let offset = self.source.pread::<uint32_t>(msl_off + i * 4).unwrap();
                mslr_idx.push(offset);
            }

            // 解析 FieldRegionIndex
            let mut field_idx = FieldRegionIndex::default();
            let field_idx_off = region_header.field_idx_off() as usize;
            let field_idx_size = region_header.field_idx_size() as usize;
            if field_idx_size <= 65536 {
                for i in 0..region_header.field_idx_size() as usize {
                    let offset = self
                        .source
                        .pread_with::<uint32_t>(field_idx_off + i * 4, scroll::LE)
                        .unwrap();
                    field_idx.push(offset);
                }
            }

            // 解析 ProtoRegionIndex
            let mut proto_idx = ProtoRegionIndex::default();
            let proto_idx_off = region_header.proto_idx_off() as usize;
            let proto_idx_size = region_header.proto_idx_size() as usize;
            if proto_idx_size <= 65536 {
                for i in 0..region_header.proto_idx_size() as usize {
                    let offset = self
                        .source
                        .pread_with::<uint32_t>(proto_idx_off + i * 4, scroll::LE)
                        .unwrap();
                    proto_idx.push(offset);
                }
            }

            let region = Region::new(
                region_header,
                class_region_idx,
                mslr_idx,
                field_idx,
                proto_idx,
            );
            self.regions.push(region);
        }
    }

    /// 判断数据是否在外部区域
    fn is_foreign_off(&self, class_idx: u32) -> bool {
        let start = self.header().foreign_off();
        let end = start + self.header().foreign_size();
        start <= class_idx && class_idx <= end
    }

    /// 获取所有的类名
    pub fn get_class_names(&self) -> Vec<String> {
        let mut class_names = Vec::new();
        for clz in self.classes.values() {
            let class_name = clz.name().str();
            class_names.push(class_name);
        }
        class_names
    }

    /// 获取所有的方法名
    pub fn get_method_names(&self) -> Vec<String> {
        let mut method_names = Vec::new();
        for clz in self.classes.values() {
            let class_name = clz.name().str();
            for (_, method) in clz.method_map().iter() {
                let name = self.get_string_by_off(*method.name_off());
                method_names.push(class_name.to_string() + "->" + &name);
            }
        }
        method_names
    }

    /// 获取所有的字符串
    pub fn get_strings(&self) -> Vec<String> {
        let mut method_offsets = HashSet::new();
        for clz in self.classes.values() {
            for (offset, _) in clz.method_map().iter() {
                method_offsets.insert(offset);
            }
        }

        let mut strings = Vec::new();
        for region in self.regions.iter() {
            let offsets = region.method_string_literal_region_idx().offsets();
            for offset in offsets {
                let offset = *offset as usize;
                if method_offsets.contains(&offset) {
                    continue;
                }

                if self.literal_array_map.contains_key(&offset) {
                    continue;
                }

                // tracing::debug!("string offset -> {}", offset);
                let string = self.get_string_by_off(offset as u32);
                if string == "-utf8-error-" {
                    tracing::warn!("{} -> 解析错误，不是字符串", offset);
                    continue;
                }
                tracing::debug!("{} -> {}", offset, string);
                strings.push(string);
            }
        }

        strings
    }
}

/// 用于读取 `Abc` 文件
pub struct AbcReader {}

// 100MB以上的文件为大文件
const LARGE_FILE: u64 = 100 * 1024 * 1024;

impl AbcReader {
    fn read_file_to_vec<P: AsRef<Path>>(path: P) -> Result<Vec<u8>, error::Error> {
        let metadata = fs::metadata(path.as_ref())?;
        let file_size = metadata.len();

        if file_size > LARGE_FILE {
            let file = File::open(path.as_ref())?;
            let mmap = unsafe { Mmap::map(&file)? };
            return Ok(Vec::from(&mmap[..]));
        }

        let mut file = File::open(path.as_ref())?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        Ok(buffer)
    }

    /// 从文件中加载 Abc 文件到内存
    pub fn from_file<P>(file: P) -> Result<AbcFile<Vec<u8>>, error::Error>
    where
        P: AsRef<Path>,
    {
        let buf = AbcReader::read_file_to_vec(file)?;
        AbcReader::from_vec(buf)
    }

    pub fn from_array(buf: &[u8]) -> Result<AbcFile<Vec<u8>>, error::Error> {
        let buf = Vec::from(buf);
        AbcReader::from_vec(buf)
    }

    pub fn from_vec(buf: Vec<u8>) -> Result<AbcFile<Vec<u8>>, error::Error> {
        init_logging();

        let source = Source::new(buf);
        let mut abc_file = AbcFile {
            source: source.clone(),
            header: Header::default(),
            classes: HashMap::new(),
            foreign_classes: HashMap::new(),
            regions: Vec::new(),
            literal_array_map: HashMap::new(),
        };
        abc_file.init();

        Ok(abc_file)
    }
}
