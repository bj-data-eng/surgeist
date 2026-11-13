//! Shadow and template-contents associations.
//!
//! A shadow root is an unowned shadow contents fragment attached to a host element; a template
//! element owns a contents fragment, or refers to shadow contents instead. Each association adds
//! or releases the edge between a host or template and a fragment, moves the fragment's subtree
//! to its new depth, and sets the owner of the fragment and the entry of the association table.
//!
//! The table is sorted by owner and kind, so an owner finds its entries by a search, and a node
//! that has no entry does not search: its row says so. An entry is added in order, and the entries
//! after it move, which the operation charges.

use super::depth::{DepthPlan, Placement, Request};
use super::shape::{ordinary_owned_contents, unowned_fragment};
use super::*;
use crate::internal::construction::rows::AssocTable;

/// The association a change writes.
#[derive(Clone, Copy, Debug)]
enum Write {
    /// `contents` becomes the shadow root of `host`.
    AttachShadow { host: NodeIx, contents: NodeIx },
    /// `contents` becomes the shadow root of `host`, and `template` refers to it in place of the
    /// contents `released` that it owned.
    AttachTemplateShadow {
        template: NodeIx,
        host: NodeIx,
        contents: NodeIx,
        released: NodeIx,
    },
    /// `template` refers to `contents` in place of the contents `released` that it owned.
    SetTemplateShadowContents {
        template: NodeIx,
        contents: NodeIx,
        released: NodeIx,
    },
}
impl Write {
    /// The shadow entry the write adds to the table, as its owner and contents.
    fn added(self) -> Option<(NodeIx, NodeIx)> {
        match self {
            Self::AttachShadow { host, contents }
            | Self::AttachTemplateShadow { host, contents, .. } => Some((host, contents)),
            Self::SetTemplateShadowContents { .. } => None,
        }
    }
}

impl HtmlConstructionStorage {
    /// Checks that `host` is an element with no shadow root, and that `contents` is an unowned
    /// shadow contents fragment.
    fn check_shadow(&self, host: NodeIx, contents: NodeIx) -> Result<(), HtmlConstructionError> {
        let row = self.row(host)?;
        if row.kind != RowKind::Element {
            return Err(HtmlConstructionCause::ShadowHostMustBeElement.error());
        }
        if row.flags.has(NodeFlags::HAS_ASSOC) && self.arena.shadow_of(host).is_some() {
            return Err(HtmlConstructionCause::ShadowAlreadyAttached.error());
        }
        self.row(contents)?;
        unowned_fragment(&self.arena, contents, true)
    }

    /// Attaches the unowned shadow contents fragment `contents` as the declarative shadow root of
    /// `host`, which must be an element without one and must not lie inside `contents`.
    pub fn attach_shadow<A: HtmlRecordAdmission>(
        &mut self,
        host: &HtmlBuilderNode,
        contents: &HtmlBuilderNode,
        admission: &mut A,
    ) -> Result<(), HtmlConstructionRecordError<A::Stop>> {
        let host = self.operand(host, admission)?;
        let contents = self.operand(contents, admission)?;
        self.check_shadow(host, contents).map_err(faulted)?;
        self.require_acyclic(host, contents, admission)?;
        let mut after = self.arena.usage;
        after.edges = plus(after.edges, 1).map_err(faulted)?;
        let depths = self.plan_depths(
            &[Request {
                root: contents,
                placement: Placement::Under(host),
            }],
            None,
            true,
            admission,
        )?;
        self.commit_association(
            Write::AttachShadow { host, contents },
            depths,
            after,
            admission,
        )
    }

    /// Attaches `contents` as the shadow root of `host` and turns the `template` element's own
    /// contents into a reference to it, releasing the contents fragment that the template owned.
    /// `contents` must differ from the contents the template owned.
    pub fn attach_template_shadow<A: HtmlRecordAdmission>(
        &mut self,
        template: &HtmlBuilderNode,
        host: &HtmlBuilderNode,
        contents: &HtmlBuilderNode,
        admission: &mut A,
    ) -> Result<(), HtmlConstructionRecordError<A::Stop>> {
        let template = self.operand(template, admission)?;
        let host = self.operand(host, admission)?;
        let contents = self.operand(contents, admission)?;
        let released = ordinary_owned_contents(&self.arena, template).map_err(faulted)?;
        self.check_shadow(host, contents).map_err(faulted)?;
        if contents == released {
            return Err(fault(HtmlConstructionCause::InvalidContentsReference));
        }
        self.require_acyclic(host, contents, admission)?;
        let after = self.arena.usage;
        // The host of a template shadow may sit inside the contents the template gives up, so its
        // depth is measured after the change, by a walk. A record is inside the contents only
        // through a child of theirs, so a host cannot be inside contents that have none.
        let placement = if self.row(released).map_err(faulted)?.is_childless() {
            Placement::Under(host)
        } else {
            Placement::AfterHost(host)
        };
        let depths = self.plan_depths(
            &[
                Request {
                    root: released,
                    placement: Placement::Detached,
                },
                Request {
                    root: contents,
                    placement,
                },
            ],
            Some(released),
            false,
            admission,
        )?;
        self.commit_association(
            Write::AttachTemplateShadow {
                template,
                host,
                contents,
                released,
            },
            depths,
            after,
            admission,
        )
    }

    /// Makes the contents of the `template` element a reference to the already-created shadow
    /// contents `contents`, releasing the contents fragment that the template owned. `contents`
    /// must differ from the contents the template owned.
    pub fn set_template_shadow_contents<A: HtmlRecordAdmission>(
        &mut self,
        template: &HtmlBuilderNode,
        contents: &HtmlBuilderNode,
        admission: &mut A,
    ) -> Result<(), HtmlConstructionRecordError<A::Stop>> {
        let template = self.operand(template, admission)?;
        let contents = self.operand(contents, admission)?;
        let released = ordinary_owned_contents(&self.arena, template).map_err(faulted)?;
        self.row(contents).map_err(faulted)?;
        unowned_fragment(&self.arena, contents, true).map_err(faulted)?;
        if contents == released {
            return Err(fault(HtmlConstructionCause::InvalidContentsReference));
        }
        let mut after = self.arena.usage;
        after.edges = minus(after.edges, 1).map_err(faulted)?;
        let depths = self.plan_depths(
            &[Request {
                root: released,
                placement: Placement::Detached,
            }],
            Some(released),
            false,
            admission,
        )?;
        self.commit_association(
            Write::SetTemplateShadowContents {
                template,
                contents,
                released,
            },
            depths,
            after,
            admission,
        )
    }

    fn commit_association<A: HtmlRecordAdmission>(
        &mut self,
        write: Write,
        depths: DepthPlan,
        mut after: HtmlConstructionUsage,
        admission: &mut A,
    ) -> Result<(), HtmlConstructionRecordError<A::Stop>> {
        let before = self.arena.usage;
        after.depth = depths.depth;
        // The entries the table moves to open the place of the entry the write adds.
        let (added, shifted) = match write.added() {
            Some((host, _)) => {
                let table = &self.arena.associations;
                let at = table.place(host, BuildAssocKind::Shadow);
                (1usize, table.len().saturating_sub(at))
            }
            None => (0, 0),
        };
        let table_writes = if added == 0 {
            0
        } else {
            plus(1, shifted).map_err(faulted)?
        };
        // The owner of the contents, the flag of the host, the entry of the table that the write
        // changes, the entries it adds and moves, and for a template shadow the released
        // contents' owner.
        let fields = match write {
            Write::AttachShadow { .. } => 2,
            Write::AttachTemplateShadow { .. } => 4,
            Write::SetTemplateShadowContents { .. } => 2,
        };
        let copied = plus(
            plus(
                times(fields, size_of::<u32>()).map_err(faulted)?,
                times(table_writes, size_of::<BuildAssoc>()).map_err(faulted)?,
            )
            .map_err(faulted)?,
            size_of::<NodeFlags>(),
        )
        .map_err(faulted)?;
        let growth = if added == 0 {
            None
        } else {
            self.associations_growth(plus(self.arena.associations.len(), 1).map_err(faulted)?)
        };
        let index_growth = self.index_growth(depths.levels().map_err(faulted)?);
        let (depth_visits, depth_writes) = depths.work().map_err(faulted)?;
        let usage = HtmlConstructionChange { before, after };
        admission
            .record_largest(HtmlResource::TreeDepth, wide(after.depth))
            .map_err(stopped)?;
        admission.admit_change(self, usage).map_err(stopped)?;
        // The writes are charged once the limits admitted the change.
        Work {
            record_visits: depth_visits,
            provider_calls: usize::from(growth.is_some())
                + index_growth.into_iter().flatten().count(),
            copied_bytes: plus(
                plus(
                    copied,
                    times(depth_writes, size_of::<u32>()).map_err(faulted)?,
                )
                .map_err(faulted)?,
                size_of::<HtmlConstructionUsage>(),
            )
            .map_err(faulted)?,
            record_writes: plus(
                plus(plus(fields, table_writes).map_err(faulted)?, depth_writes)
                    .map_err(faulted)?,
                1,
            )
            .map_err(faulted)?,
            ..Work::default()
        }
        .charge(admission)?;
        let chosen = growth
            .map(|growth| Self::admit_bank(growth, admission))
            .transpose()?;
        let index_chosen = Self::admit_index(index_growth, admission)?;
        if let (Some(growth), Some(chosen)) = (growth, chosen) {
            self.grow(growth, chosen, admission, |storage, additional| {
                storage.reserve_association_bank(additional)
            })?;
        }
        self.grow_index(index_growth, index_chosen, admission)?;
        // Every record the writes name exists, and the table has room for the entry it adds,
        // checked before the last step that can refuse.
        self.verify_association(write).map_err(faulted)?;
        admission.record_change(self, usage).map_err(stopped)?;
        depths.apply(&mut self.arena).map_err(faulted)?;
        self.apply_association(write).map_err(faulted)?;
        self.arena.usage = after;
        Ok(())
    }

    /// Checks that every record `write` names exists, that the entries it changes are in the
    /// table, and that the table has room for the entry it adds.
    fn verify_association(&self, write: Write) -> Result<(), HtmlConstructionError> {
        let names: [Option<NodeIx>; 4] = match write {
            Write::AttachShadow { host, contents } => [Some(host), Some(contents), None, None],
            Write::AttachTemplateShadow {
                template,
                host,
                contents,
                released,
            } => [Some(template), Some(host), Some(contents), Some(released)],
            Write::SetTemplateShadowContents {
                template,
                contents,
                released,
            } => [Some(template), Some(contents), Some(released), None],
        };
        for name in names.into_iter().flatten() {
            self.row(name)?;
        }
        let table = &self.arena.associations;
        if write.added().is_some() && table.len() >= table.capacity() {
            return Err(HtmlConstructionCause::CapacityMismatch.error());
        }
        if let Write::AttachTemplateShadow { template, .. }
        | Write::SetTemplateShadowContents { template, .. } = write
        {
            self.arena
                .template_contents_of(template)
                .ok_or(HtmlConstructionCause::InvalidTemplateAssociation.error())?;
        }
        Ok(())
    }

    /// Sets the entry of the table that `template` holds for its contents to refer to `contents`.
    fn refer_template(
        &mut self,
        template: NodeIx,
        contents: NodeIx,
    ) -> Result<(), HtmlConstructionError> {
        let at = self
            .arena
            .associations
            .place(template, BuildAssocKind::Owned);
        let entry = self
            .arena
            .associations
            .get_mut(at)
            .filter(|entry| entry.owner == template && entry.kind.is_contents())
            .ok_or(HtmlConstructionCause::InvalidTemplateAssociation.error())?;
        entry.contents = contents;
        entry.kind = BuildAssocKind::ShadowReference;
        Ok(())
    }

    /// Adds the shadow entry of `host`, in its place in the table, which has room for it.
    fn add_shadow(&mut self, host: NodeIx, contents: NodeIx) -> Result<(), HtmlConstructionError> {
        let table = &mut self.arena.associations;
        if table.len() >= table.capacity() {
            return Err(HtmlConstructionCause::CapacityMismatch.error());
        }
        let at = table.place(host, BuildAssocKind::Shadow);
        table.insert(
            at,
            BuildAssoc {
                owner: host,
                contents,
                kind: BuildAssocKind::Shadow,
            },
        );
        self.row_mut(host)?.flags.set(NodeFlags::HAS_ASSOC, true);
        Ok(())
    }

    /// Writes the association `verify_association` accepted. A lookup that fails here is a defect
    /// and is returned, never skipped.
    fn apply_association(&mut self, write: Write) -> Result<(), HtmlConstructionError> {
        match write {
            Write::AttachShadow { host, contents } => {
                self.row_mut(contents)?.set_owner(Some(host));
                self.add_shadow(host, contents)?;
            }
            Write::AttachTemplateShadow {
                template,
                host,
                contents,
                released,
            } => {
                self.row_mut(released)?.set_owner(None);
                self.refer_template(template, contents)?;
                self.row_mut(contents)?.set_owner(Some(host));
                self.add_shadow(host, contents)?;
            }
            Write::SetTemplateShadowContents {
                template,
                contents,
                released,
            } => {
                self.row_mut(released)?.set_owner(None);
                self.refer_template(template, contents)?;
            }
        }
        Ok(())
    }
}
