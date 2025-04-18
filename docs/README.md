# ![Logo](./assets/logo.svg) HexPatch

<div align=center>

[![Crate Badge]][Crate] [![CI Badge]][CI] [![Deps.rs Badge]][Deps.rs] [![License Badge]][License] [![GitHub IO Badge]][GitHub IO]

</div>

HexPatch is a binary patcher and editor with terminal user interface (TUI), it's capable of disassembling instructions and assembling patches.
It supports a variety of architectures and file formats.
Also, it can edit remote files via SSH.

Take a look at [GitHub Pages][GitHub IO] for more information and some screenshots.

## Installation

### Using cargo

If you already have the requirements installed, you only need to run the following command:

```bash
cargo install hex-patch
```

### Requirements

- [CMake](https://cmake.org/download/)
- [Python](https://www.python.org/downloads/)
- [MSVC](https://visualstudio.microsoft.com/visual-cpp-build-tools/) on Windows
- [GCC](https://gcc.gnu.org/) on Linux
- [Xcode Command Line Tools](https://developer.apple.com/) on macOS
- [Rust](https://www.rust-lang.org/tools/install)

### Using your package manager

#### NetBSD

On NetBSD, a package is available from the [official repositories](https://pkgsrc.se/devel/hexpatch/).
To install it, simply run:

```bash
pkgin install hexpatch
```

##### Build from source on NetBSD

If you prefer to build from source:

```bash
cd /usr/pkgsrc/devel/hexpatch
make install
```

#### Arch Linux

On Arch Linux, a package is available from the [official repositories](https://archlinux.org/packages/extra/x86_64/hexpatch/):

```bash
pacman -S hexpatch
```

#### X-CMD

If you are a user of [x-cmd](https://x-cmd.com), you can run:

```bash
x install hexpatch
```

### Building from source with cargo

Clone the repository

```bash
git clone https://github.com/Etto48/HexPatch.git
cd HexPatch
```

Build and install

```bash
cargo install --path .
```

## SSH connection

In order to connect via SSH, you can use the following command:

```bash
hex-patch --ssh <user>@<host>[:<port>] [--password <password>] [additional arguments]
```

If you don't specify a password, the client must be set up with keypair authentication and you must have a key in your `~/.ssh` directory.

Keys are searched in the following order:

- id_rsa
- id_ed25519
- id_ecdsa
- id_dsa

The first key found will be used.

## Supported file formats and architectures

The following file formats are supported by default:

- Coff
- CoffBig
- Elf32
- Elf64
- MachO32
- MachO64
- Pe32
- Pe64
- Xcoff32
- Xcoff64

Other file formats can be added with [plugins](#plugins).

The following architectures are supported:

- Aarch64
- Aarch64_Ilp32
- Arm
- I386
- X86_64
- X86_64_X32
- Mips
- Mips64
- PowerPc
- PowerPc64
- Riscv32
- Riscv64
- S390x
- Sparc64

## Settings

Read the [settings documentation](./SETTINGS.md) for more information.

## Plugins

HexPatch supports plugins written in Lua.
Plugins must be placed in the `plugins` directory in hex-patch's configuration directory.
A different plugins directory can be specified with the `--plugins` flag.

You can find more information about the Plugin Lua API [here](./PLUGIN_API.md).

## Known issues

- Some key combinations may not work as expected, notably `SHIFT + ENTER` on VSCode terminal. Please refer to [this issue](https://github.com/crossterm-rs/crossterm/issues/685) for more information. Unfortunately, this behavior is out of my control.
- If you try to write an invalid relative jump instruction with more than one register (e.g. `jmp [rip+rax]`) in X86 architectures the program will crash. This problem depends on the `keystone-engine` crate, and it's out of my control.

## Special thanks

Thanks to [Lorenzo Colombini](https://github.com/Lorenzinco) for the instruction highlighting.

[Crate]: https://crates.io/crates/hex-patch
[Crate Badge]: https://img.shields.io/crates/v/hex-patch?logo=rust&style=flat-square&logoColor=E05D44&color=E05D44
[CI]: https://github.com/Etto48/HexPatch/actions/workflows/ci.yml
[CI Badge]: https://img.shields.io/github/actions/workflow/status/Etto48/HexPatch/ci.yml?style=flat-square&logo=github
[Deps.rs]: https://deps.rs/repo/github/Etto48/HexPatch
[Deps.rs Badge]: https://deps.rs/repo/github/Etto48/HexPatch/status.svg?style=flat-square
[License]: https://github.com/Etto48/HexPatch/blob/master/LICENSE
[License Badge]: https://img.shields.io/github/license/Etto48/HexPatch?style=flat-square&color=blue
[GitHub IO Badge]: https://img.shields.io/badge/GitHub-IO-black?style=flat-square&logo=github
[GitHub IO]: https://etto48.github.io/HexPatch/
