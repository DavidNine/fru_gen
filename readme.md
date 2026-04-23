# fru_gen

This is a simple tool written in Rust that generates a FRU (Field Replaceable Unit) file compatible with `ipmitool`. The tool automatically builds the Common Header, Chassis Info Area, Board Info Area, and Product Info Area of the FRU file, ensuring each area’s checksum is correctly calculated.

## v1.0.0 Formal Release Update
- Version   : v1.0.0
- Author    : Guanyan Wang
- Date      : April 23, 2026

## Major Enhancements in v1.0.0

1. **Modernized TUI**: Improved visual style with rounded borders, color-coded sections, and native terminal background support.
2. **Multi-Page Editor**: Added a **Settings** page (switch with `Tab`) to enable/disable fields and customize reserved space for each column.
3. **Dynamic Hex View**: Real-time binary preview in `hexdump -C` format that updates instantly as you edit data or settings.
4. **Enhanced Board/Product Areas**: Added support for Board MFG Date/Time (including timestamp string parsing) and Product FRU ID to ensure full IPMI FRU specification compliance.
5. **Robust Config Loading**: Unified configuration loader that automatically detects and supports both TOML and YAML formats.
6. **Improved Controls**: Implemented standard `Ctrl+S` to save and `Esc` to exit without saving, plus mouse wheel support for the Hex View.
7. **Strict Spec Compliance**: Enhanced checksum calculations and 8-byte alignment padding across all areas.

## Use fru_gen utility

1. Install `Rust` before installing this utility. (Use Linux for example)

```Bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. After installation of `Rust`, clone repository.

```Bash
git clone git@github.com:DavidNine/fru_gen.git
```
3. Build fru_gen utility.

```Bash
cargo build --release
```

## Running example

Type the data want to be written and press `ESC` to leave.

![fru_gen utility](/images/FRU_Gen_editor.png)

A .Yaml file will be created.

![Yaml file](/images/output_yaml_file.png)

Program will load this file and create corresponding binary FRU file.

## License

This project is licensed under the MIT License. You are free to use, modify, and distribute this software; however, attribution to the original author is required. See the [LICENSE](LICENSE) file for details.
