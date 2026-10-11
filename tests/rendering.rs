use image::{GenericImageView, Rgba};
use ratatui::{
    buffer::Buffer,
    layout::{Rect, Size},
    widgets::Widget,
};
use ratatui_image::{FontSize, picker::Picker};
use ratatui_typst::{Equation, MathRenderer, PrepareError, RenderError, RenderOptions};

#[test]
fn transparent_pages_keep_clear_corners_and_antialiased_glyphs() {
    let renderer = MathRenderer::new();
    let options = RenderOptions {
        transparent_background: true,
        background: [245, 242, 232],
        foreground: [30, 38, 50],
        ..Default::default()
    };
    let equation = renderer.render("x = sqrt(2)", &options).unwrap();
    let pixels = equation.image().to_rgba8();
    for (x, y) in [(0, 0), (pixels.width() - 1, pixels.height() - 1)] {
        assert_eq!(pixels.get_pixel(x, y)[3], 0);
    }
    assert!(pixels.pixels().any(|p| p[3] == 255 && p[0] < 40));
    assert!(pixels.pixels().any(|p| p[3] > 0 && p[3] < 255));
    let opaque = renderer
        .render("x = sqrt(2)", &RenderOptions::default())
        .unwrap();
    assert!(opaque.image().to_rgba8().pixels().all(|p| p[3] == 255));
    assert_eq!(opaque.image().get_pixel(0, 0), Rgba([16, 20, 28, 255]));
}

#[test]
fn renders_representative_math_and_reuses_the_renderer() {
    let renderer = MathRenderer::new();
    let options = RenderOptions::default();
    for source in [
        "x = (-b plus.minus sqrt(b^2 - 4 a c)) / (2 a)",
        "integral_0^infinity e^(-x^2) dif x = sqrt(pi) / 2",
        "sum_(k=1)^n k = n(n+1) / 2",
        "A = mat(1, 2; 3, 4)",
        "a &= b + c \\ &= d",
    ] {
        let equation = renderer.render(source, &options).unwrap();
        assert_eq!(equation.source(), source);
        assert!(equation.warnings().is_empty(), "{:?}", equation.warnings());
        let image = equation.image();
        assert!(image.width() > 40 && image.height() > 40);
        assert!(equation.natural_size_pt().iter().all(|n| *n > 0.0));
        assert_eq!(image.get_pixel(0, 0), Rgba([16, 20, 28, 255]));
        assert!(image.pixels().any(|(_, _, pixel)| pixel.0[0] > 150));
        assert!(image.pixels().all(|(_, _, pixel)| pixel.0[3] == 255));
        assert_eq!(
            equation.required_cells(Size::new(9, 17)),
            Some(Size::new(
                image.width().div_ceil(9) as u16,
                image.height().div_ceil(17) as u16,
            ))
        );
        assert_eq!(equation.required_cells(Size::new(0, 17)), None);
    }
    let first = renderer.render("x", &options).unwrap();
    let other = renderer.render("x + y + z", &options).unwrap();
    let again = renderer.render("x", &options).unwrap();
    assert!(other.image().width() > first.image().width());
    assert_eq!(first.image().as_bytes(), again.image().as_bytes());
}

#[test]
fn diagnostics_use_expression_locations_and_preserve_warnings() {
    let renderer = MathRenderer::new();
    let RenderError::Compilation { diagnostics, .. } = renderer
        .render("α + missingname", &RenderOptions::default())
        .err()
        .unwrap()
    else {
        panic!("expected compilation failure")
    };
    assert!(
        diagnostics
            .iter()
            .any(|d| d.location == Some((1, 5)) && d.range == Some(5..16))
    );
    let RenderError::Compilation { diagnostics, .. } = renderer
        .render("α +\nmissingname", &RenderOptions::default())
        .err()
        .unwrap()
    else {
        panic!("expected compilation failure")
    };
    assert!(diagnostics.iter().any(|d| d.location == Some((2, 1))));
    let warned = renderer
        .render(
            "#text(font: \"nonexistent-font-for-test\")[x]",
            &RenderOptions::default(),
        )
        .unwrap();
    assert!(!warned.warnings().is_empty());
}

#[test]
fn controls_theme_scale_and_rejects_invalid_or_excessive_output() {
    let renderer = MathRenderer::new();
    let options = RenderOptions {
        background: [200, 40, 20],
        foreground: [10, 20, 220],
        ..Default::default()
    };
    let equation = renderer.render("x", &options).unwrap();
    assert_eq!(equation.image().get_pixel(0, 0), Rgba([200, 40, 20, 255]));
    assert!(
        equation
            .image()
            .pixels()
            .any(|(_, _, p)| p.0[2] > 200 && p.0[0] < 30)
    );
    let larger = renderer
        .render(
            "x",
            &RenderOptions {
                pixels_per_pt: 4.0,
                ..options.clone()
            },
        )
        .unwrap();
    assert!(
        larger
            .image()
            .width()
            .abs_diff(equation.image().width() * 2)
            <= 1
    );
    assert!(renderer.render(" ", &options).is_err());
    for scale in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert!(
            renderer
                .render(
                    "x",
                    &RenderOptions {
                        pixels_per_pt: scale,
                        ..options.clone()
                    }
                )
                .is_err()
        );
    }
    assert!(
        renderer
            .render(
                "x",
                &RenderOptions {
                    pixels_per_pt: 1e12,
                    ..options
                }
            )
            .is_err()
    );
    assert!(
        renderer
            .render("#read(\"Cargo.toml\")", &RenderOptions::default())
            .is_err()
    );
}

#[test]
fn prepared_widget_composes_with_a_buffer_and_handles_empty_areas() {
    let renderer = MathRenderer::new();
    let equation = renderer
        .render("x = sqrt(2)", &RenderOptions::default())
        .unwrap();
    let picker = Picker::halfblocks();
    let area = Rect::new(2, 2, 24, 8);
    let prepared = equation.prepare(&picker, area.as_size()).unwrap();
    assert!(prepared.size().width <= area.width && prepared.size().height <= area.height);
    assert!(prepared.size().width > 0 && prepared.size().height > 0);
    let mut buffer = Buffer::empty(Rect::new(0, 0, 30, 12));
    buffer[(0, 0)].set_symbol("!");
    Equation::new(&prepared).render(area, &mut buffer);
    assert_eq!(buffer[(0, 0)].symbol(), "!");
    assert!(buffer.content.iter().any(|cell| cell.symbol() == "▀"));
    for size in [Size::new(0, 0), Size::new(0, 10), Size::new(10, 0)] {
        let empty = equation.prepare(&picker, size).unwrap();
        assert_eq!(empty.size(), Size::new(0, 0));
        let before = buffer.clone();
        Equation::new(&empty).render(area, &mut buffer);
        assert_eq!(buffer, before);
    }
    // Preparation for a tiny area should resize safely rather than divide by zero.
    let tiny = equation.prepare(&picker, Size::new(1, 1)).unwrap();
    assert_eq!(tiny.size(), Size::new(1, 1));
    Equation::new(&tiny).render(Rect::new(2, 2, 1, 1), &mut buffer);
}

#[test]
// The upstream deprecated constructor is its only public way to inject bad
// cell dimensions. Exercise it to ensure legacy consumers cannot overallocate.
#[allow(deprecated)]
fn preparation_rejects_invalid_or_excessive_cell_canvases_before_allocation() {
    let equation = MathRenderer::new()
        .render("x", &RenderOptions::default())
        .unwrap();
    for font in [FontSize::new(0, 20), FontSize::new(10, 0)] {
        assert!(matches!(
            equation.prepare(&Picker::from_fontsize(font), Size::new(10, 10)),
            Err(PrepareError::InvalidCellSize)
        ));
    }
    for font in [FontSize::new(u16::MAX, u16::MAX), FontSize::new(4096, 4096)] {
        assert!(matches!(
            equation.prepare(&Picker::from_fontsize(font), Size::new(1, 1)),
            Err(PrepareError::OutputTooLarge)
        ));
    }
}
