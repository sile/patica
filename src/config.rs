use crate::command::Command;
use orfail::OrFail;
use pagurus::event::KeyEvent;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub key: KeyConfig,
}

impl Config {
    pub fn load_config_file() -> orfail::Result<Option<Self>> {
        let Ok(home_dir) = std::env::var("HOME") else {
            return Ok(None);
        };

        let path = Path::new(&home_dir).join(".config").join("patica.json");
        if !path.exists() {
            return Ok(None);
        }

        let json = std::fs::read_to_string(&path)
            .or_fail_with(|e| format!("Failed to read config file {}: {e}", path.display()))?;
        serde_json::from_str(&json)
            .or_fail_with(|e| {
                format!(
                    "Failed to parse config file: path={}, reason={e}",
                    path.display()
                )
            })
            .map(Some)
    }
}

impl Default for Config {
    fn default() -> Self {
        serde_json::from_str(include_str!("../default-config.json")).expect("unreachable")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyConfig(BTreeMap<Key, Vec<Command>>);

impl KeyConfig {
    pub fn get_command(&self, key: KeyEvent) -> Option<Vec<Command>> {
        let key = Key(key);
        self.0.get(&key).cloned()
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
            pagurus::event::Key::Return => write!(f, "Enter"),
            pagurus::event::Key::Left => write!(f, "Left"),
            pagurus::event::Key::Right => write!(f, "Right"),
            pagurus::event::Key::Up => write!(f, "Up"),
            pagurus::event::Key::Down => write!(f, "Down"),
            pagurus::event::Key::Backspace => write!(f, "Backspace"),
            pagurus::event::Key::Delete => write!(f, "Delete"),
            pagurus::event::Key::Tab => write!(f, "Tab"),
            pagurus::event::Key::BackTab => write!(f, "BackTab"),
            pagurus::event::Key::Esc => write!(f, "Esc"),
            pagurus::event::Key::Char(c) => write!(f, "{}", c),
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
    type Error = orfail::Failure;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        let mut ctrl = false;
        let mut alt = false;
        let mut tokens = if s == "+" {
            // TODO
            vec!["+"]
        } else {
            s.split('+').collect::<Vec<_>>()
        };

        let last = tokens
            .pop()
            .or_fail_with(|()| "Empty key string".to_owned())?;
        let key = match last {
            "Enter" => pagurus::event::Key::Return,
            "Left" => pagurus::event::Key::Left,
            "Right" => pagurus::event::Key::Right,
            "Up" => pagurus::event::Key::Up,
            "Down" => pagurus::event::Key::Down,
            "Backspace" => pagurus::event::Key::Backspace,
            "Delete" => pagurus::event::Key::Delete,
            "Tab" => pagurus::event::Key::Tab,
            "BackTab" => pagurus::event::Key::BackTab,
            "Esc" => pagurus::event::Key::Esc,
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
                | '~') => pagurus::event::Key::Char(c),
                _ => return Err(orfail::Failure::new(format!("Unknown key: {last:?}"))),
            },
            _ => return Err(orfail::Failure::new(format!("Unknown key: {last:?}"))),
        };
        for token in tokens {
            match token {
                "Ctrl" => ctrl = true,
                "Alt" => alt = true,
                _ => {
                    return Err(orfail::Failure::new(format!(
                        "Unknown key modifier: {token:?}"
                    )))
                }
            }
        }

        Ok(Self(KeyEvent { key, ctrl, alt }))
    }
}
