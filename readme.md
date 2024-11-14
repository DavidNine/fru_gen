# fru_gen

This is a simple tool written in Rust that generates a FRU (Field Replaceable Unit) file compatible with `ipmitool`. The tool automatically builds the Common Header, Chassis Info Area, Board Info Area, and Product Info Area of the FRU file, ensuring each area’s checksum is correctly calculated.

## v0.12 Update Information
- Version   : v0.12 beta
- Author    : Guanyan Wang
- Date      : November 11, 2024

1. Make this utility into CUI, Write data in the interface and build binary file after leave.


## License

This project is licensed under the MIT License. You are free to use, modify, and distribute this software; however, attribution to the original author is required. See the [LICENSE](LICENSE) file for details.
