use crate::{
    HtmlParseResource, HtmlParseStop, HtmlSource,
    live::{
        HtmlLiveAttribute, HtmlLiveAttributes, HtmlLiveElement, HtmlLiveParseError,
        HtmlLiveParseProgress, HtmlLiveParser, HtmlLiveTree, HtmlLiveUnsupported,
    },
};

use std::num::NonZeroUsize;
use surgeist_html::values::DocumentString;

use super::super::tests::limits;

#[derive(Debug, Eq, PartialEq)]
enum Kind {
    Document,
    Doctype,
    Element(HtmlLiveElement, Vec<(HtmlLiveAttribute, DocumentString)>),
    Comment(DocumentString),
    Text(Vec<u16>),
}

#[derive(Debug, Eq, PartialEq)]
struct Node {
    kind: Kind,
    children: Vec<usize>,
}

struct Tree {
    nodes: Vec<Node>,
    refuse_div: bool,
}

impl Tree {
    fn new() -> Self {
        Self {
            nodes: vec![Node {
                kind: Kind::Document,
                children: Vec::new(),
            }],
            refuse_div: false,
        }
    }

    fn append(&mut self, parent: usize, kind: Kind) -> usize {
        let node = self.nodes.len();
        self.nodes.push(Node {
            kind,
            children: Vec::new(),
        });
        self.nodes[parent].children.push(node);
        node
    }
}

impl HtmlLiveTree for Tree {
    type Node = usize;
    type Error = &'static str;

    fn insert_doctype(&mut self, document: usize) -> Result<(), Self::Error> {
        self.append(document, Kind::Doctype);
        Ok(())
    }

    fn insert_element(
        &mut self,
        parent: usize,
        name: HtmlLiveElement,
        attributes: HtmlLiveAttributes<'_>,
    ) -> Result<usize, Self::Error> {
        if name == HtmlLiveElement::Div && self.refuse_div {
            self.refuse_div = false;
            return Err("element publication refused");
        }
        Ok(self.append(
            parent,
            Kind::Element(
                name,
                attributes
                    .iter()
                    .map(|(name, text)| (name, text.clone()))
                    .collect(),
            ),
        ))
    }

    fn insert_comment(&mut self, parent: usize, text: &DocumentString) -> Result<(), Self::Error> {
        self.append(parent, Kind::Comment(text.clone()));
        Ok(())
    }

    fn insert_text(&mut self, parent: usize, text: &DocumentString) -> Result<(), Self::Error> {
        let previous = self.nodes[parent].children.last().copied();
        if let Some(previous) = previous
            && let Kind::Text(value) = &mut self.nodes[previous].kind
        {
            value.extend(text.code_units());
        } else {
            self.append(parent, Kind::Text(text.code_units().to_vec()));
        }
        Ok(())
    }
}

fn tokens(count: usize) -> NonZeroUsize {
    NonZeroUsize::new(count).unwrap()
}

#[test]
fn paused_prefix_and_suffix_share_text_identity_and_original_eof_is_transferred_once() {
    let original = "<!doctype html><!--before--><html><head></head><body><div class=q id=p>A😀&amp;β</div></body></html>";
    let source = HtmlSource::utf16(DocumentString::from(original));
    let mut tree = Tree::new();
    let mut parser = HtmlLiveParser::new(&source, 0, limits(8)).unwrap();
    assert_eq!(
        parser.advance(&mut tree, tokens(8)).unwrap(),
        HtmlLiveParseProgress::Paused
    );
    assert!(parser.take_stop_parsing().unwrap().is_none());
    assert_eq!(tree.nodes[0].children, [1, 2, 3]);
    assert_eq!(tree.nodes[1].kind, Kind::Doctype);
    assert_eq!(tree.nodes[2].kind, Kind::Comment("before".into()));
    assert_eq!(tree.nodes[5].children, [6]);
    assert_eq!(
        tree.nodes[6].kind,
        Kind::Element(
            HtmlLiveElement::Div,
            vec![
                (HtmlLiveAttribute::Class, "q".into()),
                (HtmlLiveAttribute::Id, "p".into()),
            ]
        )
    );
    assert_eq!(tree.nodes[6].children, [7]);
    assert_eq!(tree.nodes[7].kind, Kind::Text(vec![0x41]));

    assert_eq!(
        parser.advance(&mut tree, tokens(64)).unwrap(),
        HtmlLiveParseProgress::AwaitingEof
    );
    assert!(parser.take_stop_parsing().unwrap().is_none());
    assert_eq!(tree.nodes[6].children, [7]);
    assert_eq!(
        tree.nodes[7].kind,
        Kind::Text("A😀&β".encode_utf16().collect())
    );
    parser.accept_eof();
    assert_eq!(
        parser.advance(&mut tree, tokens(1)).unwrap(),
        HtmlLiveParseProgress::StopParsing
    );
    let stop = parser.take_stop_parsing().unwrap().unwrap();
    assert_eq!(stop.document(), 0);
    assert_eq!(stop.eof().source(), source.identity());
    assert_eq!(stop.eof().start(), original.encode_utf16().count());
    assert!(stop.eof().is_empty());
    assert_eq!(stop.open_elements().collect::<Vec<_>>(), [3, 5]);
    assert!(stop.parser_scripts_are_absent());
    assert!(parser.take_stop_parsing().unwrap().is_none());
    assert_eq!(
        parser.advance(&mut tree, tokens(1)),
        Err(HtmlLiveParseError::HandedOff)
    );
}

#[test]
fn host_refusal_resumes_the_original_element_with_neither_duplicate_nor_skipped_edits() {
    let input = "<!doctype html><html><head></head><body><div>A</div>";
    let source = HtmlSource::utf8(input);
    let mut tree = Tree::new();
    tree.refuse_div = true;
    let mut parser = HtmlLiveParser::new(&source, 0, limits(8)).unwrap();
    assert_eq!(
        parser.advance(&mut tree, tokens(64)),
        Err(HtmlLiveParseError::Host("element publication refused"))
    );
    assert_eq!(tree.nodes.len(), 5);
    let anchor = parser.first_host_refusal().unwrap().clone();
    assert_eq!(anchor.start(), input.find("<div>").unwrap());
    assert_eq!(anchor.end(), input.find('A').unwrap());
    assert_eq!(
        parser.advance(&mut tree, tokens(64)).unwrap(),
        HtmlLiveParseProgress::AwaitingEof
    );
    assert_eq!(parser.first_host_refusal(), Some(&anchor));
    assert_eq!(tree.nodes[0].children, [1, 2]);
    assert_eq!(tree.nodes[4].children, [5]);
    assert_eq!(tree.nodes[5].children, [6]);
    assert_eq!(tree.nodes[6].kind, Kind::Text(vec![0x41]));
    parser.accept_eof();
    assert_eq!(
        parser.advance(&mut tree, tokens(1)).unwrap(),
        HtmlLiveParseProgress::StopParsing
    );
    assert!(parser.take_stop_parsing().unwrap().is_some());
}

#[test]
fn reached_script_refuses_at_the_original_source_before_host_mutation() {
    let input = "<!doctype html><html><head></head><body><div>A</div><script src=x></script>";
    let source = HtmlSource::utf8(input);
    let mut tree = Tree::new();
    let mut parser = HtmlLiveParser::new(&source, 0, limits(8)).unwrap();
    let first = parser.advance(&mut tree, tokens(64)).unwrap_err();
    let HtmlLiveParseError::Unsupported {
        reason: HtmlLiveUnsupported::Element,
        source: span,
    } = &first
    else {
        panic!("unexpected refusal: {first:?}");
    };
    assert_eq!(span.source(), source.identity());
    assert_eq!(&input[span.start()..span.end()], "<script src=x>");
    assert_eq!(tree.nodes.len(), 7);
    parser.accept_eof();
    assert_eq!(parser.advance(&mut tree, tokens(64)), Err(first.clone()));
    assert!(matches!(
        parser.take_stop_parsing(),
        Err(HtmlLiveParseError::Unsupported {
            reason: HtmlLiveUnsupported::Element,
            ..
        })
    ));
    parser.abort().unwrap();
    assert_eq!(parser.advance(&mut tree, tokens(64)), Err(first));
}

#[test]
fn cancellation_after_eof_revokes_unclaimed_handoff_and_abort_releases_custody() {
    let source = HtmlSource::utf8("<!doctype html><html><head></head><body><div></div>");
    let mut tree = Tree::new();
    let mut parser = HtmlLiveParser::new(&source, 0, limits(8)).unwrap();
    parser.accept_eof();
    assert_eq!(
        parser.advance(&mut tree, tokens(64)).unwrap(),
        HtmlLiveParseProgress::StopParsing
    );
    parser.cancellation().cancel();
    assert!(matches!(
        parser.take_stop_parsing(),
        Err(HtmlLiveParseError::Parser(HtmlParseStop::Cancelled))
    ));
    parser.abort().unwrap();
    assert!(parser.usage().is_none());
    assert!(matches!(
        parser.take_stop_parsing(),
        Err(HtmlLiveParseError::Parser(HtmlParseStop::Cancelled))
    ));
}

#[test]
fn open_stack_admission_refuses_before_creating_the_element() {
    let source = HtmlSource::utf8("<!doctype html><html><head></head><body><div></div>");
    let mut tree = Tree::new();
    let mut parser = HtmlLiveParser::new(&source, 0, limits(2)).unwrap();
    let first = parser.advance(&mut tree, tokens(64)).unwrap_err();
    assert!(matches!(
        first,
        HtmlLiveParseError::Parser(HtmlParseStop::LimitExceeded {
            resource: HtmlParseResource::OpenElements,
            ..
        })
    ));
    assert_eq!(tree.nodes.len(), 5);
    assert_eq!(parser.advance(&mut tree, tokens(64)), Err(first));
}

#[test]
fn abort_before_eof_preserves_the_published_prefix_and_cannot_issue_a_continuation() {
    let source = HtmlSource::utf8("<!doctype html><html><head></head><body><div>AB</div>");
    let mut tree = Tree::new();
    let mut parser = HtmlLiveParser::new(&source, 0, limits(8)).unwrap();
    assert_eq!(
        parser.advance(&mut tree, tokens(7)).unwrap(),
        HtmlLiveParseProgress::Paused
    );
    assert_eq!(tree.nodes[6].kind, Kind::Text(vec![0x41]));
    parser.abort().unwrap();
    parser.accept_eof();
    assert!(matches!(
        parser.take_stop_parsing(),
        Err(HtmlLiveParseError::Aborted)
    ));
    assert_eq!(
        parser.advance(&mut tree, tokens(64)),
        Err(HtmlLiveParseError::Aborted)
    );
    assert!(parser.usage().is_none());
    assert_eq!(tree.nodes[6].kind, Kind::Text(vec![0x41]));
}
