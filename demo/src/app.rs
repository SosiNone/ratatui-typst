use std::{
    path::Path,
    time::{Duration, Instant},
};

use ratatui::{
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    layout::{Rect, Size},
    widgets::ListState,
};
use ratatui_image::picker::{Picker, ProtocolType};
use ratatui_typst::{MathRenderer, PreparedEquation, RenderOptions, RenderedEquation};

use crate::{editor::Editor, examples::EXAMPLES, export_png, preview::PreviewContext};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Gallery,
    Settings,
    Editor,
}

pub const PALETTES: [(&str, [u8; 3], [u8; 3]); 3] = [
    ("Night", [227, 234, 244], [16, 20, 28]),
    ("Paper", [30, 38, 50], [245, 242, 232]),
    ("Teal", [115, 245, 213], [9, 35, 39]),
];

/// Compact presentation matched to the terminal; headless exports use 20 px.
pub fn demo_options(cell_height_pixels: u16) -> RenderOptions {
    let mut options = RenderOptions {
        transparent_background: true,
        padding_pt: 2.0,
        ..Default::default()
    };
    options.text_size_pt =
        (1.2 * f64::from(cell_height_pixels) / options.pixels_per_pt).clamp(8.0, 48.0);
    options
}

pub struct App {
    renderer: MathRenderer,
    pub picker: Picker,
    detected_protocol: ProtocolType,
    pub halfblocks: bool,
    pub focus: Focus,
    pub context: PreviewContext,
    pub gallery: ListState,
    pub settings: ListState,
    pub source: String,
    pub editor: Editor,
    pub options: RenderOptions,
    pub palette: usize,
    pub rendered: Option<RenderedEquation>,
    pub prepared: Option<PreparedEquation>,
    prepared_size: Option<Size>,
    pub error: Option<String>,
    pub prepare_error: Option<String>,
    pub render_time: Duration,
    pub status: String,
    pub diagnostic_scroll: u16,
}

impl App {
    pub fn new(renderer: MathRenderer, picker: Picker, source: &str, status: String) -> Self {
        let selected = EXAMPLES.iter().position(|example| example.source == source);
        let options = demo_options(picker.font_size().height);
        let mut app = Self {
            renderer,
            detected_protocol: picker.protocol_type(),
            picker,
            halfblocks: false,
            focus: Focus::Gallery,
            context: PreviewContext::Note,
            gallery: ListState::default().with_selected(selected),
            settings: ListState::default().with_selected(Some(0)),
            source: source.to_owned(),
            editor: Editor::new(source),
            options,
            palette: 0,
            rendered: None,
            prepared: None,
            prepared_size: None,
            error: None,
            prepare_error: None,
            render_time: Duration::ZERO,
            status,
            diagnostic_scroll: 0,
        };
        app.render();
        app
    }

    fn render(&mut self) {
        let start = Instant::now();
        match self.renderer.render(&self.source, &self.options) {
            Ok(rendered) => {
                self.rendered = Some(rendered);
                self.error = None;
            }
            Err(error) => {
                self.rendered = None;
                self.error = Some(error.to_string());
            }
        }
        self.render_time = start.elapsed();
        self.prepared = None;
        self.prepared_size = None;
        self.prepare_error = None;
        self.diagnostic_scroll = 0;
    }

    pub fn prepare(&mut self, area: Rect) {
        let size = area.as_size();
        if self.prepared_size == Some(size) {
            return;
        }
        self.prepared_size = Some(size);
        self.prepared = None;
        self.prepare_error = None;
        let Some(rendered) = &self.rendered else {
            return;
        };
        let padding = if self.picker.protocol_type() == ProtocolType::Halfblocks {
            let [r, g, b] = self.options.background;
            image::Rgba([r, g, b, 255])
        } else {
            image::Rgba([0, 0, 0, 0])
        };
        self.picker.set_background_color(Some(padding));
        match rendered.prepare(&self.picker, size) {
            Ok(prepared) => self.prepared = Some(prepared),
            Err(error) => {
                self.prepare_error = Some(format!(
                    "Preview failed: {error}\nTry the half-block backend in Rendering."
                ))
            }
        }
    }

    pub fn description(&self) -> &str {
        self.gallery.selected().map_or(
            "Custom Typst math. Edit the source or browse an example.",
            |index| EXAMPLES[index].description,
        )
    }

    pub fn diagnostics(&self) -> String {
        if let Some(error) = &self.error {
            return error.clone();
        }
        if let Some(error) = &self.prepare_error {
            return error.clone();
        }
        let Some(rendered) = &self.rendered else {
            return String::new();
        };
        if rendered.warnings().is_empty() {
            "Compiled successfully. No warnings.\nSource locations in diagnostics refer to your expression.".to_owned()
        } else {
            rendered
                .warnings()
                .iter()
                .map(|warning| format!("Warning: {warning}"))
                .collect::<Vec<_>>()
                .join("\n")
        }
    }

    /// Returns true when the user requests exit.
    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            return true;
        }
        if self.focus == Focus::Editor {
            match key.code {
                KeyCode::Esc => {
                    self.editor = Editor::new(&self.source);
                    self.focus = Focus::Gallery;
                    self.status = "Edit cancelled".to_owned();
                }
                KeyCode::Enter
                    if key
                        .modifiers
                        .intersects(KeyModifiers::ALT | KeyModifiers::SHIFT) =>
                {
                    self.editor.insert("\n")
                }
                KeyCode::Enter | KeyCode::Tab => {
                    self.source = self.editor.text.clone();
                    self.gallery.select(
                        EXAMPLES
                            .iter()
                            .position(|example| example.source == self.source),
                    );
                    self.render();
                    self.focus = if key.code == KeyCode::Tab {
                        Focus::Settings
                    } else {
                        Focus::Gallery
                    };
                    self.status = "Source applied. Press e to edit again.".to_owned();
                }
                _ if !key
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
                {
                    self.editor.handle(key.code)
                }
                _ => {}
            }
            return false;
        }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Tab | KeyCode::BackTab => {
                self.focus = if self.focus == Focus::Gallery {
                    Focus::Settings
                } else {
                    Focus::Gallery
                };
            }
            KeyCode::Char('e') => {
                self.editor = Editor::new(&self.source);
                self.focus = Focus::Editor;
            }
            KeyCode::Char('c') => {
                self.context = self.context.next();
                self.status = format!(
                    "{} context. Press c to compare views.",
                    self.context.label()
                );
            }
            KeyCode::Char('n') => self.browse(1),
            KeyCode::Char('p') => self.browse(-1),
            KeyCode::Up | KeyCode::Char('k') => {
                if self.focus == Focus::Gallery {
                    self.browse(-1);
                } else {
                    self.select_setting(-1);
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.focus == Focus::Gallery {
                    self.browse(1);
                } else {
                    self.select_setting(1);
                }
            }
            KeyCode::Left | KeyCode::Char('-') if self.focus == Focus::Settings => self.adjust(-1),
            KeyCode::Right | KeyCode::Char('+') | KeyCode::Enter
                if self.focus == Focus::Settings =>
            {
                self.adjust(1)
            }
            KeyCode::Char('r') => {
                self.options = demo_options(self.picker.font_size().height);
                self.palette = 0;
                self.halfblocks = false;
                self.picker.set_protocol_type(self.detected_protocol);
                self.render();
                self.status = "Rendering defaults restored".to_owned();
            }
            KeyCode::Char('s') => self.export(),
            KeyCode::PageDown => self.diagnostic_scroll = self.diagnostic_scroll.saturating_add(3),
            KeyCode::PageUp => self.diagnostic_scroll = self.diagnostic_scroll.saturating_sub(3),
            _ => {}
        }
        false
    }

    fn browse(&mut self, delta: isize) {
        let index = self
            .gallery
            .selected()
            .map_or(if delta < 0 { EXAMPLES.len() - 1 } else { 0 }, |index| {
                (index as isize + delta).rem_euclid(EXAMPLES.len() as isize) as usize
            });
        self.gallery.select(Some(index));
        self.source = EXAMPLES[index].source.to_owned();
        self.editor = Editor::new(&self.source);
        self.render();
        self.status = format!(
            "Example {} / {}: {}",
            index + 1,
            EXAMPLES.len(),
            EXAMPLES[index].name
        );
    }

    fn select_setting(&mut self, delta: isize) {
        let index = (self.settings.selected().unwrap_or(0) as isize + delta).rem_euclid(5) as usize;
        self.settings.select(Some(index));
    }

    fn adjust(&mut self, delta: isize) {
        match self.settings.selected().unwrap_or(0) {
            0 => {
                self.options.text_size_pt =
                    (self.options.text_size_pt + delta as f64 * 2.0).clamp(8.0, 48.0)
            }
            1 => {
                self.options.padding_pt =
                    (self.options.padding_pt + delta as f64 * 2.0).clamp(0.0, 32.0)
            }
            2 => {
                self.options.pixels_per_pt =
                    (self.options.pixels_per_pt + delta as f64 * 0.5).clamp(0.5, 4.0)
            }
            3 => {
                self.palette =
                    (self.palette as isize + delta).rem_euclid(PALETTES.len() as isize) as usize;
                self.options.foreground = PALETTES[self.palette].1;
                self.options.background = PALETTES[self.palette].2;
            }
            _ => {
                self.halfblocks = !self.halfblocks;
                self.picker.set_protocol_type(if self.halfblocks {
                    ProtocolType::Halfblocks
                } else {
                    self.detected_protocol
                });
                self.prepared_size = None;
                self.status = format!("Preview backend: {:?}", self.picker.protocol_type());
                return;
            }
        }
        self.render();
        self.status = "Rendering updated. r restores defaults; s saves a PNG.".to_owned();
    }

    fn export(&mut self) {
        let Some(rendered) = &self.rendered else {
            self.status = "Cannot export: fix the source error first.".to_owned();
            return;
        };
        let path = Path::new("output/equation.png");
        self.status = match export_png(rendered, path) {
            Ok(()) => format!("Saved {} (replaces the previous export)", path.display()),
            Err(error) => format!("Export failed: {error:#}"),
        };
    }
}
