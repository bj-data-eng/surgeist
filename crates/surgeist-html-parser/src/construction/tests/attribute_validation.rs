use super::*;
use surgeist_html::values::{Attribute, Namespace, QualifiedName};

#[test]
fn controlled_attribute_validation_preserves_parser_admission_and_refused_backing() {
    let backing = Arc::new(vec![
        Attribute::new(
            QualifiedName::new(Namespace::from_uri("urn:a"), None, "ab"),
            "left",
        ),
        Attribute::new(
            QualifiedName::new(Namespace::from_uri("urn:a"), None, "ac"),
            "right",
        ),
    ]);
    // Pair + two string-header pairs = 6 visits. Five namespace and two
    // local-name comparisons inspect 28 bytes. No copies/providers/reserves.
    for credits in 0..=34 {
        let selected = limits()
            .into_builder()
            .work_credits(credits)
            .finish()
            .unwrap();
        let mut control = ParseControl::new(selected, HtmlParseCancellation::new());
        match validate_parser_attributes(backing.clone(), &mut control) {
            Ok(attributes) => {
                assert_eq!(credits, 34);
                assert_eq!(attributes.as_slice().as_ptr(), backing.as_ptr());
                assert_eq!(control.usage().work(HtmlParseWorkCategory::RecordVisit), 6);
                assert_eq!(control.usage().work(HtmlParseWorkCategory::InspectByte), 28);
            }
            Err((error, recovered)) => {
                assert!(credits < 34);
                assert_eq!(error, SessionError::Stop(control.stop().unwrap()));
                assert!(Arc::ptr_eq(&recovered, &backing));
                assert_eq!(recovered.capacity(), backing.capacity());
                let stop = control.stop();
                drop(recovered);
                assert_eq!(control.stop(), stop);
            }
        }
        assert_eq!(control.usage().work(HtmlParseWorkCategory::CopyByte), 0);
        assert_eq!(control.usage().work(HtmlParseWorkCategory::ReserveByte), 0);
        assert_eq!(control.usage().work(HtmlParseWorkCategory::ProviderCall), 0);
        assert_eq!(control.usage().actual_capacity_bytes, 0);
        assert_eq!(Arc::strong_count(&backing), 1);
    }
}

#[test]
fn controlled_attribute_validation_keeps_cancellation_and_first_duplicate_order() {
    let backing = Arc::new(vec![
        Attribute::new(QualifiedName::new(Namespace::none(), None, "x"), "left"),
        Attribute::new(QualifiedName::new(Namespace::none(), None, "x"), "right"),
    ]);
    for credits in [7, 8] {
        let selected = limits()
            .into_builder()
            .work_credits(credits)
            .finish()
            .unwrap();
        let mut control = ParseControl::new(selected, HtmlParseCancellation::new());
        let (error, recovered) =
            validate_parser_attributes(backing.clone(), &mut control).unwrap_err();
        assert!(Arc::ptr_eq(&recovered, &backing));
        if credits == 7 {
            assert_eq!(error, SessionError::Stop(control.stop().unwrap()));
            assert_eq!(control.usage().work(HtmlParseWorkCategory::InspectByte), 0);
        } else {
            assert_eq!(error, SessionError::DuplicateAttribute);
            assert!(control.stop().is_none());
            assert_eq!(control.usage().work(HtmlParseWorkCategory::InspectByte), 4);
        }
    }
    let cancellation = HtmlParseCancellation::new();
    cancellation.cancel();
    let mut control = ParseControl::new(limits(), cancellation);
    let (error, recovered) = validate_parser_attributes(backing.clone(), &mut control).unwrap_err();
    assert_eq!(error, SessionError::Stop(HtmlParseStop::Cancelled));
    assert!(Arc::ptr_eq(&recovered, &backing));
    assert_eq!(control.usage().work(HtmlParseWorkCategory::RecordVisit), 0);
    assert_eq!(control.usage().work(HtmlParseWorkCategory::InspectByte), 0);
}
