//! The one definition of an opaque identity.
//!
//! An identity is an allocation with no payload: a value equals only itself and
//! its clones, never another value made the same way. Producers and consumers
//! compare identities to bind facts to the producer or revision that issued them.

/// Defines a public identity newtype over `Arc<()>`.
///
/// The type derives `Clone` and `Debug` and compares by allocation, so a clone
/// equals its source and every other value differs. Flags add behavior:
/// `fresh` adds a public `fresh()` that issues a distinct identity, and `hash`
/// hashes the allocation address (consistent with equality). Without `fresh`
/// the defining module is the only issuer.
macro_rules! identity {
    (@flag $name:ident, fresh) => {
        impl $name {
            /// Issues a distinct identity. Clone it to share the identity.
            pub fn fresh() -> Self {
                Self(::std::sync::Arc::new(()))
            }
        }
    };
    (@flag $name:ident, hash) => {
        impl ::std::hash::Hash for $name {
            fn hash<H: ::std::hash::Hasher>(&self, state: &mut H) {
                ::std::hash::Hash::hash(&::std::sync::Arc::as_ptr(&self.0), state);
            }
        }
    };
    ($(#[$meta:meta])* $name:ident $(, $flag:ident)* $(,)?) => {
        $(#[$meta])*
        #[derive(Clone, Debug)]
        pub struct $name(::std::sync::Arc<()>);
        impl PartialEq for $name {
            fn eq(&self, other: &Self) -> bool {
                ::std::sync::Arc::ptr_eq(&self.0, &other.0)
            }
        }
        impl Eq for $name {}
        $(identity!(@flag $name, $flag);)*
    };
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    identity!(
        /// An identity that anyone may issue and that can key a set.
        Issued,
        fresh,
        hash
    );
    identity!(
        /// An identity with no public issuer.
        Held
    );

    #[test]
    fn a_clone_equals_its_source_and_every_other_identity_differs() {
        let first = Issued::fresh();
        let second = Issued::fresh();
        assert_eq!(first, first.clone());
        assert_ne!(first, second);
    }

    #[test]
    fn hashing_follows_equality() {
        let first = Issued::fresh();
        let second = Issued::fresh();
        let mut set = HashSet::new();
        assert!(set.insert(first.clone()));
        assert!(!set.insert(first));
        assert!(set.insert(second));
    }

    #[test]
    fn identities_without_fresh_are_issued_by_their_owner() {
        let owner = Held(::std::sync::Arc::new(()));
        assert_eq!(owner, owner.clone());
        assert_ne!(owner, Held(::std::sync::Arc::new(())));
    }
}
