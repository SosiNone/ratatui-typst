//! Typeset block equations with embedded Typst and display them in Ratatui.
//!
//! Compile and prepare outside `Terminal::draw`. Reprepare when the available
//! area or terminal cell dimensions change. The application owns capability
//! detection and its event loop.
//!
//! ```no_run
//! use ratatui::layout::Size;
//! use ratatui_image::picker::Picker;
//! use ratatui_typst::{Equation, MathRenderer, RenderOptions};
//!
//! let renderer = MathRenderer::new();
//! let equation = renderer.render("x = sqrt(2)", &RenderOptions::default())?;
//! let picker = Picker::from_query_stdio()?; // Query once, before reading events.
//! let prepared = equation.prepare(&picker, Size::new(40, 10))?;
//! // Inside terminal.draw: frame.render_widget(Equation::new(&prepared), area);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

mod renderer;
mod widget;

pub(crate) const MAX_RASTER_SIDE: u32 = 16_384;
pub(crate) const MAX_RASTER_PIXELS: u64 = 16_000_000;

pub use renderer::{Diagnostic, MathRenderer, RenderError, RenderOptions, RenderedEquation};
pub use widget::{Equation, PrepareError, PreparedEquation};
