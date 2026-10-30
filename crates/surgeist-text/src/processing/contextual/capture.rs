//! Canonical identities and event observations from one immutable capture.

use std::{
    cmp::Ordering,
    fmt,
    hash::{Hash, Hasher},
    sync::Arc,
};

use super::{
    CapturedDomainData, ScopeAlignmentBoundary, ScopeAlignmentLimits, ScopeAlignmentUsage,
    ScopeCapture, ScopeFontReference, SourceProcessingEvent, SourceProcessingTopology,
};
use crate::{ParagraphControlKind, SourcePosition, Style};

macro_rules! captured_handle {
    ($($name:ident),+ $(,)?) => {$ (
        #[doc = "An owner-issued occurrence in one immutable contextual capture."]
        #[doc = "Clones preserve identity; equal ordinals from another capture do not."]
        #[derive(Clone)]
        pub struct $name {
            pub(crate) capture: Arc<ScopeCapture>,
            pub(crate) index: usize,
        }
        impl PartialEq for $name {
            fn eq(&self, other: &Self) -> bool {
                self.index == other.index && Arc::ptr_eq(&self.capture, &other.capture)
            }
        }
        impl Eq for $name {}
        impl Hash for $name {
            fn hash<H: Hasher>(&self, state: &mut H) {
                Arc::as_ptr(&self.capture).hash(state);
                self.index.hash(state);
            }
        }
        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_struct(stringify!($name)).field("ordinal", &self.index).finish_non_exhaustive()
            }
        }
    )+};
}
captured_handle!(
    CapturedScope,
    CapturedControl,
    CapturedParagraph,
    CapturedEventPosition
);

/// Actual scope role, independent of whether any preparation emits output.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ScopeRole {
    Domain,
    Inline,
}

/// The actual source-defined kind and atomic association of a content domain.
///
/// An atomic receipt identifies the nominated occurrence, never a nearby box at
/// the same source position. These values observe a capture; they author no input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CapturedDomainDefinition {
    Block,
    AtomicBlock { atomic: CapturedEventPosition },
    OutOfFlowBlock { atomic: CapturedEventPosition },
}

/// One genuine observation in a domain's complete captured flow.
///
/// Atomic content has its own flow, independent of its capture scheduling.
/// Plain child Block entry and exit remain in their parent's flow at their
/// actual positions. These observations create no text, controls or geometry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CapturedFlowEvent {
    ParagraphStart(CapturedParagraph),
    ParagraphEnd(CapturedParagraph),
    Origin {
        paragraph: CapturedParagraph,
        position: CapturedEventPosition,
    },
    Atomic {
        paragraph: CapturedParagraph,
        position: CapturedEventPosition,
    },
    Control(CapturedControl),
    BlockStart(CapturedScope),
    BlockEnd(CapturedScope),
}

impl CapturedEventPosition {
    /// Whether these positions belong to the exact same immutable event stream.
    #[must_use]
    pub fn same_capture(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.capture, &other.capture)
    }
}
impl PartialOrd for CapturedEventPosition {
    /// Orders genuine mixed events only within one capture. Foreign captures
    /// have no shared traversal order, even when their numeric offsets coincide.
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.same_capture(other)
            .then(|| self.index.cmp(&other.index))
    }
}

impl CapturedScope {
    /// Capture-local diagnostic ordinal. It cannot authorize an association:
    /// genuine scope identity is this immutable capture plus the ordinal.
    #[must_use]
    pub const fn capture_ordinal(&self) -> usize {
        self.index
    }

    /// Actual early inline formatting classification. Domains and absent
    /// producer evidence return None, never an inferred Initial value.
    #[must_use]
    pub fn alignment_boundary(&self) -> Option<ScopeAlignmentBoundary> {
        self.capture
            .scope_alignment_fact(self.index)
            .and_then(|fact| fact.alignment_boundary)
    }

    /// Supplied same-scope font/reference context; no lookup or measurement.
    /// Context alone neither makes an alignment target nor activates a font.
    #[must_use]
    pub fn font_reference(&self) -> Option<ScopeFontReference> {
        self.capture
            .scope_alignment_fact(self.index)
            .and_then(|fact| fact.font_reference)
    }

    /// Immutable limits on the entire capture's alignment feature, not just
    /// this scope. Ordinary non-demanding captures may have no feature limits.
    #[must_use]
    pub fn scope_alignment_limits(&self) -> Option<ScopeAlignmentLimits> {
        self.capture
            .scope_alignment
            .as_ref()
            .map(|facts| facts.limits())
    }

    /// Completed early capture accounting, not later preparation/selection
    /// usage. Reading it does not resolve fonts or mutate a shared ledger.
    #[must_use]
    pub fn scope_alignment_usage(&self) -> Option<ScopeAlignmentUsage> {
        self.capture
            .scope_alignment
            .as_ref()
            .map(|facts| facts.usage())
    }

    /// The complete owner-issued flow on a domain root; inline scopes return None.
    ///
    /// This is a stable projection of the admitted original events, independent
    /// of output selection and supplied consumer bindings. Unknown inline
    /// topology does not erase supplied flow. Child atomic/OOF internals are
    /// opaque; ordinary child Block boundaries retain their real parent slots.
    /// Inline entry/exit and connection facts remain separately observable and
    /// do not invent additional flow participants.
    pub fn flow_events(&self) -> Option<impl Iterator<Item = CapturedFlowEvent> + '_> {
        let fact = &self.capture.scopes[self.index];
        fact.definition.as_ref()?;
        Some(self.capture.flow_orders[fact.domain].iter().map(|&index| {
            let event = &self.capture.events[index];
            let paragraph = |index| CapturedParagraph {
                capture: Arc::clone(&self.capture),
                index,
            };
            match event.event {
                SourceProcessingEvent::EnterParagraph(index) => {
                    CapturedFlowEvent::ParagraphStart(paragraph(index))
                }
                SourceProcessingEvent::ExitParagraph(index) => {
                    CapturedFlowEvent::ParagraphEnd(paragraph(index))
                }
                SourceProcessingEvent::Origin { .. } => CapturedFlowEvent::Origin {
                    paragraph: paragraph(
                        event
                            .paragraph
                            .expect("contextual admission proves original paragraph membership"),
                    ),
                    position: self.position(index),
                },
                SourceProcessingEvent::Atomic { .. } => CapturedFlowEvent::Atomic {
                    paragraph: paragraph(
                        event
                            .paragraph
                            .expect("contextual admission proves atomic paragraph membership"),
                    ),
                    position: self.position(index),
                },
                SourceProcessingEvent::Control { control, .. } => {
                    CapturedFlowEvent::Control(CapturedControl {
                        capture: Arc::clone(&self.capture),
                        index: control,
                    })
                }
                SourceProcessingEvent::EnterScope(index) => CapturedFlowEvent::BlockStart(Self {
                    capture: Arc::clone(&self.capture),
                    index,
                }),
                SourceProcessingEvent::ExitScope(index) => CapturedFlowEvent::BlockEnd(Self {
                    capture: Arc::clone(&self.capture),
                    index,
                }),
                SourceProcessingEvent::Connection(_) => {
                    unreachable!("flow indexing excludes nonparticipating connections")
                }
            }
        }))
    }

    /// The actual role retained before analysis or output selection.
    #[must_use]
    pub fn role(&self) -> ScopeRole {
        if self.capture.scopes[self.index].definition.is_some() {
            ScopeRole::Domain
        } else {
            ScopeRole::Inline
        }
    }

    /// Actual global parent, including across a nested content domain.
    #[must_use]
    pub fn parent(&self) -> Option<Self> {
        self.capture.scopes[self.index].parent.map(|index| Self {
            capture: Arc::clone(&self.capture),
            index,
        })
    }

    /// Canonical owning domain root; a domain root returns itself.
    #[must_use]
    pub fn domain(&self) -> Self {
        Self {
            capture: Arc::clone(&self.capture),
            index: self.capture.domain_scopes[self.capture.scopes[self.index].domain],
        }
    }

    /// Actual definition on domain roots; inline scopes return None.
    #[must_use]
    pub fn domain_definition(&self) -> Option<CapturedDomainDefinition> {
        self.capture.scopes[self.index]
            .definition
            .map(|definition| match definition {
                CapturedDomainData::Block => CapturedDomainDefinition::Block,
                CapturedDomainData::AtomicBlock(atomic) => CapturedDomainDefinition::AtomicBlock {
                    atomic: self.position(self.capture.atomic_orders[atomic]),
                },
                CapturedDomainData::OutOfFlowBlock(atomic) => {
                    CapturedDomainDefinition::OutOfFlowBlock {
                        atomic: self.position(self.capture.atomic_orders[atomic]),
                    }
                }
            })
    }

    /// Whether the producer supplied complete topology for this scope's domain.
    #[must_use]
    pub fn domain_topology(&self) -> SourceProcessingTopology {
        if self.capture.complete[self.capture.scopes[self.index].domain] {
            SourceProcessingTopology::Complete
        } else {
            SourceProcessingTopology::Unknown
        }
    }

    /// Captured Style, honestly absent when the producer did not supply it.
    /// Observing a dormant scope does not resolve or validate fonts.
    #[must_use]
    pub fn style(&self) -> Option<&Style> {
        self.capture.scopes[self.index].style.as_ref()
    }

    /// Genuine scope entry in the sole global mixed-event stream.
    #[must_use]
    pub fn enter_position(&self) -> CapturedEventPosition {
        self.position(self.capture.scopes[self.index].enter)
    }

    /// Genuine scope exit in the sole global mixed-event stream.
    #[must_use]
    pub fn exit_position(&self) -> CapturedEventPosition {
        self.position(self.capture.scopes[self.index].exit)
    }

    /// Genuine global Start control, including in a different paragraph.
    /// Both endpoint observations are present together or absent together.
    #[must_use]
    pub fn entry_control(&self) -> Option<CapturedControl> {
        self.capture.scopes[self.index]
            .boundaries
            .map(|pair| CapturedControl {
                capture: Arc::clone(&self.capture),
                index: pair.start,
            })
    }

    /// Genuine global End control, including in a different paragraph.
    #[must_use]
    pub fn exit_control(&self) -> Option<CapturedControl> {
        self.capture.scopes[self.index]
            .boundaries
            .map(|pair| CapturedControl {
                capture: Arc::clone(&self.capture),
                index: pair.end,
            })
    }

    fn position(&self, index: usize) -> CapturedEventPosition {
        CapturedEventPosition {
            capture: Arc::clone(&self.capture),
            index,
        }
    }
}

impl CapturedParagraph {
    /// Actual canonical content domain of this paragraph occurrence.
    #[must_use]
    pub fn domain(&self) -> CapturedScope {
        CapturedScope {
            capture: Arc::clone(&self.capture),
            index: self.capture.domain_scopes[self.capture.paragraphs[self.index].domain],
        }
    }

    /// Genuine paragraph entry, independent of source offsets or output.
    #[must_use]
    pub fn enter_position(&self) -> CapturedEventPosition {
        CapturedEventPosition {
            capture: Arc::clone(&self.capture),
            index: self.capture.paragraphs[self.index].enter,
        }
    }

    /// Genuine paragraph exit, retaining nested-domain holes.
    #[must_use]
    pub fn exit_position(&self) -> CapturedEventPosition {
        CapturedEventPosition {
            capture: Arc::clone(&self.capture),
            index: self.capture.paragraphs[self.index].exit,
        }
    }

    /// Own-domain overlapping scopes and actual structural domain ancestors.
    ///
    /// Each extent is an intersection in global event order, not a contiguous
    /// source range or geometry envelope. Foreign content domains remain holes;
    /// occurrence receipts and canonical domains establish actual participation.
    /// This observation is independent of Style and generated output.
    pub fn scope_intersections(&self) -> impl Iterator<Item = CapturedScopeIntersection> + '_ {
        let paragraph = &self.capture.paragraphs[self.index];
        self.capture
            .scopes
            .iter()
            .enumerate()
            .filter(move |(index, scope)| {
                if scope.domain == paragraph.domain {
                    return scope.enter < paragraph.exit && paragraph.enter < scope.exit;
                }
                let mut ancestor = Some(self.capture.domain_scopes[paragraph.domain]);
                while let Some(current) = ancestor {
                    if current == *index {
                        return true;
                    }
                    ancestor = self.capture.scopes[current].parent;
                }
                false
            })
            .map(|(scope, _)| CapturedScopeIntersection {
                capture: Arc::clone(&self.capture),
                paragraph: self.index,
                scope,
            })
    }
}

/// An authenticated paragraph/scope event intersection, not an invented edge.
#[derive(Clone, Debug)]
pub struct CapturedScopeIntersection {
    capture: Arc<ScopeCapture>,
    paragraph: usize,
    scope: usize,
}
impl CapturedScopeIntersection {
    /// Actual canonical paragraph issuing this intersection.
    #[must_use]
    pub fn paragraph(&self) -> CapturedParagraph {
        CapturedParagraph {
            capture: Arc::clone(&self.capture),
            index: self.paragraph,
        }
    }

    /// The one authentic global scope, including structural ancestors.
    #[must_use]
    pub fn scope(&self) -> CapturedScope {
        CapturedScope {
            capture: Arc::clone(&self.capture),
            index: self.scope,
        }
    }

    /// Later of the genuine paragraph and scope entry events.
    #[must_use]
    pub fn enter_position(&self) -> CapturedEventPosition {
        CapturedEventPosition {
            capture: Arc::clone(&self.capture),
            index: self.capture.paragraphs[self.paragraph]
                .enter
                .max(self.capture.scopes[self.scope].enter),
        }
    }

    /// Earlier of the genuine paragraph and scope exit events.
    #[must_use]
    pub fn exit_position(&self) -> CapturedEventPosition {
        CapturedEventPosition {
            capture: Arc::clone(&self.capture),
            index: self.capture.paragraphs[self.paragraph]
                .exit
                .min(self.capture.scopes[self.scope].exit),
        }
    }
}

impl CapturedControl {
    /// Actual enclosing source scope at this control's admitted mixed event.
    ///
    /// This retains source custody; it does not invent a captured scope for a
    /// native-only box or identify an allocated selected frame.
    #[must_use]
    pub fn scope(&self) -> CapturedScope {
        let order = self.capture.controls[self.index].order;
        CapturedScope {
            capture: Arc::clone(&self.capture),
            index: self.capture.events[order]
                .before
                .expect("admitted control event has an enclosing scope"),
        }
    }

    /// Actual paragraph containing this control event, even if it emits no text.
    #[must_use]
    pub fn paragraph(&self) -> CapturedParagraph {
        CapturedParagraph {
            capture: Arc::clone(&self.capture),
            index: self.capture.controls[self.index].paragraph,
        }
    }

    /// Actual semantic role validated by contextual capture.
    #[must_use]
    pub fn kind(&self) -> &ParagraphControlKind {
        &self.capture.controls[self.index].fact.kind
    }

    /// Original source anchor; coincident anchors never establish identity.
    #[must_use]
    pub fn source_position(&self) -> SourcePosition {
        SourcePosition::from_unchecked(self.capture.controls[self.index].fact.at)
    }

    /// Genuine control occurrence in the mixed-event stream.
    #[must_use]
    pub fn event_position(&self) -> CapturedEventPosition {
        CapturedEventPosition {
            capture: Arc::clone(&self.capture),
            index: self.capture.controls[self.index].order,
        }
    }
}

captured_handle!(CapturedComputedLineBoundary);

/// Canonical target of one input computed mandatory cause.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CapturedComputedLineBoundaryTarget {
    Scope {
        scope: CapturedScope,
        side: super::SourceBoxSide,
    },
    Endpoint(CapturedControl),
    Atomic {
        position: CapturedEventPosition,
        side: super::SourceBoxSide,
    },
}

impl CapturedComputedLineBoundary {
    /// The actual requested occurrence, preserving computed-cause provenance.
    #[must_use]
    pub fn target(&self) -> CapturedComputedLineBoundaryTarget {
        use super::ComputedLineBoundaryInput;
        match self.capture.computed[self.index].target {
            ComputedLineBoundaryInput::Scope { scope, side } => {
                CapturedComputedLineBoundaryTarget::Scope {
                    scope: CapturedScope {
                        capture: Arc::clone(&self.capture),
                        index: scope,
                    },
                    side,
                }
            }
            ComputedLineBoundaryInput::Endpoint { control } => {
                CapturedComputedLineBoundaryTarget::Endpoint(CapturedControl {
                    capture: Arc::clone(&self.capture),
                    index: control,
                })
            }
            ComputedLineBoundaryInput::Atomic { atomic, side } => {
                CapturedComputedLineBoundaryTarget::Atomic {
                    position: CapturedEventPosition {
                        capture: Arc::clone(&self.capture),
                        index: self.capture.atomic_orders[atomic],
                    },
                    side,
                }
            }
        }
    }
    /// The canonical formatting domain owning this cause.
    #[must_use]
    pub fn domain(&self) -> CapturedScope {
        CapturedScope {
            capture: Arc::clone(&self.capture),
            index: self.capture.domain_scopes[self.capture.computed[self.index].domain],
        }
    }
    #[must_use]
    pub fn paragraph(&self) -> CapturedParagraph {
        CapturedParagraph {
            capture: Arc::clone(&self.capture),
            index: self.capture.computed[self.index].paragraph,
        }
    }
    /// Exact original target anchor; this inserts no source character.
    #[must_use]
    pub fn source_position(&self) -> SourcePosition {
        SourcePosition::from_unchecked(self.capture.computed[self.index].source)
    }
    /// Original event immediately preceding this authenticated canonical cut.
    #[must_use]
    pub fn preceding_event(&self) -> Option<CapturedEventPosition> {
        self.capture.computed[self.index]
            .capture_cut
            .checked_sub(1)
            .map(|index| CapturedEventPosition {
                capture: Arc::clone(&self.capture),
                index,
            })
    }
    /// Original event immediately following this authenticated canonical cut.
    #[must_use]
    pub fn following_event(&self) -> Option<CapturedEventPosition> {
        let index = self.capture.computed[self.index].capture_cut;
        (index < self.capture.events.len()).then(|| CapturedEventPosition {
            capture: Arc::clone(&self.capture),
            index,
        })
    }
}

impl crate::ProcessedSource {
    fn contextual_capture(&self) -> Option<&Arc<ScopeCapture>> {
        self.paragraphs
            .first()?
            .data
            .scope_topology
            .as_ref()
            .map(|topology| &topology.capture)
    }
    /// Resolves an ordinal from this operation's actual source-scope inventory.
    /// The returned handle, rather than the ordinal, carries capture identity.
    #[must_use]
    pub fn captured_scope(&self, input_ordinal: usize) -> Option<CapturedScope> {
        let capture = self.contextual_capture()?;
        capture.scopes.get(input_ordinal)?;
        Some(CapturedScope {
            capture: Arc::clone(capture),
            index: input_ordinal,
        })
    }
    #[must_use]
    pub fn captured_control(&self, input_ordinal: usize) -> Option<CapturedControl> {
        let capture = self.contextual_capture()?;
        capture.controls.get(input_ordinal)?;
        Some(CapturedControl {
            capture: Arc::clone(capture),
            index: input_ordinal,
        })
    }
    #[must_use]
    pub fn captured_origin(&self, input_ordinal: usize) -> Option<CapturedEventPosition> {
        let capture = self.contextual_capture()?;
        Some(CapturedEventPosition {
            capture: Arc::clone(capture),
            index: *capture.origin_orders.get(input_ordinal)?,
        })
    }
    #[must_use]
    pub fn captured_atomic(&self, input_ordinal: usize) -> Option<CapturedEventPosition> {
        let capture = self.contextual_capture()?;
        Some(CapturedEventPosition {
            capture: Arc::clone(capture),
            index: *capture.atomic_orders.get(input_ordinal)?,
        })
    }
    #[must_use]
    pub fn captured_paragraph(&self, input_ordinal: usize) -> Option<CapturedParagraph> {
        let capture = self.contextual_capture()?;
        capture.paragraphs.get(input_ordinal)?;
        Some(CapturedParagraph {
            capture: Arc::clone(capture),
            index: input_ordinal,
        })
    }
    #[must_use]
    pub fn captured_computed_line_boundary(
        &self,
        input_ordinal: usize,
    ) -> Option<CapturedComputedLineBoundary> {
        let capture = self.contextual_capture()?;
        let index = *capture.computed_input_rows.get(input_ordinal)?;
        Some(CapturedComputedLineBoundary {
            capture: Arc::clone(capture),
            index,
        })
    }
}
