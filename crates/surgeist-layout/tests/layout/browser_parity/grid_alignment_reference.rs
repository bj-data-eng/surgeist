//! Closed source-qualified Grid-owner reference, independent of paragraph metrics.
use crate::atomic_paragraph_metrics::Font;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Profile {
    pub profile: u8,
    pub owners: Vec<String>,
}
impl Profile {
    pub fn validate(&self, actual: &[String]) -> Result<(), &'static str> {
        if self.profile != 1 {
            return Err("unsupported Grid alignment profile");
        }
        if self.owners != actual {
            return Err(
                "Grid alignment owner census is incomplete, duplicate, orphaned or out of source order",
            );
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Reference {
    Central,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Capture {
    pub profile: u8,
    pub display: String,
    pub writing_mode: String,
    pub direction: String,
    pub text_orientation: String,
    pub dominant_baseline: String,
    pub alignment_baseline: String,
    pub font: Font,
    pub reference: Reference,
}
impl Capture {
    pub fn validate(&self) -> Result<(), &'static str> {
        self.font.validate()?;
        if self.profile != 1
            || !["grid", "inline-grid"].contains(&self.display.as_str())
            || !["vertical-rl", "vertical-lr"].contains(&self.writing_mode.as_str())
            || !["ltr", "rtl"].contains(&self.direction.as_str())
            || !["mixed", "upright"].contains(&self.text_orientation.as_str())
            || self.dominant_baseline != "auto"
            || !["auto", "baseline"].contains(&self.alignment_baseline.as_str())
        {
            return Err("unsupported own Grid typography/reference context");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn actual() -> serde_json::Value {
        json!({"profile":1,"display":"grid","writingMode":"vertical-rl","direction":"rtl","textOrientation":"mixed","dominantBaseline":"auto","alignmentBaseline":"auto","font":{"sha256":crate::atomic_paragraph_metrics::FONT_SHA256,"family":"ahem","size":10,"style":"normal","weight":400,"stretch":"normal","loaded":true},"reference":"central"})
    }
    fn parse(value: serde_json::Value) -> Result<Capture, String> {
        let capture: Capture = serde_json::from_value(value).map_err(|e| e.to_string())?;
        capture.validate().map_err(str::to_owned)?;
        Ok(capture)
    }
    #[test]
    fn own_mixed_and_upright_contexts_do_not_require_paragraph_metrics() {
        for orientation in ["mixed", "upright"] {
            for direction in ["ltr", "rtl"] {
                let mut value = actual();
                value["textOrientation"] = json!(orientation);
                value["direction"] = json!(direction);
                let capture = parse(value).unwrap();
                assert_eq!(capture.reference, Reference::Central);
            }
        }
    }
    #[test]
    fn missing_foreign_non_grid_and_unsupported_authority_refuses() {
        for field in [
            "profile",
            "display",
            "writingMode",
            "direction",
            "textOrientation",
            "dominantBaseline",
            "alignmentBaseline",
            "font",
            "reference",
        ] {
            let mut value = actual();
            value.as_object_mut().unwrap().remove(field);
            assert!(parse(value).is_err(), "{field}");
        }
        for (field, bad) in [
            ("display", json!("block")),
            ("writingMode", json!("horizontal-tb")),
            ("textOrientation", json!("sideways")),
            ("dominantBaseline", json!("central")),
            ("alignmentBaseline", json!("alphabetic")),
            ("reference", json!("alphabetic")),
        ] {
            let mut value = actual();
            value[field] = bad;
            assert!(parse(value).is_err(), "{field}");
        }
        let mut value = actual();
        value["font"]["sha256"] = json!("foreign");
        assert!(parse(value).is_err());
    }
    #[test]
    fn complete_profile_retains_real_zero_owner_census_and_rejects_deletion() {
        Profile {
            profile: 1,
            owners: Vec::new(),
        }
        .validate(&[])
        .unwrap();
        let actual = ["/".to_owned(), "/0".to_owned()];
        Profile {
            profile: 1,
            owners: actual.to_vec(),
        }
        .validate(&actual)
        .unwrap();
        for owners in [
            vec!["/".to_owned()],
            vec!["/".to_owned(), "/".to_owned()],
            vec!["/0".to_owned(), "/".to_owned()],
        ] {
            assert!(Profile { profile: 1, owners }.validate(&actual).is_err());
        }
    }
}
