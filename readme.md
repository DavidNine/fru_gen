# fru_gen

This is a simple tool written in Rust that generates a FRU (Field Replaceable Unit) file compatible with `ipmitool`. The tool automatically builds the Common Header, Chassis Info Area, Board Info Area, and Product Info Area of the FRU file, ensuring each area’s checksum is correctly calculated.

## v0.12 Update Information
- Version   : v0.12 beta
- Author    : Guanyan Wang
- Date      : November 14, 2024

1. Make this utility into CUI, Write data in the interface and build binary file after leave.

# Installation

Install `Rust` before installing this utility. (Here use WSL2 environment)

```Bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

clone from repository and start to generate !

```Bash
git clone git@github.com:DavidNine/fru_gen.git
cargo run
```

# How to use this tool

Run below command while in same directory with `Cargo.toml` file.

```Bash
cargo run
```
# Running example

Type the data want to be written and press `ESC` to leave.

![fru_gen utility](/images/FRU_Gen_editor.png)

A .Yaml file will be created.

![Yaml file](/images/output_yaml_file.png)

Program will load this file and create corresponding binary FRU file.

# Installation

Please install Rust before installing this utility. (Here use WSL2 environment)
```Bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

clone this repository and run !
```Bash
cargo build
```

## License

This project is licensed under the MIT License. You are free to use, modify, and distribute this software; however, attribution to the original author is required. See the [LICENSE](LICENSE) file for details.
