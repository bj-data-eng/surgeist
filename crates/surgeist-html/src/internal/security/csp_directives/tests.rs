use super::*;
use std::convert::Infallible;

fn step() -> Result<(), Infallible> {
    Ok(())
}

fn directives(text: &str, source: HtmlCspPolicySource) -> Vec<(String, Vec<String>)> {
    let units: Vec<_> = text.encode_utf16().collect();
    let mut cursor = HtmlCspSerializedPolicy::new(&units, source).directives();
    let mut observed = Vec::new();
    while let Some(directive) = cursor.next_with(&mut step).unwrap() {
        let mut values = directive.values();
        let mut tokens = Vec::new();
        while let Some(token) = values.next_with(&mut step).unwrap() {
            tokens.push(String::from_utf16(token).unwrap());
        }
        observed.push((directive.name().map(char::from).collect(), tokens));
    }
    observed
}

#[test]
fn first_ascii_directives_keep_source_order_and_token_spelling() {
    let text = " ; SCRIPT-src\t'None' \n HTTPS: ; script-SRC *; vendor-mode é; \
                VENDOR-MODE\tKeepCase; trusted-types; x x\u{000b}y; ";
    assert_eq!(
        directives(text, HtmlCspPolicySource::Header),
        [
            ("script-src".into(), vec!["'None'".into(), "HTTPS:".into()]),
            ("vendor-mode".into(), vec!["KeepCase".into()]),
            ("trusted-types".into(), vec![]),
            ("x".into(), vec!["x\u{000b}y".into()]),
        ],
    );
}

#[test]
fn meta_removes_only_the_three_excluded_directives() {
    let text = "Report-URI /legacy; frame-ancestors 'none'; SANDBOX; \
                report-to new; require-trusted-types-for 'script'";
    assert_eq!(
        directives(text, HtmlCspPolicySource::Meta),
        [
            ("report-to".into(), vec!["new".into()]),
            ("require-trusted-types-for".into(), vec!["'script'".into()]),
        ],
    );
    let units: Vec<_> = text.encode_utf16().collect();
    let header = HtmlCspSerializedPolicy::new(&units, HtmlCspPolicySource::Header);
    let directive = header
        .directive_with(b"REPORT-uri", &mut step)
        .unwrap()
        .unwrap();
    assert_eq!(
        String::from_utf16(&units[directive.value_range()]).unwrap(),
        " /legacy",
    );
    let meta = HtmlCspSerializedPolicy::new(&units, HtmlCspPolicySource::Meta);
    assert!(
        meta.directive_with(b"report-uri", &mut step)
            .unwrap()
            .is_none()
    );
}

#[test]
fn refused_duplicate_inspection_does_not_repeat_an_emitted_directive() {
    let units: Vec<_> = "x first; X ignored; y next".encode_utf16().collect();
    let mut cursor = HtmlCspSerializedPolicy::new(&units, HtmlCspPolicySource::Header).directives();
    let first = cursor.next_with(&mut step).unwrap().unwrap();
    assert!(first.name().eq(*b"x"));
    let mut left = 14_u8;
    assert!(
        cursor
            .next_with(&mut || {
                left = left.checked_sub(1).ok_or("stopped")?;
                Ok::<(), &str>(())
            })
            .is_err()
    );
    let second = cursor.next_with(&mut step).unwrap().unwrap();
    assert!(second.name().eq(*b"y"));
    assert!(cursor.next_with(&mut step).unwrap().is_none());
}
