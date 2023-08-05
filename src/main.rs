use clap::Parser;

#[derive(Parser)]
struct Args {}

fn main() -> pagurus::Result<()> {
    let args = Args::parse();
    Ok(())
}
