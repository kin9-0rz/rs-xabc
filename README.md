# rs-xabc

## TODO

- [ ] 类名
  - [ ] 获取指定类名的信息
  - [ ] 获取所有的类名
- [ ] 方法名
  - [ ] 获取指定方法名的信息
  - [ ] 获取所以的方法信息
- [ ] 字段名，遍历
- [ ] 解析方法里面的指令
- [ ] 获取字符串(注：abc文件没有直接能够遍历字符串表的方式)
  - [ ] 字符串类型的字段值获取
  - [ ] 方法里面的字符串数据获取

## 参考

- [方舟字节码文件格式](https://developer.huawei.com/consumer/cn/doc/harmonyos-guides-V5/arkts-bytecode-file-format-V5)
- [Panda Binary File Format](https://gitee.com/openharmony/arkcompiler_runtime_core/blob/master/static_core/docs/file_format.md)
- [Yricky/abcde](https://github.com/Yricky/abcde)

## 指令的解析

opcode，操作码的解析，根据操作码，可以知道数据的格式。

- 指令的长度
- 数据的格式
  - opcode
  - PR
  - 立即数, imm
  - 索引；字符串、方法、数组; string_id, method_id, array_id

1. 读取第一个字节，那是 opcode

   - opcode有两类：

     1. 0x01，一个字节
     2. 0x08fb，2个字节；prefix opcode，前缀操作码，如throw.deletesuperproperty。

     如果是prefix opcode，就按照prefix opcode来解析。

2. 从 opcode 配置中获取数据。
   - 长度
   - 数据格式；pr、imm、string_id、method_id、array_id
   - ID16, `[opcode, id16]`
   - IMM16,`[opcode, imm16]`
