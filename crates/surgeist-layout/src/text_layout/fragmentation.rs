//! Checked per-binding family custody; Text remains the source/fragment owner.
use super::{AdmittedInlineBoxes, InlineBoxFragmentationDemand, TextParagraphBinding};
use std::sync::Arc;
use surgeist_text::{PreparedInlineBoxFamily, ScopeAlignmentLimitKind};

/// One optional whole-input allocation, shared by associations and operations.
/// The absent state allocates nothing for wholly non-demanding input.
#[derive(Clone, Debug, Default)]
pub(crate) struct InlineBoxFamilies {
    slots: Option<Arc<InlineBoxFamilySlots>>,
}
#[derive(Debug)]
struct InlineBoxFamilySlots {
    slots: Vec<Option<PreparedInlineBoxFamily>>,
    // A captured witness from an actual successful funded family, used to
    // pre-admit comparison discovery. It never activates operation funding.
    scope: Option<surgeist_text::CapturedScope>,
}
impl PartialEq for InlineBoxFamilies {
    fn eq(&self, other: &Self) -> bool {
        match (&self.slots, &other.slots) {
            (None, None) => true,
            (Some(left), Some(right)) => Arc::ptr_eq(left, right) || left.slots == right.slots,
            _ => false,
        }
    }
}
impl Eq for InlineBoxFamilies {}
impl InlineBoxFamilies {
    pub(crate) fn same_owner(&self, other: &Self) -> bool {
        match (&self.slots, &other.slots) {
            (None, None) => true,
            (Some(left), Some(right)) => Arc::ptr_eq(left, right),
            _ => false,
        }
    }

    pub(crate) fn slots(&self) -> &[Option<PreparedInlineBoxFamily>] {
        self.slots
            .as_deref()
            .map_or(&[], |owner| owner.slots.as_slice())
    }

    pub(crate) fn policy_scope(&self) -> Option<&surgeist_text::CapturedScope> {
        self.slots.as_ref().and_then(|owner| owner.scope.as_ref())
    }

    pub(crate) fn state(&self, binding: usize) -> AdmittedInlineBoxes {
        match self.slots().get(binding).and_then(Option::as_ref) {
            None => AdmittedInlineBoxes::NotRequested,
            Some(family) => AdmittedInlineBoxes::CompleteBodies(family.clone()),
        }
    }

    /// Called only after all constructor operations have succeeded off-side.
    pub(crate) fn from_slots<Node>(
        slots: Vec<Option<PreparedInlineBoxFamily>>,
        paragraphs: &[TextParagraphBinding<Node>],
        mut inspect: impl FnMut() -> surgeist_text::Result<()>,
    ) -> surgeist_text::Result<Self> {
        if slots.len() != paragraphs.len() {
            return Err(foreign_family());
        }
        let mut scope: Option<surgeist_text::CapturedScope> = None;
        for (slot, paragraph) in slots.iter().zip(paragraphs) {
            inspect()?;
            match (paragraph.inline_box_fragments, slot) {
                (InlineBoxFragmentationDemand::NotRequested, None) => {}
                (InlineBoxFragmentationDemand::CompleteBodies, Some(family))
                    if family.prepared_text() == paragraph.prepared =>
                {
                    if family.scope_alignment_retention().is_some() {
                        let prepared = family.prepared_text();
                        let actual = prepared
                            .captured_owner()
                            .ok_or_else(foreign_family)?
                            .domain();
                        let limits = actual.scope_alignment_limits().ok_or_else(foreign_family)?;
                        let replace = match &scope {
                            Some(old) => {
                                limits.structural_work()
                                    < old
                                        .scope_alignment_limits()
                                        .ok_or_else(foreign_family)?
                                        .structural_work()
                            }
                            None => true,
                        };
                        if replace {
                            scope = Some(actual);
                        }
                    }
                }
                _ => return Err(foreign_family()),
            }
        }
        Ok(Self {
            slots: Some(Arc::new(InlineBoxFamilySlots { slots, scope })),
        })
    }

    pub(crate) fn validate_line(
        expected: &AdmittedInlineBoxes,
        line: &surgeist_text::FinalizedLine,
    ) -> surgeist_text::Result<()> {
        if line.inline_box_family() != expected.family() {
            return Err(foreign_family());
        }
        Ok(())
    }

    /// Pre-admit the actual descriptor preflight plus the conservative native
    /// declaration-key bound before pure semantic Eq. Ordinary Native families
    /// with no applicable feature retention do not acquire a universal quota.
    pub(crate) fn admit_comparison<S: crate::LayoutScalar>(
        &self,
        other: &Self,
        limits: surgeist_text::ScopeAlignmentLimits,
        resources: &super::scope_alignment::ScopeAlignmentResources<S>,
    ) -> surgeist_text::Result<()> {
        use super::scope_alignment::ScopeAlignmentFacts;
        match (&self.slots, &other.slots) {
            (Some(left), Some(right)) if Arc::ptr_eq(left, right) => return Ok(()),
            (Some(_), Some(_)) if self.slots().len() == other.slots().len() => {}
            _ => return Ok(()),
        }
        let visits = self.slots().len().checked_mul(2).ok_or_else(|| {
            super::scope_alignment::overflow(ScopeAlignmentLimitKind::StructuralWork)
        })?;
        ScopeAlignmentFacts::visit(resources, limits, visits)?;
        for (left, right) in self.slots().iter().zip(other.slots()) {
            if let (Some(left), Some(right)) = (left, right) {
                let left = left
                    .scope_alignment_retention()
                    .map_or(0, |value| value.records());
                let right = right
                    .scope_alignment_retention()
                    .map_or(0, |value| value.records());
                ScopeAlignmentFacts::visit(resources, limits, left.min(right))?;
            }
        }
        Ok(())
    }
}

pub(crate) fn foreign_family() -> surgeist_text::Error {
    surgeist_text::Error::new(
        surgeist_text::ErrorCode::InvalidTextProtocol,
        "inline box family does not belong to this Layout binding",
    )
    .with_detail(surgeist_text::ErrorDetail::InvalidTextProtocol {
        reason: surgeist_text::ProtocolError::InlineBoxFragmentation(
            surgeist_text::InlineBoxFragmentationError::ForeignFamily,
        ),
    })
}

type ResultOf<Tree, T> = crate::LayoutResultOf<
    <Tree as crate::Traverse>::Node,
    T,
    <Tree as crate::Traverse>::Scalar,
    <Tree as crate::LayoutTree>::MeasureError,
>;

fn text_error<Tree: crate::LayoutTree>(
    node: Tree::Node,
    error: surgeist_text::Error,
) -> crate::LayoutErrorOf<Tree::Node, Tree::Scalar, Tree::MeasureError> {
    crate::LayoutErrorOf::new(
        crate::LayoutErrorSiteOf::Node(node),
        crate::LayoutOperation::TextCapture,
        crate::LayoutErrorKindOf::Text(error),
    )
}
fn allocation<Tree: crate::LayoutTree>(
    node: Tree::Node,
) -> crate::LayoutErrorOf<Tree::Node, Tree::Scalar, Tree::MeasureError> {
    crate::LayoutErrorOf::new(
        crate::LayoutErrorSiteOf::Node(node),
        crate::LayoutOperation::TextCapture,
        crate::LayoutErrorKindOf::Allocation,
    )
}

/// Only a genuinely requested input allocates this canonical initialized slot
/// buffer. Its Vec header becomes the one shared Arc payload, not another
/// embedded-header charge in each row or resource clone.
pub(super) fn reserve_slots<Tree: crate::LayoutTree>(
    root: Tree::Node,
    paragraphs: &[TextParagraphBinding<Tree::Node>],
    admission: &mut super::scope_alignment::ScopeAlignmentAdmission<Tree::Scalar>,
) -> ResultOf<Tree, Vec<Option<PreparedInlineBoxFamily>>> {
    if admission.feature_active() {
        let work = admission.work();
        work.retain_header::<InlineBoxFamilySlots>()
            .map_err(|error| text_error::<Tree>(root, error))?;
        let (slots, held) = work
            .copy_map(paragraphs, |_| None)
            .map_err(|error| text_error::<Tree>(root, error))?;
        work.retain_existing(held)
            .map_err(|error| text_error::<Tree>(root, error))?;
        Ok(slots)
    } else {
        let mut slots = Vec::new();
        slots
            .try_reserve_exact(paragraphs.len())
            .map_err(|_| allocation::<Tree>(root))?;
        slots.resize_with(paragraphs.len(), || None);
        Ok(slots)
    }
}

fn extra_native_source<Node: Copy + Eq>(
    binding: &TextParagraphBinding<Node>,
    proof: &crate::inline_wrapping::WrappingScopeProof<Node>,
) -> surgeist_text::Result<Option<surgeist_text::InlineBoxSource>> {
    let crate::inline_wrapping::WrappingScopeProof::NativePair {
        flow, start, end, ..
    } = proof
    else {
        return Ok(None);
    };
    let Some(owner) = binding.prepared.captured_owner() else {
        return Ok(None);
    };
    if *flow != binding.container || start.paragraph().domain() != owner.domain() {
        return Ok(None);
    }
    let source = binding
        .prepared
        .native_inline_box(start.clone(), end.clone())?;
    // Exact captured nominations are already borrowed by the Text family;
    // they do not become duplicate native overlay declarations.
    Ok(source.captured_scope().is_none().then_some(source))
}

fn validate_inventory<Tree: crate::LayoutTree>(
    binding: &TextParagraphBinding<Tree::Node>,
    proofs: &[crate::inline_wrapping::WrappingScopeProof<Tree::Node>],
    inventory: &[crate::InlineChildOrderInput<Tree::Node>],
    mut inspect: impl FnMut() -> surgeist_text::Result<()>,
) -> ResultOf<Tree, ()> {
    for row in inventory {
        inspect().map_err(|error| text_error::<Tree>(binding.container, error))?;
        if row.formatting_context != binding.container {
            continue;
        }
        for item in std::iter::once(&row.parent).chain(row.children.iter().map(|row| &row.box_)) {
            inspect().map_err(|error| text_error::<Tree>(binding.container, error))?;
            let crate::GeneratedBoxRef::Inline(crate::InlineBoxRef::NativePair { start, end }) =
                item
            else {
                continue;
            };
            let mut found = false;
            for proof in proofs {
                inspect().map_err(|error| text_error::<Tree>(binding.container, error))?;
                found = match proof {
                    crate::inline_wrapping::WrappingScopeProof::NativePair {
                        flow, pair, ..
                    } => *flow == binding.container && *pair == (*start, *end),
                    crate::inline_wrapping::WrappingScopeProof::Captured(proof) => {
                        proof.flow == binding.container && proof.pair == Some((*start, *end))
                    }
                };
                if found {
                    break;
                }
            }
            if !found {
                return Err(crate::inline_wrapping::wrapping_error(
                    binding.container,
                    crate::WrappingScopeError::CaptureAssociation,
                ));
            }
        }
    }
    Ok(())
}

/// Build each demanded family once against the already checked canonical
/// native inventory. All earlier families remain actual census roots; no
/// product-reference scratch or independent receipt sum is constructed.
pub(super) fn prepare_families<Tree: crate::LayoutTree>(
    paragraphs: &[TextParagraphBinding<Tree::Node>],
    proofs: &[crate::inline_wrapping::WrappingScopeProof<Tree::Node>],
    inventory: &[crate::InlineChildOrderInput<Tree::Node>],
    resources: &super::scope_alignment::ScopeAlignmentResources<Tree::Scalar>,
    mut slots: Vec<Option<PreparedInlineBoxFamily>>,
) -> ResultOf<Tree, InlineBoxFamilies> {
    use super::scope_alignment::{ScopeAlignmentFacts, allocation_buffer};
    let mut validation_limits = None;
    for (index, binding) in paragraphs.iter().enumerate() {
        if binding.inline_box_fragments == InlineBoxFragmentationDemand::NotRequested {
            continue;
        }
        let mut native_boxes = Vec::new();
        let operation = resources
            .begin_inline_box_family(&binding.prepared, &native_boxes, &slots)
            .map_err(|error| text_error::<Tree>(binding.container, error))?;
        let inspect = || match &operation {
            Some(operation) => ScopeAlignmentFacts::visit(resources, operation.limits(), 1),
            None => Ok(()),
        };
        validate_inventory::<Tree>(binding, proofs, inventory, inspect)?;
        let mut count = 0usize;
        for proof in proofs {
            inspect().map_err(|error| text_error::<Tree>(binding.container, error))?;
            if extra_native_source(binding, proof)
                .map_err(|error| text_error::<Tree>(binding.container, error))?
                .is_some()
            {
                count += 1;
            }
        }
        let lease = match &operation {
            Some(operation) => {
                let (values, lease) = allocation_buffer::<surgeist_text::InlineBoxSource>(
                    resources, operation, count,
                )
                .map_err(|error| text_error::<Tree>(binding.container, error))?;
                native_boxes = values;
                Some(lease)
            }
            None => {
                native_boxes
                    .try_reserve_exact(count)
                    .map_err(|_| allocation::<Tree>(binding.container))?;
                None
            }
        };
        for proof in proofs {
            inspect().map_err(|error| text_error::<Tree>(binding.container, error))?;
            if let Some(source) = extra_native_source(binding, proof)
                .map_err(|error| text_error::<Tree>(binding.container, error))?
            {
                native_boxes.push(source);
            }
        }
        // Census now owns this exact input backing in its operation input. The
        // previous Layout lease is released before that transfer, without a
        // double debit; its real vector moves into Text below.
        drop(operation);
        drop(lease);
        let operation = resources
            .begin_inline_box_family(&binding.prepared, &native_boxes, &slots)
            .map_err(|error| text_error::<Tree>(binding.container, error))?;
        let allowance = operation
            .as_ref()
            .map(|operation| operation.allowance())
            .transpose()
            .map_err(|error| text_error::<Tree>(binding.container, error))?;
        let family = binding
            .prepared
            .inline_box_family(native_boxes, allowance)
            .map_err(|error| text_error::<Tree>(binding.container, error))?;
        if let Some(operation) = &operation {
            let limits = operation.limits();
            if validation_limits.is_none_or(|old: surgeist_text::ScopeAlignmentLimits| {
                limits.structural_work() < old.structural_work()
            }) {
                validation_limits = Some(limits);
            }
        }
        drop(operation);
        slots[index] = Some(family);
    }
    InlineBoxFamilies::from_slots(slots, paragraphs, || match validation_limits {
        Some(limits) => ScopeAlignmentFacts::visit(resources, limits, 1),
        None => Ok(()),
    })
    .map_err(|error| text_error::<Tree>(paragraphs[0].container, error))
}
