use std::path::PathBuf;

use super::{Options, Result};

/// File dialog backend contract.
pub trait Backend {
    fn open_file(&mut self, options: &Options) -> Result<Option<PathBuf>>;
    fn open_files(&mut self, options: &Options) -> Result<Option<Vec<PathBuf>>>;
    fn open_folder(&mut self, options: &Options) -> Result<Option<PathBuf>>;
    fn open_folders(&mut self, options: &Options) -> Result<Option<Vec<PathBuf>>>;
    fn save_file(&mut self, options: &Options) -> Result<Option<PathBuf>>;
}
