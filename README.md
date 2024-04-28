# rtlilicious

[Nom](https://crates.io/crates/nom)-based parser for [Yosys](https://yosyshq.readthedocs.io/projects/yosys/en/manual-rewrite/index.html) RTLIL [text representation](https://yosyshq.readthedocs.io/projects/yosys/en/manual-rewrite/yosys_internals/formats/rtlil_text.html).

## Usage:

```rust
    use rtlilicious;
    let src =
    r#"module \test
    wire $a;
    end
    "#;
    let design = rtlilicious::parse(src).unwrap();
    assert_eq!(design.modules().len(), 1);
```