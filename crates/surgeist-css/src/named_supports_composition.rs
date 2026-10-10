//! Joint native admission of borrowed test children, without output or reparse.
use crate::component_values::{ComponentAdmission, CssCanonicalBuilder, CssCanonicalToken};
use crate::{
    CssComponentValueError, CssComponentValueErrorKind, CssComponentValueLimits,
    CssSupportsTestChildRef as Child,
};

type Result<T> = std::result::Result<T, CssComponentValueError>;
pub(crate) fn admit(children: &[Child<'_>], limits: CssComponentValueLimits) -> Result<()> {
    let mut context = ComponentAdmission::new(limits);
    let mut pending: Vec<_> = children
        .iter()
        .rev()
        .map(|child| (*child, 0usize))
        .collect();
    while let Some((child, depth)) = pending.pop() {
        if depth > limits.max_nesting_depth() as usize {
            return Err(CssComponentValueError::new(
                CssComponentValueErrorKind::NestingLimit,
                child.origin().clone(),
            ));
        }
        match child {
            Child::Declaration(v) => {
                context.visit(v.components(), depth)?;
                context.grammar(&v.terminator)?;
            }
            Child::QualifiedRule(v) => {
                context.visit(v.prelude().items(), depth)?;
                context.grammar(&v.data.opening)?;
                if depth >= limits.max_nesting_depth() as usize {
                    return Err(CssComponentValueError::new(
                        CssComponentValueErrorKind::NestingLimit,
                        v.data.opening.clone(),
                    ));
                }
                pending.extend(v.body().children().map(|child| (child, depth + 1)));
            }
            Child::AtRule(v) => {
                context.visit(std::slice::from_ref(v.at_keyword()), depth)?;
                context.visit(v.prelude().items(), depth)?;
                context.grammar(v.data.opening.as_ref().unwrap_or(&v.data.closing))?;
                if let Some(body) = v.body() {
                    if depth >= limits.max_nesting_depth() as usize {
                        return Err(CssComponentValueError::new(
                            CssComponentValueErrorKind::NestingLimit,
                            v.data.opening.as_ref().expect("block opening").clone(),
                        ));
                    }
                    pending.extend(body.children().map(|child| (child, depth + 1)));
                }
            }
        }
    }
    let mut sink = CssCanonicalBuilder::counting(limits.max_css_bytes());
    for child in children {
        match child {
            Child::Declaration(v) => {
                sink.push_grammar(CssCanonicalToken::Ident(v.property()), v.origin())?;
                sink.push_grammar(CssCanonicalToken::Colon, v.origin())?;
                sink.push_components(v.value_components())?;
                if let Some(importance) = v.importance_components() {
                    sink.push_components(importance)?;
                }
                sink.push_grammar(CssCanonicalToken::Semicolon, &v.terminator)?;
            }
            Child::QualifiedRule(v) => {
                sink.push_components(v.prelude().items())?;
                sink.push_grammar(CssCanonicalToken::OpenBrace, &v.data.opening)?;
                v.body().write_to(&mut sink)?;
                sink.push_grammar(CssCanonicalToken::CloseBrace, &v.data.closing)?;
            }
            Child::AtRule(v) => {
                sink.push_component(v.at_keyword())?;
                sink.push_components(v.prelude().items())?;
                if let Some(body) = v.body() {
                    sink.push_grammar(
                        CssCanonicalToken::OpenBrace,
                        v.data.opening.as_ref().expect("block opening"),
                    )?;
                    body.write_to(&mut sink)?;
                    sink.push_grammar(CssCanonicalToken::CloseBrace, &v.data.closing)?;
                } else {
                    sink.push_grammar(CssCanonicalToken::Semicolon, &v.data.closing)?;
                }
            }
        }
    }
    Ok(())
}
