//! HTML877d DOMStringMap name transformations over exact UTF-16.

const DATA: [u16; 5] = [0x64, 0x61, 0x74, 0x61, 0x2d];
const DASH: u16 = 0x2d;
const CASE_OFFSET: u16 = 0x20;

fn is_lower(unit: u16) -> bool {
    (0x61..=0x7a).contains(&unit)
}

fn is_upper(unit: u16) -> bool {
    (0x41..=0x5a).contains(&unit)
}

fn attribute_width(unit: u16) -> usize {
    1 + usize::from(is_upper(unit))
}

/// An eligible attribute's borrowed dataset property spelling.
///
/// Its suffix remains in the original attribute. The HTML mapping ignores
/// the attribute namespace and prefix, so the caller supplies the local name.
#[derive(Clone, Copy, Debug)]
pub struct HtmlDatasetPropertyName<'a> {
    suffix: &'a [u16],
    len: usize,
}

impl<'a> HtmlDatasetPropertyName<'a> {
    /// Qualify an exact local name and calculate the number of generated units.
    /// Accepts the empty suffix and excludes ASCII uppercase in the suffix.
    pub fn from_attribute_with<E>(
        local_name: &'a [u16],
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        step()?;
        let Some(suffix) = local_name.get(DATA.len()..) else {
            return Ok(None);
        };
        for (actual, expected) in local_name.iter().zip(DATA) {
            step()?;
            if *actual != expected {
                return Ok(None);
            }
        }
        let mut len = 0;
        let mut after_dash = false;
        for unit in suffix {
            step()?;
            let unit = *unit;
            if is_upper(unit) {
                return Ok(None);
            }
            len += usize::from(!(after_dash && is_lower(unit)));
            after_dash = unit == DASH;
        }
        Ok(Some(Self { suffix, len }))
    }

    /// The exact length of the generated UTF-16 spelling.
    pub const fn len(self) -> usize {
        self.len
    }

    /// Whether the generated name holds zero units.
    pub const fn is_empty(self) -> bool {
        self.len == 0
    }

    /// Visit the transformed units in order, without allocating a name.
    /// Each source-unit read and produced unit is charged before use.
    pub fn visit_with<E>(
        self,
        mut step: impl FnMut() -> Result<(), E>,
        mut write: impl FnMut(u16) -> Result<(), E>,
    ) -> Result<(), E> {
        let mut dash = false;
        for unit in self.suffix {
            step()?;
            let unit = *unit;
            if dash {
                step()?;
                if is_lower(unit) {
                    write(unit - CASE_OFFSET)?;
                    dash = false;
                    continue;
                }
                write(DASH)?;
            }
            dash = unit == DASH;
            if !dash {
                step()?;
                write(unit)?;
            }
        }
        if dash {
            step()?;
            write(DASH)?;
        }
        Ok(())
    }
}

/// The virtual null-namespace local name generated from a dataset property.
///
/// A DOM owner validates its local-name rules if it sets this spelling.
#[derive(Clone, Copy, Debug)]
pub struct HtmlDatasetAttributeName<'a> {
    property: &'a [u16],
    len: usize,
}

impl<'a> HtmlDatasetAttributeName<'a> {
    /// Qualify a property for setting or looking up. A hyphen followed by
    /// lowercase ASCII is invalid on setting and cannot identify a dataset pair.
    pub fn for_set_with<E>(
        property: &'a [u16],
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        step()?;
        let mut len = DATA.len();
        let mut after_dash = false;
        for unit in property {
            step()?;
            let unit = *unit;
            if after_dash && is_lower(unit) {
                return Ok(None);
            }
            // A well-formed u16 slice occupies at most isize::MAX bytes.
            // Even expanding every unit to two leaves room for the prefix.
            len += attribute_width(unit);
            after_dash = unit == DASH;
        }
        Ok(Some(Self { property, len }))
    }

    /// Measure the deletion spelling after Web IDL has selected the supported
    /// property. The deletion algorithm itself has no syntax-rejection step.
    pub fn for_delete_with<E>(
        property: &'a [u16],
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<Self, E> {
        step()?;
        let mut len = DATA.len();
        for unit in property {
            step()?;
            len += attribute_width(*unit);
        }
        Ok(Self { property, len })
    }

    /// The exact length of the generated UTF-16 spelling.
    pub const fn len(self) -> usize {
        self.len
    }

    /// Whether the generated spelling holds zero units. The prefix is always present.
    pub const fn is_empty(self) -> bool {
        self.len == 0
    }

    /// Visit the generated local-name units, including the `data-` prefix.
    pub fn visit_with<E>(
        self,
        step: impl FnMut() -> Result<(), E>,
        mut write: impl FnMut(u16) -> Result<(), E>,
    ) -> Result<(), E> {
        self.scan_with(step, |unit| {
            write(unit)?;
            Ok(true)
        })?;
        Ok(())
    }

    /// Test a candidate local name with no allocated intermediate spelling.
    pub fn matches_local_name_with<E>(
        self,
        candidate: &[u16],
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<bool, E> {
        step()?;
        if candidate.len() != self.len {
            return Ok(false);
        }
        let mut units = candidate.iter();
        self.scan_with(step, |unit| Ok(units.next() == Some(&unit)))
    }

    fn scan_with<E>(
        self,
        mut step: impl FnMut() -> Result<(), E>,
        mut emit: impl FnMut(u16) -> Result<bool, E>,
    ) -> Result<bool, E> {
        for unit in DATA {
            step()?;
            if !emit(unit)? {
                return Ok(false);
            }
        }
        for unit in self.property {
            step()?;
            let mut unit = *unit;
            if is_upper(unit) {
                step()?;
                if !emit(DASH)? {
                    return Ok(false);
                }
                unit += CASE_OFFSET;
            }
            step()?;
            if !emit(unit)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
