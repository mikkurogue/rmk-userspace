# rmk-userspace

Personal keyboard firmware configs using [RMK](https://rmk.rs). Structured as a userspace repo with shared crates and per-keyboard firmware.

## Repo Structure

```
rmk-userspace/
  crates/
    oneshot/          # Callum-style oneshot mod state machine
    layer-rgb/        # Per-layer RGB color definitions (HSV/RGB)
  keyboards/
    crkbd-v4-1/       # Corne v4.1 Standard (RP2040, split)
  rust-toolchain.toml # Stable Rust + thumbv6m-none-eabi target
```

Keyboard crates are standalone (excluded from the workspace) because RMK's proc macros require `rmk` in the local `Cargo.toml`. Shared crates under `crates/` are workspace members.

## Prerequisites

Install the required cargo tools:

```sh
cargo install cargo-make cargo-binutils cargo-hex-to-uf2 flip-link
rustup component add llvm-tools
```

## Building

From a keyboard directory:

```sh
cd keyboards/crkbd-v4-1

# Build both halves and generate UF2 files
cargo make uf2

# Or build individually
cargo build --bin central --release
cargo build --bin peripheral --release
```

Output files are written to the keyboard directory:
- `crkbd-v4-1-central.uf2` — left half
- `crkbd-v4-1-peripheral.uf2` — right half

## Flashing

1. Hold the BOOT button on the left half and plug in USB (or double-tap reset). It mounts as `RPI-RP2`.
2. Copy `crkbd-v4-1-central.uf2` to the drive. It flashes and reboots automatically.
3. Repeat with the right half using `crkbd-v4-1-peripheral.uf2`.

Each half needs its own UF2 — central handles USB/keymap/behaviors, peripheral only does matrix scanning.

## Keyboards

### Corne v4.1 (`keyboards/crkbd-v4-1`)

Split 3x6+3 with RP2040, 23 WS2812 LEDs per half.

**Layers:**

| # | Name | Activation |
|---|------|------------|
| 0 | BASE | Default (QWERTY) |
| 1 | NUM  | D+F or J+K combo (hold) |
| 2 | SYM  | Right thumb MO(2) |
| 3 | EXT  | Left thumb MO(3) |
| 4 | FUNC | Tri-layer (EXT + SYM) |

**Features:**
- One-shot modifiers (`OSM`) on SYM/EXT/FUNC home rows (Ctrl, Shift, Alt, Gui)
- Combos: D+F / J+K for NUM layer, B+N for CapsWord
- Tri-layer: holding EXT + SYM activates FUNC
- Per-layer RGB: off (BASE), blue (NUM), purple (SYM), green (EXT), red (FUNC)
- Split communication via PIO half-duplex serial on GP12

**Config files:**
- `keyboard.toml` — pin mapping, keymap, split config, behaviors
- `src/central.rs` — left half firmware + RGB controller
- `src/peripheral.rs` — right half firmware

## Customization

Edit `keyboard.toml` to change the keymap, combos, or behavior settings. See the [RMK docs](https://rmk.rs/docs/configuration/index.html) for all available options.

Layer colors are defined in `src/central.rs` in the `layer_color()` function.
