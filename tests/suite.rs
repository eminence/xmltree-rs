
extern crate xmltree;
use xmltree::Element;
use xmltree::parse;


#[test]
fn test_01() {
    let e: Element = parse("tests/data/01.xml");
    println!("{:#?}", e);

}
#[test]
fn test_02() {
    let e: Element = parse("tests/data/02.xml");
    println!("{:#?}", e);

}
#[test]
fn test_03() {
    let e: Element = parse("tests/data/03.xml");
    println!("{:#?}", e);

}
#[test]
fn test_04() {
    let e: Element = parse("tests/data/04.xml");
    println!("{:#?}", e);

}
