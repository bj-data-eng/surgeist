use crate::{Error, ErrorCode, Result, Selector};

#[derive(Clone, Debug, PartialEq)]
pub struct ScopeSelectorList {
    selectors: Vec<Selector>,
}

impl ScopeSelectorList {
    pub fn try_new(selectors: impl IntoIterator<Item = Selector>) -> Result<Self> {
        let selectors = selectors.into_iter().collect::<Vec<_>>();
        if selectors.is_empty() {
            return Err(Error::new(
                ErrorCode::InvalidSelector,
                "scope selector list cannot be empty",
            ));
        }
        Ok(Self { selectors })
    }

    #[must_use]
    pub fn selectors(&self) -> &[Selector] {
        &self.selectors
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuleScope {
    roots: Option<ScopeSelectorList>,
    limits: Option<ScopeSelectorList>,
}

impl RuleScope {
    pub fn try_new(
        roots: Option<ScopeSelectorList>,
        limits: Option<ScopeSelectorList>,
    ) -> Result<Self> {
        // Selector does not currently expose a pseudo-element predicate. Root
        // lowering is responsible for excluding pseudo-element scoped roots
        // until that predicate exists here.
        Ok(Self { roots, limits })
    }

    #[must_use]
    pub const fn roots(&self) -> Option<&ScopeSelectorList> {
        self.roots.as_ref()
    }

    #[must_use]
    pub const fn limits(&self) -> Option<&ScopeSelectorList> {
        self.limits.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scope_selector_lists_reject_empty_lists() {
        assert!(ScopeSelectorList::try_new([]).is_err());
    }

    #[test]
    fn rule_scopes_store_optional_roots_and_limits() {
        let roots = ScopeSelectorList::try_new([Selector::class("card").unwrap()]).unwrap();
        let limits = ScopeSelectorList::try_new([Selector::class("limit").unwrap()]).unwrap();

        let scope = RuleScope::try_new(Some(roots), Some(limits)).unwrap();

        assert_eq!(scope.roots().unwrap().selectors().len(), 1);
        assert_eq!(scope.limits().unwrap().selectors().len(), 1);
    }
}
