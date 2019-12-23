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

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::io::{Read, Write};

pub use xml::namespace::Namespace;
use xml::reader::{EventReader, XmlEvent};
pub use xml::writer::{EmitterConfig, Error};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XMLNode {
    Element(Element),
    Comment(String),
    CData(String),
    Text(String),
    ProcessingInstruction(String, Option<String>),
}

/// Represents an XML element.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Element {
    /// This elements prefix, if any
    pub prefix: Option<String>,

    /// This elements namespace, if any
    pub namespace: Option<String>,

    /// The full list of namespaces, if any
    ///
    /// The `Namespace` type is exported from the `xml-rs` crate.
    pub namespaces: Option<Namespace>,

    /// The name of the Element.  Does not include any namespace info
    pub name: String,

    /// The Element attributes
    pub attributes: HashMap<String, String>,

    /// Children
    pub children: Vec<XMLNode>,
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
        match *self {
            ParseError::MalformedXml(ref e) => write!(f, "Malformed XML. {}", e),
            ParseError::CannotParse => write!(f, "Cannot parse"),
        }
    }
}

impl std::error::Error for ParseError {
    fn description(&self) -> &str {
        match *self {
            ParseError::MalformedXml(..) => "Malformed XML",
            ParseError::CannotParse => "Cannot parse",
        }
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        match *self {
            ParseError::MalformedXml(ref e) => Some(e),
            ParseError::CannotParse => None,
        }
    }
}

fn build<B: Read>(reader: &mut EventReader<B>, mut elem: Element) -> Result<Element, ParseError> {
    loop {
        match reader.next() {
            Ok(XmlEvent::EndElement { ref name }) => {
                if name.local_name == elem.name {
                    return Ok(elem);
                } else {
                    return Err(ParseError::CannotParse);
                }
            }
            Ok(XmlEvent::StartElement {
                name,
                attributes,
                namespace,
            }) => {
                let mut attr_map = HashMap::new();
                for attr in attributes {
                    attr_map.insert(attr.name.local_name, attr.value);
                }

                let new_elem = Element {
                    prefix: name.prefix,
                    namespace: name.namespace,
                    namespaces: if namespace.is_essentially_empty() {
                        None
                    } else {
                        Some(namespace)
                    },
                    name: name.local_name,
                    attributes: attr_map,
                    children: Vec::new(),
                };
                elem.children
                    .push(XMLNode::Element(build(reader, new_elem)?));
            }
            Ok(XmlEvent::Characters(s)) => elem.children.push(XMLNode::Text(s)),
            Ok(XmlEvent::Whitespace(..)) => (),
            Ok(XmlEvent::Comment(s)) => elem.children.push(XMLNode::Comment(s)),
            Ok(XmlEvent::CData(s)) => elem.children.push(XMLNode::Text(s)),
            Ok(XmlEvent::ProcessingInstruction { name, data }) => elem
                .children
                .push(XMLNode::ProcessingInstruction(name, data)),
            Ok(XmlEvent::StartDocument { .. }) | Ok(XmlEvent::EndDocument) => {
                return Err(ParseError::CannotParse)
            }
            Err(e) => return Err(ParseError::MalformedXml(e)),
        }
    }
}

impl Element {
    /// Create a new empty element with given name
    ///
    /// All other fields are empty
    pub fn new(name: &str) -> Element {
        Element {
            name: String::from(name),
            prefix: None,
            namespace: None,
            namespaces: None,
            attributes: HashMap::new(),
            children: Vec::new(),
        }
    }

    /// Parses some data into an Element
    pub fn parse<R: Read>(r: R) -> Result<Vec<XMLNode>, ParseError> {
        let mut reader = EventReader::new(r);
        let mut root_nodes = Vec::new();
        loop {
            match reader.next() {
                Ok(XmlEvent::StartElement {
                    name,
                    attributes,
                    namespace,
                }) => {
                    let mut attr_map = HashMap::with_capacity(attributes.len());
                    for attr in attributes {
                        attr_map.insert(attr.name.local_name, attr.value);
                    }

                    let root = Element {
                        prefix: name.prefix,
                        namespace: name.namespace,
                        namespaces: if namespace.is_essentially_empty() {
                            None
                        } else {
                            Some(namespace)
                        },
                        name: name.local_name,
                        attributes: attr_map,
                        children: Vec::new(),
                    };
                    root_nodes.push(XMLNode::Element(build(&mut reader, root)?));
                }
                Ok(XmlEvent::Comment(comment_string)) => {
                    root_nodes.push(XMLNode::Comment(comment_string))
                }
                Ok(XmlEvent::Characters(text_string)) => {
                    root_nodes.push(XMLNode::Text(text_string))
                }
                Ok(XmlEvent::CData(cdata_string)) => root_nodes.push(XMLNode::CData(cdata_string)),
                Ok(XmlEvent::Whitespace(..)) | Ok(XmlEvent::StartDocument { .. }) => continue,
                Ok(XmlEvent::ProcessingInstruction { name, data }) => {
                    root_nodes.push(XMLNode::ProcessingInstruction(name, data))
                }
                Ok(XmlEvent::EndDocument) | Ok(XmlEvent::EndElement { .. }) => {
                    return Err(ParseError::CannotParse)
                }
                Err(e) => return Err(ParseError::MalformedXml(e)),
            }
        }
    }

    fn _write<B: Write>(&self, emitter: &mut xml::writer::EventWriter<B>) -> Result<(), Error> {
        use xml::attribute::Attribute;
        use xml::name::Name;
        use xml::writer::events::XmlEvent;

        let mut name = Name::local(&self.name);
        if let Some(ref ns) = self.namespace {
            name.namespace = Some(ns);
        }
        if let Some(ref p) = self.prefix {
            name.prefix = Some(p);
        }

        let mut attributes = Vec::with_capacity(self.attributes.len());
        for (k, v) in &self.attributes {
            attributes.push(Attribute {
                name: Name::local(k),
                value: v,
            });
        }

        let empty_ns = Namespace::empty();
        let namespace = if let Some(ref ns) = self.namespaces {
            Cow::Borrowed(ns)
        } else {
            Cow::Borrowed(&empty_ns)
        };

        emitter.write(XmlEvent::StartElement {
            name,
            attributes: Cow::Owned(attributes),
            namespace,
        })?;
        for node in &self.children {
            match node {
                XMLNode::Element(elem) => elem._write(emitter)?,
                XMLNode::Text(text) => emitter.write(XmlEvent::Characters(text))?,
                XMLNode::Comment(comment) => emitter.write(XmlEvent::Comment(comment))?,
                XMLNode::CData(comment) => emitter.write(XmlEvent::CData(comment))?,
                XMLNode::ProcessingInstruction(name, data) => match data.to_owned() {
                    Some(string) => emitter.write(XmlEvent::ProcessingInstruction {
                        name,
                        data: Some(&string),
                    })?,
                    None => emitter.write(XmlEvent::ProcessingInstruction { name, data: None })?,
                },
            }
            // elem._write(emitter)?;
        }
        emitter.write(XmlEvent::EndElement { name: Some(name) })?;

        Ok(())
    }

    /// Writes out this element as the root element in an new XML document
    pub fn write<W: Write>(&self, w: W) -> Result<(), Error> {
        self.write_with_config(w, EmitterConfig::new())
    }

    /// Writes out this element as the root element in a new XML document using the provided configuration
    pub fn write_with_config<W: Write>(&self, w: W, config: EmitterConfig) -> Result<(), Error> {
        use xml::common::XmlVersion;
        use xml::writer::events::XmlEvent;
        use xml::writer::EventWriter;

        let mut emitter = EventWriter::new_with_config(w, config);
        emitter.write(XmlEvent::StartDocument {
            version: XmlVersion::Version10,
            encoding: None,
            standalone: None,
        })?;
        self._write(&mut emitter)
    }

    /// Find a child element with the given name and return a reference to it.
    pub fn get_child<P: ElementPredicate>(&self, k: P) -> Option<&Element> {
        self.children
            .iter()
            .filter_map(|e| match e {
                XMLNode::Element(elem) => Some(elem),
                _ => None,
            })
            .find(|e| k.match_element(e))
    }

    /// Find a child element with the given name and return a mutable reference to it.
    pub fn get_mut_child<P: ElementPredicate>(&mut self, k: P) -> Option<&mut Element> {
        self.children
            .iter_mut()
            .filter_map(|e| match e {
                XMLNode::Element(elem) => Some(elem),
                _ => None,
            })
            .find(|e| k.match_element(e))
    }

    /// Find a child element with the given name, remove and return it.
    pub fn take_child<P: ElementPredicate>(&mut self, k: P) -> Option<Element> {
        let index = self.children.iter().position(|e| match e {
            XMLNode::Element(elem) => k.match_element(elem),
            _ => false,
        });
        match index {
            Some(index) => match self.children.remove(index) {
                XMLNode::Element(elem) => Some(elem),
                _ => None,
            },
            None => None,
        }
    }
}

/// A predicate for matching elements.
///
/// The default implementations allow you to match by tag name or a tuple of
/// tag name and namespace.
pub trait ElementPredicate {
    fn match_element(&self, e: &Element) -> bool;
}

// Unfortunately,
// `impl<TN> ElementPredicate for TN where String: PartialEq<TN>` and
// `impl<TN, NS> ElementPredicate for (TN, NS) where String: PartialEq<TN>, String: PartialEq<NS>`
// are conflicting implementations, even though we know that there is no
// implementation for tuples. We just manually implement `ElementPredicate` for
// all `PartialEq` impls of `String` and forward them to the 1-tuple version.
//
// This can probably be fixed once specialization is stable.
impl<TN> ElementPredicate for (TN,)
where
    String: PartialEq<TN>,
{
    fn match_element(&self, e: &Element) -> bool {
        e.name == self.0
    }
}

impl<'a> ElementPredicate for &'a str {
    /// Search by tag name
    fn match_element(&self, e: &Element) -> bool {
        (*self,).match_element(e)
    }
}

impl<'a> ElementPredicate for Cow<'a, str> {
    /// Search by tag name
    fn match_element(&self, e: &Element) -> bool {
        (&**self,).match_element(e)
    }
}

impl ElementPredicate for String {
    /// Search by tag name
    fn match_element(&self, e: &Element) -> bool {
        (&**self,).match_element(e)
    }
}

impl<TN, NS> ElementPredicate for (TN, NS)
where
    String: PartialEq<TN>,
    String: PartialEq<NS>,
{
    /// Search by a tuple of (tagname, namespace)
    fn match_element(&self, e: &Element) -> bool {
        e.name == self.0
            && e.namespace
                .as_ref()
                .map(|ns| ns == &self.1)
                .unwrap_or(false)
    }
}
