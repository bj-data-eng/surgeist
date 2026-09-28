//! Relative color channel arithmetic at the authored parsing boundary.
//!
//! CSS Color 5, 2026-09-08, #relative-syntax and #relative-HSL (and the
//! corresponding RGB, HWB, Lab, LCH, and color() sections) define channel
//! references as numbers. An explicit unit can give a complete expression an
//! angle or percentage type; it does not give the channel reference that type.

use surgeist_css::{CssKnownProperty, parse_style_attribute};

fn assert_accepted(color: &str) {
    let source = format!("color: {color}; opacity: 0.5");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 2, "{source}");
    assert_eq!(
        report.syntax()[0].known().map(|known| known.property()),
        Some(CssKnownProperty::Color),
        "{source}"
    );
    assert_eq!(
        report.syntax()[1].known().map(|known| known.property()),
        Some(CssKnownProperty::Opacity),
        "{source}"
    );
}

fn assert_rejected_with_sibling(color: &str) {
    let source = format!("color: {color}; opacity: 0.5");
    let report = parse_style_attribute(&source);
    assert_eq!(report.diagnostics().len(), 1, "{source}");
    assert_eq!(report.syntax().len(), 1, "{source}");
    assert_eq!(
        report.syntax()[0].known().map(|known| known.property()),
        Some(CssKnownProperty::Opacity),
        "{source}"
    );
}

#[test]
fn relative_channels_accept_number_arithmetic_in_every_selected_environment() {
    for color in [
        "rgb(from red calc(r + 10) calc(g - 10) calc(b + 1))",
        "hsl(from red calc(h + 180) calc(s + 10) calc(l - 10) / calc(alpha * 0.5))",
        "hwb(from red calc(h + 180) calc(w + 10) calc(b - 10))",
        "lab(from red calc(l + 10) calc(a + 1) calc(b - 1))",
        "lch(from red calc(l + 10) calc(c + 1) calc(h + 180))",
        "oklab(from red calc(l + 0.1) calc(a + 0.1) calc(b - 0.1))",
        "oklch(from red calc(l + 0.1) calc(c + 0.1) calc(h + 180))",
        "color(from red display-p3 calc(r + 0.1) calc(g + 0.1) calc(b - 0.1))",
        "color(from red xyz-d50 calc(x + 0.1) calc(y + 0.1) calc(z - 0.1))",
    ] {
        assert_accepted(color);
    }
}

#[test]
fn relative_number_channels_reject_mixed_dimension_addition() {
    for color in [
        "hsl(from red calc(h + 1deg) s l)",
        "hsl(from red h calc(s + 1%) l)",
        "hwb(from red h calc(w + 1%) b)",
        "lab(from red calc(l + 1%) a b)",
        "lch(from red l c calc(h + 1deg))",
        "oklab(from red calc(l + 1%) a b)",
        "oklch(from red l c calc(h + 1deg))",
    ] {
        assert_rejected_with_sibling(color);
    }
}

#[test]
fn relative_number_channels_allow_explicit_unit_multiplication() {
    for color in [
        "hsl(from red calc(h * 1deg) calc(s * 1%) l)",
        "hwb(from red calc(h * 1deg) calc(w * 1%) b)",
        "lab(from red calc(l * 1%) a b)",
        "lch(from red l c calc(h * 1deg))",
    ] {
        assert_accepted(color);
    }
}

#[test]
fn relative_direct_literals_and_foreign_channel_controls() {
    for color in [
        "hsl(from red 180 50 50)",
        "hsl(from red 180deg 50% 50%)",
        "hwb(from red 180 20% 30%)",
        "lab(from red 50 10 20)",
        "lch(from red 50 20 180deg)",
    ] {
        assert_accepted(color);
    }
    for color in [
        "hsl(from red r s l)",
        "hwb(from red h s b)",
        "lab(from red l c b)",
    ] {
        assert_rejected_with_sibling(color);
    }
}
