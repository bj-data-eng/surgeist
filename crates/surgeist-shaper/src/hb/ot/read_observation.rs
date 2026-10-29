//! Qualification of reads actually consumed by native lookup execution.
use core::cell::Cell;
use read_fonts::{
    tables::layout::{ClassDef, CoverageTable},
    types::GlyphId,
    ReadError,
};

pub(crate) struct Reads {
    active: bool,
    failed: Cell<bool>,
    work: Cell<Option<usize>>,
}
impl Reads {
    pub(crate) fn new(active: bool) -> Self {
        Self {
            active,
            failed: Cell::new(false),
            work: Cell::new(Some(0)),
        }
    }
    pub(crate) fn active(&self) -> bool {
        self.active
    }
    pub(crate) fn failed(&self) -> bool {
        self.failed.get()
    }
    pub(crate) fn work(&self) -> Option<usize> {
        self.work.get()
    }
    pub(crate) fn check(&self, qualified: bool) {
        if self.active {
            self.work
                .set(self.work.get().and_then(|n| n.checked_add(1)));
            if !qualified {
                self.failed.set(true);
            }
        }
    }
    pub(crate) fn read<T>(&self, value: Result<T, ReadError>) -> Option<T> {
        self.check(value.is_ok());
        value.ok()
    }
    /// Only selected required indices belong here; nullable offsets and search
    /// misses retain their separate native absence semantics.
    pub(crate) fn required<T>(&self, value: Option<T>) -> Option<T> {
        self.check(value.is_some());
        value
    }
    pub(crate) fn nullable<T>(&self, value: Option<Result<T, ReadError>>) -> Option<T> {
        value.and_then(|v| self.read(v))
    }
    pub(crate) fn view(&self, len: usize, declared: usize) {
        self.check(len == declared);
    }
    pub(crate) fn slice<'a, T>(&self, value: &'a [T], declared: usize) -> &'a [T] {
        self.view(value.len(), declared);
        value
    }
    pub(crate) fn coverage(&self, value: Result<CoverageTable<'_>, ReadError>) {
        if !self.active {
            return;
        }
        match value {
            Ok(CoverageTable::Format1(t)) => {
                self.view(t.glyph_array().len(), usize::from(t.glyph_count()));
            }
            Ok(CoverageTable::Format2(t)) => {
                self.view(t.range_records().len(), usize::from(t.range_count()));
            }
            Err(_) => self.check(false),
        }
    }
    pub(crate) fn coverage_read<'a>(
        &self,
        value: Result<CoverageTable<'a>, ReadError>,
    ) -> Option<CoverageTable<'a>> {
        let value = self.read(value)?;
        if self.active {
            self.coverage(Ok(value.clone()));
        }
        Some(value)
    }
    pub(crate) fn class(&self, value: Result<ClassDef<'_>, ReadError>, gid: GlyphId) {
        if !self.active {
            return;
        }
        match value {
            Ok(ClassDef::Format1(t)) => {
                let index = gid.to_u32().checked_sub(t.start_glyph_id().to_u32());
                if index.is_some_and(|i| i < u32::from(t.glyph_count())) {
                    self.view(t.class_value_array().len(), usize::from(t.glyph_count()));
                }
            }
            Ok(ClassDef::Format2(t)) => self.view(
                t.class_range_records().len(),
                usize::from(t.class_range_count()),
            ),
            Err(_) => self.check(false),
        }
    }
}
