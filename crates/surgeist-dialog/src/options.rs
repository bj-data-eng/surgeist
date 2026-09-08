use std::path::PathBuf;

use super::{Error, ErrorCode, Result};

/// File extension filter shown by native file dialogs where supported.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Filter {
    pub name: String,
    pub extensions: Vec<String>,
}

impl Filter {
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        extensions: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            name: name.into(),
            extensions: extensions.into_iter().map(Into::into).collect(),
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(Error::new(
                ErrorCode::InvalidOptions,
                "dialog filter name must not be empty",
            ));
        }
        if self.extensions.is_empty() {
            return Err(Error::new(
                ErrorCode::InvalidOptions,
                "dialog filter must include at least one extension",
            ));
        }
        for extension in &self.extensions {
            if extension.trim().is_empty() {
                return Err(Error::new(
                    ErrorCode::InvalidOptions,
                    "dialog filter extension must not be empty",
                ));
            }
        }
        Ok(())
    }
}

/// Options shared by open/save/folder native file dialogs.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Options {
    pub title: Option<String>,
    pub directory: Option<PathBuf>,
    pub file_name: Option<String>,
    pub filters: Vec<Filter>,
}

impl Options {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    #[must_use]
    pub fn directory(mut self, directory: impl Into<PathBuf>) -> Self {
        self.directory = Some(directory.into());
        self
    }

    #[must_use]
    pub fn file_name(mut self, file_name: impl Into<String>) -> Self {
        self.file_name = Some(file_name.into());
        self
    }

    #[must_use]
    pub fn filter(
        mut self,
        name: impl Into<String>,
        extensions: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.filters.push(Filter::new(name, extensions));
        self
    }

    pub fn validate(&self) -> Result<()> {
        for filter in &self.filters {
            filter.validate()?;
        }
        Ok(())
    }
}
