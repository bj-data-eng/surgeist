//! The controlled color parse is exhaustive on purpose and prints its state.
//!
//! The step, action, outcome and bank enums carry no wildcard arm below: a
//! new variant must fail this crate's consumers at compile time, which is the
//! proof that a driver services every action.

use surgeist_css::color::controlled::{
    CssColorBank, CssColorOperationOwner, CssColorParse, CssColorParseAction, CssColorParseOutcome,
    CssColorParseStep,
};
use surgeist_css::color::{
    CssColorFactIdentity, CssColorFacts, CssColorGrammarRequest, CssColorInputKind,
    CssColorPropertyFacts, CssColorSchemeFact, CssDeviceColorFact,
};

fn bank_name(bank: CssColorBank) -> &'static str {
    match bank {
        CssColorBank::Algorithm => "algorithm",
        CssColorBank::Arithmetic => "arithmetic",
        CssColorBank::Evaluation => "evaluation",
    }
}

fn action_name(action: &CssColorParseAction<'_, '_>) -> &'static str {
    match action {
        CssColorParseAction::ReadInput(_) => "read input",
        CssColorParseAction::ReadSlot(_) => "read slot",
        CssColorParseAction::AppendSlot(_) => "append slot",
        CssColorParseAction::UpdateSlot(_) => "update slot",
        CssColorParseAction::ReadFact(_) => "read fact",
        CssColorParseAction::Compute(_) => "compute",
    }
}

fn outcome_name(outcome: &CssColorParseOutcome<'_>) -> &'static str {
    match outcome {
        CssColorParseOutcome::Used(_) => "used",
        CssColorParseOutcome::StandardFailure(_) => "standard failure",
    }
}

fn step_name(step: &CssColorParseStep<'_, '_>) -> String {
    match step {
        CssColorParseStep::Pending(action) => format!("pending {}", action_name(action)),
        CssColorParseStep::Complete(outcome) => format!("complete {}", outcome_name(outcome)),
    }
}

fn absent_facts() -> CssColorFacts<'static> {
    CssColorFacts::new(
        CssColorFactIdentity::new(1, 1),
        CssColorPropertyFacts::Unavailable,
        CssColorSchemeFact::Unavailable,
        &[],
        &[],
        &[],
        CssDeviceColorFact::Unknown,
    )
}

#[test]
fn a_started_parse_first_asks_for_its_input_and_its_owner_parse_and_step_print() {
    let facts = absent_facts();
    let mut owner = CssColorOperationOwner::new();
    assert!(format!("{owner:?}").starts_with("CssColorOperationOwner"));
    let parse = CssColorParse::start(
        &mut owner,
        CssColorInputKind::Utf8,
        3,
        CssColorGrammarRequest::WholeColor,
        &facts,
        facts.identity(),
    )
    .unwrap();
    let printed = format!("{parse:?}");
    assert!(printed.starts_with("CssColorParse"), "{printed}");
    assert!(printed.contains("Utf8"), "{printed}");
    assert_eq!(parse.bank_length(CssColorBank::Algorithm), 0);
    assert_eq!(bank_name(CssColorBank::Arithmetic), "arithmetic");
    let step = parse.step();
    assert_eq!(step_name(&step), "pending read input");
    let printed = format!("{step:?}");
    assert!(printed.contains("ReadInput"), "{printed}");
}
