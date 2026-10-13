use crate::{
    app::{App, Focus, Mode, PALETTES},
    examples::EXAMPLES,
    preview,
};

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, List, ListItem, Paragraph, Wrap},
};

pub fn equation_area(area: Rect, app: &App) -> Rect {
    if app.mode == Mode::Tour {
        crate::tour::equation_area(area, app)
    } else {
        areas(area).equation(app)
    }
}

const BACKGROUND: Color = Color::Rgb(16, 20, 28);
const FOREGROUND: Color = Color::Rgb(227, 234, 244);
const MUTED: Color = Color::Rgb(145, 160, 180);
const ACCENT: Color = Color::Rgb(115, 210, 240);

pub struct Panel {
    pub outer: Rect,
    pub inner: Rect,
}

impl Panel {
    fn new(outer: Rect) -> Self {
        Self {
            outer,
            inner: Block::bordered().inner(outer),
        }
    }
}

pub struct Areas {
    header: Rect,
    gallery: Panel,
    settings: Panel,
    source: Panel,
    pub preview: Panel,
    details: Panel,
    footer: Rect,
}

impl Areas {
    pub fn equation(&self, app: &App) -> Rect {
        preview::areas(self.preview.inner, app).equation
    }
}

pub fn areas(area: Rect) -> Areas {
    let [header, body, footer] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(3),
    ])
    .areas(area);
    let [sidebar, content] =
        Layout::horizontal([Constraint::Length(27), Constraint::Min(0)]).areas(body);
    let [gallery, settings] =
        Layout::vertical([Constraint::Length(11), Constraint::Min(0)]).areas(sidebar);
    let compact = area.height < 30;
    let [source, preview, details] = Layout::vertical([
        Constraint::Length(if compact { 3 } else { 5 }),
        Constraint::Min(0),
        Constraint::Length(if compact { 5 } else { 8 }),
    ])
    .areas(content);
    Areas {
        header,
        gallery: Panel::new(gallery),
        settings: Panel::new(settings),
        source: Panel::new(source),
        preview: Panel::new(preview),
        details: Panel::new(details),
        footer,
    }
}

fn block(title: &str, focused: bool) -> Block<'_> {
    Block::bordered()
        .title(title)
        .border_style(Style::default().fg(if focused { ACCENT } else { MUTED }))
}

pub fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    frame.render_widget(
        Block::default().style(Style::default().bg(BACKGROUND).fg(FOREGROUND)),
        area,
    );
    if area.width < 76 || area.height < 24 {
        frame.render_widget(
            Paragraph::new(
                "Enlarge the terminal to at least 76 columns x 24 rows.\nq / Esc / Ctrl+C: quit",
            )
            .wrap(Wrap { trim: false })
            .block(block(" ratatui-typst explorer ", true)),
            area,
        );
        return;
    }
    if app.mode == Mode::Tour {
        crate::tour::draw(frame, app);
        return;
    }
    let a = areas(area);
    frame.render_widget(
        Paragraph::new(vec![
            Line::from(" ratatui-typst / capability explorer")
                .style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
            Line::from(format!(" {}", app.description())).style(Style::default().fg(MUTED)),
        ])
        .wrap(Wrap { trim: false }),
        a.header,
    );

    let items = EXAMPLES
        .iter()
        .enumerate()
        .map(|(index, example)| {
            ListItem::new(format!(
                "{}  {}{}",
                index + 1,
                example.name,
                if example.valid { "" } else { " !" }
            ))
        })
        .collect::<Vec<_>>();
    frame.render_stateful_widget(
        List::new(items)
            .block(block(" Examples ", app.focus == Focus::Gallery))
            .highlight_symbol("> ")
            .highlight_style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        a.gallery.outer,
        &mut app.gallery,
    );

    let settings = [
        format!("Text size    {:>4.0} pt", app.options.text_size_pt),
        format!("Padding      {:>4.0} pt", app.options.padding_pt),
        format!("Raster scale {:.1} px/pt", app.options.pixels_per_pt),
        format!("Palette      {}", PALETTES[app.palette].0),
        format!(
            "Backend      {}",
            if app.halfblocks { "Blocks" } else { "Auto" }
        ),
    ];
    frame.render_stateful_widget(
        List::new(settings)
            .block(block(" Rendering ", app.focus == Focus::Settings))
            .highlight_symbol("> ")
            .highlight_style(Style::default().fg(ACCENT)),
        a.settings.outer,
        &mut app.settings,
    );

    let editing = app.focus == Focus::Editor;
    let (row, column) = app.editor.position();
    let source_text = if editing {
        &app.editor.text
    } else {
        &app.source
    };
    let scroll_y = if editing {
        row.saturating_sub(a.source.inner.height.saturating_sub(1) as usize)
    } else {
        0
    };
    let scroll_x = if editing {
        column.saturating_sub(a.source.inner.width.saturating_sub(1) as usize)
    } else {
        0
    };
    let mut source = Paragraph::new(source_text.as_str()).block(block(
        if editing {
            " Source / editing "
        } else {
            " Typst source / e: edit "
        },
        editing,
    ));
    if editing {
        source = source.scroll((
            scroll_y.min(u16::MAX as usize) as u16,
            scroll_x.min(u16::MAX as usize) as u16,
        ));
    } else {
        source = source.wrap(Wrap { trim: false });
    }
    frame.render_widget(source, a.source.outer);
    if editing {
        frame.set_cursor_position((
            a.source.inner.x + (column - scroll_x) as u16,
            a.source.inner.y + (row - scroll_y) as u16,
        ));
    }

    let preview_title = format!(" Preview / {} / c: context ", app.context.label());
    frame.render_widget(block(&preview_title, false), a.preview.outer);
    preview::draw(frame, app, a.preview.inner);

    let font = app.picker.font_size();
    let mut lines = vec![Line::from(format!(
        "{:?} | cell {}x{} px | compile {:.1} ms",
        app.picker.protocol_type(),
        font.width,
        font.height,
        app.render_time.as_secs_f64() * 1000.0
    ))];
    if let Some(rendered) = &app.rendered {
        let [width, height] = rendered.natural_size_pt();
        let required = rendered.required_cells(ratatui::layout::Size::new(font.width, font.height));
        lines.push(Line::from(format!(
            "{width:.1}x{height:.1} pt | {}x{} px | natural {} | panel {}x{} cells",
            rendered.image().width(),
            rendered.image().height(),
            required.map_or_else(
                || "unknown".to_owned(),
                |size| format!("{}x{} cells", size.width, size.height)
            ),
            a.equation(app).width,
            a.equation(app).height
        )));
    }
    let diagnostics = app.diagnostics();
    lines.extend(diagnostics.lines().map(|line| {
        Line::from(line.to_owned()).style(
            Style::default().fg(if app.error.is_some() || app.prepare_error.is_some() {
                Color::Rgb(245, 145, 130)
            } else if app
                .rendered
                .as_ref()
                .is_some_and(|rendered| !rendered.warnings().is_empty())
            {
                Color::Yellow
            } else {
                MUTED
            }),
        )
    }));
    // Wrap explicitly so scrolling is bounded without Ratatui's unstable line-count API.
    let lines = wrap_lines(lines, a.details.inner.width);
    let max_scroll = lines.len().saturating_sub(a.details.inner.height as usize);
    let paragraph = Paragraph::new(lines);
    app.diagnostic_scroll = app
        .diagnostic_scroll
        .min(max_scroll.min(u16::MAX as usize) as u16);
    frame.render_widget(
        paragraph
            .scroll((app.diagnostic_scroll, 0))
            .block(block(" Render info & diagnostics / PgUp PgDn ", false)),
        a.details.outer,
    );

    let help = if editing {
        " Enter render | Alt+Enter newline | arrows move | Esc cancel | Ctrl+C quit"
    } else {
        " Tab panel | arrows | e edit | x tour | c context | n/p | r reset | s PNG | q quit"
    };
    frame.render_widget(
        Paragraph::new(vec![
            Line::from(help).style(Style::default().fg(ACCENT)),
            Line::from(format!(" {}", app.status)).style(Style::default().fg(MUTED)),
        ]),
        a.footer,
    );
}

fn wrap_lines(lines: Vec<Line<'static>>, width: u16) -> Vec<Line<'static>> {
    let mut wrapped = Vec::new();
    for line in lines {
        let mut text = String::new();
        let mut used = 0;
        for character in line.spans.iter().flat_map(|span| span.content.chars()) {
            let size = Line::from(character.to_string()).width();
            if used + size > usize::from(width) && !text.is_empty() {
                wrapped.push(Line::from(std::mem::take(&mut text)).style(line.style));
                used = 0;
            }
            text.push(character);
            used += size;
        }
        wrapped.push(Line::from(text).style(line.style));
    }
    wrapped
}
