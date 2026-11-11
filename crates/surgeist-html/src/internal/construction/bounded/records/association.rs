//! Shadow and template-contents associations.
//!
//! A shadow root is an unowned shadow contents fragment attached to a host element; a template
//! element owns a contents fragment, or refers to shadow contents instead. Each association adds
//! or releases the edge between a host or template and a fragment, moves the fragment's subtree
//! to its new depth, and sets the owner and association fields.

use super::depth::{DepthPlan, Placement, Request};
use super::*;
use crate::internal::construction::bounded::operations::{
    ordinary_owned_contents, unowned_fragment,
};

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

impl HtmlConstructionStorage {
    /// Checks that `host` is an element with no shadow root, and that `contents` is an unowned
    /// shadow contents fragment.
    fn check_shadow(&self, host: NodeIx, contents: NodeIx) -> Result<(), HtmlConstructionError> {
        let record = self.record(host)?;
        if !matches!(record.kind, NodeContent::Element(_)) {
            return Err(HtmlConstructionCause::ShadowHostMustBeElement.error());
        }
        if record.shadow.is_some() {
            return Err(HtmlConstructionCause::ShadowAlreadyAttached.error());
        }
        self.record(contents)?;
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
        let placement = if self.record(released).map_err(faulted)?.children.is_empty() {
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
        self.record(contents).map_err(faulted)?;
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
        // The owner of the contents, the association of the host or template, and for a template
        // shadow the released contents' owner.
        let (fields, copied) = match write {
            Write::AttachShadow { .. } => (
                2,
                size_of::<Option<HtmlFragmentOwner>>() + size_of::<Option<ShadowAssociation>>(),
            ),
            Write::AttachTemplateShadow { .. } => (
                4,
                2 * size_of::<Option<HtmlFragmentOwner>>()
                    + size_of::<Option<TemplateContents>>()
                    + size_of::<Option<ShadowAssociation>>(),
            ),
            Write::SetTemplateShadowContents { .. } => (
                2,
                size_of::<Option<HtmlFragmentOwner>>() + size_of::<Option<TemplateContents>>(),
            ),
        };
        let (depth_visits, depth_writes) = depths.work().map_err(faulted)?;
        let usage = HtmlConstructionChange { before, after };
        admission
            .record_largest(HtmlResource::TreeDepth, wide(after.depth))
            .map_err(stopped)?;
        admission.admit_change(self, usage).map_err(stopped)?;
        // The writes are charged once the limits admitted the change.
        Work {
            record_visits: depth_visits,
            copied_bytes: plus(
                plus(
                    copied,
                    times(depth_writes, size_of::<usize>()).map_err(faulted)?,
                )
                .map_err(faulted)?,
                size_of::<HtmlConstructionUsage>(),
            )
            .map_err(faulted)?,
            record_writes: plus(plus(fields, depth_writes).map_err(faulted)?, 1)
                .map_err(faulted)?,
            ..Work::default()
        }
        .charge(admission)?;
        // Every record the writes name exists, checked before the last step that can refuse.
        self.verify_association(write).map_err(faulted)?;
        admission.record_change(self, usage).map_err(stopped)?;
        depths.apply(&mut self.arena);
        self.apply_association(write).map_err(faulted)?;
        self.arena.usage = after;
        Ok(())
    }

    /// Checks that every record `write` names exists.
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
            self.record(name)?;
        }
        Ok(())
    }

    /// Writes the association `verify_association` accepted. A lookup that fails here is a defect
    /// and is returned, never skipped.
    fn apply_association(&mut self, write: Write) -> Result<(), HtmlConstructionError> {
        match write {
            Write::AttachShadow { host, contents } => {
                let (host_id, contents_id) = (self.arena.id(host), self.arena.id(contents));
                self.record_mut(contents)?.owner = Some(HtmlFragmentOwner::ShadowHost(host_id));
                self.record_mut(host)?.shadow = Some(ShadowAssociation {
                    host: host_id,
                    contents: contents_id,
                });
            }
            Write::AttachTemplateShadow {
                template,
                host,
                contents,
                released,
            } => {
                let (host_id, contents_id) = (self.arena.id(host), self.arena.id(contents));
                self.record_mut(released)?.owner = None;
                self.record_mut(template)?.template_contents =
                    Some(TemplateContents::ShadowReference(contents));
                self.record_mut(contents)?.owner = Some(HtmlFragmentOwner::ShadowHost(host_id));
                self.record_mut(host)?.shadow = Some(ShadowAssociation {
                    host: host_id,
                    contents: contents_id,
                });
            }
            Write::SetTemplateShadowContents {
                template,
                contents,
                released,
            } => {
                self.record_mut(released)?.owner = None;
                self.record_mut(template)?.template_contents =
                    Some(TemplateContents::ShadowReference(contents));
            }
        }
        Ok(())
    }
}
