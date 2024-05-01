# rtlicious

[![CI](https://github.com/oxim-rs/rtlicious/actions/workflows/main.yml/badge.svg)](https://github.com/oxim-rs/rtlicious/actions/workflows/main.yml)
[![codecov](https://codecov.io/gh/oxim-rs/rtlicious/graph/badge.svg?token=OKJENSAI7Z)](https://codecov.io/gh/oxim-rs/rtlicious)

[Nom](https://crates.io/crates/nom)-based parser for [Yosys](https://yosyshq.readthedocs.io/projects/yosys/en/manual-rewrite/index.html) RTLIL [text representation](https://yosyshq.readthedocs.io/projects/yosys/en/manual-rewrite/yosys_internals/formats/rtlil_text.html).

## Usage:

```rust
    use rtlicious;
    let src =
    r#"module \test
    wire $a
    end
    "#;
    let design = rtlicious::parse(src).unwrap();
    assert_eq!(design.modules().len(), 1);
    dbg!({:?}, design);
```
```text
> Design {
    autoidx: None,
    modules: {
        "test": Module {
            attributes: {},
            parameters: {},
            wires: {
                "a": Wire {
                    width: 1,
                    offset: 0,
                    input: false,
                    output: false,
                    inout: false,
                    upto: false,
                    signed: false,
                    attributes: {},
                },
            },
            memories: {},
            cells: {},
            processes: {},
            connections: [],
        },
    },
}
```
