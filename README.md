# expand-mod

[![Crates.io](https://img.shields.io/crates/v/expand-mod.svg)](https://crates.io/crates/expand-mod)
[![Actions Status](https://github.com/frozenlib/expand-mod/workflows/CI/badge.svg)](https://github.com/frozenlib/expand-mod/actions)

Expand `mod module_name;` in `.rs` files and combine the module tree consisting of multiple files into a single file.

## Install

```sh
cargo install expand-mod
```

## Usage

```sh
expand-mod path_to_src/lib.rs
```

## Command line options

| option        | description                                         |
| ------------- | --------------------------------------------------- |
| `--clipboard` | Copy the result to the clipboard instead of stdout. |

## License

This project is dual licensed under Apache-2.0/MIT. See the two LICENSE-\* files for details.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
