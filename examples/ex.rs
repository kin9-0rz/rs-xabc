use rs_xabc::abc::AbcReader;

fn main() {
    println!("Hello, world!");
    //let path = "/Users/lyb/DevEcoStudioProjects/Healthy_life/entry/build/default/outputs/default/ets/modules.abc";
    //let mut abc = AbcReader::from_file(path).unwrap();
    //
    let mut abc = AbcReader::from_file("fixtures/demo.abc").unwrap();
    abc.parse();

    let header = abc.header();
    println!("{}", header);

    for cls in abc.classes() {
        println!("{:?}", cls);
    }

    for r in abc.regions() {
        println!("{:?}", r.header());

        let class_region_idx = r.class_region_idx();
        for item in class_region_idx.offsets() {
            println!("{}", item);
        }

        println!("{:?}", r.method_string_literal_region_idx().offsets());
    }
}
