//! Host-independent font descriptor and map edits over owning captures.
use surgeist_css::*;
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
    let map = before.font_feature_values(&features)?.styleset().clone();
    let style = before
        .font_face(&face, Default::default())?
        .style_id()
        .clone();
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
        after.font_face(&face, Default::default())?.style_id(),
        &style
    );
    assert_eq!(
        after.feature_map(&map)?.get("raw key")?,
        Some([7].as_slice())
    );
    assert_eq!(
        after.feature_map(&map)?.get("present empty")?,
        Some([].as_slice())
    );
    drop(store);
    assert_eq!(
        after
            .font_feature_values(&features)?
            .font_family(Default::default())?,
        "\"serif\", Demo"
    );
    assert!(
        after
            .font_feature_values(&features)?
            .family_input()
            .is_some()
    );
    assert_eq!(
        before.font_face(&face, Default::default())?.font_family()?,
        "A"
    );
    println!(
        "current font family: {}",
        after.font_face(&face, Default::default())?.font_family()?
    );
    Ok(())
}
