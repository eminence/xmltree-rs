extern crate xmltree;
extern crate tempfile;

use xmltree::Element;
use xmltree::ParseError;
use std::fs::File;
use tempfile::TempFile;


#[test]
fn test_01() {
    let e: Element = Element::parse(File::open("tests/data/01.xml").unwrap()).unwrap();
    println!("E:=======\n{:#?}", e);
    assert_eq!(e.name, "project");
    let e2: &Element = e.get_child("libraries").expect("Missing libraries child element");
    assert_eq!(e2.name, "libraries");

    assert!(e.get_child("doesnotexist").is_none());

    let mut tmp = TempFile::shared(2).unwrap();
    let t1 = tmp.remove(0);
    let t2 = tmp.remove(0);
    e.write(t1);

    let e2 = Element::parse(t2).unwrap();
    println!("E2:======\n{:#?}", e2);

    assert_eq!(e, e2);
}

#[test]
fn test_02() {
    let e: Element = Element::parse(File::open("tests/data/02.xml").unwrap()).unwrap();
    println!("{:#?}", e);
}

#[test]
fn test_03() {
    let e: Element = Element::parse(File::open("tests/data/03.xml").unwrap()).unwrap();
    println!("{:#?}", e);
}

#[test]
fn test_04() {
    let e: Element = Element::parse(File::open("tests/data/04.xml").unwrap()).unwrap();
    println!("{:#?}", e);
}

#[test]
fn test_rw() {

    let e: Element = Element::parse(File::open("tests/data/rw.xml").unwrap()).unwrap();

    let mut tmp = TempFile::shared(2).unwrap();
    let t1 = tmp.remove(0);
    let t2 = tmp.remove(0);
    e.write(t1);

    let e2 = Element::parse(t2).unwrap();

    assert_eq!(e, e2);
}

#[test]
fn test_mut() {

    let mut e: Element = Element::parse(File::open("tests/data/rw.xml").unwrap()).unwrap();
    {
        let name = e.get_mut_child("name").unwrap();
        name.attributes.insert("suffix".to_owned(), "mr".to_owned());
    }
}


#[test]
fn test_mal_01() {
    // some tests for error handling
   
    let data = r##"
        <?xml version="1.0" encoding="utf-8" standalone="yes"?>
        <names>
            <name first="bob" last="jones />
            <name first="elizabeth" last="smith" />
        </names>
    "##;

    let names_element = Element::parse(data.as_bytes());
    if let Err(ParseError::MalformedXml(..)) = names_element {
        // OK 
    } else {
        panic!("unexpected parse result");
    }
    println!("{:?}", names_element);

}


#[test]
fn test_mal_02() {
    // some tests for error handling
   
    let data = r##"
            this is not even close 
            to XML
    "##;

    let names_element = Element::parse(data.as_bytes());
    if let Err(ParseError::MalformedXml(..)) = names_element {
        // OK 
    } else {
        panic!("unexpected parse result");
    }
    println!("{:?}", names_element);

}


#[test]
fn test_mal_03() {
    // some tests for error handling
   
    let data = r##"
        <?xml version="1.0" encoding="utf-8" standalone="yes"?>
        <names>
            <name first="bob" last="jones"></badtag>
            <name first="elizabeth" last="smith" />
        </names>
    "##;

    let names_element = Element::parse(data.as_bytes());
    if let Err(ParseError::MalformedXml(..)) = names_element {
        // OK 
    } else {
        panic!("unexpected parse result");
    }
    println!("{:?}", names_element);

}


#[test]
fn test_take() {
    let data_xml_1 = r##"
        <?xml version="1.0" encoding="utf-8" standalone="yes"?>
        <names>
            <name first="bob" last="jones"></name>
            <name first="elizabeth" last="smith" />
            <remove_me key="value">
                <child />
            </remove_me>
        </names>
    "##;
    
    let data_xml_2 = r##"
        <?xml version="1.0" encoding="utf-8" standalone="yes"?>
        <names>
            <name first="bob" last="jones"></name>
            <name first="elizabeth" last="smith" />
        </names>
    "##;

    let mut data_1 = Element::parse(data_xml_1.as_bytes()).unwrap();
    let data_2 = Element::parse(data_xml_2.as_bytes()).unwrap();

    if let Some(removed) = data_1.take_child("remove_me") {
        assert_eq!(removed.children.len(), 1);

    } else {
        panic!("take_child failed");
    }


    assert_eq!(data_1, data_2);
}
