// Remap the CapsLock key. Uses kanata (user service) when installed, else GNOME gsettings.
use std::{env, fs, path::PathBuf, process::Command};

const SCHEMA: &str = "org.gnome.desktop.input-sources";
const KEY: &str = "xkb-options";
// (name, xkb option, label)
const MODES: [(&str, &str, &str); 2] = [
    ("ctrl", "ctrl:nocaps", "Ctrl"),
    ("esc", "caps:escape", "Esc"),
];

// kanata backend: (name, caps action, label)
const KANATA_MODES: [(&str, &str, &str); 3] = [
    ("ctrl", "lctl", "Ctrl"),
    ("esc", "esc", "Esc"),
    ("both", "(tap-hold-press 200 200 esc lctl)", "Esc on tap, Ctrl on hold"),
];

const HELP: &str = "capstoggle - remap the CapsLock key

USAGE:
  capstoggle [ctrl]    toggle CapsLock <-> Ctrl (default)
  capstoggle esc       toggle CapsLock <-> Esc
  capstoggle both      toggle CapsLock <-> Esc on tap / Ctrl on hold (kanata only)
  capstoggle status    show current mapping
  capstoggle reset     restore normal CapsLock
  capstoggle help      show this help

Backend: if the user service kanata.service exists, capstoggle rewrites
~/.config/kanata/kanata.kbd (old one saved once as kanata.kbd.bak) and
starts/stops the service. Otherwise it edits GNOME xkb-options via gsettings.
Force xkb with CAPSTOGGLE_BACKEND=xkb. Changes persist across logins.";

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

fn xkb_main(arg: &str) {
    let mut opts = parse(&gs(&["get", SCHEMA, KEY]));
    match arg {
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

fn systemctl(args: &[&str]) -> bool {
    Command::new("systemctl").arg("--user").args(args).status().map_or(false, |s| s.success())
}

fn kanata_cfg() -> PathBuf {
    PathBuf::from(env::var("HOME").expect("HOME not set")).join(".config/kanata/kanata.kbd")
}

// Caps action from the `(deflayer base ...)` block, mapped to a mode label.
fn kanata_label(cfg: &str) -> &'static str {
    let body = cfg.split("(deflayer base").nth(1).unwrap_or("").trim();
    let action = body.strip_suffix(')').unwrap_or(body).trim();
    KANATA_MODES.iter().find(|m| m.1 == action).map_or("custom", |m| m.2)
}

fn kanata_current() -> &'static str {
    if !systemctl(&["is-active", "--quiet", "kanata"]) {
        return "normal";
    }
    fs::read_to_string(kanata_cfg()).map_or("custom", |c| kanata_label(&c))
}

fn kanata_off() {
    assert!(systemctl(&["disable", "--now", "kanata"]), "failed to stop kanata");
}

fn kanata_main(arg: &str) {
    match arg {
        "status" => println!("CapsLock: {}", kanata_current()),
        "reset" => {
            kanata_off();
            println!("CapsLock: normal");
        }
        name => {
            let Some(target) = KANATA_MODES.iter().find(|m| m.0 == name) else {
                eprintln!("unknown option '{name}'\n\n{HELP}");
                std::process::exit(2);
            };
            if kanata_current() == target.2 {
                kanata_off();
            } else {
                let path = kanata_cfg();
                let bak = path.with_extension("kbd.bak");
                if path.exists() && !bak.exists() {
                    fs::copy(&path, &bak).expect("backup failed");
                }
                let cfg = format!("(defcfg\n  process-unmapped-keys yes\n)\n\n(defsrc\n  caps\n)\n\n(deflayer base\n  {}\n)\n", target.1);
                fs::write(&path, cfg).expect("write kanata config failed");
                assert!(systemctl(&["enable", "kanata"]) && systemctl(&["restart", "kanata"]), "failed to start kanata");
            }
            println!("CapsLock: {}", kanata_current());
        }
    }
}

fn main() {
    let arg = env::args().nth(1).unwrap_or_else(|| "ctrl".into());
    if matches!(arg.as_str(), "help" | "-h" | "--help") {
        return println!("{HELP}");
    }
    let xkb = env::var("CAPSTOGGLE_BACKEND").is_ok_and(|b| b == "xkb");
    let kanata = !xkb && Command::new("systemctl").args(["--user", "cat", "kanata.service"]).output().is_ok_and(|o| o.status.success());
    if kanata {
        kanata_main(&arg)
    } else {
        if arg == "both" {
            eprintln!("'both' needs kanata\n\n{HELP}");
            std::process::exit(2);
        }
        xkb_main(&arg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels() {
        let mk = |a: &str| format!("(defsrc\n  caps\n)\n\n(deflayer base\n  {a}\n)\n");
        assert_eq!(kanata_label(&mk("lctl")), "Ctrl");
        assert_eq!(kanata_label(&mk("esc")), "Esc");
        assert_eq!(kanata_label(&mk("(tap-hold-press 200 200 esc lctl)")), "Esc on tap, Ctrl on hold");
        assert_eq!(kanata_label(&mk("caps")), "custom");
    }
}
