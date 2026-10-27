//! Crate-local macros for error types.

/// Defines one documented `is_*` predicate per pattern of an error's private
/// kind.
///
/// `field` names the kind field; `*field` names a boxed one. Every predicate is
/// documented, because the invocation is part of the public surface of the
/// type it appears in.
///
/// The macro has no production caller until the first error type with a
/// private kind arrives; the expectation below then becomes unfulfilled, which
/// the crate denies, so whoever adds that caller removes it.
#[cfg_attr(not(test), expect(unused_macros))]
macro_rules! kind_predicates {
    ($field:ident; $($(#[doc = $doc:expr])+ $name:ident => $pattern:pat,)+) => {
        $(
            $(#[doc = $doc])+
            #[must_use]
            pub const fn $name(&self) -> bool {
                matches!(self.$field, $pattern)
            }
        )+
    };
    (*$field:ident; $($(#[doc = $doc:expr])+ $name:ident => $pattern:pat,)+) => {
        $(
            $(#[doc = $doc])+
            #[must_use]
            pub fn $name(&self) -> bool {
                matches!(*self.$field, $pattern)
            }
        )+
    };
}

#[cfg(test)]
mod tests {
    #[derive(Debug)]
    enum Kind {
        Overflow,
        Refused,
        Detail(u8),
    }

    struct Plain {
        kind: Kind,
    }

    impl Plain {
        kind_predicates! { kind;
            /// Whether the kind is an overflow.
            is_overflow => Kind::Overflow,
            /// Whether the kind is a refusal.
            is_refused => Kind::Refused,
            /// Whether the kind carries a detail above nine.
            is_large_detail => Kind::Detail(10..),
        }
    }

    struct Boxed {
        kind: Box<Kind>,
    }

    impl Boxed {
        kind_predicates! { *kind;
            /// Whether the kind is an overflow.
            is_overflow => Kind::Overflow,
            /// Whether the kind carries any detail.
            is_detailed => Kind::Detail(_),
        }
    }

    #[test]
    fn a_predicate_is_true_only_for_its_own_pattern() {
        let plain = Plain {
            kind: Kind::Overflow,
        };
        assert!(plain.is_overflow());
        assert!(!plain.is_refused());
        assert!(!plain.is_large_detail());
        let refused = Plain {
            kind: Kind::Refused,
        };
        assert!(refused.is_refused());
        assert!(!refused.is_overflow());
    }

    #[test]
    fn a_predicate_pattern_may_look_inside_the_kind() {
        assert!(
            Plain {
                kind: Kind::Detail(10)
            }
            .is_large_detail()
        );
        assert!(
            !Plain {
                kind: Kind::Detail(9)
            }
            .is_large_detail()
        );
    }

    #[test]
    fn predicates_of_an_unboxed_kind_are_usable_in_constants() {
        const {
            assert!(
                Plain {
                    kind: Kind::Overflow
                }
                .is_overflow()
            );
        }
    }

    #[test]
    fn predicates_see_through_a_boxed_kind() {
        let boxed = Boxed {
            kind: Box::new(Kind::Detail(3)),
        };
        assert!(boxed.is_detailed());
        assert!(!boxed.is_overflow());
        let overflow = Boxed {
            kind: Box::new(Kind::Overflow),
        };
        assert!(overflow.is_overflow());
        assert!(!overflow.is_detailed());
    }
}
