# Windows installer build

This repository can build the modified RICO application for Windows x64 from the Linux build environment. Required tools are stable Rust, the `x86_64-pc-windows-msvc` target, `cargo-xwin`, Clang/LLVM, and NSIS. On Debian/Ubuntu, install the system tools with `sudo apt install clang lld llvm ninja-build nsis`. Install the Rust cross-build helper with `cargo install cargo-xwin --locked`.

From the repository root, run:

```bash
packaging/windows/build-installer.sh
```

The script compiles the launcher, checks that the RICO executable icon was embedded, and packages it as `dist/windows/RICO-Setup-0.6.0.exe` (the version is read from the workspace manifest). The setup installs the executable as `RICO.exe` under Program Files and adds a Start menu shortcut. A desktop shortcut is offered as an optional component. Uninstall removes the installation folder, shortcuts, and registry entry. The uninstaller asks whether to also remove the default local data folder (`%LOCALAPPDATA%\RICO`), including profiles, accounts, settings, and Minecraft worlds. The previous `%LOCALAPPDATA%\ArcticLauncher` folder remains untouched, and data in a custom storage location is not removed automatically.

The generated installer targets Windows x64. It must still be smoke-tested on a Windows machine before public distribution.
