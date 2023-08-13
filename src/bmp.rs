use crate::model::{PixelPosition, PixelRegion};
use pagurus::{failure::OrFail, image::Color};
use std::{collections::BTreeMap, io::Write};

pub fn write_image<W: Write>(
    mut writer: W,
    region: PixelRegion,
    pixels: impl Iterator<Item = (PixelPosition, Color)>,
) -> pagurus::Result<()> {
    let image_data_offset: u32 = 54;
    let file_size: u32 = image_data_offset + region.size.area() as u32 * 4;

    // File header.
    writer.write_all(b"BM").or_fail()?;
    writer.write_all(&file_size.to_le_bytes()).or_fail()?;
    writer.write_all(&[0, 0, 0, 0]).or_fail()?;
    writer
        .write_all(&image_data_offset.to_le_bytes())
        .or_fail()?;

    // Information header.
    writer.write_all(&[40, 0, 0, 0]).or_fail()?; // Header size.
    writer
        .write_all(&(region.size.width as i32).to_le_bytes())
        .or_fail()?;
    writer
        .write_all(&(region.size.height as i32).to_le_bytes())
        .or_fail()?;
    writer.write_all(&[1, 0]).or_fail()?; // Planes.
    writer.write_all(&[32, 0]).or_fail()?; // Bits per pixel.
    writer.write_all(&[0, 0, 0, 0]).or_fail()?; // No compression.
    writer
        .write_all(&(region.size.area() as u32 * 4).to_le_bytes())
        .or_fail()?; // Image size.
    writer.write_all(&[0, 0, 0, 0]).or_fail()?; // Horizontal resolution.
    writer.write_all(&[0, 0, 0, 0]).or_fail()?; // Vertical resolution.
    writer.write_all(&[0, 0, 0, 0]).or_fail()?; // Colors in palette.
    writer.write_all(&[0, 0, 0, 0]).or_fail()?; // Important colors.

    // Image data.
    let pixels = pixels.collect::<BTreeMap<_, _>>();
    for position in region.positions() {
        let color = pixels
            .get(&position)
            .copied()
            .unwrap_or(Color::rgba(255, 255, 255, 0));
        let c = color.to_rgba();
        writer.write_all(&[c.b, c.g, c.r, c.a]).or_fail()?;
    }
    Ok(())
}
