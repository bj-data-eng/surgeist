use std::{error, fmt};

use surgeist::{
    adapters::{
        AdapterError, RetainedStyleTree, lower_css_sheet_to_style, lower_style_text_to_text_style,
        lower_style_to_layout_with_store,
    },
    css, retained, style,
};
use surgeist_test::fixtures::{
    ArtifactPaths, FixtureExpectation, FixtureIdentity, FixturePath, GeneratorProvenance,
    LayoutReadyFixtureMetadata, SchemaError,
};

pub const INLINE_METRICS_SCHEMA_LIMITATION: &str = concat!(
    "layout-ready fixture metadata schema currently has no inline metric fields; ",
    "values such as surgeist-inline-line-height and surgeist-inline-baseline ",
    "are adapter-validated but not represented in metadata yet"
);

const GENERATOR_NAME: &str = "surgeist-fixture-metadata";
const ADAPTER_PROVENANCE: &str = "root css-style/style-text/style-layout adapters";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixtureMetadataRequest {
    pub identity: String,
    pub source_path: String,
    pub generated_path: String,
    pub css_source: String,
    pub expectation: Option<String>,
}

#[derive(Debug)]
pub enum FixtureMetadataError {
    Css(css::Error),
    Adapter(AdapterError),
    Schema(SchemaError),
    EmptyStyleSheet,
    Retained(retained::Error),
    Style(style::Error),
}

pub type FixtureMetadataResult<T> = Result<T, FixtureMetadataError>;

pub fn generate_layout_ready_metadata(
    request: FixtureMetadataRequest,
) -> FixtureMetadataResult<LayoutReadyFixtureMetadata> {
    let css_sheet = css::parse_sheet(&request.css_source)?;
    let style_sheet = lower_css_sheet_to_style(&css_sheet)?;
    let declarations = style_sheet
        .rules()
        .first()
        .ok_or(FixtureMetadataError::EmptyStyleSheet)?
        .declarations()
        .clone();

    let resolved = resolve_first_fixture_rule(style_sheet, &declarations)?;
    let text_value = style::TextValue::default()
        .size(resolved.font_size())
        .line_height(resolved_length(&resolved, style::Property::LineHeight))
        .color(resolved.text_color());
    let _text_style = lower_style_text_to_text_style(&text_value)?;
    let _layout_ready = lower_style_to_layout_with_store(&resolved)?;

    let metadata = LayoutReadyFixtureMetadata::new(
        FixtureIdentity::new(request.identity)?,
        ArtifactPaths::new(
            FixturePath::new(request.source_path)?,
            FixturePath::new(request.generated_path)?,
        ),
        GeneratorProvenance::new(GENERATOR_NAME)?.with_adapter(ADAPTER_PROVENANCE)?,
    );

    match request.expectation {
        Some(expectation) => Ok(metadata.with_expectation(FixtureExpectation::new(expectation)?)),
        None => Ok(metadata),
    }
}

fn resolve_first_fixture_rule(
    sheet: style::Sheet,
    declarations: &style::Declarations,
) -> FixtureMetadataResult<style::Resolved> {
    let mut model = retained::Model::empty();
    let root = model.root();
    let fixture = model
        .apply(retained::Patch::Insert {
            parent: root,
            index: 0,
            element: retained::Element::tagged(
                retained::Tag::new("fixture").expect("static fixture tag is valid"),
            ),
        })?
        .changes()
        .inserted()[0];
    let tree = RetainedStyleTree::new(model.snapshot());
    Ok(style::Resolver::new(sheet)
        .resolve(style::Context::new(&tree, fixture).local(declarations))?)
}

fn resolved_length(resolved: &style::Resolved, property: style::Property) -> style::Length {
    match resolved.get(property) {
        style::Value::Length(length) => length.clone(),
        _ => match property.metadata().default() {
            style::Value::Length(length) => length.clone(),
            _ => style::Length::Px(16.0),
        },
    }
}

impl fmt::Display for FixtureMetadataError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Css(error) => write!(formatter, "{error}"),
            Self::Adapter(error) => write!(formatter, "{error}"),
            Self::Schema(error) => write!(formatter, "{error}"),
            Self::EmptyStyleSheet => write!(formatter, "fixture CSS did not contain any rules"),
            Self::Retained(error) => write!(formatter, "{error}"),
            Self::Style(error) => write!(formatter, "{error}"),
        }
    }
}

impl error::Error for FixtureMetadataError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::Css(error) => Some(error),
            Self::Adapter(error) => Some(error),
            Self::Schema(error) => Some(error),
            Self::Retained(error) => Some(error),
            Self::Style(error) => Some(error),
            Self::EmptyStyleSheet => None,
        }
    }
}

impl From<css::Error> for FixtureMetadataError {
    fn from(error: css::Error) -> Self {
        Self::Css(error)
    }
}

impl From<AdapterError> for FixtureMetadataError {
    fn from(error: AdapterError) -> Self {
        Self::Adapter(error)
    }
}

impl From<SchemaError> for FixtureMetadataError {
    fn from(error: SchemaError) -> Self {
        Self::Schema(error)
    }
}

impl From<retained::Error> for FixtureMetadataError {
    fn from(error: retained::Error) -> Self {
        Self::Retained(error)
    }
}

impl From<style::Error> for FixtureMetadataError {
    fn from(error: style::Error) -> Self {
        Self::Style(error)
    }
}
