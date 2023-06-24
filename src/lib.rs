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
#[cfg(all(feature = "attribute-order", not(feature = "attribute-sorted")))]
/// The type used to store element attributes.
pub type AttributeMap<K, V> = indexmap::map::IndexMap<K, V>;
#[cfg(all(feature = "attribute-sorted", not(feature = "attribute-order")))]
/// The type used to store element attributes.
pub type AttributeMap<K, V> = std::collections::BTreeMap<K, V>;
// When both features disabled or both enabled, use a fallback so irrelevant compiler errors don't
// appear…
#[cfg(any(
    not(any(feature = "attribute-sorted", feature = "attribute-order")),
    all(feature = "attribute-order", feature = "attribute-sorted")
))]
/// The type used to store element attributes.
///
/// By default this is a HashMap, but this can be changed with the "attribute-sorted" or "attribute-order" features
pub type AttributeMap<K, V> = std::collections::HashMap<K, V>;
// But don't let the invalid case off easy, now that we've made sure this is the only compiler
// error they'll see.
#[cfg(all(feature = "attribute-order", feature = "attribute-sorted"))]
compile_error!("`attribute-order` and `attribute-sorted` are mutually exclusive — pick one");

use std::borrow::Cow;
use std::fmt;
use std::io::{Read, Write};
use std::iter::FromIterator as _;

mod namespace;
pub use namespace::Namespace;
use xml::namespace::Namespace as XmlNamespace;
pub use xml::reader::ParserConfig;
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

trait AttributeMapExt {
    fn allocate(capacity: usize) -> Self;
}

#[cfg(feature = "attribute-sorted")]
impl<K: Ord, V> AttributeMapExt for AttributeMap<K, V> {
    fn allocate(_capacity: usize) -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "attribute-sorted"))]
impl<K, V> AttributeMapExt for AttributeMap<K, V> {
    fn allocate(capacity: usize) -> Self {
        Self::with_capacity(capacity)
    }
}

impl XMLNode {
    pub fn as_element(&self) -> Option<&Element> {
        if let XMLNode::Element(e) = self {
            Some(e)
        } else {
            None
        }
    }
    pub fn as_mut_element(&mut self) -> Option<&mut Element> {
        if let XMLNode::Element(e) = self {
            Some(e)
        } else {
            None
        }
    }
    pub fn as_comment(&self) -> Option<&str> {
        if let XMLNode::Comment(c) = self {
            Some(c)
        } else {
            None
        }
    }
    pub fn as_cdata(&self) -> Option<&str> {
        if let XMLNode::CData(c) = self {
            Some(c)
        } else {
            None
        }
    }
    pub fn as_text(&self) -> Option<&str> {
        if let XMLNode::Text(c) = self {
            Some(c)
        } else {
            None
        }
    }
    pub fn as_processing_instruction(&self) -> Option<(&str, Option<&str>)> {
        if let XMLNode::ProcessingInstruction(s, o) = self {
            Some((s, o.as_ref().map(|s| s.as_str())))
        } else {
            None
        }
    }
}

/// Represents an XML element.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Element {
    /// This elements prefix, if any
    pub prefix: Option<String>,

    /// This elements namespace, if any
    pub namespace: Option<String>,

    /// The full list of namespaces, if any
    pub namespaces: Option<Namespace>,

    /// The name of the Element.  Does not include any namespace info
    pub name: String,

    /// The Element attributes
    ///
    /// By default, this is a `HashMap`, but there are two optional features that can change this:
    ///
    /// * If the "attribute-order" feature is enabled, then this is an [IndexMap](https://docs.rs/indexmap/1.4.0/indexmap/),
    /// which will retain item insertion order.
    /// * If the "attribute-sorted" feature is enabled, then this is a [`std::collections::BTreeMap`], which maintains keys in sorted order.
    pub attributes: AttributeMap<String, String>,

    /// Attribute namespaces, such as `xml:lang="en"` in the below snippet:
    /// ```xml
    /// <mdui:DisplayName xml:lang="en">TestShib Test IdP</mdui:DisplayName>
    /// ```
    pub attribute_namespaces: AttributeMap<String, Namespace>,

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

trait ToAttributeMaps {
    fn to_attribute_maps(self) -> (AttributeMap<String, String>, AttributeMap<String, Namespace>);
}

impl ToAttributeMaps for Vec<xml::attribute::OwnedAttribute> {
    fn to_attribute_maps(self) -> (AttributeMap<String, String>, AttributeMap<String, Namespace>) {
        let mut attr_map = AttributeMap::with_capacity(self.len());
        let mut attr_map_ns = AttributeMap::new();

        for attr in self {
            if let Some(ns) = attr.name.prefix {
                let xns = Namespace::from_iter([(ns, attr.name.namespace.unwrap_or_else(||xml::namespace::NS_EMPTY_URI.to_string()))]);
                attr_map_ns.insert(attr.name.local_name.clone(), xns);
            }
            attr_map.insert(attr.name.local_name, attr.value);
        }

        (attr_map, attr_map_ns)
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
                let (attributes, attribute_namespaces) = attributes.to_attribute_maps();

                let new_elem = Element {
                    prefix: name.prefix,
                    namespace: name.namespace,
                    namespaces: if namespace.is_essentially_empty() {
                        None
                    } else {
                        Some(namespace.into())
                    },
                    name: name.local_name,
                    attributes,
                    attribute_namespaces,
                    children: Vec::new(),
                };
                elem.children
                    .push(XMLNode::Element(build(reader, new_elem)?));
            }
            Ok(XmlEvent::Characters(s)) => elem.children.push(XMLNode::Text(s)),
            Ok(XmlEvent::Whitespace(..)) => (),
            Ok(XmlEvent::Comment(s)) => elem.children.push(XMLNode::Comment(s)),
            Ok(XmlEvent::CData(s)) => elem.children.push(XMLNode::CData(s)),
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
            attributes: AttributeMap::new(),
            attribute_namespaces: AttributeMap::new(),
            children: Vec::new(),
        }
    }

    /// Parses some data into a list of `XMLNode`s
    ///
    /// This is useful when you want to capture comments or processing instructions that appear
    /// before or after the root node
    pub fn parse_all<R: Read>(r: R) -> Result<Vec<XMLNode>, ParseError> {
        let parser_config = ParserConfig::new().ignore_comments(false);
        Element::parse_all_with_config(r, parser_config)
    }

    pub fn parse_all_with_config<R: Read>(r: R, parser_config: ParserConfig) -> Result<Vec<XMLNode>, ParseError> {
        let mut reader = EventReader::new_with_config(r, parser_config);
        let mut root_nodes = Vec::new();
        loop {
            match reader.next() {
                Ok(XmlEvent::StartElement {
                    name,
                    attributes,
                    namespace,
                }) => {
                    let (attributes, attribute_namespaces) = attributes.to_attribute_maps();
                    let root = Element {
                        prefix: name.prefix,
                        namespace: name.namespace,
                        namespaces: if namespace.is_essentially_empty() {
                            None
                        } else {
                            Some(namespace.into())
                        },
                        name: name.local_name,
                        attributes,
                        attribute_namespaces,
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
                Ok(XmlEvent::EndElement { .. }) => (),
                Ok(XmlEvent::EndDocument) => return Ok(root_nodes),
                Err(e) => return Err(ParseError::MalformedXml(e)),
            }
        }
    }

    /// Parses some data into an Element
    pub fn parse<R: Read>(r: R) -> Result<Element, ParseError> {
        let nodes = Element::parse_all(r)?;
        for node in nodes {
            if let XMLNode::Element(elem) = node { return Ok(elem) }
        }
        // This assume the underlying xml library throws an error on no root element
        unreachable!();
    }

    pub fn parse_with_config<R: Read>(r: R, config: ParserConfig) -> Result<Element, ParseError> {
        let nodes = Element::parse_all_with_config(r, config)?;
        for node in nodes {
            if let XMLNode::Element(elem) = node { return Ok(elem) }
        }
        // This assume the underlying xml library throws an error on no root element
        unreachable!();
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
                name: if let Some(ns) = self.attribute_namespaces.get(k) {
                    let nskv = ns.into_iter().next();
                    if let Some((nsk, nsv)) = nskv {
                        Name::qualified(k, nsv, Some(nsk))
                    } else {
                        Name::local(k)
                    }
                } else {
                    Name::local(k)
                },
                value: v,
            });
        }

        let empty_ns = XmlNamespace::empty();
        let namespace = if let Some(ns) = self.namespaces.as_ref().map(|ns|ns.as_ref()) {
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

        let write_document_declaration = config.write_document_declaration;
        let mut emitter = EventWriter::new_with_config(w, config);
        if write_document_declaration {
            emitter.write(XmlEvent::StartDocument {
                version: XmlVersion::Version10,
                encoding: None,
                standalone: None,
            })?;
        }
        self._write(&mut emitter)
    }

    /// Find a child element with the given name and return a reference to it.
    ///
    /// Both `&str` and `String` implement `ElementPredicate` and can be used to search for child
    /// elements that match the given element name with `.get_child("element_name")`.  You can also
    /// search by `("element_name", "tag_name")` tuple.
    ///
    ///
    /// Note: this will only return Elements.  To get other nodes (like comments), iterate through
    /// the `children` field.
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

    /// Returns the inner text/cdata of this element, if any.
    ///
    /// If there are multiple text/cdata nodes, they will be all concatenated into one string.
    pub fn get_text<'a>(&'a self) -> Option<Cow<'a, str>> {
        let text_nodes: Vec<&'a str> = self
            .children
            .iter()
            .filter_map(|node| node.as_text().or_else(|| node.as_cdata()))
            .collect();
        if text_nodes.is_empty() {
            None
        } else if text_nodes.len() == 1 {
            Some(Cow::Borrowed(text_nodes[0]))
        } else {
            let mut full_text = String::new();
            for text in text_nodes {
                full_text.push_str(text);
            }
            Some(Cow::Owned(full_text))
        }
    }

    /// Checks if this element matches the predicate.
    pub fn matches<P: ElementPredicate>(&self, k: P) -> bool {
        k.match_element(self)
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
