use clap::Parser;
use pagurus::event::{Event, Key, KeyEvent};
use pagurus::failure::OrFail;
use pagurus::Game;
use pagurus_tui::TuiSystem;
use std::fs::OpenOptions;
use std::io::Write;

#[derive(Parser)]
struct Args {}

fn main() -> pagurus::Result<()> {
    let _args = Args::parse();

    pagurus::io::set_println_fn(file_println).or_fail()?;
    let mut system = TuiSystem::new().or_fail()?;
    let mut game = dotedit::game::Game::default();
    game.initialize(&mut system).or_fail()?;
    while let Ok(event) = system.next_event() {
        if is_quit_key(&event) {
            break;
        }
        if !game.handle_event(&mut system, event).or_fail()? {
            break;
        }
    }

    Ok(())
}

fn file_println(msg: &str) {
    let _ = OpenOptions::new()
        .create(true)
        .append(true)
        .open("dotedit.log")
        .and_then(|mut file| writeln!(file, "{}", msg));
}

fn is_quit_key(event: &Event) -> bool {
    let Event::Key(KeyEvent { key, ctrl,.. }) = event else {
        return false;
    };
    matches!((key, ctrl), (Key::Esc, _) | (Key::Char('c'), true))
}
