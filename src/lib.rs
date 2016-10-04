//! A simple library for parsing an XML file into an in-memory tree structure
//!
//! Not recommended for large XML files, as it will load the entire file into memory.
//!
//! # Example
//!
//! ```no_run
//! use xmltree::Element;
//! use std::fs::File;
//!
//! let data: &'static str = r##"
//! <?xml version="1.0" encoding="utf-8" standalone="yes"?>
//! <names>
//!     <name first="bob" last="jones" />
//!     <name first="elizabeth" last="smith" />
//! </names>
//! "##;
//!
//! let mut names_element = Element::parse(data.as_bytes()).unwrap();
//!
//! println!("{:#?}", names_element);
//! {
//!     // get first `name` element
//!     let name = names_element.get_mut_child("name").expect("Can't find name element");
//!     name.attributes.insert("suffix".to_owned(), "mr".to_owned());
//! }
//! names_element.write(File::create("result.xml").unwrap());
//!
//! 
//! ```
extern crate xml;

use std::collections::HashMap;
use std::io::{Read, Write};
use std::borrow::Cow;
use std::fmt;

use xml::reader::{EventReader, XmlEvent};

/// Represents an XML element.
#[derive(Debug, Clone, PartialEq, Eq)]
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

/// Errors that can occur parsing XML
#[derive(Debug)]
pub enum ParseError {
    /// The XML is invalid
    MalformedXml(xml::reader::Error),
    /// This library is unable to process this XML. This can occur if, for
    /// example, the XML contains processing instructions.
    CannotParse,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &ParseError::MalformedXml(ref e) => write!(f, "Malformed XML. {}", e),
            &ParseError::CannotParse => write!(f, "Cannot parse"),
        }
    }
}

impl std::error::Error for ParseError {
    fn description(&self) -> &str {
        match self {
            &ParseError::MalformedXml(..) => "Malformed XML",
            &ParseError::CannotParse      => "Cannot parse",
        }
    }

    fn cause(&self) -> Option<&std::error::Error> {
        match self {
            &ParseError::MalformedXml(ref e) => Some(e),
            &ParseError::CannotParse => None,
        }
    }
}

fn build<B: Read>(reader: &mut EventReader<B>, mut elem: Element) -> Result<Element, ParseError> {
    loop {
        match reader.next() {
            Ok(XmlEvent::EndElement{ref name}) => {
                if name.local_name == elem.name {
                    return Ok(elem);
                }
                else {
                    return Err(ParseError::CannotParse);
                }
            },
            Ok(XmlEvent::StartElement{name, attributes, ..}) => {
                let mut attr_map = HashMap::new();
                for attr in attributes { attr_map.insert(attr.name.local_name, attr.value); }
                let new_elem = Element{name: name.local_name, attributes: attr_map, children: Vec::new(), text: None};
                elem.children.push(try!(build(reader, new_elem)));
            }
            Ok(XmlEvent::Characters(s)) => { elem.text = Some(s); }
            Ok(XmlEvent::Whitespace(..)) => (),
            Ok(XmlEvent::CData(s)) => { elem.text = Some(s) }
            Ok(XmlEvent::Comment(..)) => (),
            Ok(XmlEvent::StartDocument { .. }) |
            Ok(XmlEvent::EndDocument) |
            Ok(XmlEvent::ProcessingInstruction { .. }) => return Err(ParseError::CannotParse),
            Err(e) => return Err(ParseError::MalformedXml(e)),
        }
    }

}

impl Element {
    /// Create a new empty element with given name
    pub fn new(name: &str) -> Element {
        Element {
            name: String::from(name),
            attributes: HashMap::new(),
            children: Vec::new(),
            text: None,
        }
    }
    
    /// Parses some data into an Element
    pub fn parse<R: Read>(r: R) -> Result<Element, ParseError> {
        let mut reader = EventReader::new(r);
        loop {
            match reader.next() {
                Ok(XmlEvent::StartElement{name, attributes, ..}) => {
                    let mut attr_map = HashMap::new();
                    for attr in attributes { attr_map.insert(attr.name.local_name, attr.value); }

                    let root = Element{name: name.local_name, attributes: attr_map, children: Vec::new(), text: None};
                    return build(&mut reader, root);
                }
                Ok(XmlEvent::Comment(..)) |
                Ok(XmlEvent::Whitespace(..)) |
                Ok(XmlEvent::StartDocument { .. }) => continue,
                Ok(XmlEvent::EndDocument) |
                Ok(XmlEvent::EndElement { .. }) |
                Ok(XmlEvent::Characters(..)) |
                Ok(XmlEvent::CData(..)) |
                Ok(XmlEvent::ProcessingInstruction { .. }) => return Err(ParseError::CannotParse),
                Err(e) => return Err(ParseError::MalformedXml(e)),
            }
        }
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


        emitter.write(XmlEvent::StartElement{name: name, attributes: Cow::Owned(attributes), namespace: Cow::Borrowed(&namespace)}).unwrap();
        if let Some(ref t) = self.text {
            emitter.write(XmlEvent::Characters(t)).unwrap();
        }
        for elem in &self.children {
            elem._write(emitter);
        }
        emitter.write(XmlEvent::EndElement{name: Some(name)}).unwrap();

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

    /// Find a child element with the given name and return a reference to it.
    pub fn get_child<K>(&self, k: K) -> Option<&Element> 
      where String: PartialEq<K> {
          self.children.iter().find(|e| e.name == k)
    }

    /// Find a child element with the given name and return a mutable reference to it.
    pub fn get_mut_child<'a, K>(&'a mut self, k: K) -> Option<&'a mut Element> 
      where String: PartialEq<K> {
          self.children.iter_mut().find(|e| e.name == k)
    }

    /// Find a child element with the given name, remove and return it.
    pub fn take_child<'a, K>(&'a mut self, k: K) -> Option<Element>
      where String: PartialEq<K> {
          self.children.iter().position(|e| e.name == k).map(|i| self.children.remove(i))
    }
}

