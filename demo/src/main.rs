mod app;
mod editor;
mod examples;
mod preview;
mod ui;

#[cfg(test)]
mod tests;

use std::{
    env,
    io::{self, IsTerminal},
    path::Path,
};

use anyhow::{Context, Result, bail};
use ratatui::crossterm::event::{self, Event, KeyEventKind};
use ratatui_image::picker::Picker;
use ratatui_typst::{MathRenderer, RenderedEquation};

use app::App;
use examples::EXAMPLES;

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let mut png_path = None;
    let mut source = None;
    let mut halfblocks = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => {
                println!(
                    "ratatui-typst-demo [--halfblocks] [TYPST_MATH]\n\
                     ratatui-typst-demo --png FILE.png [TYPST_MATH]\n\n\
                     Explore examples, edit Typst math, and adjust rendering in a TUI.\n\
                     Pass math without dollar delimiters. --halfblocks skips terminal queries.\n\
                     Tab: change panel | arrows: browse / adjust | e: edit | c: context | q: quit\n\
                     Editor: Enter renders, Alt+Enter inserts a line, Esc cancels."
                );
                return Ok(());
            }
            "--png" => {
                png_path = Some(args.next().context("--png requires an output filename")?);
            }
            "--halfblocks" => halfblocks = true,
            _ if source.is_none() => source = Some(arg),
            _ => bail!("Too many arguments. Quote the equation as one argument; see --help."),
        }
    }
    let source = source.as_deref().unwrap_or(EXAMPLES[0].source);
    let renderer = MathRenderer::new();
    if let Some(path) = png_path {
        let rendered = renderer.render(source, &app::demo_options(20))?;
        for warning in rendered.warnings() {
            eprintln!("warning: {warning}");
        }
        export_png(&rendered, Path::new(&path))?;
        println!(
            "Wrote {path} ({} x {} pixels)",
            rendered.image().width(),
            rendered.image().height()
        );
        return Ok(());
    }
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        bail!("Run in an interactive terminal, or use --png FILE.png to export.");
    }

    // run() restores the terminal on normal exit, errors, and panics.
    ratatui::run(|terminal| {
        // Query once in the alternate screen, before reading keyboard events.
        let (picker, status) = if halfblocks {
            (
                Picker::halfblocks(),
                "Half-block preview requested".to_owned(),
            )
        } else {
            match Picker::from_query_stdio() {
                Ok(picker) => (
                    picker,
                    "Ready. Select an example or press e to edit.".to_owned(),
                ),
                Err(error) => (
                    Picker::halfblocks(),
                    format!("Detection failed; using half-blocks: {error}"),
                ),
            }
        };
        let mut app = App::new(renderer, picker, source, status);
        loop {
            let size = terminal.size()?;
            app.prepare(
                ui::areas(ratatui::layout::Rect::new(0, 0, size.width, size.height)).equation(&app),
            );
            terminal.draw(|frame| ui::draw(frame, &mut app))?;
            match event::read()? {
                Event::Key(key) if key.kind != KeyEventKind::Release => {
                    if app.handle_key(key) {
                        return Ok(());
                    }
                }
                Event::Resize(_, _) => {}
                _ => {}
            }
        }
    })
}

fn export_png(rendered: &RenderedEquation, path: &Path) -> Result<()> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Cannot create {}", parent.display()))?;
    }
    rendered
        .image()
        .save_with_format(path, image::ImageFormat::Png)
        .with_context(|| format!("Cannot write {}", path.display()))
}
