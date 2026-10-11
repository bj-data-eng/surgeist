use surgeist_html::*;

// Expected predicates follow HTML 877d6146487f and ONLY the delegated
// editable definition at w3c/editing 85de9ff4a981; no browser oracle helpers.
fn builder() -> (HtmlCaptureBuilder, HtmlCaptureBuilderSubject) {
    let mut b = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(1000, 10000, 1000, 100000, 1000, 0, 0),
        HtmlHostFactsRevision::fresh(),
    );
    let d = b
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    (b, d)
}
fn expanded(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    ns: Namespace,
    local: &str,
    attrs: &[(&str, &str)],
) -> HtmlCaptureBuilderSubject {
    b.add_element(
        d,
        QualifiedName::new(ns, None, local),
        OrderedAttributes::new(
            attrs
                .iter()
                .map(|(n, v)| Attribute::new(QualifiedName::new(Namespace::none(), None, *n), *v)),
        )
        .unwrap(),
        HtmlCapturedIsValue::Unavailable,
        HtmlCaptureRegistryInput::Null,
    )
    .unwrap()
}
fn element(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    local: &str,
    attrs: &[(&str, &str)],
) -> HtmlCaptureBuilderSubject {
    expanded(b, d, Namespace::html(), local, attrs)
}
fn key() -> HtmlGlobalInheritanceRevision {
    HtmlGlobalInheritanceRevision::new(HtmlGlobalInheritanceProducerId::fresh(), 7, 11)
}
fn facts(
    c: &HtmlCapture,
    k: HtmlGlobalInheritanceRevision,
    modes: Vec<HtmlDocumentDesignModeFact>,
    defaults: Vec<HtmlSpellcheckDefaultFact>,
) -> HtmlGlobalInheritanceFacts {
    HtmlGlobalInheritanceFacts::new(
        c,
        c.host_facts_revision().clone(),
        k,
        modes,
        defaults,
        HtmlGlobalInheritanceLimits::new(1000, 1000000),
    )
    .unwrap()
}
fn context(c: &HtmlCapture, f: Option<HtmlGlobalInheritanceFacts>) -> HtmlSemanticContext {
    let input = match f {
        Some(f) => HtmlSemanticContextInput::new(vec![])
            .with_global_inheritance(f.revision().clone(), Some(f)),
        None => HtmlSemanticContextInput::new(vec![]),
    };
    HtmlSemanticContext::new(c, c.host_facts_revision().clone(), input, 0).unwrap()
}
fn limits() -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(10000000, 100, 1000, 1000, 1000000, 1000)
}
fn session<'a>(c: &'a HtmlCapture, cx: &'a HtmlSemanticContext) -> HtmlSemanticSession<'a> {
    HtmlSemanticSession::new(
        c,
        cx,
        HtmlValueInterpreter::new(100000),
        limits(),
        HtmlSemanticCancellation::new(),
    )
    .unwrap()
}
fn req(
    c: &HtmlCapture,
    n: &HtmlCaptureBuilderSubject,
    q: HtmlSemanticQuery,
) -> HtmlSemanticRequest {
    HtmlSemanticRequest::new(c.subject_id(n).unwrap(), q)
}
fn query(
    c: &HtmlCapture,
    cx: &HtmlSemanticContext,
    n: &HtmlCaptureBuilderSubject,
    q: HtmlSemanticQuery,
) -> HtmlSemanticFact {
    session(c, cx).query(&[req(c, n, q)]).unwrap().results()[0]
        .fact()
        .clone()
}
fn mode(
    c: &HtmlCapture,
    d: &HtmlCaptureBuilderSubject,
    state: HtmlDesignMode,
) -> HtmlDocumentDesignModeFact {
    HtmlDocumentDesignModeFact::new(c.subject_id(d).unwrap(), state)
}
fn default(
    c: &HtmlCapture,
    n: &HtmlCaptureBuilderSubject,
    b: Option<HtmlSpellcheckDefaultBehavior>,
) -> HtmlSpellcheckDefaultFact {
    HtmlSpellcheckDefaultFact::new(c.subject_id(n).unwrap(), b)
}
fn heading(c: &HtmlCapture, cx: &HtmlSemanticContext, n: &HtmlCaptureBuilderSubject) -> u8 {
    let HtmlSemanticFact::ComputedHeadingLevel(HtmlSemanticValue::Known(level)) =
        query(c, cx, n, HtmlSemanticQuery::ComputedHeadingLevel)
    else {
        panic!("known heading expected")
    };
    level.value()
}
fn shadow(
    b: &mut HtmlCaptureBuilder,
    host: &HtmlCaptureBuilderSubject,
) -> HtmlCaptureBuilderSubject {
    b.add_shadow_root(
        host,
        HtmlCapturedShadowProperties::new(
            HtmlShadowRootProperties::new(
                HtmlShadowRootMode::Closed,
                HtmlSlotAssignment::Named,
                false,
                false,
                false,
            ),
            false,
            false,
            false,
        ),
        HtmlCaptureRegistryInput::Null,
    )
    .unwrap()
}
#[test]
fn authored_hosts_are_not_editable_and_union_preserves_empty_plaintext_and_case() {
    let (mut b, d) = builder();
    let nodes = ["true", "", "PlAiNtExT-OnLy"]
        .map(|v| element(&mut b, &d, "div", &[("contenteditable", v)]));
    let c = b.finish().unwrap();
    let cx = context(&c, None);
    for n in &nodes {
        assert_eq!(
            query(&c, &cx, n, HtmlSemanticQuery::EditingHost),
            HtmlSemanticFact::EditingHost(HtmlSemanticValue::Known(true))
        );
        assert_eq!(
            query(&c, &cx, n, HtmlSemanticQuery::Editable),
            HtmlSemanticFact::Editable(HtmlSemanticValue::Known(false))
        );
        assert_eq!(
            query(&c, &cx, n, HtmlSemanticQuery::IsContentEditable),
            HtmlSemanticFact::IsContentEditable(HtmlSemanticValue::Known(true))
        );
    }
}
#[test]
fn inherited_editable_false_barrier_and_nested_host_use_ordinary_tree() {
    let (mut b, d) = builder();
    let host = element(&mut b, &d, "div", &[("contenteditable", "true")]);
    let inherited = element(&mut b, &d, "span", &[("contenteditable", "invalid")]);
    let barrier = element(&mut b, &d, "div", &[("contenteditable", "FALSE")]);
    let blocked = element(&mut b, &d, "span", &[]);
    let nested = element(&mut b, &d, "div", &[("contenteditable", "plaintext-only")]);
    let enabled = element(&mut b, &d, "span", &[]);
    b.establish_root(&host).unwrap();
    for (child, parent) in [
        (&inherited, &host),
        (&barrier, &host),
        (&blocked, &barrier),
        (&nested, &barrier),
        (&enabled, &nested),
    ] {
        b.establish_parent(child, parent).unwrap();
    }
    let c = b.finish().unwrap();
    let cx = context(&c, None);
    for (n, want) in [
        (&inherited, true),
        (&barrier, false),
        (&blocked, false),
        (&nested, false),
        (&enabled, true),
    ] {
        assert_eq!(
            query(&c, &cx, n, HtmlSemanticQuery::Editable),
            HtmlSemanticFact::Editable(HtmlSemanticValue::Known(want))
        );
    }
    assert_eq!(
        query(&c, &cx, &nested, HtmlSemanticQuery::IsContentEditable),
        HtmlSemanticFact::IsContentEditable(HtmlSemanticValue::Known(true))
    );
}
#[test]
fn document_child_false_is_host_only_when_actual_design_mode_enabled() {
    let (mut b, d) = builder();
    let child = element(&mut b, &d, "html", &[("contenteditable", "false")]);
    let descendant = element(&mut b, &d, "body", &[]);
    b.establish_parent(&child, &d).unwrap();
    b.establish_parent(&descendant, &child).unwrap();
    let c = b.finish().unwrap();
    for (state, want) in [
        (HtmlDesignMode::Enabled, true),
        (HtmlDesignMode::Disabled, false),
    ] {
        let cx = context(
            &c,
            Some(facts(&c, key(), vec![mode(&c, &d, state)], vec![])),
        );
        assert_eq!(
            query(&c, &cx, &child, HtmlSemanticQuery::EditingHost),
            HtmlSemanticFact::EditingHost(HtmlSemanticValue::Known(want))
        );
        assert_eq!(
            query(&c, &cx, &child, HtmlSemanticQuery::Editable),
            HtmlSemanticFact::Editable(HtmlSemanticValue::Known(false))
        );
        assert_eq!(
            query(&c, &cx, &child, HtmlSemanticQuery::IsContentEditable),
            HtmlSemanticFact::IsContentEditable(HtmlSemanticValue::Known(want))
        );
        assert_eq!(
            query(&c, &cx, &descendant, HtmlSemanticQuery::Editable),
            HtmlSemanticFact::Editable(HtmlSemanticValue::Known(want))
        );
    }
    for cx in [
        context(&c, None),
        context(
            &c,
            Some(facts(
                &c,
                key(),
                vec![mode(&c, &d, HtmlDesignMode::Unavailable)],
                vec![],
            )),
        ),
    ] {
        assert_eq!(
            query(&c, &cx, &child, HtmlSemanticQuery::IsContentEditable),
            HtmlSemanticFact::IsContentEditable(HtmlSemanticValue::Missing(
                HtmlSemanticMissing::DocumentDesignMode(c.subject_id(&d).unwrap())
            ))
        );
    }
}
#[test]
fn owner_document_design_mode_does_not_create_detached_hosts() {
    let (mut b, d) = builder();
    let detached = element(&mut b, &d, "div", &[]);
    b.establish_root(&detached).unwrap();
    let c = b.finish().unwrap();
    let cx = context(
        &c,
        Some(facts(
            &c,
            key(),
            vec![mode(&c, &d, HtmlDesignMode::Enabled)],
            vec![],
        )),
    );
    assert_eq!(
        query(&c, &cx, &detached, HtmlSemanticQuery::EditingHost),
        HtmlSemanticFact::EditingHost(HtmlSemanticValue::Known(false))
    );
    assert_eq!(
        query(&c, &cx, &detached, HtmlSemanticQuery::IsContentEditable),
        HtmlSemanticFact::IsContentEditable(HtmlSemanticValue::Known(false))
    );
}
#[test]
fn svg_math_entry_is_expanded_name_qualified_and_foreign_descendants_break_propagation() {
    let (mut b, d) = builder();
    let host = element(&mut b, &d, "div", &[("contenteditable", "true")]);
    let svg = expanded(
        &mut b,
        &d,
        Namespace::svg(),
        "svg",
        &[("contenteditable", "false")],
    );
    let math = expanded(&mut b, &d, Namespace::mathml(), "math", &[]);
    let arbitrary = expanded(
        &mut b,
        &d,
        Namespace::svg(),
        "div",
        &[("contenteditable", "true")],
    );
    let wrong = expanded(&mut b, &d, Namespace::mathml(), "svg", &[]);
    let descendant = element(&mut b, &d, "span", &[]);
    let under_svg = element(&mut b, &d, "span", &[]);
    for n in [&svg, &math, &arbitrary, &wrong] {
        b.establish_parent(n, &host).unwrap();
    }
    b.establish_parent(&descendant, &arbitrary).unwrap();
    b.establish_parent(&under_svg, &svg).unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c, None);
    for (n, want) in [
        (&svg, true),
        (&math, true),
        (&arbitrary, false),
        (&wrong, false),
        (&descendant, false),
        (&under_svg, true),
    ] {
        assert_eq!(
            query(&c, &cx, n, HtmlSemanticQuery::Editable),
            HtmlSemanticFact::Editable(HtmlSemanticValue::Known(want))
        );
    }
    assert_eq!(
        query(&c, &cx, &svg, HtmlSemanticQuery::IsContentEditable),
        HtmlSemanticFact::IsContentEditable(HtmlSemanticValue::Inapplicable)
    );
}
#[test]
fn non_element_eligibility_requires_html_parent_and_documents_fragments_are_false() {
    let (mut b, d) = builder();
    let host = element(&mut b, &d, "div", &[("contenteditable", "")]);
    let svg = expanded(&mut b, &d, Namespace::svg(), "svg", &[]);
    b.establish_parent(&svg, &host).unwrap();
    let text = b.add_text(&d, "text".into()).unwrap();
    let comment = b.add_comment(&d, "comment".into()).unwrap();
    let blocked = b.add_text(&d, "svg".into()).unwrap();
    let fragment = b.add_fragment(&d).unwrap();
    b.establish_parent(&text, &host).unwrap();
    b.establish_parent(&comment, &host).unwrap();
    b.establish_parent(&blocked, &svg).unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c, None);
    for (n, want) in [
        (&text, true),
        (&comment, true),
        (&blocked, false),
        (&d, false),
        (&fragment, false),
    ] {
        assert_eq!(
            query(&c, &cx, n, HtmlSemanticQuery::Editable),
            HtmlSemanticFact::Editable(HtmlSemanticValue::Known(want))
        );
    }
}
#[test]
fn shadow_and_template_fragments_stop_editability_and_translation_but_not_heading_shadow_offset() {
    let (mut b, d) = builder();
    let host = element(
        &mut b,
        &d,
        "div",
        &[
            ("contenteditable", "true"),
            ("translate", "no"),
            ("headingoffset", "3"),
        ],
    );
    b.establish_root(&host).unwrap();
    let root = shadow(&mut b, &host);
    let plain = b.add_fragment(&d).unwrap();
    let in_shadow = element(&mut b, &d, "h2", &[]);
    let in_plain = element(&mut b, &d, "h2", &[]);
    b.establish_parent(&in_shadow, &root).unwrap();
    b.establish_parent(&in_plain, &plain).unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c, None);
    for n in [&in_shadow, &in_plain] {
        assert_eq!(
            query(&c, &cx, n, HtmlSemanticQuery::Editable),
            HtmlSemanticFact::Editable(HtmlSemanticValue::Known(false))
        );
        assert_eq!(
            query(&c, &cx, n, HtmlSemanticQuery::TranslationMode),
            HtmlSemanticFact::TranslationMode(HtmlSemanticValue::Known(
                HtmlTranslationMode::Enabled
            ))
        );
    }
    assert_eq!(heading(&c, &cx, &in_shadow), 5);
    assert_eq!(heading(&c, &cx, &in_plain), 2);
}
#[test]
fn unknown_parent_is_missing_except_independent_false_or_local_true_proofs() {
    let (mut b, d) = builder();
    let inherit = element(&mut b, &d, "div", &[]);
    let no = element(&mut b, &d, "div", &[("contenteditable", "false")]);
    let c = b.finish().unwrap();
    let cx = context(&c, None);
    assert_eq!(
        query(&c, &cx, &inherit, HtmlSemanticQuery::Editable),
        HtmlSemanticFact::Editable(HtmlSemanticValue::Missing(HtmlSemanticMissing::Parent(
            c.subject_id(&inherit).unwrap()
        )))
    );
    assert_eq!(
        query(&c, &cx, &no, HtmlSemanticQuery::Editable),
        HtmlSemanticFact::Editable(HtmlSemanticValue::Known(false))
    );
    assert_eq!(
        query(&c, &cx, &no, HtmlSemanticQuery::IsContentEditable),
        HtmlSemanticFact::IsContentEditable(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::Parent(c.subject_id(&no).unwrap())
        ))
    );
}
#[test]
fn translation_ignores_foreign_own_attributes_and_uses_nearest_html_state() {
    let (mut b, d) = builder();
    let host = element(&mut b, &d, "div", &[("translate", "NO")]);
    let foreign = expanded(&mut b, &d, Namespace::svg(), "svg", &[("translate", "yes")]);
    let inherit = element(&mut b, &d, "span", &[("translate", " yes")]);
    let local = element(&mut b, &d, "span", &[("translate", "")]);
    b.establish_parent(&foreign, &host).unwrap();
    b.establish_parent(&inherit, &foreign).unwrap();
    b.establish_parent(&local, &foreign).unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c, None);
    for n in [&foreign, &inherit] {
        assert_eq!(
            query(&c, &cx, n, HtmlSemanticQuery::TranslationMode),
            HtmlSemanticFact::TranslationMode(HtmlSemanticValue::Known(
                HtmlTranslationMode::Disabled
            ))
        );
    }
    assert_eq!(
        query(&c, &cx, &local, HtmlSemanticQuery::TranslationMode),
        HtmlSemanticFact::TranslationMode(HtmlSemanticValue::Known(HtmlTranslationMode::Enabled))
    );
}
#[test]
fn translation_root_default_and_unknown_parent_are_distinct() {
    let (mut b, d) = builder();
    let root = element(&mut b, &d, "div", &[("translate", "\u{a0}no")]);
    let unknown = element(&mut b, &d, "div", &[]);
    b.establish_root(&root).unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c, None);
    assert_eq!(
        query(&c, &cx, &root, HtmlSemanticQuery::TranslationMode),
        HtmlSemanticFact::TranslationMode(HtmlSemanticValue::Known(HtmlTranslationMode::Enabled))
    );
    assert_eq!(
        query(&c, &cx, &unknown, HtmlSemanticQuery::TranslationMode),
        HtmlSemanticFact::TranslationMode(HtmlSemanticValue::Missing(HtmlSemanticMissing::Parent(
            c.subject_id(&unknown).unwrap()
        )))
    );
}
#[test]
fn spellcheck_explicit_states_require_no_default_or_editability_facts() {
    let (mut b, d) = builder();
    let yes = element(
        &mut b,
        &d,
        "div",
        &[("spellcheck", ""), ("contenteditable", "false")],
    );
    let no = element(
        &mut b,
        &d,
        "div",
        &[("spellcheck", "FaLsE"), ("contenteditable", "true")],
    );
    let missing = element(&mut b, &d, "div", &[("spellcheck", " true")]);
    let c = b.finish().unwrap();
    let cx = context(&c, None);
    assert_eq!(
        query(&c, &cx, &yes, HtmlSemanticQuery::Spellcheck),
        HtmlSemanticFact::Spellcheck(HtmlSemanticValue::Known(true))
    );
    assert_eq!(
        query(&c, &cx, &no, HtmlSemanticQuery::Spellcheck),
        HtmlSemanticFact::Spellcheck(HtmlSemanticValue::Known(false))
    );
    assert_eq!(
        query(&c, &cx, &missing, HtmlSemanticQuery::Spellcheck),
        HtmlSemanticFact::Spellcheck(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::SpellcheckDefault(c.subject_id(&missing).unwrap())
        ))
    );
}
#[test]
fn spellcheck_default_is_getter_recursion_not_nearest_authored_attribute_search() {
    let (mut b, d) = builder();
    let ancestor = element(&mut b, &d, "div", &[("spellcheck", "false")]);
    let yes = element(&mut b, &d, "span", &[]);
    let no = element(&mut b, &d, "span", &[]);
    let inherit = element(&mut b, &d, "span", &[]);
    let inherit_yes = element(&mut b, &d, "i", &[]);
    for n in [&yes, &no, &inherit] {
        b.establish_parent(n, &ancestor).unwrap();
    }
    b.establish_parent(&inherit_yes, &yes).unwrap();
    let c = b.finish().unwrap();
    let cx = context(
        &c,
        Some(facts(
            &c,
            key(),
            vec![],
            vec![
                default(&c, &yes, Some(HtmlSpellcheckDefaultBehavior::TrueByDefault)),
                default(&c, &no, Some(HtmlSpellcheckDefaultBehavior::FalseByDefault)),
                default(
                    &c,
                    &inherit,
                    Some(HtmlSpellcheckDefaultBehavior::InheritByDefault),
                ),
                default(
                    &c,
                    &inherit_yes,
                    Some(HtmlSpellcheckDefaultBehavior::InheritByDefault),
                ),
            ],
        )),
    );
    for (n, want) in [
        (&yes, true),
        (&no, false),
        (&inherit, false),
        (&inherit_yes, true),
    ] {
        assert_eq!(
            query(&c, &cx, n, HtmlSemanticQuery::Spellcheck),
            HtmlSemanticFact::Spellcheck(HtmlSemanticValue::Known(want))
        );
    }
}
#[test]
fn unavailable_spellcheck_parent_stays_missing_and_foreign_parent_getter_is_unsupported() {
    let (mut b, d) = builder();
    let unknown = element(&mut b, &d, "span", &[]);
    let foreign = expanded(&mut b, &d, Namespace::svg(), "svg", &[]);
    let child = element(&mut b, &d, "span", &[]);
    b.establish_parent(&child, &foreign).unwrap();
    let c = b.finish().unwrap();
    let cx = context(
        &c,
        Some(facts(
            &c,
            key(),
            vec![],
            vec![
                default(
                    &c,
                    &unknown,
                    Some(HtmlSpellcheckDefaultBehavior::InheritByDefault),
                ),
                default(
                    &c,
                    &child,
                    Some(HtmlSpellcheckDefaultBehavior::InheritByDefault),
                ),
            ],
        )),
    );
    assert_eq!(
        query(&c, &cx, &unknown, HtmlSemanticQuery::Spellcheck),
        HtmlSemanticFact::Spellcheck(HtmlSemanticValue::Missing(HtmlSemanticMissing::Parent(
            c.subject_id(&unknown).unwrap()
        )))
    );
    assert_eq!(
        query(&c, &cx, &child, HtmlSemanticQuery::Spellcheck),
        HtmlSemanticFact::Spellcheck(HtmlSemanticValue::Unsupported(
            HtmlSemanticUnsupported::ForeignNamespace
        ))
    );
}
#[test]
fn heading_adds_own_offset_before_reset_and_clamps_huge_recovery_prefixes() {
    let (mut b, d) = builder();
    let ancestor = element(&mut b, &d, "section", &[("headingoffset", "5")]);
    let reset = element(
        &mut b,
        &d,
        "h2",
        &[("headingoffset", "  +2suffix"), ("headingreset", "false")],
    );
    let inherited = element(&mut b, &d, "h3", &[("headingoffset", "1")]);
    let huge = element(&mut b, &d, "h1", &[("headingoffset", &"9".repeat(2000))]);
    let negative = element(&mut b, &d, "h4", &[("headingoffset", "-2")]);
    b.establish_parent(&reset, &ancestor).unwrap();
    b.establish_parent(&inherited, &ancestor).unwrap();
    b.establish_root(&ancestor).unwrap();
    b.establish_root(&negative).unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c, None);
    assert_eq!(heading(&c, &cx, &reset), 4);
    assert_eq!(heading(&c, &cx, &inherited), 9);
    assert_eq!(heading(&c, &cx, &huge), 9);
    assert_eq!(heading(&c, &cx, &negative), 4);
}
#[test]
fn heading_foreign_offsets_ignored_base_local_name_preserved_and_aria_level_not_used() {
    let (mut b, d) = builder();
    let foreign = expanded(
        &mut b,
        &d,
        Namespace::svg(),
        "h2",
        &[("headingoffset", "8"), ("headingreset", "")],
    );
    let child = element(&mut b, &d, "h3", &[("aria-level", "9")]);
    b.establish_root(&foreign).unwrap();
    b.establish_parent(&child, &foreign).unwrap();
    let other = element(&mut b, &d, "div", &[]);
    let upper = element(&mut b, &d, "H1", &[]);
    let c = b.finish().unwrap();
    let cx = context(&c, None);
    assert_eq!(heading(&c, &cx, &foreign), 2);
    assert_eq!(heading(&c, &cx, &child), 3);
    for n in [&other, &upper] {
        assert_eq!(
            query(&c, &cx, n, HtmlSemanticQuery::ComputedHeadingLevel),
            HtmlSemanticFact::ComputedHeadingLevel(HtmlSemanticValue::Inapplicable)
        );
    }
}
#[test]
fn heading_reset_can_prove_level_without_unknown_ancestry() {
    let (mut b, d) = builder();
    let reset = element(&mut b, &d, "h1", &[("headingreset", "")]);
    let unknown = element(&mut b, &d, "h1", &[]);
    let c = b.finish().unwrap();
    let cx = context(&c, None);
    assert_eq!(heading(&c, &cx, &reset), 1);
    assert_eq!(
        query(&c, &cx, &unknown, HtmlSemanticQuery::ComputedHeadingLevel),
        HtmlSemanticFact::ComputedHeadingLevel(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::Parent(c.subject_id(&unknown).unwrap())
        ))
    );
}
#[test]
fn checked_global_records_reject_duplicate_foreign_wrong_kind_and_parentless_inherit() {
    let (mut b, d) = builder();
    let e = element(&mut b, &d, "div", &[]);
    b.establish_parent(&e, &d).unwrap();
    let svg = expanded(&mut b, &d, Namespace::svg(), "svg", &[]);
    let c = b.finish().unwrap();
    let (other, od) = builder();
    let other = other.finish().unwrap();
    let cases = [
        (
            vec![
                mode(&c, &d, HtmlDesignMode::Enabled),
                mode(&c, &d, HtmlDesignMode::Disabled),
            ],
            vec![],
            HtmlGlobalInheritanceError::DuplicateSubject(c.subject_id(&d).unwrap()),
        ),
        (
            vec![HtmlDocumentDesignModeFact::new(
                c.subject_id(&e).unwrap(),
                HtmlDesignMode::Enabled,
            )],
            vec![],
            HtmlGlobalInheritanceError::InapplicableSubject(c.subject_id(&e).unwrap()),
        ),
        (
            vec![mode(&other, &od, HtmlDesignMode::Enabled)],
            vec![],
            HtmlGlobalInheritanceError::Capture(HtmlCaptureError::ForeignCapture),
        ),
        (
            vec![],
            vec![default(
                &c,
                &svg,
                Some(HtmlSpellcheckDefaultBehavior::TrueByDefault),
            )],
            HtmlGlobalInheritanceError::InapplicableSubject(c.subject_id(&svg).unwrap()),
        ),
        (
            vec![],
            vec![default(
                &c,
                &e,
                Some(HtmlSpellcheckDefaultBehavior::InheritByDefault),
            )],
            HtmlGlobalInheritanceError::ParentlessInherit(c.subject_id(&e).unwrap()),
        ),
    ];
    for (modes, defaults, expected) in cases {
        assert_eq!(
            HtmlGlobalInheritanceFacts::new(
                &c,
                c.host_facts_revision().clone(),
                key(),
                modes,
                defaults,
                HtmlGlobalInheritanceLimits::new(100, 100000)
            )
            .unwrap_err(),
            expected
        );
    }
}
#[test]
fn moved_capacity_and_validation_work_are_admitted_before_context_identity() {
    let (b, d) = builder();
    let c = b.finish().unwrap();
    assert!(matches!(
        HtmlGlobalInheritanceFacts::new(
            &c,
            c.host_facts_revision().clone(),
            key(),
            vec![mode(&c, &d, HtmlDesignMode::Disabled)],
            vec![],
            HtmlGlobalInheritanceLimits::new(0, 100)
        )
        .unwrap_err(),
        HtmlGlobalInheritanceError::RecordCapacity { limit: 0, .. }
    ));
    for work in [0, 1] {
        assert!(matches!(
            HtmlGlobalInheritanceFacts::new(
                &c,
                c.host_facts_revision().clone(),
                key(),
                vec![mode(&c, &d, HtmlDesignMode::Disabled)],
                vec![],
                HtmlGlobalInheritanceLimits::new(1, work)
            )
            .unwrap_err(),
            HtmlGlobalInheritanceError::ValidationWork { .. }
        ));
    }
    HtmlGlobalInheritanceFacts::new(
        &c,
        c.host_facts_revision().clone(),
        key(),
        vec![mode(&c, &d, HtmlDesignMode::Disabled)],
        vec![],
        HtmlGlobalInheritanceLimits::new(1, 2),
    )
    .unwrap();
}
#[test]
fn exact_work_output_limits_and_sticky_cancellation_qualify_global_state() {
    let (mut b, d) = builder();
    let n = element(&mut b, &d, "div", &[("translate", "no")]);
    let c = b.finish().unwrap();
    let cx = context(&c, Some(facts(&c, key(), vec![], vec![])));
    let r = req(&c, &n, HtmlSemanticQuery::TranslationMode);
    let report = session(&c, &cx).query(std::slice::from_ref(&r)).unwrap();
    let exact = report.usage().work();
    for (work, pass) in [(exact, true), (exact - 1, false)] {
        let mut s = HtmlSemanticSession::new(
            &c,
            &cx,
            HtmlValueInterpreter::new(100000),
            HtmlSemanticLimits::new(work, 1, 1000, 1000, 1000000, 1000),
            HtmlSemanticCancellation::new(),
        )
        .unwrap();
        let result = s.query(std::slice::from_ref(&r));
        assert_eq!(result.is_ok(), pass);
        if !pass {
            let failure = result.unwrap_err();
            assert_eq!(s.query(&[]).unwrap_err(), failure);
            assert_eq!(
                failure
                    .qualification()
                    .unwrap()
                    .global_inheritance_revision(),
                cx.global_inheritance_revision()
            );
        }
    }
    let cancel = HtmlSemanticCancellation::new();
    cancel.cancel();
    let mut s =
        HtmlSemanticSession::new(&c, &cx, HtmlValueInterpreter::new(100000), limits(), cancel)
            .unwrap();
    let failure = s.query(&[]).unwrap_err();
    assert_eq!(s.query(&[r]).unwrap_err(), failure);
    assert_eq!(
        failure
            .qualification()
            .unwrap()
            .editability_source()
            .revision(),
        "85de9ff4a9816139d06fc2dc39d5dacdc456fddc"
    );
    assert_eq!(
        failure
            .qualification()
            .unwrap()
            .editability_source()
            .complete_source_sha256(),
        "da222a4ea70349eba9d9036a20708f1fa63690118f6c657da3f7f06c56fa78cf"
    );
}
#[test]
fn late_raw_admission_failure_discards_global_batch_prefix() {
    let (mut b, d) = builder();
    let host = element(&mut b, &d, "div", &[("contenteditable", "")]);
    let heading = element(&mut b, &d, "h1", &[("headingoffset", "123")]);
    let c = b.finish().unwrap();
    let cx = context(&c, None);
    let mut s = HtmlSemanticSession::new(
        &c,
        &cx,
        HtmlValueInterpreter::new(0),
        limits(),
        HtmlSemanticCancellation::new(),
    )
    .unwrap();
    let failure = s
        .query(&[
            req(&c, &host, HtmlSemanticQuery::EditingHost),
            req(&c, &heading, HtmlSemanticQuery::ComputedHeadingLevel),
        ])
        .unwrap_err();
    assert!(matches!(
        failure.kind(),
        HtmlSemanticFailureKind::Attribute(_)
    ));
    assert_eq!(failure.usage().output_records(), 2);
    assert_eq!(failure.usage().scratch_records(), 0);
    assert_eq!(s.query(&[]).unwrap_err(), failure);
}

#[test]
fn exact_utf16_invalid_values_preserve_raw_and_use_only_defined_fallbacks() {
    let (mut b, d) = builder();
    let raw = DocumentString::from_utf16(vec![0xd800]);
    let node = b
        .add_element(
            &d,
            QualifiedName::new(Namespace::html(), None, "h2"),
            OrderedAttributes::new(
                [
                    "contenteditable",
                    "translate",
                    "spellcheck",
                    "headingoffset",
                ]
                .map(|name| {
                    Attribute::new(
                        QualifiedName::new(Namespace::none(), None, name),
                        raw.clone(),
                    )
                })
                .into_iter()
                .chain([Attribute::new(
                    QualifiedName::new(Namespace::none(), None, "headingreset"),
                    "",
                )]),
            )
            .unwrap(),
            HtmlCapturedIsValue::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    b.establish_root(&node).unwrap();
    let c = b.finish().unwrap();
    let cx = context(
        &c,
        Some(facts(
            &c,
            key(),
            vec![],
            vec![default(
                &c,
                &node,
                Some(HtmlSpellcheckDefaultBehavior::FalseByDefault),
            )],
        )),
    );
    assert_eq!(
        query(&c, &cx, &node, HtmlSemanticQuery::IsContentEditable),
        HtmlSemanticFact::IsContentEditable(HtmlSemanticValue::Known(false))
    );
    assert_eq!(
        query(&c, &cx, &node, HtmlSemanticQuery::TranslationMode),
        HtmlSemanticFact::TranslationMode(HtmlSemanticValue::Known(HtmlTranslationMode::Enabled))
    );
    assert_eq!(
        query(&c, &cx, &node, HtmlSemanticQuery::Spellcheck),
        HtmlSemanticFact::Spellcheck(HtmlSemanticValue::Known(false))
    );
    assert_eq!(heading(&c, &cx, &node), 2);
    let id = c.subject_id(&node).unwrap();
    let captured = c.node(&id).unwrap();
    let HtmlCapturedNodeKind::Element(element) = captured.kind() else {
        panic!("element")
    };
    for attribute in element.attributes().iter().take(4) {
        assert_eq!(attribute.value(), &raw);
    }
}
#[test]
fn heading_fixed_result_uses_no_variable_integer_digits_or_scratch() {
    let (mut b, d) = builder();
    let h = element(
        &mut b,
        &d,
        "h6",
        &[(
            "headingoffset",
            "999999999999999999999999999999999999999999",
        )],
    );
    let c = b.finish().unwrap();
    let cx = context(&c, None);
    let mut s = HtmlSemanticSession::new(
        &c,
        &cx,
        HtmlValueInterpreter::new(100000),
        HtmlSemanticLimits::new(10000000, 1, 0, 0, 1000000, 0),
        HtmlSemanticCancellation::new(),
    )
    .unwrap();
    let report = s
        .query(&[req(&c, &h, HtmlSemanticQuery::ComputedHeadingLevel)])
        .unwrap();
    let HtmlSemanticFact::ComputedHeadingLevel(HtmlSemanticValue::Known(level)) =
        report.results()[0].fact()
    else {
        panic!("level")
    };
    assert_eq!(level.value(), 9);
    assert_eq!(report.usage().scratch_records(), 0);
}
#[test]
fn zero_output_admission_stops_before_known_query_and_latches_original_failure() {
    let (mut b, d) = builder();
    let n = element(&mut b, &d, "div", &[("contenteditable", "")]);
    let c = b.finish().unwrap();
    let cx = context(&c, None);
    let mut s = HtmlSemanticSession::new(
        &c,
        &cx,
        HtmlValueInterpreter::new(100000),
        HtmlSemanticLimits::new(10000000, 0, 1000, 1000, 1000000, 1000),
        HtmlSemanticCancellation::new(),
    )
    .unwrap();
    let failure = s
        .query(&[req(&c, &n, HtmlSemanticQuery::EditingHost)])
        .unwrap_err();
    assert!(matches!(
        failure.kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::OutputRecords,
            limit: 0,
            ..
        }
    ));
    assert_eq!(s.query(&[]).unwrap_err(), failure);
    assert_eq!(failure.qualification().unwrap().capture(), c.identity());
}
#[test]
fn explicit_unavailable_default_and_absent_default_are_both_missing_not_false() {
    let (mut b, d) = builder();
    let n = element(&mut b, &d, "div", &[]);
    let c = b.finish().unwrap();
    for cx in [
        context(&c, None),
        context(
            &c,
            Some(facts(&c, key(), vec![], vec![default(&c, &n, None)])),
        ),
    ] {
        assert_eq!(
            query(&c, &cx, &n, HtmlSemanticQuery::Spellcheck),
            HtmlSemanticFact::Spellcheck(HtmlSemanticValue::Missing(
                HtmlSemanticMissing::SpellcheckDefault(c.subject_id(&n).unwrap())
            ))
        );
    }
}
#[test]
fn global_facts_reject_stale_host_revision_and_duplicate_spellcheck_records() {
    let (mut b, d) = builder();
    let n = element(&mut b, &d, "div", &[]);
    let c = b.finish().unwrap();
    assert_eq!(
        HtmlGlobalInheritanceFacts::new(
            &c,
            HtmlHostFactsRevision::fresh(),
            key(),
            vec![],
            vec![],
            HtmlGlobalInheritanceLimits::new(0, 0)
        )
        .unwrap_err(),
        HtmlGlobalInheritanceError::HostRevisionMismatch
    );
    assert_eq!(
        HtmlGlobalInheritanceFacts::new(
            &c,
            c.host_facts_revision().clone(),
            key(),
            vec![],
            vec![
                default(&c, &n, None),
                default(&c, &n, Some(HtmlSpellcheckDefaultBehavior::TrueByDefault))
            ],
            HtmlGlobalInheritanceLimits::new(2, 10000)
        )
        .unwrap_err(),
        HtmlGlobalInheritanceError::DuplicateSubject(c.subject_id(&n).unwrap())
    );
}

#[test]
fn shadow_spellcheck_does_not_inherit_host_getter_and_invalid_policy_is_rejected() {
    let (mut b, d) = builder();
    let host = element(&mut b, &d, "div", &[("spellcheck", "false")]);
    let root = shadow(&mut b, &host);
    let child = element(&mut b, &d, "span", &[]);
    b.establish_parent(&child, &root).unwrap();
    let c = b.finish().unwrap();
    assert_eq!(
        HtmlGlobalInheritanceFacts::new(
            &c,
            c.host_facts_revision().clone(),
            key(),
            vec![],
            vec![default(
                &c,
                &child,
                Some(HtmlSpellcheckDefaultBehavior::InheritByDefault)
            )],
            HtmlGlobalInheritanceLimits::new(1, 1000)
        )
        .unwrap_err(),
        HtmlGlobalInheritanceError::ParentlessInherit(c.subject_id(&child).unwrap())
    );
    let cx = context(
        &c,
        Some(facts(
            &c,
            key(),
            vec![],
            vec![default(
                &c,
                &child,
                Some(HtmlSpellcheckDefaultBehavior::TrueByDefault),
            )],
        )),
    );
    assert_eq!(
        query(&c, &cx, &child, HtmlSemanticQuery::Spellcheck),
        HtmlSemanticFact::Spellcheck(HtmlSemanticValue::Known(true))
    );
}
