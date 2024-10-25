# rs-xabc

方舟字节码解析库

> [!WARNING]
>
> 1. 这是用于练习Rust的项目，不保证稳定。
> 2. 不保证解析所有版本的ABC文件。

## 功能

- 获取文件的基本信息

  - 类名
  - 方法名
  - 字符串
    - 字段
    - 方法

## 参考

- [方舟字节码文件格式](https://developer.huawei.com/consumer/cn/doc/harmonyos-guides-V5/arkts-bytecode-file-format-V5)
- [Panda Binary File Format](https://gitee.com/openharmony/arkcompiler_runtime_core/blob/master/static_core/docs/file_format.md)
- [arkcompiler_runtime_core/libpandafile](https://gitee.com/openharmony/arkcompiler_runtime_core/tree/master/libpandafile)
- [arkcompiler_runtime_core/disassembler](https://gitee.com/openharmony/arkcompiler_runtime_core/tree/master/disassembler)
- [Yricky/abcde](https://github.com/Yricky/abcde)
