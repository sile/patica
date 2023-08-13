use dotedit::model::{Command, DefineCommand};
use pagurus::{failure::OrFail, image::Color};
use std::io::Write;

fn main() -> pagurus::Result<()> {
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    for color in copic_colors::ALL_COLORS {
        let commnad = Command::Comment(serde_json::json!({
            "code": color.code,
            "name": color.name,
            "family": format!("{:?}", color.family),
            "group": format!("{:?}", color.group),
            "value": format!("{:?}", color.value),
        }));
        serde_json::to_writer(&mut stdout, &commnad).or_fail()?;
        writeln!(&mut stdout).or_fail()?;

        let rgb = color.rgb;
        let commnad = Command::Define(DefineCommand::new(
            format!("copic.{}", color.code.to_lowercase()),
            Color::rgb(rgb.r, rgb.g, rgb.b),
        ));
        serde_json::to_writer(&mut stdout, &commnad).or_fail()?;
        writeln!(&mut stdout).or_fail()?;
    }
    Ok(())
}
