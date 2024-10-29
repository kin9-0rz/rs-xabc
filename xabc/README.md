# rs-xabc

> ABC文件解析工具

![Crates.io Version](https://img.shields.io/crates/v/xabc?style=for-the-badge) ![Crates.io Total Downloads](https://img.shields.io/crates/d/xabc?style=for-the-badge) ![Crates.io License](https://img.shields.io/crates/l/xabc?style=for-the-badge)

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
❯ xabc -p xabc-lib/fixtures/demo.abc -z
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


➜ xabc -p xabc-lib/fixtures/demo.abc -c "Lcom.example.myapplication/entry/ets/entryability/EntryAbility;->onCreate"
0x44C0 : mov v0 v12
0x44D1 : mov v1 v13
0x44E2 : mov v2 v14
0x44F3 : mov v3 v15
0x450410 : mov v4 v16
0x7E01 : ldexternalmodulevar +1
0xFE09000F : throw.undefinedifholewithname "hilog"
0x6107 : sta v7
0x6007 : lda v7
0x42000010 : ldobjbyname +0 "info"
0x6106 : sta v6
0x6200000000 : ldai IMM32+0
0x6108 : sta v8
0x3E0016 : lda.str "testTag"
0x6109 : sta v9
0x3E0001 : lda.str "%{public}s"
0x610A : sta v10
0x3E0003 : lda.str "Ability onCreate"
0x610B : sta v11
0x6006 : lda v6
0x310407 : callthisrange +4 v7
0x00 : ldundefined
0x65 : returnundefined
```
