"""Validate Web IDL choice shapes and lower finite effective overload sets."""

from dataclasses import dataclass
from itertools import combinations


class SelectionShapeError(ValueError):
    """A declaration has no valid Web IDL selection shape."""


@dataclass(frozen=True)
class CategoryAtom:
    """One flattened member with its category and resolved name, if nominal."""

    kind: str
    name: str


@dataclass(frozen=True)
class TypeShape:
    """Choice facts in authored member order, after resolving local typedefs."""

    atoms: tuple[CategoryAtom, ...]
    nullable: bool = False
    includes_undefined: bool = False
    deferred: frozenset[str] = frozenset()


@dataclass(frozen=True)
class SelectionRow:
    """Categories describe the distinguishing index in candidate order."""

    candidate_ids: tuple[str, ...]
    distinguishing_index: int | None = None
    categories: tuple[TypeShape, ...] = ()


@dataclass(frozen=True)
class SelectionGroup:
    """Authored arguments by member ID and finite rows by effective arity.

    A deferred group has no rows or maximum arity available for emission.
    """

    arguments: dict[str, tuple[dict, ...]]
    rows: dict[int, SelectionRow] | None
    deferred: str | None = None

    @property
    def maximum_arity(self):
        return None if self.rows is None else max(self.rows)


_STRINGS = {"DOMString", "USVString", "ByteString"}
_NUMERICS = {
    "byte", "octet", "short", "unsigned short", "long", "unsigned long",
    "long long", "unsigned long long", "float", "unrestricted float", "double",
    "unrestricted double",
}
_BUFFER_SOURCES = {
    "ArrayBuffer", "SharedArrayBuffer", "DataView", "Int8Array", "Int16Array",
    "Int32Array", "Uint8Array", "Uint16Array", "Uint32Array", "Uint8ClampedArray",
    "Float16Array", "Float32Array", "Float64Array", "BigInt64Array", "BigUint64Array",
}
# Above-diagonal entries of the pinned distinguishability table. Interface
# overlap and callback attributes require the additional checks below.
_DISTINGUISHABLE = {
    "undefined": {"boolean", "numeric", "bigint", "string", "object", "symbol",
                  "interface", "callback", "async_sequence", "sequence"},
    "boolean": {"numeric", "bigint", "string", "object", "symbol", "interface",
                "callback", "dictionary", "async_sequence", "sequence"},
    "numeric": {"bigint", "string", "object", "symbol", "interface", "callback",
                "dictionary", "async_sequence", "sequence"},
    "bigint": {"string", "object", "symbol", "interface", "callback", "dictionary",
               "async_sequence", "sequence"},
    "string": {"object", "symbol", "interface", "callback", "dictionary",
               "async_sequence", "sequence"},
    "object": {"symbol"},
    "symbol": {"interface", "callback", "dictionary", "async_sequence", "sequence"},
    "interface": {"callback", "dictionary", "async_sequence", "sequence"},
    "callback": {"dictionary", "async_sequence", "sequence"},
    "dictionary": {"async_sequence", "sequence"},
    "async_sequence": set(),
    "sequence": set(),
}


def _table_kind(atom):
    if atom.kind == "buffer_source":
        return "interface"
    if atom.kind in {"callback_interface", "record"}:
        return "dictionary"
    if atom.kind == "frozen_array":
        return "sequence"
    return atom.kind


def _has_dictionary(shape):
    return any(atom.kind == "dictionary" for atom in shape.atoms)


class _SelectionShapes:
    def __init__(self, definitions, names):
        self._primary = {d["name"]: d for d in definitions.values() if not d.get("partial")}
        self._names = names
        self.groups = {}

    def _ancestry(self, name):
        ancestors = set()
        while name is not None:
            if name in ancestors:
                raise SelectionShapeError(f"interface inheritance cycle: {name}")
            ancestors.add(name)
            if name not in self._primary:
                return ancestors, False
            name = self._primary[name].get("inherits")
        return ancestors, True

    def _distinguish_atoms(self, left, right):
        if left == right:
            return False
        a, b = _table_kind(left), _table_kind(right)
        if a in {"any", "promise"} or b in {"any", "promise"}:
            return False
        if a == "external" or b == "external":
            return None
        if a == b == "interface":
            if "buffer_source" in {left.kind, right.kind}:
                return True
            left_ancestors, left_complete = self._ancestry(left.name)
            right_ancestors, right_complete = self._ancestry(right.name)
            if left.name in right_ancestors or right.name in left_ancestors:
                return False
            if left_complete and right_complete or left_ancestors & right_ancestors:
                return True
            return None
        if a == b == "dictionary" and (
            self._names.get(left.name) == "external_dictionary"
            or self._names.get(right.name) == "external_dictionary"
        ):
            # A retained reference to a delegated dictionary cannot establish
            # the complete source-level selection contract for this pair.
            return None
        if {a, b} == {"callback", "dictionary"}:
            callback = left if a == "callback" else right
            definition = self._primary.get(callback.name)
            if definition is None:
                return None
            return all(attr["name"] != "LegacyTreatNonObjectAsNull"
                       for attr in definition["attributes"])
        return b in _DISTINGUISHABLE.get(a, ()) or a in _DISTINGUISHABLE.get(b, ())

    def _distinguish(self, left, right):
        if ((left.nullable and (right.nullable or _has_dictionary(right)))
                or (right.nullable and _has_dictionary(left))):
            return False
        deferred = False
        for a in left.atoms:
            for b in right.atoms:
                result = self._distinguish_atoms(a, b)
                if result is False:
                    return False
                deferred |= result is None
        return None if deferred else True

    def _named_atom(self, name):
        kind = self._names.get(name)
        if name in _STRINGS or kind == "enumeration":
            return CategoryAtom("string", name)
        if name in _NUMERICS:
            return CategoryAtom("numeric", name)
        if name in _BUFFER_SOURCES:
            return CategoryAtom("buffer_source", name)
        if name in {"undefined", "boolean", "bigint", "symbol", "object", "any"}:
            return CategoryAtom(name, name)
        if kind in {"interface", "dictionary", "callback", "callback_interface"}:
            return CategoryAtom(kind, name)
        if kind in {"external_interface", "external_dictionary", "external_callback"}:
            return CategoryAtom(kind.removeprefix("external_"), name)
        if kind in {"external_typedef", "external_type"}:
            return CategoryAtom("external", name)
        raise SelectionShapeError(f"unknown selection type: {name}")

    def _type(self, value, aliases):
        """Return choice facts and typedef-resolved type equality information."""
        kind = value["kind"]
        if kind == "named":
            name = value["name"]
            annotation = tuple(attr["name"] for attr in value.get("attributes", ()))
            if self._names.get(name) == "typedef":
                if name in aliases:
                    raise SelectionShapeError(f"typedef cycle: {name}")
                shape, identity = self._type(self._primary[name]["type"], aliases | {name})
                return shape, ("annotated", annotation, identity) if annotation else identity
            atom = self._named_atom(name)
            external = self._names.get(name, "").startswith("external_")
            shape = TypeShape((atom,), includes_undefined=atom.kind == "undefined",
                              deferred=frozenset({name}) if external else frozenset())
            identity = (kind, name)
            return shape, ("annotated", annotation, identity) if annotation else identity
        if kind == "union":
            members = [self._type(member, aliases) for member in value["members"]]
            atoms = tuple(atom for shape, _ in members for atom in shape.atoms)
            nullable = sum(shape.nullable for shape, _ in members)
            deferred = set().union(*(shape.deferred for shape, _ in members))
            for left, right in combinations(atoms, 2):
                result = self._distinguish_atoms(left, right)
                if result is False:
                    raise SelectionShapeError(
                        f"indistinguishable union members: {left.name} and {right.name}"
                    )
                if result is None:
                    deferred.update((left.name, right.name))
            shape = TypeShape(atoms, bool(nullable),
                              any(atom.kind == "undefined" for atom in atoms), frozenset(deferred))
            if nullable > 1 or (nullable and _has_dictionary(shape)):
                raise SelectionShapeError("union has incompatible nullable and dictionary members")
            return shape, (kind, frozenset(identity for _, identity in members))

        inner, identity = self._type(value["inner"], aliases)
        if kind == "nullable":
            if (inner.nullable or any(atom.kind in {"any", "promise"} for atom in inner.atoms)
                    or (len(inner.atoms) > 1 and _has_dictionary(inner))):
                raise SelectionShapeError("invalid inner type of a nullable type")
            return TypeShape(inner.atoms, True, inner.includes_undefined, inner.deferred), (kind, identity)
        if kind in {"sequence", "promise", "frozen_array", "async_sequence"}:
            return TypeShape((CategoryAtom(kind, kind),), deferred=inner.deferred), (kind, identity)
        raise SelectionShapeError(f"unknown selection type kind: {kind}")

    def flatten_type(self, value):
        """Validate a structured type and expose its flattened choice facts."""
        return self._type(value, frozenset())[0]

    def _argument_type(self, argument):
        shape, identity = self._type(argument["type"], frozenset())
        self._require_nonnullable_dictionary(shape)
        return shape, identity, argument["presence"]

    @staticmethod
    def _require_nonnullable_dictionary(shape):
        if shape.nullable and _has_dictionary(shape):
            raise SelectionShapeError("nullable dictionary type in an argument or dictionary member")

    def _row(self, label, arity, candidates):
        ids = tuple(member for member, _ in candidates)
        if len(candidates) == 1:
            return SelectionRow(ids)
        columns = zip(*(types for _, types in candidates), strict=True)
        shared_prefix = True
        for index, column in enumerate(columns):
            shapes = tuple(shape for shape, _, _ in column)
            compared = [self._distinguish(a, b) for a, b in combinations(shapes, 2)]
            if False not in compared:
                if shared_prefix:
                    if None in compared:
                        return None
                    direct = {shape.atoms[0].kind for shape, _, _ in column if len(shape.atoms) == 1}
                    if {"bigint", "numeric"} <= direct:
                        raise SelectionShapeError(f"{label} arity {arity}: bigint and numeric overloads")
                    return SelectionRow(ids, index, shapes)
                if None not in compared:
                    raise SelectionShapeError(f"{label} arity {arity}: differing overload prefix")
            shared_prefix &= all(entry[1:] == column[0][1:] for entry in column[1:])
        raise SelectionShapeError(f"{label} arity {arity}: indistinguishable overloads")

    def _group(self, key, arguments):
        signatures = {member: tuple(map(self._argument_type, args))
                      for member, args in arguments.items()}
        deferred = {name for signature in signatures.values() for shape, _, _ in signature
                    for name in shape.deferred}
        variadic = any(signature and signature[-1][2] == "variadic" for signature in signatures.values())
        longest = max(map(len, signatures.values()))
        rows_by_arity = {}
        for member, signature in signatures.items():
            arity = len(signature)
            rows_by_arity.setdefault(arity, []).append((member, signature))
            if arity and signature[-1][2] == "variadic":
                # Beyond the longest declaration, every surviving row just
                # repeats its last type. One extra arity proves the whole tail.
                for extended in range(arity + 1, longest + 2):
                    types = signature + (signature[-1],) * (extended - arity)
                    rows_by_arity.setdefault(extended, []).append((member, types))
            while arity and signature[arity - 1][2] in {"optional", "variadic"}:
                arity -= 1
                rows_by_arity.setdefault(arity, []).append((member, signature[:arity]))

        label = f"{key[0]}.{key[2]}"
        rows = {arity: self._row(label, arity, candidates)
                for arity, candidates in sorted(rows_by_arity.items())}
        if deferred:
            return SelectionGroup(arguments, None, "external selection facts: " + ", ".join(sorted(deferred)))
        if any(row is None for row in rows.values()):
            return SelectionGroup(arguments, None, "unresolved shared interface brand")
        if variadic:
            return SelectionGroup(arguments, None, "variadic group has no finite selector")
        return SelectionGroup(arguments, rows)


def lower_selection_shapes(definitions, names):
    """Validate retained declarations and return their selection model.

    `definitions` maps declaration IDs to entries and `names` maps nominal
    names to definition kinds, including the generator's `external_*` kinds.
    The result exposes `flatten_type(type_dict)` and `groups`, keyed by
    `(interface_name, kind, identifier)`. Regular operations, including named
    specials, share the `operation` kind across partial definitions. Other
    kinds are `static`, `constructor`, `legacy_factory`, `callback_interface`,
    and `namespace`. A constructor uses `constructor` as its identifier.
    """
    model = _SelectionShapes(definitions, names)
    groups = {}
    for definition in definitions.values():
        interface = definition["name"]
        definition_kind = definition["kind"]
        if "type" in definition:
            model.flatten_type(definition["type"])
        for argument in definition.get("arguments", ()):
            model._argument_type(argument)
        for member in definition.get("members", ()):
            kind = member["kind"]
            if "type" in member:
                shape = model.flatten_type(member["type"])
                if kind == "dictionary_member":
                    model._require_nonnullable_dictionary(shape)
            if kind == "constructor":
                key = (interface, "constructor", "constructor")
            elif kind == "operation" and "name" in member:
                group_kind = (definition_kind if definition_kind in {"namespace", "callback_interface"}
                              else "static" if member.get("static") else "operation")
                key = (interface, group_kind, member["name"])
            else:
                for argument in member.get("arguments", ()):
                    model._argument_type(argument)
                continue
            groups.setdefault(key, {})[member["id"]] = tuple(member["arguments"])
        for attribute in definition["attributes"]:
            if attribute["name"] == "LegacyFactoryFunction":
                factory = attribute["value"]
                key = (interface, "legacy_factory", factory["name"])
                member = f'{definition["id"]}.{factory["name"]}'
                groups.setdefault(key, {})[member] = tuple(factory["arguments"])
    model.groups = {key: model._group(key, signatures) for key, signatures in groups.items()}
    return model
