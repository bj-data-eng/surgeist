use surgeist_html::*;

fn report(role: KnownHtmlElement, attrs: &[(KnownHtmlAttribute, &str)]) -> HtmlRelReport {
    let attrs = OrderedAttributes::new(
        attrs
            .iter()
            .map(|(name, value)| Attribute::new(name.qualified_name(), *value)),
    )
    .unwrap();
    HtmlValueInterpreter::new(1024)
        .rel_with_attributes(&role.qualified_name(), &attrs)
        .unwrap()
}

#[test]
fn defining_role_families_keep_effects_and_wrong_role_tokens_separate() {
    // HTML877d complete defining clauses; independent literal role domains.
    let families = [
        (
            "alternate author privacy-policy terms-of-service",
            [true, true, true, false],
            HtmlRelEffect::Hyperlink,
        ),
        (
            "canonical",
            [true, false, false, false],
            HtmlRelEffect::Hyperlink,
        ),
        (
            "bookmark tag",
            [false, true, true, false],
            HtmlRelEffect::Hyperlink,
        ),
        (
            "help license next prev search",
            [true, true, true, true],
            HtmlRelEffect::Hyperlink,
        ),
        (
            "dns-prefetch icon manifest modulepreload pingback preconnect prefetch preload stylesheet",
            [true, false, false, false],
            HtmlRelEffect::ExternalResource,
        ),
        (
            "expect",
            [true, false, false, false],
            HtmlRelEffect::InternalResource,
        ),
        (
            "external nofollow noopener noreferrer opener",
            [false, true, true, true],
            HtmlRelEffect::Annotation,
        ),
    ];
    for (raw, allowed, effect) in families {
        for (i, role) in [
            KnownHtmlElement::Link,
            KnownHtmlElement::A,
            KnownHtmlElement::Area,
            KnownHtmlElement::Form,
        ]
        .into_iter()
        .enumerate()
        {
            let r = report(
                role,
                &[
                    (KnownHtmlAttribute::Rel, raw),
                    (KnownHtmlAttribute::Href, "x"),
                ],
            );
            for token in r.occurrences() {
                assert_eq!(
                    token.effect(),
                    allowed[i].then_some(effect),
                    "{role:?}: {raw}"
                );
                assert_eq!(
                    token.applicability(),
                    if allowed[i] {
                        HtmlRelTokenApplicability::Allowed
                    } else {
                        HtmlRelTokenApplicability::NotAllowed
                    }
                );
            }
        }
    }
}
#[test]
fn authored_case_duplicates_aliases_and_whole_shortcut_exception_are_preserved() {
    let r = report(
        KnownHtmlElement::A,
        &[
            (
                KnownHtmlAttribute::Rel,
                "NEXT\tnext\nprevious\rcopyright\u{c}alien\u{a0}x",
            ),
            (KnownHtmlAttribute::Href, ""),
        ],
    );
    assert_eq!(r.occurrences().len(), 5);
    assert_eq!(
        r.occurrence_units(0),
        Some("NEXT".encode_utf16().collect::<Vec<_>>().as_slice())
    );
    assert_eq!(r.occurrences()[1].duplicate_of(), Some(0));
    assert_eq!(r.occurrences()[2].keyword(), Some(HtmlRelKeyword::Prev));
    assert_eq!(
        r.occurrences()[2].spelling(),
        HtmlRelSpelling::ForbiddenSynonym
    );
    assert_eq!(r.occurrences()[3].keyword(), Some(HtmlRelKeyword::License));
    assert_eq!(r.occurrences()[4].spelling(), HtmlRelSpelling::Unknown);
    for raw in ["shortcut icon", "SHORTCUT ICON"] {
        let r = report(KnownHtmlElement::Link, &[(KnownHtmlAttribute::Rel, raw)]);
        assert_eq!(
            r.occurrences()[0].spelling(),
            HtmlRelSpelling::ShortcutException
        );
    }
    for raw in [
        "shortcut  icon",
        "shortcut\ticon",
        "icon shortcut",
        "shortcut icon next",
    ] {
        let r = report(KnownHtmlElement::Link, &[(KnownHtmlAttribute::Rel, raw)]);
        assert!(
            r.occurrences()
                .iter()
                .any(|t| t.spelling() == HtmlRelSpelling::InvalidShortcut)
        );
    }
}
#[test]
fn presence_no_links_implied_links_and_independent_author_failures_coexist() {
    let r = report(KnownHtmlElement::Link, &[]);
    assert_eq!(
        r.presence_author(),
        HtmlRelPresenceAuthor::NeitherRelNorItemprop
    );
    assert_eq!(r.links(), HtmlRelLinks::None);
    assert_eq!(r.body_allowance(), HtmlRelBodyAllowance::RelAbsent);
    let r = report(
        KnownHtmlElement::Link,
        &[
            (KnownHtmlAttribute::Rel, ""),
            (KnownHtmlAttribute::Itemprop, ""),
        ],
    );
    assert_eq!(
        r.presence_author(),
        HtmlRelPresenceAuthor::BothRelAndItemprop
    );
    assert_eq!(r.body_allowance(), HtmlRelBodyAllowance::Itemprop);
    for role in [KnownHtmlElement::A, KnownHtmlElement::Area] {
        assert_eq!(
            report(role, &[(KnownHtmlAttribute::Rel, "noopener")]).links(),
            HtmlRelLinks::None
        );
        assert_eq!(
            report(
                role,
                &[
                    (KnownHtmlAttribute::Rel, "noopener"),
                    (KnownHtmlAttribute::Href, "")
                ]
            )
            .links(),
            HtmlRelLinks::ImpliedHyperlink
        );
    }
    assert_eq!(
        report(
            KnownHtmlElement::Form,
            &[(KnownHtmlAttribute::Rel, "alien")]
        )
        .links(),
        HtmlRelLinks::ExtensionEffectsUnchecked
    );
    assert_eq!(
        report(
            KnownHtmlElement::Form,
            &[(KnownHtmlAttribute::Rel, "help alien")]
        )
        .links(),
        HtmlRelLinks::Declared
    );
}
#[test]
fn alternate_stylesheet_and_feed_meanings_are_independent_of_discovery() {
    let r = report(
        KnownHtmlElement::Link,
        &[
            (KnownHtmlAttribute::Rel, "alternate stylesheet"),
            (KnownHtmlAttribute::Type, "application/rss+xml"),
            (KnownHtmlAttribute::Href, "x"),
        ],
    );
    assert_eq!(r.alternate(), HtmlRelAlternate::StylesheetModifier);
    assert_eq!(
        r.occurrences()[0].effect(),
        Some(HtmlRelEffect::StylesheetModifier)
    );
    assert_eq!(
        r.alternate_stylesheet_title_author(),
        HtmlRelTitleAuthor::MissingOrEmpty
    );
    assert_eq!(
        report(
            KnownHtmlElement::Link,
            &[
                (KnownHtmlAttribute::Rel, "alternate"),
                (KnownHtmlAttribute::Type, "application/atom+xml")
            ]
        )
        .alternate(),
        HtmlRelAlternate::Feed
    );
    assert_eq!(
        report(
            KnownHtmlElement::Link,
            &[
                (KnownHtmlAttribute::Rel, "alternate"),
                (KnownHtmlAttribute::Type, "Application/atom+xml")
            ]
        )
        .alternate(),
        HtmlRelAlternate::Representation
    );
}

#[test]
fn historical_rev_is_the_source_whole_value_and_does_not_enter_the_rel_token_list() {
    let from_rev = |role: KnownHtmlElement, raw: &str| {
        let attributes = OrderedAttributes::new([
            Attribute::new(QualifiedName::new(Namespace::none(), None, "rev"), raw),
            Attribute::new(KnownHtmlAttribute::Href.qualified_name(), ""),
        ])
        .unwrap();
        HtmlValueInterpreter::new(1024)
            .rel_with_attributes(&role.qualified_name(), &attributes)
            .unwrap()
    };
    for role in [
        KnownHtmlElement::Link,
        KnownHtmlElement::A,
        KnownHtmlElement::Area,
        KnownHtmlElement::Form,
    ] {
        let r = from_rev(role, "made");
        assert_eq!(
            r.occurrences().len(),
            usize::from(role != KnownHtmlElement::Form)
        );
        if role != KnownHtmlElement::Form {
            assert_eq!(r.occurrences()[0].keyword(), Some(HtmlRelKeyword::Author));
            assert_eq!(r.occurrences()[0].source(), HtmlRelOccurrenceSource::Rev);
            assert_eq!(
                r.occurrences()[0].spelling(),
                HtmlRelSpelling::HistoricalRev
            );
        }
        for raw in ["MADE", "Made", " made", "made ", "made author", "made\t"] {
            assert!(
                from_rev(role, raw).occurrences().is_empty(),
                "{role:?} {raw:?}"
            );
        }
    }
}

#[test]
fn link_address_presence_nonempty_href_and_image_preload_are_independent() {
    for (rel, dest, selected) in [
        ("preload", "IMAGE", true),
        ("stylesheet", "image", false),
        ("preload", "script", false),
        ("preload", " image", false),
    ] {
        let r = report(
            KnownHtmlElement::Link,
            &[
                (KnownHtmlAttribute::Rel, rel),
                (KnownHtmlAttribute::As, dest),
                (KnownHtmlAttribute::Imagesrcset, "candidate.png 1x"),
            ],
        );
        assert_eq!(r.address_presence_author(), HtmlRelAddressAuthor::Satisfied);
        assert_eq!(r.image_source_preload_context(), selected);
        assert_eq!(r.href_author(), HtmlRelHrefAuthor::NotRequired);
    }
    for href in ["", " \t\r\n\u{c}"] {
        let r = report(
            KnownHtmlElement::Link,
            &[
                (KnownHtmlAttribute::Rel, "icon"),
                (KnownHtmlAttribute::Href, href),
            ],
        );
        assert_eq!(r.links(), HtmlRelLinks::Declared);
        assert_eq!(r.href_author(), HtmlRelHrefAuthor::EmptyOrOnlyAsciiSpaces);
    }
    let r = report(KnownHtmlElement::Link, &[(KnownHtmlAttribute::Rel, "icon")]);
    assert_eq!(
        r.address_presence_author(),
        HtmlRelAddressAuthor::MissingHrefAndImagesrcset
    );
    assert_eq!(r.links(), HtmlRelLinks::None);
    let r = report(
        KnownHtmlElement::Link,
        &[(KnownHtmlAttribute::Href, "\u{a0}")],
    );
    assert_eq!(r.href_author(), HtmlRelHrefAuthor::GrammarUnchecked);
    let r = report(KnownHtmlElement::A, &[(KnownHtmlAttribute::Href, "")]);
    assert_eq!(r.href_author(), HtmlRelHrefAuthor::NotRequired);
}
