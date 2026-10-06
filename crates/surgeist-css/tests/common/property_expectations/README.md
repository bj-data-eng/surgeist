# Common property expectations

[records.rs](records.rs) holds independently authored property expectations used
by the public contract tests and the executable metadata consumer. Each record
names a public property variant and canonical spelling once, then supplies the
contracts that the existing suites exercise for that property.

These are handwritten expectations. Select values, IDs, provenance, inheritance,
and shorthand membership from the selected specifications and the intended
public contract. Do not generate them from `src/properties.rs`, parser dispatch,
or production metadata. The public APIs are the implementation under test.
The metadata consumer's opening source notes and the feature catalog identify
the selected specifications; consult the pinned references when adding coverage.

## Record fields

| Field | Meaning |
| --- | --- |
| Variant and spelling | Independent expected property identity and canonical name |
| `metadata` | Longhand, ordinary shorthand, four-side shorthand, universal reset, or unavailable metadata |
| `catalog` | A grammar sample and explicit boundary outcome, or the supplemental complete-support contract |
| `source` | Exact source ID where the existing suite selects representative provenance |
| `aliases` | Exact aliases for a property covered by the grammar catalog; omission means an empty expected list |
| `dispatch` | Ordinary public parser stimulus; an optional `=>` supplies a different important stimulus |
| `wrapper` | `yes` checks the independently named property and typed value variant, then its retained authored text |

Coverage is optional per contract. A metadata-only property need not invent a
parser sample, and catalog and dispatch samples may differ. `all` deliberately
has no ordinary typed wrapper: its rejected ordinary stimulus and accepted
important global stimulus exercise its distinct contract.

Longhands specify inheritance and either a typed intrinsic-initial assertion or
a user-agent requirement. Typed callbacks can check arbitrary public structure,
including omitted slots, numeric representation, and list entries. Ordinary
shorthands supply ordered settable and reset-only members separately. Four-side
and universal-reset records identify their metadata kind; their detailed
mode-dependent behavior remains in specialized tests.

Boundary outcomes are explicit. Most grammar boundary samples are rejected;
three historical overflow samples now accept `auto` and retain assertions about
their typed value and authored omissions. Preserve that evidence when revising a
contract instead of silently deleting a historical stimulus.

## Consumers and maintenance

[property_expectations.rs](../property_expectations.rs) builds the private record
table and typed assertion callbacks. It imports public CSS types, not production
schema macros. It is shared by:

- [catalog_inventory.rs](../../catalog_inventory.rs), for lookup, aliases,
  provenance, grammar acceptance/rejection, and complete-support rows;
- [property_schema.rs](../../property_schema.rs), for broad dispatch, typed
  wrappers, and retained authored text;
- [property_metadata_consumer.rs](../../../examples/property_metadata_consumer.rs),
  for metadata availability, inheritance, intrinsic initials, ordered members,
  resets, and public grammar identity.

The metadata consumer rejects duplicate fixture identities and compares actual
available metadata with the independent fixture set. There is no fixed property
tally to maintain. Its integration test executes the example as a real external
consumer.

For a new property, add one record with the applicable fields and extend the
specialized family tests for behavior that the common runners do not express.
Run the affected consumer targets and family suite using the repository's
verification policy. Do not move recovery, expansion, numeric, normalization,
or authored-omission regressions into a generic row merely to centralize them.
Those suites remain separate behavioral evidence. This format consolidates the
common inventories; it does not require every CSS test to use the same format.
