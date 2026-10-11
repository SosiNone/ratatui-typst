use std::fmt;

use image::{DynamicImage, Rgba, imageops::FilterType};
use ratatui::{
    buffer::Buffer,
    layout::{Rect, Size},
    widgets::Widget,
};
use ratatui_image::{
    FontSize, Image, Resize,
    picker::{Picker, ProtocolType},
    protocol::Protocol,
};

use crate::{MAX_RASTER_PIXELS, MAX_RASTER_SIDE, RenderedEquation};

#[derive(Debug)]
pub enum PrepareError {
    InvalidCellSize,
    /// Fitting and whole-cell padding would exceed the raster allocation limit.
    OutputTooLarge,
    Graphics(ratatui_image::errors::Errors),
}

impl fmt::Display for PrepareError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCellSize => {
                f.write_str("Terminal cell dimensions must be nonzero and fit in u16 cells")
            }
            Self::OutputTooLarge => f.write_str(
                "Prepared image exceeds the raster limit (16 megapixels or 16384 pixels per side)",
            ),
            Self::Graphics(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for PrepareError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Graphics(error) => Some(error),
            _ => None,
        }
    }
}

/// Encoded terminal image. Keep it between draws; recreate after area changes.
pub struct PreparedEquation {
    protocol: Option<Protocol>,
}

impl PreparedEquation {
    /// Encoded image dimensions in terminal cells, after proportional fitting.
    ///
    /// Use this to place or center the widget within the area passed to
    /// [`RenderedEquation::prepare`]. An empty preparation returns `0 x 0`.
    pub fn size(&self) -> Size {
        self.protocol
            .as_ref()
            .map_or(Size::default(), Protocol::size)
    }
}

impl RenderedEquation {
    /// Resize proportionally only when necessary, then encode outside drawing.
    /// A zero-sized area produces an empty widget without calling the encoder.
    pub fn prepare(&self, picker: &Picker, area: Size) -> Result<PreparedEquation, PrepareError> {
        if area.width == 0 || area.height == 0 {
            return Ok(PreparedEquation { protocol: None });
        }
        let font = picker.font_size();
        self.required_cells(Size::new(font.width, font.height))
            .ok_or(PrepareError::InvalidCellSize)?;
        let size = Resize::Fit(None).size_for(self.image(), font, area);
        // A caller-supplied cell size can otherwise turn a small raster into
        // an enormous whole-cell canvas. Validate before allocating padding.
        let width = u32::from(size.width) * u32::from(font.width);
        let height = u32::from(size.height) * u32::from(font.height);
        if width > MAX_RASTER_SIDE
            || height > MAX_RASTER_SIDE
            || u64::from(width) * u64::from(height) > MAX_RASTER_PIXELS
        {
            return Err(PrepareError::OutputTooLarge);
        }
        let image = prepare_pixels(
            self.image(),
            font,
            size,
            picker.protocol_type(),
            self.background,
        );
        // The image already fills whole cells: the Picker encodes it without
        // another resize (which could reintroduce partial alpha for SIXEL).
        let protocol = picker
            .new_protocol(image, size, Resize::Fit(None))
            .map_err(PrepareError::Graphics)?;
        Ok(PreparedEquation {
            protocol: Some(protocol),
        })
    }
}

fn prepare_pixels(
    source: &DynamicImage,
    font: FontSize,
    size: Size,
    protocol: ProtocolType,
    background: [u8; 3],
) -> DynamicImage {
    let width = u32::from(size.width) * u32::from(font.width);
    let height = u32::from(size.height) * u32::from(font.height);
    let fitted = if source.width() > width || source.height() > height {
        // image's filtered resize expects premultiplied alpha. Use floats to
        // retain faint glyph edges through the conversion and downsampling.
        let mut premultiplied = source.to_rgba32f();
        for pixel in premultiplied.pixels_mut() {
            let alpha = pixel[3];
            for channel in &mut pixel.0[..3] {
                *channel *= alpha;
            }
        }
        let mut resized = DynamicImage::ImageRgba32F(premultiplied)
            .resize(width, height, FilterType::Triangle)
            .to_rgba32f();
        for pixel in resized.pixels_mut() {
            let alpha = pixel[3];
            if alpha > 0.0 {
                for channel in &mut pixel.0[..3] {
                    *channel /= alpha;
                }
            }
        }
        DynamicImage::ImageRgba8(DynamicImage::ImageRgba32F(resized).to_rgba8())
    } else {
        source.clone()
    };
    // Crop only pads here; fitting happened above without enlarging small images.
    let mut pixels = Resize::Crop(None)
        .resize(&fitted, font, size, Some(Rgba([0, 0, 0, 0])))
        .to_rgba8();
    if matches!(protocol, ProtocolType::Sixel | ProtocolType::Halfblocks) {
        for pixel in pixels.pixels_mut() {
            let alpha = u32::from(pixel[3]);
            for (channel, matte) in pixel.0[..3].iter_mut().zip(background) {
                *channel = ((u32::from(*channel) * alpha + u32::from(matte) * (255 - alpha) + 127)
                    / 255) as u8;
            }
            // SIXEL has binary transparency. Undrawn pixels also carry the
            // matte RGB so palette quantization is not biased towards black.
            pixel[3] = if alpha == 0 && protocol == ProtocolType::Sixel {
                0
            } else {
                255
            };
        }
    }
    DynamicImage::ImageRgba8(pixels)
}

/// Cheap drawing adapter; performs no typesetting, resizing, or encoding.
/// If the prepared image no longer fits, reprepare it for the new area first.
pub struct Equation<'a> {
    prepared: &'a PreparedEquation,
}

impl<'a> Equation<'a> {
    pub fn new(prepared: &'a PreparedEquation) -> Self {
        Self { prepared }
    }
}

impl Widget for Equation<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        if let Some(protocol) = &self.prepared.protocol {
            Image::new(protocol).render(area, buffer);
        }
    }
}

#[cfg(test)]
mod tests {
    use image::RgbaImage;

    use super::*;

    #[test]
    fn prepared_sixel_encodes_the_final_binary_alpha_canvas_without_resizing_again() {
        let equation = crate::MathRenderer::new()
            .render(
                "x = sqrt(2)",
                &crate::RenderOptions {
                    transparent_background: true,
                    ..Default::default()
                },
            )
            .unwrap();
        let mut picker = Picker::halfblocks();
        picker.set_protocol_type(ProtocolType::Sixel);
        // An opaque Picker fill must not flatten the transparent canvas.
        picker.set_background_color(Some(Rgba([255, 0, 0, 255])));
        let area = Size::new(5, 2);
        let size = Resize::Fit(None).size_for(equation.image(), picker.font_size(), area);
        let expected_pixels = prepare_pixels(
            equation.image(),
            picker.font_size(),
            size,
            ProtocolType::Sixel,
            equation.background,
        );
        let prepared = equation.prepare(&picker, area).unwrap();
        let Some(Protocol::Sixel(encoded)) = prepared.protocol else {
            panic!("expected SIXEL")
        };
        let expected =
            ratatui_image::protocol::sixel::Sixel::new(expected_pixels, size, encoded.is_tmux)
                .unwrap();
        assert_eq!(encoded.data, expected.data);
        assert_eq!(encoded.size, size);
    }

    #[test]
    fn cell_padding_stays_transparent_for_graphics_and_opaque_for_halfblocks() {
        let mut source = RgbaImage::from_pixel(13, 11, Rgba([0, 0, 0, 0]));
        source.put_pixel(4, 4, Rgba([200, 160, 120, 64]));
        let source = DynamicImage::ImageRgba8(source);
        let font = FontSize::new(10, 20);
        let size = Size::new(2, 1);
        let background = [16, 20, 28];
        for protocol in [
            ProtocolType::Kitty,
            ProtocolType::Sixel,
            ProtocolType::Halfblocks,
        ] {
            let image = prepare_pixels(&source, font, size, protocol, background).to_rgba8();
            assert_eq!(image.dimensions(), (20, 20));
            let edge = image.get_pixel(4, 4);
            let padding = image.get_pixel(19, 19);
            match protocol {
                ProtocolType::Kitty => {
                    assert_eq!(edge.0, [200, 160, 120, 64]);
                    assert_eq!(padding[3], 0);
                }
                ProtocolType::Sixel => {
                    assert_eq!(edge.0, [62, 55, 51, 255]);
                    assert_eq!(padding.0, [16, 20, 28, 0]);
                }
                ProtocolType::Halfblocks => {
                    assert_eq!(edge.0, [62, 55, 51, 255]);
                    assert_eq!(padding.0, [16, 20, 28, 255]);
                    assert!(image.pixels().all(|pixel| pixel[3] == 255));
                }
                _ => unreachable!(),
            }
        }
    }

    #[test]
    fn sixel_edges_are_composited_after_downsampling() {
        let mut source = RgbaImage::from_pixel(83, 83, Rgba([0, 0, 0, 0]));
        for y in 21..62 {
            for x in 21..62 {
                source.put_pixel(x, y, Rgba([240, 240, 240, 255]));
            }
        }
        let source = DynamicImage::ImageRgba8(source);
        let font = FontSize::new(10, 20);
        let size = Size::new(2, 1);
        let background = [16, 20, 28];
        let kitty = prepare_pixels(&source, font, size, ProtocolType::Kitty, background).to_rgba8();
        let sixel = prepare_pixels(&source, font, size, ProtocolType::Sixel, background).to_rgba8();
        assert!(kitty.pixels().any(|p| p[3] > 0 && p[3] < 255));
        // Transparent surroundings must not darken a white shape's edges.
        assert!(kitty.pixels().filter(|p| p[3] > 0).all(|p| p[0] >= 239));
        for (original, prepared) in kitty.pixels().zip(sixel.pixels()) {
            let alpha = u32::from(original[3]);
            assert_eq!(prepared[3], if alpha == 0 { 0 } else { 255 });
            for channel in 0..3 {
                assert_eq!(
                    u32::from(prepared[channel]),
                    (u32::from(original[channel]) * alpha
                        + u32::from(background[channel]) * (255 - alpha)
                        + 127)
                        / 255
                );
            }
        }
    }
}
