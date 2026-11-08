// Remap the CapsLock key (GNOME gsettings, persistent).
use std::{env, process::Command};

const SCHEMA: &str = "org.gnome.desktop.input-sources";
const KEY: &str = "xkb-options";
// (name, xkb option, label)
const MODES: [(&str, &str, &str); 2] = [
    ("ctrl", "ctrl:nocaps", "Ctrl"),
    ("esc", "caps:escape", "Esc"),
];

const HELP: &str = "capstoggle - remap the CapsLock key

USAGE:
  capstoggle [ctrl]    toggle CapsLock <-> Ctrl (default)
  capstoggle esc       toggle CapsLock <-> Esc
  capstoggle status    show current mapping
  capstoggle reset     restore normal CapsLock
  capstoggle help      show this help

Switching from ctrl to esc (or back) replaces the old mapping.
Other xkb options are kept. GNOME only; change persists across logins.";

fn gs(args: &[&str]) -> String {
    let o = Command::new("gsettings").args(args).output().expect("gsettings not found");
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    String::from_utf8_lossy(&o.stdout).trim().to_string()
}

fn parse(s: &str) -> Vec<String> {
    // shortcut: assumes option names contain no quotes/commas, true for xkb options
    s.split('\'').skip(1).step_by(2).map(String::from).collect()
}

fn save(opts: &[String]) {
    let list = format!("[{}]", opts.iter().map(|o| format!("'{o}'")).collect::<Vec<_>>().join(", "));
    gs(&["set", SCHEMA, KEY, &list]);
}

fn current(opts: &[String]) -> &'static str {
    MODES.iter().find(|m| opts.iter().any(|o| o == m.1)).map_or("normal", |m| m.2)
}

fn main() {
    let arg = env::args().nth(1).unwrap_or_else(|| "ctrl".into());
    let mut opts = parse(&gs(&["get", SCHEMA, KEY]));
    match arg.as_str() {
        "help" | "-h" | "--help" => println!("{HELP}"),
        "status" => println!("CapsLock: {}", current(&opts)),
        "reset" => {
            opts.retain(|o| !MODES.iter().any(|m| m.1 == o));
            save(&opts);
            println!("CapsLock: normal");
        }
        name => {
            let Some(target) = MODES.iter().find(|m| m.0 == name) else {
                eprintln!("unknown option '{name}'\n\n{HELP}");
                std::process::exit(2);
            };
            let was_on = opts.iter().any(|o| o == target.1);
            opts.retain(|o| !MODES.iter().any(|m| m.1 == o));
            if !was_on {
                opts.push(target.1.into());
            }
            save(&opts);
            println!("CapsLock: {}", current(&opts));
        }
    }
}
