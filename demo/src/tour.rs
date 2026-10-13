use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, List, ListItem, Paragraph, Wrap},
};

use crate::{
    app::App,
    preview::{self, PreviewContext},
};

pub struct Scene {
    pub title: &'static str,
    pub example: usize,
    pub context: PreviewContext,
    pub palette: usize,
    pub explanation: &'static str,
}

pub const SCENES: &[Scene] = &[
    Scene {
        title: "Terminal notebook",
        example: 0,
        context: PreviewContext::Note,
        palette: 0,
        explanation: "Write the explanation in terminal text. Let Typst handle fractions, roots, and mathematical spacing. Press c to compare contexts.",
    },
    Scene {
        title: "Calculator console",
        example: 1,
        context: PreviewContext::Console,
        palette: 0,
        explanation: "A command result can contain typeset math. Integral limits and differentials come from Typst; the prompt stays ordinary terminal text.",
    },
    Scene {
        title: "Linear algebra",
        example: 3,
        context: PreviewContext::Note,
        palette: 0,
        explanation: "Show structured results in a scientific tool. Matrix rows and columns keep their spacing as the image fits the available terminal area.",
    },
    Scene {
        title: "Worked derivation",
        example: 4,
        context: PreviewContext::Note,
        palette: 0,
        explanation: "Walk through a calculation in a teaching app. Alignment points and line breaks keep successive steps readable. Press e to try your own derivation.",
    },
    Scene {
        title: "Presentation controls",
        example: 6,
        context: PreviewContext::Note,
        palette: 2,
        explanation: "Match equations to your application's palette. Transparent padding blends into this Teal panel. Press Tab to experiment with size, padding, and color.",
    },
    Scene {
        title: "Error recovery",
        example: 8,
        context: PreviewContext::Note,
        palette: 0,
        explanation: "Invalid source replaces the image with diagnostics. Press e, replace missingname with beta, then Enter to recover. The surrounding note stays readable.",
    },
];

const ACCENT: Color = Color::Rgb(115, 210, 240);
const WARM: Color = Color::Rgb(255, 193, 120);
const MUTED: Color = Color::Rgb(145, 160, 180);

pub struct Areas {
    header: Rect,
    navigation: Rect,
    story: Rect,
    pub preview: Rect,
    pub source: Rect,
    details: Rect,
    footer: Rect,
}

pub fn areas(area: Rect) -> Areas {
    let wide = area.width >= 100 && area.height >= 30;
    let [header, body, footer] = Layout::vertical([
        Constraint::Length(if wide { 3 } else { 2 }),
        Constraint::Min(0),
        Constraint::Length(if wide { 3 } else { 2 }),
    ])
    .areas(area);
    let [navigation, content] = Layout::horizontal([
        Constraint::Length(if wide { 27 } else { 0 }),
        Constraint::Min(0),
    ])
    .areas(body);
    let [story, preview, source, details] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(4),
        Constraint::Length(if wide { 5 } else { 3 }),
    ])
    .areas(content);
    Areas {
        header,
        navigation,
        story,
        preview,
        source,
        details,
        footer,
    }
}

pub fn equation_area(area: Rect, app: &App) -> Rect {
    preview::areas(Block::bordered().inner(areas(area).preview), app).equation
}

fn panel(title: &str) -> Block<'_> {
    Block::bordered()
        .title(title)
        .border_style(Style::default().fg(ACCENT))
}

pub fn draw(frame: &mut Frame, app: &mut App) {
    let a = areas(frame.area());
    let scene_index = app.scene_index();
    let scene = scene_index.map(|index| &SCENES[index]);
    let title = scene.map_or("Custom equation", |scene| scene.title);
    let progress = scene_index.map_or("CUSTOM".to_owned(), |index| {
        format!("{:02} / {:02}", index + 1, SCENES.len())
    });
    frame.render_widget(
        Paragraph::new(vec![
            Line::from(" RATATUI + TYPST  /  MATH THAT BELONGS IN YOUR TERMINAL")
                .style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
            Line::from(format!(" GUIDED TOUR   {progress}   /   {title}"))
                .style(Style::default().fg(WARM)),
        ]),
        a.header,
    );
    let items = SCENES.iter().enumerate().map(|(index, scene)| {
        ListItem::new(vec![
            Line::from(format!("{:02} {}", index + 1, scene.title)),
            Line::from(""),
        ])
        .style(Style::default().fg(if Some(index) == scene_index {
            WARM
        } else {
            MUTED
        }))
    });
    frame.render_widget(List::new(items).block(panel(" THE TOUR ")), a.navigation);
    frame.render_widget(Paragraph::new(scene.map_or(
        "Your equation and rendering settings are preserved. Press n to resume the tour, or x to continue exploring.",
        |scene| scene.explanation,
    )).wrap(Wrap { trim: false }).style(Style::default().fg(MUTED)), a.story);
    frame.render_widget(panel(" LIVE RESULT / c: context "), a.preview);
    preview::draw(frame, app, Block::bordered().inner(a.preview));
    frame.render_widget(
        Paragraph::new(app.source.as_str())
            .wrap(Wrap { trim: false })
            .block(panel(" TYPST SOURCE / e: make it yours ")),
        a.source,
    );
    let font = app.picker.font_size();
    let mut info = vec![Line::from(format!(
        "{:?} | cell {}x{} px | {:.1} pt | compile {:.1} ms",
        app.picker.protocol_type(),
        font.width,
        font.height,
        app.options.text_size_pt,
        app.render_time.as_secs_f64() * 1000.0,
    ))];
    info.extend(
        app.diagnostics()
            .lines()
            .map(|line| Line::from(line.to_owned())),
    );
    let info = Paragraph::new(info).wrap(Wrap { trim: false });
    let max_scroll = info
        .line_count(a.details.width)
        .saturating_sub(usize::from(a.details.height));
    app.diagnostic_scroll = app
        .diagnostic_scroll
        .min(max_scroll.min(u16::MAX as usize) as u16);
    frame.render_widget(
        info.scroll((app.diagnostic_scroll, 0))
            .style(
                Style::default().fg(if app.error.is_some() || app.prepare_error.is_some() {
                    WARM
                } else {
                    MUTED
                }),
            ),
        a.details,
    );
    frame.render_widget(
        Paragraph::new(vec![
            Line::from(" n/p arrows: scene | x explorer | e edit | Tab controls | c context")
                .style(Style::default().fg(ACCENT)),
            Line::from(format!(
                " s PNG | r reset | PgUp/PgDn details | q quit   /   {}",
                app.status
            ))
            .style(Style::default().fg(MUTED)),
        ]),
        a.footer,
    );
}
