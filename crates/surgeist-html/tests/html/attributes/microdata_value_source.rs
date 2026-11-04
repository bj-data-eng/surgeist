//! Microdata property value sources from the HTML877d "property value" list (`#values`), the
//! URL property elements (`#url-property-elements`) and the time datetime value
//! (`#datetime-value`). Expected values are the spec's lists written out per element.

use surgeist_html::{
    attributes::microdata::{
        HtmlMicrodataUrlSource as Url, HtmlMicrodataValueSource as Value, microdata_url_source,
        microdata_value_source,
    },
    vocabulary::{KnownHtmlAttribute as Attr, KnownHtmlElement as Element},
};

#[test]
fn url_property_elements_read_the_attribute_the_property_value_list_names() {
    use Element::*;
    for (element, expected) in [
        (Audio, Url::Src),
        (Embed, Url::Src),
        (Iframe, Url::Src),
        (Img, Url::Src),
        (Element::Source, Url::Src),
        (Track, Url::Src),
        (Video, Url::Src),
        (A, Url::Href),
        (Area, Url::Href),
        (Link, Url::Href),
        (Object, Url::Data),
    ] {
        assert_eq!(
            microdata_url_source(Some(element)),
            Some(expected),
            "{element:?}"
        );
        assert_eq!(
            microdata_value_source(Some(element)),
            Value::Url(expected),
            "{element:?}"
        );
    }
    assert_eq!(Url::Src.attribute(), Attr::Src);
    assert_eq!(Url::Href.attribute(), Attr::Href);
    assert_eq!(Url::Data.attribute(), Attr::Data);
}

#[test]
fn other_elements_are_not_url_property_elements() {
    use Element::*;
    for element in [
        Div, Span, Meta, Data, Meter, Time, Script, Base, Form, Input,
    ] {
        assert_eq!(microdata_url_source(Some(element)), None, "{element:?}");
    }
    assert_eq!(microdata_url_source(None), None);
}

#[test]
fn value_attribute_datetime_and_text_cases_follow_the_property_value_list() {
    assert_eq!(
        microdata_value_source(Some(Element::Meta)),
        Value::Attribute(Attr::Content)
    );
    assert_eq!(
        microdata_value_source(Some(Element::Data)),
        Value::Attribute(Attr::Value)
    );
    assert_eq!(
        microdata_value_source(Some(Element::Meter)),
        Value::Attribute(Attr::Value)
    );
    assert_eq!(
        microdata_value_source(Some(Element::Time)),
        Value::DatetimeValue
    );
    for element in [Element::Div, Element::Span, Element::P, Element::Button] {
        assert_eq!(
            microdata_value_source(Some(element)),
            Value::DescendantText,
            "{element:?}"
        );
    }
    assert_eq!(microdata_value_source(None), Value::DescendantText);
}
