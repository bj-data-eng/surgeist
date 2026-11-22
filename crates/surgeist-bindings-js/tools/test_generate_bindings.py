#!/usr/bin/env python3
"""Functional tests for strict declaration validation and typed value lowering.

Run with Python -B. --rustc additionally compiles and executes a temporary,
standalone consumer of the generated typed values; it uses no
Cargo, downloads, or the repository target directory. Run through pisct run.
"""

import copy
import json
import re
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

import generate_bindings as g


RUSTC = "--rustc" in sys.argv
if RUSTC:
    sys.argv.remove("--rustc")


class DescriptionTests(unittest.TestCase):
    def test_reflected_integer_default_preserves_the_pinned_literal(self):
        self.assertEqual(
            g.attributes([{"name": "ReflectDefault", "value": 1}]),
            "[ReflectDefault=1] ",
        )

    def test_reflected_integer_defaults_serialize_exact_signed_long_literals(self):
        for value, expected in [
            (-2147483648, "[ReflectDefault=-2147483648] "),
            (0, "[ReflectDefault=0] "),
            (2147483647, "[ReflectDefault=2147483647] "),
        ]:
            with self.subTest(value=value):
                self.assertEqual(
                    g.attributes([{"name": "ReflectDefault", "value": value}]),
                    expected,
                )

    def test_reflected_integer_defaults_refuse_missing_mistyped_and_unrepresentable_values(self):
        invalid = [{"name": "ReflectDefault"}]
        invalid.extend(
            {"name": "ReflectDefault", "value": value}
            for value in [None, True, False, "1", 1.0, [], {}, -2147483649, 4294967296]
        )
        for annotation in invalid:
            with self.subTest(annotation=annotation):
                with self.assertRaises(g.DescriptionError):
                    g.attributes([annotation])
        with self.assertRaisesRegex(g.DescriptionError, "duplicate"):
            g.attributes([
                {"name": "ReflectDefault", "value": 1},
                {"name": "ReflectDefault", "value": 1},
            ])

    def test_unsigned_long_reflection_renders_an_independent_default_and_clamp(self):
        member = {
            "id": "Fixture.span", "kind": "attribute", "name": "span", "readonly": False,
            "type": {"kind": "named", "name": "unsigned long"},
            "attributes": [
                {"name": "Reflect"},
                {"name": "ReflectDefault", "value": 4294967295},
                {"name": "ReflectRange", "value": [1, 1000]},
            ],
        }
        self.assertEqual(
            g.render_member(member, {}),
            "[Reflect, ReflectDefault=4294967295, ReflectRange=(1, 1000)] "
            "attribute unsigned long span;",
        )

    def test_numeric_reflection_annotations_validate_their_member_context(self):
        for value_type, annotations, reason in [
            ("long", [{"name": "ReflectDefault", "value": 2147483648}], "reflection default"),
            ("unsigned long", [{"name": "ReflectDefault", "value": -1}], "reflection default"),
            ("DOMString", [{"name": "ReflectDefault", "value": 1}], "ReflectDefault requires"),
            ("long", [{"name": "ReflectRange", "value": [0, 1]}], "ReflectRange requires"),
            ("unsigned long", [{"name": "ReflectRange", "value": [-1, 1]}], "reflection range bound"),
            ("unsigned long", [{"name": "ReflectRange", "value": [2, 1]}], "ascending"),
            ("unsigned long", [{"name": "ReflectRange", "value": [0, 4294967296]}], "reflection range bound"),
        ]:
            with self.subTest(value_type=value_type, annotations=annotations):
                member = {
                    "id": "Fixture.value", "kind": "attribute", "name": "value", "readonly": False,
                    "type": {"kind": "named", "name": value_type},
                    "attributes": [{"name": "Reflect"}, *annotations],
                }
                with self.assertRaisesRegex(g.DescriptionError, reason):
                    g.render_member(member, {})

    def test_positive_double_reflection_requires_a_finite_decimal_default(self):
        default = {"kind": "decimal", "literal": "1.0"}
        member = {
            "id": "Fixture.max", "kind": "attribute", "name": "max", "readonly": False,
            "type": {"kind": "named", "name": "double"},
            "attributes": [{"name": "ReflectPositive"}, {"name": "ReflectDefault", "value": default}],
        }
        self.assertEqual(
            g.render_member(member, {}),
            "[ReflectPositive, ReflectDefault=1.0] attribute double max;",
        )
        for value_type in ["unsigned long", "DOMString", "unrestricted double"]:
            with self.subTest(value_type=value_type):
                member["type"]["name"] = value_type
                with self.assertRaises(g.DescriptionError):
                    g.render_member(member, {})
        member["type"]["name"] = "double"
        for literal in ["1", "1.0 2", "NaN", "1e999", 1.0]:
            with self.subTest(literal=literal):
                default["literal"] = literal
                with self.assertRaises(g.DescriptionError):
                    g.render_member(member, {})
        member["attributes"][-1]["value"] = 1
        with self.assertRaises(g.DescriptionError):
            g.render_member(member, {})

    def test_range_indicators_preserve_the_pinned_numeric_declarations(self):
        mappings = self.data["access_mappings"]
        for interface, properties in [
            ("HTMLProgressElement", ["value", "max", "position"]),
            ("HTMLMeterElement", ["value", "min", "max", "low", "high", "optimum"]),
        ]:
            with self.subTest(interface=interface):
                definition = self.definition(interface)
                self.assertEqual(definition["inherits"], "HTMLElement")
                ctor, *members = definition["members"]
                self.assertEqual(ctor["kind"], "constructor")
                self.assertEqual([member["name"] for member in members], [*properties, "labels"])
                for member in members[:-1]:
                    self.assertEqual(member["type"], {"kind": "named", "name": "double"})
                    self.assertFalse(mappings[member["id"] + "/get"]["requirements"])
                    readonly = member["name"] == "position"
                    self.assertEqual(member["readonly"], readonly)
                    if readonly:
                        self.assertNotIn(member["id"] + "/set", mappings)
                    else:
                        self.assertFalse(mappings[member["id"] + "/set"]["requirements"])
                labels = mappings[members[-1]["id"] + "/get"]
                self.assertEqual(labels["result_view"], "live")
                self.assertTrue(labels.get("requirements") or labels.get("requirement"))
        maximum = self.member("HTMLProgressElement.max")
        self.assertEqual(maximum["attributes"], [
            {"name": "CEReactions"},
            {"name": "ReflectPositive"},
            {"name": "ReflectDefault", "value": {"kind": "decimal", "literal": "1.0"}},
        ])
        maximum["attributes"][-1]["value"]["literal"] = "2.0"
        self.rejected("source declaration mismatch: HTMLProgressElement.source")

    def test_canvas_preserves_complete_idl_and_unavailable_native_providers(self):
        definition = self.definition("HTMLCanvasElement")
        self.assertEqual(definition["inherits"], "HTMLElement")
        ctor, width, height, *operations = definition["members"]
        self.assertEqual(ctor["kind"], "constructor")
        self.assertEqual([width["name"], height["name"]], ["width", "height"])
        mappings = self.data["access_mappings"]
        for member in [width, height]:
            self.assertEqual(member["type"], {"kind": "named", "name": "unsigned long"})
            self.assertEqual(member["attributes"], [{"name": "CEReactions"}])
            self.assertFalse(member["readonly"])
            for access in ["get", "set"]:
                self.assertFalse(mappings[member["id"] + "/" + access]["requirements"])
        self.assertEqual([member["name"] for member in operations],
                         ["getContext", "toDataURL", "toBlob", "transferControlToOffscreen"])
        for member in operations:
            self.assertEqual(mappings[member["id"] + "/call"]["kind"], "host")

        context = self.definition("RenderingContext")
        self.assertEqual(context["type"]["kind"], "union")
        self.assertEqual([member["name"] for member in context["type"]["members"]],
                         ["CanvasRenderingContext2D", "ImageBitmapRenderingContext",
                          "WebGLRenderingContext", "WebGL2RenderingContext", "GPUCanvasContext"])
        callback = self.definition("BlobCallback")
        self.assertEqual(callback["kind"], "callback")
        self.assertEqual(callback["type"], {"kind": "named", "name": "undefined"})
        self.assertEqual(callback["arguments"], [{
            "name": "blob", "presence": "required",
            "type": {"kind": "nullable", "inner": {"kind": "named", "name": "Blob"}},
        }])
        callback["arguments"][0]["type"] = {"kind": "named", "name": "Blob"}
        self.rejected("source declaration mismatch: HTMLCanvasElement.source")

    def test_media_retains_unloaded_observations_and_provider_boundaries(self):
        media = self.definition("HTMLMediaElement")
        self.assertEqual(media["inherits"], "HTMLElement")
        members = {member["name"]: member for member in media["members"]}
        mappings = self.data["access_mappings"]
        for name, value_type in [
            ("currentSrc", "USVString"),
            ("networkState", "unsigned short"),
            ("readyState", "unsigned short"),
            ("paused", "boolean"),
        ]:
            with self.subTest(member=name):
                member = members[name]
                self.assertEqual(member["type"], {"kind": "named", "name": value_type})
                self.assertTrue(member["readonly"])
                getter = mappings[member["id"] + "/get"]
                self.assertFalse(getter["requirements"])
                self.assertNotIn(member["id"] + "/set", mappings)
        for name, value in [
            ("NETWORK_EMPTY", 0), ("NETWORK_IDLE", 1), ("NETWORK_LOADING", 2), ("NETWORK_NO_SOURCE", 3),
            ("HAVE_NOTHING", 0), ("HAVE_METADATA", 1), ("HAVE_CURRENT_DATA", 2),
            ("HAVE_FUTURE_DATA", 3), ("HAVE_ENOUGH_DATA", 4),
        ]:
            self.assertEqual(members[name]["kind"], "constant")
            self.assertEqual(members[name]["value"], value)
        self.assertEqual(self.definition("CanPlayTypeResult")["values"], ["", "maybe", "probably"])
        provider = self.definition("MediaProvider")
        self.assertEqual(provider["type"]["kind"], "union")
        self.assertEqual([member["name"] for member in provider["type"]["members"]],
                         ["MediaStream", "MediaSource", "Blob"])
        for name in ["error", "src", "srcObject", "buffered", "currentTime", "duration", "audioTracks"]:
            self.assertEqual(mappings[members[name]["id"] + "/get"]["kind"], "host")
        for name in ["load", "play", "pause", "addTextTrack"]:
            self.assertEqual(mappings[members[name]["id"] + "/call"]["kind"], "host")
        members["NETWORK_EMPTY"]["value"] = 9
        self.rejected("source declaration mismatch: HTMLMediaElement.source")

    def test_unsigned_numeric_reflection_requires_reflect_on_the_same_member(self):
        for annotation in [
            {"name": "ReflectDefault", "value": 1},
            {"name": "ReflectRange", "value": [1, 1000]},
        ]:
            with self.subTest(annotation=annotation):
                member = {
                    "id": "Fixture.span", "kind": "attribute", "name": "span", "readonly": False,
                    "type": {"kind": "named", "name": "unsigned long"},
                    "attributes": [annotation],
                }
                with self.assertRaisesRegex(g.DescriptionError, annotation["name"] + " requires"):
                    g.render_member(member, {})

    def test_numeric_reflection_modes_keep_their_types_and_defaults(self):
        member = {
            "id": "Fixture.extent", "kind": "attribute", "name": "extent", "readonly": False,
            "type": {"kind": "named", "name": "unsigned long"},
            "attributes": [{"name": "ReflectPositiveWithFallback"},
                           {"name": "ReflectDefault", "value": 20}],
        }
        self.assertEqual(
            g.render_member(member, {}),
            "[ReflectPositiveWithFallback, ReflectDefault=20] attribute unsigned long extent;",
        )
        member["type"]["name"] = "long"
        with self.assertRaisesRegex(g.DescriptionError, "ReflectPositiveWithFallback requires"):
            g.render_member(member, {})
        member["attributes"] = [{"name": "ReflectNonNegative", "value": "max-length"}]
        self.assertEqual(
            g.render_member(member, {}),
            '[ReflectNonNegative="max-length"] attribute long extent;',
        )
        member["attributes"].append({"name": "Reflect"})
        with self.assertRaisesRegex(g.DescriptionError, "only one primary reflection"):
            g.render_member(member, {})
        member["attributes"].pop()
        member["type"]["name"] = "unsigned long"
        with self.assertRaisesRegex(g.DescriptionError, "ReflectNonNegative requires"):
            g.render_member(member, {})

    def test_positive_fallback_does_not_admit_the_reflect_only_range(self):
        member = {
            "id": "Fixture.span", "kind": "attribute", "name": "span", "readonly": False,
            "type": {"kind": "named", "name": "unsigned long"},
            "attributes": [{"name": "ReflectPositiveWithFallback"},
                           {"name": "ReflectRange", "value": [1, 1000]}],
        }
        with self.assertRaisesRegex(g.DescriptionError, "ReflectRange requires"):
            g.render_member(member, {})

    def test_retained_ol_start_default_and_its_reflection_context_cannot_change(self):
        # Source correspondence validates altered declarations through the real generator.
        for change in ["altered", "omitted", "without_reflect", "string_type"]:
            with self.subTest(change=change):
                self.setUp()
                member = self.member("HTMLOListElement.start")
                if change == "altered":
                    next(a for a in member["attributes"] if a["name"] == "ReflectDefault")["value"] = 2
                elif change == "omitted":
                    member["attributes"] = [a for a in member["attributes"] if a["name"] != "ReflectDefault"]
                elif change == "without_reflect":
                    member["attributes"] = [a for a in member["attributes"] if a["name"] != "Reflect"]
                else:
                    member["type"] = {"kind": "named", "name": "DOMString"}
                with self.assertRaises(g.DescriptionError):
                    g.generate(self.data)

    def test_named_reflection_preserves_the_source_attribute(self):
        self.assertEqual(g.attributes([{"name": "Reflect", "value": "for"}]), '[Reflect="for"] ')
        with self.assertRaisesRegex(g.DescriptionError, "invalid reflected attribute name"):
            g.attributes([{"name": "Reflect", "value": 'invalid"name'}])

    def test_url_reflection_preserves_its_name_and_requires_usvstring(self):
        member = {
            "id": "Fixture.source", "kind": "attribute", "name": "source", "readonly": False,
            "type": {"kind": "named", "name": "USVString"},
            "attributes": [{"name": "ReflectURL", "value": "src"}],
        }
        self.assertEqual(g.render_member(member, {}),
                         '[ReflectURL="src"] attribute USVString source;')
        member["type"]["name"] = "DOMString"
        with self.assertRaisesRegex(g.DescriptionError, "ReflectURL requires"):
            g.render_member(member, {})
        member["type"]["name"] = "USVString"
        member["attributes"].append({"name": "ReflectSetter"})
        with self.assertRaisesRegex(g.DescriptionError, "only one primary reflection"):
            g.render_member(member, {})

    @classmethod
    def setUpClass(cls):
        cls.original = g.load()

    def setUp(self):
        self.data = copy.deepcopy(self.original)

    def member(self, identifier):
        return next(m for d in self.data["declarations"] for m in d.get("members", []) if m["id"] == identifier)

    def definition(self, identifier):
        return next(d for d in self.data["declarations"] if d["id"] == identifier)


    def rejected(self, message):
        with self.assertRaisesRegex(g.DescriptionError, message):
            g.generate(self.data)

    def test_actual_retained_blocks_and_generated_bytes_correspond(self):
        # This is an explicit generated-artifact contract. Source token equality
        # is the oracle, not a copy of today's generated Rust as a semantic test.
        for path, output in g.generate(self.data).items():
            self.assertEqual(output, path.read_text(encoding="utf-8"))

    def test_option_factory_preserves_omitted_value_without_a_default(self):
        option = self.definition("HTMLOptionElement")
        factory = next(a["value"] for a in option["attributes"] if a["name"] == "LegacyFactoryFunction")
        lowered = [
            entry for entry in g.lower_arguments(self.data, g.lower_values(self.data))
            if entry["id"].startswith("HTMLOptionElement.Option/")
        ]
        self.assertEqual([(entry["index"], entry["argument"]["name"],
                           entry["argument"]["presence"], entry["info"][0],
                           entry["argument"].get("default")) for entry in lowered], [
            (0, "text", "optional", "DomString", {"kind": "string", "value": ""}),
            (1, "value", "optional", "DomString", None),
            (2, "defaultSelected", "optional", "bool", {"kind": "boolean", "value": False}),
            (3, "selected", "optional", "bool", {"kind": "boolean", "value": False}),
        ])
        value = next(argument for argument in factory["arguments"] if argument["name"] == "value")
        value["default"] = {"kind": "string", "value": ""}
        self.rejected("source declaration mismatch: HTMLOptionElement.source")

    def test_option_factory_requires_structured_arguments(self):
        option = self.definition("HTMLOptionElement")
        factory = next(a["value"] for a in option["attributes"] if a["name"] == "LegacyFactoryFunction")
        factory["arguments"] = {"text": "DOMString"}
        self.rejected("arguments must be an array")

    def test_select_indexed_setter_and_option_factory_require_distinct_mappings(self):
        for key in ("HTMLSelectElement.operation:0/index-set", "HTMLOptionElement.Option/factory"):
            with self.subTest(key=key):
                self.setUp()
                del self.data["access_mappings"][key]
                self.rejected("indexed setter and legacy factory")

    def test_unnamed_select_indexed_setter_cannot_be_mapped_as_callable(self):
        mappings = self.data["access_mappings"]
        mappings["HTMLSelectElement.operation:0/call"] = mappings.pop(
            "HTMLSelectElement.operation:0/index-set"
        )
        self.rejected("indexed setter and legacy factory")

    def test_node_filter_legacy_constants_cannot_be_dropped_from_pinned_idl(self):
        # The exact declaration is the artifact contract: even obsolete SHOW
        # constants remain part of the pinned NodeFilter interface object.
        for name in ("SHOW_ENTITY_REFERENCE", "SHOW_ENTITY", "SHOW_NOTATION"):
            with self.subTest(name=name):
                self.setUp()
                definition = self.definition("NodeFilter")
                definition["members"] = [m for m in definition["members"] if m["name"] != name]
                self.rejected("source declaration mismatch: NodeFilter.source")

    def test_node_filter_invocation_requires_its_exact_guest_operation(self):
        key = "NodeFilter.acceptNode:0/call"
        self.data["access_mappings"][key] = {
            "kind": "native", "adapter": "nodefilter.acceptnode.call",
            "producers": [], "requirements": [{
                "owner": "surgeist-runtime", "contract": "fixture",
                "source": self.data["semantics"]["argument_conversion"]}],
        }
        self.rejected("callback-interface operation requires the guest invocation protocol")
        self.setUp()
        self.data["access_mappings"][key]["operation"] = "handleEvent"
        self.rejected("callback-interface operation requires the guest invocation protocol")
        self.setUp()
        self.data["access_mappings"][key]["interface"] = "EventListener"
        self.rejected("wrong callback interface")

    def test_tree_walker_source_preserves_exposure_identity_and_live_cursor(self):
        # The retained standard block is the exact artifact contract. Reject
        # changes that would silently alter installation or native conversion.
        for change in ("exposure", "root_identity", "readonly_cursor", "nullable_cursor",
                       "navigation_argument", "constructible"):
            with self.subTest(change=change):
                self.setUp()
                if change == "exposure":
                    self.definition("TreeWalker")["attributes"][0]["value"] = "*"
                elif change == "root_identity":
                    self.member("TreeWalker.root")["attributes"] = []
                elif change == "readonly_cursor":
                    self.member("TreeWalker.currentNode")["readonly"] = True
                elif change == "nullable_cursor":
                    member = self.member("TreeWalker.currentNode")
                    member["type"] = {"kind": "nullable", "inner": member["type"]}
                elif change == "navigation_argument":
                    self.member("TreeWalker.parentNode:0")["arguments"] = [
                        {"name": "node", "type": {"kind": "named", "name": "Node"},
                         "presence": "required"}]
                else:
                    self.definition("TreeWalker")["members"].append({
                        "id": "TreeWalker.constructor:0", "kind": "constructor",
                        "attributes": [], "arguments": []})
                self.rejected("source declaration mismatch: TreeWalker.source")

    def test_tree_walker_factory_preserves_native_brand_and_callback_defaults(self):
        factory = self.member("Document.createTreeWalker:0")
        self.assertEqual(factory["attributes"], [{"name": "NewObject"}])
        self.assertEqual(factory["type"], {"kind": "named", "name": "TreeWalker"})
        root, mask, callback = factory["arguments"]
        self.assertEqual(root, {"name": "root", "type": {"kind": "named", "name": "Node"},
                                "presence": "required"})
        self.assertEqual(mask["default"], {"kind": "integer", "value": 4294967295})
        self.assertEqual(callback, {"name": "filter", "type": {"kind": "nullable", "inner": {
            "kind": "named", "name": "NodeFilter"}}, "presence": "optional", "default": {"kind": "null"}})
        arguments = {a["id"]: a for a in g.lower_arguments(self.data, g.lower_values(self.data))}
        self.assertEqual(arguments["Document.createTreeWalker:0/whatToShow"]["index"], 1)
        self.assertNotIn("Document.createTreeWalker:0/root", arguments)
        self.assertNotIn("Document.createTreeWalker:0/filter", arguments)

    def test_namespace_source_qualifies_exposure_and_cssom_string_profile(self):
        self.definition("CSS")["attributes"][0]["value"] = "*"
        self.rejected("only Window namespace exposure is selected")
        self.setUp()
        self.definition("CSSOMString")["type"]["name"] = "USVString"
        self.rejected("source declaration mismatch")
        self.setUp()
        self.member("CSS.escape:0")["arguments"][0]["presence"] = "optional"
        self.rejected("only regular fixed-arity namespace operations are selected")
        self.setUp()
        del self.member("CSS.escape:0")["arguments"]
        self.rejected("only regular fixed-arity namespace operations are selected")

    def test_dom_exception_requires_pinned_signature_and_names_table(self):
        self.member("DOMException.constructor:0")["arguments"][1]["default"]["value"] = ""
        self.rejected("source declaration mismatch")
        self.setUp()
        self.member("DOMException.DATA_CLONE_ERR")["value"] = 0
        self.rejected("source declaration mismatch")
        self.setUp()
        source = next(s for s in self.data["sources"] if s["id"] == "DOMException.source")
        del source["names_table_sha256"]
        self.rejected("DOMException names table is missing")
        source["names_table_sha256"] = "0" * 64
        self.rejected("source digest mismatch: DOMException.source")

    def test_changed_standard_member_type_is_refused(self):
        self.member("Node.nodeName")["type"]["name"] = "USVString"
        self.rejected("source declaration mismatch")

    def test_changed_default_is_refused_even_when_native_mapping_is_unchanged(self):
        self.member("Text.constructor:0")["arguments"][0]["default"]["value"] = "changed"
        self.rejected("source declaration mismatch")

    def test_source_identity_mismatch_is_not_accepted_as_new_standard(self):
        self.data["sources"][0]["sha256"] = "0" * 64
        self.rejected("source digest mismatch")

    def test_unknown_extended_attribute_is_rejected(self):
        self.member("Node.childNodes")["attributes"].append({"name": "LiveObject"})
        self.rejected("unknown/duplicate extended attribute")

    def test_type_annotation_cannot_be_moved_to_attribute_scope(self):
        m = self.member("CharacterData.data")
        m["attributes"] = m["type"].pop("attributes")
        self.rejected("wrong declaration scope")

    def test_unresolved_type_and_wrong_includes_kind_are_refused(self):
        self.member("Node.childNodes")["type"]["name"] = "MissingList"
        self.rejected("unresolved type")
        self.setUp()
        self.data["includes"][0]["mixin"] = "Element"
        self.rejected("includes source is not a mixin")

    def test_external_element_mixin_inclusions_match_the_source(self):
        mixin = "HTMLOrSVGOrMathMLElement"
        definitions, names = g.validate(self.data)
        actual = {edge["interface"] for edge in self.data["includes"] if edge["mixin"] == mixin}
        self.assertEqual(actual, {"HTMLElement", "SVGElement", "MathMLElement"})
        for interface in ("SVGElement", "MathMLElement"):
            with self.subTest(interface=interface):
                self.assertEqual(names[interface], "external_interface")
                self.assertNotIn(interface, definitions)
                edge = next(record for record in self.data["includes"] if record["interface"] == interface)
                edge["mixin"] = "GlobalEventHandlers"
                self.rejected(f"source declaration mismatch: {interface}.mixin.source")
                edge["mixin"] = mixin

    def test_external_mixin_target_still_requires_interface_kind(self):
        external = next(record for record in self.data["externals"] if record["name"] == "SVGElement")
        external["kind"] = "dictionary"
        self.rejected("includes target is not an interface")

    def test_duplicate_relation_and_forged_overload_identity_are_refused(self):
        edge = copy.deepcopy(self.data["includes"][0])
        edge["id"] = "duplicate"
        self.data["includes"].append(edge)
        self.rejected("duplicate includes relation")
        self.setUp()
        self.member("Document.html.open:1")["id"] = "Document.html.open:9"
        self.rejected("member/overload ID")

    def test_inheritance_cycle_and_missing_primary_definition_are_rejected(self):
        self.definition("Node")["inherits"] = "Document"
        self.rejected("inheritance cycle")
        self.setUp()
        self.definition("Document.html")["name"] = "NotDocument"
        self.rejected("partial definition lacks its primary")

    def test_every_accessor_and_constructor_requires_its_own_mapping(self):
        del self.data["access_mappings"]["CharacterData.data/set"]
        self.rejected("cover each accessor/constructor/overload")

    def test_numeric_dictionary_required_flag_is_rejected_before_rust_emission(self):
        self.member("GetRootNodeOptions.composed")["required"] = 0
        self.rejected("required must be boolean")

    def test_empty_native_producer_owner_and_symbol_are_not_evidence(self):
        producer = self.data["access_mappings"]["Node.nodeName/get"]["producers"][0]
        producer["owner"] = ""
        self.rejected("producer owner")
        self.setUp()
        self.data["access_mappings"]["Node.nodeName/get"]["producers"][0]["symbol"] = ""
        self.rejected("producer symbol")

    def test_json_field_types_are_not_inferred_from_truthiness(self):
        cases = [
            lambda d: d.update(schema=True),
            lambda d: d.update(sources={}),
            lambda d: d["sources"][0].update(block=True),
            lambda d: d["sources"][0].update(block=0.0),
            lambda d: d["sources"][0].update(anchor=[]),
            lambda d: d["sources"][0].update(id=""),
            lambda d: d["sources"][0].update(entries="Node"),
            lambda d: d["sources"][0].update(revision=40),
            lambda d: d["sources"][0].update(url="https://"),
            lambda d: d["sources"][0].update(url="https://[bad"),
            lambda d: d["sources"][0].update(url="https://@example.com/source"),
            lambda d: d["sources"][0].update(path="./references/source.md"),
            lambda d: d["sources"][0].update(id="Node source"),
            lambda d: d["declarations"][0].update(partial=0),
            lambda d: d["declarations"][0].update(partial=""),
            lambda d: d["declarations"][0].update(inherits=False),
            lambda d: d["declarations"][0].update(members={}),
            lambda d: d["declarations"][0].update(attributes={}),
            lambda d: d["externals"][0].update(owner=0),
            lambda d: d["externals"][0].update(requirement=[]),
            lambda d: d["externals"][0].update(kind=False),
            lambda d: d["access_mappings"]["Node.nodeName/get"].update(adapter=0),
            lambda d: d["access_mappings"]["Node.nodeName/get"].update(producers="producer"),
            lambda d: d["access_mappings"]["Node.nodeName/get"]["producers"][0].update(symbol=[]),
            lambda d: d["access_mappings"]["Node.nodeName/get"]["producers"][0].update(owner="surgeist-dom/src"),
            lambda d: d["access_mappings"]["Node.nodeName/get"]["producers"][0].update(path="/absolute/path.rs"),
            lambda d: d["access_mappings"]["Element.attributes/get"]["requirements"][0].update(issue=True),
            lambda d: d["access_mappings"]["Element.attributes/get"]["requirements"][0].update(issue=2**32),
        ]
        for index, mutate in enumerate(cases):
            with self.subTest(case=index):
                self.setUp()
                mutate(self.data)
                with self.assertRaises(g.DescriptionError):
                    g.generate(self.data)
        for field, identifier, invalid in [
            ("required", "GetRootNodeOptions.composed", None),
            ("readonly", "Node.nodeName", 1),
            ("static", "Document.html.parseHTML:0", 0),
            ("id", "Node.nodeName", []),
            ("attributes", "Node.nodeName", "SameObject"),
            ("type", "Node.nodeName", []),
            ("arguments", "Text.constructor:0", {}),
            ("value", "Node.ELEMENT_NODE", True),
        ]:
            with self.subTest(field=field):
                self.setUp()
                self.member(identifier)[field] = invalid
                with self.assertRaises(g.DescriptionError):
                    g.generate(self.data)
        self.setUp()
        self.member("GetRootNodeOptions.composed")["default"]["value"] = 0
        self.rejected("default value must be boolean")
        self.setUp()
        self.definition("ShadowRootMode")["values"][0] = 1
        self.rejected("enum value")

    def test_unknown_schema_fields_do_not_silently_disappear(self):
        self.member("Node.nodeName")["installed"] = True
        self.rejected("unknown fields")


    def test_omission_nullable_default_and_legacy_string_annotation_remain_distinct(self):
        title = self.member("DOMImplementation.createHTMLDocument:0")["arguments"][0]
        self.assertEqual(title["presence"], "optional")
        self.assertNotIn("default", title)
        doctype = self.member("DOMImplementation.createDocument:0")["arguments"][2]
        self.assertEqual(doctype["type"], {"kind": "nullable", "inner": {"kind": "named", "name": "DocumentType"}})
        self.assertEqual(doctype["default"], {"kind": "null"})
        text = self.member("Text.constructor:0")["arguments"][0]
        self.assertEqual(text["default"], {"kind": "string", "value": ""})
        self.assertEqual(self.member("CharacterData.data")["type"]["attributes"], [{"name": "LegacyNullToEmptyString"}])
        self.assertEqual(self.member("Node.nodeValue")["type"]["kind"], "nullable")


    def test_body_partial_lowers_five_null_to_empty_setters_and_raw_background(self):
        body = self.definition("HTMLBodyElement.obsolete")
        self.assertEqual([member["name"] for member in body["members"]],
                         ["text", "link", "vLink", "aLink", "bgColor", "background"])
        model = g.lower_values(self.data)
        selected = {identity: info for identity, _, info in g.lower_selected_setters(self.data, model)}
        for member in body["members"][:-1]:
            with self.subTest(member=member["name"]):
                self.assertEqual(selected[member["id"]],
                                 ("DomString", "Type<DomString>", "types::DOM_STRING_NULL_TO_EMPTY"))
        background = body["members"][-1]
        self.assertEqual(g.conversion_type(background["type"], model),
                         ("DomString", "Type<DomString>", "types::DOM_STRING"))
        self.assertNotIn(background["id"], selected)
        background["type"]["attributes"] = [{"name": "LegacyNullToEmptyString"}]
        self.rejected("source declaration mismatch: HTMLBodyElement.obsolete.source")

    def test_frameset_primary_preserves_plain_strings_and_window_event_mixin(self):
        frameset = self.definition("HTMLFrameSetElement")
        self.assertEqual(frameset["inherits"], "HTMLElement")
        ctor, cols, rows = frameset["members"]
        self.assertEqual(ctor["id"], "HTMLFrameSetElement.constructor:0")
        self.assertEqual([cols["name"], rows["name"]], ["cols", "rows"])
        model = g.lower_values(self.data)
        for member in [cols, rows]:
            with self.subTest(member=member["name"]):
                self.assertEqual(g.conversion_type(member["type"], model),
                                 ("DomString", "Type<DomString>", "types::DOM_STRING"))
        include = next(item for item in self.data["includes"] if item["interface"] == "HTMLFrameSetElement")
        self.assertEqual(include["mixin"], "WindowEventHandlers")
        include["mixin"] = "GlobalEventHandlers"
        self.rejected("source declaration mismatch: HTMLFrameSetElement.source")

    def test_label_catalogue_preserves_the_retained_interface(self):
        definitions, _ = g.validate(self.data)
        label = definitions.get("HTMLLabelElement")
        self.assertIsNotNone(label, "missing HTMLLabelElement in the validated catalogue")
        source = next(s for s in self.data["sources"] if "HTMLLabelElement" in s["entries"])
        self.assertEqual(source["path"],
                         "references/html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md")
        self.assertEqual((source["anchor"], source["block"]), ("htmllabelelement", 0))
        self.assertEqual(source["sha256"],
                         "332cd00a0d3af09002d9b5579d58fde2c9081806e1d8fce4bd86b6dafe5ccba3")
        self.assertEqual(source["revision"], "877d6146487f15b15bc8dd0e3c625063a37b456a")
        self.assertEqual(label["inherits"], "HTMLElement")
        constructor, form, html_for, control = label["members"]
        self.assertEqual(constructor["kind"], "constructor")
        self.assertEqual(constructor["attributes"], [{"name": "HTMLConstructor"}])
        self.assertEqual(constructor["arguments"], [])
        self.assertEqual([member["name"] for member in (form, html_for, control)],
                         ["form", "htmlFor", "control"])
        for member, interface in [(form, "HTMLFormElement"), (control, "HTMLElement")]:
            with self.subTest(member=member["name"]):
                self.assertTrue(member["readonly"])
                self.assertEqual(member["type"],
                                 {"kind": "nullable", "inner": {"kind": "named", "name": interface}})
        self.assertFalse(html_for["readonly"])
        self.assertEqual(html_for["attributes"],
                         [{"name": "CEReactions"}, {"name": "Reflect", "value": "for"}])
        self.assertEqual(g.conversion_type(html_for["type"], g.lower_values(self.data)),
                         ("DomString", "Type<DomString>", "types::DOM_STRING"))

    def test_legend_keeps_readonly_nullable_form_and_plain_alignment(self):
        legend = self.definition("HTMLLegendElement")
        self.assertEqual(legend["inherits"], "HTMLElement")
        self.assertEqual([member["id"] for member in legend["members"]],
                         ["HTMLLegendElement.constructor:0", "HTMLLegendElement.form"])
        form = self.member("HTMLLegendElement.form")
        self.assertTrue(form["readonly"])
        self.assertEqual(form["type"],
                         {"kind": "nullable", "inner": {"kind": "named", "name": "HTMLFormElement"}})
        align = self.member("HTMLLegendElement.obsolete.align")
        self.assertEqual(align["attributes"], [{"name": "CEReactions"}, {"name": "Reflect"}])
        self.assertEqual(g.conversion_type(align["type"], g.lower_values(self.data)),
                         ("DomString", "Type<DomString>", "types::DOM_STRING"))
        form["type"] = form["type"]["inner"]
        self.rejected("source declaration mismatch: HTMLLegendElement.source")

    def test_marquee_primary_preserves_signed_loop_and_unsigned_defaults(self):
        marquee = self.definition("HTMLMarqueeElement")
        self.assertEqual(marquee["inherits"], "HTMLElement")
        ctor, *attributes, start, stop = marquee["members"]
        self.assertEqual(ctor["id"], "HTMLMarqueeElement.constructor:0")
        self.assertEqual([member["name"] for member in attributes],
                         ["behavior", "bgColor", "direction", "height", "hspace", "loop",
                          "scrollAmount", "scrollDelay", "trueSpeed", "vspace", "width"])
        for method, name in [(start, "start"), (stop, "stop")]:
            self.assertEqual((method["name"], method["arguments"], method["attributes"], method["type"]["name"]),
                             (name, [], [], "undefined"))
        model = g.lower_values(self.data)
        for name, kind, default in [("hspace", "UnsignedLong", None),
                                    ("loop", "Long", None),
                                    ("scrollAmount", "UnsignedLong", 6),
                                    ("scrollDelay", "UnsignedLong", 85),
                                    ("vspace", "UnsignedLong", None)]:
            with self.subTest(member=name):
                member = self.member("HTMLMarqueeElement." + name)
                self.assertEqual(g.conversion_type(member["type"], model),
                                 ("IntegerValue", "IntegerType",
                                  f"IntegerType::new(IntegerKind::{kind}, IntegerMode::Wrap)"))
                expected = [{"name": "CEReactions"}]
                if name != "loop":
                    expected.append({"name": "Reflect"})
                if default is not None:
                    expected.append({"name": "ReflectDefault", "value": default})
                self.assertEqual(member["attributes"], expected)
        self.member("HTMLMarqueeElement.scrollAmount")["attributes"][-1]["value"] = 0
        self.rejected("source declaration mismatch: HTMLMarqueeElement.source")

    def test_form_catalog_preserves_legacy_operations_and_encoding(self):
        form = self.definition("HTMLFormElement")
        self.assertEqual(form["inherits"], "HTMLElement")
        self.assertEqual(form["attributes"], [
            {"name": "Exposed", "value": "Window"},
            {"name": "LegacyOverrideBuiltIns"},
            {"name": "LegacyUnenumerableNamedProperties"},
        ])
        getters = [member for member in form["members"] if member.get("special") == "getter"]
        self.assertEqual([member["arguments"][0]["type"]["name"] for member in getters],
                         ["unsigned long", "DOMString"])
        mappings = self.data["access_mappings"]
        for property_name in ["acceptCharset", "autocomplete", "enctype", "encoding", "method",
                              "name", "noValidate", "target", "rel"]:
            for access in ["get", "set"]:
                row = mappings[f"HTMLFormElement.{property_name}/{access}"]
                self.assertFalse(row["requirements"])
        for property_name in ["enctype", "encoding"]:
            getter = mappings[f"HTMLFormElement.{property_name}/get"]
            self.assertIn("form_enctype_keyword_with", [row["symbol"] for row in getter["producers"]])
        self.assertEqual(mappings["HTMLFormElement.operation:0/call"]["requirement"]["issue"], 2480)
        form["attributes"].pop(1)
        self.rejected("source declaration mismatch: HTMLFormElement.source")

    def test_fieldset_retains_required_error_and_live_object_requirements(self):
        fieldset = self.definition("HTMLFieldSetElement")
        self.assertEqual(fieldset["inherits"], "HTMLElement")
        ctor, *members = fieldset["members"]
        self.assertEqual(ctor["id"], "HTMLFieldSetElement.constructor:0")
        self.assertEqual([member["name"] for member in members],
                         ["disabled", "form", "name", "type", "elements", "willValidate", "validity",
                          "validationMessage", "checkValidity", "reportValidity", "setCustomValidity"])
        setter = members[-1]
        self.assertEqual(setter["arguments"], [
            {"name": "error", "presence": "required", "type": {"kind": "named", "name": "DOMString"}},
        ])
        mappings = self.data["access_mappings"]
        writer = mappings[setter["id"] + "/call"]
        self.assertFalse(writer["requirements"])
        self.assertEqual({producer["symbol"] for producer in writer["producers"]},
                         {"control_set_custom_validity", "set_custom_validity"})
        for name, issues in [("elements", {1235, 2480}), ("validity", {1235})]:
            with self.subTest(member=name):
                member = self.member("HTMLFieldSetElement." + name)
                self.assertEqual(member["attributes"], [{"name": "SameObject"}])
                getter = mappings[member["id"] + "/get"]
                self.assertEqual(getter["result_view"], "live")
                self.assertEqual({requirement["issue"] for requirement in getter["requirements"]}, issues)
        setter["arguments"][0]["presence"] = "optional"
        self.rejected("source declaration mismatch: HTMLFieldSetElement.source")

    def test_output_keeps_native_value_writers_and_live_object_contracts(self):
        output = self.definition("HTMLOutputElement")
        self.assertEqual(output["inherits"], "HTMLElement")
        ctor, *members = output["members"]
        self.assertEqual(ctor["id"], "HTMLOutputElement.constructor:0")
        self.assertEqual([member["name"] for member in members],
                         ["htmlFor", "form", "name", "type", "defaultValue", "value",
                          "willValidate", "validity", "validationMessage", "checkValidity",
                          "reportValidity", "setCustomValidity", "labels"])
        mappings = self.data["access_mappings"]
        for name, command in [("defaultValue", "set_output_default_value"), ("value", "set_output_value")]:
            with self.subTest(member=name):
                member = self.member("HTMLOutputElement." + name)
                self.assertEqual(member["attributes"], [{"name": "CEReactions"}])
                self.assertEqual(member["type"], {"kind": "named", "name": "DOMString"})
                writer = mappings[member["id"] + "/set"]
                self.assertFalse(writer["requirements"])
                self.assertEqual({producer["symbol"] for producer in writer["producers"]},
                                 {"write_output_value", command})
        self.assertEqual(mappings["HTMLOutputElement.htmlFor/set"]["target"], "value")
        self.assertEqual(mappings["HTMLOutputElement.htmlFor/set"]["kind"], "forwarding")
        for name in ["validity", "labels"]:
            with self.subTest(member=name):
                getter = mappings[f"HTMLOutputElement.{name}/get"]
                self.assertEqual(getter["result_view"], "live")
                self.assertTrue(getter.get("requirements") or getter.get("requirement"))
        self.member("HTMLOutputElement.defaultValue")["attributes"].clear()
        self.rejected("source declaration mismatch: HTMLOutputElement.source")

    def test_html_token_lists_use_the_exact_live_association_and_forward_set(self):
        mappings = self.data["access_mappings"]
        for interface, member, attribute in [
            ("HTMLScriptElement", "blocking", None),
            ("HTMLLinkElement", "blocking", None),
            ("HTMLStyleElement", "blocking", None),
            ("HTMLAnchorElement", "relList", "rel"),
            ("HTMLAreaElement", "relList", "rel"),
            ("HTMLFormElement", "relList", "rel"),
            ("HTMLLinkElement", "relList", "rel"),
            ("HTMLLinkElement", "sizes", None),
            ("HTMLIFrameElement", "sandbox", None),
            ("HTMLOutputElement", "htmlFor", "for"),
        ]:
            key = f"{interface}.{member}"
            with self.subTest(member=key):
                declaration = self.member(key)
                self.assertTrue(declaration["readonly"])
                self.assertEqual(declaration["type"], {"kind": "named", "name": "DOMTokenList"})
                self.assertEqual({item["name"]: item.get("value") for item in declaration["attributes"]},
                                 {"SameObject": None, "PutForwards": "value", "Reflect": attribute})
                getter = mappings[key + "/get"]
                self.assertEqual((getter["kind"], getter["result_view"]), ("native", "live"))
                self.assertFalse(getter["requirements"])
                self.assertEqual({(item["owner"], item["symbol"]) for item in getter["producers"]},
                                 {("surgeist-runtime", "element_token_list"),
                                  ("surgeist-dom", "reflect_html")})
                setter = mappings[key + "/set"]
                self.assertEqual((setter["kind"], setter["getter"], setter["target"]),
                                 ("forwarding", key + "/get", "value"))

    def test_button_keeps_owned_keyword_rules_and_the_popover_mixin(self):
        button = self.definition("HTMLButtonElement")
        self.assertEqual(button["inherits"], "HTMLElement")
        ctor, *members = button["members"]
        self.assertEqual(ctor["id"], "HTMLButtonElement.constructor:0")
        self.assertEqual([member["name"] for member in members],
                         ["command", "commandForElement", "disabled", "form", "formAction", "formEnctype",
                          "formMethod", "formNoValidate", "formTarget", "name", "type", "value",
                          "willValidate", "validity", "validationMessage", "checkValidity",
                          "reportValidity", "setCustomValidity", "labels"])
        include = next(row for row in self.data["includes"] if row["interface"] == "HTMLButtonElement")
        self.assertEqual(include["mixin"], "PopoverTargetAttributes")
        mappings = self.data["access_mappings"]
        for name, producer in [("command", "button_command_readback_with"), ("type", "button_type_keyword_with"),
                               ("formMethod", "form_method_override_keyword_with"),
                               ("formEnctype", "form_enctype_override_keyword_with")]:
            with self.subTest(member=name):
                read = mappings[f"HTMLButtonElement.{name}/get"]
                self.assertFalse(read["requirements"])
                self.assertIn(producer, [item["symbol"] for item in read["producers"]])
        for operation, native, runtime in [
            ("get", "html_command_for_with", "read_reflected_element"),
            ("set", "set_command_for_element", "write_reflected_element"),
        ]:
            with self.subTest(operation=operation):
                access = mappings[f"HTMLButtonElement.commandForElement/{operation}"]
                self.assertEqual(access["kind"], "native")
                self.assertFalse(access["requirements"])
                self.assertEqual({(item["owner"], item["symbol"]) for item in access["producers"]},
                                 {("surgeist-dom", native), ("surgeist-runtime", runtime)})
        for name in ["formAction", "validity", "labels"]:
            with self.subTest(member=name):
                read = mappings[f"HTMLButtonElement.{name}/get"]
                self.assertTrue(read.get("requirements") or read.get("requirement"))
        for name, producer in [("labels", "control_labels")]:
            with self.subTest(member=name):
                read = mappings[f"HTMLButtonElement.{name}/get"]
                self.assertIn(producer, [item["symbol"] for item in read["producers"]])
                self.assertEqual({item["owner"] for item in read["requirements"]},
                                 {"surgeist-dom", "surgeist-runtime"})
        self.member("HTMLButtonElement.type")["attributes"][-1]["name"] = "Reflect"
        self.rejected("source declaration mismatch: HTMLButtonElement.source")

    def test_live_will_validate_uses_native_candidacy(self):
        mappings = self.data["access_mappings"]
        for interface in ("HTMLButtonElement", "HTMLInputElement", "HTMLSelectElement", "HTMLTextAreaElement"):
            with self.subTest(interface=interface):
                getter = mappings[f"{interface}.willValidate/get"]
                self.assertEqual(getter["kind"], "native")
                self.assertFalse(getter["requirements"])
                self.assertEqual({(item["owner"], item["symbol"]) for item in getter["producers"]},
                                 {("surgeist-dom", "html_will_validate_with"),
                                  ("surgeist-html-semantics", "validation_candidacy"),
                                  ("surgeist-runtime", "will_validate")})

    def test_textarea_preserves_value_conversion_numeric_metadata_and_selection_overloads(self):
        textarea = self.definition("HTMLTextAreaElement")
        self.assertEqual(textarea["inherits"], "HTMLElement")
        for name, fallback in [("cols", 20), ("rows", 2)]:
            with self.subTest(member=name):
                member = self.member("HTMLTextAreaElement." + name)
                self.assertEqual(member["type"]["name"], "unsigned long")
                self.assertEqual(member["attributes"], [
                    {"name": "CEReactions"}, {"name": "ReflectPositiveWithFallback"},
                    {"name": "ReflectDefault", "value": fallback},
                ])
        for name in ["maxLength", "minLength"]:
            with self.subTest(member=name):
                member = self.member("HTMLTextAreaElement." + name)
                self.assertEqual(member["type"]["name"], "long")
                self.assertEqual(member["attributes"],
                                 [{"name": "CEReactions"}, {"name": "ReflectNonNegative"}])

        model = g.lower_values(self.data)
        default = self.member("HTMLTextAreaElement.defaultValue")
        self.assertEqual(g.conversion_type(default["type"], model),
                         ("DomString", "Type<DomString>", "types::DOM_STRING"))
        self.assertEqual(default["attributes"], [{"name": "CEReactions"}])
        selected = {identity: info for identity, _, info in g.lower_selected_setters(self.data, model)}
        self.assertEqual(selected["HTMLTextAreaElement.value"],
                         ("DomString", "Type<DomString>", "types::DOM_STRING_NULL_TO_EMPTY"))
        self.assertEqual(self.member("HTMLTextAreaElement.value")["attributes"], [])
        self.assertEqual(self.definition("SelectionMode")["values"], ["select", "start", "end", "preserve"])
        replacement = self.member("HTMLTextAreaElement.setRangeText:0")["arguments"]
        self.assertEqual([argument["name"] for argument in replacement], ["replacement"])
        selection = self.member("HTMLTextAreaElement.setRangeText:1")["arguments"]
        self.assertEqual([argument["name"] for argument in selection],
                         ["replacement", "start", "end", "selectionMode"])
        self.assertEqual(selection[-1]["default"], {"kind": "string", "value": "preserve"})
        arguments = g.lower_arguments(self.data, model)
        self.assertIn("Ok(cx.plain(SelectionMode::Preserve))", g.emit_conversions(model, arguments))
        selection[-1]["default"]["value"] = "unknown"
        with self.assertRaisesRegex(g.DescriptionError, "enumeration argument default is not a member"):
            g.emit_conversions(model, arguments)
        selection[-1]["default"]["value"] = "preserve"
        self.member("HTMLTextAreaElement.cols")["attributes"][-1]["value"] = 2
        self.rejected("source declaration mismatch: HTMLTextAreaElement.source")

    def test_input_preserves_value_conversion_file_type_and_signed_step_defaults(self):
        input_element = self.definition("HTMLInputElement")
        self.assertEqual(input_element["inherits"], "HTMLElement")
        self.assertEqual(self.member("HTMLInputElement.src")["attributes"],
                         [{"name": "CEReactions"}, {"name": "ReflectURL"}])
        self.assertEqual(self.member("HTMLInputElement.src")["type"]["name"], "USVString")
        for name, kind in [("files", "FileList"), ("list", "HTMLDataListElement")]:
            with self.subTest(member=name):
                self.assertEqual(self.member("HTMLInputElement." + name)["type"],
                                 {"kind": "nullable", "inner": {"kind": "named", "name": kind}})

        model = g.lower_values(self.data)
        selected = {identity: info for identity, _, info in g.lower_selected_setters(self.data, model)}
        self.assertEqual(selected["HTMLInputElement.value"],
                         ("DomString", "Type<DomString>", "types::DOM_STRING_NULL_TO_EMPTY"))
        self.assertNotIn("HTMLInputElement.defaultValue", selected)
        for name in ["stepUp", "stepDown"]:
            with self.subTest(member=name):
                self.assertEqual(self.member(f"HTMLInputElement.{name}:0")["arguments"], [{
                    "name": "n", "presence": "optional", "type": {"kind": "named", "name": "long"},
                    "default": {"kind": "integer", "value": 1},
                }])
        arguments = g.lower_arguments(self.data, model)
        output = g.emit_conversions(model, arguments)
        self.assertEqual(output.count("Ok(cx.plain(IntegerValue::Long(1)))"), 2)

        default = self.member("HTMLInputElement.stepUp:0")["arguments"][0]["default"]
        for value in [-(2**31) - 1, 2**31]:
            with self.subTest(default=value):
                default["value"] = value
                with self.assertRaisesRegex(g.DescriptionError, "integer argument default"):
                    g.emit_conversions(model, arguments)
        default["value"] = 2
        self.rejected("source declaration mismatch: HTMLInputElement.source")

    def test_table_row_indexes_use_native_scalar_observations(self):
        row = self.definition("HTMLTableRowElement")
        members = row["members"]
        self.assertEqual([member["id"] for member in members[:3]],
                         ["HTMLTableRowElement.constructor:0", "HTMLTableRowElement.rowIndex",
                          "HTMLTableRowElement.sectionRowIndex"])
        mappings = self.data["access_mappings"]
        for member in members[1:3]:
            self.assertEqual(member["kind"], "attribute")
            self.assertTrue(member["readonly"])
            self.assertEqual(member["type"], {"kind": "named", "name": "long"})
            accessor = mappings[member["id"] + "/get"]
            self.assertFalse(accessor["requirements"])
            self.assertEqual({producer["owner"] for producer in accessor["producers"]},
                             {"surgeist-dom", "surgeist-html-semantics", "surgeist-runtime"})
            self.assertNotIn(member["id"] + "/set", mappings)
        cells = mappings["HTMLTableRowElement.cells/get"]
        self.assertEqual(cells["result_view"], "live")
        self.assertIn(2480, {provider["issue"] for provider in cells["requirements"]
                             if "issue" in provider})

    def test_table_parts_keep_pinned_writable_and_creation_contracts(self):
        table = self.definition("HTMLTableElement")
        members = table["members"][1:10]
        parts = [("caption", "Caption", "HTMLTableCaptionElement"),
                 ("tHead", "THead", "HTMLTableSectionElement"),
                 ("tFoot", "TFoot", "HTMLTableSectionElement")]
        mappings = self.data["access_mappings"]
        reactions = [{"name": "CEReactions"}]
        for index, (attribute, suffix, result_interface) in enumerate(parts):
            with self.subTest(attribute=attribute):
                value, create, delete = members[index * 3:index * 3 + 3]
                self.assertEqual([member["name"] for member in (value, create, delete)],
                                 [attribute, "create" + suffix, "delete" + suffix])
                interface = {"kind": "named", "name": result_interface}
                self.assertEqual(value["type"], {"kind": "nullable", "inner": interface})
                self.assertFalse(value["readonly"])
                self.assertEqual(value["attributes"], reactions)
                self.assertEqual(create["type"], interface)
                self.assertEqual(create["attributes"], [])
                self.assertEqual(delete["type"], {"kind": "named", "name": "undefined"})
                self.assertEqual(delete["attributes"], reactions)
                self.assertEqual(create["arguments"], [])
                self.assertEqual(delete["arguments"], [])
                for member, edge, producer in [
                    (value, "get", "html_table_part_with"),
                    (value, "set", "set_html_table_part"),
                    (create, "call", "create_html_table_part_in_attachment"),
                    (delete, "call", "delete_html_table_part"),
                ]:
                    access = mappings[member["id"] + "/" + edge]
                    self.assertFalse(access["requirements"])
                    self.assertEqual({item["owner"] for item in access["producers"]},
                                     {"surgeist-dom", "surgeist-runtime"})
                    self.assertIn(producer, {item["symbol"] for item in access["producers"]
                                             if item["owner"] == "surgeist-dom"})
        for attribute in ["tBodies", "rows"]:
            access = mappings[f"HTMLTableElement.{attribute}/get"]
            self.assertEqual(access["result_view"], "live")
            self.assertIn(2480, {provider["issue"] for provider in access["requirements"]
                                 if "issue" in provider})

    def test_table_legacy_partials_preserve_pinned_aliases_and_string_types(self):
        members = {
            "HTMLTableElement": ["align", "border", "frame", "rules", "summary", "width",
                                 "bgColor", "cellPadding", "cellSpacing"],
            "HTMLTableCaptionElement": ["align"],
            "HTMLTableSectionElement": ["align", "ch", "chOff", "vAlign"],
            "HTMLTableRowElement": ["align", "ch", "chOff", "vAlign", "bgColor"],
        }
        aliases = {"ch": "char", "chOff": "charoff"}
        mappings = self.data["access_mappings"]
        for interface, names in members.items():
            with self.subTest(interface=interface):
                self.assertEqual(self.definition(interface)["inherits"], "HTMLElement")
                partial = self.definition(f"{interface}.obsolete")
                self.assertTrue(partial["partial"])
                self.assertEqual([member["name"] for member in partial["members"]], names)
                for member in partial["members"]:
                    name = member["name"]
                    reflect = {"name": "Reflect"}
                    if name in aliases:
                        reflect["value"] = aliases[name]
                    self.assertEqual(member["attributes"], [{"name": "CEReactions"}, reflect])
                    expected = {"kind": "named", "name": "DOMString"}
                    if name in {"bgColor", "cellPadding", "cellSpacing"}:
                        expected["attributes"] = [{"name": "LegacyNullToEmptyString"}]
                    self.assertEqual(member["type"], expected)
                    for access in ["get", "set"]:
                        self.assertFalse(mappings[f"{member['id']}/{access}"]["requirements"])
        self.member("HTMLTableRowElement.obsolete.chOff")["attributes"][1].pop("value")
        self.rejected("source declaration mismatch: HTMLTableRowElement.obsolete.source")

    def test_custom_validity_shares_native_source_with_required_dom_string(self):
        mappings = self.data["access_mappings"]
        for name in ["Button", "FieldSet", "Input", "Output", "Select", "TextArea"]:
            with self.subTest(control=name):
                identity = f"HTML{name}Element.setCustomValidity:0"
                self.assertEqual(self.member(identity)["arguments"], [{
                    "name": "error",
                    "presence": "required",
                    "type": {"kind": "named", "name": "DOMString"},
                }])
                call = mappings[f"{identity}/call"]
                self.assertFalse(call["requirements"])
                self.assertEqual(
                    {(item["owner"], item["symbol"]) for item in call["producers"]},
                    {("surgeist-dom", "set_custom_validity"), ("surgeist-runtime", "control_set_custom_validity")},
                )

    def test_control_autocomplete_uses_its_own_pinned_idl_projection(self):
        mappings = self.data["access_mappings"]
        for interface in ["HTMLInputElement", "HTMLSelectElement", "HTMLTextAreaElement"]:
            with self.subTest(interface=interface):
                identity = f"{interface}.autocomplete"
                member = self.member(identity)
                self.assertEqual(member["attributes"], [{"name": "CEReactions"}, {"name": "ReflectSetter"}])
                getter = mappings[f"{identity}/get"]
                setter = mappings[f"{identity}/set"]
                self.assertFalse(getter["requirements"])
                self.assertFalse(setter["requirements"])
                self.assertIn("control_autocomplete_idl_with", [item["symbol"] for item in getter["producers"]])
                self.assertIn("set_attribute_ns", [item["symbol"] for item in setter["producers"]])

    def test_integer_controls_name_the_live_reflection_and_source_providers(self):
        mappings = self.data["access_mappings"]
        for interface, dimensions in [("HTMLInputElement", ["size"]), ("HTMLTextAreaElement", ["cols", "rows"])]:
            for name in ["maxLength", "minLength"] + dimensions:
                with self.subTest(interface=interface, member=name):
                    getter = mappings[f"{interface}.{name}/get"]
                    setter = mappings[f"{interface}.{name}/set"]
                    self.assertFalse(getter["requirements"])
                    self.assertFalse(setter["requirements"])
                    self.assertEqual({item["owner"] for item in getter["producers"]},
                                     {"surgeist-dom", "surgeist-html"})
                    self.assertEqual({item["owner"] for item in setter["producers"]},
                                     {"surgeist-dom", "surgeist-html"})
                    self.assertIn("set_attribute_ns", [item["symbol"] for item in setter["producers"]])
                    length = name in ["maxLength", "minLength"]
                    getter_symbol = "control_length_reflection_with" if length else "read_with"
                    setter_symbol = "control_length_setter_value" if length else "setter_value"
                    self.assertIn(getter_symbol, [item["symbol"] for item in getter["producers"]])
                    self.assertIn(setter_symbol, [item["symbol"] for item in setter["producers"]])

    def test_input_local_reflections_retain_the_native_producers_and_obsolete_partial(self):
        obsolete = self.definition("HTMLInputElement.obsolete")
        self.assertTrue(obsolete["partial"])
        self.assertEqual(obsolete["name"], "HTMLInputElement")
        self.assertEqual([member["name"] for member in obsolete["members"]], ["align", "useMap"])
        mappings = self.data["access_mappings"]
        for name, producer in [
            ("colorSpace", "input_colorspace_with"),
            ("formEnctype", "form_enctype_override_keyword_with"),
            ("formMethod", "form_method_override_keyword_with"),
            ("obsolete.align", "get_null_namespace_attribute_with"),
            ("obsolete.useMap", "get_null_namespace_attribute_with"),
        ]:
            with self.subTest(getter=name):
                getter = mappings[f"HTMLInputElement.{name}/get"]
                self.assertFalse(getter["requirements"])
                self.assertIn(producer, [item["symbol"] for item in getter["producers"]])
        for name in ["alpha", "colorSpace", "max", "min", "step"]:
            with self.subTest(setter=name):
                setter = mappings[f"HTMLInputElement.{name}/set"]
                self.assertFalse(setter["requirements"])
                self.assertIn("input_attribute_change", [item["symbol"] for item in setter["producers"]])
        self.member("HTMLInputElement.obsolete.useMap")["attributes"].pop()
        self.rejected("source declaration mismatch: HTMLInputElement.obsolete.source")

    def test_dialog_state_and_native_methods_preserve_the_primary_declaration(self):
        dialog = self.definition("HTMLDialogElement")
        self.assertEqual(dialog["inherits"], "HTMLElement")
        constructor, opened, value, closed_by, *methods = dialog["members"]
        self.assertEqual(constructor["id"], "HTMLDialogElement.constructor:0")
        self.assertEqual([member["name"] for member in [opened, value, closed_by]],
                         ["open", "returnValue", "closedBy"])
        self.assertEqual(opened["attributes"], [{"name": "CEReactions"}, {"name": "Reflect"}])
        self.assertEqual(value["type"]["name"], "DOMString")
        self.assertEqual(value["attributes"], [])
        self.assertEqual(closed_by["attributes"], [{"name": "CEReactions"}, {"name": "ReflectSetter"}])
        self.assertEqual([method["name"] for method in methods],
                         ["show", "showModal", "close", "requestClose"])
        native_commands = {
            "show": "show_dialog",
            "showModal": "show_modal_dialog",
            "close": "close_dialog",
            "requestClose": "request_close_dialog",
        }
        for method in methods:
            with self.subTest(method=method["name"]):
                self.assertEqual(method["type"]["name"], "undefined")
                self.assertEqual(method["attributes"], [{"name": "CEReactions"}])
                arguments = method["arguments"]
                if method["name"] in {"close", "requestClose"}:
                    self.assertEqual(arguments, [{"name": "returnValue", "presence": "optional",
                                                 "type": {"kind": "named", "name": "DOMString"}}])
                else:
                    self.assertFalse(arguments)
                mapping = self.data["access_mappings"][method["id"] + "/call"]
                self.assertEqual({entry["issue"] for entry in mapping["requirements"]}, {1225})
                self.assertEqual({producer["symbol"] for producer in mapping["producers"]},
                                 {native_commands[method["name"]], "dialog_method",
                                  "service_guest_source_event"})
        closed_by["attributes"][-1]["name"] = "Reflect"
        self.rejected("source declaration mismatch: HTMLDialogElement.source")

    def test_core_node_values_and_constructors_name_the_live_routes(self):
        routes = {
            "Node.nodeValue/get": {"read_node_text", "data", "value"},
            "Node.nodeValue/set": {"write_node_text", "replace_data", "set_attribute_value"},
            "Node.textContent/get": {"read_node_text", "visit_descendant_text_with", "value"},
            "Node.textContent/set": {"write_node_text", "set_text_content"},
            "Node.normalize:0/call": {"normalize_node", "normalize"},
            "Document.constructor:0/construct": {
                "construct_document", "document_constructor_context",
            },
            "DocumentFragment.constructor:0/construct": {"construct_fragment", "create_detached"},
            "Text.constructor:0/construct": {"construct_character", "create_detached"},
            "Comment.constructor:0/construct": {"construct_character", "create_detached"},
            "ProcessingInstruction.constructor:0/construct": {
                "construct_instruction", "create_detached",
            },
        }
        attached_constructors = {"Document", "DocumentFragment", "Text", "Comment"}
        mappings = self.data["access_mappings"]
        for access, symbols in routes.items():
            with self.subTest(access=access):
                mapping = mappings[access]
                issues = {r.get("issue") for r in mapping["requirements"]}
                interface = access.split(".", 1)[0]
                self.assertEqual(issues, {2257} if interface in attached_constructors else set())
                self.assertTrue(symbols <= {p["symbol"] for p in mapping["producers"]})

    def test_direct_attr_custody_keeps_the_named_collection_requirement(self):
        mappings = self.data["access_mappings"]
        direct = {
            "Document.createAttribute:0/call": "create_attribute",
            "Document.createAttributeNS:0/call": "create_attribute",
            "Element.getAttributeNode:0/call": "get_named_item_with",
            "Element.getAttributeNodeNS:0/call": "get_named_item_ns_with",
            "Element.setAttributeNode:0/call": "attach_attribute_for_script",
            "Element.setAttributeNodeNS:0/call": "attach_attribute_for_script",
            "Element.removeAttributeNode:0/call": "remove_attribute_node",
            "Attr.value/set": "set_attribute_value",
        }
        direct.update({access: "attr_member" for access in mappings if access.startswith("Attr.")
                       and access.endswith("/get")})
        for access, symbol in direct.items():
            with self.subTest(access=access):
                mapping = mappings[access]
                self.assertFalse(mapping["requirements"])
                self.assertIn(symbol, {p["symbol"] for p in mapping["producers"]})
        for access, mapping in mappings.items():
            if access == "Element.attributes/get" or access.startswith("NamedNodeMap."):
                with self.subTest(access=access):
                    self.assertEqual({r.get("issue") for r in mapping["requirements"]}, {2308, 2480})
        self.assertEqual(mappings["Element.attributes/get"]["result_view"], "live")

    def test_selected_pi_constructor_and_factory_are_both_described(self):
        pi = self.member("ProcessingInstruction.constructor:0")
        self.assertEqual(pi["arguments"][0]["name"], "target")
        self.assertEqual(pi["arguments"][1]["default"], {"kind": "string", "value": ""})
        self.assertEqual(self.member("Document.createProcessingInstruction:0")["type"]["name"], "ProcessingInstruction")
        self.assertTrue(self.data["access_mappings"]["ProcessingInstruction.setAttribute:0/call"]["requirements"])


    def test_forwarding_and_callback_protocols_cannot_be_relabeled_native(self):
        mappings = self.data["access_mappings"]
        for access, expected in [
            ("Document.html.location/set", "PutForwards setter must use guest forwarding"),
            ("EventListener.handleEvent:0/call", "guest invocation protocol"),
        ]:
            with self.subTest(access=access):
                self.setUp()
                self.data["access_mappings"][access] = copy.deepcopy(mappings["Node.nodeName/get"])
                self.rejected(expected)
        self.setUp()
        self.data["access_mappings"]["Document.html.location/set"]["target"] = "value"
        self.rejected("forwarding does not match")
        self.setUp()
        self.data["access_mappings"]["EventListener.handleEvent:0/call"]["interface"] = "Node"
        self.rejected("wrong callback interface")


    def test_role_specific_fields_are_strict_and_cannot_hide_native_claims(self):
        for access, field, value in [
            ("Document.html.location/set", "adapter", "document.location.set"),
            ("EventListener.handleEvent:0/call", "producers", []),
            ("HTMLElement.lang/set", "adapter", "htmlelement.lang.set"),
            ("Document.html.lastModified/get", "adapter", "document.lastmodified.get"),
            ("Document.html.lastModified/get", "requirement", []),
        ]:
            with self.subTest(access=access, field=field):
                self.setUp()
                self.data["access_mappings"][access][field] = value
                with self.assertRaises(g.DescriptionError):
                    g.generate(self.data)




    def test_duplicate_json_keys_are_rejected_before_validation(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "invalid.json"
            path.write_text('{"schema": 1, "schema": 2}')
            with self.assertRaisesRegex(g.DescriptionError, "duplicate JSON key"):
                g.load(path)

    def test_nonfinite_json_is_rejected_instead_of_becoming_a_python_float(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "invalid.json"
            for literal in ("NaN", "Infinity", "-Infinity"):
                with self.subTest(literal=literal):
                    path.write_text('{"schema": ' + literal + '}')
                    with self.assertRaisesRegex(g.DescriptionError, "nonfinite JSON number"):
                        g.load(path)

    def test_check_is_path_independent_read_only_and_reproducible(self):
        before = {path: path.read_bytes() for path in g.generate(self.data)}
        with tempfile.TemporaryDirectory() as directory:
            result = subprocess.run([sys.executable, "-B", str(Path(g.__file__)), "--check"], cwd=directory, capture_output=True, text=True, timeout=30)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual({path: path.read_bytes() for path in before}, before)
        self.assertEqual(g.generate(self.data), g.generate(copy.deepcopy(self.data)))



class ValueLoweringTests(unittest.TestCase):
    def test_element_factories_share_the_declared_string_or_dictionary(self):
        data = g.load()
        model = g.lower_values(data)
        dictionary = next((d for d in model["dictionaries"]
                           if d["name"] == "ElementCreationOptions"), None)
        self.assertIsNotNone(dictionary)
        self.assertEqual([m["name"] for m in dictionary["members"]],
                         ["customElementRegistry", "is"])
        arguments = {a["id"]: a for a in g.lower_arguments(data, model)}
        for name, index in [("createElement", 1), ("createElementNS", 2)]:
            argument = arguments[f"Document.{name}:0/options"]
            self.assertEqual(argument["index"], index)
            self.assertEqual(argument["argument"]["default"], {"kind": "empty_dictionary"})
            self.assertEqual(argument["info"][0], "ElementCreationArgument")

    def test_shadow_attachment_argument_includes_the_nullable_registry(self):
        data = g.load()
        model = g.lower_values(data)
        dictionary = next(
            (d for d in model["dictionaries"] if d["name"] == "ShadowRootInit"),
            None,
        )
        self.assertIsNotNone(dictionary)
        self.assertEqual(
            [member["name"] for member in dictionary["members"]],
            ["clonable", "customElementRegistry", "delegatesFocus", "mode",
             "serializable", "slotAssignment"],
        )
        self.assertIn(
            "Element.attachShadow:0/init",
            [argument["id"] for argument in g.lower_arguments(data, model)],
        )

    def test_current_dictionary_outputs_cover_their_admitted_profiles(self):
        model = g.lower_values(g.load())
        self.assertEqual([d["name"] for d in model["dictionaries"]],
                         ["GetRootNodeOptions", "ElementCreationOptions", "ImportNodeOptions",
                          "ShadowRootInit", "EventInit", "EventListenerOptions", "AssignedNodesOptions",
                          "PageTransitionEventInit"])
        self.assertEqual([d["name"] for d in model["enumerations"]],
                         ["ShadowRootMode", "SlotAssignmentMode", "DocumentReadyState",
                          "DocumentVisibilityState", "CanPlayTypeResult", "SelectionMode"])
        # No member may be discarded. Signal and source combinations outside
        # the selected conversion profiles stay unavailable.
        for name in ("AddEventListenerOptions",
                     "ShowPopoverOptions", "TogglePopoverOptions"):
            self.assertNotIn(name, [d["name"] for d in model["dictionaries"]])

    def test_inheritance_precedes_derived_lexicographic_members(self):
        member = lambda name: {"name": name, "type": {"kind": "named", "name": "boolean"}, "required": False}
        model = g.lower_values({"declarations": [
            {"name": "Base", "kind": "dictionary", "members": [member("z"), member("a")]},
            {"name": "Derived", "kind": "dictionary", "inherits": "Base", "members": [member("y"), member("b")]},
        ]})
        self.assertEqual([m["name"] for m in model["dictionaries"][1]["members"]], ["a", "z", "b", "y"])

    def test_converter_visits_inherited_then_derived_members_immediately(self):
        def member(name):
            return {"id": "fixture." + name, "name": name,
                    "type": {"kind": "named", "name": "boolean"}, "required": False}
        model = g.lower_values({"declarations": [
            {"name": "Base", "kind": "dictionary", "members": [member("z"), member("a")]},
            {"name": "Derived", "kind": "dictionary", "inherits": "Base", "members": [member("y"), member("b")]},
        ]})
        output = g.emit_conversions(model)
        derived = output[output.index("fn convert_derived"):]
        # This is the generator's declared output contract, independently
        # ordered from the unsorted source fixture. Conversion occurs within
        # each member's match, before the following member Get.
        reads = re.findall(r"cx.member\(input, object, &\[([0-9, ]+)\]\)", derived)
        self.assertEqual(reads, ["97", "122", "98", "121"])
        self.assertEqual(re.findall(r"Some\(value\) => (.*)", derived),
                         ["Some(*cx.value(&types::BOOLEAN, &value)?.get()),"] * 4)

    def test_stylesheet_constructor_derives_fixed_capture_and_lexicographic_keys(self):
        data = g.load()
        definitions, _ = g.validate(data)
        output = g.emit_stylesheet(definitions)
        self.assertIn("CAPTURED_OPTIONS: u32 = 1;", output)
        self.assertIn("CONSTRUCTOR_LENGTH: u32 = 0;", output)
        self.assertLess(output.index("const BASE_URL"), output.index("const DISABLED"))
        self.assertLess(output.index("const DISABLED"), output.index("const MEDIA:"))
        model = g.lower_values(data)
        self.assertNotIn("CSSStyleSheetInit", [item["name"] for item in model["dictionaries"]])

    def test_rust_names_keep_acronyms_as_words(self):
        expected = {"DOMImplementation": "dom_implementation", "createHTMLDocument": "create_html_document",
                    "lookupNamespaceURI": "lookup_namespace_uri", "XMLDocument": "xml_document",
                    "Document_html_open_1_url": "document_html_open_1_url", "type": "r#type"}
        for source, rust in expected.items():
            self.assertEqual(g.field_name(source), rust)
        data = g.load()
        arguments = g.lower_arguments(data, g.lower_values(data))
        title = next(a for a in arguments if a["id"] == "DOMImplementation.createHTMLDocument:0/title")
        self.assertEqual(title["function"], "dom_implementation_create_html_document_0_title")

    def test_argument_facts_and_context_restrictions_come_from_declarations(self):
        data = g.load()
        model = g.lower_values(data)
        lowered = {a["id"]: a for a in g.lower_arguments(data, model)}
        title = lowered["DOMImplementation.createHTMLDocument:0/title"]
        self.assertEqual(title["argument"]["presence"], "optional")
        self.assertNotIn("default", title["argument"])
        mask = lowered["Document.createNodeIterator:0/whatToShow"]
        self.assertEqual(mask["index"], 1)
        self.assertEqual(mask["argument"]["default"], {"kind": "integer", "value": 4294967295})
        self.assertEqual(lowered["Node.getRootNode:0/options"]["argument"]["default"], {"kind": "empty_dictionary"})
        self.assertNotIn("Document.createNodeIterator:0/filter", lowered)
        self.assertIsNone(g.conversion_type({"kind": "nullable", "inner": {"kind": "named", "name": "EventInit"}}, model))
        self.assertIsNone(g.conversion_type({"kind": "nullable", "inner": {"kind": "nullable", "inner": {"kind": "named", "name": "DOMString"}}}, model))

    def test_hidden_setter_lowers_the_exact_nullable_boolean_double_string_union(self):
        data = g.load()
        model = g.lower_values(data)
        setters = g.lower_selected_setters(data, model)
        hidden = next(setter for setter in setters if setter[0] == "HTMLElement.hidden")
        self.assertEqual(hidden, (
            "HTMLElement.hidden", "html_element_hidden",
            ("Option<HiddenAttributeValue>", "Nullable<Type<HiddenAttributeValue>>",
             "Type::scalar(super::composition::hidden_attribute_value).nullable()"),
        ))
        output = g.emit_conversions(model, selected_setters=setters)
        self.assertIn("use super::HiddenAttributeValue;", output)
        self.assertIn("pub fn html_element_hidden_setter() -> Nullable<Type<HiddenAttributeValue>>", output)
        hidden_type = next(member["type"] for declaration in data["declarations"]
                           for member in declaration.get("members", [])
                           if member["id"] == "HTMLElement.hidden")
        finite_double = copy.deepcopy(hidden_type)
        finite_double["inner"]["members"][1]["name"] = "double"
        self.assertIsNone(g.conversion_type(finite_double, model))
        self.assertIsNone(g.conversion_type({"kind": "nullable", "inner": hidden_type}, model))

    def test_unavailable_inherited_member_excludes_whole_derived_dictionary(self):
        model = g.lower_values({"declarations": [
            {"name": "Base", "kind": "dictionary", "members": [
                {"name": "node", "type": {"kind": "named", "name": "Node"}, "required": False}]},
            {"name": "Derived", "kind": "dictionary", "inherits": "Base", "members": []},
        ]})
        self.assertEqual(model["dictionaries"], [])

    def test_inherited_member_and_rust_name_collisions_are_rejected(self):
        member = lambda name: {"name": name, "type": {"kind": "named", "name": "boolean"}, "required": False}
        definitions = [
            {"name": "Base", "kind": "dictionary", "members": [member("flag")]},
            {"name": "Derived", "kind": "dictionary", "inherits": "Base", "members": [member("flag")]},
        ]
        with self.assertRaisesRegex(g.DescriptionError, "duplicate inherited"):
            g.lower_values({"declarations": definitions})
        definitions = [{"name": "Flags", "kind": "dictionary", "members": [member("aB"), member("a_b")]}]
        with self.assertRaisesRegex(g.DescriptionError, "field collision"):
            g.lower_values({"declarations": definitions})
        definitions = [{"name": "Choice", "kind": "enumeration", "values": ["a-b", "a_b"]}]
        with self.assertRaisesRegex(g.DescriptionError, "variant collision"):
            g.lower_values({"declarations": definitions})

    @unittest.skipUnless(RUSTC, "standalone compiler check requires explicit --rustc")
    def test_typed_values_preserve_presence_defaults_and_enum_identity(self):
        # Test-only IDL-shaped declarations exercise the lowering semantics
        # absent from the current brand-free production subset. They declare
        # no platform interfaces and make no native execution claim.
        model = g.lower_values({"declarations": [
            {"name": "Mode", "kind": "enumeration", "values": ["on", "off", "", "self", "2d"]},
            {"name": "Base", "kind": "dictionary", "members": [
                {"name": "enabled", "type": {"kind": "named", "name": "boolean"}, "required": False,
                 "default": {"kind": "boolean", "value": True}}]},
            {"name": "Options", "kind": "dictionary", "inherits": "Base", "members": [
                {"name": "capture", "type": {"kind": "named", "name": "boolean"}, "required": False},
                {"name": "self", "type": {"kind": "named", "name": "boolean"}, "required": False},
                {"name": "mode", "type": {"kind": "named", "name": "Mode"}, "required": False,
                 "default": {"kind": "string", "value": "off"}}]},
            {"name": "Required", "kind": "dictionary", "members": [
                {"name": "flag", "type": {"kind": "named", "name": "boolean"}, "required": True}]},
        ]})
        source = '''
pub mod values;
fn main() {
    let mut options = values::Options::default();
    assert!(options.enabled);
    assert_eq!(options.capture, None);
    assert_eq!(options.mode, values::Mode::Off);
    assert_eq!(options.self_value, None);
    options.capture = Some(false);
    assert_eq!(options.capture, Some(false));
    assert_eq!(values::Mode::from_code_units(&[111, 110]), Some(values::Mode::On));
    assert_eq!(values::Mode::from_code_units(&[0xd800]), None);
    assert_eq!(values::Mode::Off.as_code_units(), &[111, 102, 102]);
    assert_eq!(values::Mode::Empty.as_code_units(), &[]);
    assert_eq!(values::Mode::from_code_units(&[115, 101, 108, 102]), Some(values::Mode::SelfValue));
    assert_eq!(values::Mode::Value2d.as_code_units(), &[50, 100]);
    assert!(values::Required { flag: true }.flag);
}
'''
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "values.rs").write_text(g.emit_values(model))
            path = root / "consumer.rs"
            output = root / "consumer"
            path.write_text(source)
            for command in (["rustc", "--edition=2024", "-F", "unsafe-code", "-D", "warnings", str(path), "-o", str(output)], [str(output)]):
                print("\nstandalone command: " + " ".join(command), file=sys.stderr)
                result = subprocess.run(command, capture_output=True, text=True, timeout=60)
                print(result.stdout, end="", file=sys.stdout)
                print(result.stderr, end="", file=sys.stderr)
                self.assertEqual(result.returncode, 0, result.stderr)
                print(f"standalone exit: {result.returncode}", file=sys.stderr)


if __name__ == "__main__":
    unittest.main(verbosity=2)
