# fru_gen

This is a simple tool written in Rust that generates a FRU (Field Replaceable Unit) file compatible with `ipmitool`. The tool automatically builds the Common Header, Chassis Info Area, Board Info Area, and Product Info Area of the FRU file, ensuring each area’s checksum is correctly calculated.

## v0.14 Update Information
- Version   : v0.14-alpha
- Author    : Guanyan Wang
- Date      : November 23, 2024

## Update Detail 

1. Update UI style and provide a more comprehensive interface.
2. Change chassis type access method.

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
