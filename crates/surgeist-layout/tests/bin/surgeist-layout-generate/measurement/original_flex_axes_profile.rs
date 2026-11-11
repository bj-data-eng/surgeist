//! Exact inputs for the original Flex axis fixtures.

pub(crate) const SOURCES: &[&str; 20] = &[
    "html/flex/flex_axes_horizontal_tb_column.html",
    "html/flex/flex_axes_horizontal_tb_column_reverse.html",
    "html/flex/flex_axes_horizontal_tb_row.html",
    "html/flex/flex_axes_horizontal_tb_row_reverse.html",
    "html/flex/flex_axes_sideways_lr_column.html",
    "html/flex/flex_axes_sideways_lr_column_reverse.html",
    "html/flex/flex_axes_sideways_lr_row.html",
    "html/flex/flex_axes_sideways_lr_row_reverse.html",
    "html/flex/flex_axes_sideways_rl_column.html",
    "html/flex/flex_axes_sideways_rl_column_reverse.html",
    "html/flex/flex_axes_sideways_rl_row.html",
    "html/flex/flex_axes_sideways_rl_row_reverse.html",
    "html/flex/flex_axes_vertical_lr_column.html",
    "html/flex/flex_axes_vertical_lr_column_reverse.html",
    "html/flex/flex_axes_vertical_lr_row.html",
    "html/flex/flex_axes_vertical_lr_row_reverse.html",
    "html/flex/flex_axes_vertical_rl_column.html",
    "html/flex/flex_axes_vertical_rl_column_reverse.html",
    "html/flex/flex_axes_vertical_rl_row.html",
    "html/flex/flex_axes_vertical_rl_row_reverse.html",
];
const DIGESTS: [&str; SOURCES.len()] = [
    "07cceb0d4b60b342a0ce0789c0c711cef0a4f7d9945cc6d9b3c56e9830be7aea",
    "483dbedd5b12688c6725adfdf0a57960b06c716257dd1c141f8bc123818832b9",
    "3de23bb39b8dd2706c6f3bd81272e66c726930b70642faee0e2cb475674f7a61",
    "e6a828ac6d3ccf114ad3dde82fc33ea015f115c1c1e980d7f0df5681984e1430",
    "8b78543a9eb030bb47bdb3796cf33cceb3abcaa4e13777c4fd41238b111dee14",
    "2b595cb86939d0eebc947f60c2a3b40e577bf8b48e60fecff53f5119e6c761a6",
    "c99176f946c3eae47fb32243082496aaf5db507aee65e80f8ec9c16e2e425278",
    "78f01f6d894c49231cb180558b5c9235e554968400317b58243edb28d7b42244",
    "d59dcb628a2584642410fa044829043326b0710fc5d31446aa66dc2dc2770ab0",
    "4dfc65d3a291a4b089f93b61f90472f5663b6a22c1d6ff485a518be7d1d1fb54",
    "48b2da8265f560729ac4edf44eeb20d6866c959cd7dd8813642640379d387483",
    "7dd041f8c747757c78409a53cc7d220d6dbf3b1f1c7c7fffd099c820d49301bf",
    "a75cde380ae527453424841e9a1558192f10b61c4d152322a363043f3bd8f42b",
    "a36cd00c14d406e89b8071a5f75e6c5d2dc2f9815f10f981ed808a540cd3adac",
    "69d7c338a63c6c71fc8b0c03707fb4935a0d1196b7d3903a0f89fafbb1eced75",
    "f311bb019a05bd85cd9f0dfb0a7ea7f1fb7ecfaa4de89f67531b5794582e2fae",
    "6e0ef3497cefd89b12730507d85a8a94dd77ea5a4d4bc335cff194fa1e7c2b67",
    "e0de050a028375542b6acb02c653d8d31cac0d1abf20ac2b7f5138b1e23c89e0",
    "3f39b0dba9b93bd910261bf95358037b9b1c252eb8278346cfec434bb36bba3f",
    "259139deae98700e02eac34af44fdb6dfc55de3a12ad84a896fecd7a1206a269",
];

pub(crate) fn source_digest(source: &str) -> Option<&'static str> {
    SOURCES
        .iter()
        .zip(DIGESTS)
        .find_map(|(path, digest)| (*path == source).then_some(digest))
}

pub(crate) const SCRIPT_PATH: &str = "scripts/gentest/original_flex_axes.js";
pub(crate) const SCRIPT: &str = include_str!("original_flex_axes.js");
pub(crate) const REQUEST: &[u8] = include_bytes!("original_flex_axes_request.json");

pub(crate) fn accepts_helper(digest: &str) -> bool {
    digest == crate::capture_helper_identity::FIXED_ORTHOGONAL_HELPER_SHA256
}
