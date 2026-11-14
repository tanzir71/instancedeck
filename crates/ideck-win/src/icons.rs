//! Rasterize InstanceDeck's own neutral deck mark. Never extract vendor icons.
use ideck_core::{Error, ProfileColor, Result};
use image::{Rgba, RgbaImage};

fn token_rgb(group: &str, name: &str) -> Result<[u8; 3]> {
    let tokens: serde_json::Value =
        serde_json::from_str(include_str!("../../../design/tokens.json"))?;
    let hex = tokens["color"][group][name]["value"]
        .as_str()
        .ok_or_else(|| Error::Invalid("Missing icon color token".into()))?;
    let hex = hex
        .strip_prefix('#')
        .ok_or_else(|| Error::Invalid("Invalid icon color token".into()))?;
    let read = |offset| {
        u8::from_str_radix(&hex[offset..offset + 2], 16)
            .map_err(|_| Error::Invalid("Invalid icon color channel".into()))
    };
    Ok([read(0)?, read(2)?, read(4)?])
}
pub fn color_rgb(color: ProfileColor) -> Result<[u8; 3]> {
    let value = serde_json::to_value(color)?;
    token_rgb("profile", value.as_str().expect("color is a string"))
}
pub fn profile_mark(color: ProfileColor, size: u32) -> Result<RgbaImage> {
    if !(16..=256).contains(&size) {
        return Err(Error::Invalid("Profile icons support 16–256 pixels".into()));
    }
    let rgb = color_rgb(color)?;
    let ink = token_rgb("ui", "ink")?;
    let paper = token_rgb("ui", "paper")?;
    let mut image = RgbaImage::from_pixel(size, size, Rgba([0, 0, 0, 0]));
    let mut rect = |x: u32, y: u32, w: u32, h: u32, fill: [u8; 3]| {
        for py in (y * size / 32)..((y + h) * size / 32) {
            for px in (x * size / 32)..((x + w) * size / 32) {
                image.put_pixel(px, py, Rgba([fill[0], fill[1], fill[2], 255]));
            }
        }
    };
    rect(4, 10, 20, 20, ink);
    rect(10, 4, 20, 20, rgb);
    // A neutral I monogram in the foreground card; all shapes are flat fills.
    rect(16, 8, 8, 3, paper);
    rect(19, 11, 2, 9, paper);
    rect(16, 20, 8, 3, paper);
    Ok(image)
}
pub fn encode_profile_icon(color: ProfileColor) -> Result<Vec<u8>> {
    let mut directory = ico::IconDir::new(ico::ResourceType::Icon);
    for size in [16, 24, 32, 48, 64, 128, 256] {
        let image = profile_mark(color, size)?;
        let icon = ico::IconImage::from_rgba_data(size, size, image.into_raw());
        directory.add_entry(
            ico::IconDirEntry::encode_as_png(&icon).map_err(|e| Error::Platform(e.to_string()))?,
        );
    }
    let mut bytes = Vec::new();
    directory
        .write(&mut bytes)
        .map_err(|e| Error::Platform(e.to_string()))?;
    Ok(bytes)
}
