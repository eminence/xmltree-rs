xmltree-rs
==========

[Documention](https://eminence.github.io/xmltree-rs/doc/xmltree/index.html)

A small library for parsing an XML file into an in-memory tree structure.

Not recommended for large XML files, as it will load the entire file into memory.

https://crates.io/crates/xmltree

## Usage

Add the following to your `Cargo.toml` file:

```toml
[dependencies]
xmltree = "0.6"
```

and this to yoru crate root:

```rust
extern crate xmltree;
```

## Example

See the documentation for the latest version:

https://docs.rs/xmltree/0.6.0/xmltree/
