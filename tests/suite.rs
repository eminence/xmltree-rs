extern crate xmltree;

use std::borrow::Cow;
use std::fs::File;
use std::io::Cursor;
use xmltree::*;

#[test]
fn test_01() {
    let e: Element = Element::parse(File::open("tests/data/01.xml").unwrap()).unwrap();
    println!("E:=======\n{:#?}", e);
    assert_eq!(e.name, "project");
    let e2: &Element = e
        .get_child("libraries")
        .expect("Missing libraries child element");
    assert_eq!(e2.name, "libraries");

    assert!(e.get_child("doesnotexist").is_none());

    let mut buf = Vec::new();
    e.write(&mut buf).unwrap();

    let e2 = Element::parse(Cursor::new(buf)).unwrap();
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
    let pi = e.children[0].as_processing_instruction();
    assert!(pi.is_some());
    let pi = pi.unwrap();
    assert_eq!(pi.0, "pi");
    assert_eq!(pi.1.unwrap(), "foo=\"blah\"");
}

#[test]
fn test_parse_all() {
    let nodes = Element::parse_all(File::open("tests/data/04.xml").unwrap()).unwrap();
    println!("{:#?}", nodes);
    assert_eq!(nodes.len(), 4);
    assert!(nodes[0].as_comment().is_some());
}

#[test]
fn test_no_root_node() {
    let result = Element::parse_all(File::open("tests/data/05.xml").unwrap());
    assert!(result.is_err())
}

#[test]
fn test_rw() {
    let e: Element = Element::parse(File::open("tests/data/rw.xml").unwrap()).unwrap();

    let mut buf = Vec::new();
    e.write(&mut buf).unwrap();

    let e2 = Element::parse(Cursor::new(buf)).unwrap();

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
fn test_new() {
    let e = Element::new("foo");
    assert_eq!(e.name.as_str(), "foo");
    assert_eq!(e.attributes.len(), 0);
    assert_eq!(e.children.len(), 0);
    // assert_eq!(e.text, None);
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

#[test]
fn test_ns_rw() {
    {
        let e: Element = Element::parse(File::open("tests/data/ns1.xml").unwrap()).unwrap();

        let mut buf = Vec::new();
        e.write(&mut buf).unwrap();

        let e2 = Element::parse(Cursor::new(buf)).unwrap();

        assert_eq!(e, e2);
    }
    {
        let e: Element = Element::parse(File::open("tests/data/ns2.xml").unwrap()).unwrap();

        let mut buf = Vec::new();
        e.write(&mut buf).unwrap();

        let e2 = Element::parse(Cursor::new(buf)).unwrap();

        assert_eq!(e, e2);
    }
}

#[test]
fn test_write_with_config() {
    let e: Element = Element::parse(File::open("tests/data/01.xml").unwrap()).unwrap();

    let cfg = EmitterConfig {
        perform_indent: true,
        ..EmitterConfig::default()
    };

    let mut buf = Vec::new();
    e.write_with_config(&mut buf, cfg).unwrap();

    let s = String::from_utf8(buf).unwrap();
    println!("{}", s);
}

#[test]
fn test_ns() {
    let e: Element = Element::parse(File::open("tests/data/ns1.xml").unwrap()).unwrap();

    let htbl = e
        .get_child(("table", "http://www.w3.org/TR/html4/"))
        .unwrap();
    let ftbl = e
        .get_child(("table", "https://www.w3schools.com/furniture"))
        .unwrap();

    assert_ne!(htbl, ftbl);
}

#[test]
fn test_text() {
    let data = r##"
        <?xml version="1.0" encoding="utf-8" standalone="yes"?>
        <elem><inner/></elem>
    "##;

    let elem = Element::parse(data.as_bytes()).unwrap();
    assert!(elem.get_text().is_none());

    let data = r##"
        <?xml version="1.0" encoding="utf-8" standalone="yes"?>
        <elem>hello world<inner/></elem>
    "##;

    let elem = Element::parse(data.as_bytes()).unwrap();
    assert_eq!(elem.get_text().unwrap(), Cow::Borrowed("hello world"));

    let data = r##"
        <?xml version="1.0" encoding="utf-8" standalone="yes"?>
        <elem>hello <inner/>world</elem>
    "##;

    let elem = Element::parse(data.as_bytes()).unwrap();
    assert_eq!(
        elem.get_text().unwrap(),
        Cow::from("hello world".to_owned())
    );

    let data = r##"
        <?xml version="1.0" encoding="utf-8" standalone="yes"?>
        <elem>hello <inner/><![CDATA[<world>]]></elem>
    "##;

    let elem = Element::parse(data.as_bytes()).unwrap();
    assert_eq!(
        elem.get_text().unwrap(),
        Cow::from("hello <world>".to_owned())
    );
}

#[test]
fn test_pre_order_dft_xmlnode() {
    let nodes: Vec<XMLNode> = parse_all(File::open("tests/data/dft.xml").unwrap()).unwrap();
    let root_node = nodes.get(0).expect("no root node");

    let pre_order_nodes = root_node.dft_pre_order(None);
    let collected_nodes = pre_order_nodes.collect::<Vec<&XMLNode>>();
    
    assert_eq!(collected_nodes.len(), 5);
    assert_eq!(collected_nodes.get(0).unwrap().as_element().unwrap().name, "a");
    assert_eq!(collected_nodes.get(1).unwrap().as_element().unwrap().name, "b");
    assert_eq!(collected_nodes.get(2).unwrap().as_element().unwrap().name, "c");
    assert_eq!(collected_nodes.get(3).unwrap().as_element().unwrap().name, "b");
    assert_eq!(collected_nodes.get(4).unwrap().as_element().unwrap().name, "c");
}

#[test]
fn test_pre_order_dft_early_stop_xmlnode() {
    let nodes: Vec<XMLNode> = parse_all(File::open("tests/data/dft.xml").unwrap()).unwrap();
    let root_node = nodes.get(0).expect("no root node");

    let pre_order_nodes = root_node.dft_pre_order(Some(|node| node.as_element().unwrap().name == "b"));
    let collected_nodes = pre_order_nodes.collect::<Vec<&XMLNode>>();
    
    assert_eq!(collected_nodes.len(), 4);
    assert_eq!(collected_nodes.get(0).unwrap().as_element().unwrap().name, "a");
    assert_eq!(collected_nodes.get(1).unwrap().as_element().unwrap().name, "b");
    assert_eq!(collected_nodes.get(2).unwrap().as_element().unwrap().name, "b");
    assert_eq!(collected_nodes.get(3).unwrap().as_element().unwrap().name, "c");
}

#[test]
fn test_post_order_dft_xmlnode() {
    let nodes: Vec<XMLNode> = parse_all(File::open("tests/data/dft.xml").unwrap()).unwrap();
    let root_node = nodes.get(0).expect("no root node");

    let post_order_nodes = root_node.dft_post_order(None);
    let collected_nodes = post_order_nodes.collect::<Vec<&XMLNode>>();
    
    assert_eq!(collected_nodes.len(), 5);
    assert_eq!(collected_nodes.get(0).unwrap().as_element().unwrap().name, "c");
    assert_eq!(collected_nodes.get(1).unwrap().as_element().unwrap().name, "b");
    assert_eq!(collected_nodes.get(2).unwrap().as_element().unwrap().name, "b");
    assert_eq!(collected_nodes.get(3).unwrap().as_element().unwrap().name, "c");
    assert_eq!(collected_nodes.get(4).unwrap().as_element().unwrap().name, "a");
}

#[test]
fn test_post_order_dft_early_stop_xmlnode() {
    let nodes: Vec<XMLNode> = parse_all(File::open("tests/data/dft.xml").unwrap()).unwrap();
    let root_node = nodes.get(0).expect("no root node");

    fn predicate(node: &XMLNode) -> bool {
        node.as_element().unwrap().name == "b"
    }

    let post_order_nodes = root_node.dft_post_order(Some(predicate));
    let collected_nodes = post_order_nodes.collect::<Vec<&XMLNode>>();
    
    assert_eq!(collected_nodes.len(), 4);
    assert_eq!(collected_nodes.get(0).unwrap().as_element().unwrap().name, "b");
    assert_eq!(collected_nodes.get(1).unwrap().as_element().unwrap().name, "b");
    assert_eq!(collected_nodes.get(2).unwrap().as_element().unwrap().name, "c");
    assert_eq!(collected_nodes.get(3).unwrap().as_element().unwrap().name, "a");
}

#[test]
fn test_pre_order_dft_early_stop_element() {
    let root_node: Element = parse(File::open("tests/data/dft.xml").unwrap()).unwrap();

    let pre_order_nodes = root_node.dft_pre_order(Some(|node| node.name == "b"));
    let collected_nodes = pre_order_nodes.collect::<Vec<&Element>>();
    
    assert_eq!(collected_nodes.len(), 4);
    assert_eq!(collected_nodes.get(0).unwrap().name, "a");
    assert_eq!(collected_nodes.get(1).unwrap().name, "b");
    assert_eq!(collected_nodes.get(2).unwrap().name, "b");
    assert_eq!(collected_nodes.get(3).unwrap().name, "c");
}

#[test]
fn test_post_order_dft_element() {
    let root_node: Element = parse(File::open("tests/data/dft.xml").unwrap()).unwrap();

    let post_order_nodes = root_node.dft_post_order(None);
    let collected_nodes = post_order_nodes.collect::<Vec<&Element>>();
    
    assert_eq!(collected_nodes.len(), 5);
    assert_eq!(collected_nodes.get(0).unwrap().name, "c");
    assert_eq!(collected_nodes.get(1).unwrap().name, "b");
    assert_eq!(collected_nodes.get(2).unwrap().name, "b");
    assert_eq!(collected_nodes.get(3).unwrap().name, "c");
    assert_eq!(collected_nodes.get(4).unwrap().name, "a");
}

#[test]
fn test_post_order_dft_early_stop_element() {
    let root_node: Element = parse(File::open("tests/data/dft.xml").unwrap()).unwrap();

    fn predicate(node: &Element) -> bool {
        node.name == "b"
    }

    let post_order_nodes = root_node.dft_post_order(Some(predicate));
    let collected_nodes = post_order_nodes.collect::<Vec<&Element>>();
    
    assert_eq!(collected_nodes.len(), 4);
    assert_eq!(collected_nodes.get(0).unwrap().name, "b");
    assert_eq!(collected_nodes.get(1).unwrap().name, "b");
    assert_eq!(collected_nodes.get(2).unwrap().name, "c");
    assert_eq!(collected_nodes.get(3).unwrap().name, "a");
}