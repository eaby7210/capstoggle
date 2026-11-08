# capstoggle

Tiny Rust CLI to remap the CapsLock key on GNOME. Toggle it between **Ctrl**, **Esc** and normal CapsLock with one command, handy as a keyboard shortcut.

It edits GNOME's `xkb-options` setting via `gsettings`, so the change is persistent and other xkb options are left alone.

## Quick install

Requires [Rust](https://rustup.rs) and GNOME.

```sh
cargo install --git https://github.com/eaby7210/capstoggle
```

This puts `capstoggle` in `~/.cargo/bin` (already on `PATH` with a standard rustup install).

Or build from source:

```sh
git clone https://github.com/eaby7210/capstoggle
cd capstoggle
cargo install --path .
```

## Usage

```
capstoggle [ctrl]    toggle CapsLock <-> Ctrl (default)
capstoggle esc       toggle CapsLock <-> Esc
capstoggle status    show current mapping
capstoggle reset     restore normal CapsLock
capstoggle help      show help
```

Switching from Ctrl to Esc (or back) replaces the old mapping.

### Bind to a hotkey

GNOME Settings -> Keyboard -> Custom Shortcuts -> add a command such as `capstoggle esc` (use the full path, e.g. `/home/you/.cargo/bin/capstoggle`, since shortcuts may not see your shell `PATH`).

## How it works

Toggles `ctrl:nocaps` or `caps:escape` in `org.gnome.desktop.input-sources xkb-options`.

## Limitations

- GNOME only (needs `gsettings`).
- Other desktops: use `setxkbmap -option ctrl:nocaps` / `caps:escape` instead.

## License

MIT
