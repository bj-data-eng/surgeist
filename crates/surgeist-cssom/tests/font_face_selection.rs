//! Public resource admission for the new owning current FontFace selection.
use surgeist_css::*;
use surgeist_cssom::*;

#[test]
fn new_font_face_selection_initialization_rejects_resource_failure_atomically() {
    for css in [
        CssSpecifiedValueSerializationLimits::new(0, 1000, 1000),
        CssSpecifiedValueSerializationLimits::new(1000, 0, 1000),
    ] {
        let mut store = CssomStore::new(
            CssomLimits {
                css,
                ..Default::default()
            },
            CssomContext::default(),
        )
        .unwrap();
        let before = store.snapshot();
        let mut edit = store.batch(CssomEditGuard::from_snapshot(&before)).unwrap();
        // Existing empty sheet creation is staged first; the failed new facet must publish neither.
        let inputs = CssomSheetInputs::constructed(
            CssomInputVersion {
                role: CssomInputRole::Document,
                identity: "doc".into(),
                revision: 1,
            },
            "doc".into(),
            None,
        );
        edit.create_parsed_sheet("", inputs.clone()).unwrap();
        let result = edit.create_parsed_sheet("@font-face {font-family:A}", inputs);
        assert!(
            matches!(
                result,
                Err(CssomError::FontFace(
                    CssFontFaceDeclarationBlockError::Serialization(_)
                ))
            ),
            "new selected FontFace projection returned {result:?}"
        );
        assert!(matches!(store.commit(edit), Err(CssomError::BatchAborted)));
        assert_eq!(store.snapshot().revision(), before.revision());
        assert_eq!(store.snapshot().sheets(), before.sheets());
    }
}
