use std::path::PathBuf;

use super::{Backend, Options, Result, SystemBackend};

/// Convenience facade over a file dialog backend.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileDialog {
    options: Options,
}

impl FileDialog {
    #[must_use]
    pub fn new() -> Self {
        Self {
            options: Options::new(),
        }
    }

    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.options = self.options.title(title);
        self
    }

    #[must_use]
    pub fn directory(mut self, directory: impl Into<PathBuf>) -> Self {
        self.options = self.options.directory(directory);
        self
    }

    #[must_use]
    pub fn file_name(mut self, file_name: impl Into<String>) -> Self {
        self.options = self.options.file_name(file_name);
        self
    }

    #[must_use]
    pub fn filter(
        mut self,
        name: impl Into<String>,
        extensions: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.options = self.options.filter(name, extensions);
        self
    }

    #[must_use]
    pub fn options(&self) -> &Options {
        &self.options
    }

    pub fn open_file(self) -> Result<Option<PathBuf>> {
        self.open_file_with(&mut SystemBackend)
    }

    pub fn open_files(self) -> Result<Option<Vec<PathBuf>>> {
        self.open_files_with(&mut SystemBackend)
    }

    pub fn open_folder(self) -> Result<Option<PathBuf>> {
        self.open_folder_with(&mut SystemBackend)
    }

    pub fn open_folders(self) -> Result<Option<Vec<PathBuf>>> {
        self.open_folders_with(&mut SystemBackend)
    }

    pub fn save_file(self) -> Result<Option<PathBuf>> {
        self.save_file_with(&mut SystemBackend)
    }

    pub fn open_file_with(self, backend: &mut impl Backend) -> Result<Option<PathBuf>> {
        self.options.validate()?;
        backend.open_file(&self.options)
    }

    pub fn open_files_with(self, backend: &mut impl Backend) -> Result<Option<Vec<PathBuf>>> {
        self.options.validate()?;
        backend.open_files(&self.options)
    }

    pub fn open_folder_with(self, backend: &mut impl Backend) -> Result<Option<PathBuf>> {
        self.options.validate()?;
        backend.open_folder(&self.options)
    }

    pub fn open_folders_with(self, backend: &mut impl Backend) -> Result<Option<Vec<PathBuf>>> {
        self.options.validate()?;
        backend.open_folders(&self.options)
    }

    pub fn save_file_with(self, backend: &mut impl Backend) -> Result<Option<PathBuf>> {
        self.options.validate()?;
        backend.save_file(&self.options)
    }
}

impl Default for FileDialog {
    fn default() -> Self {
        Self::new()
    }
}
