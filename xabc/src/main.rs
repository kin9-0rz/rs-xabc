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

    /// 输出类信息
    #[arg(short, long)]
    classes: bool,

    /// 输出方法信息
    #[arg(short, long)]
    methods: bool,

    /// 输出字符串信息
    #[arg(short, long)]
    strings: bool,
}

fn main() {
    let args = Args::parse();
    let path = args.path;

    let abc = AbcReader::from_file(path).unwrap();
    if args.infos {
        println!("{:?}", abc.header());
    }

    if args.classes {
        println!("类名：");
        let mut classes = abc.get_class_names();
        classes.sort();
        for cls in classes {
            println!("{}", cls);
        }
    }

    if args.methods {
        println!("方法名：");
        let mut methods = abc.get_method_names();
        methods.sort();
        for method in methods {
            println!("{}", method);
        }
    }

    if args.strings {
        println!("字符串：");
        let mut strings = abc.get_strings();
        strings.sort();
        for string in strings {
            println!("{}", string);
        }
    }

    // TODO: 解析指定类?

    // abc.parse_code();

    // println!("Header -> {:?}", abc.header());

    // for cls in abc.classes() {
    // println!("-> {:?}", cls.0);

    // let field = &cls.1.fields()[0];
    // let name_off = field.name_off();
    // let x = abc.get_string_by_off(*name_off);
    // println!("Field -> {:?}", x);

    // for method in cls.1.method_map() {
    // println!("{:?}", method);
    // let class_idx = method.class_idx();
    // let cls = abc.get_field_type_by_class_idx(class_idx);
    // println!("{:?}", cls);
    // let name_off = method.name_off();
    // let x = abc.get_string_by_off(*name_off);
    // println!("{:?}", x);
    // }
    // }
}
