use xabc_parser::abc::AbcReader;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// 目标文件
    #[arg(short, long)]
    path: String,

    /// 输出文件信息
    #[arg(short, long)]
    infos: bool,

    /// 输出类列表
    #[arg(short = 'z', long)]
    classes: bool,

    /// 输出方法列表
    #[arg(short, long)]
    methods: bool,

    /// 输出字符串列表
    #[arg(short, long)]
    strings: bool,

    /// 解析指定方法, 格式：类名->方法名，如: La/b/c;->mtd
    #[arg(short = 'c', long)]
    method: Option<String>,
}

fn main() {
    let args = Args::parse();
    let path = args.path;

    let abc = AbcReader::from_file(path).unwrap();
    if args.infos {
        println!("{}", abc.header());
    }

    if args.classes {
        let mut classes = abc.get_class_names();
        classes.sort();
        for cls in classes {
            println!("{}", cls);
        }
    }

    if args.methods {
        let mut methods = abc.get_method_names();
        methods.sort();
        for method in methods {
            println!("{}", method);
        }
    }

    if args.strings {
        let mut strings = abc.get_strings();
        strings.sort();
        for string in strings {
            println!("{}", string);
        }
    }

    if let Some(method) = args.method {
        if !method.contains("->") {
            println!("方法格式错误，正确格式：\"类名->方法名\"");
            return;
        }
        abc.parse_method(method);
    }
}
