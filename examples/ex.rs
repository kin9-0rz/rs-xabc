use rs_xabc::abc::AbcReader;

fn main() {
    println!("Hello, world!");
    let abc = AbcReader::from_file("fixtures/demo.abc").unwrap();
    let header = abc.header();
    println!("{}", header);
    abc.parse_region_index();
}
