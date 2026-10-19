# Selected HTML conformance coverage

This matrix connects selected language requirements to executable expectations
and their actual construction contexts. The retained parsing and corpus
selection is anchored at `729e551b5775278ffd1079e20dea582d12acea01`;
later intrinsic construction evidence is identified by its owning tests below.
This is not a full-browser conformance certificate. The [selected source editions](../../../references/SOURCE-EDITIONS.md#pinned-html-language-baseline)
make HTML `877d6146487f15b15bc8dd0e3c625063a37b456a` the language
authority. html5ever `d7232d746d3112f08926d169591576bd12fe2fdf`
identifies implementation provenance, not a different language.

An **independent expectation** is a source-derived literal or retained corpus
result, rather than another parse of the same input. **Ownership evidence**
checks admission, qualification, cancellation or publication through actual
owners; it does not independently establish every language outcome.
**Comparative evidence** compares producers under a declared projection.
**Measurements** characterize the recorded workload and build, without turning
its limits or observed costs into universal product thresholds.

Test names below identify ordinary Rust tests in the linked owning files.
“Every cut” means all two-part source partitions plus unit-at-a-time exposure
for that test's selected literals. Public requests consume complete, already
decoded Unicode sources. Progressive exposure is a private ordinary-document
operation; it is not public transport streaming or fragment feeding.

## Parsing and construction contexts

| Selected defining clauses | Actual context and independent expectations | Executable evidence and qualification |
| --- | --- | --- |
| [Preprocessing the input stream][preprocessing] and [tokenization][tokenization] | UTF-8 byte and UTF-16 unit coordinates remain tied to the original source; CR/CRLF processing, literal U+FEFF, NUL recovery, supplementary characters and isolated surrogates have distinct outcomes. | [Conformance tests][conformance]: `literal_utf16_surrogate_pair_crlf_and_reference_keep_distinct_source_origins`, `literal_initial_pi_and_leading_feff_preserve_source_defined_mode_and_errors`. [Public tests][public]: `successful_utf16_keeps_bom_surrogate_newline_origins_and_ordered_recovery_diagnostics`. Already-decoded U+FEFF is not external byte-BOM detection. |
| [Initial][initial], [in head][inhead], [in body][inbody] and [after head][afterhead] rules | Fresh ordinary documents, Disabled scripting: explicit/implied structure, mode selection, late head input, void elements, body recovery and exact source order. | [Tree tests][tree]: `owning_head_places_prehead_comment_pi_and_head_void_children_in_source_order`, `body_generic_names_void_recovery_rawtext_and_plaintext_follow_actual_source_rows`, `afterhead_template_removes_exact_head_identity_and_retains_source_diagnostic`. Corpus mode bindings are listed below; they are not derived from actual output. |
| [Character-reference states][references] and [named reference table][names] | Selected named, decimal and hexadecimal references; longest match, attribute exception, failed numeric rewind, invalid scalars, C1/control/noncharacter recovery and semicolon chronology. UTF-8 and UTF-16, whole source and every cut. | [Conformance tests][conformance]: `reference_recovery_keeps_literal_typed_records_and_exact_utf8_utf16_output_spans`, `reference_recovery_every_cut_and_unit_feed_uses_independent_records_and_output_atoms`, `attribute_reference_exception_keeps_literal_quotes_and_two_output_value_atoms_at_every_cut`. [Public tests][public]: `recovered_references_public_reader_preserves_typed_chronology_and_exact_dual_coordinates`. Selected outputs, typed diagnostics and overlapping source maps are separate assertions. |
| [Numeric-reference states][numericreferences] | Complete ordinary execution of 512-digit decimal and hexadecimal references in both encodings: out-of-range replacement, distinct native text identities, NoQuirks structure and the earlier NULL-reference diagnostic followed by the out-of-range diagnostic. Exact original-source intervals distinguish UTF-8 bytes from UTF-16 units. | [Tree tests][tree]: `decimal_and_hex_complete_partners_have_literal_replacement_and_retained_diagnostics`. This positive language oracle executes four Complete documents without instrumentation; it is separate from the cancellation/work-refusal partners below and does not certify every long reference or D1 row. |
| [Comment states][comments] | All 16 retained comments01 inputs, Disabled ordinary documents, both encodings and every cut; 27 selected chronological typed diagnostics. Separate UTF-16 literals cover surrogate pairs, isolates and CRLF. | [Conformance tests][conformance]: `all_comments01_disabled_cases_match_selected_typed_diagnostics_and_native_trees`, `comments01_utf8_utf16_every_cut_and_unit_feed_keep_literal_trees_and_typed_diagnostics`, `comment_utf16_surrogate_crlf_pair_and_end_bang_keep_exact_evidence_across_all_cuts`. These selected source expectations do not validate every historical corpus error section. |
| [Processing-instruction states][pi] | Eighteen source literals distinguish target case, data whitespace, literal ampersands/NUL, question-mark fallback, both terminators, invalid/reserved-target comment recovery and unfinished EOF drafts. Both encodings and every cut; separate paired/isolated-surrogate and CRLF data. | [Conformance tests][conformance]: `pi_recovery_and_eof_keep_literal_native_records_and_original_field_delimiters`, `pi_recovery_every_cut_and_unit_feed_keeps_independent_chronology_and_source_atoms`, `pi_utf16_non_scalars_pair_and_crlf_keep_exact_data_at_every_feed_boundary`. [Public tests][public]: `pi_recovery_chronology_and_abandoned_eof_are_public_exact_source_evidence`. Typed processing state and normative rule anchor remain separate; an engine EOF branch label is not a replacement source rule. |
| [Active formatting and adoption][inbody], [in table][intable] and [in table text][intabletext] | Independently expected ordinary trees plus hand-derived actual sink operations: foster routing, preceding-text merge, detached/moved tables, template precedence, reconstruction and adoption. | [Tree tests][tree]: `foster_text_uses_actual_table_parent_and_merges_only_preceding_text`, `foster_parent_uses_moved_table_parent_instead_of_stack_predecessor`, `latest_template_or_table_determines_foster_route`, `formatting_adoption_outer_loop_stops_after_exactly_eight_source_iterations`. `progressive_partitions_keep_fostered_native_destination_and_buffered_diagnostic_roles` checks a selected buffered-text diagnostic sequence and Replay destination across every UTF-8 cut. All 19 retained table trees run; their entire historical error metadata is not certified. |
| [Initial][initial], [in body EOF][inbody], [in table][intable], [in table text][intabletext], [in column group][incolgroup] and [in table body][intbody] | Tables01/1,4,6,19 have independent typed sequences of 3/7/14/5 records: missing doctype, implied-row/ignored-tag recovery, buffered-list and per-character foster errors, ignored late doctype and disallowed-open-element EOF. Both encodings, Complete execution and every cut. | [Conformance tests][conformance]: `table_recovery_literal_diagnostics_complete_utf8_utf16`, `table_recovery_literal_diagnostics_every_cut_and_unit_feed`. Eight Complete, 312 single-cut and eight unit-feed documents preserve literal trees, Quirks mode, chronological records and qualified ranges. Processing context records the actual outer emitter; a delegated InBody rule can retain InCell/InTable context. ASCII fixtures exercise both coordinate kinds, not surrogate/CRLF behavior. |
| [Foreign-content rules][foreign] | Ordinary documents and captured SVG/MathML contexts: exact namespaces, adjusted SVG names/attributes, XML/XLink/XMLNS distinctions, HTML integration points, MathML text exceptions, CDATA, breakout and SVG script metadata. | [Tree tests][tree]: `ordinary_svg_adjusted_names_and_foreign_integration_preserve_native_namespaces_and_order`, `foreign_all_namespace_adjustments_unknown_colons_and_xmlns_errors_keep_original_values`. [Fragment tests][fragment]: `captured_integration_points_use_actual_qualified_encoding_and_math_text_exceptions`, `arbitrary_captured_namespace_is_copied_exactly_and_cdata_uses_adjusted_context`. Script metadata does not establish script execution. |
| [In template][intemplate] and [template construction][template] | 111 ordinary template inputs in both encodings, actual inert template documents and separate hosted contents. Seven selected transitions run every cut in both encodings, totaling 670 progressive runs. | [Conformance tests][conformance]: `all_retained_ordinary_templates_match_independent_trees_in_both_encodings`, `template_transitions_every_cut_and_unit_feed_match_independent_trees`, `nested_template_eof_diagnostics_retain_two_distinct_source_invocations`. [Tree tests][tree]: `ordinary_template_routes_rows_into_separate_hosted_contents_and_keeps_body_sibling_order`, `template_selectedcontent_deep_clone_keeps_distinct_contents_and_adopts_real_documents`. EOF records at the same position remain separate invocations. |
| [Fragment parsing][fragments] | Actual captured contexts select tokenizer state, insertion mode, namespace/integration attributes, document mode and nearest-form facts. Inert/Disabled profiles retain parser Document, context document, target document and output Fragment separately. Whole source only. | [Fragment tests][fragment]: `real_div_fragment_returns_only_parsed_children_and_inherits_parser_document_mode`, `real_table_context_selects_table_mode_without_context_wrapper_or_implied_table`, `real_utf16_rcdata_context_preserves_isolates_entity_cr_and_literal_end_like_units`. Corpus template/109 uses a real template capture, checked by [Conformance tests][conformance]: `retained_template_context_fragment_matches_actual_output_and_content_owner`. No div wrapper or invented appropriate end tag substitutes for context. |
| [Template/shadow construction][template] and [fragment parsing][fragments] | Qualified declarative shadow decisions can create owned roots or retain captured attachment effects. Light children, separate shadow/template trees, captured identity and immutable output remain distinct. | [Fragment tests][fragment]: `captured_shadow_attachment_retains_actual_root_and_second_template_uses_planned_presence`, `local_shadow_host_keeps_light_children_and_qualified_root_separate`, `selectedcontent_clones_local_clonable_shadow_after_light_children`. [Public tests][public]: `fragment_parse_retains_actual_captured_attachment_and_fallback_after_cancel`. These are construction results/effects, not installation into a live DOM. |
| [Script processing][scripts] and parser script states | Disabled ordinary HTML script preparation, Inert fragment scripts, raw/escaped text and separately selected SVG processed-state transitions. No JavaScript evaluation or network activity. | [Tree tests][tree]: `standalone_script_end_prepares_disabled_classic_and_restores_head_and_insertion_state`, `standalone_script_eof_marks_started_without_preparing_or_changing_initial_facts`, `svg_script_parser_end_defers_then_restores_wrapper_without_html_preparation`. [Fragment tests][fragment]: `inert_fragment_scripts_retain_parser_document_and_started_state_without_disabled_preparation`. Scripting-Enabled corpus replay is excluded. |

## Intrinsic values, authoring and semantic queries

These operations have different callers. Native authoring builds immutable
values without parsing or generic content-conformance checking. Pure intrinsic
rules return effects for their owning adapter to apply. Semantic queries borrow
capture-qualified facts, with missing/inapplicable/unsupported outcomes rather
than guessed live state. A recognized element or attribute name alone does not
establish an implemented semantic operation.

| Selected source or contract | Actual caller/context and expected outcome | Executable evidence and limits |
| --- | --- | --- |
| [DOM-tree terminology][domtrees], [native construction contract](../../surgeist-html/README.md) and [PI syntax][pi] | `HtmlDocument`/`HtmlFragment` builders retain exact UTF-16, namespace/prefix/ordered attributes, distinct qualified IDs, mixed sibling order and separate template/shadow ownership. Checked PI authoring remains distinct from recovered parser input. | [Construction tests][construction]: `mixed_siblings_retain_order_and_exact_utf16_without_reparsing`, `strict_pi_authoring_follows_html_target_and_data_rules`, `template_contents_are_a_separate_fragment_with_a_reverse_association`, `shadow_host_and_contents_keep_properties_and_separate_ordinary_children`, `each_finite_budget_rejects_atomically_and_later_smaller_append_succeeds`. Builder limits count retained construction content; they do not provide parser diagnostics or a process-memory bound. |
| [Boolean/enumerated microsyntaxes][microsyntaxes] and element-specific attribute definitions | Attribute interpretation distinguishes absent, empty, locally invalid, Unapplied and context-Unchecked results. Exact expanded names and the original raw value are retained; applicability does not manufacture host facts. | [Attribute tests](../../surgeist-html/tests/attributes.rs): `disabled_local_presence_uses_each_standard_elements_defining_clause`, `disabled_custom_controls_require_actual_supplied_form_association`, `disabled_admits_exact_raw_units_before_even_unapplied_contexts`. [Interpretation tests][interpretation]: `absent_empty_and_admission_are_separate_from_syntax`. Selected processing and author restrictions remain distinct from global content validation. |
| [Common microsyntaxes][microsyntaxes] | Checked author grammars and recovery processing remain distinct. Raw values survive recovery, limits and isolated surrogates; binary64 conversion overflow is distinct from invalid number spelling. | [Interpretation tests][interpretation]: `integer_author_grammar_and_recovery_have_distinct_results`, `floating_point_overflow_is_not_lexical_invalidity`, `exact_time_recovery_preserves_long_fractions_without_author_projection`. These selected numeric/temporal tests are not blanket coverage of every contextual attribute grammar. |
| [HTML media surrounding rules][htmlmedia] | Exact presence/empty/ASCII-whitespace branches, per-element applicability, picture sibling requirements and ordinary-document theme-color uniqueness. CSS grammar and matching are separate qualified observations. | [Media tests](../../surgeist-html-semantics/tests/media_rules.rs): `role_defaults_empty_match_and_forbidden_placement_remain_independent_of_css`, `external_grammar_matching_and_missing_are_independent_and_qualified`, `picture_requirement_uses_only_actual_following_siblings_and_presence`, `semantic_atomic_limits_cancellation_scratch_release_and_sticky_stop`. Missing environment is not mismatch; HTML performs no CSS matching or resource application. |
| [Template marker selection][template] | Captured same-tree for relation, independent author witness, first marker/start selection and following sibling start/end counting. Shared XML pseudo-attribute grammar rejects duplicate names and malformed suffixes. | [Captured tests](../../surgeist-html-semantics/tests/semantics_template_for.rs): `first_marker_wins_over_later_start_and_later_missing_subtree`, `earlier_unknown_descendants_preserve_later_author_witness_only`, `template_contents_and_shadow_associations_do_not_join_ordinary_marker_trees`. The [native template owner](../src/construction/template.rs) applies insertion/replacement through construction; captured queries do not install live effects. |
| [Microdata model and algorithms][microdata] | Item eligibility, raw property values, first-ID itemref lookup, source-ordered crawl, nested-item pruning, type paths, cycle/revisit distinctions and independent known versus incomplete coverage. | [Microdata tests](../../surgeist-html-semantics/tests/semantics_microdata.rs): `properties_sort_in_tree_order_and_nested_items_prune`, `item_graph_cycle_is_distinct_from_crawl_revisit`, `property_url_uses_qualified_historical_operation_not_current_parse`, `partial_receiving_crawl_preserves_proved_type_seed_and_contribution`. Actual URL/vocabulary facts remain qualified external inputs; this is no network crawler or schema registry. |
| [Nonce lifecycle][nonce] | Cryptographic slot differs from content. Initialization, content mutation, slot writes, clone and conditional connected header-CSP hiding run at native construction hooks; semantic readback observes the actual slot. | [Public tests](../tests/parser/construction/nonce.rs): `standalone_nonce_slots_preserve_content_reference_and_absence_in_both_encodings`, `captured_connected_shadow_header_policy_hides_content_preserving_real_slot`, `captured_connected_unknown_header_is_typed_failure_not_false`. [Semantic tests](../../surgeist-html-semantics/tests/semantics_nonce.rs): `getter_observes_exact_slot_without_content_or_policy_dependency`. Unknown policy is not false; CSP parsing/enforcement and live setters remain external. |
| [Document base URLs][baseurls] and target selection | First href and target candidates are independent. Historical frozen base differs from current getter; non-href updates preserve it. Target overrides, own empty values and source recovery precede default selection. | [Captured tests](../../surgeist-html-semantics/tests/semantics_base.rs): `captured_first_href_is_historical_while_getter_uses_current_parse_and_target_is_independent`, `unresolved_first_href_cannot_skip_to_a_later_qualified_frozen_base`. [Public tests](../tests/parser/construction/base_target.rs): `first_href_binds_decoded_value_and_actual_node_before_later_bases`. The [native owner](../src/construction/base.rs) tests mutation/clone hooks; URL and CSP observations are inputs, not a URL engine or navigation. |
| [Link relationships][linktypes] and document selectors | Role-qualified rel/rev effects, aliases/support capabilities, native creating-parser versus current-document history, formed noopener inputs, exact feed types, first manifest and icon tree-order selection/retry. | [Native rules](../../surgeist-html/tests/rel_native.rs): `defining_role_families_keep_effects_and_wrong_role_tokens_separate`. [History tests](../tests/parser/construction/link_history.rs): `template_link_birth_document_differs_from_its_actual_inert_node_document`. [Captured rules](../../surgeist-html-semantics/tests/semantics_rel.rs): `noopener_ordered_dominance_and_actual_url_settings_facts_cover_all_roles`. [Selector tests](../../surgeist-html-semantics/tests/semantics_rel_selectors.rs): `feed_discovery_includes_stylesheet_alternates_and_default_is_first_declaration`, `equally_appropriate_icons_use_tree_order_and_rejection_tries_same_group_before_lower_group`. Qualified suitability, media and actual attempted-use observations are required when influential; these operations do not fetch, install or decode resources. |
| [Input states and APIs][input] | Shared canonical Hidden/Text/Search/Telephone/Password, Checkbox/Radio, Email/URL, Submit/Reset/Button, Date/Month/Week/Time/LocalDateTime, Number, File/Image, Range and Color rules through canonical `HtmlInputType`; ordered fresh-state initialization, reset/type/clone flags and actual parser consumption. | [Input rules][inputtests]: `authoritative_type_recognition_preserves_all_states_and_text_defaults`, `reset_orders_flags_default_checked_files_then_sanitization`, `state_changes_have_source_order_and_same_state_has_no_type_effects`. [Tree tests][tree]: `ordinary_text_input_keeps_exact_default_value_and_form_siblings_through_eof`, `ordinary_temporal_input_local_datetime_normalizes_separator_and_shortest_time_only`, `ordinary_number_input_initial_value_is_not_mandatorily_clamped_or_step_rounded`, `ordinary_range_input_step_ties_and_base_precedence_use_actual_content_not_default_minimum`. Color's controlled integration and capability limits are listed separately below. |
| [Input value modes][input] | DefaultOn supplies `on` only for absent Checkbox/Radio value content; present empty stays empty. Default-mode buttons preserve content with no display-label fallback. Multiple Email retains the produced count even when joined strings coincide. | [Email/URL rules](../../surgeist-html/tests/input_email_url_rules.rs): `multiple_email_preserves_actual_empty_field_count_after_joining`. [Public tests][public]: `canonical_control_metadata_lends_actual_nodes_flags_raw_values_and_form_identity`, `multiple_email_count_retains_produced_empty_values_and_immutable_source_after_cancel`, `button_default_value_preserves_ordered_content_and_retained_raw_after_cancel`, `local_datetime_normalization_retains_exact_space_origin_and_original_value_history`. Source/native/default/current values are independent assertions. |
| [Color state and serialization][colorinput] | Value mode without selection; clean/default versus dirty/current updates, ordered type/alpha/colorspace effects, reset and clone. Actual alpha presence and Limited sRGB/Display P3 determine conversion and serialization. Only standard invalidity selects black; unavailable CSS facts, numerical/unsupported processing, resource refusal and cancellation remain distinct outcomes. | [Public Color tests](../tests/parser/construction/input/color.rs): `color_input_preserves_red_default_and_publishes_sanitized_current_with_form_sibling`, `color_independent_literals_preserve_defaults_and_source_ordered_alpha_colorspace`, `color_utf16_reference_default_and_generated_history_keep_distinct_actual_values`, `color_currentcolor_without_palette_is_unavailable_and_reclaims_partial_construction`. [Controlled adapter tests](../src/construction/input_color.rs): `color_managed_clean_dirty_reset_and_clone_keep_actual_value_and_lineage`, `color_type_removal_and_number_transition_keep_generated_source_whole_or_omitted`, `color_initial_header_one_short_and_cancel_preserve_first_stop_before_input`. Generated computation causes retain dependency groups without fabricated lexical atoms. CSS-owned grammar/serialization is consumed through the original controller; these cases do not establish style resolution, host palettes/profiles, a live DOM or interactive wells. |
| [File/Image input states][input] | File starts with an empty selected-resource fact; native Filename readback differs from value content. Offline Image selects Disabled image support; fragment creation requires actual qualified environment evidence when influential. | [File/Image public tests](../tests/parser/construction/input/file_image.rs): `filename_is_empty_with_exact_raw_and_current_default_fields_and_real_form_owner`, `fragment_image_policy_is_explicit_conditional_and_capture_qualified`, `temporary_and_nested_inert_documents_require_actual_creation_environment_attestation`. No file picker, image fetch, decoding or live resource transition is performed. |
| [Select][select], [textarea][textarea], [output][output] and [selectedcontent][selectedcontent] | Actual parser flags, raw/current values, source-ordered reset/form association and cloned contents; pure flags alone do not perform tree effects. | [Tree tests][tree]: `ordinary_table_textarea_delegation_preserves_foster_tree_and_returns_to_table`, `ordinary_table_output_fosters_actual_container_and_restores_cell_dispatch`, `ordinary_eof_pops_open_option_and_clones_selected_content_before_completion`. [Text semantic tests](../../surgeist-html-semantics/tests/semantics_control_text.rs): `textarea_default_is_only_immediate_text_exact_utf16_and_ignores_unknown_descendants`, `missing_override_is_not_inferred_null_even_with_complete_text`. |
| [Details/summary][details] | Exact nonempty names group members within one ordinary tree. Insertion/name changes close the given member; adding open closes another member. Owned ordinary/fragment construction proves a closed notification domain from actual owners, not scripting mode alone. | [Fragment tests][fragment]: `owned_fragment_details_open_peers_templates_and_real_clones_complete`, `owned_fragment_details_managed_name_open_and_insertion_use_distinct_rules`, `owned_fragment_details_foreign_capability_and_cancel_refuse_before_take`. [Public tests][public]: `owned_fragment_details_publishes_canonical_exclusivity_without_changing_capture`. [Semantic tests](../../surgeist-html-semantics/tests/semantics_details.rs): `first_direct_summary_after_text_and_other_children_wins_without_later_descendant_facts`, `groups_and_summary_predicates_do_not_cross_shadow_or_template_host_associations`. No live toggle dispatch or task tracker is claimed. |
| [Form controls and labels][forms] | Qualified established form-owner/current flags and ordinary-tree relationships, rather than default attributes used as current state. Missing influencing facts remain missing. | [Control semantic tests](../../surgeist-html-semantics/tests/semantics_controls.rs): `markup_checked_selected_do_not_override_established_current_flags`, `actual_owner_pointer_can_be_read_with_unavailable_ancestry_but_proved_cross_tree_is_rejected`. [Label tests](../../surgeist-html-semantics/tests/semantics_labels.rs): `explicit_labels_before_and_after_control_preserve_tree_order`, `shadow_and_template_hosts_do_not_supply_ordinary_labels`. [Disabledness tests](../../surgeist-html-semantics/tests/semantics.rs): `first_legend_exception_uses_first_legend_element_not_first_child`. |
| [Table elements][tableelements], [select][select] and [meter/progress][forms] | Capture-qualified direct table collections, supplied current option selectedness and source-dependent range defaults. They do not mutate a live table/control or execute validation UI. | [Table collection tests](../../surgeist-html-semantics/tests/semantics_table_collections.rs): `table_rows_exclude_wrapped_sections_rows_and_nested_tables_without_reading_their_children`. [Select tests](../../surgeist-html-semantics/tests/semantics_select.rs): `selectedness_is_current_state_and_does_not_use_authored_selected_or_defaults`. [Range semantic tests](../../surgeist-html-semantics/tests/semantics_ranges.rs): `meter_missing_defaults_are_one_complete_six_point_state`, `progress_missing_value_and_present_invalid_are_distinct`. |
| HTML §3.2.6 language/directionality and global inheritance; [language/DOM sources](../../../references/SOURCE-EDITIONS.md#html-language-and-directionality-sources), [inertness sources](../../../references/SOURCE-EDITIONS.md#html-inertness-and-dom-flat-tree-facts) and [editing delegation](../../../references/SOURCE-EDITIONS.md#bounded-html-editability-and-spellcheck-readback) | Exact language/directionality and editable/translation/spellcheck/heading queries use qualified producer facts. Ordinary ancestry, shadow hosts and flat-tree assignments differ where required. | [Language tests](../../surgeist-html-semantics/tests/semantics_language.rs): `language_shadow_inherits_host_for_text_and_preserves_original_source`. [Inheritance tests](../../surgeist-html-semantics/tests/semantics_inheritance.rs): `unknown_parent_is_missing_except_independent_false_or_local_true_proofs`. [Inert tests](../../surgeist-html-semantics/tests/semantics_inert.rs): `assigned_node_inherits_from_actual_flat_slot_instead_of_ordinary_parent`. Supplied modal, style or runtime facts are not produced by these queries. |

The [semantic owner](../../surgeist-html-semantics/src/semantics.rs) and [language inventory](../../surgeist-html/INVENTORY.md)
identify further implemented query/value families and defining clauses. The
rows above select executable outcomes; they neither certify every query branch
nor infer live operations from the inventory's recognized names.

## Retained corpus bindings and differences

The [bundled html5lib fixtures](../tests/fixtures/html5lib-f994590f/tree-construction/README.md)
retain revision `f994590f528ac8b6073665791ddb1ed85c66dfb2`, complete files,
format metadata and [MIT attribution](../NOTICE.md#html5lib-tests-fixtures).
`all_complete_retained_files_decode_with_stable_ordinals_and_raw_sections`
checks all **169** cases as fixture-format evidence. It does not execute all
contexts or scripting variants. Exactly **167 distinct file/ordinal bindings**
run as ordinary documents with Disabled scripting; dual encodings and feeds
are additional executions of those bindings.

| Retained file | Selected ordinary ordinals | Additional selected evidence |
| --- | --- | --- |
| adoption02.dat | 1 | Independent tree; selected hand-derived sink tests above cover separate adoption/foster operations. |
| namespace-sensitivity.dat | 1 | Expanded namespace/name structure. |
| quirks01.dat | 1–4 | Literal document-mode selection. |
| tests14.dat | 1–7 | Independent structural expectations. |
| tests24.dat | 1–8 | Independent structural expectations. |
| tables01.dat | 1–19 | Independent structural expectations. Cases 1,4,6,19 additionally have 3/7/14/5 source-derived typed records, both encodings and every cut; the remaining historical error sections are not certified. |
| comments01.dat | 1–16 | Both encodings, every cut, independent 27-record typed chronology plus separate Unicode literals. |
| template.dat | 1–108, 110–112 | All 111 ordinary bindings in both encodings; 81, 94, 100, 108, 110–112 also contribute the 670 progressive runs. |

Template/109 is a separate, actual captured-template fragment in both encodings.
The parser does not replace its context with an ordinary request. The corpus's
`Both` scripting metadata is retained, while this ordinary selection explicitly
executes Disabled. Enabled corpus replay and private fragment feeding are not
covered by this selection.

Adoption02/2 remains outside the selected structural replay bindings: its input
and expected format are decoded, but that does not credit an actual parser
comparison. This selection limit is not a claim that the input is unsupported.

The historical corpus says HTML parsing cannot emit processing instructions;
HTML877d and the independent PI literals above establish the selected contrary
behavior. Corpus doctype dumps collapse absent/empty identifiers: native literal
tests check those distinctions separately. Sorted corpus attribute dumps use an
explicit comparison permutation; native order, namespace and prefix remain
unchanged. Template/shadow contents remain separate owned fragment trees.
Historical select-mode/error wording is not substituted for newer normative
rules. Apart from the explicitly selected typed expectations, retained
`#errors`/`#new-errors` sections remain metadata, not verified diagnostic
conformance. No namespace, source-range or diagnostic difference is erased to
obtain a structural match.

## Ownership, comparisons and measured scope

[Public parsing tests][public] such as
`public_failure_keeps_first_stop_snapshot_separate_from_cleanup_and_cancel` and
`retained_source_and_capture_aliases_do_not_alias_terminal_parser_backing`
exercise real terminal evidence and cleanup. The [request tests](../tests/parser/requests.rs)
name all nineteen required limits. The [completed observer](../src/construction/native_view.rs)
and [conformance adapter][conformance] borrow qualified native storage and admit
consumer capacity/work through the actual parser controller. Selected source
associations, history, reverse reads and chronology checks do not establish
complete provenance or whole-parser accounting for every construction.

| Selected owner requirement | Actual expected observation | Executable evidence and qualification |
| --- | --- | --- |
| Within-Complete numeric-reference cancellation and terminal ownership | Decimal/hexadecimal 8192-digit inputs in both encodings reach the 1024th consumed digit before the existing owner-qualified cancellation hook stops the next charged read. Original source range, request ownership and the earlier committed NULL-reference diagnostic remain retained. | [Tree tests][tree]: `decimal_and_hex_complete_cancel_at_exact_original_digit_frontier`. The test checks actual hook admission, sticky first stop, inert step/feed/seal retries, rejected completed take and same-controller release. It introduces no new production polling mechanism and supplies no cancellation-preemption guarantee. |
| Finite work admission during real Complete numeric-reference scanning | Decimal/hexadecimal 90000-digit inputs in both encodings cross a request-local work limit after entering the reference and before finishing it. The existing per-digit StateStep, peek/read, CopyByte and accumulator-write charges provide a conservative floor, even after numeric saturation. | [Tree tests][tree]: `decimal_and_hex_complete_digit_action_floor_forces_finite_work_refusal`. This is an implementation/accounting expectation, not a normative HTML fee or an output-derived exact cost. It checks uncancelled WorkCredits refusal, retained committed diagnostics, unchanged first-stop/work snapshots and zero reclaimable capacity after cleanup. No global budget, fee or universal 100M-fit claim follows. |
| Empty-source implicit construction and retained depth, distinct from open stack | In both encodings, the literal empty ordinary tree has Document/html/head/body: four admitted nodes, three edges and two parent links. Exact caps succeed; one-short and zero caps stop in their own resource domain. For two div/Text siblings, retained text depth is four while open-elements high-water is three. | [Public construction-limit tests](../tests/parser/parse/construction_limits.rs): `empty_source_implied_document_has_exact_four_node_admission`, `empty_source_implied_document_has_exact_three_edge_admission`, `empty_source_implied_document_has_exact_two_parent_link_depth`, `empty_source_zero_construction_caps_refuse_required_implicit_work`, `ordinary_text_depth_exact_and_one_short_are_independent_of_open_elements`. The empty outline/Quirks expectation follows the [initial][initial]/before-html/before-head/[after-head][afterhead] rules; cap definitions belong to [settings](../src/settings.rs). Failures retain admitted original source, exact typed stop and stable at-stop/cleanup snapshots without a partial successful document. Source/committed diagnostics and cumulative node/work observations are not asserted refundable. |

The retained three-engine release checkpoint compares unchanged html5ever,
safe-only ingestion and the completed Surgeist producer: **360 literal-checked
parses**, twenty scalar UTF-8 ordinary Disabled whole-source inputs, N0/1/8/32,
three repetitions and seventeen completed finite commands. Its result record
SHA256 is `fc9cadae0c91bde1c7ffc8eabf7b99a1aea8f645ad3fdeb4fa69285870d2da66`.
It records toolchain, sinks, producer/source identities, separate caller parse
and observer/evidence/protocol/drop clocks, and process RSS. Surgeist parsing
includes source/control/native/D1 freezing and report ownership; upstream arena
parsing does not provide equivalent D1 or resource accounting. Their times and
ledgers therefore do not demonstrate equal work or identify one causal cost.
The influencing HTML source was the earlier `40406368`-qualified checkpoint,
not this matrix's later production source.

The separate current-workload checkpoint retains 40 literal variants across
UTF-8/UTF-16 and N0/1/8/32, actual credited work/capacity snapshots and separately
timed observation/publication phases; its result record SHA256 is
`caa4df8db8fc9510ac67c753fcda60e17ddf1a81958043ef8ea2ce9e44d4000c`.
These historical measurements do not cover larger/adversarial scaling,
comparative captured-fragment/feed execution, every provider, allocator totals
or all platforms/features. They supply neither a universal 100M-work fit nor
an RSS/latency threshold. Independent language oracles remain necessary even
when producers agree.

## Capability exclusions

Public document execution selects a fresh ordinary document with Disabled
scripting. Captured-document, hosted, srcdoc and mode-fixed document execution
remain typed unsupported profiles. Qualified fragments support their selected
Inert/Disabled construction branches; missing influential capture, registry,
template, shadow or Image creation-environment facts refuse rather than become
guessed defaults. Required custom reactions remain unsupported.

Neither Unicode entry decodes bytes, executes JavaScript, fetches network
resources, runs browser events, installs live shadow roots or supplies a live
DOM. Color construction does not supply missing CSS context, palette, profile,
scheme or device facts; an influential unavailable fact remains a typed stop.
Private ordinary source exposure supplies no public streaming API
or incremental fragment feed. These owner/capability boundaries do not select
an alternative HTML dialect. Additional fuzz adoption/campaigns are outside
this coverage selection; finite fuzz outcomes would not replace its independent
expectations or certify all HTML behavior.

[preprocessing]: ../../../references/html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#preprocessing-the-input-stream
[tokenization]: ../../../references/html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#tokenization
[initial]: ../../../references/html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#the-initial-insertion-mode
[inhead]: ../../../references/html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#parsing-main-inhead
[inbody]: ../../../references/html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#parsing-main-inbody
[afterhead]: ../../../references/html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#the-after-head-insertion-mode
[references]: ../../../references/html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#character-reference-state
[numericreferences]: ../../../references/html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#numeric-character-reference-state
[names]: ../../../references/html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#named-character-references
[comments]: ../../../references/html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#comment-start-state
[pi]: ../../../references/html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#processing-instruction-open-state
[intable]: ../../../references/html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#parsing-main-intable
[intabletext]: ../../../references/html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#parsing-main-intabletext
[incolgroup]: ../../../references/html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#parsing-main-incolgroup
[intbody]: ../../../references/html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#parsing-main-intbody
[foreign]: ../../../references/html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#parsing-main-inforeign
[intemplate]: ../../../references/html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#parsing-main-intemplate
[fragments]: ../../../references/html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#parsing-html-fragments
[template]: ../../../references/html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-template-element
[scripts]: ../../../references/html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#script-processing-model
[details]: ../../../references/html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-details-element
[domtrees]: ../../../references/html-language-foundations--877d6146487f--b55e73c31978.md#dom-trees
[microsyntaxes]: ../../../references/html-language-foundations--877d6146487f--b55e73c31978.md#common-microsyntaxes
[htmlmedia]: ../../../references/html-language-foundations--877d6146487f--b55e73c31978.md#mq
[microdata]: ../../../references/html-microdata--877d6146487f.md#the-microdata-model
[nonce]: ../../../references/html-nonce--877d6146487f.md#nonce-attributes
[baseurls]: ../../../references/html-language-foundations--877d6146487f--b55e73c31978.md#document-base-urls
[linktypes]: ../../../references/html-language-elements-text-links--877d6146487f--8833a85fd242.md#linkTypes
[forms]: ../../../references/html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md
[input]: ../../../references/html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-input-element
[colorinput]: ../../../references/html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#color-state-(type=color)
[select]: ../../../references/html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-select-element
[textarea]: ../../../references/html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-textarea-element
[output]: ../../../references/html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-output-element
[selectedcontent]: ../../../references/html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-selectedcontent-element
[tableelements]: ../../../references/html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-table-element
[tree]: ../src/parser/tree_builder/tests.rs
[conformance]: ../src/parser/tree_builder/conformance.rs
[fragment]: ../src/parser/tree_builder/context.rs
[public]: ../tests/parser/parse/reports.rs
[construction]: ../tests/construction.rs
[interpretation]: ../tests/interpretation.rs
[inputtests]: ../tests/parser/construction/input/rules.rs
