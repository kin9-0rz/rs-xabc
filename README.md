# rs-xabc

方舟字节码解析库

## 功能

- 获取文件的基本信息
  - 类名
  - 方法名
  - 字符串
    - 字段
    - 方法

## TODO

- [ ] 类名
  - [ ] 获取指定类名的信息
  - [ ] 获取所有的类名
- [ ] 方法名
  - [ ] 获取指定方法名的信息
  - [ ] 获取所以的方法信息
- [ ] 字段名，遍历
- [ ] 解析方法里面的指令
- [ ] 获取字符串(注：abc文件没有直接能够遍历字符串表的方式)。
  - [ ] 方法
  - [ ] 字段

## 参考

- [方舟字节码文件格式](https://developer.huawei.com/consumer/cn/doc/harmonyos-guides-V5/arkts-bytecode-file-format-V5)
- [Panda Binary File Format](https://gitee.com/openharmony/arkcompiler_runtime_core/blob/master/static_core/docs/file_format.md)
- [Yricky/abcde](https://github.com/Yricky/abcde)
