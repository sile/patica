use clap::Parser;
use orfail::OrFail;
use patica::cli::Args;

fn main() -> orfail::Result<()> {
    let args = Args::parse();
    args.run().or_fail()?;
    Ok(())
}
