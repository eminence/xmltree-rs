//! A simple library for parsing an XML file into an in-memory tree structure
//!
//! Not well tested, and not recommended for large XML files
extern crate xml;

use std::convert::AsRef;
use std::path::Path;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use xml::reader::EventReader;
use xml::reader::events::XmlEvent;

#[derive(Debug)]
pub struct Element {
    /// The name of the Element.  Does not include any namespace info
    pub name: String,

    /// The Element attributes
    pub attributes: HashMap<String, String>,

    /// Children
    pub children: Vec<Element>,

    /// The text data for this element
    pub text: Option<String>
}


fn build<B: Read>(reader: &mut EventReader<B>, mut elem: Element) -> Element {
    
    loop {
        match reader.next() {
            XmlEvent::EndElement{ref name} if name.local_name == elem.name => { return elem; }
            XmlEvent::StartElement{name, attributes, namespace} => {
                let mut attr_map = HashMap::new();
                for attr in attributes { attr_map.insert(attr.name.local_name, attr.value); }
                let new_elem = Element{name: name.local_name, attributes: attr_map, children: Vec::new(), text: None};
                elem.children.push(build(reader, new_elem));
            }
            XmlEvent::Characters(s) => { elem.text = Some(s); }
            XmlEvent::Whitespace(..) => (),
            XmlEvent::CData(s) => { elem.text = Some(s) }
            x => {panic!("Unsure what to do with {:?}", x)}
        }
    }

    elem
}

/// Parses a file into an Element 
///
/// # Panics
///
/// Panics on error or other unhandled condition
pub fn parse<P: AsRef<Path>>(p: P) -> Element {
    let f = File::open(p).unwrap();
    let mut reader = EventReader::new(f);


    loop {
        match reader.next() {
            XmlEvent::StartElement{name, attributes, namespace} => {
                let mut attr_map = HashMap::new();
                for attr in attributes { attr_map.insert(attr.name.local_name, attr.value); }

                let root = Element{name: name.local_name, attributes: attr_map, children: Vec::new(), text: None};
                return build(&mut reader, root);
            }
            XmlEvent::EndDocument => break,
            XmlEvent::Error(e) => panic!("{:?}", e),
            _ => ()
            
        }
    }
    panic!("Error")



}
