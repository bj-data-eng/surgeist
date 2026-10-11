# Servo selectors: matching integration evidence

Research capture: 2026-10-10. Upstream commit `64416a22f9d06d07e108176d724dd20509a8c52d`; manifest version `0.41.0`. This is source inspection, not a compiled prototype, adoption, standards-conformance verdict or security audit.

Source files identify Mozilla Public License 2.0. See [local attribution](../licenses/servo-selectors/NOTICE.md). Excerpts preserve upstream bytes within each fenced block; ranges refer to the original file and hashes identify the full original file. No source is added to a production target.

## Evaluation

The engine is a strong reuse candidate for a Surgeist-owned matching boundary. It already implements combinator backtracking, logical/relational selectors, structural matching and caches. Host-specific pseudo-class and pseudo-element semantics remain caller responsibilities. It does not implement the cascade.

The published surface takes its own selector graph. Its general builder and component-vector constructor are private; a lossless typed bridge from Surgeist CSS is not established. A serialize/reparse route would add a second grammar boundary. Its combinator enum lacks the column combinator present in Surgeist CSS. Matcher entry points return booleans rather than budget/cancellation outcomes. Pointer-shaped opaque identity needs stable snapshot-local storage and cache scoping. External dependency unsafe is distinct from ingested code: an ingested implementation would require safe replacements for layout casts, opaque identity and unchecked unreachable operations.

Recommended next experiment: typed graph lowering without serialization, an immutable DOM read adapter with external invalidation collection, bounded/cancellable matching, and differential semantic tests. Compare published dependency plus upstream improvements against a scoped safe ingestion; do not adopt either until the conversion and semantic gaps are demonstrated.

## selectors/Cargo.toml

[Exact source](https://github.com/servo/stylo/blob/64416a22f9d06d07e108176d724dd20509a8c52d/selectors/Cargo.toml); full-file SHA-256 `bb46c495b7e90ee701534cd197b5d97db4b6732b0ff85f669bfb4b4f05e7a35d`.

Original lines 1–33:

````text
[package]
name = "selectors"
version = "0.41.0"
authors = ["The Servo Project Developers"]
documentation = "https://docs.rs/selectors/"
description = "CSS Selectors matching for Rust"
repository = "https://github.com/servo/stylo"
readme = "README.md"
keywords = ["css", "selectors"]
license = "MPL-2.0"
edition = "2024"

[lib]
name = "selectors"
path = "lib.rs"

[features]
bench = []
to_shmem = ["dep:to_shmem", "dep:to_shmem_derive"]

[dependencies]
bitflags = "2"
cssparser = "0.38"
derive_more = { version = "2", features = ["add", "add_assign"] }
hashbrown = "0.17"
rustc-hash = "2.1.1"
log = "0.4"
precomputed-hash = "0.1"
servo_arc = { workspace = true }
smallvec = "1.0"
to_shmem = { workspace = true, optional = true }
to_shmem_derive = { workspace = true, optional = true }
new_debug_unreachable = "1"
````

## selectors/lib.rs

[Exact source](https://github.com/servo/stylo/blob/64416a22f9d06d07e108176d724dd20509a8c52d/selectors/lib.rs); full-file SHA-256 `02cb2bf0d9b45e9967a8de8663babfc2bbbf7c2d3cf5c93b35d0f3c3ab384647`.

Original lines 1–27:

````text
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// Make |cargo bench| work.
#![cfg_attr(feature = "bench", feature(test))]

pub mod attr;
pub mod bloom;
mod builder;
pub mod context;
pub mod kleene_value;
pub mod matching;
mod nth_index_cache;
pub mod parser;
pub mod relative_selector;
pub mod sink;
pub mod subtree_filter;
mod tree;
pub mod visitor;

pub use crate::nth_index_cache::NthIndexCache;
pub use crate::parser::{Parser, SelectorImpl, SelectorList};
pub use crate::tree::{Element, OpaqueElement};

/// A hash map using the Fx hasher.
pub type FxHashMap<K, V> = hashbrown::HashMap<K, V, rustc_hash::FxBuildHasher>;
````

## selectors/tree.rs

[Exact source](https://github.com/servo/stylo/blob/64416a22f9d06d07e108176d724dd20509a8c52d/selectors/tree.rs); full-file SHA-256 `08f5dd18f589e252f68ea85f653ff52869b0907519577533ec07a1774148f135`.

Original lines 1–200:

````text
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Traits that nodes must implement. Breaks the otherwise-cyclic dependency
//! between layout and style.

use crate::attr::{AttrSelectorOperation, CaseSensitivity, NamespaceConstraint};
use crate::bloom::BloomFilter;
use crate::matching::{ElementSelectorFlags, MatchingContext};
use crate::parser::SelectorImpl;
use std::fmt::Debug;
use std::ptr::NonNull;

/// Opaque representation of an Element, for identity comparisons.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct OpaqueElement(NonNull<()>);

unsafe impl Send for OpaqueElement {}
// This should be safe given that we do not provide a way to recover
// the original reference.
unsafe impl Sync for OpaqueElement {}

impl OpaqueElement {
    /// Creates a new OpaqueElement from an arbitrarily-typed pointer.
    pub fn new<T>(ptr: &T) -> Self {
        // TODO: with rust 1.89, this can just be
        // `OpaqueElement(NonNull::from_ref(ptr).cast())`
        unsafe {
            OpaqueElement(NonNull::new_unchecked(
                ptr as *const T as *const () as *mut (),
            ))
        }
    }

    /// Creates a new OpaqueElement from a type-erased non-null pointer
    pub fn from_ptr(ptr: NonNull<()>) -> Self {
        Self(ptr)
    }

    /// Returns the reference as an untyped pointer.
    #[inline]
    pub fn to_ptr(&self) -> NonNull<()> {
        self.0
    }
}

pub trait Element: Sized + Copy + Clone + Debug {
    type Impl: SelectorImpl;

    /// Converts self into an opaque representation.
    fn opaque(&self) -> OpaqueElement;

    fn parent_element(&self) -> Option<Self>;

    /// Whether the parent node of this element is a shadow root.
    fn parent_node_is_shadow_root(&self) -> bool;

    /// The host of the containing shadow root, if any.
    fn containing_shadow_host(&self) -> Option<Self>;

    /// The parent of a given pseudo-element, after matching a pseudo-element
    /// selector.
    ///
    /// This is guaranteed to be called in a pseudo-element.
    fn pseudo_element_originating_element(&self) -> Option<Self> {
        debug_assert!(self.is_pseudo_element());
        self.parent_element()
    }

    /// Whether we're matching on a pseudo-element.
    fn is_pseudo_element(&self) -> bool;

    /// Skips non-element nodes
    fn prev_sibling_element(&self) -> Option<Self>;

    /// Skips non-element nodes
    fn next_sibling_element(&self) -> Option<Self>;

    /// Skips non-element nodes
    fn first_element_child(&self) -> Option<Self>;

    fn is_html_element_in_html_document(&self) -> bool;

    fn has_local_name(&self, local_name: &<Self::Impl as SelectorImpl>::BorrowedLocalName) -> bool;

    /// Empty string for no namespace
    fn has_namespace(&self, ns: &<Self::Impl as SelectorImpl>::BorrowedNamespaceUrl) -> bool;

    /// Whether this element and the `other` element have the same local name and namespace.
    fn is_same_type(&self, other: &Self) -> bool;

    fn attr_matches(
        &self,
        ns: &NamespaceConstraint<&<Self::Impl as SelectorImpl>::NamespaceUrl>,
        local_name: &<Self::Impl as SelectorImpl>::LocalName,
        operation: &AttrSelectorOperation<&<Self::Impl as SelectorImpl>::AttrValue>,
    ) -> bool;

    fn has_attr_in_no_namespace(
        &self,
        local_name: &<Self::Impl as SelectorImpl>::LocalName,
    ) -> bool {
        self.attr_matches(
            &NamespaceConstraint::Specific(&crate::parser::namespace_empty_string::<Self::Impl>()),
            local_name,
            &AttrSelectorOperation::Exists,
        )
    }

    fn match_non_ts_pseudo_class(
        &self,
        pc: &<Self::Impl as SelectorImpl>::NonTSPseudoClass,
        context: &mut MatchingContext<Self::Impl>,
    ) -> bool;

    fn match_pseudo_element(
        &self,
        pe: &<Self::Impl as SelectorImpl>::PseudoElement,
        context: &mut MatchingContext<Self::Impl>,
    ) -> bool;

    /// Sets selector flags on the elemnt itself or the parent, depending on the
    /// flags, which indicate what kind of work may need to be performed when
    /// DOM state changes.
    fn apply_selector_flags(&self, flags: ElementSelectorFlags);

    /// Whether this element is a `link`.
    fn is_link(&self) -> bool;

    /// Returns whether the element is an HTML <slot> element.
    fn is_html_slot_element(&self) -> bool;

    /// Returns the assigned <slot> element this element is assigned to.
    ///
    /// Necessary for the `::slotted` pseudo-class.
    fn assigned_slot(&self) -> Option<Self> {
        None
    }

    fn has_id(
        &self,
        id: &<Self::Impl as SelectorImpl>::Identifier,
        case_sensitivity: CaseSensitivity,
    ) -> bool;

    fn has_class(
        &self,
        name: &<Self::Impl as SelectorImpl>::Identifier,
        case_sensitivity: CaseSensitivity,
    ) -> bool;

    fn has_custom_state(&self, name: &<Self::Impl as SelectorImpl>::Identifier) -> bool;

    /// Returns the mapping from the `exportparts` attribute in the reverse
    /// direction, that is, in an outer-tree -> inner-tree direction.
    fn imported_part(
        &self,
        name: &<Self::Impl as SelectorImpl>::Identifier,
    ) -> Option<<Self::Impl as SelectorImpl>::Identifier>;

    fn is_part(&self, name: &<Self::Impl as SelectorImpl>::Identifier) -> bool;

    /// Returns whether this element matches `:empty`.
    ///
    /// That is, whether it does not contain any child element or any non-zero-length text node.
    /// See http://dev.w3.org/csswg/selectors-3/#empty-pseudo
    fn is_empty(&self) -> bool;

    /// Returns whether this element matches `:root`,
    /// i.e. whether it is the root element of a document.
    ///
    /// Note: this can be false even if `.parent_element()` is `None`
    /// if the parent node is a `DocumentFragment`.
    fn is_root(&self) -> bool;

    /// Returns whether this element should ignore matching nth child
    /// selector.
    fn ignores_nth_child_selectors(&self) -> bool {
        false
    }

    /// Add hashes unique to this element to the given filter, returning true
    /// if any got added.
    fn add_element_unique_hashes(&self, filter: &mut BloomFilter) -> bool;

    /// The filter summarizing the names present in this element's subtree,
    /// including the element itself. See the `subtree_filter` module.
    ///
    /// The default implementation has every bit set, which never rejects.
    fn subtree_filter(&self) -> u64 {
        u64::MAX
    }

    /// Whether this element's subtree may contain all of the required hashes.
    #[inline]
    fn subtree_may_have_hashes(&self, hashes: u64) -> bool {
        (self.subtree_filter() & hashes) == hashes
    }
}
````

## selectors/parser.rs

[Exact source](https://github.com/servo/stylo/blob/64416a22f9d06d07e108176d724dd20509a8c52d/selectors/parser.rs); full-file SHA-256 `317a73f1041dccccaf8de407b3131eefd5f670b5c00e555e87d57511e20f4e3a`.

Original lines 232–259:

````text
        pub trait SelectorImpl: Clone + Debug + Sized + 'static {
            type ExtraMatchingData<'a>: Sized + Default;
            type AttrValue: $($InSelector)*;
            type Identifier: $($InSelector)* + PrecomputedHash;
            type LocalName: $($InSelector)* + Borrow<Self::BorrowedLocalName> + PrecomputedHash;
            type NamespaceUrl: $($CommonBounds)* + Default + Borrow<Self::BorrowedNamespaceUrl> + PrecomputedHash;
            type NamespacePrefix: $($InSelector)* + Default;
            type BorrowedNamespaceUrl: ?Sized + Eq;
            type BorrowedLocalName: ?Sized + Eq;

            /// non tree-structural pseudo-classes
            /// (see: https://drafts.csswg.org/selectors/#structural-pseudos)
            type NonTSPseudoClass: $($CommonBounds)* + NonTSPseudoClass;

            /// pseudo-elements
            type PseudoElement: $($CommonBounds)* + PseudoElement;

            /// Whether attribute hashes should be collected for filtering
            /// purposes.
            fn should_collect_attr_hash(_name: &Self::LocalName) -> bool {
                false
            }
        }
    }
}

macro_rules! with_bounds {
    ( [ $( $CommonBounds: tt )* ] [ $( $FromStr: tt )* ]) => {
````

Original lines 396–433:

````text
    pub fn from_one(selector: Selector<Impl>) -> Self {
        #[cfg(debug_assertions)]
        let selector_repr = unsafe { *(&selector as *const _ as *const usize) };
        let list = Self(ThinArcUnion::from_first(selector.into_data()));
        #[cfg(debug_assertions)]
        debug_assert_eq!(
            selector_repr,
            unsafe { *(&list as *const _ as *const usize) },
            "We rely on the same bit representation for the single selector variant"
        );
        list
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_iter(mut iter: impl ExactSizeIterator<Item = Selector<Impl>>) -> Self {
        if iter.len() == 1 {
            Self::from_one(iter.next().unwrap())
        } else {
            Self(ThinArcUnion::from_second(ThinArc::from_header_and_iter(
                (),
                iter,
            )))
        }
    }

    #[inline]
    pub fn slice(&self) -> &[Selector<Impl>] {
        match self.0.borrow() {
            ArcUnionBorrow::First(..) => {
                // SAFETY: see from_one.
                let selector: &Selector<Impl> = unsafe { std::mem::transmute(self) };
                std::slice::from_ref(selector)
            },
            ArcUnionBorrow::Second(list) => list.get().slice(),
        }
    }

    #[inline]
````

Original lines 1129–1145:

````text
    pub(crate) fn from_vec(
        vec: Vec<Component<Impl>>,
        specificity: u32,
        flags: SelectorFlags,
    ) -> Self {
        let mut builder = SelectorBuilder::default();
        for component in vec.into_iter() {
            if let Some(combinator) = component.as_combinator() {
                builder.push_combinator(combinator);
            } else {
                builder.push_simple_selector(component);
            }
        }
        let spec = SpecificityAndFlags { specificity, flags };
        Selector(builder.build_with_specificity_and_flags(spec, ParseRelative::No))
    }

````

Original lines 1613–1640:

````text
        // See if there are more sequences. If so, skip any non-ancestor sequences.
        if !self.0.next_sequence()?.is_ancestor() {
            self.0.skip_until_ancestor();
        }
        self.0.next()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "to_shmem", derive(ToShmem))]
pub enum Combinator {
    Child,        //  >
    Descendant,   // space
    NextSibling,  // +
    LaterSibling, // ~
    /// A dummy combinator we use to the left of pseudo-elements.
    ///
    /// It serializes as the empty string, and acts effectively as a child
    /// combinator in most cases.  If we ever actually start using a child
    /// combinator for this, we will need to fix up the way hashes are computed
    /// for revalidation selectors.
    PseudoElement,
    /// Another combinator used for ::slotted(), which represent the jump from
    /// a node to its assigned slot.
    SlotAssignment,
    /// Another combinator used for `::part()`, which represents the jump from
    /// the part to the containing shadow host.
    Part,
````

Original lines 2070–2177:

````text
#[derive(Clone, Eq, PartialEq)]
#[cfg_attr(feature = "to_shmem", derive(ToShmem))]
#[cfg_attr(feature = "to_shmem", shmem(no_bounds))]
pub enum Component<Impl: SelectorImpl> {
    LocalName(LocalName<Impl>),

    ID(#[cfg_attr(feature = "to_shmem", shmem(field_bound))] Impl::Identifier),
    Class(#[cfg_attr(feature = "to_shmem", shmem(field_bound))] Impl::Identifier),

    AttributeInNoNamespaceExists {
        #[cfg_attr(feature = "to_shmem", shmem(field_bound))]
        local_name: Impl::LocalName,
        local_name_lower: Impl::LocalName,
    },
    // Used only when local_name is already lowercase.
    AttributeInNoNamespace {
        local_name: Impl::LocalName,
        operator: AttrSelectorOperator,
        #[cfg_attr(feature = "to_shmem", shmem(field_bound))]
        value: Impl::AttrValue,
        case_sensitivity: ParsedCaseSensitivity,
    },
    // Use a Box in the less common cases with more data to keep size_of::<Component>() small.
    AttributeOther(Box<AttrSelectorWithOptionalNamespace<Impl>>),

    ExplicitUniversalType,
    ExplicitAnyNamespace,

    ExplicitNoNamespace,
    DefaultNamespace(#[cfg_attr(feature = "to_shmem", shmem(field_bound))] Impl::NamespaceUrl),
    Namespace(
        #[cfg_attr(feature = "to_shmem", shmem(field_bound))] Impl::NamespacePrefix,
        #[cfg_attr(feature = "to_shmem", shmem(field_bound))] Impl::NamespaceUrl,
    ),

    /// Pseudo-classes
    Negation(SelectorList<Impl>),
    Root,
    Empty,
    Scope,
    /// :scope added implicitly into scoped rules (i.e. In `@scope`) not
    /// explicitly using `:scope` or `&` selectors.
    ///
    /// https://drafts.csswg.org/css-cascade-6/#scoped-rules
    ///
    /// Unlike the normal `:scope` selector, this does not add any specificity.
    /// See https://github.com/w3c/csswg-drafts/issues/10196
    ImplicitScope,
    ParentSelector,
    Nth(NthSelectorData),
    NthOf(NthOfSelectorData<Impl>),
    NonTSPseudoClass(#[cfg_attr(feature = "to_shmem", shmem(field_bound))] Impl::NonTSPseudoClass),
    /// The ::slotted() pseudo-element:
    ///
    /// https://drafts.csswg.org/css-scoping/#slotted-pseudo
    ///
    /// The selector here is a compound selector, that is, no combinators.
    ///
    /// NOTE(emilio): This should support a list of selectors, but as of this
    /// writing no other browser does, and that allows them to put ::slotted()
    /// in the rule hash, so we do that too.
    ///
    /// See https://github.com/w3c/csswg-drafts/issues/2158
    Slotted(Selector<Impl>),
    /// The `::part` pseudo-element.
    ///   https://drafts.csswg.org/css-shadow-parts/#part
    Part(#[cfg_attr(feature = "to_shmem", shmem(field_bound))] Box<[Impl::Identifier]>),
    /// The `:host` pseudo-class:
    ///
    /// https://drafts.csswg.org/css-scoping/#host-selector
    ///
    /// NOTE(emilio): This should support a list of selectors, but as of this
    /// writing no other browser does, and that allows them to put :host()
    /// in the rule hash, so we do that too.
    ///
    /// See https://github.com/w3c/csswg-drafts/issues/2158
    Host(Option<Selector<Impl>>),
    /// The `:where` pseudo-class.
    ///
    /// https://drafts.csswg.org/selectors/#zero-matches
    ///
    /// The inner argument is conceptually a SelectorList, but we move the
    /// selectors to the heap to keep Component small.
    Where(SelectorList<Impl>),
    /// The `:is` pseudo-class.
    ///
    /// https://drafts.csswg.org/selectors/#matches-pseudo
    ///
    /// Same comment as above re. the argument.
    Is(SelectorList<Impl>),
    /// The `:has` pseudo-class.
    ///
    /// https://drafts.csswg.org/selectors/#has-pseudo
    ///
    /// Same comment as above re. the argument.
    Has(Box<[RelativeSelector<Impl>]>),
    /// An invalid selector inside :is() / :where().
    Invalid(Arc<String>),
    /// An implementation-dependent pseudo-element selector.
    PseudoElement(#[cfg_attr(feature = "to_shmem", shmem(field_bound))] Impl::PseudoElement),

    Combinator(Combinator),

    /// Used only for relative selectors, which starts with a combinator
    /// (With an implied descendant combinator if not specified).
    ///
    /// https://drafts.csswg.org/selectors-4/#typedef-relative-selector
    RelativeSelectorAnchor,
````

## selectors/matching.rs

[Exact source](https://github.com/servo/stylo/blob/64416a22f9d06d07e108176d724dd20509a8c52d/selectors/matching.rs); full-file SHA-256 `d2863afd22d6afad8aa7ca339a91d2f14e159cb5f421b4e9e4bf5887ecaa4cb3`.

Original lines 121–147:

````text

#[inline(always)]
pub fn matches_selector_list<E>(
    selector_list: &SelectorList<E::Impl>,
    element: E,
    context: &mut MatchingContext<E::Impl>,
) -> bool
where
    E: Element,
{
    // This is pretty much any(..) but manually inlined because the compiler
    // refuses to do so from querySelector / querySelectorAll.
    for selector in selector_list.slice() {
        let matches = matches_selector(selector, 0, None, element, context);
        if matches {
            return true;
        }
    }

    false
}

/// Given the ancestor hashes from a selector, see if the current element,
/// represented by the bloom filter, has a chance of matching at all.
#[inline(always)]
pub fn selector_may_match(hashes: &AncestorHashes, bf: &BloomFilter) -> bool {
    // Check the first three hashes. Note that we can check for zero before
````

Original lines 1328–1347:

````text
                matches_complex_selector_list(list.slice(), element, context, rightmost)
            });
        },
        Component::Negation(ref list) => {
            return context.shared.nest_for_negation(|context| {
                !matches_complex_selector_list(list.slice(), element, context, rightmost)
            });
        },
        Component::Has(ref relative_selectors) => {
            return match_relative_selectors(
                relative_selectors,
                element,
                context.shared,
                rightmost,
            );
        },
        Component::Combinator(_) => unsafe {
            debug_unreachable!("Shouldn't try to selector-match combinators")
        },
        Component::RelativeSelectorAnchor => {
````

## selectors/context.rs

[Exact source](https://github.com/servo/stylo/blob/64416a22f9d06d07e108176d724dd20509a8c52d/selectors/context.rs); full-file SHA-256 `3d21edee153827d4d5bb3e2df7a8dda1f07ab537555d990cf4846c4f35a50add`.

Original lines 130–180:

````text
/// Data associated with the matching process for a element.  This context is
/// used across many selectors for an element, so it's not appropriate for
/// transient data that applies to only a single selector.
pub struct MatchingContext<'a, Impl>
where
    Impl: SelectorImpl,
{
    /// Input with the matching mode we should use when matching selectors.
    matching_mode: MatchingMode,
    /// Input with the bloom filter used to fast-reject selectors.
    pub bloom_filter: Option<&'a BloomFilter>,
    /// The element which is going to match :scope pseudo-class. It can be
    /// either one :scope element, or the scoping element.
    ///
    /// Note that, although in theory there can be multiple :scope elements,
    /// in current specs, at most one is specified, and when there is one,
    /// scoping element is not relevant anymore, so we use a single field for
    /// them.
    ///
    /// When this is None, :scope will match the root element.
    ///
    /// See https://drafts.csswg.org/selectors-4/#scope-pseudo
    pub scope_element: Option<OpaqueElement>,

    /// The current shadow host we're collecting :host rules for.
    pub current_host: Option<OpaqueElement>,

    /// Controls how matching for links is handled.
    visited_handling: VisitedHandlingMode,

    /// Whether we're currently matching a featureless element.
    pub featureless: bool,

    /// The current nesting level of selectors that we're matching.
    nesting_level: usize,

    /// Whether we're inside a negation or not.
    in_negation: bool,

    /// An optional hook function for checking whether a pseudo-element
    /// should match when matching_mode is ForStatelessPseudoElement.
    #[allow(clippy::type_complexity)]
    pub pseudo_element_matching_fn: Option<&'a dyn Fn(&Impl::PseudoElement) -> bool>,

    /// Extra implementation-dependent matching data.
    pub extra_data: Impl::ExtraMatchingData<'a>,

    /// The current element we're anchoring on for evaluating the relative selector.
    current_relative_selector_anchor: Option<OpaqueElement>,

    quirks_mode: QuirksMode,
````

Original lines 380–395:

````text
    #[inline]
    pub fn classes_and_ids_case_sensitivity(&self) -> CaseSensitivity {
        self.classes_and_ids_case_sensitivity
    }

    /// Runs F with a deeper nesting level.
    #[inline]
    pub fn nest<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        self.nesting_level += 1;
        let result = f(self);
        self.nesting_level -= 1;
        result
    }
````
