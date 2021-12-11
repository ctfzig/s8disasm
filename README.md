Slede8 debugger, runtime and disassembler.

1. [Install Rust](https://rustup.rs/)
2. `cargo build --release`
3. Run the binary from target/release/s8disasm

The binary, _even the disassembler_, requires a second command line argument STDIN, which is i a _file_ containing the FØDE to feed into slede8 (in binary, not in ascii encoded hex).
