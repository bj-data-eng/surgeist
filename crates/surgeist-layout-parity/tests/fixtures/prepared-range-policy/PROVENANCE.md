# Prepared Range policy section

`text-layout-contract.md` holds the heading `### Nonempty Text range fragments`,
the section under it and the heading line that follows it, copied byte for byte
from the repository's [text layout contract](../../../../../docs/text-layout-contract.md).
The section is the policy that the prepared Range qualification pins: its SHA-256
is `ef6219d28d15d9131a45d0c48edef12436333a1e1f2868f54d9ec8847d09ada0`, the value
`POLICY_SHA256` records and the registered qualification report retains. The
following heading ends the section, as it does in the full document.

The policy-identity tests include this fixture so that they read no file outside
the crate. They check the parser, the section hash and the shadowing refusals.
The qualification itself still compares the current repository document with the
retained digest at run time, because that comparison is the drift check between
the policy owner's text and the recorded input.
