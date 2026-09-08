use surgeist_style::{CalcLength, CalcLengthTerm};

fn main() {
    let _term = CalcLengthTerm {
        operator: surgeist_style::CalcOperator::Add,
        value: Box::new(CalcLength::Px(1.0)),
    };
}
