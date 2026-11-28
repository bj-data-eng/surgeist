//! The roots a parse asks for equal a full climb along the parent edges, for every retained
//! html5lib tree-construction case and for sources with the moves the corpus holds few of, with
//! detail off and on.
//!
//! The oracle is the parent edges themselves: every cached root a climb stops at is compared, in
//! a test build, with the root found by following the edges of the record all the way, and a
//! difference ends the test. The test counts how many answers were compared and how many moves
//! of each kind were announced, so that a corpus that stopped exercising the cache is noticed.
use super::g2_d2::each_corpus_parse;
use super::g2e_corpus::{SOURCES, fragment_report};
use super::*;
use crate::construction::tree_root::counts::{self, Counts};

/// Sources that move records between trees and inside one: adoption-agency reroots, foster
/// parenting out of a table, a table inside a template, a template whose contents are moved by
/// the parse of a table, and declarative shadow roots nested in templates and in each other.
const MOVES: [&str; 12] = [
    "<!doctype html><a href=x><b><i><p>x</b>y</i>z</p></a><div><a>1<div>2</a>3</div></div>",
    "<!doctype html><p><b><i><u>1</p>2</b></u>3</i><p>4",
    "<!doctype html><a>1<table><a>2</table>3",
    "<!doctype html><table>t<div><b>x</b></div><tr><td>a</td></tr><span>s</span></table>",
    "<!doctype html><table><tr><td><table>x<div></div><tr><td>y</table></table><b>z</b>",
    "<!doctype html><table><template><tr><td>a</template></table><template><table>b</table></template>",
    "<!doctype html><template><div><template shadowrootmode=open><b></b></template></div></template>",
    "<!doctype html><div><template shadowrootmode=open><div><template shadowrootmode=open>\
     <p></p></template></div></template></div>",
    "<!doctype html><b><table><b></table>x</b>y<svg><b></b><p></p></svg>",
    "<!doctype html><select><button><selectedcontent></selectedcontent></button>\
     <option selected><b><i><p>x</b>y</i></p></option></select>",
    "<!doctype html><form><table><form><input></table><form></form>",
    "<!doctype html><ul><li><b>1<li><i>2<li>3</b></i></ul><dl><dt><a>1<dd>2</a></dl>",
];

#[derive(Default)]
struct Tally {
    parses: usize,
    asked: usize,
    unasked: Vec<String>,
    counts: Counts,
}

impl Tally {
    /// Adds what the thread has counted since the last reset, as one parse's.
    fn take(&mut self, label: &str) {
        let counts = counts::read();
        counts::reset();
        self.parses += 1;
        if counts.queries == 0 {
            self.unasked.push(label.to_string());
        } else {
            self.asked += 1;
        }
        self.counts.queries += counts.queries;
        self.counts.verified += counts.verified;
        self.counts.bare += counts.bare;
        self.counts.same += counts.same;
        self.counts.cross += counts.cross;
    }
}

#[test]
fn every_root_a_parse_asks_for_is_the_root_a_full_climb_finds() {
    let tally = std::cell::RefCell::new(Tally::default());
    counts::reset();
    each_corpus_parse(
        |_, label| tally.borrow_mut().take(label),
        |label| panic!("{label}: parse failed"),
    );
    let corpus = tally.borrow().counts;
    let corpus_parses = tally.borrow().parses;
    let corpus_unasked = tally.borrow().unasked.len();
    let mut tally = tally.into_inner();
    let details = g2_d2::details();
    for source in SOURCES.iter().chain(MOVES.iter()) {
        for detail in &details {
            let label = format!("{source} detail={detail:?}");
            let report = crate::HtmlDocumentParseRequest::offline(
                crate::HtmlSource::utf8(*source),
                g2_d2::corpus_limits(),
            )
            .with_detail(*detail)
            .unwrap()
            .parse()
            .unwrap();
            drop(report);
            tally.take(&label);
            let fragment = source.strip_prefix("<!doctype html>").unwrap_or(source);
            for (context, shadow) in [("div", false), ("div", true), ("svg g", true)] {
                let label = format!("{context} fragment shadow={shadow} {label}");
                drop(fragment_report(fragment, context, shadow, *detail));
                tally.take(&label);
            }
        }
    }
    eprintln!(
        "g2e2 parses {} (corpus {}) asked {} unasked {} (corpus {}) queries {} (corpus {}) \
         verified {} (corpus {}) bare {} same {} cross {} (corpus {} {} {})",
        tally.parses,
        corpus_parses,
        tally.asked,
        tally.unasked.len(),
        corpus_unasked,
        tally.counts.queries,
        corpus.queries,
        tally.counts.verified,
        corpus.verified,
        tally.counts.bare,
        tally.counts.same,
        tally.counts.cross,
        corpus.bare,
        corpus.same,
        corpus.cross,
    );
    for label in tally.unasked.iter().take(10) {
        eprintln!("unasked: {label}");
    }
    assert_eq!(corpus_parses, 1_767 * details.len());
    assert_eq!(
        tally.parses,
        corpus_parses + 4 * details.len() * (SOURCES.len() + MOVES.len())
    );
    // Every parse of the corpus and of the sources asks for at least one root.
    assert!(tally.unasked.is_empty(), "{:?}", tally.unasked);
    // The floors are for each detail the build parses with.
    let runs = u64::try_from(details.len()).expect("a count of details fits in u64");
    assert!(
        tally.counts.verified >= 18_000 * runs,
        "{} answers compared",
        tally.counts.verified
    );
    assert!(
        tally.counts.bare >= 12_000 * runs,
        "{} bare moves",
        tally.counts.bare
    );
    // A node that is moved is detached first, so no move of one with children stays in its tree.
    assert_eq!(tally.counts.same, 0);
    assert!(
        tally.counts.cross >= 400 * runs,
        "{} cross-tree moves",
        tally.counts.cross
    );
}
