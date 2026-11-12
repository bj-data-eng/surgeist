#!/usr/bin/env python3
"""Functional tests for strict declaration validation and typed value lowering.

Run with Python -B. --rustc additionally compiles and executes a temporary,
standalone consumer of the generated typed values; it uses no
Cargo, V8, downloads, or repository target directory. Run through pisct run.
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
            lambda d: d["access_mappings"]["ProcessingInstruction.constructor:0/construct"]["requirements"][0].update(issue=True),
            lambda d: d["access_mappings"]["ProcessingInstruction.constructor:0/construct"]["requirements"][0].update(issue=2**32),
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


    def test_selected_pi_constructor_and_factory_are_both_described(self):
        pi = self.member("ProcessingInstruction.constructor:0")
        self.assertEqual(pi["arguments"][0]["name"], "target")
        self.assertEqual(pi["arguments"][1]["default"], {"kind": "string", "value": ""})
        self.assertEqual(self.member("Document.createProcessingInstruction:0")["type"]["name"], "ProcessingInstruction")
        contracts = self.data["access_mappings"][pi["id"] + "/construct"]["requirements"]
        self.assertTrue(any(r.get("issue") == 2259 and "associated Document" in r["contract"] for r in contracts))
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
            ("Document.html.cookie/set", "adapter", "document.cookie.set"),
            ("Document.html.cookie/set", "requirement", []),
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
                         ["GetRootNodeOptions", "ShadowRootInit", "EventInit", "EventListenerOptions"])
        self.assertEqual([d["name"] for d in model["enumerations"]],
                         ["ShadowRootMode", "SlotAssignmentMode", "DocumentReadyState", "DocumentVisibilityState"])
        # No member may be discarded. Signal, native registry and source
        # combinations outside the selected conversion profiles stay unavailable.
        for name in ("AddEventListenerOptions", "ElementCreationOptions",
                     "ImportNodeOptions", "ShowPopoverOptions", "TogglePopoverOptions"):
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
