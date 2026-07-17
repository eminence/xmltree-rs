//! Smoke tests for deeply nested documents.
//!
//! Regression coverage for <https://github.com/eminence/xmltree-rs/issues/58>:
//! parsing or writing a deeply nested tree must not overflow the thread
//! stack. These tests are expected to abort the test harness when run
//! against a recursive implementation.
//!
//! Note: dropping a deeply nested tree still recurses through
//! `Vec<XMLNode>` drop glue. That is out of scope for the parse/write fix,
//! so the trees below are wrapped in `ManuallyDrop` to keep both the
//! success and the panic paths off the recursive drop glue.

use std::io::Cursor;
use std::mem::ManuallyDrop;

use xmltree::{Element, ParseError, XMLNode};

/// Matches the nesting depth from the issue #58 reproducer.
const DEPTH: usize = 30_000;

fn deeply_nested_document(depth: usize) -> String {
    let mut xml = String::with_capacity(depth * 7 + 14);
    xml.push_str("<root>");
    for _ in 0..depth {
        xml.push_str("<x>");
    }
    for _ in 0..depth {
        xml.push_str("</x>");
    }
    xml.push_str("</root>");
    xml
}

fn deeply_nested_tree(depth: usize) -> Element {
    let mut root = Element::new("root");
    {
        let mut current = &mut root;
        for _ in 0..depth {
            current.children.push(XMLNode::Element(Element::new("x")));
            current = current
                .children
                .last_mut()
                .and_then(XMLNode::as_mut_element)
                .unwrap();
        }
    }
    root
}

fn assert_nesting_depth(elem: &Element, expected: usize) {
    let mut current = elem;
    let mut depth = 0;
    while let Some(child) = current.get_child("x") {
        depth += 1;
        current = child;
    }
    assert_eq!(depth, expected);
    assert!(current.children.is_empty());
}

#[test]
fn parses_deeply_nested_document() {
    let xml = deeply_nested_document(DEPTH);

    let root = ManuallyDrop::new(Element::parse(Cursor::new(xml)).unwrap());

    assert_eq!(root.name, "root");
    assert_nesting_depth(&root, DEPTH);
}

#[test]
fn writes_deeply_nested_document() {
    // Build the tree directly so this test exercises `_write` even if
    // parsing were to gain an independent depth limit in the future.
    let root = ManuallyDrop::new(deeply_nested_tree(DEPTH));

    let mut buf = Vec::new();
    root.write(&mut buf).unwrap();

    // Re-parse the output and check it structurally with an iterative
    // walk. Comparing the trees with the derived `PartialEq` (or `Debug`,
    // `Clone`, drop) would recurse over the nesting depth; those derived
    // impls are out of scope for the parse/write fix.
    let reparsed = ManuallyDrop::new(Element::parse(Cursor::new(buf)).unwrap());
    assert_eq!(reparsed.name, "root");
    assert_nesting_depth(&reparsed, DEPTH);
}

#[test]
fn error_path_inside_build_does_not_overflow_drop() {
    // The deep chain is fully closed inside `<a>`, so at the moment the
    // malformed input arrives, `parents`/`elem` in `build()` hold the
    // completely assembled 30k-deep tree. Returning the error must not
    // drop that tree recursively (drop glue would overflow the stack).
    let mut xml = String::from("<root><a>");
    for _ in 0..DEPTH {
        xml.push_str("<x>");
    }
    for _ in 0..DEPTH {
        xml.push_str("</x>");
    }
    xml.push('<'); // truncated tag: xml-rs errors inside build()

    let result = Element::parse(Cursor::new(xml));
    assert!(matches!(result, Err(ParseError::MalformedXml(_))));
}

#[test]
fn error_path_after_completed_root_does_not_overflow_drop() {
    // A complete deep document followed by trailing junk: `parse_all`
    // errors while `root_nodes` holds the finished deep tree. Same
    // drop-glue hazard on the error path.
    let mut xml = deeply_nested_document(DEPTH);
    xml.push('<');

    let result = Element::parse(Cursor::new(xml));
    assert!(matches!(result, Err(ParseError::MalformedXml(_))));
}
