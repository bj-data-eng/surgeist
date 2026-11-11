//! Exact inputs for the original Subgrid axis fixtures.

pub(crate) const SOURCES: &[&str; 9] = &[
    "html/subgrid/subgrid_axes_horizontal_parent_orthogonal_child.html",
    "html/subgrid/subgrid_axes_horizontal_tb_parallel.html",
    "html/subgrid/subgrid_axes_sideways_lr_parallel.html",
    "html/subgrid/subgrid_axes_sideways_opposing.html",
    "html/subgrid/subgrid_axes_sideways_rl_parallel.html",
    "html/subgrid/subgrid_axes_vertical_lr_parallel.html",
    "html/subgrid/subgrid_axes_vertical_opposing.html",
    "html/subgrid/subgrid_axes_vertical_parent_orthogonal_child.html",
    "html/subgrid/subgrid_axes_vertical_rl_parallel.html",
];
const DIGESTS: [&str; SOURCES.len()] = [
    "d818a00e9c3b4992bae806b9ee0c412de2f8d78aa09710d0ddecfb3f4d3dc2dd",
    "4ba62fbfd85bc039302566656f5e71d42aa6253a7821d487366703c8a95460b7",
    "a2328653fdcbb11e9f18b7460af532c360c812032519147e3a5318304a864112",
    "2e2993ad939905d0d8a4299d9674e77e478766952524531887e1bcce18c984bf",
    "67044201046bcc52eb21d4e78afe927620cfd7e636fb6f595b6ac33871515685",
    "1f4d87ba0d4de4c07b9c429e00925411f2589c99d237faa0c414ab92b7570fef",
    "6ba0ba9d22e80125c753e544404cc99d8b36ee73d6a8094b11da0054949be03f",
    "d891897d16e21ff2624a1a1a35adccb8a38128b4e9e83ccca9a182cd91e04c1c",
    "4c764e6c42e41ebfff1148ae07dfa1af75f35465af5d05f69aa85cb0fa698c7b",
];

pub(crate) fn source_digest(source: &str) -> Option<&'static str> {
    SOURCES
        .iter()
        .zip(DIGESTS)
        .find_map(|(path, digest)| (*path == source).then_some(digest))
}

pub(crate) const SCRIPT_PATH: &str = "scripts/gentest/original_subgrid_axes.js";
pub(crate) const SCRIPT: &str = include_str!("original_subgrid_axes.js");
pub(crate) const REQUEST: &[u8] = include_bytes!("original_subgrid_axes_request.json");

pub(crate) fn accepts_helper(digest: &str) -> bool {
    digest == crate::capture_helper_identity::FIXED_ORTHOGONAL_HELPER_SHA256
}
