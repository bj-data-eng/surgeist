#!/usr/bin/env python3
"""Validate the source-qualified description and emit typed IDL values.

Run --check or --write explicitly. No downloads, build-time generation, native
algorithms or runtime handlers are supplied. The authored JSON is the only
binding description; retained standard blocks are its independent source oracle.
Only the selected declaration grammar is rendered, not arbitrary Web IDL parsed.
WHATWG-derived declarations retain licenses/whatwg/LICENSE and NOTICE.md.
"""

import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import re
import sys
import textwrap
from urllib.parse import urlsplit


ROOT = Path(__file__).resolve().parents[3]
DESCRIPTION = ROOT / "crates/surgeist-bindings-js/bindings/dom-core.json"
VALUE_OUTPUT = ROOT / "crates/surgeist-bindings-js/src/declarations.rs"
CONVERSION_OUTPUT = ROOT / "crates/surgeist-bindings-js/src/conversion/declarations.rs"
DOM_EXCEPTION_OUTPUT = ROOT / "crates/surgeist-bindings-js/src/dom_exception/declarations.rs"
IDENTIFIER = re.compile(r"[A-Za-z_][A-Za-z_0-9]*\Z")
PRIMITIVES = {"undefined", "boolean", "byte", "octet", "short", "unsigned short",
              "long", "unsigned long", "long long", "unsigned long long", "float",
              "unrestricted float", "double", "unrestricted double", "DOMString",
              "USVString", "ByteString", "object", "any"}
ATTRIBUTES = {"Exposed", "Serializable", "SameObject", "NewObject", "CEReactions", "Unscopable",
              "PutForwards", "LegacyNullToEmptyString", "LegacyUnenumerableNamedProperties",
              "LegacyUnforgeable", "LegacyOverrideBuiltIns", "LegacyLenientThis",
              "HTMLConstructor", "Reflect", "ReflectSetter", "ReflectRange"}
TOKEN = re.compile(r'\s+|//[^\n]*|/\*[\s\S]*?\*/|"(?:\\.|[^"\\])*"|'
                   r'0[xX][0-9A-Fa-f]+|[A-Za-z_][A-Za-z_0-9]*|[0-9]+|\.\.\.|[^\w\s]', re.ASCII)


class DescriptionError(ValueError):
    """An authored description or its retained source is inconsistent."""


def require(condition, message):
    if not condition:
        raise DescriptionError(message)


def keys(value, required, optional=()):
    require(isinstance(value, dict), "expected an object")
    require(set(required) <= value.keys(), f"missing fields: {set(required) - value.keys()}")
    require(value.keys() <= set(required) | set(optional),
            f"unknown fields: {value.keys() - set(required) - set(optional)}")


def identifier(value):
    require(type(value) is str and IDENTIFIER.fullmatch(value), f"invalid identifier: {value!r}")


def text(value, label, empty=False):
    require(type(value) is str and value.isascii() and
            (empty or bool(value.strip())) and
            all(ord(c) >= 32 and ord(c) != 127 for c in value),
            f"{label} must be {'an ASCII string' if empty else 'a nonempty ASCII string'}")


def boolean(value, label):
    require(type(value) is bool, f"{label} must be boolean")


def integer(value, label, minimum=-(2**63), maximum=2**63 - 1):
    require(type(value) is int and minimum <= value <= maximum,
            f"{label} must be an integer in [{minimum}, {maximum}]")


def identity(value, label):
    text(value, label)
    require(re.fullmatch(r"[A-Za-z_][A-Za-z_0-9]*(?:[.:][A-Za-z_0-9]+)*", value),
            f"{label} has invalid identity syntax")


def relative_path(value, label):
    text(value, label)
    path = PurePosixPath(value)
    require(not path.is_absolute() and ".." not in path.parts and
            path.as_posix() == value and value != ".", f"{label} must be a normalized repository-relative path")


def owner(value, label):
    text(value, label)
    require(re.fullmatch(r"[a-z][a-z0-9]*(?:-[a-z0-9]+)*", value),
            f"{label} must identify one crate/semantic owner")


def sequence(value, label, nonempty=False):
    require(type(value) is list and (not nonempty or len(value) > 0),
            f"{label} must be {'a nonempty array' if nonempty else 'an array'}")


def url(value, label):
    text(value, label)
    try:
        parsed = urlsplit(value)
        host = parsed.hostname
    except ValueError as error:
        raise DescriptionError(f"{label} is not a valid URL") from error
    require(parsed.scheme == "https" and host and parsed.username is None
            and parsed.password is None and not any(c.isspace() for c in value),
            f"{label} must be an absolute HTTPS source URL")


def attributes_shape(items):
    sequence(items, "extended attributes")
    for item in items:
        keys(item, ["name"], ["value"])
        text(item["name"], "extended attribute name")
        if item["name"] in {"Exposed", "PutForwards"}:
            text(item.get("value"), "extended attribute value")
        elif item["name"] == "ReflectRange":
            sequence(item.get("value"), "reflection range", nonempty=True)
            require(len(item["value"]) == 2, "reflection range needs two integers")
            for number in item["value"]:
                integer(number, "reflection range bound")
        else:
            require("value" not in item, "valueless extended attribute cannot have a value field")


def type_shape(value):
    keys(value, ["kind"], ["name", "inner", "members", "attributes"])
    text(value["kind"], "type kind")
    if value["kind"] == "named":
        keys(value, ["kind", "name"], ["attributes"])
        text(value["name"], "type name")
        attributes_shape(value.get("attributes", []))
    elif value["kind"] in {"nullable", "sequence", "promise"}:
        keys(value, ["kind", "inner"])
        type_shape(value["inner"])
    elif value["kind"] == "union":
        keys(value, ["kind", "members"])
        sequence(value["members"], "union members", nonempty=True)
        for member in value["members"]:
            type_shape(member)
    else:
        raise DescriptionError("unknown type kind")


def default_shape(value):
    keys(value, ["kind"], ["value"])
    text(value["kind"], "default kind")
    kind = value["kind"]
    if kind in {"null", "empty_dictionary"}:
        keys(value, ["kind"])
    else:
        keys(value, ["kind", "value"])
        if kind == "boolean":
            boolean(value["value"], "default value")
        elif kind == "integer":
            integer(value["value"], "default value")
        elif kind == "string":
            text(value["value"], "default value", empty=True)
        else:
            raise DescriptionError("unknown default kind")


def member_shape(member):
    keys(member, ["id", "kind", "attributes"],
         ["name", "type", "readonly", "arguments", "special", "static", "value", "default", "required"])
    identity(member["id"], "member ID")
    text(member["kind"], "member kind")
    attributes_shape(member["attributes"])
    if "name" in member:
        identifier(member["name"])
    if "type" in member:
        type_shape(member["type"])
    for flag in ("readonly", "static", "required"):
        if flag in member:
            boolean(member[flag], flag)
    if "special" in member:
        text(member["special"], "special operation kind")
    if "value" in member:
        integer(member["value"], "constant value")
    if "default" in member:
        default_shape(member["default"])
    if "arguments" in member:
        sequence(member["arguments"], "arguments")
        for arg in member["arguments"]:
            keys(arg, ["name", "type", "presence"], ["default"])
            identifier(arg["name"])
            type_shape(arg["type"])
            text(arg["presence"], "argument presence")
            if "default" in arg:
                default_shape(arg["default"])


def producer_shape(producer):
    keys(producer, ["owner", "path", "symbol"])
    owner(producer["owner"], "producer owner")
    relative_path(producer["path"], "producer path")
    text(producer["symbol"], "producer symbol")
    identifier(producer["symbol"])
    require(producer["path"].startswith("crates/" + producer["owner"] + "/"),
            "producer path does not belong to its declared owner")


def requirement_shape(req):
    keys(req, ["owner", "contract", "source"], ["issue"])
    owner(req["owner"], "requirement owner")
    text(req["contract"], "requirement contract")
    url(req["source"], "requirement source")
    if "issue" in req:
        integer(req["issue"], "requirement issue", 1, 2**32 - 1)


def mapping_shape(mapping):
    require(type(mapping) is dict, "access mapping must be an object")
    text(mapping.get("kind"), "mapping kind")
    kind = mapping["kind"]
    common = {"kind"}
    optional = {"result_view"}
    if kind == "native":
        keys(mapping, common | {"adapter", "producers", "requirements"}, optional)
        text(mapping["adapter"], "adapter ID")
    elif kind == "derived":
        keys(mapping, common | {"algorithm", "owner", "contract", "source", "inputs", "requirements"}, optional)
        for field in ("algorithm", "owner", "contract"):
            text(mapping[field], "derived " + field)
        owner(mapping["owner"], "derived owner")
        url(mapping["source"], "derived source")
    elif kind == "callback":
        keys(mapping, common | {"interface", "operation", "source", "requirements"}, optional)
        identifier(mapping["interface"])
        identifier(mapping["operation"])
        url(mapping["source"], "callback invocation source")
    elif kind == "forwarding":
        keys(mapping, common | {"getter", "target", "source"}, optional)
        text(mapping["getter"], "forwarding getter")
        identifier(mapping["target"])
        url(mapping["source"], "forwarding source")
    elif kind == "host":
        keys(mapping, common | {"requirement"}, optional)
        requirement_shape(mapping["requirement"])
    else:
        raise DescriptionError("unknown access mapping kind")
    if "result_view" in mapping:
        text(mapping["result_view"], "result projection")
    for field in ("inputs", "producers"):
        if field in mapping:
            sequence(mapping[field], "mapping " + field)
            for producer in mapping[field]:
                producer_shape(producer)
    if "requirements" in mapping:
        sequence(mapping["requirements"], "mapping requirements")
        for req in mapping["requirements"]:
            requirement_shape(req)


def document_shape(data):
    """Reject coercible/falsey JSON values before any rendering or file access."""
    keys(data, ["schema", "sources", "declarations", "includes", "externals", "access_mappings", "semantics"])
    integer(data["schema"], "schema", 1, 1)
    for name in ("sources", "declarations", "includes", "externals"):
        sequence(data[name], name)
    for d in data["declarations"]:
        keys(d, ["id", "kind", "name", "attributes"], ["partial", "inherits", "members", "type", "values"])
        identity(d["id"], "declaration ID")
        text(d["kind"], "declaration kind")
        identifier(d["name"])
        attributes_shape(d["attributes"])
        if "partial" in d:
            boolean(d["partial"], "partial")
        if "inherits" in d:
            identifier(d["inherits"])
        if "type" in d:
            type_shape(d["type"])
        if "members" in d:
            sequence(d["members"], "members")
            for member in d["members"]:
                member_shape(member)
        if "values" in d:
            sequence(d["values"], "enum values", nonempty=True)
            for value in d["values"]:
                text(value, "enum value", empty=True)
    for edge in data["includes"]:
        keys(edge, ["id", "interface", "mixin"])
        identity(edge["id"], "includes ID")
        identifier(edge["interface"])
        identifier(edge["mixin"])
    for e in data["externals"]:
        keys(e, ["name", "kind", "owner", "url", "requirement"])
        identifier(e["name"])
        for field in ("kind", "owner", "requirement"):
            text(e[field], "external " + field)
        owner(e["owner"], "external owner")
        url(e["url"], "external source")
    for s in data["sources"]:
        keys(s, ["id", "path", "anchor", "block", "sha256", "revision", "url", "entries"],
             ["names_table_sha256"])
        if "names_table_sha256" in s:
            require(s["entries"] == ["DOMException"], "names table must accompany DOMException")
            text(s["names_table_sha256"], "names table digest")
            require(re.fullmatch(r"[0-9a-f]{64}", s["names_table_sha256"]), "invalid names table digest")
        for field in ("id", "path", "sha256", "revision"):
            text(s[field], "source " + field)
        identity(s["id"], "source ID")
        relative_path(s["path"], "source path")
        text(s["anchor"], "source anchor", empty=True)
        integer(s["block"], "source block", 0)
        require(re.fullmatch(r"[0-9a-f]{64}", s["sha256"]), "invalid source digest")
        require(re.fullmatch(r"[0-9a-f]{40}", s["revision"]), "invalid source revision")
        url(s["url"], "source URL")
        require("/" + s["revision"] + "/" in s["url"], "source URL does not identify the retained revision")
        sequence(s["entries"], "source entries", nonempty=True)
        for entry in s["entries"]:
            identity(entry, "source entry ID")
    require(type(data["access_mappings"]) is dict, "access mappings must be an object")
    for key, mapping in data["access_mappings"].items():
        text(key, "mapping access ID")
        mapping_shape(mapping)
    require(type(data["semantics"]) is dict, "semantics must be an object")
    for name, source in data["semantics"].items():
        text(name, "semantic source kind")
        url(source, "semantic source URL")


def tokens(text):
    """Ignore formatting/comments and equivalent integer spelling, not strings."""
    output = []
    position = 0
    for match in TOKEN.finditer(text):
        require(match.start() == position, "unrecognized source token")
        position = match.end()
        token = match.group()
        if not token.isspace() and not token.startswith(("//", "/*")):
            output.append(str(int(token, 16)) if re.fullmatch(r"0[xX][0-9A-Fa-f]+", token) else token)
    require(position == len(text), "unrecognized trailing source token")
    return output


def attributes(items):
    attributes_shape(items)
    values = []
    seen = set()
    for item in items:
        keys(item, ["name"], ["value"])
        name = item["name"]
        require(name in ATTRIBUTES and name not in seen, f"unknown/duplicate extended attribute: {name}")
        seen.add(name)
        value = item.get("value")
        if name in {"Exposed", "PutForwards", "ReflectRange"}:
            require(value is not None, f"{name} requires a value")
        else:
            require(value is None, f"{name} takes no value")
        if name == "Exposed":
            require(value in {"Window", "*"}, "unexpected selected exposure")
        elif name == "PutForwards":
            identifier(value)
        elif name == "ReflectRange":
            require(isinstance(value, list) and len(value) == 2 and
                    all(type(x) is int for x in value), "invalid reflection range")
        suffix = ""
        if isinstance(value, list):
            suffix = "=(" + ", ".join(map(str, value)) + ")"
        elif value is not None:
            suffix = "=" + value
        values.append(name + suffix)
    return "[" + ", ".join(values) + "] " if values else ""


def render_type(value, names):
    type_shape(value)
    keys(value, ["kind"], ["name", "inner", "members", "attributes"])
    kind = value["kind"]
    annotation = value.get("attributes", [])
    require(all(x["name"] == "LegacyNullToEmptyString" for x in annotation),
            "only the selected string annotation is a type attribute")
    prefix = attributes(annotation)
    if kind == "named":
        require(set(value) <= {"kind", "name", "attributes"}, "invalid named type fields")
        name = value.get("name")
        require(name in PRIMITIVES or name in names, f"unresolved type: {name}")
        require(names.get(name) not in {"mixin", "external_mixin", "namespace"}, f"non-type definition used as a type: {name}")
        if annotation:
            require(name in {"DOMString", "USVString"}, "legacy string annotation on non-string")
        return prefix + name
    require(not annotation, "annotation must be on its exact inner type")
    if kind in {"nullable", "sequence", "promise"}:
        require(set(value) == {"kind", "inner"}, f"invalid {kind} fields")
        inner = render_type(value["inner"], names)
        if kind == "nullable":
            require(value["inner"]["kind"] != "nullable", "nested nullable type")
            return inner + "?"
        return ("sequence" if kind == "sequence" else "Promise") + "<" + inner + ">"
    require(kind == "union" and set(value) == {"kind", "members"}, "invalid type form")
    require(len(value["members"]) >= 2, "union needs at least two types")
    members = [render_type(x, names) for x in value["members"]]
    require(len(set(members)) == len(members), "duplicate union member")
    return "(" + " or ".join(members) + ")"


def default(value):
    default_shape(value)
    keys(value, ["kind"], ["value"])
    kind = value["kind"]
    if kind in {"null", "empty_dictionary"}:
        require(set(value) == {"kind"}, "invalid null/dictionary default")
        return "null" if kind == "null" else "{}"
    require(set(value) == {"kind", "value"}, "default value is missing")
    v = value["value"]
    if kind == "boolean":
        require(type(v) is bool, "invalid boolean default")
        return "true" if v else "false"
    if kind == "string":
        require(isinstance(v, str), "invalid string default")
        return json.dumps(v, ensure_ascii=True)
    require(kind == "integer" and type(v) is int, "invalid default")
    return str(v)


def arguments(items, names):
    sequence(items, "arguments")
    result = []
    seen = set()
    optional_seen = False
    for index, arg in enumerate(items):
        keys(arg, ["name", "type", "presence"], ["default"])
        identifier(arg["name"])
        require(arg["name"] not in seen, "duplicate argument")
        seen.add(arg["name"])
        presence = arg["presence"]
        require(presence in {"required", "optional", "variadic"}, "invalid argument presence")
        require(not optional_seen or presence != "required", "required argument after optional")
        optional_seen |= presence == "optional"
        require(presence != "variadic" or index == len(items) - 1, "variadic argument must be last")
        require("default" not in arg or presence == "optional", "default on nonoptional argument")
        text = ("optional " if presence == "optional" else "") + render_type(arg["type"], names)
        text += ("..." if presence == "variadic" else "") + " " + arg["name"]
        if "default" in arg:
            text += " = " + default(arg["default"])
        result.append(text)
    return ", ".join(result)


def render_member(member, names):
    member_shape(member)
    keys(member, ["id", "kind", "attributes"],
         ["name", "type", "readonly", "arguments", "special", "static", "value", "default", "required"])
    kind = member["kind"]
    fields = {
        "constructor": ({"arguments"}, set()),
        "attribute": ({"name", "type", "readonly"}, set()),
        "operation": ({"type", "arguments"}, {"name", "static", "special"}),
        "constant": ({"name", "type", "value"}, set()),
        "iterable": ({"type"}, set()),
        "dictionary_member": ({"name", "type", "required"}, {"default"}),
    }
    require(kind in fields, f"unknown member kind: {kind}")
    required, optional = fields[kind]
    keys(member, {"id", "kind", "attributes"} | required, optional)
    annotations = {x["name"] for x in member["attributes"]}
    if "SameObject" in annotations or "PutForwards" in annotations:
        require(kind == "attribute" and member.get("readonly"), "SameObject/PutForwards requires a readonly attribute")
    require("NewObject" not in annotations or kind == "operation", "NewObject requires an operation")
    require("HTMLConstructor" not in annotations or kind == "constructor", "HTMLConstructor requires a constructor")
    require(not annotations & {"Exposed", "Serializable", "LegacyOverrideBuiltIns", "LegacyUnenumerableNamedProperties", "LegacyNullToEmptyString"}, "extended attribute on the wrong declaration scope")
    prefix = attributes(member["attributes"])
    if "name" in member:
        identifier(member["name"])
    if kind == "constructor":
        return prefix + "constructor(" + arguments(member["arguments"], names) + ");"
    if kind == "attribute":
        require(type(member["readonly"]) is bool, "readonly must be boolean")
        require(not any(a["name"] == "CEReactions" for a in member["attributes"]) or
                not member["readonly"], "CEReactions on readonly attribute")
        return prefix + ("readonly " if member["readonly"] else "") + "attribute " + render_type(member["type"], names) + " " + member["name"] + ";"
    if kind == "operation":
        require(type(member.get("static", False)) is bool, "static must be boolean")
        special = member.get("special")
        require(special in {None, "getter", "setter", "deleter"}, "unknown special operation")
        return prefix + ("static " if member.get("static", False) else "") + (special + " " if special else "") + render_type(member["type"], names) + (" " + member["name"] if member.get("name") else "") + "(" + arguments(member["arguments"], names) + ");"
    if kind == "constant":
        require(type(member["value"]) is int, "selected constants are integers")
        return prefix + "const " + render_type(member["type"], names) + " " + member["name"] + " = " + str(member["value"]) + ";"
    if kind == "iterable":
        return prefix + "iterable<" + render_type(member["type"], names) + ">;"
    require(kind == "dictionary_member", f"unknown member kind: {kind}")
    text = prefix + ("required " if member.get("required", False) else "") + render_type(member["type"], names) + " " + member["name"]
    require(not member.get("required", False) or "default" not in member, "required dictionary member has default")
    if "default" in member:
        text += " = " + default(member["default"])
    return text + ";"


def render_definition(definition, names):
    keys(definition, ["id", "kind", "name", "attributes"],
         ["inherits", "partial", "members", "values", "type"])
    identifier(definition["name"])
    if "partial" in definition:
        boolean(definition["partial"], "partial")
    kind = definition["kind"]
    require(kind in {"interface", "mixin", "callback_interface", "dictionary", "enumeration", "typedef", "namespace"}, "unknown definition kind")
    prefix = attributes(definition["attributes"])
    require(not definition.get("partial", False) or kind in {"interface", "mixin"}, "invalid partial definition")
    require(all(x["name"] in {"Exposed", "Serializable", "LegacyOverrideBuiltIns", "LegacyUnenumerableNamedProperties"} for x in definition["attributes"]), "member attribute on a definition")
    require(kind == "interface" or all(x["name"] != "Serializable" for x in definition["attributes"]),
            "Serializable requires an interface")
    if kind == "typedef":
        keys(definition, ["id", "kind", "name", "attributes", "type"])
        return prefix + "typedef " + render_type(definition["type"], names) + " " + definition["name"] + ";"
    if kind == "enumeration":
        keys(definition, ["id", "kind", "name", "attributes", "values"])
        require(all(isinstance(x, str) for x in definition["values"]), "enum values must be strings")
        require(len(set(definition["values"])) == len(definition["values"]), "duplicate enum value")
        return prefix + "enum " + definition["name"] + " { " + ", ".join(json.dumps(x) for x in definition["values"]) + " };"
    keys(definition, ["id", "kind", "name", "attributes", "members"], ["inherits", "partial"])
    if kind == "namespace":
        keys(definition, ["id", "kind", "name", "attributes", "members"])
        require(definition["attributes"] == [{"name": "Exposed", "value": "Window"}],
                "only Window namespace exposure is selected")
        require(all(member["kind"] == "operation" and member.get("name") and
                    not member.get("static", False) and "special" not in member and
                    not member["attributes"] and "arguments" in member and
                    all(argument["presence"] == "required" for argument in member["arguments"])
                    for member in definition["members"]),
                "only regular fixed-arity namespace operations are selected")
        require(len({m["name"] for m in definition["members"]}) == len(definition["members"]),
                "namespace overloads are outside the selected grammar")
    words = {"mixin": "interface mixin", "callback_interface": "callback interface"}
    header = prefix + ("partial " if definition.get("partial", False) else "") + words.get(kind, kind) + " " + definition["name"]
    if definition.get("inherits"):
        parent = definition["inherits"]
        require(parent in names, f"unresolved parent: {parent}")
        require(names[parent] == kind or (kind == "interface" and names[parent] == "external_interface"), "invalid inheritance kind")
        header += " : " + parent
    return header + " {\n" + "\n".join("  " + render_member(m, names) for m in definition["members"]) + "\n};"


def mapping_keys(member):
    if member["kind"] == "constructor":
        return [member["id"] + "/construct"]
    if member["kind"] == "operation":
        return [member["id"] + "/call"]
    if member["kind"] == "attribute":
        result = [member["id"] + "/get"]
        if not member["readonly"] or any(x["name"] == "PutForwards" for x in member["attributes"]):
            result.append(member["id"] + "/set")
        return result
    return []


def source_block(unit, root):
    keys(unit, ["id", "path", "anchor", "block", "sha256", "revision", "url", "entries"],
         ["names_table_sha256"])
    path = (root / unit["path"]).resolve()
    require(path.is_relative_to(root.resolve()) and path.is_file(), "source path outside repository or missing")
    text = path.read_text(encoding="utf-8")
    anchor = re.search(r'(?:id=["\']' + re.escape(unit["anchor"]) + r'["\'])', text) if unit["anchor"] else None
    require(not unit["anchor"] or anchor is not None, f"source anchor missing: {unit['id']}")
    blocks = list(re.finditer(r"^```[^\n]*\n([\s\S]*?)^```", text[anchor.end() if anchor else 0:], re.MULTILINE))
    require(0 <= unit["block"] < len(blocks), "source block missing")
    block = blocks[unit["block"]].group(1)
    require(hashlib.sha256(block.encode()).hexdigest() == unit["sha256"], f"source digest mismatch: {unit['id']}")
    require(unit["revision"] in text, f"source revision not retained: {unit['id']}")
    return block


def validate(data, root=ROOT):
    document_shape(data)
    keys(data, ["schema", "sources", "declarations", "includes", "externals", "access_mappings", "semantics"])
    require(data["schema"] == 1, "unknown schema edition")
    names = {}
    definitions = {}
    members = {}
    for d in data["declarations"]:
        require(d["id"] not in definitions, "duplicate declaration ID")
        definitions[d["id"]] = d
        if not d.get("partial", False):
            require(d["name"] not in names, "duplicate definition")
            names[d["name"]] = d["kind"]
        ordinals = {}
        for m in d.get("members", []):
            base = d["id"] + "." + m.get("name", m["kind"])
            if m["kind"] in {"constructor", "operation"}:
                ordinal = ordinals.get(base, 0)
                ordinals[base] = ordinal + 1
                base += ":" + str(ordinal)
            require(m["id"] == base, "member/overload ID does not match its declaration")
            require(m["id"] not in members, "duplicate member ID")
            members[m["id"]] = m
    for external in data["externals"]:
        keys(external, ["name", "kind", "owner", "url", "requirement"])
        identifier(external["name"])
        require(external["name"] not in names, "duplicate external definition")
        require(external["kind"] in {"interface", "mixin", "callback", "typedef", "dictionary", "type"}, "invalid external kind")
        require(external["owner"] and external["requirement"] and external["url"].startswith("https://"), "incomplete external requirement")
        names[external["name"]] = "external_" + external["kind"]
    rendered = {k: render_definition(v, names) for k, v in definitions.items()}
    includes = {}
    edges = set()
    for inc in data["includes"]:
        keys(inc, ["id", "interface", "mixin"])
        require(inc["id"] not in definitions and inc["id"] not in includes, "duplicate includes ID")
        require(names.get(inc["interface"]) == "interface", "includes target is not an interface")
        require(names.get(inc["mixin"]) in {"mixin", "external_mixin"}, "includes source is not a mixin")
        edge = (inc["interface"], inc["mixin"])
        require(edge not in edges, "duplicate includes relation")
        edges.add(edge)
        includes[inc["id"]] = inc
        rendered[inc["id"]] = inc["interface"] + " includes " + inc["mixin"] + ";"
    for d in definitions.values():
        if d.get("partial", False):
            require(names.get(d["name"]) == d["kind"], "partial definition lacks its primary definition")
    parents = {d["name"]: d.get("inherits") for d in definitions.values() if not d.get("partial", False)}
    for name in parents:
        seen = set()
        cursor = name
        while cursor in parents:
            require(cursor not in seen, "inheritance cycle")
            seen.add(cursor)
            cursor = parents[cursor]
    covered = []
    source_ids = set()
    for unit in data["sources"]:
        require(unit["id"] not in source_ids, "duplicate source unit")
        source_ids.add(unit["id"])
        require(all(x in rendered for x in unit["entries"]), "unknown source entry")
        require(tokens(source_block(unit, root)) == tokens("\n".join(rendered[x] for x in unit["entries"])),
                f"source declaration mismatch: {unit['id']}")
        covered.extend(unit["entries"])
    require(len(covered) == len(set(covered)) and set(covered) == set(rendered), "source coverage is not exactly once")
    expected = {key for member in members.values() for key in mapping_keys(member)}
    require(set(data["access_mappings"]) == expected, "access mappings must cover each accessor/constructor/overload exactly")
    callback_members = {m["id"] for d in definitions.values() if d["kind"] == "callback_interface" for m in d["members"]}
    for key, mapping in data["access_mappings"].items():
        require(mapping.get("result_view", "value") in {"value", "live", "static", "sequence"}, "invalid result projection")
        if mapping["kind"] == "native":
            require(re.fullmatch(r"[a-z][a-z0-9_.]*", mapping["adapter"]), "invalid logical adapter ID")
            require(mapping["producers"] or mapping["requirements"], f"mapping lacks a producer or requirement: {key}")
        elif mapping["kind"] == "derived":
            require(re.fullmatch(r"[a-z][a-z0-9_.]*", mapping["algorithm"]), "invalid derived algorithm ID")
        member_id, access = key.rsplit("/", 1)
        member = members[member_id]
        forwarding = next((a["value"] for a in member["attributes"] if a["name"] == "PutForwards"), None)
        if access == "set" and forwarding:
            require(mapping["kind"] == "forwarding", "PutForwards setter must use guest forwarding")
            require(mapping["getter"] == member_id + "/get" and mapping["target"] == forwarding,
                    "forwarding does not match the declared getter/target")
        else:
            require(mapping["kind"] != "forwarding", "forwarding without a declared PutForwards setter")
        if member_id in callback_members:
            require(mapping["kind"] == "callback" and mapping["operation"] == member["name"],
                    "callback-interface operation requires the guest invocation protocol")
            require(any(d["name"] == mapping["interface"] and d["kind"] == "callback_interface" and member in d["members"] for d in definitions.values()), "wrong callback interface")
        else:
            require(mapping["kind"] != "callback", "callback mapping on a non-callback interface")
        for producer in mapping.get("producers", mapping.get("inputs", [])):
            keys(producer, ["owner", "path", "symbol"])
            path = (root / producer["path"]).resolve()
            require(path.is_relative_to(root.resolve()) and path.is_file(), "native producer path missing")
            require(re.search(r"\b" + re.escape(producer["symbol"]) + r"\b", path.read_text()), "native producer symbol missing")
    required_semantics = {"interface_objects", "namespace_objects", "property_descriptors", "brands", "argument_conversion",
                          "legacy_platform_objects", "iterable", "same_object", "new_object", "unscopables"}
    require(set(data["semantics"]) == required_semantics, "shared installation semantics are incomplete")
    require(all(x.startswith("https://") for x in data["semantics"].values()), "invalid semantics source")
    return definitions, names


def variant(name):
    result = "".join(part[:1].upper() + part[1:] for part in re.split(r"[^A-Za-z0-9]+", name))
    if not result:
        result = "Empty"
    elif result == "Self":
        result = "SelfValue"
    elif result[0].isdigit():
        result = "Value" + result
    require(IDENTIFIER.fullmatch(result), "invalid Rust enum variant")
    return result


def field_name(name):
    # Keep acronym runs together, splitting before the following word and
    # between a lowercase/digit and a capital (DOMImplementation -> dom_implementation).
    result = re.sub(r"([A-Z]+)([A-Z][a-z])", r"\1_\2", name)
    result = re.sub(r"([a-z0-9])([A-Z])", r"\1_\2", result).lower()
    require(IDENTIFIER.fullmatch(result), "invalid Rust field name")
    if result in {"self", "super", "crate"}:
        return result + "_value"
    keywords = {"as", "async", "await", "break", "const", "continue", "crate", "dyn", "else",
                "enum", "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop",
                "match", "mod", "move", "mut", "pub", "ref", "return", "self", "static",
                "struct", "super", "trait", "true", "type", "unsafe", "use", "where", "while"}
    return "r#" + result if result in keywords else result


def null_registry_member(member):
    return (member.get("id") in {"ShadowRootInit.customElementRegistry", "ElementCreationOptions.customElementRegistry"}
            and member["type"] == {"kind": "nullable", "inner": {"kind": "named", "name": "CustomElementRegistry"}}
            and not member["required"] and "default" not in member)


def branded_registry_member(member):
    return null_registry_member(member) and member.get("id") == "ElementCreationOptions.customElementRegistry"

def required_registry_brand_member(member):
    return (member.get("id") == "ImportNodeOptions.customElementRegistry"
            and member["type"] == {"kind": "named", "name": "CustomElementRegistry"}
            and not member["required"] and "default" not in member)


def string_member(member):
    return (member.get("id") == "ElementCreationOptions.is"
            and member["type"] == {"kind": "named", "name": "DOMString"}
            and not member["required"] and "default" not in member)


def member_field(member):
    name = field_name(member["name"])
    return name + "_is_null" if null_registry_member(member) and not branded_registry_member(member) else name


def lower_values(data):
    """Lower validated declarations for the selected conversion profiles.

    Scalar members need no interface brand. Registry members use the selected
    brand conversion profile of their owning dictionary.
    The normalized member order is also the later conversion read order.
    """
    definitions = {d["name"]: d for d in data["declarations"] if not d.get("partial", False)}
    enumerations = [d for d in definitions.values() if d["kind"] == "enumeration"]
    for d in enumerations:
        variants = [variant(value) for value in d["values"]]
        require(len(set(variants)) == len(variants), "enumeration Rust variant collision")
    dictionaries = []

    def members(name, seen):
        require(name not in seen, "dictionary inheritance cycle")
        d = definitions.get(name)
        if d is None or d["kind"] != "dictionary":
            return None
        inherited = members(d["inherits"], seen | {name}) if "inherits" in d else []
        if inherited is None:
            return None
        result = list(inherited)
        for m in sorted(d["members"], key=lambda m: m["name"]):
            ty = m["type"]
            if not (null_registry_member(m) or required_registry_brand_member(m)):
                if ty["kind"] != "named" or ty.get("attributes"):
                    return None
                target = definitions.get(ty["name"])
                if ty["name"] != "boolean" and (target is None or target["kind"] != "enumeration") and not (
                    string_member(m)
                ):
                    return None
            require(all(previous["name"] != m["name"] for previous in result), "duplicate inherited dictionary member")
            result.append(m)
        fields = [member_field(m) for m in result]
        require(len(set(fields)) == len(fields), "dictionary Rust field collision")
        return result

    for d in definitions.values():
        if d["kind"] == "dictionary":
            flattened = members(d["name"], set())
            if flattened is not None:
                dictionaries.append({"name": d["name"], "members": flattened})
    return {"enumerations": enumerations, "dictionaries": dictionaries,
            "typedefs": {d["name"]: d["type"] for d in definitions.values() if d["kind"] == "typedef"},
            "namespaces": [d for d in definitions.values() if d["kind"] == "namespace"]}


def code_units(value):
    encoded = value.encode("utf-16-le", errors="surrogatepass")
    return [int.from_bytes(encoded[i:i + 2], "little") for i in range(0, len(encoded), 2)]


def units_literal(value):
    return "[" + ", ".join(str(unit) for unit in code_units(value)) + "]"


def member_type(member):
    if required_registry_brand_member(member):
        return "Option<CustomElementRegistryIdentity>"
    if branded_registry_member(member):
        return "Option<Option<CustomElementRegistryIdentity>>"
    if null_registry_member(member):
        return "bool"
    name = member["type"]["name"]
    value = {"boolean": "bool", "DOMString": "DomString"}.get(name, name)
    return value if member["required"] or "default" in member else "Option<" + value + ">"


def member_default(member, enumerations):
    if required_registry_brand_member(member):
        return "None"
    if branded_registry_member(member):
        return "None"
    if null_registry_member(member):
        return "false"
    value = member.get("default")
    if value is None:
        require(not member["required"], "required dictionary member has no default")
        return "None"
    name = member["type"]["name"]
    if name == "boolean":
        require(value["kind"] == "boolean", "Boolean dictionary default has the wrong type")
        return str(value["value"]).lower()
    require(value["kind"] == "string" and value["value"] in enumerations[name]["values"],
            "enumeration dictionary default is not a member")
    return name + "::" + variant(value["value"])


def emit_values(model):
    result = ["// Generated by tools/generate_bindings.py; do not edit.",
              "// Source-adapted WHATWG declarations: licenses/whatwg/LICENSE and NOTICE.md.",
              "//! Typed IDL values from the retained binding description.",
              "//!", "//! These values do not install interfaces or convert guest inputs.", ""]
    imports = [name for name, accepts in [("CustomElementRegistryIdentity", lambda member:
                                           branded_registry_member(member) or required_registry_brand_member(member)),
                                           ("DomString", string_member)]
               if any(accepts(m) for d in model["dictionaries"] for m in d["members"])]
    if len(imports) == 1:
        result += ["use crate::" + imports[0] + ";", ""]
    elif imports:
        result += ["use crate::{" + ", ".join(imports) + "};", ""]
    enumerations = {d["name"]: d for d in model["enumerations"]}
    for d in model["enumerations"]:
        result += ["/// Values of the declared `" + d["name"] + "` enumeration.",
                   "#[derive(Clone, Copy, Debug, PartialEq, Eq)]", "pub enum " + d["name"] + " {"]
        for value in d["values"]:
            result += ["    /// The `" + value + "` value.", "    " + variant(value) + ","]
        result += ["}", "", "impl " + d["name"] + " {",
                   "    /// Matches exact UTF-16 code units without guest coercion.",
                   "    pub fn from_code_units(units: &[u16]) -> Option<Self> {", "        match units {"]
        for value in d["values"]:
            result.append("            " + units_literal(value) + " => Some(Self::" + variant(value) + "),")
        result += ["            _ => None,", "        }", "    }", "",
                   "    /// The exact code units of this enumeration value.",
                   "    pub fn as_code_units(self) -> &'static [u16] {", "        match self {"]
        for value in d["values"]:
            result.append("            Self::" + variant(value) + " => &" + units_literal(value) + ",")
        result += ["        }", "    }", "}", ""]
    for d in model["dictionaries"]:
        defaults = None if any(m["required"] for m in d["members"]) else [member_default(m, enumerations) for m in d["members"]]
        derive_default = defaults is not None and all(value in {"false", "None"} for value in defaults)
        copy = "" if any(string_member(m) for m in d["members"]) else "Clone, Copy, "
        derives = copy + "Debug, " + ("Default, " if derive_default else "") + "PartialEq, Eq"
        doc = ("Complete `" + d["name"] + "` dictionary value for the admitted registry profile."
               if any(null_registry_member(m) or required_registry_brand_member(m) for m in d["members"])
               else "Complete brand-free `" + d["name"] + "` dictionary value.")
        result += ["/// " + doc,
                   "#[derive(" + derives + ")]", "pub struct " + d["name"] + " {"]
        for m in d["members"]:
            doc = ("Whether `customElementRegistry` was explicitly supplied as null."
                   if null_registry_member(m) else "The declared `" + m["name"] + "` member.")
            if branded_registry_member(m):
                doc = "A supplied null or exact branded registry identity."
            result += ["    /// " + doc,
                       "    pub " + member_field(m) + ": " + member_type(m) + ","]
        result += ["}", ""]
        if defaults is not None and not derive_default:
            result += ["impl Default for " + d["name"] + " {", "    fn default() -> Self {", "        Self {"]
            result += ["            " + member_field(m) + ": " + value + "," for m, value in zip(d["members"], defaults)]
            result += ["        }", "    }", "}", ""]
    if element_creation_dictionary(model):
        result += ["/// The declared string or dictionary input to the Document element factories.",
                   "#[derive(Debug, PartialEq, Eq)]",
                   "pub enum ElementCreationArgument {",
                   "    /// The legacy string alternative.",
                   "    String(DomString),",
                   "    /// The dictionary alternative in the admitted registry profile.",
                   "    Options(ElementCreationOptions),", "}", ""]
    if import_node_dictionary(model):
        result += ["/// The declared Boolean or dictionary input to Document.importNode.",
                   "#[derive(Clone, Copy, Debug, PartialEq, Eq)]",
                   "pub enum ImportNodeArgument {",
                   "    /// The Boolean subtree flag.",
                   "    Subtree(bool),",
                   "    /// The converted options dictionary.",
                   "    Options(ImportNodeOptions),", "}", ""]
    return "\n".join(result)


def element_creation_dictionary(model):
    return any(d["name"] == "ElementCreationOptions" for d in model["dictionaries"])

def import_node_dictionary(model):
    return any(d["name"] == "ImportNodeOptions" for d in model["dictionaries"])


def conversion_type(ty, model, aliases=frozenset()):
    """A complete supported type, never a successful partial brand conversion."""
    if ty.get("attributes"):
        if (ty["kind"] == "named" and ty["name"] == "DOMString"
                and ty["attributes"] == [{"name": "LegacyNullToEmptyString"}]):
            return "DomString", "Type<DomString>", "types::DOM_STRING_NULL_TO_EMPTY"
        return None
    if ty["kind"] == "nullable":
        inner = conversion_type(ty["inner"], model, aliases)
        if inner is None or ty["inner"]["kind"] != "named":
            return None
        # Dictionary nullability is forbidden in arguments and members.
        if inner[0] in {d["name"] for d in model["dictionaries"]}:
            return None
        return ("Option<" + inner[0] + ">", "Nullable<" + inner[1] + ">", inner[2] + ".nullable()")
    if ty["kind"] == "union" and ty["members"] == [
        {"kind": "named", "name": "TrustedType"}, {"kind": "named", "name": "DOMString"}
    ]:
        return ("TrustedAttributeValue", "Type<TrustedAttributeValue>",
                "types::TRUSTED_ATTRIBUTE_VALUE")
    if ty["kind"] == "union" and ty["members"] == [
        {"kind": "named", "name": "DOMString"}, {"kind": "named", "name": "ElementCreationOptions"}
    ] and element_creation_dictionary(model):
        return ("ElementCreationArgument", "Type<ElementCreationArgument>",
                "Type::scalar(super::composition::element_creation_argument)")
    if ty["kind"] == "union" and ty["members"] == [
        {"kind": "named", "name": "boolean"}, {"kind": "named", "name": "ImportNodeOptions"}
    ] and import_node_dictionary(model):
        return ("ImportNodeArgument", "Type<ImportNodeArgument>",
                "Type::scalar(super::composition::import_node_argument)")
    if ty["kind"] != "named":
        return None
    name = ty["name"]
    typedefs = model["typedefs"]
    if name in typedefs:
        require(name not in aliases, "typedef cycle")
        underlying = typedefs[name]
        if underlying["kind"] != "named":
            return None
        return conversion_type(underlying, model, aliases | {name})
    scalars = {"boolean": ("bool", "BOOLEAN"), "DOMString": ("DomString", "DOM_STRING"),
               "USVString": ("UsvString", "USV_STRING"), "ByteString": ("ByteString", "BYTE_STRING"),
               "float": ("f32", "FLOAT"), "double": ("f64", "DOUBLE"),
               "unrestricted float": ("f32", "UNRESTRICTED_FLOAT"),
               "unrestricted double": ("f64", "UNRESTRICTED_DOUBLE"), "undefined": ("()", "UNDEFINED")}
    if name in scalars:
        output, descriptor = scalars[name]
        return output, "Type<" + output + ">", "types::" + descriptor
    integers = {"byte": "Byte", "octet": "Octet", "short": "Short", "unsigned short": "UnsignedShort",
                "long": "Long", "unsigned long": "UnsignedLong", "long long": "LongLong", "unsigned long long": "UnsignedLongLong"}
    if name in integers:
        return "IntegerValue", "IntegerType", "IntegerType::new(IntegerKind::" + integers[name] + ", IntegerMode::Wrap)"
    if name in {d["name"] for d in model["enumerations"]}:
        return name, "Type<" + name + ">", field_name(name) + "().0"
    if name in {d["name"] for d in model["dictionaries"]}:
        return name, "Type<" + name + ">", field_name(name) + "()"
    return None


def lower_arguments(data, model):
    arguments = []
    for declaration in data["declarations"]:
        for member in declaration.get("members", []):
            for index, argument in enumerate(member.get("arguments", [])):
                info = conversion_type(argument["type"], model)
                if info is None or argument["presence"] == "variadic":
                    continue
                identifier = member["id"] + "/" + argument["name"]
                function = field_name(re.sub(r"[^A-Za-z0-9_]", "_", member["id"]) + "_" + argument["name"])
                arguments.append({"id": identifier, "function": function, "index": index,
                                  "argument": argument, "info": info})
    functions = [a["function"] for a in arguments]
    require(len(set(functions)) == len(functions), "argument Rust function collision")
    return arguments


def lower_annotated_setters(data, model):
    setters = []
    for declaration in data["declarations"]:
        for member in declaration.get("members", []):
            if (member["kind"] != "attribute" or member["readonly"]
                    or not member["type"].get("attributes")):
                continue
            info = conversion_type(member["type"], model)
            if info is not None:
                setters.append((member["id"], field_name(member["id"].replace(".", "_")), info))
    return setters


def emit_conversions(model, arguments=(), annotated_setters=()):
    result = ["// Generated by tools/generate_bindings.py; do not edit.",
              "// Source-adapted WHATWG and W3C declarations: see the crate NOTICE.md.",
              "//! Closed scoped conversion descriptors from the retained declarations.", "",
              "use super::{",
              "    Argument, Conversion, Converted, Enumeration, Failure, IntegerType, Nullable, Type, types,", "};"]
    if any(a["info"][0] == "TrustedAttributeValue" for a in arguments):
        result.append("use crate::TrustedAttributeValue;")
    imports = [d["name"] for d in model["enumerations"] + model["dictionaries"]]
    if any(a["info"][0] == "ElementCreationArgument" for a in arguments):
        imports.append("ElementCreationArgument")
    if any(a["info"][0] == "ImportNodeArgument" for a in arguments):
        imports.append("ImportNodeArgument")
    for name in sorted(imports):
        result.append("use crate::declarations::" + name + ";")
    result += ["use crate::{DomString, IntegerKind, IntegerMode, IntegerValue, UsvString, Value};", ""]
    for definition in model["enumerations"]:
        name = definition["name"]
        result += ["/// Scoped conversion for `" + name + "`.",
                   "pub fn " + field_name(name).removeprefix("r#") + "() -> Enumeration<" + name + "> {"]
        arguments_text = json.dumps(name) + ", " + name + "::from_code_units"
        if len(arguments_text) <= 60:
            result.append("    Enumeration::new(" + arguments_text + ")")
        else:
            result += ["    Enumeration::new(", "        " + json.dumps(name) + ",",
                       "        " + name + "::from_code_units,", "    )"]
        result += ["}", ""]
    enumerations = {d["name"]: d for d in model["enumerations"]}
    for definition in model["dictionaries"]:
        name, function = definition["name"], field_name(definition["name"])
        retained_string = next((m for m in definition["members"] if string_member(m)), None)
        result += ["/// Complete dictionary conversion for `" + name + "`.",
                   "pub fn " + function + "() -> Type<" + name + "> {",
                   "    Type::scalar(convert_" + function + ")", "}", "",
                   "fn convert_" + function + "<'s>(",
                   "    cx: &mut Conversion<'_, 's>,", "    input: &Value<'s>,",
                   ") -> Result<Converted<" + name + ">, Failure> {",
                   "    let object = cx.dictionary(" + json.dumps(name) + ", input)?;"]
        for member in definition["members"]:
            field = member_field(member)
            member_units = units_literal(member["name"])
            member_arguments = "input, object, &" + member_units
            if len(member_arguments) <= 60:
                result.append("    let " + field + " = match cx.member(" + member_arguments + ")? {")
            else:
                result += ["    let " + field + " = match cx.member(", "        input,", "        object,"]
                if len(member_units) <= 60:
                    result += ["        &" + member_units + ","]
                else:
                    result += ["        &["]
                    result += ["            " + line for line in textwrap.wrap(member_units[1:-1] + ",", width=88)]
                    result += ["        ],"]
                result += ["    )? {"]
            if required_registry_brand_member(member):
                result += ["        Some(value) => Some(cx.required_custom_element_registry_member(&value)?),"]
            elif branded_registry_member(member):
                result += ["        Some(value) => Some(cx.custom_element_registry_member(&value)?),"]
            elif null_registry_member(member):
                result += ["        Some(value) => cx.null_interface_member(\"CustomElementRegistry\", &value)?,"]
            elif string_member(member):
                result += ["        Some(value) => Some(cx.value(&types::DOM_STRING, &value)?),"]
            else:
                descriptor = conversion_type(member["type"], model)[2]
                optional = not member["required"] and "default" not in member
                result += ["        Some(value) => " + ("Some(" if optional else "") + "*cx.value(&" + descriptor + ", &value)?.get()" + (")" if optional else "") + ","]
            if member["required"]:
                result.append("        None => return Err(cx.validation(super::Rule::RequiredMember(" + json.dumps(member["id"]) + "))),")
            else:
                default = member_default(member, enumerations)
                result.append("        None => " + default + ",")
            result += ["    };"]
        if retained_string:
            field = member_field(retained_string)
            result += ["    let Converted { value: " + field + ", storage } = match " + field + " {",
                       "        Some(Converted { value, storage }) => Converted {",
                       "            value: Some(value),", "            storage,", "        },",
                       "        None => cx.plain(None),", "    };"]
        fields = ", ".join(member_field(m) for m in definition["members"])
        if retained_string:
            result += ["    Ok(Converted {", "        value: " + name + " {"]
            result += ["            " + member_field(m) + "," for m in definition["members"]]
            result += ["        },", "        storage,", "    })", "}", ""]
        elif len(fields) <= 18:
            result += ["    Ok(cx.plain(" + name + " { " + fields + " }))", "}", ""]
        else:
            result += ["    Ok(cx.plain(" + name + " {"]
            result += ["        " + member_field(m) + "," for m in definition["members"]]
            result += ["    }))", "}", ""]
    for namespace in model["namespaces"]:
        stem = "WINDOW_" + field_name(namespace["name"]).upper()
        result += ["pub(crate) const " + stem + "_NAME: &[u16] = &" +
                   units_literal(namespace["name"]) + ";"]
        for member in namespace["members"]:
            operation = stem + "_" + field_name(member["name"]).upper()
            result += ["pub(crate) const " + operation + "_NAME: &[u16] = &" +
                       units_literal(member["name"]) + ";",
                       "pub(crate) const " + operation + "_ARITY: u32 = " +
                       str(len(member["arguments"])) + ";"]
        result.append("")
    for lowered in arguments:
        arg, function = lowered["argument"], lowered["function"]
        output, descriptor_type, descriptor = lowered["info"]
        default = arg.get("default")
        default_fn = "None"
        if default is not None:
            default_fn = "Some(default_" + function + ")"
            kind = default["kind"]
            if kind == "empty_dictionary":
                expression = output + "::default()"
                if output == "ElementCreationArgument":
                    expression = "ElementCreationArgument::Options(\n        ElementCreationOptions::default(),\n    )"
            elif kind == "boolean":
                expression = str(default["value"]).lower()
                if output == "ImportNodeArgument":
                    expression = "ImportNodeArgument::Subtree(" + expression + ")"
            elif kind == "integer":
                expression = "IntegerValue::" + {"unsigned long": "UnsignedLong"}[arg["type"]["name"]] + "(" + str(default["value"]) + ")"
            elif kind == "string":
                require(arg["type"]["name"] == "DOMString", "unsupported declared string default")
                expression = "DomString::new(Vec::new())"
            elif kind == "null":
                expression = "None"
            else:
                raise DescriptionError("unsupported declared argument default")
            statement = "Ok(cx.plain(" + expression + "))"
            if kind == "string" and default["value"]:
                statement = "cx.default_dom_string(&" + units_literal(default["value"]) + ")"
            result += ["fn default_" + function + "(", "    cx: &mut Conversion<'_, '_>,", ") -> Result<Converted<" + output + ">, Failure> {",
                       "    " + statement, "}", ""]
        result += ["/// Selected argument `" + lowered["id"] + "` (not overload selection).",
                   "pub fn " + function + "() -> Argument<" + descriptor_type + "> {"]
        pieces = [json.dumps(lowered["id"]), str(lowered["index"]), str(arg["presence"] == "optional").lower(), descriptor, default_fn]
        if len(", ".join(pieces)) <= 60:
            result.append("    Argument::new(" + ", ".join(pieces) + ")")
        else:
            result += ["    Argument::new("] + ["        " + piece + "," for piece in pieces] + ["    )"]
        result += ["}", ""]
    for identifier, function, (_, descriptor_type, descriptor) in annotated_setters:
        result += ["/// Selected setter conversion for `" + identifier + "`.",
                   "pub fn " + function + "_setter() -> " + descriptor_type + " {",
                   "    " + descriptor, "}", ""]
    return "\n".join(result)


def dom_exception_names(source, root, constants):
    require("names_table_sha256" in source, "DOMException names table is missing")
    table = source_block(dict(source, block=source["block"] + 1,
                              sha256=source["names_table_sha256"]), root)
    require(table.startswith('<table id="error-names" '), "invalid DOMException names table")
    names = {}
    codes = set()
    for row in re.findall(r"<tr[^>]*>([\s\S]*?)</tr>", table):
        if "<th>" in row:
            continue
        cells = re.findall(r"<td>([\s\S]*?)</td>", row)
        require(len(cells) == 3, "DOMException names table row must have three cells")
        name = re.fullmatch(r'"(?:<dfn [^>]+>)?<code>([A-Za-z]+)</code>(?:</dfn>)?"', cells[0])
        require(name and name[1] not in names, "invalid or repeated DOMException name")
        code = 0
        if cells[2] != "—":
            legacy = re.fullmatch(r'<dfn [^>]+><code>([A-Z_]+)</code></dfn>&nbsp;\((\d+)\)', cells[2])
            require(legacy, "invalid DOMException legacy code")
            code = int(legacy[2])
            require(constants.get(legacy[1]) == code and code not in codes,
                    "DOMException legacy code differs from the IDL constants")
            codes.add(code)
        names[name[1]] = code
    require(names and codes, "DOMException names table has no named legacy codes")
    return names


def emit_dom_exception(declaration, names):
    constructors = [m for m in declaration["members"] if m["kind"] == "constructor"]
    require(len(constructors) == 1, "DOMException must have one constructor")
    arguments = constructors[0]["arguments"]
    require(all(a["presence"] != "variadic" for a in arguments),
            "DOMException requires fixed argument capture")
    result = ["// Generated by tools/generate_bindings.py; do not edit.",
              "// Source-adapted WHATWG declarations: see the crate NOTICE.md.",
              "//! Selected Web IDL DOMException interface and exception names.", "",
              "const fn ascii<const N: usize>(bytes: &[u8; N]) -> [u16; N] {",
              "    let mut units = [0; N];",
              "    let mut source: &[u8] = bytes;",
              "    let mut target: &mut [u16] = &mut units;",
              "    while let ([byte, remaining @ ..], [unit, pending @ ..]) = (source, target) {",
              "        *unit = *byte as u16;",
              "        source = remaining;",
              "        target = pending;",
              "    }",
              "    units",
              "}", "",
              "pub(super) const INTERFACE: &[u16] = &ascii(b" + json.dumps(declaration["name"]) + ");",
              "pub(super) const ARITY: u32 = " +
              str(sum(a["presence"] == "required" for a in arguments)) + ";",
              "pub(super) const CAPTURED_ARGUMENTS: u32 = " + str(len(arguments)) + ";"]
    for m in declaration["members"]:
        if m["kind"] == "attribute":
            require(m["readonly"], "DOMException requires readonly accessors")
            for identifier, label in [(m["name"].upper(), m["name"]),
                                      ("GET_" + m["name"].upper(), "get " + m["name"])]:
                result.append("pub(super) const " + identifier + ": &[u16] = &ascii(b" + json.dumps(label) + ");")
    result += ["pub(super) const CONSTANTS: &[(&[u16], u16)] = &["]
    for m in declaration["members"]:
        if m["kind"] == "constant":
            result.append("    (&ascii(b" + json.dumps(m["name"]) + "), " + str(m["value"]) + "),")
    result += ["];",
               "", "/// Exception names from the Web IDL DOMException names table.",
               "#[derive(Clone, Copy, Debug, Eq, PartialEq)]", "#[non_exhaustive]",
               "pub enum DomExceptionName {"]
    for name in names:
        result += ["    /// The `" + name + "` exception.", "    " + name + ","]
    result += ["}", "", "impl DomExceptionName {",
               "    /// Returns the exact code units of the standard exception name.",
               "    pub const fn code_units(self) -> &'static [u16] {", "        match self {"]
    for name in names:
        result.append("            Self::" + name + ' => &const { ascii(b"' + name + '") },')
    result += ["        }", "    }", "}", "",
               "const LEGACY_CODES: &[(&[u16], u16)] = &["]
    for name, code in sorted(names.items()):
        if code:
            result.append("    (DomExceptionName::" + name + ".code_units(), " + str(code) + "),")
    result += ["];", "",
               "pub(crate) fn legacy_code(name: &[u16]) -> u16 {",
               "    LEGACY_CODES",
               "        .binary_search_by_key(&name, |&(candidate, _)| candidate)",
               "        .ok()",
               "        .and_then(|index| LEGACY_CODES.get(index))",
               "        .map_or(0, |&(_, code)| code)",
               "}", ""]
    return "\n".join(result)


def generate(data, root=ROOT):
    definitions, _ = validate(data, root)
    model = lower_values(data)
    outputs = {VALUE_OUTPUT: emit_values(model),
               CONVERSION_OUTPUT: emit_conversions(model, lower_arguments(data, model),
                                                   lower_annotated_setters(data, model))}
    if declaration := definitions.get("DOMException"):
        source = next(s for s in data["sources"] if "DOMException" in s["entries"])
        constants = {m["name"]: m["value"] for m in declaration["members"] if m["kind"] == "constant"}
        names = dom_exception_names(source, root, constants)
        outputs[DOM_EXCEPTION_OUTPUT] = emit_dom_exception(declaration, names)
    return outputs


def load(path=DESCRIPTION):
    def unique(pairs):
        out = {}
        for k, v in pairs:
            require(k not in out, f"duplicate JSON key: {k}")
            out[k] = v
        return out
    def nonfinite(value):
        raise DescriptionError(f"nonfinite JSON number: {value}")
    return json.loads(path.read_text(encoding="utf-8"), object_pairs_hook=unique, parse_constant=nonfinite)


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--check", action="store_true")
    mode.add_argument("--write", action="store_true")
    args = parser.parse_args(argv)
    try:
        outputs = generate(load())
        for path, output in outputs.items():
            if args.write:
                path.write_text(output, encoding="utf-8", newline="\n")
                print(f"wrote {path.relative_to(ROOT)}")
            else:
                require(path.is_file() and path.read_text(encoding="utf-8") == output, "generated IDL artifact is stale; run --write")
        if args.check:
            print("binding description matches retained sources and generated IDL artifacts")
        return 0
    except (DescriptionError, KeyError, TypeError, OSError, json.JSONDecodeError) as error:
        print(f"binding description: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
