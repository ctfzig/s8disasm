Slede8 debugger, runtime and disassembler.

1. [Install Rust](https://rustup.rs/)
2. `cargo build --release`
3. Run the binary from target/release/s8disasm

The binary, _even the disassembler_, requires a second command line argument STDIN, which is i a _file_ containing the FØDE to feed into slede8 (in binary, not in ascii encoded hex).

## Command line

`s8disasm [options] program.s8 input.bin`

* `-d` starts in debugger mode
* `-D` runs disassembler
* no flags runs the program

## Debugger commands

* `c` continues until next breakpoint or end of program
* `b 50` sets a breakpoint at 0x50
* `m 10 ff` dumps 0xff bytes of memory from 0x10
* `enter` steps one instruction
