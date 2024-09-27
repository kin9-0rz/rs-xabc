use rs_xabc::abc::AbcReader;

fn main() {
    println!("Hello, world!");
    let path = "/Users/lyb/DevEcoStudioProjects/Healthy_life/entry/build/default/outputs/default/ets/modules.abc";
    let mut abc = AbcReader::from_file(path).unwrap();
    let mut abc = AbcReader::from_file("fixtures/demo.abc").unwrap();
    abc.parse();

    for cls in abc.classes() {
        println!("-> {:?}", cls.0);

        for field in cls.1.fields() {
            println!("{:?}", field);
            let name_off = field.name_off();
            let x = abc.get_string_by_off(*name_off);
            println!("{:?}", x);
        }
    }
}
