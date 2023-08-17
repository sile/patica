use copic_colors::{Family, Group, Value};
use pagurus::{failure::OrFail, image::Color};
use patica::{
    marker::MarkKind,
    model::{AnchorName, CameraPosition, Command, SetCommand},
};
use std::{collections::HashMap, io::Write};

const PALETTE_WIDTH: i16 = 26;
const PALETTE_HEIGHT: i16 = 47;

fn main() -> pagurus::Result<()> {
    let mut colors = HashMap::new();
    for color in copic_colors::ALL_COLORS {
        let rgb = color.rgb;
        colors.insert(
            (color.family, color.group, color.value),
            Color::rgb(rgb.r, rgb.g, rgb.b),
        );
    }

    let palette_start_anchor = AnchorName::new("palette.start");
    let palette_end_anchor = AnchorName::new("palette.end");
    write_command(Command::Anchor(palette_start_anchor.clone())).or_fail()?;
    write_background().or_fail()?;
    write_command(Command::Anchor(palette_end_anchor.clone())).or_fail()?;

    // Reference: https://copic.too.com/blogs/educational/how-are-copic-colors-organized-and-named
    let families = [
        Family::Violet,
        Family::RedViolet,
        Family::Red,
        Family::YellowRed,
        Family::Yellow,
        Family::YellowGreen,
        //
        Family::BlueViolet,
        Family::Earth,
        Family::Blue,
        Family::BlueGreen,
        Family::Green,
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
    let mut row = 1;
    let mut columns = 1;
    for family in families {
        if family == Family::BlueViolet {
            row = 1;
            columns = values.len() + 2;
        }

        for group in groups {
            write_command(Command::Set(SetCommand::Cursor(
                palette_start_anchor.clone(),
            )))
            .or_fail()?;
            write_command(Command::Move((columns as i16, row as i16).into())).or_fail()?;

            if values
                .into_iter()
                .all(|value| !colors.contains_key(&(family, group, value)))
            {
                continue;
            }

            for value in values {
                let color = colors
                    .get(&(family, group, value))
                    .copied()
                    .unwrap_or(Color::WHITE);
                write_color(color).or_fail()?;
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
        write_command(Command::Set(SetCommand::Cursor(
            palette_start_anchor.clone(),
        )))
        .or_fail()?;
        write_command(Command::Move((1, row as i16).into())).or_fail()?;

        for value in values {
            let color = colors
                .get(&(family, Group::Undefined, value))
                .copied()
                .unwrap_or(Color::WHITE);
            write_color(color).or_fail()?;
        }
        row += 1;
    }

    write_command(Command::Set(SetCommand::Cursor(
        palette_start_anchor.clone(),
    )))
    .or_fail()?;
    write_command(Command::Move((1, row as i16).into())).or_fail()?;
    for color in [
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
        copic_colors::COLOR_100,
        copic_colors::COLOR_110,
    ] {
        let color = Color::rgb(color.rgb.r, color.rgb.g, color.rgb.b);
        write_color(color).or_fail()?;
    }

    write_command(Command::Set(SetCommand::Cursor(
        palette_start_anchor.clone(),
    )))
    .or_fail()?;
    write_command(Command::Move(
        (PALETTE_WIDTH / 2, PALETTE_HEIGHT / 2).into(),
    ))
    .or_fail()?;
    write_command(Command::Set(SetCommand::Camera(CameraPosition::Pixel(
        (0, 0).into(),
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

fn write_color(color: Color) -> pagurus::Result<()> {
    write_command(Command::Mark(MarkKind::Line)).or_fail()?;
    write_command(Command::Set(SetCommand::Color(color))).or_fail()?;
    write_command(Command::Draw).or_fail()?;
    write_command(Command::Move((1, 0).into())).or_fail()?;
    Ok(())
}

fn write_background() -> pagurus::Result<()> {
    write_command(Command::Mark(MarkKind::FillRectangle)).or_fail()?;
    write_command(Command::Move((PALETTE_WIDTH, PALETTE_HEIGHT).into())).or_fail()?;
    write_command(Command::Set(SetCommand::Color(Color::rgb(164, 163, 156)))).or_fail()?;
    write_command(Command::Draw).or_fail()?;
    Ok(())
}
