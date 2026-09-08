use crate::{Error, ErrorCode, Result};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PseudoElement {
    Before,
    After,
    Marker,
    Selection,
    Backdrop,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum StyleBucket {
    Element,
    Before,
    After,
    Marker,
    Selection,
    Backdrop,
    BeforeMarker,
    AfterMarker,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum StyleBucketPolicy {
    Element,
    GeneratedContentBox,
    Marker,
    Highlight,
    Backdrop,
    GeneratedContentMarker,
}

impl StyleBucket {
    #[must_use]
    pub const fn is_element(self) -> bool {
        matches!(self, Self::Element)
    }

    #[must_use]
    pub const fn policy(self) -> StyleBucketPolicy {
        match self {
            Self::Element => StyleBucketPolicy::Element,
            Self::Before | Self::After => StyleBucketPolicy::GeneratedContentBox,
            Self::Marker => StyleBucketPolicy::Marker,
            Self::Selection => StyleBucketPolicy::Highlight,
            Self::Backdrop => StyleBucketPolicy::Backdrop,
            Self::BeforeMarker | Self::AfterMarker => StyleBucketPolicy::GeneratedContentMarker,
        }
    }

    pub fn from_pseudo_elements(elements: impl IntoIterator<Item = PseudoElement>) -> Result<Self> {
        let mut elements = elements.into_iter();
        let first = elements.next();
        let second = elements.next();

        if elements.next().is_some() {
            return Err(invalid_style_bucket_sequence());
        }

        match (first, second) {
            (None, None) => Ok(Self::Element),
            (Some(PseudoElement::Before), None) => Ok(Self::Before),
            (Some(PseudoElement::After), None) => Ok(Self::After),
            (Some(PseudoElement::Marker), None) => Ok(Self::Marker),
            (Some(PseudoElement::Selection), None) => Ok(Self::Selection),
            (Some(PseudoElement::Backdrop), None) => Ok(Self::Backdrop),
            (Some(PseudoElement::Before), Some(PseudoElement::Marker)) => Ok(Self::BeforeMarker),
            (Some(PseudoElement::After), Some(PseudoElement::Marker)) => Ok(Self::AfterMarker),
            _ => Err(invalid_style_bucket_sequence()),
        }
    }
}

fn invalid_style_bucket_sequence() -> Error {
    Error::new(
        ErrorCode::InvalidValue,
        "invalid style bucket pseudo-element sequence",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn style_bucket_accepts_supported_pseudo_element_sequences() {
        let cases = [
            (Vec::new(), StyleBucket::Element),
            (vec![PseudoElement::Before], StyleBucket::Before),
            (vec![PseudoElement::After], StyleBucket::After),
            (vec![PseudoElement::Marker], StyleBucket::Marker),
            (vec![PseudoElement::Selection], StyleBucket::Selection),
            (vec![PseudoElement::Backdrop], StyleBucket::Backdrop),
            (
                vec![PseudoElement::Before, PseudoElement::Marker],
                StyleBucket::BeforeMarker,
            ),
            (
                vec![PseudoElement::After, PseudoElement::Marker],
                StyleBucket::AfterMarker,
            ),
        ];

        for (elements, bucket) in cases {
            assert_eq!(StyleBucket::from_pseudo_elements(elements), Ok(bucket));
        }
    }

    #[test]
    fn style_bucket_rejects_invalid_pseudo_element_sequences() {
        let cases = [
            vec![PseudoElement::Marker, PseudoElement::Before],
            vec![PseudoElement::Before, PseudoElement::Selection],
            vec![
                PseudoElement::Before,
                PseudoElement::Marker,
                PseudoElement::Marker,
            ],
            vec![PseudoElement::Selection, PseudoElement::Marker],
        ];

        for elements in cases {
            let error = StyleBucket::from_pseudo_elements(elements).unwrap_err();
            assert_eq!(error.code(), ErrorCode::InvalidValue);
            assert!(error.message().contains("style bucket"));
        }
    }

    #[test]
    fn style_bucket_reports_element_status() {
        assert!(StyleBucket::Element.is_element());
        assert!(!StyleBucket::Before.is_element());
    }

    #[test]
    fn style_bucket_reports_policy() {
        let cases = [
            (StyleBucket::Element, StyleBucketPolicy::Element),
            (StyleBucket::Before, StyleBucketPolicy::GeneratedContentBox),
            (StyleBucket::After, StyleBucketPolicy::GeneratedContentBox),
            (StyleBucket::Marker, StyleBucketPolicy::Marker),
            (StyleBucket::Selection, StyleBucketPolicy::Highlight),
            (StyleBucket::Backdrop, StyleBucketPolicy::Backdrop),
            (
                StyleBucket::BeforeMarker,
                StyleBucketPolicy::GeneratedContentMarker,
            ),
            (
                StyleBucket::AfterMarker,
                StyleBucketPolicy::GeneratedContentMarker,
            ),
        ];

        for (bucket, policy) in cases {
            assert_eq!(bucket.policy(), policy);
        }
    }
}
