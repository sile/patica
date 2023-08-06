use clap::Parser;
use dotedit::commands::Command;
use pagurus::failure::OrFail;
use std::fs::OpenOptions;
use std::io::Write;

#[derive(Parser)]
#[clap(version, about)]
struct Args {
    #[clap(subcommand)]
    command: Command,
}

fn main() -> pagurus::Result<()> {
    pagurus::io::set_println_fn(file_println).or_fail()?;

    let args = Args::parse();
    args.command.run()?;

    Ok(())
}

fn file_println(msg: &str) {
    let _ = OpenOptions::new()
        .create(true)
        .append(true)
        .open("dotedit.log")
        .and_then(|mut file| writeln!(file, "{}", msg));
}
