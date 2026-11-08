# capstoggle

Tiny Rust CLI to remap the CapsLock key. Toggle it between **Ctrl**, **Esc** and normal CapsLock with one command, handy as a keyboard shortcut.

It picks a backend automatically:

- **kanata** (if a `kanata.service` systemd user unit exists): rewrites `~/.config/kanata/kanata.kbd` and starts/stops the service. Needed on setups where kanata already grabs the keyboard, since it overrides xkb.
- **xkb** (otherwise): edits GNOME's `xkb-options` via `gsettings`, leaving other xkb options alone.

Force xkb with `CAPSTOGGLE_BACKEND=xkb`.

## Quick install

Requires [Rust](https://rustup.rs). GNOME for the xkb backend, or [kanata](https://github.com/jtroo/kanata) with a `kanata.service` user unit for the kanata backend.

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
capstoggle both      toggle CapsLock <-> Esc on tap / Ctrl on hold (kanata only)
capstoggle status    show current mapping
capstoggle reset     restore normal CapsLock
capstoggle help      show help
```

Switching between modes replaces the old mapping. On the kanata backend the first change saves your existing config as `kanata.kbd.bak` and then overwrites `kanata.kbd` (a caps-only config), so keep other kanata mappings elsewhere. Stopping (normal) runs `systemctl --user disable --now kanata`.

### Bind to a hotkey

GNOME Settings -> Keyboard -> Custom Shortcuts -> add a command such as `capstoggle esc` (use the full path, e.g. `/home/you/.cargo/bin/capstoggle`, since shortcuts may not see your shell `PATH`).

## How it works

xkb backend toggles `ctrl:nocaps` or `caps:escape` in `org.gnome.desktop.input-sources xkb-options`. kanata backend writes a one-key `defsrc caps` config.

## Limitations

- xkb backend is GNOME only (needs `gsettings`); elsewhere use `setxkbmap -option ctrl:nocaps` / `caps:escape`.
- kanata backend assumes the service runs `~/.config/kanata/kanata.kbd`.

## License

MIT
