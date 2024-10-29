# xabc

## 安装

```bash
cargo install xabc
```

## 用法

```bash
Usage: xabc [OPTIONS] --path <PATH>

Options:
  -p, --path <PATH>      目标文件
  -i, --infos            输出文件信息
  -z, --classes          输出类列表
  -m, --methods          输出方法列表
  -s, --strings          输出字符串列表
  -c, --method <METHOD>  解析指定方法, 格式：类名->方法名，如: La/b/c;->mtd
  -h, --help             Print help
  -V, --version          Print version
```

## 例子

```bash
❯ xabc -p crates/parser/fixtures/demo.abc -z
L@ohos.app;
L@ohos.curves;
L@ohos.matrix4;
L@system.app;
L@system.curves;
L@system.matrix4;
L@system.router;
L_ESConcurrentModuleRequestsAnnotation;
L_ESSlotNumberAnnotation;
Lcom.example.myapplication/entry/ets/entryability/EntryAbility;
Lcom.example.myapplication/entry/ets/entrybackupability/EntryBackupAbility;
Lcom.example.myapplication/entry/ets/pages/Index;
```
