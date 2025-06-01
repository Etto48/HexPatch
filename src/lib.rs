#![doc(
    html_favicon_url = "https://raw.githubusercontent.com/Etto48/HexPatch/master/docs/assets/logo.ico"
)]
/*!
# HexPatch

HexPatch is a binary patcher and editor with terminal user interface (TUI),
it's capable of disassembling instructions and assembling patches.
It supports a variety of architectures and file formats.
Also, it can edit remote files via SSH.

## Documentation

The up-to-date documentation on the frontend application can be found
[here](https://etto48.github.io/HexPatch/).

## Disclaimer

The library is not meant to be used as a dependency,
it's a part of the frontend application, for this reason,
the library API is not stable, subject to change and
its documentation is not complete.

*/

pub mod app;
pub mod args;
pub mod asm;
pub mod fuzzer;
pub mod headers;

#[macro_use]
extern crate macro_rules_attribute;
#[macro_use]
extern crate rust_i18n;

i18n!();
