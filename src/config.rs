use pagurus::{
    event::KeyEvent,
    failure::{Failure, OrFail},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::model::Command;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub key: KeyConfig,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct KeyConfig(BTreeMap<Key, Command>);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "&str", into = "String")]
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

impl TryFrom<&str> for Key {
    type Error = Failure;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let mut ctrl = false;
        let mut alt = false;
        let mut tokens = s.split('+').collect::<Vec<_>>();

        let last = tokens
            .pop()
            .or_fail()
            .map_err(|f| f.message("Empty key string"))?;
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
                'a'..='z' => pagurus::event::Key::Char(last.chars().next().or_fail()?),
                'A'..='Z' => pagurus::event::Key::Char(last.chars().next().or_fail()?),
                '0'..='9' => pagurus::event::Key::Char(last.chars().next().or_fail()?),
                _ => return Err(Failure::new().message("Unknown key: {last:?}")),
            },
            _ => return Err(Failure::new().message("Unknown key: {last:?}")),
        };
        for token in tokens {
            match token {
                "Ctrl" => ctrl = true,
                "Alt" => alt = true,
                _ => return Err(Failure::new().message(format!("Unknown key modifier: {token:?}"))),
            }
        }

        Ok(Self(KeyEvent { key, ctrl, alt }))
    }
}
