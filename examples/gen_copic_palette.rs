use copic_colors::{Family, Group, Value};
use pagurus::{failure::OrFail, image::Color};
use patica::{
    marker::MarkKind,
    model::{AnchorName, CameraPosition, ColorName, Command, DefineCommand, SetCommand},
};
use std::{collections::HashMap, io::Write};

fn main() -> pagurus::Result<()> {
    let mut colors = HashMap::new();
    for color in copic_colors::ALL_COLORS {
        let command = Command::Comment(serde_json::json!({
            "code": color.code,
            "name": color.name,
            "family": format!("{:?}", color.family),
            "group": format!("{:?}", color.group),
            "value": format!("{:?}", color.value),
        }));
        write_command(command).or_fail()?;

        let rgb = color.rgb;
        let command = Command::Define(DefineCommand::new(
            format!("copic.{}", color.code.to_lowercase()),
            Color::rgb(rgb.r, rgb.g, rgb.b),
        ));
        write_command(command).or_fail()?;

        colors.insert((color.family, color.group, color.value), color);
    }

    let command = Command::Anchor(AnchorName("palette".to_owned()));
    write_command(command).or_fail()?;

    // Reference: https://copic.too.com/blogs/educational/how-are-copic-colors-organized-and-named
    let families = [
        Family::BlueViolet,
        Family::Violet,
        Family::RedViolet,
        Family::Red,
        Family::YellowRed,
        Family::Yellow,
        Family::YellowGreen,
        Family::Green,
        Family::BlueGreen,
        Family::Blue,
        Family::Earth,
    ];
    let groups = [
        Group::S0,
        Group::S1,
        Group::S2,
        Group::S3,
        Group::S4,
        Group::S5,
        Group::S6,
        Group::S7,
        Group::S8,
        Group::S9,
    ];
    let values = [
        Value::B000,
        Value::B00,
        Value::B0,
        Value::B1,
        Value::B2,
        Value::B3,
        Value::B4,
        Value::B5,
        Value::B6,
        Value::B7,
        Value::B8,
        Value::B9,
    ];
    let mut row = 0;
    for family in families {
        for group in groups {
            write_command(Command::Set(SetCommand::Cursor(AnchorName(
                "palette".to_owned(),
            ))))
            .or_fail()?;
            write_command(Command::Move((0, row as i16).into())).or_fail()?;

            if values
                .into_iter()
                .all(|value| !colors.contains_key(&(family, group, value)))
            {
                continue;
            }

            for value in values {
                let color_name = if let Some(color) = colors.get(&(family, group, value)) {
                    ColorName(format!("copic.{}", color.code.to_lowercase()))
                } else {
                    ColorName("copic.0".to_owned())
                };
                write_command(Command::Mark(MarkKind::Line)).or_fail()?;
                write_command(Command::Set(SetCommand::Color(color_name))).or_fail()?;
                write_command(Command::Draw).or_fail()?;
                write_command(Command::Move((1, 0).into())).or_fail()?;
            }
            row += 1;
        }
        row += 1;
    }

    for family in [
        Family::CoolGray,
        Family::NeutralGray,
        Family::TonerGray,
        Family::WarmGray,
    ] {
        write_command(Command::Set(SetCommand::Cursor(AnchorName(
            "palette".to_owned(),
        ))))
        .or_fail()?;
        write_command(Command::Move((0, row as i16).into())).or_fail()?;

        for value in values {
            let color_name = if let Some(color) = colors.get(&(family, Group::Undefined, value)) {
                ColorName(format!("copic.{}", color.code.to_lowercase()))
            } else {
                ColorName("copic.0".to_owned())
            };
            write_command(Command::Mark(MarkKind::Line)).or_fail()?;
            write_command(Command::Set(SetCommand::Color(color_name))).or_fail()?;
            write_command(Command::Draw).or_fail()?;
            write_command(Command::Move((1, 0).into())).or_fail()?;
        }
        row += 1;
    }

    write_command(Command::Set(SetCommand::Cursor(AnchorName(
        "palette".to_owned(),
    ))))
    .or_fail()?;
    write_command(Command::Move((0, row as i16).into())).or_fail()?;
    for color in [
        copic_colors::COLOR_0,
        copic_colors::COLOR_0,
        copic_colors::COLOR_0,
        copic_colors::COLOR_0,
        copic_colors::COLOR_FV,
        copic_colors::COLOR_FRV,
        copic_colors::COLOR_FYR,
        copic_colors::COLOR_FY,
        copic_colors::COLOR_FYG,
        copic_colors::COLOR_FG,
        copic_colors::COLOR_FBG,
        copic_colors::COLOR_FB,
    ] {
        let color_name = ColorName(format!("copic.{}", color.code.to_lowercase()));
        write_command(Command::Mark(MarkKind::Line)).or_fail()?;
        write_command(Command::Set(SetCommand::Color(color_name))).or_fail()?;
        write_command(Command::Draw).or_fail()?;
        write_command(Command::Move((1, 0).into())).or_fail()?;
    }

    write_command(Command::Set(SetCommand::Cursor(AnchorName(
        "palette".to_owned(),
    ))))
    .or_fail()?;
    write_command(Command::Move((0, row as i16 / 2).into())).or_fail()?;
    write_command(Command::Set(SetCommand::Camera(CameraPosition::Pixel(
        (0, 0).into(),
    ))))
    .or_fail()?;
    write_command(Command::Set(SetCommand::Color(ColorName(
        "copic.n-10".to_owned(),
    ))))
    .or_fail()?;

    Ok(())
}

fn write_command(command: Command) -> pagurus::Result<()> {
    let mut stdout = std::io::stdout();
    serde_json::to_writer(&mut stdout, &command).or_fail()?;
    writeln!(&mut stdout).or_fail()?;
    Ok(())
}
