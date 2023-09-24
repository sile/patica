use copic_colors::{Family, Group, Value};
use orfail::OrFail;
use pati::{Color, ImageCommand, Point};
use std::{
    collections::{BTreeMap, HashMap},
    io::Write,
};

const PALETTE_WIDTH: i16 = 27;
const PALETTE_HEIGHT: i16 = 48;

fn main() -> pagurus::Result<()> {
    let mut pixels: BTreeMap<Point, Color> = BTreeMap::new();
    add_background(&mut pixels);
    add_colors(&mut pixels);

    write_command(ImageCommand::draw_pixels(pixels.into_iter())).or_fail()?;
    write_command(ImageCommand::anchor(
        "palette.start",
        Some(Point::new(0, 0)),
    ))
    .or_fail()?;
    write_command(ImageCommand::anchor(
        "palette.end",
        Some(Point::new(PALETTE_WIDTH - 1, PALETTE_HEIGHT - 1)),
    ))
    .or_fail()?;
    write_command(ImageCommand::anchor(
        "origin",
        Some(Point::new(PALETTE_WIDTH / 2, PALETTE_HEIGHT / 2)),
    ))
    .or_fail()?;

    Ok(())
}

fn write_command(command: ImageCommand) -> orfail::Result<()> {
    let mut stdout = std::io::stdout();
    serde_json::to_writer(&mut stdout, &command).or_fail()?;
    writeln!(&mut stdout).or_fail()?;
    Ok(())
}

fn add_background(pixels: &mut BTreeMap<Point, Color>) {
    for y in 0..PALETTE_HEIGHT {
        for x in 0..PALETTE_WIDTH {
            pixels.insert((x, y).into(), Color::rgb(164, 163, 156));
        }
    }
}

fn add_colors(pixels: &mut BTreeMap<Point, Color>) {
    let mut colors = HashMap::new();
    for color in copic_colors::ALL_COLORS {
        let rgb = color.rgb;
        colors.insert(
            (color.family, color.group, color.value),
            Color::rgb(rgb.r, rgb.g, rgb.b),
        );
    }

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
            let mut point = Point::new(columns as i16, row as i16);
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
                    .unwrap_or(Color::rgb(255, 255, 255));
                pixels.insert(point, color);
                point.x += 1;
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
        let mut point = Point::new(1, row as i16);
        for value in values {
            let color = colors
                .get(&(family, Group::Undefined, value))
                .copied()
                .unwrap_or(Color::rgb(255, 255, 255));
            pixels.insert(point, color);
            point.x += 1;
        }
        row += 1;
    }

    let mut point = Point::new(1, row as i16);
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
        pixels.insert(point, color);
        point.x += 1;
    }
}
