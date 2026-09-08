use std::path::PathBuf;

use super::{Backend, Options, Result};
#[cfg(not(feature = "system"))]
use super::{Error, ErrorCode};

/// Native system dialog backend.
#[derive(Clone, Copy, Debug, Default)]
pub struct SystemBackend;

impl Backend for SystemBackend {
    fn open_file(&mut self, options: &Options) -> Result<Option<PathBuf>> {
        #[cfg(feature = "system")]
        {
            Ok(system_dialog(options).pick_file())
        }
        #[cfg(not(feature = "system"))]
        {
            let _ = options;
            unavailable()
        }
    }

    fn open_files(&mut self, options: &Options) -> Result<Option<Vec<PathBuf>>> {
        #[cfg(feature = "system")]
        {
            Ok(system_dialog(options).pick_files())
        }
        #[cfg(not(feature = "system"))]
        {
            let _ = options;
            unavailable()
        }
    }

    fn open_folder(&mut self, options: &Options) -> Result<Option<PathBuf>> {
        #[cfg(feature = "system")]
        {
            Ok(system_dialog(options).pick_folder())
        }
        #[cfg(not(feature = "system"))]
        {
            let _ = options;
            unavailable()
        }
    }

    fn open_folders(&mut self, options: &Options) -> Result<Option<Vec<PathBuf>>> {
        #[cfg(feature = "system")]
        {
            Ok(system_dialog(options).pick_folders())
        }
        #[cfg(not(feature = "system"))]
        {
            let _ = options;
            unavailable()
        }
    }

    fn save_file(&mut self, options: &Options) -> Result<Option<PathBuf>> {
        #[cfg(feature = "system")]
        {
            Ok(system_dialog(options).save_file())
        }
        #[cfg(not(feature = "system"))]
        {
            let _ = options;
            unavailable()
        }
    }
}

#[cfg(feature = "system")]
fn system_dialog(options: &Options) -> rfd::FileDialog {
    let mut dialog = rfd::FileDialog::new();
    if let Some(title) = &options.title {
        dialog = dialog.set_title(title);
    }
    if let Some(directory) = &options.directory {
        dialog = dialog.set_directory(directory);
    }
    if let Some(file_name) = &options.file_name {
        dialog = dialog.set_file_name(file_name);
    }
    for filter in &options.filters {
        dialog = dialog.add_filter(&filter.name, &filter.extensions);
    }
    dialog
}

#[cfg(not(feature = "system"))]
fn unavailable<T>() -> Result<T> {
    Err(Error::new(
        ErrorCode::BackendUnavailable,
        "system dialog backend is disabled",
    ))
}
