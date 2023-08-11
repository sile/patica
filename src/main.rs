use clap::Parser;
use dotedit::cli::Args;
use pagurus::failure::OrFail;
use std::fs::OpenOptions;
use std::io::Write;

fn main() -> pagurus::Result<()> {
    pagurus::io::set_println_fn(file_println).or_fail()?;
    std::panic::set_hook(Box::new(|info| {
        // NOTE: TODO
        println!("{info}");
        pagurus::println!("{info}");
    }));

    let args = Args::parse();
    let result = args.run().or_fail();

    if result.is_err() {
        // NOTE: This is necessary in order to return from raw terminal mode.
        println!();
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
