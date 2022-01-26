use delegate::delegate;
use derive_more::{AsMut, AsRef, Deref, DerefMut, Display, From, Into};
pub use xml::namespace::Namespace as XmlNamespace;
use xml::namespace::{NS_NO_PREFIX, NamespaceMappings, UriMapping};

use std::iter::FromIterator;
use std::collections::BTreeMap;

/// A wrapper around [`xml::namespace::Namespace`] (＝[`XmlNamespace`]).
#[rustfmt::skip]
#[derive(AsRef, AsMut, Deref, DerefMut, Debug, Display, Clone, PartialEq, Eq, From, Into)]
#[display(fmt = "{}", "self.0.0.iter().next().map(|ns| ns.0).expect(\"Can't display empty namespace\")")]
pub struct Namespace(XmlNamespace);

impl Default for Namespace {
    fn default() -> Self {
        Self(XmlNamespace(BTreeMap::from([(
            NS_NO_PREFIX.to_string(),
            String::new(),
        )])))
    }
}

/// See § [Methods from `Deref<Target = XmlNamespace>`](struct.Namespace.html#deref-methods-XmlNamespace).
impl Namespace {
    pub fn empty() -> Namespace {
        Self(XmlNamespace::empty())
    }
    delegate! {
        to self.0 {
            pub fn is_empty(&self) -> bool;
            pub fn is_essentially_empty(&self) -> bool;
            pub fn contains<P: ?Sized + AsRef<str>>(&self, prefix: &P) -> bool;
            pub fn put<P, U>(&mut self, prefix: P, uri: U) -> bool where P: Into<String>, U: Into<String>;
            pub fn force_put<P, U>(&mut self, prefix: P, uri: U) -> Option<String> where P: Into<String>, U: Into<String>;
            pub fn get<'a, P: ?Sized>(&'a self, prefix: &P) -> Option<&'a str> where P: AsRef<str>;
        }
    }
}

impl<'a> IntoIterator for &'a Namespace {
    type Item = UriMapping<'a>;
    type IntoIter = NamespaceMappings<'a>;
    fn into_iter(self) -> NamespaceMappings<'a> {
        self.0.into_iter()
    }
}

impl FromIterator<(String, String)> for Namespace {
    fn from_iter<I: IntoIterator<Item = (String, String)>>(iter: I) -> Self {
        Self(XmlNamespace(BTreeMap::from_iter(iter)))
    }
}
