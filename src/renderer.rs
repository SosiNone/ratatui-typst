use std::{fmt, ops::Range, sync::LazyLock};

use image::{DynamicImage, RgbaImage};
use ratatui::layout::Size;
use typst::{
    Library, LibraryExt, World, WorldExt,
    diag::{FileError, FileResult, SourceDiagnostic},
    foundations::{Bytes, Datetime, Duration},
    syntax::{FileId, Source},
    text::{Font, FontBook},
    utils::{LazyHash, Scalar},
};
use typst_layout::PagedDocument;

use crate::{MAX_RASTER_PIXELS, MAX_RASTER_SIDE};

/// Styling and raster resolution. Colors are sRGB.
#[derive(Debug, Clone)]
pub struct RenderOptions {
    /// Font size in points. Must be finite and positive; defaults to 20.
    pub text_size_pt: f64,
    /// Page padding in points. Must be finite and nonnegative; defaults to 10.
    pub padding_pt: f64,
    /// Raster pixels per point. Must be finite and positive; defaults to 2.
    /// This also affects the natural size in terminal cells, not just sharpness.
    pub pixels_per_pt: f64,
    /// Text color as 8-bit sRGB channels.
    pub foreground: [u8; 3],
    /// Page fill, or the compositing matte when transparency is enabled.
    pub background: [u8; 3],
    /// Render without page fill. Background remains the compositing color for
    /// SIXEL antialiasing and the opaque half-block fallback.
    pub transparent_background: bool,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            text_size_pt: 20.0,
            padding_pt: 10.0,
            pixels_per_pt: 2.0,
            foreground: [227, 234, 244],
            background: [16, 20, 28],
            transparent_background: false,
        }
    }
}

/// A compiler diagnostic. Locations refer to the original expression.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub message: String,
    /// UTF-8 byte range; absent for errors in generated setup or other sources.
    pub range: Option<Range<usize>>,
    /// One-based line and Unicode scalar column, when a location is available.
    pub location: Option<(usize, usize)>,
    pub hints: Vec<String>,
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some((line, column)) = self.location {
            write!(f, "{line}:{column}: ")?;
        }
        write!(f, "{}", self.message)?;
        for hint in &self.hints {
            write!(f, "\n  hint: {hint}")?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum RenderError {
    InvalidInput(&'static str),
    Compilation {
        diagnostics: Vec<Diagnostic>,
        warnings: Vec<Diagnostic>,
    },
    Output(&'static str),
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput(message) | Self::Output(message) => f.write_str(message),
            Self::Compilation {
                diagnostics,
                warnings,
            } => {
                f.write_str("Typst compilation failed")?;
                for diagnostic in diagnostics {
                    write!(f, "\n{diagnostic}")?;
                }
                for warning in warnings {
                    write!(f, "\nwarning: {warning}")?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for RenderError {}

struct Fonts {
    fonts: Vec<Font>,
    book: LazyHash<FontBook>,
}

static FONTS: LazyLock<Fonts> = LazyLock::new(|| {
    let fonts: Vec<_> = typst_assets::fonts()
        .flat_map(|bytes| Font::iter(Bytes::new(bytes)))
        .collect();
    let book = LazyHash::new(FontBook::from_fonts(&fonts));
    Fonts { fonts, book }
});

/// Reusable synchronous compiler with bundled fonts and no system-font lookup.
///
/// File imports, package downloads, and the current date are unavailable.
/// Typst evaluation can still consume substantial CPU/memory: this is not a
/// resource sandbox for hostile expressions. Use a separate process for those.
pub struct MathRenderer {
    library: LazyHash<Library>,
    fonts: &'static Fonts,
}

impl Default for MathRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl MathRenderer {
    pub fn new() -> Self {
        Self {
            library: LazyHash::new(Library::default()),
            fonts: &FONTS,
        }
    }

    /// Typeset math without surrounding `$` delimiters and rasterize one page.
    /// Call outside the draw callback, optionally on an application-owned worker.
    pub fn render(
        &self,
        expression: &str,
        options: &RenderOptions,
    ) -> Result<RenderedEquation, RenderError> {
        if expression.trim().is_empty() {
            return Err(RenderError::InvalidInput("Equation must not be empty"));
        }
        if !options.text_size_pt.is_finite()
            || options.text_size_pt <= 0.0
            || !options.padding_pt.is_finite()
            || options.padding_pt < 0.0
            || !options.pixels_per_pt.is_finite()
            || options.pixels_per_pt <= 0.0
        {
            return Err(RenderError::InvalidInput(
                "Text size and raster scale must be finite and positive; padding must be finite and nonnegative",
            ));
        }
        let [br, bg, bb] = options.background;
        let [fr, fg, fb] = options.foreground;
        let fill = if options.transparent_background {
            "none".to_owned()
        } else {
            format!("rgb(\"{br:02x}{bg:02x}{bb:02x}\")")
        };
        let prefix = format!(
            "#set page(width: auto, height: auto, margin: {}pt, fill: {fill})\n\
             #set text(size: {}pt, fill: rgb(\"{fr:02x}{fg:02x}{fb:02x}\"))\n$ ",
            options.padding_pt, options.text_size_pt,
        );
        let world = EquationWorld {
            renderer: self,
            source: Source::detached(format!("{prefix}{expression} $\n")),
        };
        let map = |diagnostic: &SourceDiagnostic| {
            map_diagnostic(diagnostic, &world, expression, prefix.len())
        };
        let compiled = typst::compile::<PagedDocument>(&world);
        // Typst's process-wide memoization caches retain recent compilations.
        // Age them on failures too; this is not a byte-bounded equation cache.
        typst::comemo::evict(5);
        let warnings = compiled.warnings.iter().map(map).collect();
        let document = compiled.output.map_err(|errors| RenderError::Compilation {
            diagnostics: errors.iter().map(map).collect(),
            warnings: compiled.warnings.iter().map(map).collect(),
        })?;
        let [page] = document.pages() else {
            return Err(RenderError::Output("Expected exactly one equation page"));
        };
        let natural_size_pt = [page.frame.size().x.to_pt(), page.frame.size().y.to_pt()];
        // Conservatively round up before checking allocation limits.
        // Typst's rasterizer rounds to the nearest pixel using f32 dimensions.
        let dimensions = natural_size_pt.map(|pt| (pt * options.pixels_per_pt).ceil().max(1.0));
        if dimensions
            .iter()
            .any(|n| !n.is_finite() || *n > f64::from(MAX_RASTER_SIDE))
            || dimensions[0] * dimensions[1] > MAX_RASTER_PIXELS as f64
        {
            return Err(RenderError::Output(
                "Equation exceeds the raster limit (16 megapixels or 16384 pixels per side)",
            ));
        }
        let pixmap = typst_render::render(
            page,
            &typst_render::RenderOptions {
                pixel_per_pt: Scalar::new(options.pixels_per_pt),
                render_bleed: false,
            },
        );
        // tiny-skia uses premultiplied RGBA; image uses straight RGBA.
        // Demultiplication also keeps conversion correct if input overrides fill.
        let pixels: Vec<_> = pixmap
            .pixels()
            .iter()
            .flat_map(|pixel| {
                let straight = pixel.demultiply();
                [
                    straight.red(),
                    straight.green(),
                    straight.blue(),
                    straight.alpha(),
                ]
            })
            .collect();
        let image = RgbaImage::from_raw(pixmap.width(), pixmap.height(), pixels)
            .ok_or(RenderError::Output("Invalid raster dimensions"))?;
        Ok(RenderedEquation {
            source: expression.to_owned(),
            image: DynamicImage::ImageRgba8(image),
            background: options.background,
            natural_size_pt,
            warnings,
        })
    }
}

/// A successful render, independent of terminal protocol and cell size.
pub struct RenderedEquation {
    source: String,
    image: DynamicImage,
    pub(crate) background: [u8; 3],
    natural_size_pt: [f64; 2],
    warnings: Vec<Diagnostic>,
}

impl RenderedEquation {
    /// Original math expression, without generated Typst setup.
    pub fn source(&self) -> &str {
        &self.source
    }
    /// Natural raster with straight (unpremultiplied) RGBA alpha.
    /// Terminal fitting and protocol-specific compositing do not modify it.
    pub fn image(&self) -> &DynamicImage {
        &self.image
    }
    /// Natural page width and height in points, including configured padding.
    pub fn natural_size_pt(&self) -> [f64; 2] {
        self.natural_size_pt
    }
    /// Nonfatal compiler diagnostics retained alongside the rendered image.
    pub fn warnings(&self) -> &[Diagnostic] {
        &self.warnings
    }

    /// Required cells, rounded up. None means zero cell dimensions or overflow.
    pub fn required_cells(&self, cell_pixels: Size) -> Option<Size> {
        if cell_pixels.width == 0 || cell_pixels.height == 0 {
            return None;
        }
        Some(Size::new(
            self.image
                .width()
                .div_ceil(u32::from(cell_pixels.width))
                .try_into()
                .ok()?,
            self.image
                .height()
                .div_ceil(u32::from(cell_pixels.height))
                .try_into()
                .ok()?,
        ))
    }
}

struct EquationWorld<'a> {
    renderer: &'a MathRenderer,
    source: Source,
}

impl World for EquationWorld<'_> {
    fn library(&self) -> &LazyHash<Library> {
        &self.renderer.library
    }
    fn book(&self) -> &LazyHash<FontBook> {
        &self.renderer.fonts.book
    }
    fn main(&self) -> FileId {
        self.source.id()
    }
    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.main() {
            Ok(self.source.clone())
        } else {
            Err(FileError::AccessDenied)
        }
    }
    fn file(&self, id: FileId) -> FileResult<Bytes> {
        self.source(id)
            .map(|source| Bytes::from_string(source.text().to_owned()))
    }
    fn font(&self, index: usize) -> Option<Font> {
        self.renderer.fonts.fonts.get(index).cloned()
    }
    fn today(&self, _: Option<Duration>) -> Option<Datetime> {
        None
    }
}

fn map_diagnostic(
    diagnostic: &SourceDiagnostic,
    world: &EquationWorld<'_>,
    expression: &str,
    prefix_len: usize,
) -> Diagnostic {
    let range = world.range(diagnostic.span).and_then(|range| {
        let start = range.start.checked_sub(prefix_len)?;
        let end = range.end.checked_sub(prefix_len)?;
        (end <= expression.len() && start <= end).then_some(start..end)
    });
    let location = range.as_ref().and_then(|range| {
        let before = expression.get(..range.start)?;
        Some((
            before.bytes().filter(|&byte| byte == b'\n').count() + 1,
            before
                .rsplit('\n')
                .next()
                .unwrap_or_default()
                .chars()
                .count()
                + 1,
        ))
    });
    Diagnostic {
        message: diagnostic.message.to_string(),
        range,
        location,
        hints: diagnostic
            .hints
            .iter()
            .map(|hint| hint.v.to_string())
            .collect(),
    }
}
