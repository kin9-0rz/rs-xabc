use memmap2::{Mmap, MmapOptions};
use std::collections::HashMap;
use std::{fs::File, path::Path};

use crate::bytecode::BytecodeMap;
use crate::class::Class;
use crate::class::ForeignClass;
use crate::code::Code;
use crate::header::Header;
use crate::lnp::LineNumberProgramIndex;
use crate::method::Method;
use crate::region::ClassRegionIndex;
use crate::region::FieldRegionIndex;
use crate::region::FieldType;
use crate::region::MethodStringLiteralRegionIndex;
use crate::region::ProtoRegionIndex;
use crate::region::Region;
use crate::region::RegionHeader;
use crate::source::Source;
use crate::string::ABCString;
use crate::{error, uint16_t};

use super::uint32_t;

use scroll::Pread;

/// 对外暴露的接口
pub struct AbcFile<T> {
    source: Source<T>,
    header: Header,
    /// offset -> Class 类定义
    // offset 可以确定 Region 范围
    classes: HashMap<uint32_t, Class>,
    foreign_classes: HashMap<uint32_t, ForeignClass>,
    regions: Vec<Region>,
    /// TODO: StringMap (offset: String)
    string_map: HashMap<uint32_t, ABCString>,
    /// TODO: Method Map (offset: String)
    method_map: HashMap<usize, String>,
}

impl<T> AbcFile<T>
where
    T: AsRef<[u8]>,
{
    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn regions(&self) -> &Vec<Region> {
        &self.regions
    }

    /// 获取所有的类名
    // TODO: 使用lru_cache，已经获取过，则不再重复获取
    pub fn get_classes_name(&self) -> Vec<String> {
        let mut class_names = Vec::new();
        for clz in self.classes.values() {
            let class_name = clz.name().str();
            class_names.push(class_name);
        }
        class_names
    }

    fn init_method_map(&mut self) {
        for clz in self.classes.values() {
            for (offset, method) in clz.method_map().iter() {
                let name = self.get_string_by_off(*method.name_off());
                self.method_map.insert(*offset as usize, name);
            }
        }
    }

    /// 获取所有的方法名
    // pub fn get_methods_name(&self) -> Vec<String> {}

    /// 获取所有的字符串
    pub fn get_strings(&self) -> String {
        // TODO: 从 Field 中获取
        // TODO: 从 Method 中获取
        todo!()
    }

    pub fn classes(&self) -> &HashMap<uint32_t, Class> {
        &self.classes
    }

    pub fn parse(&mut self) {
        self.parse_header();
        self.parse_class_index();
        self.parse_region();
        self.parse_code();
        println!("StringID{} -> {:?}", 4429, self.get_string_by_off(4429));
        println!("StringID{} -> {:?}", 1087, self.get_string_by_off(1087));
        println!("LiteralID{} -> {:?}", 5550, self.get_string_by_off(5550));
        println!("LiteralID{} -> {:?}", 4821, self.get_string_by_off(4821));
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

    pub fn parse_code(&mut self) {
        for item in &self.classes {
            let offset = item.0;
            let region = self.get_region(*offset as usize).unwrap();
            let clazz = item.1;

            let class_name = clazz.name().str();
            if class_name
                != "Lcom.example.myapplication/entry/ets/entrybackupability/EntryBackupAbility;"
            {
                continue;
            }
            for method in clazz.methods().iter() {
                let name = self.get_string_by_off(*method.name_off());
                if name != "func_main_0" {
                    continue;
                }
                println!("{} -> {}", class_name, name);
                let data = method.method_data();
                let code_off = data.code_off();
                let code = self
                    .source
                    .as_ref()
                    .pread::<Code>(*code_off as usize)
                    .unwrap();
                println!("{} -> {:?}", code_off, code);

                let bytecode_map = BytecodeMap::new();
                // TODO: 解析字节码的时候，它可以访问 Region，这样才用可能获取数据。
                bytecode_map.parse(&code, region, self.source.as_ref());
            }
        }
    }

    /// 解析 Class
    fn parse_class_index(&mut self) {
        let num_classes = self.header().num_classes() as usize;
        let class_idx_off = self.header().class_idx_off() as usize;

        // 一次性解析所有的Class
        for i in 0..num_classes {
            let off = class_idx_off + i * 4;
            let class_idx_off = self.source.as_ref().pread::<uint32_t>(off).unwrap();

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

    pub fn parse_lnp_idx(&mut self) {
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

    pub fn parse_literalarray_idx(&mut self) {}

    pub fn get_class_name_by_off(&self, idx: uint32_t) -> ABCString {
        if self.is_foreign_off(idx) {
            let v = self.foreign_classes.get(&idx).unwrap();
            return v.name().clone();
        }

        self.classes.get(&idx).unwrap().name().clone()
    }

    /// 按索引查找类型定义，在这里找 [`ClassRegionIndex`]
    pub fn get_field_type_by_class_idx(&self, idx: &uint16_t) -> String {
        // FIXME: 如果 regions 有多个，我怎么知道是哪个？
        let mut clz = String::new();
        let idx = *idx as usize;
        self.regions.iter().for_each(|region| {
            let class_region_idx = region.class_region_idx();
            let class_idx = class_region_idx.get(&idx);
            clz = class_idx.name.clone();
        });

        clz
    }

    fn get_primitive_type(&self, i: uint32_t) -> FieldType {
        let names = [
            "i8", "u8", "i16", "u16", "i32", "u32", "f32", "f64", "i64", "u64", "any",
        ];

        let n = names[i as usize];
        FieldType {
            name: n.to_string(),
        }
    }

    pub fn parse_field_type(&mut self, idx: uint32_t) -> FieldType {
        if idx <= 0xb {
            return self.get_primitive_type(idx);
        }

        let item = self.get_class_name_by_off(idx).to_string();
        FieldType { name: item }
    }

    pub fn get_string_by_off(&self, off: uint32_t) -> String {
        self.source
            .as_ref()
            .pread::<ABCString>(off as usize)
            .unwrap()
            .str()
    }

    /// 解析Region
    pub fn parse_region(&mut self) {
        for i in 0..self.header().region_size() as usize {
            let off = self.header().region_off() as usize + i * 4;
            let region_header = self.source.as_ref().pread::<RegionHeader>(off).unwrap();

            // 解析 ClassRegionIndex
            let mut class_region_idx = ClassRegionIndex::default();
            let class_idx_off = region_header.class_idx_off() as usize;
            for i in 0..region_header.class_idx_size() as usize {
                let off = class_idx_off + i * 4;

                // 一个FiedType 大小是u32
                let class_offset = self
                    .source
                    .as_ref()
                    .pread_with::<uint32_t>(off, scroll::LE)
                    .unwrap();

                let f = self.parse_field_type(class_offset);
                println!("{} -> {:?}", off, &f);
                class_region_idx.push(f);
            }

            // 解析 MethodStringLiteralRegionIndex
            let msl_off = region_header.method_string_literal_region_idx_off() as usize;
            let mut mslr_idx = MethodStringLiteralRegionIndex::default();
            for i in 0..region_header.method_string_literal_region_idx_size() as usize {
                let offset = self
                    .source
                    .as_ref()
                    .pread::<uint32_t>(msl_off + i * 4)
                    .unwrap();
                mslr_idx.push(offset);
                // NOTE: 存放的是偏移地址，但是，这个偏移地址的内容是啥，不知道。
                // 只有解析代码的时候，才知道。
                println!("{} -> {}", i, offset);
                // println!("{} -> {:?}", offset, self.get_string_by_off(offset));

                // FIXME: 这个 Region 里面有3类数据，怎么区分？
                // 难道是实时，解析？
            }

            // 解析 FieldRegionIndex
            let mut field_idx = FieldRegionIndex::default();
            let field_idx_off = region_header.field_idx_off() as usize;
            let field_idx_size = region_header.field_idx_size() as usize;
            if field_idx_size <= 65536 {
                for i in 0..region_header.field_idx_size() as usize {
                    let offset = self
                        .source
                        .as_ref()
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
                        .as_ref()
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

    fn parse_header(&mut self) {
        self.header = self.source.as_ref().pread::<Header>(0).unwrap();
    }
}

/// 用于读取 `Abc` 文件
pub struct AbcReader {}

impl AbcReader {
    /// Try to read a `Dex` from the given path, returns error if
    /// the file is not a dex or in case of I/O errors
    pub fn from_file<P: AsRef<Path>>(file: P) -> Result<AbcFile<Mmap>, error::Error> {
        let map = unsafe { MmapOptions::new().map(&File::open(file.as_ref())?)? };
        let source = Source::new(map);
        Ok(AbcFile {
            source: source.clone(),
            header: Header::default(),
            classes: HashMap::new(),
            foreign_classes: HashMap::new(),
            regions: Vec::new(),
            string_map: HashMap::new(),
            method_map: HashMap::new(),
        })
    }

    pub fn parse_header(&mut self) {
        todo!()
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
