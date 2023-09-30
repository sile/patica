use orfail::{Failure, OrFail};
use pagurus::event::{self, KeyEvent};
use paticanvas::CanvasCommand;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub key: KeyConfig,

    #[serde(default)]
    pub on_create: Option<CanvasCommand>,

    #[serde(default)]
    pub on_open: Option<CanvasCommand>,
}

// impl Config {
//     pub fn load_config_file() -> orfail::Result<Option<Self>> {
//         let Ok(home_dir) = std::env::var("HOME") else {
//             return Ok(None);
//         };

//         let path = Path::new(&home_dir).join(".config").join("patica.json");
//         if !path.exists() {
//             return Ok(None);
//         }

//         let json = std::fs::read_to_string(&path)
//             .or_fail_with(|e| format!("Failed to read config file {}: {e}", path.display()))?;
//         serde_json::from_str(&json)
//             .or_fail_with(|e| {
//                 format!(
//                     "Failed to parse config file: path={}, reason={e}",
//                     path.display()
//                 )
//             })
//             .map(Some)
//     }
// }

impl Default for Config {
    fn default() -> Self {
        serde_json::from_str(include_str!("../default-config.json")).expect("unreachable")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyConfig(BTreeMap<Key, CanvasCommand>);

impl KeyConfig {
    pub fn get_command(&self, key: KeyEvent) -> Option<&CanvasCommand> {
        self.0.get(&Key(key))
    }
}

impl Default for KeyConfig {
    fn default() -> Self {
        Config::default().key
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Key(KeyEvent);

impl PartialOrd for Key {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Key {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.to_string().cmp(&other.to_string())
    }
}

impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.ctrl {
            write!(f, "Ctrl+")?;
        }
        if self.0.alt {
            write!(f, "Alt+")?;
        }
        match self.0.key {
            event::Key::Return => write!(f, "Enter"),
            event::Key::Left => write!(f, "Left"),
            event::Key::Right => write!(f, "Right"),
            event::Key::Up => write!(f, "Up"),
            event::Key::Down => write!(f, "Down"),
            event::Key::Backspace => write!(f, "Backspace"),
            event::Key::Delete => write!(f, "Delete"),
            event::Key::Tab => write!(f, "Tab"),
            event::Key::BackTab => write!(f, "BackTab"),
            event::Key::Esc => write!(f, "Esc"),
            event::Key::Char(c) => write!(f, "{}", c),
            _ => unreachable!(),
        }
    }
}

impl From<Key> for String {
    fn from(key: Key) -> Self {
        key.to_string()
    }
}

impl TryFrom<String> for Key {
    type Error = Failure;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        let mut ctrl = false;
        let mut alt = false;
        let mut i = 0;
        loop {
            if s[i..].starts_with("Ctrl+") {
                ctrl = true;
                i += 5;
            } else if s[i..].starts_with("Alt+") {
                alt = true;
                i += 4;
            } else {
                break;
            }
        }

        let last = &s[i..];
        let key = match last {
            "Enter" => event::Key::Return,
            "Left" => event::Key::Left,
            "Right" => event::Key::Right,
            "Up" => event::Key::Up,
            "Down" => event::Key::Down,
            "Backspace" => event::Key::Backspace,
            "Delete" => event::Key::Delete,
            "Tab" => event::Key::Tab,
            "BackTab" => event::Key::BackTab,
            "Esc" => event::Key::Esc,
            _ if last.chars().count() == 1 => match last.chars().next().or_fail()? {
                c @ ('a'..='z'
                | 'A'..='Z'
                | '0'..='9'
                | ' '
                | '+'
                | '-'
                | '_'
                | '('
                | ')'
                | '{'
                | '}'
                | '['
                | ']'
                | '<'
                | '>'
                | ';'
                | ':'
                | '='
                | '.'
                | ','
                | '!'
                | '?'
                | '/'
                | '@'
                | '#'
                | '$'
                | '%'
                | '^'
                | '&'
                | '*'
                | '"'
                | '\''
                | '`'
                | '~') => event::Key::Char(c),
                _ => return Err(Failure::new(format!("Unknown key: {last:?}"))),
            },
            _ => return Err(Failure::new(format!("Unknown key: {last:?}"))),
        };

        Ok(Self(KeyEvent { key, ctrl, alt }))
    }
}
