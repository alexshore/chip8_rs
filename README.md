# CHIP-8 Rust Emulator

Yet another CHIP-8 emulator written in Rust. I primarily wrote this to learn about SDL and basic emulator construction.

## Dependencies

- Rust 1.79+ (only tested on this version but most likely very backward compatible)
- SDL2
    - if on Linux you can install with your package manager i.e. `sudo apt install sdl2`
    - if on Mac OS you can install with `brew install sdl2`
    - if running on Windows then `SDL2.dll` needs to be in the root directory of the project

## Restrictions

- Will only run CHIP-8 type roms, SUPER-CHIP is not supported.
- CPU speed is not modifiable through configuration.

## References

- [Cowgod's Chip-8 Technical Reference](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#4xkk)
- [Guide to making a CHIP-8 emulator - Tobias V. Langhoff](https://tobiasvl.github.io/blog/write-a-chip-8-emulator/)
- [Chip-8 Emulator - loktar00](https://github.com/loktar00/chip8)
- [Chip-8 Emulator - starrhorne](https://github.com/starrhorne/chip8-rust)

## TODO

- Configurable CPU speed.
    - via argument
    - via keypress
- Reset button.
- Sound.
- Timers/Mainloop.
