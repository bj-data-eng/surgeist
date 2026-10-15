//! Intrinsic owner-document selection from HTML877d
//! #appropriate-template-contents-owner-document.

/// Which actual document must own an HTML template's separate contents.
///
/// This decision contains no identities and does not create or adopt nodes.
/// The consuming owner retains its associated inert document cache and applies
/// the source's document creation, fragment hosting, clone and adoption steps.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlTemplateContentsOwnerDocument {
    CurrentDocument,
    AssociatedInertDocument,
    CreateAssociatedInertDocument,
}

/// Selects the appropriate template contents owner from established facts.
///
/// `created_by_template_owner_algorithm` means the document was actually
/// created by this source algorithm, not that it contains a template or happens
/// to be disconnected. Such documents reuse themselves; the cache argument is
/// ignored and the caller need not consult that fact in this branch.
///
/// Otherwise, `has_associated_inert_document` must describe the actual retained
/// cache. Missing role/cache evidence must be handled by the caller's qualified
/// availability contract before invoking a branch which requires that fact.
/// Only creation requires the document class, which this decision does not read.
/// A created owner has no browsing context, inherits HTML-document marking when
/// required, and is cached once by the consuming document owner.
pub const fn template_contents_owner_document(
    created_by_template_owner_algorithm: bool,
    has_associated_inert_document: bool,
) -> HtmlTemplateContentsOwnerDocument {
    if created_by_template_owner_algorithm {
        HtmlTemplateContentsOwnerDocument::CurrentDocument
    } else if has_associated_inert_document {
        HtmlTemplateContentsOwnerDocument::AssociatedInertDocument
    } else {
        HtmlTemplateContentsOwnerDocument::CreateAssociatedInertDocument
    }
}
