use ratatui::{
    Terminal,
    backend::TestBackend,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    layout::Rect,
};
use ratatui_image::picker::Picker;
use ratatui_typst::MathRenderer;

use crate::{
    app::{App, Focus},
    editor::Editor,
    examples::EXAMPLES,
    preview::{self, PreviewContext},
    ui,
};

fn app(source: &str) -> App {
    App::new(
        MathRenderer::new(),
        Picker::halfblocks(),
        source,
        "Ready".to_owned(),
    )
}

fn key(app: &mut App, code: KeyCode) -> bool {
    app.handle_key(KeyEvent::new(code, KeyModifiers::NONE))
}

#[test]
fn gallery_renders_all_examples_and_recovers_from_the_error_example() {
    let mut app = app(EXAMPLES[0].source);
    for example in EXAMPLES {
        assert_eq!(app.source, example.source);
        assert_eq!(
            app.rendered.is_some(),
            example.valid,
            "{}: {:?}",
            example.name,
            app.error
        );
        if example.name == "Font warning" {
            assert!(app.diagnostics().contains("Warning:"));
        }
        if !example.valid {
            assert!(app.diagnostics().contains("2:1:"));
            assert!(app.prepared.is_none());
        }
        assert!(!key(&mut app, KeyCode::Char('n')));
    }
    assert!(app.error.is_none());
    assert_eq!(app.source, EXAMPLES[0].source);
}

#[test]
fn editor_handles_unicode_and_multiline_movement() {
    let mut editor = Editor::new("α + β\nγ");
    editor.handle(KeyCode::Up);
    assert_eq!(editor.cursor, "α".len());
    editor.handle(KeyCode::Delete);
    assert_eq!(editor.text, "α+ β\nγ");
    editor.handle(KeyCode::Home);
    editor.handle(KeyCode::Right);
    editor.handle(KeyCode::Backspace);
    assert_eq!(editor.text, "+ β\nγ");
    editor.handle(KeyCode::Down);
    editor.handle(KeyCode::End);
    editor.handle(KeyCode::Char('界'));
    assert_eq!(editor.position(), (1, 3));
    editor.handle(KeyCode::Left);
    editor.handle(KeyCode::Delete);
    assert_eq!(editor.text, "+ β\nγ");
    assert!(editor.text.is_char_boundary(editor.cursor));
}

#[test]
fn editing_applies_errors_cancels_drafts_and_keeps_quit_keys_as_text() {
    let mut app = app("x");
    key(&mut app, KeyCode::Char('e'));
    assert!(!key(&mut app, KeyCode::Char('q')));
    assert_eq!(app.editor.text, "xq");
    key(&mut app, KeyCode::Esc);
    assert_eq!(app.source, "x");
    key(&mut app, KeyCode::Char('e'));
    app.editor = Editor::new("α + missingname");
    key(&mut app, KeyCode::Enter);
    assert!(app.error.is_some());
    assert!(app.rendered.is_none());
    assert!(app.focus == Focus::Gallery);
    key(&mut app, KeyCode::Char('e'));
    app.editor = Editor::new("α + beta");
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::ALT));
    app.editor.insert("+ gamma");
    key(&mut app, KeyCode::Tab);
    assert!(app.focus == Focus::Settings);
    assert!(app.error.is_none());
    assert!(app.rendered.is_some());
    assert!(app.source.contains('\n'));
    assert!(app.handle_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)));
}

#[test]
fn rendering_controls_change_pixels_and_reset_without_losing_source() {
    let mut app = app("x^2");
    let original_width = app.rendered.as_ref().unwrap().image().width();
    key(&mut app, KeyCode::Tab);
    key(&mut app, KeyCode::Right);
    assert!(app.rendered.as_ref().unwrap().image().width() > original_width);
    key(&mut app, KeyCode::Down); // padding
    key(&mut app, KeyCode::Left);
    assert_eq!(app.options.padding_pt, 0.0);
    key(&mut app, KeyCode::Down); // raster scale
    key(&mut app, KeyCode::Right);
    assert_eq!(app.options.pixels_per_pt, 2.5);
    key(&mut app, KeyCode::Down); // palette
    key(&mut app, KeyCode::Right);
    assert_eq!(app.palette, 1);
    let background = app
        .rendered
        .as_ref()
        .unwrap()
        .image()
        .to_rgba8()
        .get_pixel(0, 0)
        .0;
    assert_eq!(background[3], 0);
    key(&mut app, KeyCode::Char('r'));
    assert_eq!(app.options.text_size_pt, 12.0);
    assert_eq!(app.options.padding_pt, 2.0);
    assert!(app.options.transparent_background);
    assert_eq!(app.options.pixels_per_pt, 2.0);
    assert_eq!(
        app.rendered.as_ref().unwrap().image().width(),
        original_width
    );
    assert_eq!(app.source, "x^2");
}

#[test]
fn tui_draws_preview_diagnostics_editor_and_handles_resize() {
    let mut app = app(EXAMPLES[0].source);
    let mut terminal = Terminal::new(TestBackend::new(100, 36)).unwrap();
    for (width, height) in [(100, 36), (76, 24), (80, 24), (20, 6), (0, 0), (120, 40)] {
        terminal.backend_mut().resize(width, height);
        terminal.resize(Rect::new(0, 0, width, height)).unwrap();
        app.prepare(ui::areas(Rect::new(0, 0, width, height)).equation(&app));
        terminal.draw(|frame| ui::draw(frame, &mut app)).unwrap();
        if width >= 76 {
            let buffer = terminal.backend().buffer();
            let text = buffer
                .content
                .iter()
                .map(|cell| cell.symbol())
                .collect::<String>();
            assert!(text.contains("Examples"));
            assert!(text.contains("Typst source"));
            assert!(text.contains("Halfblocks"));
            assert!(buffer.content.iter().any(|cell| cell.symbol() == "▀"));
        }
    }
    key(&mut app, KeyCode::Char('p')); // intentional compiler error
    app.prepare(ui::areas(Rect::new(0, 0, 120, 40)).equation(&app));
    terminal.draw(|frame| ui::draw(frame, &mut app)).unwrap();
    let text = terminal
        .backend()
        .buffer()
        .content
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>();
    assert!(text.contains("Preview unavailable"));
    assert!(text.contains("2:1:"));
    assert!(
        !terminal
            .backend()
            .buffer()
            .content
            .iter()
            .any(|cell| cell.symbol() == "▀")
    );
    key(&mut app, KeyCode::Char('e'));
    app.editor = Editor::new(&format!("{}界", "x".repeat(200)));
    terminal.draw(|frame| ui::draw(frame, &mut app)).unwrap();
    key(&mut app, KeyCode::Esc);
    assert!(key(&mut app, KeyCode::Char('q')));
}

#[test]
fn contextual_previews_keep_native_text_clear_of_the_image_and_match_its_palette() {
    let mut app = app(EXAMPLES[0].source);
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    key(&mut app, KeyCode::Tab);
    for _ in 0..3 {
        key(&mut app, KeyCode::Down);
    }
    key(&mut app, KeyCode::Right); // paper palette
    for expected in [
        PreviewContext::Note,
        PreviewContext::Console,
        PreviewContext::Plain,
    ] {
        assert!(app.context == expected);
        let a = ui::areas(Rect::new(0, 0, 80, 24));
        let image_area = a.equation(&app);
        app.prepare(image_area);
        terminal.draw(|frame| ui::draw(frame, &mut app)).unwrap();
        let buffer = terminal.backend().buffer();
        let preview_text = (a.preview.inner.y..a.preview.inner.bottom())
            .map(|y| {
                (a.preview.inner.x..a.preview.inner.right())
                    .map(|x| buffer[(x, y)].symbol())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n");
        match expected {
            PreviewContext::Note => {
                assert!(preview_text.contains(EXAMPLES[0].introduction));
                assert!(preview_text.contains(EXAMPLES[0].followup));
            }
            PreviewContext::Console => {
                assert!(preview_text.contains("math> Quadratic formula"));
                assert!(preview_text.contains("Result:"));
                assert!(preview_text.contains("math> _"));
            }
            PreviewContext::Plain => {
                assert!(!preview_text.contains(EXAMPLES[0].introduction));
                assert!(!preview_text.contains("math>"));
                assert_eq!(image_area, a.preview.inner);
            }
        }
        let mut image_cells = 0;
        for y in a.preview.inner.y..a.preview.inner.bottom() {
            for x in a.preview.inner.x..a.preview.inner.right() {
                let cell = &buffer[(x, y)];
                if cell.symbol() == "▀" {
                    image_cells += 1;
                    assert!(image_area.contains((x, y).into()));
                }
                if !image_area.contains((x, y).into()) {
                    assert_eq!(cell.bg, ratatui::style::Color::Rgb(245, 242, 232));
                    assert_eq!(cell.fg, ratatui::style::Color::Rgb(30, 38, 50));
                }
            }
        }
        assert!(image_cells > 0);
        key(&mut app, KeyCode::Char('c'));
    }
    assert!(app.context == PreviewContext::Note);
    let tiny = preview::areas(Rect::new(0, 0, 0, 0), &app);
    assert_eq!(tiny.equation.width, 0);
    assert_eq!(tiny.equation.height, 0);
}

#[test]
fn compact_defaults_follow_cell_height_and_clamp_to_the_controls_range() {
    for (height, expected) in [(0, 8.0), (10, 8.0), (20, 12.0), (30, 18.0), (100, 48.0)] {
        let options = crate::app::demo_options(height);
        assert_eq!(options.text_size_pt, expected);
        assert_eq!(options.padding_pt, 2.0);
        assert!(options.transparent_background);
    }
}

#[test]
fn following_text_tracks_fitted_equations_across_sizes_and_errors() {
    for source in [
        "x",
        EXAMPLES[0].source,
        EXAMPLES[3].source,
        EXAMPLES[8].source,
    ] {
        let mut app = app(source);
        for context in [PreviewContext::Note, PreviewContext::Console] {
            app.context = context;
            for (width, height) in [(100, 30), (60, 20), (30, 12), (12, 10), (3, 3), (0, 0)] {
                let area = Rect::new(2, 3, width, height);
                let a = preview::areas(area, &app);
                assert!(a.before.bottom() <= a.equation.y);
                assert!(a.equation.bottom() <= a.after.y);
                assert!(a.after.bottom() <= area.bottom());
                if width >= 30 && height >= 12 {
                    assert!(a.equation.height > 0);
                    assert_eq!(a.after.y, a.equation.bottom() + 1);
                    assert!(a.after.height > 0);
                    assert!(a.after.bottom() < area.bottom());
                }
                if width == 100 && height == 30 {
                    if let Some(rendered) = &app.rendered {
                        assert_eq!(
                            a.equation.height,
                            rendered
                                .required_cells(ratatui::layout::Size::new(10, 20))
                                .unwrap()
                                .height
                        );
                    }
                    let mut terminal = Terminal::new(TestBackend::new(104, 36)).unwrap();
                    app.prepare(a.equation);
                    terminal
                        .draw(|frame| preview::draw(frame, &app, area))
                        .unwrap();
                    let buffer = terminal.backend().buffer();
                    let followup = if context == PreviewContext::Console {
                        "math> _"
                    } else {
                        app.gallery
                            .selected()
                            .map_or("Continue reading", |index| EXAMPLES[index].followup)
                    };
                    assert_eq!(
                        buffer[(a.after.x, a.after.y)].symbol(),
                        followup.chars().next().unwrap().to_string()
                    );
                    for x in a.equation.x..a.equation.right() {
                        assert_eq!(buffer[(x, a.equation.bottom())].symbol(), " ");
                    }
                }
            }
        }
    }
}
