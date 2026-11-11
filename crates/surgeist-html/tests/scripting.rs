//! Known scripting conditions for Window settings and their associated Document.

use surgeist_html::{
    attributes::sandbox::{HtmlSandboxKeyword, HtmlSandboxValue},
    scripting::{HtmlScriptingEngine, HtmlScriptingPolicy},
    security::nonce::HtmlBrowsingContextPresence,
};

#[test]
fn each_relevant_setting_and_browsing_context_condition_is_required() {
    use HtmlBrowsingContextPresence::{Absent, Present};
    use HtmlScriptingEngine::{NoEngine, Supported};

    let allowed = HtmlScriptingPolicy {
        user_allows: true,
        automation_allows: true,
    };
    let sandboxed = HtmlSandboxValue::new([]).unwrap().flags();
    let scripts_allowed = HtmlSandboxValue::new([HtmlSandboxKeyword::AllowScripts])
        .unwrap()
        .flags();
    assert!(allowed.document_enabled(Supported, &scripts_allowed, Present));
    assert!(!allowed.document_enabled(Supported, &sandboxed, Present));
    assert!(!allowed.document_enabled(NoEngine, &scripts_allowed, Present));
    assert!(!allowed.document_enabled(Supported, &scripts_allowed, Absent));
    assert!(
        !HtmlScriptingPolicy {
            user_allows: false,
            ..allowed
        }
        .document_enabled(Supported, &scripts_allowed, Present)
    );
    assert!(
        !HtmlScriptingPolicy {
            automation_allows: false,
            ..allowed
        }
        .document_enabled(Supported, &scripts_allowed, Present)
    );
}
