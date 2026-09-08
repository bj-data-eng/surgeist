use std::{collections::VecDeque, path::PathBuf};

use super::{Backend, Options, Result};

/// Fake dialog backend for DSL and command tests.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FakeBackend {
    calls: Vec<Call>,
    open_file: VecDeque<Option<PathBuf>>,
    open_files: VecDeque<Option<Vec<PathBuf>>>,
    open_folder: VecDeque<Option<PathBuf>>,
    open_folders: VecDeque<Option<Vec<PathBuf>>>,
    save_file: VecDeque<Option<PathBuf>>,
}

impl FakeBackend {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn calls(&self) -> &[Call] {
        &self.calls
    }

    pub fn push_open_file(&mut self, result: Option<impl Into<PathBuf>>) {
        self.open_file.push_back(result.map(Into::into));
    }

    pub fn push_open_files(&mut self, result: Option<Vec<impl Into<PathBuf>>>) {
        self.open_files
            .push_back(result.map(|paths| paths.into_iter().map(Into::into).collect()));
    }

    pub fn push_open_folder(&mut self, result: Option<impl Into<PathBuf>>) {
        self.open_folder.push_back(result.map(Into::into));
    }

    pub fn push_open_folders(&mut self, result: Option<Vec<impl Into<PathBuf>>>) {
        self.open_folders
            .push_back(result.map(|paths| paths.into_iter().map(Into::into).collect()));
    }

    pub fn push_save_file(&mut self, result: Option<impl Into<PathBuf>>) {
        self.save_file.push_back(result.map(Into::into));
    }
}

impl Backend for FakeBackend {
    fn open_file(&mut self, options: &Options) -> Result<Option<PathBuf>> {
        self.calls.push(Call::OpenFile(options.clone()));
        Ok(self.open_file.pop_front().unwrap_or(None))
    }

    fn open_files(&mut self, options: &Options) -> Result<Option<Vec<PathBuf>>> {
        self.calls.push(Call::OpenFiles(options.clone()));
        Ok(self.open_files.pop_front().unwrap_or(None))
    }

    fn open_folder(&mut self, options: &Options) -> Result<Option<PathBuf>> {
        self.calls.push(Call::OpenFolder(options.clone()));
        Ok(self.open_folder.pop_front().unwrap_or(None))
    }

    fn open_folders(&mut self, options: &Options) -> Result<Option<Vec<PathBuf>>> {
        self.calls.push(Call::OpenFolders(options.clone()));
        Ok(self.open_folders.pop_front().unwrap_or(None))
    }

    fn save_file(&mut self, options: &Options) -> Result<Option<PathBuf>> {
        self.calls.push(Call::SaveFile(options.clone()));
        Ok(self.save_file.pop_front().unwrap_or(None))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Call {
    OpenFile(Options),
    OpenFiles(Options),
    OpenFolder(Options),
    OpenFolders(Options),
    SaveFile(Options),
}
