//! A real external artifact whose evaluator distinguishes the two original texts.
use std::{convert::Infallible, mem::size_of, sync::Arc};
use surgeist_dom::{
    DomParsedHtmlStyleInlinePolicyObservation, DomParsedHtmlStyleInlinePolicyRequest,
};
use surgeist_html::{
    HtmlCspArtifactLayout, HtmlCspAssociation, HtmlCspDisposalWork, HtmlCspLayout,
    HtmlCspNativePolicy, HtmlCspOwner, HtmlCspPolicyArtifact, HtmlCspProcessorAuthority,
    HtmlCspProcessorProfile, HtmlCspStorage,
};

#[derive(Debug)]
struct ExactText {
    allow_parser_pop: bool,
}
impl HtmlCspPolicyArtifact for ExactText {
    fn retained_layout(&self) -> HtmlCspArtifactLayout {
        HtmlCspArtifactLayout::try_new(
            HtmlCspStorage::try_new(size_of::<Self>(), 0, 1).unwrap(),
            HtmlCspDisposalWork::try_new(1, size_of::<Self>()).unwrap(),
        )
        .unwrap()
    }
}
fn admit(_: HtmlCspLayout) -> Result<(), Infallible> {
    Ok(())
}
pub(super) fn policy(allow_parser_pop: bool) -> (HtmlCspAssociation, HtmlCspProcessorProfile) {
    let mut owner = HtmlCspOwner::new_with(&mut admit).unwrap();
    let mut association = owner.initial_document_with(&mut admit).unwrap();
    let mut processor = HtmlCspProcessorAuthority::new_with(&mut admit).unwrap();
    let profile = processor.profile_with(1, &mut admit).unwrap();
    // This receipt installs an actual native artifact. It is not an inline
    // permission; every request below separately borrows and evaluates it.
    let request = owner
        .request_meta_insertion_with(
            &association,
            &profile,
            "fixture exact-text policy".into(),
            &mut admit,
        )
        .unwrap();
    let prepared = processor
        .attest_changed_with(
            &request,
            Arc::new(ExactText { allow_parser_pop }),
            &mut admit,
        )
        .unwrap();
    drop(
        owner
            .prepare_publication_with(&mut association, request, prepared, &mut admit)
            .unwrap()
            .publish(),
    );
    (association, profile)
}
pub(super) fn decision(
    request: &DomParsedHtmlStyleInlinePolicyRequest,
    profile: &HtmlCspProcessorProfile,
) -> DomParsedHtmlStyleInlinePolicyObservation {
    let HtmlCspNativePolicy::External(native) = request
        .policy()
        .policy()
        .borrow_native_with::<ExactText, _>(profile, &mut admit)
        .unwrap()
    else {
        panic!("this evaluator requires its actual attested native artifact")
    };
    let parser_pop_text = request
        .text()
        .code_units()
        .iter()
        .copied()
        .eq("p{color:red}".encode_utf16());
    let allowed = native.allow_parser_pop && parser_pop_text;
    DomParsedHtmlStyleInlinePolicyObservation::evaluated(request, allowed)
}
