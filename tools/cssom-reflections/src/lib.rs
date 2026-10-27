#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;
use std::path::Path;
use surgeist_css::property::{CssKnownProperty, CssSvgGlyphOrientationVerticalDeclaration};

const OUTPUT: &str = "crates/surgeist-cssom/src/generated_property_reflections.rs";
const API: &str = "api/crates/surgeist-cssom.txt";

// Deliberate structural inventory, checked against the generated public API.
// This is not an inventory of CSS property names or a parser of Rust source.
const RESERVED: &[&str] = &[
    "id",
    "capture",
    "as_style_declaration",
    "css_text",
    "set_css_text",
    "length",
    "item",
    "get_property_value",
    "get_property_priority",
    "set_property",
    "remove_property",
    "parent_rule",
    "css_float",
    "set_css_float",
];
const CAPTURED_RESERVED: &[&str] = &[
    "handle",
    "snapshot",
    "as_style_declaration",
    "css_text",
    "length",
    "item",
    "get_property_value",
    "get_property_priority",
    "parent_rule",
    "css_float",
];

#[derive(Debug, Eq, PartialEq)]
struct Reflection {
    css_name: &'static str,
    getter: String,
    setter: String,
    origins: BTreeSet<&'static str>,
    suffixed: bool,
}

fn inventory() -> BTreeMap<&'static str, BTreeSet<&'static str>> {
    let mut names = BTreeMap::<_, BTreeSet<_>>::new();
    for property in CssKnownProperty::all() {
        names
            .entry(property.canonical_name())
            .or_default()
            .insert("canonical");
        for alias in property.aliases() {
            names.entry(*alias).or_default().insert("name alias");
        }
        for grammar in property.legacy_shorthands() {
            names
                .entry(grammar.name())
                .or_default()
                .insert("distinct legacy shorthand");
        }
    }
    // The SVG declaration has its own grammar even where its authored spelling
    // overlaps the finite writing-mode legacy shorthand. Keep both provenance.
    names
        .entry(CssSvgGlyphOrientationVerticalDeclaration::metadata().name())
        .or_default()
        .insert("independent SVG declaration");
    names
}

fn keyword(name: &str) -> bool {
    matches!(
        name,
        "as" | "async"
            | "await"
            | "break"
            | "const"
            | "continue"
            | "dyn"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "static"
            | "struct"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
            | "abstract"
            | "become"
            | "box"
            | "do"
            | "final"
            | "gen"
            | "macro"
            | "override"
            | "priv"
            | "try"
            | "typeof"
            | "unsized"
            | "virtual"
            | "yield"
    )
}

fn spelling(css_name: &str, reserved: &BTreeSet<&str>) -> Result<(String, bool), String> {
    if css_name.starts_with("--") || css_name.is_empty() {
        return Err(format!("not a static ordinary property: {css_name:?}"));
    }
    let name = css_name
        .strip_prefix('-')
        .unwrap_or(css_name)
        .replace('-', "_");
    if name.is_empty()
        || name == "_"
        || !name.chars().enumerate().all(|(index, ch)| {
            ch.is_ascii_alphabetic() || ch == '_' || (index > 0 && ch.is_ascii_digit())
        })
    {
        return Err(format!("invalid Rust property spelling: {css_name:?}"));
    }
    let suffixed = matches!(name.as_str(), "self" | "Self" | "super" | "crate")
        || reserved.contains(name.as_str())
        || reserved.contains(format!("set_{name}").as_str());
    let name = if suffixed {
        format!("{name}_property")
    } else {
        name
    };
    Ok((
        if keyword(&name) {
            format!("r#{name}")
        } else {
            name
        },
        suffixed,
    ))
}

fn reflections() -> Result<Vec<Reflection>, String> {
    map_inventory(inventory())
}

fn map_inventory(
    inventory: BTreeMap<&'static str, BTreeSet<&'static str>>,
) -> Result<Vec<Reflection>, String> {
    let reserved = RESERVED
        .iter()
        .chain(CAPTURED_RESERVED)
        .copied()
        .collect::<BTreeSet<_>>();
    let mut occupied = reserved
        .iter()
        .map(|name| (name.to_string(), "structural member"))
        .collect::<BTreeMap<_, _>>();
    let mut result = Vec::new();
    for (css_name, origins) in inventory {
        if css_name == "float" {
            continue; // The standard explicit reflection is css_float.
        }
        let (getter, suffixed) = spelling(css_name, &reserved)?;
        let semantic_name = getter.strip_prefix("r#").unwrap_or(&getter);
        let setter = format!("set_{semantic_name}");
        for name in [semantic_name, setter.as_str()] {
            if let Some(previous) = occupied.insert(name.to_owned(), css_name) {
                return Err(format!(
                    "unresolved member collision: {name}: {previous} and {css_name}"
                ));
            }
        }
        result.push(Reflection {
            css_name,
            getter,
            setter,
            origins,
            suffixed,
        });
    }
    Ok(result)
}

fn render(reflections: &[Reflection]) -> String {
    let mut output = String::from(
        "// @generated by surgeist-cssom-reflections; do not edit.\n\
         // Regenerate: cargo run --offline --locked -j 1 -p surgeist-cssom-reflections\n\
         // Provider: CssKnownProperty::{all,canonical_name,aliases,legacy_shorthands}\n\
         // and CssSvgGlyphOrientationVerticalDeclaration::metadata().\n\
         // One pair per authored spelling; float uses the explicit css_float pair.\n\
         // Exactly one vendor hyphen is removed; keywords use raw identifiers.\n\
         // Forbidden raw names and structural collisions use a _property suffix.\n\n\
         #[rustfmt::skip]\n\
         reflected_properties! {\n",
    );
    for reflection in reflections {
        let origins = reflection
            .origins
            .iter()
            .copied()
            .collect::<Vec<_>>()
            .join("; ");
        let suffix = if reflection.suffixed {
            "; structural/raw-name collision: _property suffix"
        } else {
            ""
        };
        writeln!(output, "    // {}: {origins}{suffix}", reflection.css_name).unwrap();
        writeln!(
            output,
            "    {}, {} => {:?};",
            reflection.getter, reflection.setter, reflection.css_name
        )
        .unwrap();
    }
    output.push_str("}\n");
    output
}

fn check_reserved_api(text: &str, reflections: &[Reflection]) -> Result<(), String> {
    let prefix = "pub fn surgeist_cssom::CssStyleProperties::";
    let actual = text
        .lines()
        .filter_map(|line| line.strip_prefix(prefix))
        .map(|line| line.split(['(', '<']).next().unwrap())
        .map(|name| name.strip_prefix("r#").unwrap_or(name))
        .collect::<BTreeSet<_>>();
    let generated = reflections
        .iter()
        .flat_map(|reflection| {
            [
                reflection
                    .getter
                    .strip_prefix("r#")
                    .unwrap_or(&reflection.getter),
                reflection.setter.as_str(),
            ]
        })
        .collect::<BTreeSet<_>>();
    let expected = RESERVED
        .iter()
        .copied()
        .chain(generated.iter().copied())
        .collect::<BTreeSet<_>>();
    if actual != expected {
        let missing = expected.difference(&actual).copied().collect::<Vec<_>>();
        let unexpected = actual.difference(&expected).copied().collect::<Vec<_>>();
        return Err(format!(
            "CssStyleProperties public method inventory differs: missing {missing:?}; unexpected {unexpected:?}. Refresh the owning API artifact and reconcile RESERVED explicitly."
        ));
    }
    let captured = text
        .lines()
        .filter_map(|line| {
            line.strip_prefix(
                "pub fn surgeist_cssom::CssomCapture<surgeist_cssom::CssStyleProperties>::",
            )
            .or_else(|| line.strip_prefix("pub fn surgeist_cssom::CssomCapture<T>::"))
        })
        .map(|line| line.split(['(', '<']).next().unwrap())
        .map(|name| name.strip_prefix("r#").unwrap_or(name))
        .collect::<BTreeSet<_>>();
    let expected = CAPTURED_RESERVED
        .iter()
        .copied()
        .chain(reflections.iter().map(|reflection| {
            reflection
                .getter
                .strip_prefix("r#")
                .unwrap_or(&reflection.getter)
        }))
        .collect::<BTreeSet<_>>();
    if captured != expected {
        return Err("Captured CssStyleProperties public method inventory differs; refresh the API and reconcile CAPTURED_RESERVED explicitly.".into());
    }
    Ok(())
}

/// Run deterministic regeneration or check mode from any working directory.
/// Check mode also checks the actual audited public structural-member inventory.
pub fn run(arguments: impl IntoIterator<Item = String>) -> Result<(), String> {
    let arguments = arguments.into_iter().collect::<Vec<_>>();
    let check = match arguments.as_slice() {
        [] => false,
        [argument] if argument == "--check" => true,
        _ => return Err("usage: surgeist-cssom-reflections [--check]".into()),
    };
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let reflections = reflections()?;
    let expected = render(&reflections);
    let path = root.join(OUTPUT);
    if check {
        let actual = std::fs::read_to_string(&path)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        if actual != expected {
            return Err(format!("stale generated CSSOM reflections: {OUTPUT}"));
        }
        let api = std::fs::read_to_string(root.join(API)).map_err(|error| error.to_string())?;
        check_reserved_api(&api, &reflections)?;
        println!("current: {OUTPUT}; complete property and reserved-member inventories");
    } else {
        std::fs::write(&path, expected).map_err(|error| format!("{}: {error}", path.display()))?;
        println!("generated: {OUTPUT} ({} accessor pairs)", reflections.len());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literal_rust_mapping_controls() {
        let reserved = RESERVED.iter().chain(CAPTURED_RESERVED).copied().collect();
        for (name, expected, suffixed) in [
            ("background-color", "background_color", false),
            ("-webkit-flex", "webkit_flex", false),
            ("type", "r#type", false),
            ("box", "r#box", false),
            ("self", "self_property", true),
            ("Self", "Self_property", true),
            ("super", "super_property", true),
            ("crate", "crate_property", true),
            ("length", "length_property", true),
            ("css-text", "css_text_property", true),
            ("handle", "handle_property", true),
        ] {
            assert_eq!(
                spelling(name, &reserved).unwrap(),
                (expected.into(), suffixed)
            );
        }
        assert!(spelling("--custom", &reserved).is_err());
        assert!(spelling("", &reserved).is_err());
        assert!(spelling("9bad", &reserved).is_err());
        assert!(spelling("-", &reserved).is_err());
        assert!(spelling("_", &reserved).is_err());
    }

    #[test]
    fn unresolved_getter_and_setter_collisions_are_errors() {
        for names in [
            ["foo-bar", "foo_bar"],
            ["foo", "set-foo"],
            ["length", "length-property"],
        ] {
            let inventory = names
                .into_iter()
                .map(|name| (name, BTreeSet::new()))
                .collect();
            assert!(
                map_inventory(inventory)
                    .unwrap_err()
                    .contains("unresolved member collision")
            );
        }
        let mappings = map_inventory(BTreeMap::from([("length", BTreeSet::new())])).unwrap();
        assert_eq!(mappings[0].getter, "length_property");
        assert_eq!(mappings[0].setter, "set_length_property");
    }

    #[test]
    fn complete_public_css_inventory_and_distinct_svg_provenance() {
        let reflected = reflections().unwrap();
        let actual = reflected
            .iter()
            .map(|entry| entry.css_name)
            .collect::<BTreeSet<_>>();
        let mut expected = BTreeSet::new();
        for property in CssKnownProperty::all() {
            expected.insert(property.canonical_name());
            expected.extend(property.aliases().iter().copied());
            expected.extend(
                property
                    .legacy_shorthands()
                    .iter()
                    .map(|grammar| grammar.name()),
            );
        }
        expected.insert(CssSvgGlyphOrientationVerticalDeclaration::metadata().name());
        expected.remove("float");
        assert_eq!(actual, expected);
        for name in [
            "margin",
            "background-color",
            "page-break-before",
            "-webkit-flex",
            "glyph-orientation-vertical",
        ] {
            assert!(
                actual.contains(name),
                "missing independent representative {name}"
            );
        }
        let glyph = reflected
            .iter()
            .find(|entry| entry.css_name == "glyph-orientation-vertical")
            .unwrap();
        assert_eq!(
            glyph.origins,
            BTreeSet::from(["distinct legacy shorthand", "independent SVG declaration"])
        );
        assert!(!actual.contains("--custom"));
    }

    #[test]
    fn artifact_inventory_detects_structural_drift_and_missing_generated_members() {
        let entries = reflections().unwrap();
        let names = RESERVED
            .iter()
            .copied()
            .chain(entries.iter().flat_map(|entry| {
                [
                    entry.getter.strip_prefix("r#").unwrap_or(&entry.getter),
                    entry.setter.as_str(),
                ]
            }));
        let mut text = names
            .map(|name| format!("pub fn surgeist_cssom::CssStyleProperties::{name}()\n"))
            .collect::<String>();
        for name in CAPTURED_RESERVED.iter().copied().chain(
            entries
                .iter()
                .map(|entry| entry.getter.strip_prefix("r#").unwrap_or(&entry.getter)),
        ) {
            writeln!(
                text,
                "pub fn surgeist_cssom::CssomCapture<surgeist_cssom::CssStyleProperties>::{name}()"
            )
            .unwrap();
        }
        assert!(check_reserved_api(&text, &entries).is_ok());
        assert!(
            check_reserved_api(
                &(text.clone()
                    + "pub fn surgeist_cssom::CssStyleProperties::future_structural()\n"),
                &entries
            )
            .is_err()
        );
        assert!(
            check_reserved_api(
                &text.replace("pub fn surgeist_cssom::CssStyleProperties::margin()\n", ""),
                &entries
            )
            .is_err()
        );
    }
}
