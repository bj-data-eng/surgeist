//! Facade crate for the Surgeist UI framework.
//!
//! The root crate keeps the public composition surface small while each
//! implementation boundary remains owned by its focused crate.

/// The package name.
pub const NAME: &str = "surgeist";

pub use surgeist_css as css;
pub use surgeist_dialog as dialog;
pub use surgeist_layout as layout;
pub use surgeist_render as render;
pub use surgeist_retained as retained;
pub use surgeist_shape as shape;
pub use surgeist_style as style;
pub use surgeist_text as text;
pub use surgeist_window as window;
