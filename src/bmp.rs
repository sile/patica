use orfail::OrFail;
use pati::{Color, Point};
use std::{collections::BTreeMap, io::Write};

pub fn write_image<W: Write>(
    mut writer: W,
    width: u16,
    height: u16,
    pixels: impl Iterator<Item = (Point, Color)>,
) -> orfail::Result<()> {
    let image_data_offset: u32 = 54;
    let num_of_pixels = width as u32 * height as u32;
    let file_size: u32 = image_data_offset + num_of_pixels * 4;

    // File header.
    writer.write_all(b"BM").or_fail()?;
    writer.write_all(&file_size.to_le_bytes()).or_fail()?;
    writer.write_all(&[0, 0, 0, 0]).or_fail()?;
    writer
        .write_all(&image_data_offset.to_le_bytes())
        .or_fail()?;

    // Information header.
    writer.write_all(&[40, 0, 0, 0]).or_fail()?; // Header size.
    writer.write_all(&(width as i32).to_le_bytes()).or_fail()?;
    writer.write_all(&(height as i32).to_le_bytes()).or_fail()?;
    writer.write_all(&[1, 0]).or_fail()?; // Planes.
    writer.write_all(&[32, 0]).or_fail()?; // Bits per pixel.
    writer.write_all(&[0, 0, 0, 0]).or_fail()?; // No compression.
    writer
        .write_all(&(num_of_pixels * 4).to_le_bytes())
        .or_fail()?; // Image size.
    writer.write_all(&[0, 0, 0, 0]).or_fail()?; // Horizontal resolution.
    writer.write_all(&[0, 0, 0, 0]).or_fail()?; // Vertical resolution.
    writer.write_all(&[0, 0, 0, 0]).or_fail()?; // Colors in palette.
    writer.write_all(&[0, 0, 0, 0]).or_fail()?; // Important colors.

    // Image data.
    let pixels = pixels.collect::<BTreeMap<_, _>>();
    for y in 0..height {
        for x in 0..width {
            let c = pixels
                .get(&Point::new(x as i16, y as i16))
                .copied()
                .unwrap_or(Color::rgba(255, 255, 255, 0));
            writer.write_all(&[c.b, c.g, c.r, c.a]).or_fail()?;
        }
    }
    Ok(())
}
