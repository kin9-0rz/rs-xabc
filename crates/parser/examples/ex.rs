use xabc_parser::abc::AbcReader;

// #[cfg(feature = "tracing")]
fn main() {
    println!("Hello, world!");
    // let path = "/Users/lyb/DevEcoStudioProjects/Healthy_life/entry/build/default/outputs/default/ets/modules.abc";
    // let mut abc = AbcReader::from_file(path).unwrap();
    let abc = AbcReader::from_file("fixtures/demo.abc").unwrap();

    println!("Header -> {}", abc.header());

    let class_names = abc.get_class_names();
    println!("{:?}", class_names);

    let method_names = abc.get_method_names();
    println!("{:?}", method_names);

    let strings = abc.get_strings();
    println!("{:?}", strings);

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
