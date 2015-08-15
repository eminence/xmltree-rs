//! A simple library for parsing an XML file into an in-memory tree structure
//!
//! Not well tested, and not recommended for large XML files
extern crate xml;

use std::convert::AsRef;
use std::path::Path;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};

use xml::reader::EventReader;

#[derive(Debug, PartialEq, Eq)]
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
    use xml::reader::events::XmlEvent;
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

}

impl Element {

    /// Parses a file into an Element 
    ///
    /// # Panics
    ///
    /// Panics on error or other unhandled condition
    pub fn parse<R: Read>(r: R) -> Element {
        use xml::reader::events::XmlEvent;

        let mut reader = EventReader::new(r);


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

    fn _write<B: Write>(&self, emitter: &mut xml::writer::EventWriter<B>) {
        use xml::writer::events::XmlEvent;
        use xml::name::Name;
        use xml::namespace::Namespace;
        use xml::attribute::Attribute;

        let name = Name::local(&self.name);
        let mut attributes = Vec::with_capacity(self.attributes.len());
        for (k, v) in self.attributes.iter() {
            attributes.push(Attribute{name: Name::local(k), value: v});
        }

        let namespace = Namespace::empty();


        emitter.write(XmlEvent::StartElement{name: name, attributes:attributes, namespace: &namespace}).unwrap();
        if let Some(ref t) = self.text {
            emitter.write(XmlEvent::Characters(t)).unwrap();
        }
        for elem in &self.children {
            elem._write(emitter);
        }
        emitter.write(XmlEvent::EndElement{name: name}).unwrap();

    }

    /// Writes out this element as the root element in an new XML document
    pub fn write<W: Write>(&self, w:W) {
        use xml::writer::EventWriter;
        use xml::writer::events::XmlEvent;
        use xml::common::XmlVersion;

        let mut emitter = EventWriter::new(w);
        emitter.write(XmlEvent::StartDocument{version: XmlVersion::Version10, encoding: None, standalone: None}).unwrap();
        self._write(&mut emitter);
    }

    /// Attempts to find a child element with the given name
    pub fn get_child<K>(&self, k: K) -> Option<&Element> 
      where String: PartialEq<K> {
        for elem in &self.children {
            if elem.name == k{
                return Some(&elem);
            }
        }
        None
    }

}

