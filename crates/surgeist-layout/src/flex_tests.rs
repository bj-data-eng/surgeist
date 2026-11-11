use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
};

use proptest::prelude::*;

use crate::flex::FlexAxes;
use crate::geometry::PhysicalProgression;
use crate::test_support::layout_tree::{
    OracleMeasurementOf, OracleTree, OracleTreeOf, PublicLayoutTreeOf,
};
use crate::test_support::scroll_geometry::{
    assert_geometry_error as fri06_mr02_geometry_error_assert, assert_scroll_padding_inputs_exact,
    geometry_error_input as fri06_mr02_geometry_error_input,
    geometry_error_largest_finite as fri06_mr02_geometry_error_largest_finite,
    scroll_padding_inputs,
};
use crate::*;

#[path = "flex_tests/fixtures_tests.rs"]
mod fixtures;

#[path = "flex_tests/alignment_baselines_tests.rs"]
mod alignment_baselines;
#[path = "flex_tests/internal_contracts_tests.rs"]
mod internal_contracts;
#[path = "flex_tests/intrinsic_absolute_scroll_tests.rs"]
mod intrinsic_absolute_scroll;
#[path = "flex_tests/items_tests.rs"]
mod items;
#[path = "flex_tests/lines_distribution_tests.rs"]
mod lines_distribution;
#[path = "flex_tests/wrap_edges_tests.rs"]
mod wrap_edges_tests;

#[path = "flex_tests/maximum_cross_authority.rs"]
mod maximum_cross_authority;

#[path = "flex_tests/used_main_floor.rs"]
mod used_main_floor;

#[path = "flex_tests/named_empty_absence.rs"]
mod named_empty_absence;

#[path = "flex_tests/inline_percentage_completion.rs"]
mod inline_percentage_completion;

#[path = "flex_tests/preferred_main_max_content.rs"]
mod preferred_main_max_content;
