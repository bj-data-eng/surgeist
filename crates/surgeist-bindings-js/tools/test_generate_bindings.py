#!/usr/bin/env python3
"""Functional tests for the declared source/artifact contract, not live bindings.

Run with Python -B. --rustc additionally compiles and executes a temporary,
standalone consumer of the actual shared model/generated records; it uses no
Cargo, V8, downloads, or repository target directory. Run through pisct run.
"""

import copy
import json
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

    def plans(self):
        return {p["name"]: p for p in g.interface_plans(self.data)}

    def rejected(self, message):
        with self.assertRaisesRegex(g.DescriptionError, message):
            g.generate(self.data)

    def test_actual_retained_blocks_and_generated_bytes_correspond(self):
        # This is an explicit generated-artifact contract. Source token equality
        # is the oracle, not a copy of today's generated Rust as a semantic test.
        self.assertEqual(g.generate(self.data), g.OUTPUT.read_text(encoding="utf-8"))

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

    def test_adapter_names_must_emit_distinct_valid_rust_variants(self):
        self.data["access_mappings"]["Node.nodeName/get"]["adapter"] = "self"
        self.rejected("invalid Rust enum variant")
        self.setUp()
        self.data["access_mappings"]["Node.nodeName/get"]["adapter"] = "audit.a_b"
        self.data["access_mappings"]["Node.nodeType/get"]["adapter"] = "audit_a.b"
        self.rejected("adapter Rust identifier collision")

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

    def test_all_six_dom_mixins_are_composed_without_mixin_globals(self):
        plans = self.plans()
        for name in ("NonElementParentNode", "ParentNode", "NonDocumentTypeChildNode", "ChildNode", "Slottable", "DocumentOrShadowRoot"):
            self.assertNotIn(name, plans)
        self.assertIn("ParentNode.children", plans["Document"]["members"])
        self.assertIn("ParentNode.children", plans["DocumentFragment"]["members"])
        self.assertIn("ParentNode.children", plans["Element"]["members"])
        self.assertIn("NonElementParentNode.getElementById:0", plans["Document"]["members"])
        self.assertNotIn("NonElementParentNode.getElementById:0", plans["Element"]["members"])
        # Inherited members stay on the base prototype, not copied to Text.
        self.assertNotIn("Node.nodeName", plans["Text"]["members"])
        self.assertEqual(plans["Text"]["parent"], "CharacterData")

    def test_selected_pi_constructor_and_factory_are_both_described(self):
        pi = self.member("ProcessingInstruction.constructor:0")
        self.assertEqual(pi["arguments"][0]["name"], "target")
        self.assertEqual(pi["arguments"][1]["default"], {"kind": "string", "value": ""})
        self.assertEqual(self.member("Document.createProcessingInstruction:0")["type"]["name"], "ProcessingInstruction")
        contracts = self.data["access_mappings"][pi["id"] + "/construct"]["requirements"]
        self.assertTrue(any(r.get("issue") == 2259 and "associated Document" in r["contract"] for r in contracts))
        self.assertTrue(self.data["access_mappings"]["ProcessingInstruction.setAttribute:0/call"]["requirements"])

    def test_overloads_form_one_property_and_preserve_distinct_calls(self):
        open_property = next(p for p in self.plans()["Document"]["properties"] if p["name"] == "open")
        self.assertEqual(open_property["declarations"], ["Document.html.open:0", "Document.html.open:1"])
        self.assertEqual(open_property["length"], 0)
        self.assertEqual(self.member("Document.html.open:1")["type"]["kind"], "nullable")
        self.assertIn("Document.html.open:0/call", self.data["access_mappings"])
        self.assertIn("Document.html.open:1/call", self.data["access_mappings"])
        self.assertEqual(self.data["access_mappings"]["Document.html.open:0/call"]["kind"], "native")
        self.assertEqual(self.data["access_mappings"]["Document.html.open:1/call"]["kind"], "host")

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

    def test_mapping_variants_preserve_actual_operation_roles(self):
        mappings = self.data["access_mappings"]
        location = mappings["Document.html.location/set"]
        self.assertEqual(location["kind"], "forwarding")
        self.assertEqual(location["getter"], "Document.html.location/get")
        self.assertEqual(location["target"], "href")
        self.assertNotIn("adapter", location)
        listener = mappings["EventListener.handleEvent:0/call"]
        self.assertEqual((listener["kind"], listener["interface"], listener["operation"]),
                         ("callback", "EventListener", "handleEvent"))
        self.assertTrue(listener["source"].endswith("#call-a-user-objects-operation"))
        self.assertNotIn("adapter", listener)
        self.assertNotIn("EventListener", self.plans())
        create_range = mappings["Document.createRange:0/call"]
        self.assertEqual(create_range["kind"], "native")
        self.assertEqual(create_range["producers"][0]["symbol"], "create_range")
        lang = mappings["HTMLElement.lang/set"]
        self.assertEqual(lang["kind"], "derived")
        self.assertTrue(any(p["symbol"] == "set_attribute" for p in lang["inputs"]))
        self.assertEqual(lang["requirements"], [])
        self.assertEqual(mappings["Document.html.images/get"]["kind"], "derived")
        self.assertEqual(mappings["Document.html.body/set"]["kind"], "derived")
        self.assertEqual(mappings["HTMLElement.innerText/get"]["kind"], "host")
        inner_set = mappings["HTMLElement.innerText/set"]
        self.assertEqual(inner_set["kind"], "native")
        self.assertTrue(all(r["owner"] == "surgeist-dom" for r in inner_set["requirements"]))
        for access in ("Document.html.location/get", "Document.html.cookie/set", "Document.html.open:1/call"):
            self.assertEqual(mappings[access]["kind"], "host")
            self.assertNotEqual(mappings[access]["requirement"].get("issue"), 2259)

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

    def test_descriptors_unforgeables_putforwards_and_nonconstructible_interfaces(self):
        plans = self.plans()
        child = next(p for p in plans["Node"]["properties"] if p["name"] == "childNodes")
        self.assertFalse(child["setter"])
        self.assertTrue(child["enumerable"])
        self.assertTrue(child["configurable"])
        location = next(p for p in plans["Document"]["properties"] if p["name"] == "location")
        self.assertEqual(location["placement"], "instance")
        self.assertTrue(location["setter"])
        self.assertFalse(location["configurable"])
        self.assertEqual(plans["Node"]["constructors"], [])
        self.assertEqual(plans["Document"]["constructors"], ["Document.constructor:0"])
        self.assertEqual(plans["ProcessingInstruction"]["constructor_length"], 1)
        self.assertEqual(plans["Text"]["constructor_length"], 0)
        slot = next(p for p in plans["Element"]["properties"] if p["name"] == "slot")
        self.assertTrue(slot["unscopable"])

    def test_exposure_constants_and_required_nullable_arguments_keep_their_roles(self):
        plans = self.plans()
        self.assertEqual(plans["Node"]["exposure"], "window")
        self.assertEqual(plans["EventTarget"]["exposure"], "all")
        constant = next(p for p in plans["Node"]["properties"] if p["name"] == "CDATA_SECTION_NODE")
        self.assertEqual(constant["placement"], "interface_and_prototype")
        self.assertFalse(constant["writable"])
        self.assertTrue(constant["enumerable"])
        self.assertFalse(constant["configurable"])
        method = next(p for p in plans["Node"]["properties"] if p["name"] == "insertBefore")
        self.assertEqual(method["length"], 2)
        child = self.member("Node.insertBefore:0")["arguments"][1]
        self.assertEqual(child["presence"], "required")
        self.assertEqual(child["type"]["kind"], "nullable")

    def test_collections_and_inherited_legacy_properties_are_not_plain_prototypes(self):
        plans = self.plans()
        self.assertEqual(plans["NodeList"]["indexed"], "NodeList.item:0")
        self.assertTrue(plans["NodeList"]["value_iterable"])
        self.assertFalse(plans["HTMLCollection"]["value_iterable"])
        self.assertEqual(plans["HTMLCollection"]["named"], "HTMLCollection.namedItem:0")
        self.assertTrue(plans["HTMLCollection"]["unenumerable_names"])
        self.assertTrue(plans["NamedNodeMap"]["unenumerable_names"])
        self.assertTrue(plans["XMLDocument"]["override_builtins"])
        self.assertEqual(plans["XMLDocument"]["named"], "Document.html.operation:0")
        mappings = self.data["access_mappings"]
        self.assertEqual(mappings["Node.childNodes/get"]["result_view"], "live")
        self.assertEqual(mappings["ParentNode.querySelectorAll:0/call"]["result_view"], "static")
        self.assertEqual(mappings["Element.getAttributeNames:0/call"]["result_view"], "sequence")

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
        before = g.OUTPUT.read_bytes()
        with tempfile.TemporaryDirectory() as directory:
            result = subprocess.run([sys.executable, "-B", str(Path(g.__file__)), "--check"], cwd=directory, capture_output=True, text=True, timeout=30)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(g.OUTPUT.read_bytes(), before)
        self.assertEqual(g.generate(self.data), g.generate(copy.deepcopy(self.data)))

    @unittest.skipUnless(RUSTC, "standalone compiler check requires explicit --rustc")
    def test_generated_records_compile_and_feed_a_standalone_consumer(self):
        # No dead-code/lint suppression is added to the product or fixture.
        # Unwired internal records can warn; this verifies their type contract.
        directory = g.OUTPUT.parent
        model = json.dumps(str(directory / "model.rs"))
        generated = json.dumps(str(g.OUTPUT))
        source = f'''mod description {{
#[path = {model}] pub(super) mod model;
#[path = {generated}] pub(super) mod dom_core;
pub fn verify() {{
    use dom_core::*;
    let node = INTERFACE_PLANS.iter().find(|p| p.name == "Node").unwrap();
    assert_eq!(node.parent, Some("EventTarget"));
    assert!(node.constructors.is_empty());
    let doc = INTERFACE_PLANS.iter().find(|p| p.name == "Document").unwrap();
    assert_eq!(doc.constructors, &["Document.constructor:0"]);
    let xml = INTERFACE_PLANS.iter().find(|p| p.name == "XMLDocument").unwrap();
    assert!(xml.legacy.override_builtins);
    assert_eq!(xml.legacy.named_getter, Some("Document.html.operation:0"));
    assert!(!model::INTERFACE_OBJECT_PROPERTIES.prototype.writable);
    assert!(ACCESS_MAPPINGS.iter().any(|m| m.access == "ProcessingInstruction.constructor:0/construct"));
    let forward = ACCESS_MAPPINGS.iter().find(|m| m.access == "Document.html.location/set").unwrap();
    assert!(matches!(forward.operation, model::AccessOperation::PutForwards {{ getter: "Document.html.location/get", target: "href", .. }}));
    let callback = ACCESS_MAPPINGS.iter().find(|m| m.access == "EventListener.handleEvent:0/call").unwrap();
    assert!(matches!(callback.operation, model::AccessOperation::CallbackInvocation {{ interface: "EventListener", operation: "handleEvent", .. }}));
    let cookies = ACCESS_MAPPINGS.iter().find(|m| m.access == "Document.html.cookie/set").unwrap();
    assert!(matches!(cookies.operation, model::AccessOperation::ExternalHost {{ .. }}));
    assert!(!DECLARATIONS.is_empty() && !SOURCES.is_empty() && !INCLUDES.is_empty() && !EXTERNALS.is_empty() && !SEMANTICS.is_empty());
}}
}}
fn main() {{ description::verify(); }}
'''
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / "consumer.rs"
            output = root / "consumer"
            path.write_text(source)
            command = ["rustc", "--edition=2024", "-F", "unsafe-code", "--error-format=json", str(path), "-o", str(output)]
            print("\nstandalone compile command: " + " ".join(command), file=sys.stderr)
            result = subprocess.run(command, capture_output=True, text=True, timeout=60)
            # Preserve raw subprocess output in the enclosing process log,
            # including successful diagnostics; a code summary is not raw evidence.
            print(result.stdout, end="", file=sys.stdout)
            print(result.stderr, end="", file=sys.stderr)
            self.assertEqual(result.returncode, 0, result.stderr)
            diagnostics = [json.loads(line) for line in result.stderr.splitlines()]
            codes = sorted({d["code"]["code"] for d in diagnostics if d.get("code")})
            print(f"standalone compile exit: {result.returncode}; diagnostic codes: {codes}", file=sys.stderr)
            print("standalone consumer command: " + str(output), file=sys.stderr)
            result = subprocess.run([str(output)], capture_output=True, text=True, timeout=30)
            print(result.stdout, end="", file=sys.stdout)
            print(result.stderr, end="", file=sys.stderr)
            self.assertEqual(result.returncode, 0, result.stderr)
            print(f"standalone consumer exit: {result.returncode}", file=sys.stderr)


if __name__ == "__main__":
    unittest.main(verbosity=2)
