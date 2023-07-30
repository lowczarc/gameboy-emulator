# 🎮 Gameboy Emulator 🎮

**🚧 WORK IN PROGRESS 🚧**

![demo](demo.gif)

My goal is to introduce myself to low level programming and emulators by writing a gameboy emulator in Rust 🦀.<br>
For now it only can run Tetris.

## 🧰 Structure

In exactly the same way as the [Chip8 emulator](https://github.com/lowczarc/chip-8-emulator) I wrote before

- `emulator/` - This contains the main Gameboy emulator. You'll find all of the necessary files and the core logic inside this folder.
- `asm/` - This is an assembler for the Gameboy, written in Python 🐍.

After doing it twice, I would recommend to do it this way if you plan to write an emulator: start with the assembler before writing the emulator. You will have a better understanding of the capabilities of the CPU and will already have a good idea for how the opcodes are structured.

## 🚀 Getting started

If you want to try this out or play around with the code, you can do the following:

**Clone the repository:**
```sh
git clone https://github.com/lowczarc/gameboy-emulator.git
```

**Build and run the emulator (from within the `emulator/` directory):**
```sh
cargo run <gameboy_rom>
```

***NOTE:** You need to have a gamepad to play, I didn't implement keyboard inputs yet*

There are some examples of roms in the `asm/` directory and a ton of them you can find using the power of the *information superhighway*.

**Run the assembler (from within the `asm/` directory):**
```sh
python main.py <gbasm_input> <rom_output>
```

## 📝 License

This project is licensed under the "I love you Nintendo please don't sue me" license

## 📚 References

- [GBDev.io Technical Reference](https://gbdev.io/pandocs/About.html)
- [pastraiser.com Opcode list](https://www.pastraiser.com/cpu/gameboy/gameboy_opcodes.html)
- [gekki.io Technical Reference](https://gekkio.fi/files/gb-docs/gbctr.pdf)
