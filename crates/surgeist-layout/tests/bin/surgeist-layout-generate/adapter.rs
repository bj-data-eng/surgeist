//! Layout fixture semantics supplied to the generic browser engine.

use crate::horizontal_br_font_band_profile;
use crate::measurement::{
    BoxObservationRequest, FLEX_COMPOSITION_SOURCE, FLEX_COMPOSITION_SOURCE_SHA256,
};
use crate::original_subgrid_body_profile;
use crate::prepared_atomic_percentage_profile;
use crate::xml::{escape_attr, generate_xml};
use crate::{empty_line_br_font_band_profile, vertical_br_font_band_profile};
use crate::{
    original_block_axes_profile, original_flex_axes_profile, original_flex_parent_profile,
    original_grid_axes_profile, original_subgrid_axes_profile,
};
use serde_json::Value;

#[cfg(test)]
pub(super) const TEST_HELPER_SOURCE: &str =
    include_str!("../../layout/browser_parity/scripts/gentest/test_helper.js");
#[cfg(test)]
pub(super) const TEST_BASE_STYLE_SOURCE: &str =
    include_str!("../../layout/browser_parity/scripts/gentest/test_base_style.css");
pub(super) const GRID_TEMPLATE_AREA_CAPTURE_SCRIPT: &str = r#"(() => {
  if (window.__surgeistGridTemplateAreaCaptureInstalled) return true;

  function parseSurgeistGridTemplateAreas(input) {
    if (!input || input === "none") return undefined;
    const rows = Array.from(input.matchAll(/"([^"]*)"/g), match => match[1].trim());
    if (rows.length === 0) return undefined;
    return rows.map(row => row.split(/\s+/).map(cell => /^\.+$/.test(cell) ? null : cell));
  }

  function authoredSurgeistGridTemplateAreas(element) {
    const computedStyle = getComputedStyle(element);
    if (element.style.gridTemplateAreas) return element.style.gridTemplateAreas;
    if (typeof authoredStyleValue === "function") {
      const authored = authoredStyleValue(element, "gridTemplateAreas", computedStyle);
      if (authored) return authored;
      if (computedStyle.gridTemplateAreas && computedStyle.gridTemplateAreas !== "none") {
        return computedStyle.gridTemplateAreas;
      }
    }
    return "";
  }

  const originalDescribeElement = describeElement;
  describeElement = function(element, expectedElement = null) {
    const data = originalDescribeElement(element, expectedElement);
    if (data && data.style) {
      data.style.gridTemplateAreas = parseSurgeistGridTemplateAreas(
        authoredSurgeistGridTemplateAreas(element)
      );
    }
    return data;
  };

  window.__surgeistGridTemplateAreaCaptureInstalled = true;
  return true;
})()"#;

// Inspect parsed declarations under the authenticated inline-and-base profile.
// Physical border fields cannot represent logical declaration precedence.
const LOGICAL_BORDER_CAPTURE_GUARD: &str = r#"(() => {
  if (typeof captureRootContext !== "function" || window.__surgeistLogicalBorderCaptureInstalled) return;

  const readRootContext = captureRootContext;
  captureRootContext = function(root) {
    const context = readRootContext(root);
    for (const element of [root, ...root.querySelectorAll("[style]")]) {
      if (Array.from(element.style).some(property => /^border-(block|inline)(-|$)/.test(property))) {
        throw new Error("Root context: UnrepresentedLogicalBorder");
      }
    }
    return context;
  };
  window.__surgeistLogicalBorderCaptureInstalled = true;
})()"#;

#[cfg(test)]
pub(super) fn browser_fixture_document(raw: &str, base_url: &str) -> Result<String, String> {
    fixture_document(raw, base_url)
}

fn fixture_document(raw: &str, base_url: &str) -> Result<String, String> {
    let head_injection = format!("<base href=\"{}\">", escape_attr(base_url));

    let lower = raw.to_ascii_lowercase();
    if let Some(index) = lower.find("<head>") {
        let insert_at = index + "<head>".len();
        let mut html = String::with_capacity(raw.len() + head_injection.len());
        html.push_str(&raw[..insert_at]);
        html.push_str(&head_injection);
        html.push_str(&raw[insert_at..]);
        Ok(html)
    } else if let Some(index) = lower.find("<html") {
        let html_tag_end = raw[index..]
            .find('>')
            .ok_or_else(|| "fixture html tag is missing closing `>`".to_string())?
            + index
            + 1;
        Ok(format!(
            "{}<head>{head_injection}</head>{}",
            &raw[..html_tag_end],
            &raw[html_tag_end..]
        ))
    } else if lower.starts_with("<!doctype") {
        let doctype_end = raw
            .find('>')
            .ok_or_else(|| "fixture doctype is missing closing `>`".to_string())?
            + 1;
        Ok(format!(
            "{}<html><head>{head_injection}</head><body>{}</body></html>",
            &raw[..doctype_end],
            &raw[doctype_end..]
        ))
    } else {
        Ok(format!(
            "<html><head>{head_injection}</head><body>{raw}</body></html>"
        ))
    }
}

#[cfg(test)]
pub(super) fn browser_document_write_script(html: &str) -> String {
    let html = serde_json::to_string(html).expect("serializing HTML should not fail");
    format!(
        "(() => {{ document.open(); document.write({html}); document.close(); return true; }})()"
    )
}

pub(super) use crate::fixture_variants::fixture_cases;

use std::collections::BTreeSet;
use std::fmt;
use std::path::{Component, Path};

use surgeist_generator::RelativePath;
use surgeist_generator::browser::{
    BrowserCorpusAdapter, CaseOutcome, FixtureInput, FixtureSpec, PreparedFixture,
    ResourceDependencies,
};

pub(super) struct LayoutAdapter;

#[derive(Debug)]
pub(super) enum AdapterError {
    Preparation(String),
    Transport {
        fixture: String,
    },
    Envelope {
        fixture: String,
        source: crate::envelope::EnvelopeError,
    },
    Declaration {
        fixture: String,
        detail: String,
    },
    Measurement(Box<crate::measurement::MeasurementError>),
}

impl fmt::Display for AdapterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Preparation(message) => formatter.write_str(message),
            Self::Transport { fixture } => write!(
                formatter,
                "{fixture}: measurement helper must return a JSON string"
            ),
            Self::Envelope { fixture, source } => write!(formatter, "{fixture}: {source}"),
            Self::Declaration { fixture, detail } => {
                write!(formatter, "{fixture}: invalid measurement cases: {detail}")
            }
            Self::Measurement(source) => source.fmt(formatter),
        }
    }
}

impl std::error::Error for AdapterError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Envelope { source, .. } => Some(source),
            Self::Measurement(source) => Some(source.as_ref()),
            _ => None,
        }
    }
}

impl BrowserCorpusAdapter for LayoutAdapter {
    type Error = AdapterError;

    fn prepare(&self, input: FixtureInput<'_>) -> Result<PreparedFixture, Self::Error> {
        self.prepare_with_observations(input, crate::measurement::BoxObservationRequest::Geometry)
    }

    fn lower(
        &self,
        fixture: &FixtureSpec,
        measurement: Value,
    ) -> Result<Vec<CaseOutcome>, Self::Error> {
        self.lower_with_observations(
            fixture,
            measurement,
            crate::measurement::BoxObservationRequest::Geometry,
        )
    }
}

impl LayoutAdapter {
    pub fn prepare_with_observations(
        &self,
        input: FixtureInput<'_>,
        request: crate::measurement::BoxObservationRequest,
    ) -> Result<PreparedFixture, AdapterError> {
        let request_path = RelativePath::new(crate::measurement::OBSERVATION_REQUEST_INPUT)
            .map_err(|error| AdapterError::Preparation(error.to_string()))?;
        admit_box_observation_request(input.input_bytes(&request_path), request)?;
        if request == BoxObservationRequest::FlexComposition
            && (input.fixture().source().as_str() != FLEX_COMPOSITION_SOURCE
                || surgeist_generator::Sha256Digest::from_bytes(input.source_bytes()).as_str()
                    != FLEX_COMPOSITION_SOURCE_SHA256)
        {
            return Err(AdapterError::Preparation(
                "Flex composition requires its registered original source".into(),
            ));
        }
        if request == BoxObservationRequest::OriginalFlexParent
            && !original_flex_parent_profile::source_digest(input.fixture().source().as_str())
                .is_some_and(|expected| {
                    surgeist_generator::Sha256Digest::from_bytes(input.source_bytes()).as_str()
                        == expected
                })
        {
            return Err(AdapterError::Preparation(
                "original Flex parent capture requires its exact source".into(),
            ));
        }
        if request == BoxObservationRequest::OriginalPercentageFlexParent
            && (input.fixture().source().as_str()
                != original_flex_parent_profile::percentage::SOURCE
                || surgeist_generator::Sha256Digest::from_bytes(input.source_bytes()).as_str()
                    != original_flex_parent_profile::percentage::SOURCE_SHA256)
        {
            return Err(AdapterError::Preparation(
                "original percentage Flex parent capture requires its exact source".into(),
            ));
        }
        if let Some(capture) = original_axis_capture(request)
            && (capture.source_digest)(input.fixture().source().as_str())
                != Some(surgeist_generator::Sha256Digest::from_bytes(input.source_bytes()).as_str())
        {
            return Err(AdapterError::Preparation(
                "original axis capture requires its exact source".into(),
            ));
        }
        if request == BoxObservationRequest::OriginalSubgridBody
            && (input.fixture().source().as_str() != original_subgrid_body_profile::SOURCE
                || surgeist_generator::Sha256Digest::from_bytes(input.source_bytes()).as_str()
                    != original_subgrid_body_profile::SOURCE_SHA256)
        {
            return Err(AdapterError::Preparation(
                "original subgrid body capture requires its exact source".into(),
            ));
        }
        if request == BoxObservationRequest::PreparedAtomicPercentage
            && (input.fixture().source().as_str() != prepared_atomic_percentage_profile::SOURCE
                || surgeist_generator::Sha256Digest::from_bytes(input.source_bytes()).as_str()
                    != prepared_atomic_percentage_profile::SOURCE_SHA256)
        {
            return Err(AdapterError::Preparation(
                "prepared atomic percentage capture requires its exact source".into(),
            ));
        }
        if request == BoxObservationRequest::HorizontalBrFontBand
            && (input.fixture().source().as_str() != horizontal_br_font_band_profile::SOURCE
                || surgeist_generator::Sha256Digest::from_bytes(input.source_bytes()).as_str()
                    != horizontal_br_font_band_profile::SOURCE_SHA256)
        {
            return Err(AdapterError::Preparation(
                "horizontal break font-band capture requires its exact source".into(),
            ));
        }
        if request == BoxObservationRequest::VerticalBrFontBand
            && !vertical_br_font_band_profile::source_digest(input.fixture().source().as_str())
                .is_some_and(|expected| {
                    surgeist_generator::Sha256Digest::from_bytes(input.source_bytes()).as_str()
                        == expected
                })
        {
            return Err(AdapterError::Preparation(
                "vertical break font-band capture requires its exact source".into(),
            ));
        }
        if request == BoxObservationRequest::EmptyLineBrFontBand
            && !empty_line_br_font_band_profile::source_digest(input.fixture().source().as_str())
                .is_some_and(|expected| {
                    surgeist_generator::Sha256Digest::from_bytes(input.source_bytes()).as_str()
                        == expected
                })
        {
            return Err(AdapterError::Preparation(
                "empty-line break capture requires its exact source".into(),
            ));
        }
        let source = std::str::from_utf8(input.source_bytes()).map_err(|error| {
            AdapterError::Preparation(format!("fixture source is not UTF-8: {error}"))
        })?;
        let helper_path = RelativePath::new("scripts/gentest/test_helper.js")
            .map_err(|error| AdapterError::Preparation(error.to_string()))?;
        let style_path = RelativePath::new("scripts/gentest/test_base_style.css")
            .map_err(|error| AdapterError::Preparation(error.to_string()))?;
        let helper = input.input_bytes(&helper_path).ok_or_else(|| {
            AdapterError::Preparation("captured measurement helper is missing".to_string())
        })?;
        let style = input.input_bytes(&style_path).ok_or_else(|| {
            AdapterError::Preparation("captured base stylesheet is missing".to_string())
        })?;
        let helper = std::str::from_utf8(helper).map_err(|error| {
            AdapterError::Preparation(format!("measurement helper is not UTF-8: {error}"))
        })?;
        let style = std::str::from_utf8(style).map_err(|error| {
            AdapterError::Preparation(format!("base stylesheet is not UTF-8: {error}"))
        })?;
        let document =
            fixture_document(source, input.base_url()).map_err(AdapterError::Preparation)?;
        let resources = match linked_resources(input.fixture().source(), source) {
            Ok(paths) => ResourceDependencies::Paths(paths),
            Err(reason) => ResourceDependencies::Invalid { reason },
        };
        let font_path = RelativePath::new("fonts/Ahem.ttf")
            .map_err(|error| AdapterError::Preparation(error.to_string()))?;
        let font_bytes = if matches!(
            request,
            BoxObservationRequest::EmptyLineBrFontBand
                | BoxObservationRequest::OriginalFlexParent
                | BoxObservationRequest::OriginalPercentageFlexParent
        ) || original_axis_capture(request).is_some()
        {
            &[]
        } else {
            input.input_bytes(&font_path).ok_or_else(|| {
                AdapterError::Preparation("captured pinned font resource is missing".into())
            })?
        };
        let setup = helper_setup_expression(
            helper,
            font_bytes,
            source,
            style,
            input.fixture().source(),
            request,
        )?;
        let measurement = if let Some((path, script)) = request.proof_script() {
            let path = RelativePath::new(path)
                .map_err(|error| AdapterError::Preparation(error.to_string()))?;
            if input.input_bytes(&path) != Some(script.as_bytes()) {
                return Err(AdapterError::Preparation(
                    "observation proof script changed or is missing".into(),
                ));
            }
            if request == BoxObservationRequest::OriginalFlexParent {
                let font = crate::pinned_font::AHEM_STYLESHEET_WOFF2;
                if surgeist_generator::Sha256Digest::from_bytes(font).as_str()
                    != crate::pinned_font::AHEM_STYLESHEET_WOFF2_SHA256
                {
                    return Err(AdapterError::Preparation(
                        "pinned stylesheet Ahem changed".into(),
                    ));
                }
                let arguments = serde_json::to_string(&(source, input.base_url(), style, font))
                    .map_err(|error| AdapterError::Preparation(error.to_string()))?;
                format!("globalThis.captureOriginalFlexParent(...{arguments})")
            } else if request == BoxObservationRequest::OriginalPercentageFlexParent {
                let arguments = serde_json::to_string(&(source, input.base_url(), style))
                    .map_err(|error| AdapterError::Preparation(error.to_string()))?;
                format!("globalThis.captureOriginalFlexParent(...{arguments})")
            } else if let Some(capture) = original_axis_capture(request) {
                let arguments = serde_json::to_string(&(source, input.base_url(), style))
                    .map_err(|error| AdapterError::Preparation(error.to_string()))?;
                format!("globalThis.{}(...{arguments})", capture.function)
            } else if request == BoxObservationRequest::OriginalSubgridBody {
                let serialize = |value| {
                    serde_json::to_string(value)
                        .map_err(|error| AdapterError::Preparation(error.to_string()))
                };
                let source = serialize(source)?;
                let base = serialize(input.base_url())?;
                format!("({script})({source}, {base})")
            } else if request == BoxObservationRequest::PreparedAtomicPercentage {
                let bytes = serde_json::to_string(font_bytes)
                    .map_err(|error| AdapterError::Preparation(error.to_string()))?;
                format!("globalThis.captureOriginalPreparedAtomicPercentage({bytes})")
            } else if request == BoxObservationRequest::HorizontalBrFontBand {
                format!("({script})()")
            } else if request == BoxObservationRequest::VerticalBrFontBand {
                let source = serde_json::to_string(source)
                    .map_err(|error| AdapterError::Preparation(error.to_string()))?;
                let style = serde_json::to_string(style)
                    .map_err(|error| AdapterError::Preparation(error.to_string()))?;
                format!("globalThis.captureOriginalVerticalBrFontBand({source}, {style})")
            } else if request == BoxObservationRequest::EmptyLineBrFontBand {
                let font = crate::pinned_font::AHEM_STYLESHEET_WOFF2;
                let digest = crate::pinned_font::AHEM_STYLESHEET_WOFF2_SHA256;
                if surgeist_generator::Sha256Digest::from_bytes(font).as_str() != digest {
                    return Err(AdapterError::Preparation(
                        "pinned stylesheet Ahem changed".into(),
                    ));
                }
                let arguments = serde_json::to_string(&(
                    source,
                    style,
                    crate::pinned_font::AHEM_STYLESHEET_WOFF2_ID,
                    digest,
                    font,
                ))
                .map_err(|error| AdapterError::Preparation(error.to_string()))?;
                format!("globalThis.captureOriginalEmptyLineBrFontBand(...{arguments})")
            } else {
                let font = crate::pinned_font::AHEM_STYLESHEET_WOFF2_SHA256;
                format!("({script})({font:?})")
            }
        } else {
            "getTestData()".into()
        };
        PreparedFixture::new(
            document,
            "document.readyState !== 'loading' && !!document.body".to_string(),
            setup,
            measurement,
            resources,
        )
        .map_err(|error| AdapterError::Preparation(error.to_string()))
    }

    pub fn lower_with_observations(
        &self,
        fixture: &FixtureSpec,
        measurement: Value,
        request: crate::measurement::BoxObservationRequest,
    ) -> Result<Vec<CaseOutcome>, AdapterError> {
        let raw = measurement
            .as_str()
            .ok_or_else(|| AdapterError::Transport {
                fixture: fixture.name().to_string(),
            })?;
        let measurements =
            crate::envelope::decode(raw).map_err(|source| AdapterError::Envelope {
                fixture: fixture.name().to_string(),
                source,
            })?;
        if fixture.cases().len() != fixture_cases().len() {
            return Err(AdapterError::Declaration {
                fixture: fixture.name().to_string(),
                detail: format!("expected four variants, received {}", fixture.cases().len()),
            });
        }
        fixture_cases()
            .into_iter()
            .zip(fixture.cases())
            .zip(measurements)
            .enumerate()
            .map(|(index, (((variant, _), case), raw))| {
                if case.variant() != variant {
                    return Err(AdapterError::Declaration {
                        fixture: fixture.name().to_string(),
                        detail: format!(
                            "cases[{index}].variant must be {variant}, received {}",
                            case.variant()
                        ),
                    });
                }
                let measurement =
                    crate::measurement::decode(raw.get(), case.id(), variant, request)
                        .map_err(|source| AdapterError::Measurement(Box::new(source)))?;
                Ok(match measurement {
                    crate::measurement::DecodedMeasurement::Unsupported { reason } => {
                        CaseOutcome::Unsupported {
                            case_id: case.id().to_string(),
                            reason,
                        }
                    }
                    crate::measurement::DecodedMeasurement::Supported(measurement) => {
                        CaseOutcome::Generated {
                            case_id: case.id().to_string(),
                            bytes: generate_xml(case.id(), &measurement).into_bytes(),
                        }
                    }
                })
            })
            .collect()
    }
}

fn admit_box_observation_request(
    bytes: Option<&[u8]>,
    expected: crate::measurement::BoxObservationRequest,
) -> Result<(), AdapterError> {
    let actual =
        crate::measurement::BoxObservationRequest::from_input_bytes(bytes).map_err(|detail| {
            AdapterError::Preparation(format!(
                "{}: {detail}",
                crate::measurement::OBSERVATION_REQUEST_INPUT
            ))
        })?;
    if actual != expected {
        return Err(AdapterError::Preparation(format!(
            "{}: registered observation request {actual:?} differs from requested {expected:?}",
            crate::measurement::OBSERVATION_REQUEST_INPUT,
        )));
    }
    Ok(())
}

fn linked_resources(source: &RelativePath, raw: &str) -> Result<Vec<RelativePath>, String> {
    let mut references = BTreeSet::new();
    for marker in ["src=\"", "src='", "href=\"", "href='"] {
        let quote = marker
            .chars()
            .last()
            .expect("resource marker contains a quote");
        for (start, _) in raw.match_indices(marker) {
            let value = &raw[start + marker.len()..];
            let end = value.find(quote).ok_or_else(|| {
                format!(
                    "{} has an unterminated linked-resource attribute",
                    source.as_str()
                )
            })?;
            let reference = value[..end].split(['?', '#']).next().unwrap_or_default();
            if reference.is_empty()
                || reference.starts_with('/')
                || reference.contains("://")
                || reference.starts_with("data:")
            {
                continue;
            }
            let path = resolve_reference(source, reference)?;
            if matches!(
                path.as_str(),
                "scripts/gentest/test_helper.js" | "scripts/gentest/test_base_style.css"
            ) {
                continue;
            }
            references.insert(path);
        }
    }
    Ok(references.into_iter().collect())
}

fn resolve_reference(source: &RelativePath, reference: &str) -> Result<RelativePath, String> {
    let mut resolved = Path::new(source.as_str())
        .parent()
        .unwrap_or_else(|| Path::new(""))
        .components()
        .filter_map(|component| match component {
            Component::Normal(segment) => Some(segment.to_os_string()),
            _ => None,
        })
        .collect::<Vec<_>>();
    for component in Path::new(reference).components() {
        match component {
            Component::Normal(segment) => resolved.push(segment.to_os_string()),
            Component::CurDir => {}
            Component::ParentDir => {
                if resolved.pop().is_none() {
                    return Err(format!(
                        "linked resource {reference:?} escapes the corpus root"
                    ));
                }
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(format!(
                    "linked resource {reference:?} must be corpus-relative"
                ));
            }
        }
    }
    let path = resolved.into_iter().collect::<std::path::PathBuf>();
    RelativePath::new(path.to_string_lossy().replace('\\', "/")).map_err(|error| error.to_string())
}

struct AxisCapture {
    function: &'static str,
    script: &'static str,
    source_digest: fn(&str) -> Option<&'static str>,
    accepts_helper: fn(&str) -> bool,
}

fn original_axis_capture(request: BoxObservationRequest) -> Option<AxisCapture> {
    match request {
        BoxObservationRequest::OriginalBlockAxes => Some(AxisCapture {
            function: "captureOriginalBlockAxes",
            script: original_block_axes_profile::SCRIPT,
            source_digest: original_block_axes_profile::source_digest,
            accepts_helper: original_block_axes_profile::accepts_helper,
        }),
        BoxObservationRequest::OriginalFlexAxes => Some(AxisCapture {
            function: "captureOriginalFlexAxes",
            script: original_flex_axes_profile::SCRIPT,
            source_digest: original_flex_axes_profile::source_digest,
            accepts_helper: original_flex_axes_profile::accepts_helper,
        }),
        BoxObservationRequest::OriginalGridAxes => Some(AxisCapture {
            function: "captureOriginalGridAxes",
            script: original_grid_axes_profile::SCRIPT,
            source_digest: original_grid_axes_profile::source_digest,
            accepts_helper: original_grid_axes_profile::accepts_helper,
        }),
        BoxObservationRequest::OriginalSubgridAxes => Some(AxisCapture {
            function: "captureOriginalSubgridAxes",
            script: original_subgrid_axes_profile::SCRIPT,
            source_digest: original_subgrid_axes_profile::source_digest,
            accepts_helper: original_subgrid_axes_profile::accepts_helper,
        }),
        _ => None,
    }
}

fn helper_setup_expression(
    helper: &str,
    bytes: &[u8],
    source: &str,
    base_style: &str,
    source_path: &RelativePath,
    request: BoxObservationRequest,
) -> Result<String, AdapterError> {
    // This capture version qualifies one exact zero-edge base document profile.
    // A changed stylesheet needs an explicit new source-profile qualification.
    if surgeist_generator::Sha256Digest::from_bytes(base_style.as_bytes()).to_string()
        != crate::pinned_font::ROOT_CONTEXT_STYLESHEET_SHA256
    {
        return Err(AdapterError::Preparation(
            "root-context source profile does not qualify the captured base stylesheet".into(),
        ));
    }
    if let Some(capture) = original_axis_capture(request) {
        let digest = surgeist_generator::Sha256Digest::from_bytes(helper.as_bytes());
        if !(capture.accepts_helper)(digest.as_str()) {
            return Err(AdapterError::Preparation(
                "original axis capture helper changed".into(),
            ));
        }
        let helper = serde_json::to_string(&format!(
            "{helper}\n globalThis.{} = ({});",
            capture.function, capture.script
        ))
        .map_err(|error| AdapterError::Preparation(error.to_string()))?;
        return Ok(format!(
            "(() => {{ (0, eval)({helper});\n{GRID_TEMPLATE_AREA_CAPTURE_SCRIPT}; return true; }})()"
        ));
    }
    let flex_parent_script = match request {
        BoxObservationRequest::OriginalFlexParent => Some(original_flex_parent_profile::SCRIPT),
        BoxObservationRequest::OriginalPercentageFlexParent => {
            Some(original_flex_parent_profile::percentage::SCRIPT)
        }
        _ => None,
    };
    if let Some(script) = flex_parent_script {
        let digest = surgeist_generator::Sha256Digest::from_bytes(helper.as_bytes());
        if !original_flex_parent_profile::accepts_helper(digest.as_str()) {
            return Err(AdapterError::Preparation(
                "original Flex parent capture helper changed".into(),
            ));
        }
        let helper = serde_json::to_string(&format!(
            "{helper}\n globalThis.captureOriginalFlexParent = ({script});"
        ))
        .map_err(|error| AdapterError::Preparation(error.to_string()))?;
        return Ok(format!(
            "(() => {{ (0, eval)({helper});\n{GRID_TEMPLATE_AREA_CAPTURE_SCRIPT}; return true; }})()"
        ));
    }
    if request == BoxObservationRequest::OriginalSubgridBody
        && !original_subgrid_body_profile::accepts_helper(
            surgeist_generator::Sha256Digest::from_bytes(helper.as_bytes()).as_str(),
        )
    {
        return Err(AdapterError::Preparation(
            "original subgrid body capture helper changed".into(),
        ));
    }
    if request == BoxObservationRequest::PreparedAtomicPercentage
        && !prepared_atomic_percentage_profile::accepts_helper(
            surgeist_generator::Sha256Digest::from_bytes(helper.as_bytes()).as_str(),
        )
    {
        return Err(AdapterError::Preparation(
            "prepared atomic percentage capture helper changed".into(),
        ));
    }
    if request == BoxObservationRequest::HorizontalBrFontBand
        && !horizontal_br_font_band_profile::accepts_helper(
            surgeist_generator::Sha256Digest::from_bytes(helper.as_bytes()).as_str(),
        )
    {
        return Err(AdapterError::Preparation(
            "horizontal break font-band capture helper changed".into(),
        ));
    }
    if request == BoxObservationRequest::VerticalBrFontBand
        && !vertical_br_font_band_profile::accepts_helper(
            surgeist_generator::Sha256Digest::from_bytes(helper.as_bytes()).as_str(),
        )
    {
        return Err(AdapterError::Preparation(
            "vertical break font-band capture helper changed".into(),
        ));
    }
    if request == BoxObservationRequest::EmptyLineBrFontBand {
        if !empty_line_br_font_band_profile::accepts_helper(
            surgeist_generator::Sha256Digest::from_bytes(helper.as_bytes()).as_str(),
        ) {
            return Err(AdapterError::Preparation(
                "empty-line break capture helper changed".into(),
            ));
        }
        let script = empty_line_br_font_band_profile::SCRIPT;
        let helper = serde_json::to_string(&format!(
            "{helper}\n globalThis.captureOriginalEmptyLineBrFontBand = ({script});"
        ))
        .map_err(|error| AdapterError::Preparation(error.to_string()))?;
        return Ok(format!(
            "(() => {{ (0, eval)({helper});\n{GRID_TEMPLATE_AREA_CAPTURE_SCRIPT};\n{LOGICAL_BORDER_CAPTURE_GUARD}; return true; }})()"
        ));
    }
    let specialized_helper;
    let helper = if request == BoxObservationRequest::PreparedAtomicPercentage {
        specialized_helper = format!(
            "{helper}\n globalThis.captureOriginalPreparedAtomicPercentage = ({});",
            prepared_atomic_percentage_profile::SCRIPT,
        );
        specialized_helper.as_str()
    } else if request == BoxObservationRequest::VerticalBrFontBand {
        specialized_helper = format!(
            "{helper}\n globalThis.captureOriginalVerticalBrFontBand = ({});",
            vertical_br_font_band_profile::SCRIPT,
        );
        specialized_helper.as_str()
    } else {
        helper
    };
    let helper = serde_json::to_string(helper).map_err(|error| {
        AdapterError::Preparation(format!("failed to encode helper script: {error}"))
    })?;
    if request == BoxObservationRequest::OriginalSubgridBody {
        return Ok(format!(
            "(() => {{ (0, eval)({helper});\n{GRID_TEMPLATE_AREA_CAPTURE_SCRIPT}; return true; }})()"
        ));
    }
    let digest = surgeist_generator::Sha256Digest::from_bytes(bytes);
    if digest.to_string() != "f0a92cd0cc45735591c9b5b1fa8aecd5194e8dc518895ca22af94a46c23550dc" {
        return Err(AdapterError::Preparation(
            "pinned Ahem fixture resource changed".into(),
        ));
    }
    let bytes = serde_json::to_string(bytes)
        .map_err(|error| AdapterError::Preparation(error.to_string()))?;
    let depth = source_path.as_str().split('/').count().saturating_sub(1);
    let prefix = "../".repeat(depth);
    let helper_reference = format!("{prefix}scripts/gentest/test_helper.js");
    let style_reference = format!("{prefix}scripts/gentest/test_base_style.css");
    let font_reference = format!("{prefix}fonts/Ahem.ttf");
    let mut context = vec![
        Some(source),
        Some(base_style),
        Some(helper_reference.as_str()),
        Some(style_reference.as_str()),
    ];
    let prepare_image = match request {
        BoxObservationRequest::PreparedRanges => {
            context.push(Some(font_reference.as_str()));
            ""
        }
        BoxObservationRequest::FlexComposition => {
            context.extend([None, Some("fixed-sideways-root-v1")]);
            " await prepareFlexCompositionImage();"
        }
        _ => "",
    };
    let context = serde_json::to_string(&context)
        .map_err(|error| AdapterError::Preparation(error.to_string()))?;
    let install_helper = if matches!(
        request,
        BoxObservationRequest::PreparedAtomicPercentage | BoxObservationRequest::VerticalBrFontBand
    ) {
        format!("(0, eval)({helper});")
    } else {
        format!("if (typeof getTestData !== 'function') {{ (0, eval)({helper}); }}")
    };
    Ok(format!(
        "(async () => {{ const pinnedFontBytes = {bytes}; {install_helper}\n{GRID_TEMPLATE_AREA_CAPTURE_SCRIPT};\n{LOGICAL_BORDER_CAPTURE_GUARD};\nif (typeof configureRootContextCapture === 'function') {{ configureRootContextCapture(...{context}); await document.fonts.ready; }}{prepare_image} if (typeof prepareNamedReferenceFont === 'function') {{ await prepareNamedReferenceFont(); }} if (typeof preparePinnedTextFont === 'function') {{ await preparePinnedTextFont(pinnedFontBytes); }} return true; }})()"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use surgeist_generator::browser::{CaseSpec, FixtureStatus};

    include!("script_custody_admission_tests.rs");

    fn fixture() -> FixtureSpec {
        FixtureSpec::new(
            "example".to_string(),
            RelativePath::new("html/grid/example.html").unwrap(),
            fixture_cases()
                .into_iter()
                .map(|(variant, _)| {
                    CaseSpec::new(
                        format!("example__{variant}"),
                        variant.to_string(),
                        RelativePath::new(format!("xml/grid/example__{variant}.xml")).unwrap(),
                    )
                    .unwrap()
                })
                .collect(),
            FixtureStatus::Active,
        )
        .unwrap()
    }

    fn unsupported(reason: &str) -> Value {
        serde_json::json!({"layoutInput": "unsupported", "unsupportedReason": reason})
    }

    fn supported_box() -> Value {
        let geometry = serde_json::json!({
            "x": 0, "y": 0, "width": 10, "height": 20,
            "scrollWidth": 10, "scrollHeight": 20,
            "clientWidth": 10, "clientHeight": 20,
        });
        serde_json::json!({
            "layoutInput": "box", "tagName": "div", "gridAlignmentProfile":{"profile":1,"owners":[]},"useRounding": true,
            "viewport": {"rootContext":{"kind":"block-flow-auto","parentInlineExtent":800,"parentDirection":"ltr","fallback":{"initialContainingBlock":{"width":800,"height":600},"nearestScrollport":"none","containingLimits":{"width":"unbounded","height":"unbounded"}}}},
            "style": {"position":"relative","display": "block", "boxSizing": "border-box", "writingMode": "horizontal-tb", "direction": "ltr"},
            "unroundedLayout": geometry.clone(),
            "smartRoundedLayout": geometry.clone(),
            "worldRoundedLayout": geometry.clone(),
            "naivelyRoundedLayout": geometry,
            "children": [],
        })
    }

    fn envelope(nodes: [Value; 4]) -> Value {
        let mut fields = serde_json::Map::new();
        fields.insert(
            "schemaVersion".to_string(),
            Value::from(crate::envelope::VERSION),
        );
        for ((_, key), node) in fixture_cases().into_iter().zip(nodes) {
            fields.insert(key.to_string(), node);
        }
        Value::String(Value::Object(fields).to_string())
    }

    #[test]
    fn preparation_binds_registered_request_and_refuses_missing_wrong_or_unrequested_input() {
        use crate::measurement::BoxObservationRequest::{ElementScrollSizes, Geometry};
        assert!(admit_box_observation_request(None, Geometry).is_ok());
        assert!(
            admit_box_observation_request(ElementScrollSizes.input_bytes(), ElementScrollSizes)
                .is_ok()
        );
        for (bytes, expected) in [
            (None, ElementScrollSizes),
            (ElementScrollSizes.input_bytes(), Geometry),
            (Some(b"{}".as_slice()), Geometry),
            (Some(b"{}".as_slice()), ElementScrollSizes),
        ] {
            assert!(matches!(
                admit_box_observation_request(bytes, expected),
                Err(AdapterError::Preparation(_))
            ));
        }
    }

    fn element_size_box() -> Value {
        let mut node = supported_box();
        node["style"]["position"] = serde_json::json!("absolute");
        node["style"]["overflowX"] = serde_json::json!("visible");
        node["style"]["overflowY"] = serde_json::json!("visible");
        node["style"]["scrollbarGutter"] = serde_json::json!("auto");
        node["viewport"] = serde_json::json!({"rootContext":{"kind":"absolute-root-local",
            "containingWidth":800,"containingHeight":600,"containingDirection":"ltr",
            "hypotheticalDirection":"ltr","hypotheticalDisplay":"block",
            "hypotheticalAnchor":"inline-start","hypotheticalInlineOffset":0,
            "fallback":{"initialContainingBlock":{"width":800,"height":600},
                "nearestScrollport":"none","containingLimits":{"width":"unbounded","height":"unbounded"}}}});
        node
    }

    #[test]
    fn sideways_flex_capture_requires_the_natural_size_of_its_replaced_leaf() {
        use BoxObservationRequest::{FlexComposition, Geometry};
        use serde_json::json;

        let mut root = element_size_box();
        root["style"]["display"] = json!("flex");
        root["style"]["writingMode"] = json!("sideways-rl");
        root["viewport"]["rootContext"]["hypotheticalDisplay"] = json!("flex");
        let mut leaf = supported_box();
        let fields = leaf.as_object_mut().unwrap();
        for field in ["gridAlignmentProfile", "useRounding", "viewport"] {
            fields.remove(field);
        }
        let mut image = leaf.clone();
        image["tagName"] = json!("img");
        root["children"] = json!([leaf, image, leaf, leaf]);
        let nodes = |node: &Value| envelope(std::array::from_fn(|_| node.clone()));
        let lower = |node: &Value, request| {
            LayoutAdapter.lower_with_observations(&fixture(), nodes(node), request)
        };

        let missing = lower(&root, FlexComposition);
        assert!(matches!(
            missing,
            Err(AdapterError::Measurement(error)) if error.field_path == "flexCompositionImage"
        ));

        root["flexCompositionImage"] = json!({"naturalWidth":80, "naturalHeight":40});
        for capture in lower(&root, FlexComposition).unwrap() {
            let CaseOutcome::Generated { bytes, .. } = capture else {
                panic!("expected a generated capture");
            };
            let document =
                roxmltree::Document::parse(std::str::from_utf8(&bytes).unwrap()).unwrap();
            let witness = document
                .root_element()
                .children()
                .find(|node| node.has_tag_name("flex-composition-image"))
                .unwrap();
            assert_eq!(witness.attribute("natural-width"), Some("80"));
            assert_eq!(witness.attribute("natural-height"), Some("40"));
        }
        assert!(matches!(
            lower(&root, Geometry),
            Err(AdapterError::Measurement(error)) if error.field_path == "flexCompositionImage"
        ));

        for (pointer, value) in [
            ("/children/1/tagName", json!("div")),
            ("/style/writingMode", json!("horizontal-tb")),
            (
                "/flexCompositionImage",
                json!({"naturalWidth":40,"naturalHeight":80}),
            ),
        ] {
            let mut changed = root.clone();
            *changed.pointer_mut(pointer).unwrap() = value;
            assert!(
                matches!(
                    lower(&changed, FlexComposition),
                    Err(AdapterError::Measurement(error)) if error.field_path == "flexCompositionImage"
                ),
                "{pointer}"
            );
        }
    }

    fn measured_leaf_variants() -> [Value; 4] {
        let font = "e450dd6e1bc6fb10ebc6edecef1182dcd8cad6e12e54c363ad4ba26f6b5db46c";
        let variants = fixture_cases();
        std::array::from_fn(|index| {
            let (variant, _) = variants[index];
            let (box_sizing, direction) = variant.rsplit_once('_').unwrap();
            let box_sizing = box_sizing.replace('_', "-");
            let mut root = element_size_box();
            root["style"]["display"] = "grid".into();
            root["style"]["direction"] = direction.into();
            root["style"]["boxSizing"] = box_sizing.as_str().into();
            root["viewport"]["rootContext"]["hypotheticalDisplay"] = "grid".into();

            let mut leaf = supported_box();
            let object = leaf.as_object_mut().unwrap();
            for field in ["gridAlignmentProfile", "useRounding", "viewport"] {
                object.remove(field);
            }
            leaf["style"]["direction"] = direction.into();
            leaf["style"]["boxSizing"] = box_sizing.as_str().into();
            leaf["style"]["overflowX"] = "visible".into();
            leaf["style"]["overflowY"] = "visible".into();
            leaf["style"]["scrollbarGutter"] = "auto".into();
            leaf["textContent"] = "HHHH\u{200b}HHHH".into();
            root["children"] = serde_json::json!([leaf]);
            root["measuredLeaf"] = serde_json::json!({
                "nodePath": "0.0",
                "fontResourceSha256": font,
                "sourceUnits": [72, 72, 72, 72, 8203, 72, 72, 72, 72],
                "advances": [40, 40, 80],
                "direction": direction,
                "boxSizing": box_sizing,
                "clientWidth": 10,
                "scrollWidth": 10,
            });
            root
        })
    }

    #[test]
    fn measured_grid_request_retains_the_leaf_witness_in_each_capture() {
        use crate::measurement::BoxObservationRequest;

        let request = BoxObservationRequest::from_input_bytes(Some(
            b"{\"schema_version\":1,\"element_scroll_sizes\":\"horizontal-zero-edge-divs\",\"measured_leaf\":\"ahem-10px-zwsp\"}\n",
        ))
        .expect("the measured Grid capture request must be admitted");
        let font = "e450dd6e1bc6fb10ebc6edecef1182dcd8cad6e12e54c363ad4ba26f6b5db46c";
        let nodes = measured_leaf_variants();
        let outcomes = LayoutAdapter
            .lower_with_observations(&fixture(), envelope(nodes), request)
            .expect("all four measured wire variants lower");
        assert_eq!(outcomes.len(), fixture_cases().len());
        for outcome in outcomes {
            let CaseOutcome::Generated { case_id, bytes } = outcome else {
                panic!("expected generated capture, received {outcome:?}");
            };
            let xml = std::str::from_utf8(&bytes).unwrap();
            let parsed = roxmltree::Document::parse(xml).unwrap();
            let root = parsed.root_element();
            assert_eq!(root.attribute("name"), Some(case_id.as_str()));
            let witness = root
                .children()
                .find(|node| node.has_tag_name("measured-leaf"))
                .expect("the measured leaf witness must survive lowering");
            assert_eq!(witness.attribute("node-path"), Some("0.0"));
            assert_eq!(witness.attribute("font-resource-sha256"), Some(font));
            assert_eq!(witness.attribute("first-advance"), Some("40"));
            assert_eq!(witness.attribute("second-advance"), Some("40"));
            assert_eq!(witness.attribute("full-advance"), Some("80"));
            assert!(
                root.children()
                    .any(|node| node.has_tag_name("element-scroll-sizes"))
            );
        }
    }

    #[test]
    fn measured_grid_lowers_sixteen_character_runs_and_checks_their_advances() {
        use crate::measurement::BoxObservationRequest::MeasuredLeaf;

        let text = "HHHHHHHHHHHHHHHH\u{200b}HHHHHHHHHHHHHHHH";
        let mut variants = measured_leaf_variants();
        for node in &mut variants {
            node["children"][0]["textContent"] = text.into();
            node["measuredLeaf"]["sourceUnits"] =
                serde_json::json!(text.encode_utf16().collect::<Vec<_>>());
            node["measuredLeaf"]["advances"] = serde_json::json!([160, 160, 320]);
        }
        let captures = LayoutAdapter
            .lower_with_observations(&fixture(), envelope(variants.clone()), MeasuredLeaf)
            .expect("the measured sixteen-character runs must lower");
        for capture in captures {
            let CaseOutcome::Generated { bytes, .. } = capture else {
                panic!("expected a generated capture");
            };
            let xml = std::str::from_utf8(&bytes).unwrap();
            let document = roxmltree::Document::parse(xml).unwrap();
            let witness = document
                .root_element()
                .children()
                .find(|node| node.has_tag_name("measured-leaf"))
                .unwrap();
            assert_eq!(witness.attribute("first-advance"), Some("160"));
            assert_eq!(witness.attribute("second-advance"), Some("160"));
            assert_eq!(witness.attribute("full-advance"), Some("320"));
        }
        variants[0]["measuredLeaf"]["advances"] = serde_json::json!([40, 40, 80]);
        assert!(matches!(
            LayoutAdapter.lower_with_observations(&fixture(), envelope(variants), MeasuredLeaf),
            Err(AdapterError::Measurement(error)) if error.field_path == "measuredLeaf"
        ));
    }

    #[test]
    fn measured_grid_capture_refuses_missing_unrequested_or_contradictory_witnesses() {
        use crate::measurement::BoxObservationRequest::{
            ElementScrollSizes, Geometry, MeasuredLeaf,
        };
        for request in [Geometry, ElementScrollSizes] {
            assert!(matches!(
                LayoutAdapter.lower_with_observations(&fixture(), envelope(measured_leaf_variants()), request),
                Err(AdapterError::Measurement(error)) if error.field_path == "measuredLeaf"
            ));
        }
        let mut absent = measured_leaf_variants();
        absent[0].as_object_mut().unwrap().remove("measuredLeaf");
        assert!(matches!(
            LayoutAdapter.lower_with_observations(&fixture(), envelope(absent), MeasuredLeaf),
            Err(AdapterError::Measurement(error)) if error.field_path == "measuredLeaf"
        ));
        let mut extra = measured_leaf_variants();
        let leaf = extra[0]["children"][0].clone();
        extra[0]["children"].as_array_mut().unwrap().push(leaf);
        assert!(matches!(
            LayoutAdapter.lower_with_observations(&fixture(), envelope(extra), MeasuredLeaf),
            Err(AdapterError::Measurement(error)) if error.field_path == "measuredLeaf"
        ));
        for (field, value) in [
            ("nodePath", serde_json::json!("0.1")),
            ("fontResourceSha256", serde_json::json!("different")),
            (
                "sourceUnits",
                serde_json::json!([72, 72, 72, 72, 32, 72, 72, 72, 72]),
            ),
            ("advances", serde_json::json!([40, 40, 81])),
            ("direction", serde_json::json!("rtl")),
            ("scrollWidth", serde_json::json!(11)),
        ] {
            let mut changed = measured_leaf_variants();
            changed[0]["measuredLeaf"][field] = value;
            assert!(
                matches!(
                    LayoutAdapter.lower_with_observations(&fixture(), envelope(changed), MeasuredLeaf),
                    Err(AdapterError::Measurement(error)) if error.field_path == "measuredLeaf"
                ),
                "{field}"
            );
        }
    }

    #[test]
    fn explicit_geometry_lowering_is_byte_identical_to_the_trait_front_door() {
        let measurement = envelope(std::array::from_fn(|_| element_size_box()));
        let old = LayoutAdapter
            .lower(&fixture(), measurement.clone())
            .unwrap();
        let explicit = LayoutAdapter
            .lower_with_observations(
                &fixture(),
                measurement,
                crate::measurement::BoxObservationRequest::Geometry,
            )
            .unwrap();
        for (old, explicit) in old.into_iter().zip(explicit) {
            match (old, explicit) {
                (
                    CaseOutcome::Generated {
                        case_id: old_id,
                        bytes: old_bytes,
                    },
                    CaseOutcome::Generated { case_id, bytes },
                ) => {
                    assert_eq!(case_id, old_id);
                    assert_eq!(bytes, old_bytes);
                }
                outcomes => panic!("expected complete generated geometry cases: {outcomes:?}"),
            }
        }
    }

    #[test]
    fn requested_element_sizes_lower_all_declared_variants_and_refuse_an_unsupported_case() {
        let mut unsupported = element_size_box();
        unsupported["style"]["overflowX"] = serde_json::json!("auto");
        let outcomes = LayoutAdapter
            .lower_with_observations(
                &fixture(),
                envelope([
                    element_size_box(),
                    unsupported,
                    element_size_box(),
                    element_size_box(),
                ]),
                crate::measurement::BoxObservationRequest::ElementScrollSizes,
            )
            .unwrap();
        assert_eq!(outcomes.len(), fixture().cases().len());
        for (index, outcome) in outcomes.into_iter().enumerate() {
            match outcome {
                CaseOutcome::Generated { case_id, bytes } => {
                    assert_ne!(index, 1);
                    assert_eq!(case_id, fixture().cases()[index].id());
                    let xml = std::str::from_utf8(&bytes).unwrap();
                    let document = roxmltree::Document::parse(xml).unwrap();
                    let table = document
                        .root_element()
                        .children()
                        .find(|node| node.has_tag_name("element-scroll-sizes"))
                        .unwrap();
                    assert_eq!(
                        table.attribute("profile"),
                        Some("horizontal-zero-edge-divs")
                    );
                    let row = table.first_element_child().unwrap();
                    assert_eq!(row.attribute("node-path"), Some("0"));
                    assert_eq!(row.attribute("client-width"), Some("10"));
                    assert_eq!(row.attribute("client-height"), Some("20"));
                    assert_eq!(row.attribute("scroll-width"), Some("10"));
                    assert_eq!(row.attribute("scroll-height"), Some("20"));
                    assert!(row.next_sibling_element().is_none());
                }
                CaseOutcome::Unsupported { case_id, reason } => {
                    assert_eq!(index, 1);
                    assert_eq!(case_id, fixture().cases()[index].id());
                    assert!(reason.contains("Visible, Clip or Hidden"), "{reason}");
                }
            }
        }
    }

    #[test]
    fn mixed_measurements_keep_declared_case_order_and_observed_geometry() {
        let outcomes = LayoutAdapter
            .lower(
                &fixture(),
                envelope([
                    supported_box(),
                    unsupported("second"),
                    supported_box(),
                    unsupported("fourth"),
                ]),
            )
            .unwrap();
        for (index, outcome) in outcomes.into_iter().enumerate() {
            match outcome {
                CaseOutcome::Generated { case_id, bytes } => {
                    assert!(matches!(index, 0 | 2));
                    assert_eq!(case_id, fixture().cases()[index].id());
                    let xml = std::str::from_utf8(&bytes).unwrap();
                    let document = roxmltree::Document::parse(xml).unwrap();
                    let expected = document
                        .descendants()
                        .find(|node| node.has_tag_name("expectations"))
                        .unwrap();
                    let node = expected.first_element_child().unwrap();
                    assert_eq!(node.attribute("width"), Some("10"));
                    assert_eq!(node.attribute("height"), Some("20"));
                }
                CaseOutcome::Unsupported { case_id, reason } => {
                    assert!(matches!(index, 1 | 3));
                    assert_eq!(case_id, fixture().cases()[index].id());
                    assert_eq!(reason, if index == 1 { "second" } else { "fourth" });
                }
            }
        }
    }

    #[test]
    fn malformed_measurement_keeps_case_and_field_context_in_adapter_error() {
        let mut malformed = supported_box();
        malformed["smartRoundedLayout"]["width"] = Value::String("not a number".to_string());
        let error = LayoutAdapter
            .lower(
                &fixture(),
                envelope([
                    unsupported("first"),
                    malformed,
                    supported_box(),
                    unsupported("last"),
                ]),
            )
            .unwrap_err();
        let AdapterError::Measurement(source) = &error else {
            panic!("expected measurement error: {error}");
        };
        assert_eq!(source.case_id, "example__content_box_ltr");
        assert_eq!(source.variant, "content_box_ltr");
        assert!(source.field_path.contains("smartRoundedLayout.width"));
        assert_eq!(
            source.kind,
            crate::measurement::MeasurementErrorKind::Decode
        );
        assert!(
            std::error::Error::source(&error)
                .and_then(std::error::Error::source)
                .is_some()
        );
    }

    #[test]
    fn nested_duplicate_fields_are_rejected_before_unsupported_classification() {
        let payload = envelope(std::array::from_fn(|_| unsupported("first")));
        let raw = payload.as_str().unwrap().replacen(
            "\"unsupportedReason\":\"first\"",
            "\"unsupportedReason\":\"first\",\"unsupportedReason\":\"second\"",
            1,
        );
        let error = LayoutAdapter
            .lower(&fixture(), Value::String(raw))
            .unwrap_err();
        let AdapterError::Measurement(source) = &error else {
            panic!("expected measurement error: {error}");
        };
        assert_eq!(
            source.kind,
            crate::measurement::MeasurementErrorKind::Decode
        );
        assert!(std::error::Error::source(source.as_ref()).is_some());
    }

    #[test]
    fn original_subgrid_refuses_changed_helper_before_setup() {
        let source = include_str!(
            "../../layout/browser_parity/html/subgrid/subgrid_gap_percentage_border_bottom.html"
        );
        let source_path = RelativePath::new(original_subgrid_body_profile::SOURCE).unwrap();
        let setup = |helper: &str| {
            helper_setup_expression(
                helper,
                include_bytes!("../../layout/browser_parity/fonts/Ahem.ttf"),
                source,
                TEST_BASE_STYLE_SOURCE,
                &source_path,
                BoxObservationRequest::OriginalSubgridBody,
            )
        };
        let retained_helper = include_str!(
            "../../layout/browser_parity/captured/original-subgrid-body-qualification/scripts/gentest/test_helper.js"
        );
        assert_eq!(
            surgeist_generator::Sha256Digest::from_bytes(retained_helper.as_bytes()).as_str(),
            original_subgrid_body_profile::HELPER_SHA256
        );
        assert!(setup(retained_helper).is_ok());
        assert_ne!(retained_helper, TEST_HELPER_SOURCE);
        let changed = format!("{retained_helper}\ndescribeElement = () => null;\n");
        assert!(setup(TEST_HELPER_SOURCE).is_ok());
        assert!(matches!(
            setup(&changed),
            Err(AdapterError::Preparation(reason))
                if reason == "original subgrid body capture helper changed"
        ));
    }

    #[test]
    fn preloaded_unversioned_helper_is_preserved_and_rejected_at_ingress() {
        let Value::String(versioned) = envelope(std::array::from_fn(|_| unsupported("old helper")))
        else {
            unreachable!()
        };
        let mut legacy: Value = serde_json::from_str(&versioned).unwrap();
        legacy.as_object_mut().unwrap().remove("schemaVersion");
        let original = format!(
            "function getTestData() {{ return {}; }} function describeElement() {{ return {{ style: {{}} }}; }}",
            serde_json::to_string(&legacy.to_string()).unwrap(),
        );
        let original = helper_setup_expression(
            &original,
            include_bytes!("../../layout/browser_parity/fonts/Ahem.ttf"),
            "<!DOCTYPE html><html><body><div id=\"test-root\"></div></body></html>",
            TEST_BASE_STYLE_SOURCE,
            &RelativePath::new("html/grid/example.html").unwrap(),
            BoxObservationRequest::Geometry,
        )
        .unwrap();
        let replacement = helper_setup_expression(
            TEST_HELPER_SOURCE,
            include_bytes!("../../layout/browser_parity/fonts/Ahem.ttf"),
            "<!DOCTYPE html><html><body><div id=\"test-root\"></div></body></html>",
            TEST_BASE_STYLE_SOURCE,
            &RelativePath::new("html/grid/example.html").unwrap(),
            BoxObservationRequest::Geometry,
        )
        .unwrap();
        let script = format!(
            "global.window={{}}; const vm=require('node:vm'); vm.runInThisContext({}); const before=getTestData; vm.runInThisContext({}); if(getTestData!==before) throw Error('preloaded helper replaced'); process.stdout.write(getTestData());",
            serde_json::to_string(&original).unwrap(),
            serde_json::to_string(&replacement).unwrap(),
        );
        let output = std::process::Command::new("node")
            .arg("-e")
            .arg(script)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let error = LayoutAdapter
            .lower(
                &fixture(),
                Value::String(String::from_utf8(output.stdout).unwrap()),
            )
            .unwrap_err();
        let AdapterError::Envelope { source, .. } = error else {
            panic!("expected envelope error");
        };
        assert_eq!(
            source.kind(),
            crate::envelope::EnvelopeErrorKind::MissingVersion
        );
    }

    #[test]
    fn adapter_requires_a_closed_measurement_envelope() {
        let variants = fixture_cases()
            .into_iter()
            .map(|(_, key)| format!("\"{key}\":{{\"layoutInput\":\"unsupported\",\"unsupportedReason\":\"known unsupported\"}}"))
            .collect::<Vec<_>>()
            .join(",");
        for extra in [
            "",
            "\"schemaVersion\":3,",
            "\"schemaVersion\":5,",
            "\"schemaVersion\":4.0,",
            "\"schemaVersion\":null,",
            "\"schemaVersion\":4,\"unexpected\":true,",
            "\"schemaVersion\":4,\"schemaVersion\":4,",
        ] {
            let payload = format!("{{{extra}{variants}}}");
            assert!(
                LayoutAdapter
                    .lower(&fixture(), Value::String(payload))
                    .is_err(),
                "invalid envelope prefix was accepted: {extra:?}"
            );
        }
    }

    #[test]
    fn measurement_protocol_rejects_an_object_in_place_of_the_helpers_json_string() {
        let error = LayoutAdapter
            .lower(&fixture(), serde_json::json!({}))
            .unwrap_err();
        assert_eq!(
            error.to_string(),
            "example: measurement helper must return a JSON string"
        );
    }

    #[test]
    fn missing_variant_is_a_fixture_error_before_any_outcomes_are_published() {
        let error = LayoutAdapter
            .lower(
                &fixture(),
                Value::String("{\"schemaVersion\":4}".to_string()),
            )
            .unwrap_err();
        assert_eq!(
            error.to_string(),
            "example: measurement envelope borderBoxLtrData: declared variant is required"
        );
    }

    #[test]
    fn unsupported_variants_retain_order_and_first_descendant_reason() {
        let mut description = fixture_cases()
            .into_iter()
            .map(|(_, key)| {
                (
                    key.to_string(),
                    serde_json::json!({"layoutInput": "box", "children": [
                        {"layoutInput": "box", "children": [{"layoutInput": "unsupported", "unsupportedReason": "first"}]},
                        {"layoutInput": "unsupported", "unsupportedReason": "second"}
                    ]}),
                )
            })
            .collect::<serde_json::Map<_, _>>();
        description.insert(
            "schemaVersion".to_string(),
            Value::from(crate::envelope::VERSION),
        );
        let outcomes = LayoutAdapter
            .lower(
                &fixture(),
                Value::String(Value::Object(description).to_string()),
            )
            .unwrap();
        let identities = outcomes
            .iter()
            .map(|outcome| match outcome {
                CaseOutcome::Unsupported { case_id, reason } => {
                    assert_eq!(reason, "first");
                    case_id.as_str()
                }
                CaseOutcome::Generated { .. } => {
                    panic!("unsupported measurement produced an artifact")
                }
            })
            .collect::<Vec<_>>();
        assert_eq!(
            identities,
            [
                "example__border_box_ltr",
                "example__content_box_ltr",
                "example__border_box_rtl",
                "example__content_box_rtl"
            ]
        );
    }

    #[test]
    fn linked_resource_discovery_retains_literal_reference_rules() {
        let paths = linked_resources(
            &RelativePath::new("html/grid/example.html").unwrap(),
            r#"
<link href="../../assets/a.css?version=1#fragment"><img src='../../assets/a.css'>
<script src="../../scripts/gentest/test_helper.js"></script>
<link href="../../scripts/gentest/test_base_style.css"><img src="/absolute.png">
<img src="https://example.invalid/external.png"><img src="data:image/png;base64,abc">
<img src="../shared/z.png">
"#,
        )
        .unwrap();
        assert_eq!(
            paths.iter().map(RelativePath::as_str).collect::<Vec<_>>(),
            ["assets/a.css", "html/shared/z.png"]
        );
        assert!(
            linked_resources(
                &RelativePath::new("html/example.html").unwrap(),
                "<img src='../../../outside'>"
            )
            .is_err()
        );
        assert!(
            linked_resources(
                &RelativePath::new("html/example.html").unwrap(),
                "<img src='unterminated>"
            )
            .is_err()
        );
    }

    #[test]
    fn captured_font_bytes_are_authenticated_before_browser_setup() {
        let original = include_bytes!("../../layout/browser_parity/fonts/Ahem.ttf");
        assert!(
            helper_setup_expression(
                TEST_HELPER_SOURCE,
                original,
                "<!DOCTYPE html><html><body><div id=\"test-root\"></div></body></html>",
                TEST_BASE_STYLE_SOURCE,
                &RelativePath::new("html/grid/example.html").unwrap(),
                BoxObservationRequest::Geometry,
            )
            .is_ok()
        );
        let mut changed = original.to_vec();
        changed[0] ^= 1;
        assert!(matches!(
            helper_setup_expression(
                TEST_HELPER_SOURCE,
                &changed,
                "<!DOCTYPE html><html><body><div id=\"test-root\"></div></body></html>",
                TEST_BASE_STYLE_SOURCE,
                &RelativePath::new("html/grid/example.html").unwrap(),
                BoxObservationRequest::Geometry,
            ),
            Err(AdapterError::Preparation(_))
        ));
        assert!(
            helper_setup_expression(
                TEST_HELPER_SOURCE,
                &[],
                "<!DOCTYPE html><html><body><div id=\"test-root\"></div></body></html>",
                TEST_BASE_STYLE_SOURCE,
                &RelativePath::new("html/grid/example.html").unwrap(),
                BoxObservationRequest::Geometry,
            )
            .is_err()
        );
    }

    #[test]
    fn setup_loads_authenticated_font_without_browser_webcrypto() {
        // The document.write browser profile has no secure-context WebCrypto.
        // This load/add/ready recorder qualifies exact-byte transport; real
        // browser execution independently qualifies FontFace decoding/loading.
        let font = include_bytes!("../../layout/browser_parity/fonts/Ahem.ttf");
        let setup = helper_setup_expression(
            TEST_HELPER_SOURCE,
            font,
            "<!DOCTYPE html><html><body><div id=\"test-root\"></div></body></html>",
            TEST_BASE_STYLE_SOURCE,
            &RelativePath::new("html/grid/example.html").unwrap(),
            BoxObservationRequest::Geometry,
        )
        .unwrap();
        let script = format!(
            r#"const assert=require('node:assert/strict'); const vm=require('node:vm');
const expected={}; const loaded=[]; const faces=[]; faces.add=face=>faces.push(face); faces.ready=Promise.resolve();
const context=vm.createContext({{window:{{}},document:{{querySelector:()=>({{}}),fonts:faces}},
FontFace:class{{constructor(family,buffer,descriptors){{this.family=family;this.status='unloaded';this.bytes=Array.from(new Uint8Array(buffer));this.descriptors=descriptors;loaded.push(this);}}async load(){{this.status='loaded';return this;}}}}}});
(async()=>{{await vm.runInContext({},context);assert.equal(loaded.length,1);assert.equal(faces.length,1);assert.equal(faces[0],loaded[0]);assert.deepEqual(loaded[0].bytes,expected);assert.equal(loaded[0].family,'SurgeistParityAhem');assert.equal(loaded[0].status,'loaded');assert.deepEqual(JSON.parse(JSON.stringify(loaded[0].descriptors)),{{style:'normal',weight:'400',stretch:'normal',featureSettings:'normal',variationSettings:'normal'}});}})().catch(error=>{{console.error(error);process.exitCode=1;}});"#,
            serde_json::to_string(font.as_slice()).unwrap(),
            serde_json::to_string(&setup).unwrap(),
        );
        let output = std::process::Command::new("node")
            .arg("-e")
            .arg(script)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn setup_injects_a_missing_helper_and_preserves_an_existing_helper() {
        let first = helper_setup_expression("function getTestData() { return 'original'; } function describeElement() { return { style: {} }; }",include_bytes!("../../layout/browser_parity/fonts/Ahem.ttf"), "<!DOCTYPE html><html><body><div id=\"test-root\"></div></body></html>", TEST_BASE_STYLE_SOURCE, &RelativePath::new("html/grid/example.html").unwrap(),BoxObservationRequest::Geometry,).unwrap();
        let second = helper_setup_expression(
            "function getTestData() { return 'replacement'; }",
            include_bytes!("../../layout/browser_parity/fonts/Ahem.ttf"),
            "<!DOCTYPE html><html><body><div id=\"test-root\"></div></body></html>",
            TEST_BASE_STYLE_SOURCE,
            &RelativePath::new("html/grid/example.html").unwrap(),
            BoxObservationRequest::Geometry,
        )
        .unwrap();
        let script = format!(
            "global.window = {{}}; const vm = require('node:vm'); vm.runInThisContext({}); if (getTestData() !== 'original') throw Error('missing helper was not loaded'); const original = describeElement; vm.runInThisContext({}); if (getTestData() !== 'original' || describeElement !== original) throw Error('existing helper was replaced');",
            serde_json::to_string(&first).unwrap(),
            serde_json::to_string(&second).unwrap(),
        );
        let output = std::process::Command::new("node")
            .arg("-e")
            .arg(script)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
