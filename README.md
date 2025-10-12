# Framework Fan RGB Utility

Small control utility for the Framework Desktop fan RGB lighting. It bundles a
command-line interface for scripting and a desktop UI for interactive control,
both layered on top of the upstream `framework_lib` crate.

> **Privilege note:** Framework’s EC driver checks SMBIOS data and uses raw I/O.
> Expect to run both the CLI and GUI with administrative (root/Administrator)
> privileges unless you have configured the necessary device permissions or
> capabilities.

## Features

- **Shared core** logic for parsing colors and invoking `framework_lib` EC commands.
- **CLI tool** to set a contiguous range of up to 64 keys with arbitrary colors.
- **GUI (egui/eframe)** with eight color pickers, whimsical presets (Stack Overflow Rainbow,
  Corporate Compliance Orange, Terminal Green Matrix, etc.), driver selection, auto‑apply, and a
  lighting toggle.
- **Driver selection** allowing you to force the EC transport (`portio`, `cros_ec`,
  or Windows HID) when the automatic choice is not ideal.

## Repository Layout

- `src/lib.rs`: shared helpers (color parsing/formatting and EC dispatch).
- `src/main.rs`: CLI front-end (`--start`, `--driver`, positional colors).
- `src/bin/gui.rs`: egui desktop app for the fan lighting.

The repository vendors the upstream `framework_lib` crate in `framework_lib/`,
so no sibling checkout is required.

## Building

```bash
cargo clean
cargo build --release
```

Both binaries (`fwd_rgb` and the `fwdrgb` target) link against `framework_lib`, so a
recent Rust toolchain (1.74+) and the dependencies of the upstream project
(libusb, hidapi, etc.) are required.

### CLI Usage

```bash
sudo cargo run --release -- \
  --start 0 \
  0xFF0000 0xFF7F00 0xFFFF00 0x00FF00 0x0000FF 0x4B0082 0x9400D3 0xFFFFFF
```

- Colors accept `0xRRGGBB`, `#RRGGBB`, or decimal literals.
- Elevated privileges are usually required to access SMBIOS data and the EC.

### GUI Usage / Install

```bash
sudo cargo run --release --bin fwdrgb
```

- Adjust the eight color pickers, use presets, or randomize the palette.
- Toggle the lighting on/off from the right-hand pane or the central toggle
  button.
- Enable "Auto-apply after changes" to push updates immediately whenever a
  control changes.

#### Install the GUI for later use

Build once in release mode and place the binary on your `PATH`:

```bash
cargo build --release --bin fwdrgb
sudo install -m 0755 target/release/fwdrgb /usr/local/bin/fwdrgb
```

You can now launch it any time with:

```bash
sudo fwdrgb
```

Adjust the install destination to match your environment (`/usr/local/bin`,
`~/.local/bin`, etc.). Root privileges remain necessary on Framework systems for
SMBIOS access and EC commands unless you grant the binary the required
capabilities via policy or udev rules.

## Running Outside of a Workspace

The project already includes `framework_lib`, so you can clone this repository
on its own. To track upstream changes, either periodically re-vendor
`framework_lib` or replace the dependency with a git reference:

```toml
framework_lib = { git = "https://github.com/FrameworkComputer/framework-system", package = "framework_lib" }
```

Because the library performs SMBIOS checks, you still need administrative
privileges (or equivalent capabilities) for EC access on the target system.

## License

Follows the upstream `framework_lib` license (BSD-3-Clause). See `LICENSE.md` if
you copy the library sources into a standalone repository.
