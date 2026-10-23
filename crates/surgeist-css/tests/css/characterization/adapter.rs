//! The only characterization code that calls the CSS API.
//!
//! Each front id maps to one public front. Results are written from public
//! accessors and serializers, never from `Debug`, so a renamed or privatized
//! type does not change a row. Charged work is the smallest limit at which an
//! operation still succeeds (serialization) or still yields the same
//! observation (parsing), searched one limit at a time with every other limit
//! at its default.

mod color;
mod errors;
mod facts;
mod fronts;
mod report;

use std::cell::Cell;
use std::error::Error as StdError;
use std::fmt;

use surgeist_css::{
    CssComponentValueLimits, CssParseReport, CssSerializationLimits, CssUtf16InputLimits,
};

use super::cases::Case;

/// Every observed column of one golden row. `-` marks an absent observation.
#[derive(Clone, PartialEq, Eq)]
pub struct Observation {
    pub clean: String,
    pub retained: String,
    pub authored: String,
    pub diagnostics: String,
    pub shape: String,
    pub specified: String,
    pub cssom: String,
    pub charges: String,
    pub facts: String,
}

impl Default for Observation {
    fn default() -> Self {
        let absent = || "-".to_owned();
        Self {
            clean: absent(),
            retained: absent(),
            authored: absent(),
            diagnostics: absent(),
            shape: absent(),
            specified: absent(),
            cssom: absent(),
            charges: absent(),
            facts: absent(),
        }
    }
}

impl Observation {
    fn shaped(shape: &str) -> Self {
        Self {
            shape: shape.to_owned(),
            ..Self::default()
        }
    }

    /// Clean flag and diagnostics of a report.
    fn report<T>(&mut self, report: &CssParseReport<T>) {
        self.clean = report.is_clean().to_string();
        self.diagnostics = nonempty(report::diagnostics(report.diagnostics()).join("~"));
    }

    fn add_charge(&mut self, charge: String) {
        if self.charges == "-" {
            self.charges = charge;
        } else {
            self.charges.push(';');
            self.charges.push_str(&charge);
        }
    }

    fn add_fact(&mut self, fact: impl fmt::Display) {
        if self.facts == "-" {
            self.facts = fact.to_string();
        } else {
            self.facts.push_str(" | ");
            self.facts.push_str(&fact.to_string());
        }
    }

    /// Everything except charged work, for comparing observations made under
    /// different parse limits.
    fn without_charges(&self) -> Self {
        Self {
            charges: "-".to_owned(),
            ..self.clone()
        }
    }
}

/// The observation of one case.
pub fn observe(case: &Case) -> Observation {
    match case.front.as_str() {
        "error" => errors::observe(&case.argument),
        front if front.starts_with("color.") => color::observe(front, &case.argument, &case.input),
        front => fronts::observe(front, &case.argument, &case.input),
    }
}

fn nonempty(value: String) -> String {
    if value.is_empty() {
        "-".to_owned()
    } else {
        value
    }
}

/// `Display` of an error followed by the `Display` of each `source()`.
fn error_chain(error: &(dyn StdError + 'static)) -> String {
    let mut text = error.to_string();
    let mut source = error.source();
    while let Some(cause) = source {
        text.push_str(" <- ");
        text.push_str(&cause.to_string());
        source = cause.source();
    }
    text
}

/// The error chain followed by the accessor facts of every link.
fn error_text(error: &(dyn StdError + 'static)) -> String {
    let mut text = error_chain(error);
    for fact in facts::accessor_facts(error) {
        text.push_str(" | ");
        text.push_str(&fact);
    }
    text
}

/// A serialization result: the text (`<empty>` for no text), or `err:` and
/// the error chain.
fn serialized<E: StdError + 'static>(result: Result<String, E>) -> String {
    match result {
        Ok(text) => emitted(text),
        Err(error) => format!("err:{}", error_text(&error)),
    }
}

/// Emitted text, with `<empty>` marking an empty result.
fn emitted(text: String) -> String {
    if text.is_empty() {
        "<empty>".to_owned()
    } else {
        text
    }
}

/// Which limits a serializer honours.
#[derive(Clone, Copy)]
enum Limited {
    /// Input nodes, projection nodes and output bytes.
    Values,
    /// Output bytes only.
    Bytes,
}

/// A serialization column and its charged work.
fn serialization<E: StdError + 'static>(
    observation: &mut Observation,
    column: &str,
    limited: Limited,
    serialize: impl Fn(CssSerializationLimits) -> Result<String, E>,
) {
    let defaults = CssSerializationLimits::default();
    let text = serialized(serialize(defaults));
    match column {
        "specified" => observation.specified = text,
        "cssom" => observation.cssom = text,
        other => unreachable!("unknown serialization column {other}"),
    }
    if !NESTED_PROBE.with(Cell::get) {
        return;
    }
    let charge = match limited {
        Limited::Bytes => {
            let bytes = smallest(defaults.max_css_bytes(), |bytes| {
                serialize(CssSerializationLimits::new(
                    defaults.max_input_nodes(),
                    defaults.max_projection_nodes(),
                    bytes,
                ))
                .is_ok()
            });
            format!("{column}:bytes={}", amount(bytes))
        }
        Limited::Values => {
            let input = smallest(defaults.max_input_nodes(), |value| {
                serialize(CssSerializationLimits::new(
                    value,
                    defaults.max_projection_nodes(),
                    defaults.max_css_bytes(),
                ))
                .is_ok()
            });
            let projection = smallest(defaults.max_projection_nodes(), |value| {
                serialize(CssSerializationLimits::new(
                    defaults.max_input_nodes(),
                    value,
                    defaults.max_css_bytes(),
                ))
                .is_ok()
            });
            let bytes = smallest(defaults.max_css_bytes(), |value| {
                serialize(CssSerializationLimits::new(
                    defaults.max_input_nodes(),
                    defaults.max_projection_nodes(),
                    value,
                ))
                .is_ok()
            });
            format!(
                "{column}:in={},proj={},bytes={}",
                amount(input),
                amount(projection),
                amount(bytes)
            )
        }
    };
    observation.add_charge(charge);
}

/// Parse work under component limits: the smallest depth, component count and
/// byte limit at which `observe` still yields the default-limit observation.
fn parse_charge(
    observation: &mut Observation,
    observe: impl Fn(CssComponentValueLimits) -> Observation,
) {
    let defaults = CssComponentValueLimits::default();
    let observe = |limits| probing(|| observe(limits));
    let baseline = observe(defaults).without_charges();
    let same = |limits: Option<CssComponentValueLimits>| {
        limits.is_some_and(|limits| observe(limits).without_charges() == baseline)
    };
    let depth = smallest(defaults.max_nesting_depth() as usize, |depth| {
        same(CssComponentValueLimits::try_new(
            u32::try_from(depth).unwrap(),
            defaults.max_components(),
            defaults.max_css_bytes(),
        ))
    });
    let components = smallest(defaults.max_components(), |components| {
        same(CssComponentValueLimits::try_new(
            defaults.max_nesting_depth(),
            components,
            defaults.max_css_bytes(),
        ))
    });
    let bytes = smallest(defaults.max_css_bytes(), |bytes| {
        same(CssComponentValueLimits::try_new(
            defaults.max_nesting_depth(),
            defaults.max_components(),
            bytes,
        ))
    });
    observation.add_charge(format!(
        "parse:depth={},components={},bytes={}",
        amount(depth),
        amount(components),
        amount(bytes)
    ));
}

/// One field of the UTF-16 input limits.
type Utf16Limit = fn(&mut CssUtf16InputLimits) -> &mut usize;

/// UTF-16 preparation work: the smallest value of each input limit at which
/// `observe` still yields the default-limit observation.
fn utf16_charge(
    observation: &mut Observation,
    observe: impl Fn(CssUtf16InputLimits) -> Observation,
) {
    let defaults = CssUtf16InputLimits::default();
    let observe = |limits| probing(|| observe(limits));
    let baseline = observe(defaults).without_charges();
    let fields: [(&str, Utf16Limit); 7] = [
        ("units", |limits| &mut limits.max_input_units),
        ("output", |limits| &mut limits.max_output_bytes),
        ("map", |limits| &mut limits.max_map_entries),
        ("checkpoints", |limits| &mut limits.max_checkpoints),
        ("scratch", |limits| &mut limits.max_scratch_bytes),
        ("retained", |limits| &mut limits.max_retained_bytes),
        ("work", |limits| &mut limits.max_work),
    ];
    let mut parts = Vec::new();
    for (name, field) in fields {
        let mut maximum = defaults;
        let value = smallest(*field(&mut maximum), |value| {
            let mut limits = defaults;
            *field(&mut limits) = value;
            observe(limits).without_charges() == baseline
        });
        parts.push(format!("{name}={}", amount(value)));
    }
    observation.add_charge(format!("utf16:{}", parts.join(",")));
}

thread_local! {
    /// False while a parse-limit probe re-observes a front: the probe compares
    /// observations without charges, so nested serialization searches are
    /// skipped rather than computed and discarded.
    static NESTED_PROBE: Cell<bool> = const { Cell::new(true) };
}

/// Runs `observe` with nested charge searches disabled.
fn probing<T>(observe: impl FnOnce() -> T) -> T {
    let previous = NESTED_PROBE.with(|enabled| enabled.replace(false));
    let result = observe();
    NESTED_PROBE.with(|enabled| enabled.set(previous));
    result
}

fn amount(value: Option<usize>) -> String {
    value.map_or_else(|| "refused".to_owned(), |value| value.to_string())
}

/// The smallest value in `0..=maximum` for which `accepts` holds, assuming
/// acceptance is monotone; `None` when even `maximum` is refused.
fn smallest(maximum: usize, accepts: impl Fn(usize) -> bool) -> Option<usize> {
    if !accepts(maximum) {
        return None;
    }
    if accepts(0) {
        return Some(0);
    }
    let mut refused = 0;
    let mut probe = 1usize;
    while probe < maximum && !accepts(probe) {
        refused = probe;
        probe = probe.saturating_mul(2);
    }
    let mut accepted = probe.min(maximum);
    while accepted - refused > 1 {
        let middle = refused + (accepted - refused) / 2;
        if accepts(middle) {
            accepted = middle;
        } else {
            refused = middle;
        }
    }
    Some(accepted)
}

/// The interruption a characterization poll never raises, and the one the
/// interrupted-input error case raises.
#[derive(Debug)]
struct Stop;

impl fmt::Display for Stop {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("characterization stop")
    }
}

impl StdError for Stop {}
