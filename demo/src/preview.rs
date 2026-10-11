use ratatui::{
    Frame,
    layout::{Margin, Rect, Size},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Paragraph, Wrap},
};
use ratatui_image::Resize;
use ratatui_typst::Equation;

use crate::{app::App, examples::EXAMPLES};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PreviewContext {
    Note,
    Console,
    Plain,
}

impl PreviewContext {
    pub fn next(self) -> Self {
        match self {
            Self::Note => Self::Console,
            Self::Console => Self::Plain,
            Self::Plain => Self::Note,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Note => "Note",
            Self::Console => "Console",
            Self::Plain => "Plain",
        }
    }
}

pub struct ContextAreas {
    pub before: Rect,
    pub equation: Rect,
    pub after: Rect,
}

pub fn areas(area: Rect, app: &App) -> ContextAreas {
    if app.context == PreviewContext::Plain {
        return ContextAreas {
            before: Rect::default(),
            equation: area,
            after: Rect::default(),
        };
    }
    let roomy = area.height >= 12;
    let content = area.inner(Margin::new(
        if area.width >= 8 { 2 } else { 0 },
        u16::from(roomy),
    ));
    let (before_text, after_text) = context_text(app);
    let rows = |text: &Paragraph<'_>| text.line_count(content.width).min(u16::MAX as usize) as u16;
    let before_height = rows(&before_text).min(content.height);
    let before = Rect::new(content.x, content.y, content.width, before_height);
    let remaining = content.height.saturating_sub(before_height);
    let after_height = rows(&after_text).min(remaining);
    // Reserve the surrounding text first. Short panels sacrifice blank rows
    // and the preview rather than let the image cover text.
    let image_budget = remaining.saturating_sub(after_height);
    let gap = u16::from(image_budget >= 3);
    let available = Size::new(content.width, image_budget.saturating_sub(2 * gap));
    let font = app.picker.font_size();
    let height = if available.width == 0 || available.height == 0 {
        0
    } else if let Some(rendered) = &app.rendered {
        Resize::Fit(None)
            .size_for(rendered.image(), font, available)
            .height
    } else {
        Paragraph::new("Preview unavailable. See diagnostics; e: edit.")
            .wrap(Wrap { trim: false })
            .line_count(available.width)
            .min(usize::from(available.height)) as u16
    };
    let equation = Rect::new(content.x, before.bottom() + gap, content.width, height);
    let after = Rect::new(
        content.x,
        equation.bottom() + gap,
        content.width,
        after_height,
    );
    ContextAreas {
        before,
        equation,
        after,
    }
}

fn context_text(app: &App) -> (Paragraph<'static>, Paragraph<'static>) {
    let example = app.gallery.selected().map(|index| &EXAMPLES[index]);
    let name = example.map_or("Custom equation", |example| example.name);
    let (heading, introduction, followup) = match app.context {
        PreviewContext::Note => (
            name.to_owned(),
            example.map_or("A typeset formula inside a terminal note:", |e| {
                e.introduction
            }),
            example.map_or("Continue reading in ordinary terminal text.", |e| {
                e.followup
            }),
        ),
        PreviewContext::Console => (format!("math> {name}"), "Result:", "math> _"),
        PreviewContext::Plain => (String::new(), "", ""),
    };
    (
        Paragraph::new(vec![
            Line::from(heading).style(Style::default().add_modifier(Modifier::BOLD)),
            Line::from(introduction),
        ])
        .wrap(Wrap { trim: false }),
        Paragraph::new(followup).wrap(Wrap { trim: false }),
    )
}

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let [r, g, b] = app.options.background;
    let [fr, fg, fb] = app.options.foreground;
    let style = Style::default()
        .bg(Color::Rgb(r, g, b))
        .fg(Color::Rgb(fr, fg, fb));
    frame.render_widget(Block::default().style(style), area);
    let a = areas(area, app);
    if app.context != PreviewContext::Plain {
        let (before, after) = context_text(app);
        frame.render_widget(before.style(style), a.before);
        frame.render_widget(after.style(style), a.after);
    }
    if app.error.is_some() || app.prepare_error.is_some() {
        frame.render_widget(
            Paragraph::new("Preview unavailable. See diagnostics; e: edit.")
                .style(style)
                .wrap(Wrap { trim: false }),
            a.equation,
        );
    } else if let Some(prepared) = &app.prepared {
        frame.render_widget(Equation::new(prepared), a.equation);
    }
}
