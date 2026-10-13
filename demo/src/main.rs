mod app;
mod editor;
mod examples;
mod preview;
mod tour;
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
    let args = Args::parse(env::args().skip(1))?;
    if args.help {
        println!(
            "ratatui-typst-demo [--halfblocks] [TYPST_MATH]\n\
                  ratatui-typst-demo --png FILE.png [TYPST_MATH]\n\
                  ratatui-typst-demo --showcase-assets DIR\n\n\
                  No source: guided tour. Explicit source: capability explorer.\n\
                  Pass math without dollar delimiters. --halfblocks skips terminal queries.\n\
                  n/p or arrows: scene | x: switch mode | e: edit | Tab: controls | q: quit\n\
                  c: context | r: reset | s: PNG | PgUp/PgDn: diagnostics\n\
                  Editor: Enter renders, Alt+Enter inserts a line, Esc cancels.\n\
                  --showcase-assets exports four opaque Paper examples without a TUI."
        );
        return Ok(());
    }
    let renderer = MathRenderer::new();
    if let Some(path) = &args.showcase_path {
        return export_showcase(&renderer, Path::new(path));
    }
    if let Some(path) = &args.png_path {
        let source = args.source.as_deref().unwrap_or(EXAMPLES[0].source);
        let rendered = renderer.render(source, &app::demo_options(20))?;
        for warning in rendered.warnings() {
            eprintln!("warning: {warning}");
        }
        export_png(&rendered, Path::new(path))?;
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
        let (picker, status) = if args.halfblocks {
            (
                Picker::halfblocks(),
                "Half-block preview requested".to_owned(),
            )
        } else {
            match Picker::from_query_stdio() {
                Ok(picker) => (picker, "Ready. n/p: browse; x: switch view.".to_owned()),
                Err(error) => (
                    Picker::halfblocks(),
                    format!("Detection failed; using half-blocks: {error}"),
                ),
            }
        };
        let mut app = App::new(renderer, picker, args.source.as_deref(), status.clone());
        app.status = status;
        loop {
            let size = terminal.size()?;
            app.prepare(ui::equation_area(
                ratatui::layout::Rect::new(0, 0, size.width, size.height),
                &app,
            ));
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

#[derive(Default)]
struct Args {
    png_path: Option<String>,
    showcase_path: Option<String>,
    source: Option<String>,
    halfblocks: bool,
    help: bool,
}

impl Args {
    fn parse(args: impl IntoIterator<Item = String>) -> Result<Self> {
        let mut args = args.into_iter();
        let mut parsed = Self::default();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--help" | "-h" => {
                    parsed.help = true;
                    return Ok(parsed);
                }
                "--png" => {
                    parsed.png_path = Some(
                        args.next()
                            .filter(|value| !value.starts_with("--"))
                            .context("--png requires an output filename")?,
                    );
                }
                "--showcase-assets" => {
                    parsed.showcase_path = Some(
                        args.next()
                            .filter(|value| !value.starts_with("--"))
                            .context("--showcase-assets requires an output directory")?,
                    );
                }
                "--halfblocks" => parsed.halfblocks = true,
                _ if arg.starts_with("--") => bail!("Unknown option {arg}; see --help."),
                _ if parsed.source.is_none() => parsed.source = Some(arg),
                _ => bail!("Too many arguments. Quote the equation as one argument; see --help."),
            }
        }
        if parsed.showcase_path.is_some()
            && (parsed.png_path.is_some() || parsed.source.is_some() || parsed.halfblocks)
        {
            bail!("--showcase-assets cannot be combined with source, --png, or --halfblocks.");
        }
        Ok(parsed)
    }
}

const SHOWCASE_EXAMPLES: &[(&str, usize)] = &[
    ("quadratic.png", 0),
    ("integral.png", 1),
    ("matrix.png", 3),
    ("piecewise.png", 5),
];

fn export_showcase(renderer: &MathRenderer, directory: &Path) -> Result<()> {
    let options = ratatui_typst::RenderOptions {
        text_size_pt: 20.0,
        padding_pt: 10.0,
        foreground: app::PALETTES[1].1,
        background: app::PALETTES[1].2,
        transparent_background: false,
        ..Default::default()
    };
    for &(filename, example) in SHOWCASE_EXAMPLES {
        let rendered = renderer.render(EXAMPLES[example].source, &options)?;
        let path = directory.join(filename);
        export_png(&rendered, &path)?;
        println!(
            "Wrote {} ({} x {} pixels)",
            path.display(),
            rendered.image().width(),
            rendered.image().height()
        );
    }
    Ok(())
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
