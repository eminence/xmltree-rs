
extern crate xmltree;
extern crate tempfile;

use xmltree::Element;
use std::fs::File;
use tempfile::TempFile;


#[test]
fn test_01() {
    let e: Element = Element::parse(File::open("tests/data/01.xml").unwrap());
    println!("E:=======\n{:#?}", e);
    assert_eq!(e.name, "project");
    let e2: &Element = e.get_child("libraries").expect("Missing libraries child element");
    assert_eq!(e2.name, "libraries");

    assert!(e.get_child("doesnotexist").is_none());

    let mut tmp = TempFile::shared(2).unwrap();
    let t1 = tmp.remove(0);
    let t2 = tmp.remove(0);
    e.write(t1);

    let e2 = Element::parse(t2);
    println!("E2:======\n{:#?}", e2);

    assert_eq!(e, e2);

}
#[test]
fn test_02() {
    let e: Element = Element::parse(File::open("tests/data/02.xml").unwrap());
    println!("{:#?}", e);

}
#[test]
fn test_03() {
    let e: Element = Element::parse(File::open("tests/data/03.xml").unwrap());
    println!("{:#?}", e);

}
#[test]
fn test_04() {
    let e: Element = Element::parse(File::open("tests/data/04.xml").unwrap());
    println!("{:#?}", e);

}


#[test]
fn test_rw() {

    let e: Element = Element::parse(File::open("tests/data/rw.xml").unwrap());

    let mut tmp = TempFile::shared(2).unwrap();
    let t1 = tmp.remove(0);
    let t2 = tmp.remove(0);
    e.write(t1);

    let e2 = Element::parse(t2);

    assert_eq!(e, e2);

}


#[test]
fn test_mut() {

    let mut e: Element = Element::parse(File::open("tests/data/rw.xml").unwrap());
    {
        let name = e.get_mut_child("name").unwrap();
        name.attributes.insert("suffix".to_owned(), "mr".to_owned());
    }



}
