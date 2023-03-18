# RUSTEDCHIP-8

CHIP-8 emulator written in Rust.

Keyboard emulates a hex keyboard, using:
```
 1234          123C
 qwer   -->    456D
 asdf          789E
 zxcv          A0BF
  ```            

## Running the tests:
`cargo test`

## Building the app:
`cargo build --release`

Which creates an executable located at `target/release/chip8`.
Feel free to copy this to a bin dir

## Running the app:
`chip8 /path/to/rom`

Use the `-h` flag to access the man pages.

`chip8 -h`

You must pass a ROM to the emulator. There are some test roms provided, but feel free to find your own. 

## Configuration

### Chip Type
Support for both CHIP-8 and SUPERCHIP (SCHIP).

To specify which chip's quirks to use, use the `-c` flag.

### Target Instructions per Second
This slows the processor down to hit the given target. If 500 is passed in, the chip will only process 500 2-byte instructions per second. Defaults to 1000

This is currently an extremely ~~wrong~~ simple implementation. For whatever the target IPS is, it determines the average duration of an instruction to hit the target. It tracks the duration of each instruction. If the instruction took less time than average, it sleeps for the difference. We can only sleep in 1ms increments, so a target IPS of more than 1000 isn't supported. 

This also means that even if an instruction takes 999 ms, it still forces the next instruction to wait until at least the average duration is hit. In this case, it shouldn't stall the instruction at all.

This could use a lot of work
