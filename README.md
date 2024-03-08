# 🎮 Gameboy Emulator 🎮

![demo](demo.gif)

My goal is to introduce myself to low level programming and emulators by writing a gameboy emulator in Rust 🦀.<br>

## 🧰 Structure

- `emulator/` - This contains the main Gameboy emulator. You'll find all of the necessary files and the core logic inside this folder.
- `asm/` - This is an assembler for the Gameboy, written in Python 🐍, made before starting the emulator. It's very basic and I made it only to list and understand each opcodes.

## 🚀 Getting started

If you want to try this out or play around with the code, you can do the following:

### 🎮 Emulator
**Clone the repository:**
```sh
git clone https://github.com/lowczarc/gameboy-emulator.git
```

**Build and run the emulator (from within the `emulator/` directory):**
```sh
cargo run --release <gameboy_rom>
```

***NOTE:** You need to have a gamepad to play, I didn't implement keyboard inputs yet*

By default the emulator will spin lock instead of using thread::sleep.<br>
If you're on battery or the 100% CPU usage bothers you, you can use the `--thread-sleep` option, though it might cause some lags and inaccurate timing.
```sh
cargo run --release <gameboy_rom> --thread-sleep
```

### 🧑‍💻 Assembler

There are some (not very interesting) examples of roms in the `asm/` directory and a ton of them you can find using the power of the *information superhighway*.

**Run the assembler (from within the `asm/` directory):**
```sh
python main.py <gbasm_input> <rom_output>
```

## 📝 License

This project is licensed under the [NPDSML - Nintendo Please Don't Sue Me License](LICENSE)

## 📚 References

- [GBDev.io Technical Reference](https://gbdev.io/pandocs/About.html)
- [pastraiser.com Opcode list](https://www.pastraiser.com/cpu/gameboy/gameboy_opcodes.html)
- [gekki.io Technical Reference](https://gekkio.fi/files/gb-docs/gbctr.pdf)
