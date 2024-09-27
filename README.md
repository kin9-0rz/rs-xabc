# rs-xabc

- RegionIndex 的数据先全部初始化到某个数据结构中；后续需要索引。
  - 方法索引；
  - 字段索引;
- ClassIndex ，主要是读取 Class 的信息，如果需要它的字段、方法等信息，则从 RegionIndex 中读取。
- 方法索引结构

```toml
- 文件信息
- 外部
  - 外部类
  - 外部方法
- classes
- strings
- methods
```

## 参考

- binrw
- scroll
- deku: https://docs.rs/deku/latest/deku/index.html, 它有一个功能读，一直读到\0。until。
- https://gitee.com/openharmony/arkcompiler_runtime_core/blob/master/static_core/docs/file_format.md
