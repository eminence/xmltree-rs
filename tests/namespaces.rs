use std::fs::File;
use std::io::Cursor;
use xmltree::*;

#[test]
fn test_ns_rw() {
    {
        let e: Element = Element::parse(File::open("tests/data/ns1.xml").unwrap()).unwrap();

        let mut buf = Vec::new();
        e.write(&mut buf).unwrap();

        let e2 = Element::parse(Cursor::new(buf)).unwrap();

        assert_eq!(e, e2);
        assert_eq!(e.namespaces.as_ref().map(|ns|ns.get(&Namespace::default().to_string())).unwrap(), e2.namespaces.as_ref().map(|ns|ns.get(&Namespace::default().to_string())).unwrap());
        assert_eq!(e.namespaces.as_ref().map(|ns|ns.get(&Namespace::default().to_string())).unwrap(), Some("urn:xmltree-rs:example"));
        assert_eq!(e.namespaces.as_ref().map(|ns|ns.get("svg")).unwrap(), None);
    }
    {
        let e: Element = Element::parse(File::open("tests/data/ns2.xml").unwrap()).unwrap();

        let mut buf = Vec::new();
        e.write(&mut buf).unwrap();

        let e2 = Element::parse(Cursor::new(buf)).unwrap();

        assert_eq!(e, e2);
    }
}

#[should_panic]
#[test]
fn test_empty_ns() {
    format!("{}", Namespace::empty());
}

#[test]
fn test_default_ns() {
    assert_ne!(Namespace::default().into_iter().collect::<Vec<_>>(), xml::namespace::Namespace::empty().into_iter().collect::<Vec<_>>());
    assert_eq!(Namespace::default().into_iter().collect::<Vec<_>>(), vec![(xml::namespace::NS_NO_PREFIX, xml::namespace::NS_EMPTY_URI)]);
    assert_eq!(vec![(String::new().as_str(), String::new().as_str())], vec![(xml::namespace::NS_NO_PREFIX, xml::namespace::NS_EMPTY_URI)]);
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
fn test_issue13() {
    let e: Element = Element::parse(File::open("tests/data/issue13.xml").unwrap()).unwrap();
    let mut buf = Vec::new();
    e.write(&mut buf).unwrap();

    assert!(String::from_utf8_lossy(&buf).contains(r#"xml:lang="en""#));
}
