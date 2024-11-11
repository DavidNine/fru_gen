# fru_gen

This is a simple tool written in Rust that generates a FRU (Field Replaceable Unit) file compatible with `ipmitool`. The tool automatically builds the Common Header, Chassis Info Area, Board Info Area, and Product Info Area of the FRU file, ensuring each area’s checksum is correctly calculated.

## v0.11 Update Information
- Version   : v0.11 alpha
- Author    : Guanyan Wang
- Date      : November 11, 2024

1. Modularize all areas to improve overall maintainability and consistency.
2. Add a new area: "Internal Use Area" and improve the judgment logic in certain areas.



## License

This project is licensed under the MIT License. You are free to use, modify, and distribute this software; however, attribution to the original author is required. See the [LICENSE](LICENSE) file for details.
