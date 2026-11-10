#!/usr/bin/env python3
"""Validate the source-qualified core description and emit internal Rust records.

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
from urllib.parse import urlsplit


ROOT = Path(__file__).resolve().parents[3]
DESCRIPTION = ROOT / "crates/surgeist-bindings-js/bindings/dom-core.json"
OUTPUT = ROOT / "crates/surgeist-bindings-js/src/description/dom_core.rs"
IDENTIFIER = re.compile(r"[A-Za-z_][A-Za-z_0-9]*\Z")
PRIMITIVES = {"undefined", "boolean", "byte", "octet", "short", "unsigned short",
              "long", "unsigned long", "long long", "unsigned long long", "float",
              "unrestricted float", "double", "unrestricted double", "DOMString",
              "USVString", "ByteString", "object", "any"}
ATTRIBUTES = {"Exposed", "SameObject", "NewObject", "CEReactions", "Unscopable",
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
        keys(s, ["id", "path", "anchor", "block", "sha256", "revision", "url", "entries"])
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
        require(names.get(name) not in {"mixin", "external_mixin"}, f"mixin used as a type: {name}")
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
    require(not annotations & {"Exposed", "LegacyOverrideBuiltIns", "LegacyUnenumerableNamedProperties", "LegacyNullToEmptyString"}, "extended attribute on the wrong declaration scope")
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
    require(kind in {"interface", "mixin", "callback_interface", "dictionary", "enumeration", "typedef"}, "unknown definition kind")
    prefix = attributes(definition["attributes"])
    require(not definition.get("partial", False) or kind in {"interface", "mixin"}, "invalid partial definition")
    require(all(x["name"] in {"Exposed", "LegacyOverrideBuiltIns", "LegacyUnenumerableNamedProperties"} for x in definition["attributes"]), "member attribute on a definition")
    if kind == "typedef":
        keys(definition, ["id", "kind", "name", "attributes", "type"])
        return prefix + "typedef " + render_type(definition["type"], names) + " " + definition["name"] + ";"
    if kind == "enumeration":
        keys(definition, ["id", "kind", "name", "attributes", "values"])
        require(all(isinstance(x, str) for x in definition["values"]), "enum values must be strings")
        require(len(set(definition["values"])) == len(definition["values"]), "duplicate enum value")
        return prefix + "enum " + definition["name"] + " { " + ", ".join(json.dumps(x) for x in definition["values"]) + " };"
    keys(definition, ["id", "kind", "name", "attributes", "members"], ["inherits", "partial"])
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
    keys(unit, ["id", "path", "anchor", "block", "sha256", "revision", "url", "entries"])
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
    required_semantics = {"interface_objects", "property_descriptors", "brands", "argument_conversion",
                          "legacy_platform_objects", "iterable", "same_object", "new_object", "unscopables"}
    require(set(data["semantics"]) == required_semantics, "shared installation semantics are incomplete")
    require(all(x.startswith("https://") for x in data["semantics"].values()), "invalid semantics source")
    return definitions, names


def rs(value):
    # All authored labels/locators are ASCII. Reject rather than emit invalid Rust escapes.
    require(isinstance(value, str) and value.isascii(), "Rust metadata labels must be ASCII")
    return json.dumps(value)


def array(items):
    return "&[" + ", ".join(items) + "]"


def option(value, render=rs):
    return "None" if value is None else "Some(" + render(value) + ")"


def variant(name):
    result = "".join(part[:1].upper() + part[1:] for part in re.split(r"[^A-Za-z0-9]+", name))
    require(IDENTIFIER.fullmatch(result) and result != "Self", "invalid Rust enum variant")
    return result


def emit_attributes(items):
    return array("ExtendedAttribute { kind: AttributeKind::" + variant(x["name"]) +
                 ", value: " + ("AttributeValue::None" if "value" not in x else
                  "AttributeValue::Integers(" + array(map(str, x["value"])) + ")" if isinstance(x["value"], list) else
                  "AttributeValue::Identifier(" + rs(x["value"]) + ")") + " }" for x in items)


def emit_type(value, names):
    kind = value["kind"]
    if kind == "named":
        name = value["name"]
        category = "primitive" if name in PRIMITIVES else names[name]
        inner = "IdlType::Named { name: " + rs(name) + ", kind: NamedKind::" + variant(category) + " }"
        if value.get("attributes"):
            return "IdlType::Annotated { inner: &" + inner + ", attributes: " + emit_attributes(value["attributes"]) + " }"
        return inner
    if kind == "union":
        return "IdlType::Union(" + array(emit_type(x, names) for x in value["members"]) + ")"
    return "IdlType::" + variant(kind) + "(&" + emit_type(value["inner"], names) + ")"


def emit_default(value):
    if value is None:
        return "DefaultValue::Absent"
    kind = value["kind"]
    if kind in {"null", "empty_dictionary"}:
        return "DefaultValue::" + variant(kind)
    v = value["value"]
    return "DefaultValue::" + variant(kind) + "(" + (rs(v) if kind == "string" else str(v).lower()) + ")"


def emit_arguments(items, names):
    return array("Argument { name: " + rs(x["name"]) + ", ty: " + emit_type(x["type"], names) +
                 ", presence: Presence::" + variant(x["presence"]) + ", default: " +
                 emit_default(x.get("default")) + " }" for x in items)


def emit_member(member, names):
    kind = member["kind"]
    if kind == "constructor":
        payload = "Constructor { arguments: " + emit_arguments(member["arguments"], names) + " }"
    elif kind == "operation":
        payload = "Operation { result: " + emit_type(member["type"], names) + ", arguments: " + emit_arguments(member["arguments"], names) + ", special: Special::" + variant(member.get("special") or "none") + ", is_static: " + str(member.get("static", False)).lower() + " }"
    elif kind == "attribute":
        payload = "Attribute { ty: " + emit_type(member["type"], names) + ", readonly: " + str(member["readonly"]).lower() + " }"
    elif kind == "constant":
        payload = "Constant { ty: " + emit_type(member["type"], names) + ", value: " + str(member["value"]) + " }"
    elif kind == "iterable":
        payload = "Iterable { value: " + emit_type(member["type"], names) + " }"
    else:
        payload = "DictionaryMember { ty: " + emit_type(member["type"], names) + ", required: " + str(member.get("required", False)).lower() + ", default: " + emit_default(member.get("default")) + " }"
    return "Member { id: " + rs(member["id"]) + ", name: " + option(member.get("name")) + ", attributes: " + emit_attributes(member["attributes"]) + ", kind: MemberKind::" + payload + " }"


def emit_requirement(req):
    return "Requirement { owner: " + rs(req["owner"]) + ", contract: " + rs(req["contract"]) + ", source: " + rs(req["source"]) + ", issue: " + option(req.get("issue"), str) + " }"


def emit_producers(items):
    return array("Producer { owner: " + rs(p["owner"]) + ", path: " + rs(p["path"]) + ", symbol: " + rs(p["symbol"]) + " }" for p in items)


def emit_access(key, mapping):
    kind = mapping["kind"]
    reqs = array(emit_requirement(r) for r in mapping.get("requirements", []))
    if kind == "native":
        operation = "Native { adapter: AdapterId::" + variant(mapping["adapter"]) + ", producers: " + emit_producers(mapping["producers"]) + ", requirements: " + reqs + " }"
    elif kind == "derived":
        operation = "Derived { algorithm: DerivedId::" + variant(mapping["algorithm"]) + ", owner: " + rs(mapping["owner"]) + ", contract: " + rs(mapping["contract"]) + ", source: " + rs(mapping["source"]) + ", inputs: " + emit_producers(mapping["inputs"]) + ", requirements: " + reqs + " }"
    elif kind == "callback":
        operation = "CallbackInvocation { interface: " + rs(mapping["interface"]) + ", operation: " + rs(mapping["operation"]) + ", source: " + rs(mapping["source"]) + ", requirements: " + reqs + " }"
    elif kind == "forwarding":
        operation = "PutForwards { getter: " + rs(mapping["getter"]) + ", target: " + rs(mapping["target"]) + ", source: " + rs(mapping["source"]) + " }"
    else:
        operation = "ExternalHost { requirement: " + emit_requirement(mapping["requirement"]) + " }"
    return "    AccessMapping { access: " + rs(key) + ", operation: AccessOperation::" + operation + ", result_view: ResultView::" + variant(mapping.get("result_view", "value")) + " },"


def interface_plans(data):
    """Compose declarations, without installing a function or invoking a producer."""
    definitions = data["declarations"]
    primary = {d["name"]: d for d in definitions if d["kind"] == "interface" and not d.get("partial", False)}
    external = {d["name"] for d in data["externals"] if d["kind"] == "mixin"}

    def own(name):
        mixes = [x["mixin"] for x in data["includes"] if x["interface"] == name]
        group = [x for x in definitions if x["name"] == name or x["name"] in mixes]
        members = [m for d in group for m in d.get("members", [])]
        attrs = [a["name"] for d in group for a in d["attributes"]]
        return members, attrs, [n for n in mixes if n in external]

    plans = []
    for name, d in primary.items():
        members, attrs, mixes = own(name)
        effective, effective_attrs = list(members), list(attrs)
        parent = d.get("inherits")
        while parent in primary:
            inherited, annotations, _ = own(parent)
            effective += inherited
            effective_attrs += annotations
            parent = primary[parent].get("inherits")
        def special_getter(ty):
            return next((m["id"] for m in effective if m.get("special") == "getter" and
                         m["arguments"][0]["type"] == {"kind": "named", "name": ty}), None)
        groups = {}
        for m in members:
            if m.get("name") is not None and m["kind"] != "dictionary_member":
                groups.setdefault(m["name"], []).append(m)
        props = []
        for property_name, overloads in groups.items():
            m = overloads[0]
            require(all(x["kind"] == m["kind"] for x in overloads), "conflicting property kinds")
            require(len(overloads) == 1 or m["kind"] == "operation", "duplicate non-operation property")
            a = {x["name"] for x in m["attributes"]}
            unforgeable = "LegacyUnforgeable" in a
            placement = "interface_and_prototype" if m["kind"] == "constant" else "interface" if m.get("static") else "instance" if unforgeable else "prototype"
            require(all(x.get("static", False) == m.get("static", False) for x in overloads), "mixed static/regular overloads")
            length = min(sum(arg["presence"] == "required" for arg in x.get("arguments", [])) for x in overloads)
            props.append({"name": property_name, "declarations": [x["id"] for x in overloads],
                          "placement": placement, "kind": m["kind"], "length": length,
                          "setter": m["kind"] == "attribute" and (not m["readonly"] or "PutForwards" in a),
                          "writable": m["kind"] == "operation" and not unforgeable,
                          "enumerable": True, "configurable": m["kind"] != "constant" and not unforgeable,
                          "unscopable": "Unscopable" in a})
        plans.append({"name": name, "parent": d.get("inherits"),
                      "exposure": "all" if any(a.get("value") == "*" for a in d["attributes"] if a["name"] == "Exposed") else "window",
                      "members": [m["id"] for m in members], "external_mixins": mixes,
                      "constructors": [m["id"] for m in members if m["kind"] == "constructor"],
                      "constructor_length": min((sum(arg["presence"] == "required" for arg in m["arguments"]) for m in members if m["kind"] == "constructor"), default=0),
                      "properties": props, "indexed": special_getter("unsigned long"),
                      "named": special_getter("DOMString"),
                      "override_builtins": "LegacyOverrideBuiltIns" in effective_attrs,
                      "unenumerable_names": "LegacyUnenumerableNamedProperties" in effective_attrs,
                      "value_iterable": any(m["kind"] == "iterable" for m in members)})
    return plans


def generate(data, root=ROOT):
    definitions, names = validate(data, root)
    result = ["// Generated by tools/generate_bindings.py; do not edit.",
              "// Source-adapted WHATWG declarations: licenses/whatwg/LICENSE and NOTICE.md.",
              "// Description and plans only: no installed bindings or native execution is claimed.",
              "use super::model::*;", "", "#[rustfmt::skip]", "pub(super) const DECLARATIONS: &[Definition] = &["]
    for d in data["declarations"]:
        result.append("    Definition { id: " + rs(d["id"]) + ", name: " + rs(d["name"]) + ", kind: DefinitionKind::" + variant(d["kind"]) + ", parent: " + option(d.get("inherits")) + ", partial: " + str(d.get("partial", False)).lower() + ", attributes: " + emit_attributes(d["attributes"]) + ", members: " + array(emit_member(m, names) for m in d.get("members", [])) + ", values: " + array(map(rs, d.get("values", []))) + ", alias: " + option(d.get("type"), lambda t: emit_type(t, names)) + " },")
    result += ["];", "", "pub(super) const INCLUDES: &[Includes] = &["]
    for x in data["includes"]:
        result.append("    Includes { interface: " + rs(x["interface"]) + ", mixin: " + rs(x["mixin"]) + " },")
    result += ["];", "", "pub(super) const EXTERNALS: &[External] = &["]
    for x in data["externals"]:
        result.append("    External { name: " + rs(x["name"]) + ", kind: NamedKind::External" + variant(x["kind"]) + ", owner: " + rs(x["owner"]) + ", source: " + rs(x["url"]) + ", requirement: " + rs(x["requirement"]) + " },")
    result += ["];", "", "pub(super) const SOURCES: &[SourceUnit] = &["]
    for x in data["sources"]:
        result.append("    SourceUnit { id: " + rs(x["id"]) + ", path: " + rs(x["path"]) + ", anchor: " + rs(x["anchor"]) + ", block: " + str(x["block"]) + ", sha256: " + rs(x["sha256"]) + ", revision: " + rs(x["revision"]) + ", url: " + rs(x["url"]) + ", entries: " + array(map(rs, x["entries"])) + " },")
    result += ["];", "", "#[derive(Clone, Copy, Debug, Eq, PartialEq)]", "pub(super) enum AdapterId {"]
    adapters = sorted({x["adapter"] for x in data["access_mappings"].values() if x["kind"] == "native"})
    require(len({variant(x) for x in adapters}) == len(adapters), "adapter Rust identifier collision")
    result += ["    " + variant(x) + "," for x in adapters]
    result += ["}", "", "#[derive(Clone, Copy, Debug, Eq, PartialEq)]", "pub(super) enum DerivedId {"]
    algorithms = sorted({x["algorithm"] for x in data["access_mappings"].values() if x["kind"] == "derived"})
    require(len({variant(x) for x in algorithms}) == len(algorithms), "derived Rust identifier collision")
    result += ["    " + variant(x) + "," for x in algorithms]
    result += ["}", "", "pub(super) const ACCESS_MAPPINGS: &[AccessMapping<AdapterId, DerivedId>] = &["]
    result += [emit_access(key, mapping) for key, mapping in data["access_mappings"].items()]
    result += ["];", "", "pub(super) const INTERFACE_PLANS: &[InterfacePlan] = &["]
    for plan in interface_plans(data):
        properties = []
        for p in plan["properties"]:
            form = "PropertyForm::Method { length: " + str(p["length"]) + " }" if p["kind"] == "operation" else "PropertyForm::Accessor { setter: " + str(p["setter"]).lower() + " }" if p["kind"] == "attribute" else "PropertyForm::Constant"
            properties.append("PropertyPlan { name: " + rs(p["name"]) + ", declarations: " + array(map(rs, p["declarations"])) + ", placement: Placement::" + variant(p["placement"]) + ", form: " + form + ", descriptor: Descriptor { writable: " + str(p["writable"]).lower() + ", enumerable: " + str(p["enumerable"]).lower() + ", configurable: " + str(p["configurable"]).lower() + " }, unscopable: " + str(p["unscopable"]).lower() + " }")
        result.append("    InterfacePlan { name: " + rs(plan["name"]) + ", parent: " + option(plan["parent"]) + ", exposure: Exposure::" + variant(plan["exposure"]) + ", own_members: " + array(map(rs, plan["members"])) + ", external_mixins: " + array(map(rs, plan["external_mixins"])) + ", constructors: " + array(map(rs, plan["constructors"])) + ", constructor_length: " + str(plan["constructor_length"]) + ", properties: " + array(properties) + ", legacy: LegacyProperties { indexed_getter: " + option(plan["indexed"]) + ", named_getter: " + option(plan["named"]) + ", override_builtins: " + str(plan["override_builtins"]).lower() + ", unenumerable_names: " + str(plan["unenumerable_names"]).lower() + " }, value_iterable: " + str(plan["value_iterable"]).lower() + " },")
    result += ["];", "", "pub(super) const SEMANTICS: &[SemanticSource] = &["]
    result += ["    SemanticSource { kind: SemanticKind::" + variant(k) + ", source: " + rs(v) + " }," for k, v in data["semantics"].items()]
    result += ["];", ""]
    # Preserve standalone generation with no Rust-toolchain dependency. Generated
    # table layout is owned here, as with the HTML vocabulary generator.
    for index in range(len(result) - 1, 0, -1):
        if result[index].startswith("pub(super) const ") and result[index - 1] != "#[rustfmt::skip]":
            result.insert(index, "#[rustfmt::skip]")
    return "\n".join(result)


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
        output = generate(load())
        if args.write:
            OUTPUT.parent.mkdir(parents=True, exist_ok=True)
            OUTPUT.write_text(output, encoding="utf-8", newline="\n")
            print(f"wrote {OUTPUT.relative_to(ROOT)}")
        else:
            require(OUTPUT.is_file() and OUTPUT.read_text(encoding="utf-8") == output, "generated binding description is stale; run --write")
            print("binding description matches retained sources and generated records")
        return 0
    except (DescriptionError, KeyError, TypeError, OSError, json.JSONDecodeError) as error:
        print(f"binding description: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
