//! Host-independent font descriptor and map edits over owning captures.
use surgeist_css::rule::font_face::CssFontFaceDescriptorKind;
use surgeist_cssom::*;

fn main() -> Result<(), CssomError> {
    let profile = CssomDeclarationSupport::try_new(vec![])?.with_font_face_descriptors(vec![(
        CssFontFaceDescriptorKind::FontFamily,
        CssomUsabilityRequirement::AllCheckedValues,
    )])?;
    let context = CssomContext {
        inputs: vec![CssomInput {
            version: CssomInputVersion {
                role: CssomInputRole::Support,
                identity: "font-consumer".into(),
                revision: 1,
            },
            data: CssomInputData::Support(profile),
        }],
        ..Default::default()
    };
    let mut store = CssomStore::new(Default::default(), context)?;
    let mut edit = store.batch(CssomEditGuard::from_snapshot(&store.snapshot()))?;
    let ticket = edit.create_parsed_sheet(
        "@font-face {font-family:A;src:local(A)} @font-feature-values A {}",
        CssomSheetInputs::external(
            CssomInputVersion {
                role: CssomInputRole::Document,
                identity: "supplied-source".into(),
                revision: 1,
            },
            true,
        ),
    )?;
    let sheet = store.commit(edit)?.sheet(&ticket)?.clone();
    let before = store.snapshot();
    let rules = before.rules(before.sheet(&sheet)?.rules())?;
    let face = rules[0].clone();
    let features = rules[1].clone();
    let features_object = store.css_font_feature_values_rule(&features)?;
    let map = features_object.styleset(&store)?.id().clone();
    let style = store.css_font_face_rule(&face)?.style(&store)?;
    let before_style = style.capture(&before)?;
    let mut edit = store.batch(CssomEditGuard::from_snapshot(&before))?;
    edit.set_feature(&map, "raw key", CssomFeatureValues::Scalar(7).as_sequence())?;
    edit.set_feature(&map, "present empty", &[])?;
    edit.set_font_feature_family_text(&features, "'serif', Demo")?;
    let prepared = edit.prepare_font_face_descriptor(
        &face,
        CssFontFaceDescriptorKind::FontFamily,
        "B",
        Default::default(),
    )?;
    edit.apply_declaration(prepared, &[])?;
    store.commit(edit)?;
    let after = store.snapshot();
    assert_eq!(
        store.css_font_face_rule(&face)?.style(&store)?.id(),
        style.id()
    );
    let after_map = store.css_font_feature_values_map(&map)?.capture(&after)?;
    assert_eq!(after_map.get("raw key")?, Some(vec![7]));
    assert_eq!(after_map.get("present empty")?, Some(vec![]));
    let after_style = style.capture(&after)?;
    let after_features = features_object.capture(&after)?;
    drop(store);
    assert_eq!(
        after_features.font_family(Default::default())?,
        "\"serif\", Demo"
    );
    assert!(
        after
            .inspect_font_feature_values(&features)?
            .family_input()
            .is_some()
    );
    assert_eq!(before_style.font_family(Default::default())?, "A");
    println!(
        "current font family: {}",
        after_style.font_family(Default::default())?
    );
    Ok(())
}
