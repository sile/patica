use clap::Parser;
use dotedit::commands::Command;
use pagurus::failure::OrFail;
use std::fs::OpenOptions;
use std::io::Write;

#[derive(Debug, Parser)]
#[clap(version, about)]
struct Args {
    #[clap(subcommand)]
    command: Command,
}

fn main() -> pagurus::Result<()> {
    pagurus::io::set_println_fn(file_println).or_fail()?;

    let args = Args::parse();
    let result = args.command.run().or_fail();
    if let Err(e) = &result {
        pagurus::println!("Args: {args:?}");
        pagurus::println!("Error: {e}");
    }
    result
}

fn file_println(msg: &str) {
    let _ = OpenOptions::new()
        .create(true)
        .append(true)
        .open("dotedit.log")
        .and_then(|mut file| writeln!(file, "{}", msg));
}
