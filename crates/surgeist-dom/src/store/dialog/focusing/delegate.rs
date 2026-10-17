//! Ordinary descendant walks belong to the original called focusing frame.
//! This continuation crosses no event/rebase boundary in the admitted Dialog
//! path. Publication retains its Draft keys and checks their mapped generations.
use super::*;
use surgeist_html::{
    HtmlAutofocusDelegateAction as AA, HtmlAutofocusDelegateCursor, HtmlAutofocusDelegateOutcome,
    HtmlAutofocusDelegateReply as AR, HtmlFocusDelegateAction as FA, HtmlFocusDelegateCursor,
    HtmlFocusDelegateOutcome, HtmlFocusDelegateReply as FR, HtmlFocusDelegateShadowRoot as Shadow,
    HtmlFocusDelegateTargetKind, HtmlFocusTriggerKind,
};

#[derive(Clone, Copy)]
struct Level {
    parent: Key,
    next_child: usize,
}
pub(super) struct Walk {
    levels: select::ScratchVec<Level>,
    current: Option<Key>,
}
impl Walk {
    fn new(draft: &Draft) -> Self {
        Self {
            levels: draft.effect_vec(),
            current: None,
        }
    }
    fn checkpoint(
        &self,
        a: &mut source::checkpoint::Admission<'_>,
        cancel: &AtomicBool,
    ) -> Result<Self, DomError> {
        Ok(Self {
            levels: self.levels.checkpoint(a, cancel)?,
            current: self.current,
        })
    }
    fn begin(&mut self, draft: &Draft, root: Key, cancel: &AtomicBool) -> Result<(), DomError> {
        draft.source_step(cancel)?;
        draft.node(root)?;
        self.levels.clear();
        self.current = None;
        self.reserve_level(draft, cancel)?;
        self.levels.push(Level {
            parent: root,
            next_child: 0,
        });
        Ok(())
    }
    fn reserve_level(&mut self, draft: &Draft, cancel: &AtomicBool) -> Result<(), DomError> {
        if self.levels.len() == self.levels.capacity() {
            // The same checked geometric policy used by Source banks. Admit
            // the actual old/new allocation overlap only at reached growth;
            // spare capacity does not repeat a global payload admission scan.
            let additional = self.levels.capacity().max(1);
            draft.select_reserve(&mut self.levels, additional, cancel)?;
        }
        Ok(())
    }
    fn next(&mut self, draft: &Draft, cancel: &AtomicBool) -> Result<Option<Key>, DomError> {
        draft.source_step(cancel)?;
        if let Some(current) = self.current {
            self.reserve_level(draft, cancel)?;
            self.levels.push(Level {
                parent: current,
                next_child: 0,
            });
            self.current = None;
        }
        while let Some(level) = self.levels.last().copied() {
            draft.source_step(cancel)?;
            if let Some(child) = draft
                .node(level.parent)?
                .children
                .get(level.next_child)
                .copied()
            {
                draft.source_step(cancel)?;
                draft.node(child)?;
                self.levels
                    .last_mut()
                    .expect("retained ancestor")
                    .next_child = level
                    .next_child
                    .checked_add(1)
                    .ok_or(DomError::RepresentationExhausted)?;
                self.current = Some(child);
                return Ok(Some(child));
            }
            self.levels.pop();
        }
        Ok(None)
    }
}
pub(super) struct DelegateFrame {
    cursor: HtmlFocusDelegateCursor,
    where_to_look: Key,
    autofocus: Option<HtmlAutofocusDelegateCursor>,
    walk: Walk,
}
#[derive(Clone, Copy)]
pub(in crate::store) struct PendingDelegate {
    pub root: Key,
    pub queried_node: Key,
    pub step: crate::DomSourceFocusDelegateStep,
}
impl DelegateFrame {
    pub(super) fn new(draft: &Draft, subject: Key) -> Self {
        Self {
            cursor: HtmlFocusDelegateCursor::new(HtmlFocusDelegateTargetKind::Dialog),
            where_to_look: subject,
            autofocus: None,
            walk: Walk::new(draft),
        }
    }
    pub(super) fn checkpoint(
        &self,
        a: &mut source::checkpoint::Admission<'_>,
        cancel: &AtomicBool,
    ) -> Result<Self, DomError> {
        Ok(Self {
            cursor: self.cursor,
            where_to_look: self.where_to_look,
            autofocus: self.autofocus,
            walk: self.walk.checkpoint(a, cancel)?,
        })
    }
    pub(super) fn pending(&self) -> Option<PendingDelegate> {
        Some(PendingDelegate {
            root: self.where_to_look,
            queried_node: self.walk.current?,
            step: match self.autofocus {
                Some(cursor) => crate::DomSourceFocusDelegateStep::Autofocus(cursor.action()),
                None => crate::DomSourceFocusDelegateStep::Focus(self.cursor.action()),
            },
        })
    }
    /// Missing area/sequential/click facts remain at the exact called query.
    /// Only actual null returns are currently available from this native owner.
    pub(super) fn advance(
        &mut self,
        draft: &Draft,
        subject: Key,
        cancel: &AtomicBool,
    ) -> Result<Option<bool>, DomError> {
        loop {
            draft.source_step(cancel)?;
            if let Some(cursor) = self.autofocus.as_mut() {
                let reply = match cursor.action() {
                    AA::BeginDescendants => {
                        self.walk.begin(draft, self.where_to_look, cancel)?;
                        AR::Applied
                    }
                    AA::NextDescendant => {
                        AR::DescendantPresent(self.walk.next(draft, cancel)?.is_some())
                    }
                    AA::ReadAutofocus => {
                        let node = self.walk.current.ok_or(DomError::InvalidSourceOperation)?;
                        draft.source_step(cancel)?;
                        AR::Autofocus(match &draft.node(node)?.kind {
                            DomNodeKind::Element(_) => draft.autofocus_presence(node, cancel)?,
                            _ => false,
                        })
                    }
                    AA::Complete(HtmlAutofocusDelegateOutcome::Null) => {
                        self.autofocus = None;
                        self.cursor
                            .reply(FR::AutofocusDelegatePresent(false))
                            .map_err(DomError::FocusDelegateProtocol)?;
                        continue;
                    }
                    AA::Complete(_) => return Err(DomError::InvalidSourceOperation),
                    _ => return Ok(None),
                };
                cursor
                    .reply(reply)
                    .map_err(DomError::AutofocusDelegateProtocol)?;
                continue;
            }
            let reply = match self.cursor.action() {
                FA::ReadShadowRoot => {
                    draft.source_step(cancel)?;
                    let shadow = match draft.node(subject)?.shadow_root {
                        None => Shadow::Absent,
                        Some(root) => {
                            draft.source_step(cancel)?;
                            let DomNodeKind::ShadowRoot(state) = &draft.node(root)?.kind else {
                                return Err(DomError::WrongNodeKind);
                            };
                            if state.properties().delegates_focus() {
                                self.where_to_look = root;
                                Shadow::DelegatesFocus
                            } else {
                                Shadow::DoesNotDelegateFocus
                            }
                        }
                    };
                    FR::ShadowRoot(shadow)
                }
                FA::RunAutofocusDelegate { .. } => {
                    // Dialog's original trigger is the actual default `other`;
                    // any future caller must retain its own unchanged trigger.
                    self.autofocus = Some(HtmlAutofocusDelegateCursor::new(
                        HtmlFocusTriggerKind::Other,
                    ));
                    continue;
                }
                FA::BeginDescendants { .. } => {
                    self.walk.begin(draft, self.where_to_look, cancel)?;
                    FR::Applied
                }
                FA::NextDescendant => {
                    FR::DescendantPresent(self.walk.next(draft, cancel)?.is_some())
                }
                FA::Complete(HtmlFocusDelegateOutcome::Null) => return Ok(Some(false)),
                FA::Complete(_) => return Err(DomError::InvalidSourceOperation),
                _ => return Ok(None),
            };
            self.cursor
                .reply(reply)
                .map_err(DomError::FocusDelegateProtocol)?;
        }
    }
}

#[cfg(test)]
mod tests;
