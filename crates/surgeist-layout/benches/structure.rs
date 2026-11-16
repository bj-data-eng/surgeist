//! Release-mode layout benchmark over deterministic generated box trees.
//!
//! Each document is a generated tree of native boxes. A round runs one full
//! `compute_layout` (or, for block fragmentation, the bound flow capture and
//! `compute_layout_with_flow` over the bound request) from the root, including the host-graph
//! validation the call performs. The report lists
//! each document's node count and depth, with the root at depth one, beside the median, minimum
//! and maximum time over the rounds. The documents are deep chains of Block, Flex and Grid
//! containers, wide trees, nested Flex and Grid containers with intrinsic sizing and shrinking
//! items, and native Block fragmentation over many pages.
//!
//! Run with `cargo bench --offline --locked -j 1 -p surgeist-layout --bench structure`.
//! A first argument without a leading dash selects the documents whose name contains it, as in
//! `--bench structure -- chain-grid`. A name that matches no document exits with status 2 and
//! lists the known names.

#[path = "support/harness.rs"]
mod harness;
#[path = "support/host.rs"]
mod host;

use std::hint::black_box;

use harness::{Document, Family};
use host::{Host, boxed, flow_axes, px, root_request};
use surgeist_layout::{
    BlockFragmentainerOf, Display, FlexWrap, FragmentationBreakKinds, FragmentationKind,
    LayoutBoundRequestOf, LayoutFlowInput, LayoutSourceBasis, PageProgression, Point, Size,
    TrackComponent, WrappingInput, compute_layout, compute_layout_with_flow,
};

const GRID_COLUMNS: usize = 100;

fn plain(host: Host, width: f32) -> Document {
    let (nodes, depth) = host.shape();
    let request = root_request(width);
    Document {
        nodes,
        depth,
        round: Box::new(move || {
            black_box(compute_layout(&host, 0, request).expect("the generated document lays out"));
        }),
    }
}

fn container(display: Display) -> surgeist_layout::NodeInput {
    let mut input = boxed(display, None, None);
    if display == Display::Grid {
        input.grid_template_columns = vec![TrackComponent::AUTO];
        input.grid_template_rows = vec![TrackComponent::AUTO];
    }
    input
}

/// A single-child chain of containers ending in one fixed leaf.
fn chain(display: Display, levels: usize, fixed: bool) -> Document {
    let level = || {
        let mut input = container(display);
        if fixed {
            input.size = Size::new(px(500.0), px(500.0));
        }
        input
    };
    let mut host = Host::new(level());
    let mut parent = 0;
    for _ in 1..levels {
        parent = host.add(parent, level());
    }
    host.add(parent, boxed(Display::Block, Some(30.0), Some(20.0)));
    plain(host, 600.0)
}

fn wide_block(siblings: usize) -> Document {
    let mut host = Host::new(container(Display::Block));
    for _ in 0..siblings {
        host.add(0, boxed(Display::Block, None, Some(10.0)));
    }
    plain(host, 600.0)
}

fn wide_flex_wrap(siblings: usize) -> Document {
    let mut root = container(Display::Flex);
    root.flex_wrap = FlexWrap::Wrap;
    let mut host = Host::new(root);
    for _ in 0..siblings {
        host.add(0, boxed(Display::Block, Some(10.0), Some(10.0)));
    }
    plain(host, 600.0)
}

fn wide_grid(siblings: usize) -> Document {
    let mut root = container(Display::Grid);
    root.grid_template_columns = vec![TrackComponent::AUTO; GRID_COLUMNS];
    let mut host = Host::new(root);
    for _ in 0..siblings {
        host.add(0, boxed(Display::Block, Some(10.0), Some(10.0)));
    }
    plain(host, 1_200.0)
}

/// A complete tree of `fan` children per container down to `levels` container levels, whose
/// leaves are host-measured. Container widths and flex bases are automatic.
fn nested(display: Display, fan: usize, levels: usize) -> Document {
    fn grow(host: &mut Host, parent: u32, display: Display, fan: usize, levels: usize) {
        for _ in 0..fan {
            if levels == 1 {
                let leaf = host.add(parent, boxed(Display::Block, None, None));
                host.measured[leaf as usize] = true;
            } else {
                let mut input = container(display);
                if display == Display::Grid {
                    input.grid_template_columns = vec![
                        TrackComponent::AUTO,
                        TrackComponent::MAX_CONTENT,
                        TrackComponent::AUTO,
                    ];
                }
                let child = host.add(parent, input);
                grow(host, child, display, fan, levels - 1);
            }
        }
    }
    let mut root = container(display);
    if display == Display::Grid {
        root.grid_template_columns = vec![
            TrackComponent::AUTO,
            TrackComponent::MAX_CONTENT,
            TrackComponent::AUTO,
        ];
    }
    let mut host = Host::new(root);
    grow(&mut host, 0, display, fan, levels);
    plain(host, 300.0)
}

/// Sections of fixed-height rows in Block fragmentation over many equal pages.
fn fragmented(sections: usize, rows: usize, page_height: f32) -> Document {
    let mut host = Host::new(container(Display::Block));
    for _ in 0..sections {
        let section = host.add(0, container(Display::Block));
        for _ in 0..rows {
            host.add(section, boxed(Display::Block, None, Some(10.0)));
        }
    }
    let (nodes, depth) = host.shape();
    let content = sections as f32 * rows as f32 * 10.0;
    let pages = (content / page_height).ceil() as usize;
    let frames: Vec<_> = (0..pages)
        .map(|page| BlockFragmentainerOf {
            origin: Point::ZERO,
            size: Size::new(600.0, page_height),
            breaks: if page == 0 {
                FragmentationBreakKinds::NONE
            } else {
                FragmentationBreakKinds::PAGE
            },
            page_side: None,
        })
        .collect();
    let request = LayoutBoundRequestOf::block_fragmentation(
        LayoutSourceBasis::fresh(),
        root_request(600.0),
        FragmentationKind::Page {
            progression: PageProgression::LeftToRight,
        },
        flow_axes(),
        frames,
    )
    .expect("the page frames are valid");
    Document {
        nodes,
        depth,
        round: Box::new(move || {
            let capture = LayoutFlowInput::try_new_bound(
                &host,
                0,
                request.clone(),
                Vec::new(),
                WrappingInput::default(),
                Vec::new(),
            )
            .expect("the generated document binds");
            black_box(
                compute_layout_with_flow(&capture, 0, request.root_request())
                    .expect("the generated document fragments"),
            );
        }),
    }
}

static FAMILIES: &[Family] = &[
    Family {
        name: "chain-block-1000",
        build: || chain(Display::Block, 1_000, false),
    },
    Family {
        name: "chain-flex-auto-30",
        build: || chain(Display::Flex, 30, false),
    },
    Family {
        name: "chain-flex-fixed-100",
        build: || chain(Display::Flex, 100, true),
    },
    Family {
        name: "chain-grid-auto-10",
        build: || chain(Display::Grid, 10, false),
    },
    Family {
        name: "chain-grid-fixed-1000",
        build: || chain(Display::Grid, 1_000, true),
    },
    Family {
        name: "wide-block-10000",
        build: || wide_block(10_000),
    },
    Family {
        name: "wide-flex-wrap-10000",
        build: || wide_flex_wrap(10_000),
    },
    Family {
        name: "wide-grid-1000",
        build: || wide_grid(1_000),
    },
    Family {
        name: "nested-flex-intrinsic-3",
        build: || nested(Display::Flex, 3, 3),
    },
    Family {
        name: "nested-flex-intrinsic-4",
        build: || nested(Display::Flex, 3, 4),
    },
    Family {
        name: "nested-grid-intrinsic-2",
        build: || nested(Display::Grid, 3, 2),
    },
    Family {
        name: "nested-grid-intrinsic-3",
        build: || nested(Display::Grid, 3, 3),
    },
    Family {
        name: "fragmentation-pages-10",
        build: || fragmented(5, 20, 100.0),
    },
    Family {
        name: "fragmentation-pages-20",
        build: || fragmented(10, 20, 100.0),
    },
];

fn main() {
    harness::run("structure", FAMILIES);
}
