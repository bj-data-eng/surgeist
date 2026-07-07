//! Surgeist is the fresh Rust-native UI authoring toolkit that will grow out of
//! the Data Engine Studio UI research prototype.
//!
//! This crate intentionally starts small. The public API should be rebuilt from
//! first principles around a coherent authoring language for retained document
//! UIs: Rust-first dynamic structure, CSS styling, typed behavior intent, and
//! host adapters that do not own app semantics.

pub mod adapters;
pub mod app;
pub mod css {
    pub use surgeist_css::*;
}
pub mod dialog {
    pub use surgeist_dialog::*;
}
pub mod layout {
    pub use surgeist_layout::*;
}
pub mod render {
    pub use surgeist_render::*;
}
pub mod retained {
    pub use surgeist_retained::*;
}
pub mod runtime {
    pub use surgeist_runtime::*;
}
pub mod shape {
    pub use surgeist_shape::*;
}
pub mod style {
    pub use surgeist_style::*;
}
pub mod task {
    pub use surgeist_task::*;
}
pub mod template {
    pub use surgeist_template::*;
}
pub mod text {
    pub use surgeist_text::*;
}
pub mod window {
    pub use surgeist_window::*;
}

/// Returns the crate identity while the first-principles API is being designed.
pub const fn crate_name() -> &'static str {
    "surgeist"
}

#[cfg(test)]
mod tests {
    #[test]
    fn exposes_crate_identity() {
        assert_eq!(super::crate_name(), "surgeist");
    }
}
