//! A small consumer using only the public API; also a manual lifecycle test.
//! Run with --halfblocks to skip detection, or --smoke for headless checks.
use std::{
    error::Error,
    io::{self, IsTerminal},
};

use ratatui::{
    Frame, Terminal,
    backend::TestBackend,
    crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    layout::{Constraint, Layout, Rect, Size},
    style::{Color, Style},
    widgets::{Block, Clear, Paragraph, Wrap},
};
use ratatui_image::picker::Picker;
use ratatui_typst::{Equation, MathRenderer, PreparedEquation, RenderOptions, RenderedEquation};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
const SOURCES: [&str; 3] = ["x = sqrt(2)", "A = mat(1, 2; 3, 4)", "α + missingname"];

struct App {
    renderer: MathRenderer,
    picker: Picker,
    options: RenderOptions,
    selected: usize,
    rendered: Option<RenderedEquation>,
    prepared: Option<PreparedEquation>,
    prepared_size: Option<Size>,
    diagnostic: String,
    moved: bool,
    hidden: bool,
    overlay: bool,
    narrow: bool,
}

impl App {
    fn new(picker: Picker) -> Self {
        let mut app = Self {
            renderer: MathRenderer::new(),
            picker,
            options: RenderOptions::default(),
            selected: 0,
            rendered: None,
            prepared: None,
            prepared_size: None,
            diagnostic: String::new(),
            moved: false,
            hidden: false,
            overlay: false,
            narrow: false,
        };
        app.compile();
        app
    }

    fn compile(&mut self) {
        self.prepared = None;
        self.prepared_size = None;
        match self.renderer.render(SOURCES[self.selected], &self.options) {
            Ok(rendered) => {
                self.diagnostic = rendered
                    .warnings()
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("\n");
                self.rendered = Some(rendered);
            }
            Err(error) => {
                self.diagnostic = error.to_string();
                self.rendered = None;
            }
        }
    }

    fn action(&mut self, code: KeyCode) -> bool {
        match code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char('n') => {
                self.selected = (self.selected + 1) % SOURCES.len();
                self.compile();
            }
            KeyCode::Char('m') => self.moved = !self.moved,
            KeyCode::Char('h') => self.hidden = !self.hidden,
            KeyCode::Char('o') => self.overlay = !self.overlay,
            KeyCode::Char('s') => self.narrow = !self.narrow,
            KeyCode::Char('t') => {
                self.options.transparent_background = !self.options.transparent_background;
                self.compile();
            }
            _ => {}
        }
        false
    }

    fn panel(area: Rect) -> Rect {
        Layout::vertical([
            Constraint::Length(4),
            Constraint::Min(0),
            Constraint::Length(4),
        ])
        .split(area)[1]
    }

    fn image_area(&self, area: Rect) -> Rect {
        let mut inner = Block::bordered().inner(Self::panel(area));
        if self.narrow {
            inner.width = inner.width.min(6);
            inner.height = inner.height.min(2);
        }
        if self.moved {
            inner.x += inner.width / 3;
            inner.width -= inner.width / 3;
        }
        inner
    }

    fn prepare(&mut self, area: Rect) -> Result<()> {
        let size = self.image_area(area).as_size();
        if self.prepared_size != Some(size) {
            // Compilation, proportional fitting and protocol encoding stay outside draw.
            self.prepared = self
                .rendered
                .as_ref()
                .map(|rendered| rendered.prepare(&self.picker, size))
                .transpose()?;
            self.prepared_size = Some(size);
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let area = frame.area();
        frame.render_widget(
            Block::default().style(
                Style::default()
                    .bg(Color::Rgb(16, 20, 28))
                    .fg(Color::Rgb(227, 234, 244)),
            ),
            area,
        );
        let [text, _, status] = Layout::vertical([
            Constraint::Length(4),
            Constraint::Min(0),
            Constraint::Length(4),
        ])
        .areas(area);
        frame.render_widget(Paragraph::new(format!("A note with ordinary terminal text above the equation.\nSource: {}\nCompare the glyph edges and background with this sentence.", SOURCES[self.selected]))
            .wrap(Wrap { trim: false }), text);
        frame.render_widget(
            Block::bordered().title(" Equation / adjacent text and borders must stay intact "),
            Self::panel(area),
        );
        let viewport = self.image_area(area);
        if !self.hidden && !self.overlay {
            if let Some(prepared) = &self.prepared {
                let size = prepared.size();
                // Center from the actual encoded size, not the natural raster dimensions.
                let centered = Rect::new(
                    viewport.x + viewport.width.saturating_sub(size.width) / 2,
                    viewport.y + viewport.height.saturating_sub(size.height) / 2,
                    size.width,
                    size.height,
                );
                frame.render_widget(Equation::new(prepared), centered);
            } else {
                frame.render_widget(
                    Paragraph::new(self.diagnostic.as_str()).wrap(Wrap { trim: false }),
                    viewport,
                );
            }
        }
        if self.overlay {
            // Remove the image before covering it: raster graphics need lifecycle testing.
            let popup = Block::bordered().inner(Self::panel(area));
            frame.render_widget(Clear, popup);
            frame.render_widget(Paragraph::new("Overlay text must be readable; no equation should remain here.\nPress o to restore the equation.")
                .wrap(Wrap { trim: false }).block(Block::bordered().title(" Overlay ")), popup);
        }
        let cell = self.picker.font_size();
        frame.render_widget(Paragraph::new(format!(
            "Terminal text below the equation remains readable.\n{:?} | cell {}x{} px | alpha {} | hidden {} | moved {}\nn next/error | m move | h hide | o overlay | s small | t alpha | q quit",
            self.picker.protocol_type(), cell.width, cell.height, self.options.transparent_background, self.hidden, self.moved,
        )).wrap(Wrap { trim: false }), status);
    }
}

fn smoke() -> Result<()> {
    let mut app = App::new(Picker::halfblocks());
    let mut terminal = Terminal::new(TestBackend::new(80, 24))?;
    for (width, height, action) in [
        (80, 24, ' '),
        (20, 8, 's'),
        (100, 36, 's'),
        (100, 36, 'm'),
        (100, 36, 'o'),
        (100, 36, 'o'),
        (100, 36, 'h'),
        (100, 36, 'h'),
        (100, 36, 't'),
        (100, 36, 'n'),
        (100, 36, 'n'),
        (100, 36, 'n'),
        (0, 0, ' '),
        (80, 24, ' '),
    ] {
        app.action(KeyCode::Char(action));
        terminal.backend_mut().resize(width, height);
        terminal.resize(Rect::new(0, 0, width, height))?;
        app.prepare(Rect::new(0, 0, width, height))?;
        terminal.draw(|frame| app.draw(frame))?;
        if let Some(prepared) = &app.prepared {
            let available = app.image_area(Rect::new(0, 0, width, height)).as_size();
            assert!(
                prepared.size().width <= available.width
                    && prepared.size().height <= available.height
            );
        }
        if width >= 80 {
            let text = terminal
                .backend()
                .buffer()
                .content
                .iter()
                .map(|cell| cell.symbol())
                .collect::<String>();
            assert!(text.contains("Terminal text below the equation"));
            let has_image = text.contains('▀');
            assert_eq!(
                has_image,
                app.rendered.is_some() && !app.hidden && !app.overlay
            );
            if app.overlay {
                assert!(text.contains("Overlay text must be readable"));
            }
            if app.rendered.is_none() {
                assert!(text.contains("unknown variable"));
            }
        }
    }
    println!(
        "Consumer smoke checks passed (half-block TestBackend; not a real graphics terminal)."
    );
    Ok(())
}

fn main() -> Result<()> {
    let mut halfblocks = false;
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--smoke" => return smoke(),
            "--halfblocks" => halfblocks = true,
            "--help" | "-h" => {
                println!(
                    "terminal [--halfblocks | --smoke]\nn: replace/error, m: move, h: hide, o: overlay, s: small, t: alpha, q: quit"
                );
                return Ok(());
            }
            _ => return Err(format!("Unknown argument: {arg}").into()),
        }
    }
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Err("Use an interactive terminal, or --smoke for headless checks".into());
    }
    ratatui::run(|terminal| {
        let picker = if halfblocks {
            Picker::halfblocks()
        } else {
            Picker::from_query_stdio()?
        };
        let mut app = App::new(picker);
        loop {
            let size = terminal.size()?;
            app.prepare(Rect::new(0, 0, size.width, size.height))?;
            terminal.draw(|frame| app.draw(frame))?;
            if let Event::Key(key) = event::read()?
                && key.kind != KeyEventKind::Release
                && (app.action(key.code)
                    || (key.code == KeyCode::Char('c')
                        && key.modifiers.contains(KeyModifiers::CONTROL)))
            {
                return Ok(());
            }
        }
    })
}
