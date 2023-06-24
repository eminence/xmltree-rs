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
    assert_eq!(nodes.len(), 3);
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
        name.attributes.insert(AttributeName::local("suffix"), "mr".to_owned());
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

    let mut data_1 = Element::parse(data_xml_1.trim().as_bytes()).unwrap();
    let data_2 = Element::parse(data_xml_2.trim().as_bytes()).unwrap();

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

    let elem = Element::parse(data.trim().as_bytes()).unwrap();
    assert!(elem.get_text().is_none());

    let data = r##"
        <?xml version="1.0" encoding="utf-8" standalone="yes"?>
        <elem>hello world<inner/></elem>
    "##;

    let elem = Element::parse(data.trim().as_bytes()).unwrap();
    assert_eq!(elem.get_text().unwrap(), Cow::Borrowed("hello world"));

    let data = r##"
        <?xml version="1.0" encoding="utf-8" standalone="yes"?>
        <elem>hello <inner/>world</elem>
    "##;

    let elem = Element::parse(data.trim().as_bytes()).unwrap();
    assert_eq!(
        elem.get_text().unwrap(),
        Cow::from("hello world".to_owned())
    );

    let data = r##"
        <?xml version="1.0" encoding="utf-8" standalone="yes"?>
        <elem>hello <inner/><![CDATA[<world>]]></elem>
    "##;

    let elem = Element::parse(data.trim().as_bytes()).unwrap();
    assert_eq!(
        elem.get_text().unwrap(),
        Cow::from("hello <world>".to_owned())
    );
}

#[test]
fn test_nodecl() {
    let mut c = EmitterConfig::new();
    c.write_document_declaration = false;
    let e = Element::new("n");
    let mut output = Vec::new();
    e.write_with_config(&mut output, c).unwrap();
    assert_eq!(String::from_utf8(output).unwrap(), "<n />");
}

#[test]
fn test_decl() {
    let mut c = EmitterConfig::new();
    c.write_document_declaration = true;
    let e = Element::new("n");
    let mut output = Vec::new();
    e.write_with_config(&mut output, c).unwrap();
    assert_eq!(String::from_utf8(output).unwrap(), "<?xml version=\"1.0\" encoding=\"UTF-8\"?><n />");
}

#[test]
fn test_attribute_ns1() {
    let e: Element = Element::parse(File::open("tests/data/issue13.xml").unwrap()).unwrap();

    let display_name_elem = e.get_child(("DisplayName", "urn:oasis:names:tc:SAML:metadata:ui")).unwrap();
    assert_eq!(display_name_elem.attributes.len(), 1);

    // Search map using AttributeName struct
    let attribute_name = AttributeName{ local_name: "lang".to_string(),
        namespace: Some("http://www.w3.org/XML/1998/namespace".to_string()),
        prefix: Some("xml".to_string())};
    assert_eq!(display_name_elem.attributes.get(&attribute_name).unwrap(), "en");

    // Search by name + namespace
    let attribute_value
        = display_name_elem.get_attribute(("lang",
            Some("http://www.w3.org/XML/1998/namespace"))).unwrap();
    assert_eq!(attribute_value, "en");

    assert_eq!(None, display_name_elem.get_attribute(("lang",
            Some("https://www.w3schools.com/furniture"))));

    // Search by name as a &str
    assert_eq!("en", display_name_elem.get_attribute("lang").unwrap());

    assert_eq!(None, display_name_elem.get_attribute("no_such_attribute"));
}

#[test]
fn test_multi_attribute_names() {
    let ext_ns = "http://dbus.extensions.com/schemas/dbus-extensions-v1.0";
    let mut e = Element::parse(File::open("tests/data/multi-attribute-ns.xml").unwrap()).unwrap();
    let mut member_elem = e.take_child(("member", ext_ns)).unwrap();

    // Get the first "type" attribute (in one namespace)
    assert_ne!(None, member_elem.take_attribute("type"));
    // Get the second "type" attribute (in the other namespace)
    assert_ne!(None, member_elem.take_attribute("type"));
    // Should be no more "type" attributes
    assert_eq!(None, member_elem.take_attribute("type"));

    e = Element::parse(File::open("tests/data/multi-attribute-ns.xml").unwrap()).unwrap();
    member_elem = e.take_child(("member", ext_ns)).unwrap();

    assert_eq!("i", member_elem.take_attribute(("type", None)).unwrap());
    assert_eq!(None, member_elem.take_attribute(("type", None)));

    assert_eq!("[ExtendedType]", member_elem.take_attribute(("type", Some(ext_ns))).unwrap());
    assert_eq!(None, member_elem.take_attribute(("type", Some(ext_ns))));
}

#[test]
fn test_mutable_attributes() {
    let ext_ns = "http://dbus.extensions.com/schemas/dbus-extensions-v1.0";
    let mut e = Element::parse(File::open("tests/data/multi-attribute-ns.xml").unwrap()).unwrap();
    let mut member_elem = e.take_child(("member", ext_ns)).unwrap();

    let new_val = "New value".to_string();
    let attr_val : &mut String = member_elem.get_mut_attribute(("type", None)).unwrap();
    *attr_val = new_val.clone();
    assert_eq!(&new_val, member_elem.get_attribute(("type", None)).unwrap());


    let new_ext_val = "New extended attr".to_string();
    let ext_attr_val : &mut String = member_elem.get_mut_attribute(("type", Some(ext_ns))).unwrap();
    *ext_attr_val = new_ext_val.clone();
    assert_eq!(&new_ext_val, member_elem.get_attribute(("type", Some(ext_ns))).unwrap());
}
