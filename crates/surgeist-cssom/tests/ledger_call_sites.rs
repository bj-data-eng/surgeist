//! No CSS crate calls the ledger inside a loop.
//!
//! The CSS fronts charge once per outermost call (#2899), so a ledger call
//! inside a loop over tokens, units, nodes or actions is a defect. This check
//! reads the production sources of `surgeist-css-parser`, `surgeist-css`,
//! `surgeist-color` and `surgeist-cssom` and fails when a ledger call sits
//! lexically inside a `for`, `while` or `loop` body or inside the arguments of
//! an iterator adaptor such as `map` or `for_each`.
//!
//! A ledger call is `charge` or `timed` whose first argument is a `CssFront`
//! (a `CssFront::` path or a binding named `front`), or `apply_batch` or
//! `with_phase`. The token builder's own `Meter` charges take other first
//! arguments and are not ledger calls. The check is lexical: it does not
//! follow a call into the function it calls. Test modules and test files are
//! not read.

use std::path::{Path, PathBuf};

/// Iterator adaptors whose closure runs once per item.
const ADAPTORS: [&str; 18] = [
    "all",
    "any",
    "filter",
    "filter_map",
    "find",
    "find_map",
    "flat_map",
    "fold",
    "for_each",
    "inspect",
    "map",
    "map_while",
    "position",
    "scan",
    "skip_while",
    "take_while",
    "try_fold",
    "try_for_each",
];

/// The source with comments and the contents of string and character
/// literals replaced by spaces, so that braces, parentheses and calls in them
/// are not read. Line breaks are kept.
fn strip(source: &str) -> String {
    let chars: Vec<char> = source.chars().collect();
    let mut out = String::with_capacity(source.len());
    let mut index = 0;
    let blank = |c: char| if c == '\n' { '\n' } else { ' ' };
    while index < chars.len() {
        let c = chars[index];
        let next = chars.get(index + 1).copied();
        if c == '/' && next == Some('/') {
            while index < chars.len() && chars[index] != '\n' {
                out.push(' ');
                index += 1;
            }
        } else if c == '/' && next == Some('*') {
            let mut depth = 0;
            while index < chars.len() {
                if chars[index] == '/' && chars.get(index + 1) == Some(&'*') {
                    depth += 1;
                    out.push_str("  ");
                    index += 2;
                } else if chars[index] == '*' && chars.get(index + 1) == Some(&'/') {
                    depth -= 1;
                    out.push_str("  ");
                    index += 2;
                    if depth == 0 {
                        break;
                    }
                } else {
                    out.push(blank(chars[index]));
                    index += 1;
                }
            }
        } else if c == 'r'
            && (next == Some('"') || next == Some('#'))
            && !is_ident(chars.get(index.wrapping_sub(1)).copied())
        {
            // A raw string: r"..." or r#"..."#.
            let mut hashes = 0;
            let mut cursor = index + 1;
            while chars.get(cursor) == Some(&'#') {
                hashes += 1;
                cursor += 1;
            }
            if chars.get(cursor) != Some(&'"') {
                out.push(c);
                index += 1;
                continue;
            }
            for _ in index..=cursor {
                out.push(' ');
            }
            index = cursor + 1;
            while index < chars.len() {
                if chars[index] == '"'
                    && (0..hashes).all(|n| chars.get(index + 1 + n) == Some(&'#'))
                {
                    for _ in 0..=hashes {
                        out.push(' ');
                    }
                    index += 1 + hashes;
                    break;
                }
                out.push(blank(chars[index]));
                index += 1;
            }
        } else if c == '"' {
            out.push(' ');
            index += 1;
            while index < chars.len() && chars[index] != '"' {
                if chars[index] == '\\' {
                    out.push(' ');
                    index += 1;
                }
                if index < chars.len() {
                    out.push(blank(chars[index]));
                    index += 1;
                }
            }
            out.push(' ');
            index += 1;
        } else if c == '\'' && is_char_literal(&chars, index) {
            out.push(' ');
            index += 1;
            while index < chars.len() && chars[index] != '\'' {
                if chars[index] == '\\' {
                    out.push(' ');
                    index += 1;
                }
                out.push(' ');
                index += 1;
            }
            out.push(' ');
            index += 1;
        } else {
            out.push(c);
            index += 1;
        }
    }
    out
}

fn is_ident(c: Option<char>) -> bool {
    c.is_some_and(|c| c.is_alphanumeric() || c == '_')
}

/// Whether the quote at `index` opens a character literal rather than a
/// lifetime or label.
fn is_char_literal(chars: &[char], index: usize) -> bool {
    match chars.get(index + 1) {
        Some('\\') => true,
        Some(_) => chars.get(index + 2) == Some(&'\''),
        None => false,
    }
}

/// One open brace or parenthesis.
#[derive(Clone, Copy)]
enum Frame {
    /// A block; whether it is the body of a loop, and whether it is test code.
    Brace { repeats: bool, test: bool },
    /// Parentheses; whether they hold the arguments of an iterator adaptor.
    Paren { repeats: bool },
}

/// The word before `end` in `text`, skipping spaces.
fn word_before(text: &[char], end: usize) -> String {
    let mut end = end;
    while end > 0 && text[end - 1] == ' ' {
        end -= 1;
    }
    let mut start = end;
    while start > 0 && is_ident(Some(text[start - 1])) {
        start -= 1;
    }
    text[start..end].iter().collect()
}

/// Whether `head`, the text of a statement before its `{`, opens a loop body.
fn opens_loop(head: &str) -> bool {
    let words: Vec<&str> = head
        .split(|c: char| !(c.is_alphanumeric() || c == '_'))
        .filter(|word| !word.is_empty())
        .collect();
    words.last() == Some(&"loop")
        || words.contains(&"while")
        || (words.contains(&"for") && words.contains(&"in"))
}

/// The line numbers of the ledger calls in `source` that sit inside a loop,
/// and the number of ledger calls read.
fn ledger_calls_in_loops(source: &str) -> (Vec<usize>, usize) {
    let text: Vec<char> = strip(source).chars().collect();
    let mut stack: Vec<Frame> = Vec::new();
    let mut head_start = 0;
    let mut found = Vec::new();
    let mut calls = 0;
    let mut line = 1;
    let mut pending_test = false;
    for index in 0..text.len() {
        let c = text[index];
        match c {
            '\n' => line += 1,
            '#' => {
                let rest: String = text[index..text.len().min(index + 14)].iter().collect();
                let compact: String = rest.chars().filter(|c| !c.is_whitespace()).collect();
                if compact.starts_with("#[cfg(test)]") {
                    pending_test = true;
                }
            }
            '{' => {
                let head: String = text[head_start..index].iter().collect();
                let test = pending_test
                    || stack
                        .iter()
                        .any(|frame| matches!(frame, Frame::Brace { test: true, .. }));
                stack.push(Frame::Brace {
                    repeats: opens_loop(&head),
                    test,
                });
                pending_test = false;
                head_start = index + 1;
            }
            '}' => {
                stack.pop();
                head_start = index + 1;
            }
            ';' => {
                head_start = index + 1;
                // An attribute applies to the next item only.
                if !matches!(stack.last(), Some(Frame::Paren { .. })) {
                    pending_test = false;
                }
            }
            '(' => {
                let name = word_before(&text, index);
                let dotted = {
                    let mut at = index - name.len();
                    while at > 0 && text[at - 1] == ' ' {
                        at -= 1;
                    }
                    at > 0 && text[at - 1] == '.'
                };
                let repeats = dotted && ADAPTORS.contains(&name.as_str());
                if dotted && is_ledger_call(&name, &text[index + 1..]) {
                    calls += 1;
                    let in_test = stack
                        .iter()
                        .any(|frame| matches!(frame, Frame::Brace { test: true, .. }));
                    let in_loop = stack.iter().any(|frame| match frame {
                        Frame::Brace { repeats, .. } | Frame::Paren { repeats } => *repeats,
                    });
                    if in_loop && !in_test {
                        found.push(line);
                    }
                }
                stack.push(Frame::Paren { repeats });
            }
            ')' => {
                if matches!(stack.last(), Some(Frame::Paren { .. })) {
                    stack.pop();
                }
            }
            _ => {}
        }
    }
    (found, calls)
}

/// Whether a method call of `name` whose arguments start at `arguments` is a
/// ledger call.
fn is_ledger_call(name: &str, arguments: &[char]) -> bool {
    match name {
        "apply_batch" | "with_phase" | "record_largest" => true,
        "charge" | "timed" => {
            let first: String = arguments
                .iter()
                .skip_while(|c| c.is_whitespace())
                .take_while(|c| **c != ',' && **c != ')')
                .collect();
            let first = first.trim();
            first.starts_with("CssFront::") || first == "front"
        }
        _ => false,
    }
}

/// The production sources of a crate: everything under `src` except test
/// files and directories.
fn sources(dir: &Path, out: &mut Vec<PathBuf>) {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    entries.sort();
    for path in entries {
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        if path.is_dir() {
            if name != "tests" {
                sources(&path, out);
            }
        } else if name.ends_with(".rs") && name != "tests.rs" && !name.ends_with("_tests.rs") {
            out.push(path);
        }
    }
}

#[test]
fn no_css_crate_calls_the_ledger_inside_a_loop() {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut files = Vec::new();
    for name in [
        "surgeist-css-parser",
        "surgeist-css",
        "surgeist-color",
        "surgeist-cssom",
    ] {
        sources(&crates.join(name).join("src"), &mut files);
    }
    let mut findings = Vec::new();
    let mut calls = 0;
    for file in &files {
        let source = std::fs::read_to_string(file).unwrap();
        let (lines, read) = ledger_calls_in_loops(&source);
        calls += read;
        findings.extend(
            lines
                .into_iter()
                .map(|line| format!("{}:{line}", file.display())),
        );
    }
    assert!(
        findings.is_empty(),
        "ledger calls inside loops:\n{}",
        findings.join("\n")
    );
    // The check reads the accounted fronts of every crate that charges.
    assert!(calls >= 15, "only {calls} ledger calls read");
}

#[test]
fn the_check_finds_a_ledger_call_in_each_kind_of_loop() {
    let flagged = [
        "fn a() { for item in items { account.charge(CssFront::Parse, &[]); } }",
        "fn a() { while more { self.account.timed(front, || 1); } }",
        "fn a() { let v = loop { ledger.apply_batch(scope, changes); }; }",
        "fn a() { items.iter().for_each(|item| account.charge(CssFront::Parse, &[])); }",
        "fn a() { items.iter().map(|item| { account.charge(CssFront::Color, &[item]) }).count(); }",
        "fn a() { 'outer: for x in 0..3 { if x > 1 { ledger.with_phase(scope, run); } } }",
    ];
    for source in flagged {
        assert_eq!(ledger_calls_in_loops(source).0, vec![1], "{source}");
    }
}

#[test]
fn the_check_ignores_calls_outside_loops_and_what_is_not_a_ledger_call() {
    let clean = [
        "fn a() { for item in items { count += 1; } account.charge(CssFront::Parse, &[]); }",
        "fn a() { for x in y { self.charge(at, span(range), true, depth)?; } }",
        "fn a() { // for x in y { account.charge(CssFront::Parse, &[]) }\n}",
        "fn a() { let s = \"for x in y { account.charge(CssFront::Parse) }\"; }",
        "#[cfg(test)]\nmod tests { fn t() { for x in y { account.charge(CssFront::Parse, &[]); } } }",
        "fn a() { let n = items.iter().map(|i| i + 1).sum(); account.timed(CssFront::Cssom, run); }",
    ];
    for source in clean {
        let (found, calls) = ledger_calls_in_loops(source);
        assert!(found.is_empty(), "{source}: {found:?}");
        let expected = usize::from(
            !source.contains("self.charge") && !source.contains("//") && !source.contains('"'),
        );
        assert_eq!(calls, expected, "{source}");
    }
}
