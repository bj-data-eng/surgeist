//! Versioned source-bound numeric metrics; no font/provider or Text identities.
use serde::{Deserialize, Serialize};

pub const FONT_SHA256: &str = "e450dd6e1bc6fb10ebc6edecef1182dcd8cad6e12e54c363ad4ba26f6b5db46c";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Font {
    pub sha256: String,
    pub family: String,
    pub size: f64,
    pub style: String,
    pub weight: u16,
    pub stretch: String,
    pub loaded: bool,
}
impl Font {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.sha256 != FONT_SHA256
            || self.family != "ahem"
            || self.size != 10.0
            || self.style != "normal"
            || self.weight != 400
            || self.stretch != "normal"
            || !self.loaded
        {
            return Err("unsupported or missing actual font resource/style authority");
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
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum Occurrence {
    Atomic {
        child: usize,
        source: usize,
        bidi_level: u8,
    },
    Break {
        child: usize,
        source: usize,
        reference: Reference,
        before: f64,
        after: f64,
    },
}
impl Occurrence {
    pub fn child(&self) -> usize {
        match self {
            Self::Atomic { child, .. } | Self::Break { child, .. } => *child,
        }
    }
    pub fn source(&self) -> usize {
        match self {
            Self::Atomic { source, .. } | Self::Break { source, .. } => *source,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Capture {
    pub profile: u8,
    pub font: Font,
    pub writing_mode: String,
    pub direction: String,
    pub text_orientation: String,
    pub dominant_baseline: String,
    pub alignment_baseline: String,
    pub vertical_align: String,
    pub line_fit: String,
    pub reference: Reference,
    pub before: f64,
    pub after: f64,
    pub line_height: f64,
    pub order: [Occurrence; 3],
}
impl Capture {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.profile != 1 {
            return Err("unsupported atomic paragraph metric profile");
        }
        self.font.validate()?;
        if !["vertical-rl", "vertical-lr"].contains(&self.writing_mode.as_str())
            || !["ltr", "rtl"].contains(&self.direction.as_str())
            || !["mixed", "upright"].contains(&self.text_orientation.as_str())
            || (self.text_orientation == "upright" && self.direction != "ltr")
            || self.dominant_baseline != "auto"
            || !["auto", "baseline"].contains(&self.alignment_baseline.as_str())
            || self.vertical_align != "baseline"
            || self.line_fit != "leading"
        {
            return Err("unsupported typographic reference or line-fit authority");
        }
        // Actual Ahem10 Central has A=D=5. Half-leading at H0 is -5/-5.
        // This profile never clamps an asymmetric signed tuple into this API.
        if self.before != 0.0 || self.after != 0.0 || self.line_height != 0.0 {
            return Err("metric tuple is not the qualified Central zero strut");
        }
        let level = u8::from(self.direction == "rtl");
        for (index, row) in self.order.iter().enumerate() {
            if row.child() != index || index > 0 && self.order[index - 1].source() >= row.source() {
                return Err("invalid actual occurrence order");
            }
            match (index, row) {
                (0 | 2, Occurrence::Atomic { bidi_level, .. }) if *bidi_level == level => {}
                (1, Occurrence::Break { before, after, .. }) if *before == 0.0 && *after == 0.0 => {
                }
                _ => return Err("missing or mismatched actual atomic/BR reference"),
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn parse(raw: &str) -> Result<Capture, String> {
        let capture: Capture = serde_json::from_str(raw).map_err(|error| error.to_string())?;
        capture.validate().map_err(str::to_owned)?;
        Ok(capture)
    }
    fn actual() -> Capture {
        Capture {
            profile: 1,
            font: Font {
                sha256: FONT_SHA256.into(),
                family: "ahem".into(),
                size: 10.0,
                style: "normal".into(),
                weight: 400,
                stretch: "normal".into(),
                loaded: true,
            },
            writing_mode: "vertical-rl".into(),
            direction: "rtl".into(),
            text_orientation: "mixed".into(),
            dominant_baseline: "auto".into(),
            alignment_baseline: "auto".into(),
            vertical_align: "baseline".into(),
            line_fit: "leading".into(),
            reference: Reference::Central,
            before: 0.0,
            after: 0.0,
            line_height: 0.0,
            order: [
                Occurrence::Atomic {
                    child: 0,
                    source: 1,
                    bidi_level: 1,
                },
                Occurrence::Break {
                    child: 1,
                    source: 2,
                    reference: Reference::Central,
                    before: 0.0,
                    after: 0.0,
                },
                Occurrence::Atomic {
                    child: 2,
                    source: 3,
                    bidi_level: 1,
                },
            ],
        }
    }
    #[test]
    fn qualified_resource_and_same_reference_zero_metrics_round_trip() {
        let capture = actual();
        capture.validate().unwrap();
        assert_eq!(
            parse(&serde_json::to_string(&capture).unwrap()).unwrap(),
            capture
        );
        let mut upright = capture;
        upright.text_orientation = "upright".into();
        assert!(upright.validate().is_err());
        upright.direction = "ltr".into();
        for row in &mut upright.order {
            if let Occurrence::Atomic { bidi_level, .. } = row {
                *bidi_level = 0;
            }
        }
        upright.validate().unwrap();
    }
    #[test]
    fn html_vertical_alignment_and_raw_svg_default_are_separate_authorities() {
        for raw_default in ["auto", "baseline"] {
            let mut capture = actual();
            capture.alignment_baseline = raw_default.into();
            let parsed = parse(&serde_json::to_string(&capture).unwrap()).unwrap();
            assert_eq!(parsed.alignment_baseline, raw_default);
            assert_eq!(parsed.vertical_align, "baseline");
        }
    }
    #[test]
    fn missing_foreign_asymmetric_and_reference_authority_never_defaults() {
        let value = serde_json::to_value(actual()).unwrap();
        for field in [
            "font",
            "reference",
            "order",
            "before",
            "textOrientation",
            "verticalAlign",
        ] {
            let mut raw = value.clone();
            raw.as_object_mut().unwrap().remove(field);
            assert!(parse(&raw.to_string()).is_err(), "{field}");
        }
        for (field, bad) in [
            ("profile", serde_json::json!(2)),
            ("reference", serde_json::json!("alphabetic")),
            ("before", serde_json::json!(3)),
            ("after", serde_json::json!(-3)),
            ("dominantBaseline", serde_json::json!("alphabetic")),
            ("alignmentBaseline", serde_json::json!("central")),
            ("verticalAlign", serde_json::json!("middle")),
            ("lineFit", serde_json::json!("cap")),
        ] {
            let mut raw = value.clone();
            raw[field] = bad;
            assert!(parse(&raw.to_string()).is_err(), "{field}");
        }
        for (field, bad) in [
            (
                "sha256",
                serde_json::json!(
                    "f0a92cd0cc45735591c9b5b1fa8aecd5194e8dc518895ca22af94a46c23550dc"
                ),
            ),
            ("loaded", serde_json::json!(false)),
            ("size", serde_json::json!(11)),
            ("family", serde_json::json!("Arial")),
        ] {
            let mut raw = value.clone();
            raw["font"][field] = bad;
            assert!(parse(&raw.to_string()).is_err(), "{field}");
        }
        let mut raw = value;
        raw["order"][1]["reference"] = serde_json::json!("alphabetic");
        assert!(parse(&raw.to_string()).is_err());
    }
}
