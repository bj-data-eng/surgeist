# Permissions Policy: initial native Document creation

Bounded Markdown conversion of the clauses used by [#1218](https://github.com/bj-data-eng/surgeist/issues/1218), prepared 16 October 2026. Product decisions and the admitted implementation domain belong to that issue. This reference does not select response-policy processing, reporting, script introspection or general navigation.

## Source basis and attribution

- Permissions Policy Editor's Draft, W3C Web Application Security Working Group, editors Ian Clelland and Ari Chivukula. Immutable [index.bs](https://github.com/w3c/webappsec-permissions-policy/blob/29398046f1e0cb45c091adda853df74925dc740a/index.bs), commit `29398046f1e0cb45c091adda853df74925dc740a` (22 September 2026), 78,906 bytes, SHA-256 `51a87c5f4008510e501b5a4fe553baf8271e0e3af6ce6e8feece987639794c9a`. The original is retained with the hosted-policy investigation capture. The repository's [license declaration at that pin](https://github.com/w3c/webappsec-permissions-policy/blob/29398046f1e0cb45c091adda853df74925dc740a/LICENSE.md) licenses reports under the [W3C Software and Document License, 2015](../licenses/w3c/software-license-2015.txt) and identifies the [W3C CLA](https://www.w3.org/community/about/agreements/cla/) for specification contributions.
- HTML `877d6146487f15b15bc8dd0e3c625063a37b456a`, complete captured HTML SHA-256 `877d6cc00dae6c18c16d952fdcbf944acc58919579621b4257fa0ed1888a871f`. [Origin definitions](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#origin): original raw lines 64314–64328 and 64370–64397. Copyright © WHATWG (Apple, Google, Mozilla, Microsoft); prose is [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/), with the [exact WHATWG license](../licenses/whatwg/LICENSE).
- The selected feature's definition and default are already converted in [Fullscreen, Permissions Policy Integration](fullscreen--7c38d773117aa1e6bfa13754afe77483f40c908f--7d8cd68e63c0.md#permissions-policy-integration). That source defines `fullscreen` with default allowlist `'self'`.

This software or document includes material copied from or derived from [Permissions Policy](https://github.com/w3c/webappsec-permissions-policy/blob/29398046f1e0cb45c091adda853df74925dc740a/index.bs). Copyright © 2026 W3C® (MIT, ERCIM, Keio, Beihang). The linked full W3C license and its warranty disclaimer apply. No endorsement is implied.

Conversion: manually convert the selected Bikeshed prose and algorithms into Markdown; resolve markup into readable names, normalize whitespace, and retain algorithm ordering, conditions and literal keywords. Definition lists become paragraphs. The original numbered algorithm entries all use `1.`; normal Markdown numbering is retained. The selected clauses contain no complex tables or diagrams. Sections below identify exact source spans. Editorial scope notes are explicitly marked. This is a bounded conversion, not the complete specification or a newly rendered upstream edition.

## Features and policy records

Source: index.bs lines 248–318, framework definitions.

A policy-controlled feature is a set of behaviors and APIs that can be enabled or disabled through Permissions Policy. Features have identifying string tokens and default allowlists. A user agent has a set of supported features: those it allows to be controlled through policies. It is not required to support every feature. The feature definitions themselves belong to other specifications.

A declared policy has an ordered map of feature declarations to allowlists, and an ordered map of feature reporting configurations to strings. A permissions policy has an ordered inherited-policy map from features to `Enabled` or `Disabled`, and a declared policy.

An empty permissions policy contains `Enabled` for every supported feature in its inherited map; both declared-policy maps are empty. After initialization the inherited map contains a value for each supported feature. Thus an empty policy is not a synonym for all features disabled.

## Allowlists and matching

Source: index.bs lines 366–444, allowlists and the `*`/`'self'` defaults.

An allowlist is either the special value `*`, representing every origin, or a structure with an ordered set of permissions-source expressions, a nullable self-origin and a nullable src-origin. The textual keywords `'self'`, `'src'` and `'none'` are interpreted in context; the stored allowlist contains the referenced origins, not those keywords.

To determine whether an allowlist matches an origin:

1. If the allowlist is `*`, return true.
2. If its self-origin is non-null and same origin-domain with the origin, return true.
3. If its src-origin is non-null and same origin-domain with the origin, return true.
4. If the origin is opaque, return false.
5. Parse the serialization of the origin using the URL parser.
6. For each expression, run the linked CSP URL/expression matching algorithm with that URL, expression, origin and redirect count zero; if it returns true, return true.
7. Return false.

The `*` default permits the feature in top-level and child documents unless a policy disables it. The `'self'` default permits it in top-level documents and in same-origin child documents when allowed in the parent; it does not grant cross-origin children by default.

Editorial scope note: the expression matcher and tuple-origin processing are named external algorithms, not reproduced by this reference. They are unreachable in #1218's selected opaque-target projection. This does not establish a complete stored container allowlist or support arbitrary-origin policy introspection.

## Declared origin

Source: index.bs lines 781–806, complete declared-origin algorithm and its note.

To get an Element's declared origin:

1. If its node Document's sandboxed-origin browsing-context flag is set, return a new opaque origin.
2. If its sandbox attribute is set and does not contain `allow-same-origin`, return a new opaque origin.
3. If its srcdoc attribute is set, return its node Document's origin.
4. If its src attribute is set, parse that attribute relative to its node Document. If parsing does not fail, return the resulting URL's origin.
5. Return its node Document's origin.

The source note explains that declared origin represents the origin the embedding page intends to load. An implementation that does not support sandbox or srcdoc should not take those attributes into account. #1218 does support their reached creation semantics; absent scripting does not remove them.

## Parse policy directive

Source: index.bs lines 944–988, complete algorithm.

Inputs: value string, container origin, optional target origin. Output: policy directive.

1. Start an empty ordered directive map.
2. For each serialized declaration produced by strictly splitting the value on `;`:
   1. Split the declaration on ASCII whitespace. If no tokens remain, continue.
   2. The first token is the feature name. If it does not identify a recognized policy-controlled feature, continue. Otherwise select that feature.
   3. The remaining tokens form the target list. Start a new allowlist.
   4. If any target-list token is `*`, set the allowlist to `*`.
   5. Otherwise:
      1. If the target list is empty and a target origin was supplied, set src-origin to that target origin.
      2. For each target-list token:
         1. An ASCII-insensitive match for `'self'` sets self-origin to container origin and continues to the next token.
         2. If a target origin was supplied, an ASCII-insensitive match for `'src'` sets src-origin to target origin and continues to the next token.
         3. Otherwise parse the token using the URL parser. If parsing succeeds and the resulting origin is not opaque, append that origin's serialization to the expressions.
   6. Set the directive map's entry for the feature to the allowlist.
3. Return the directive map.

Editorial reading note: the final map assignment replaces an earlier entry for the same feature. The source does not prescribe a separate overriding `'none'` branch in this algorithm. An empty target list and an explicit nonmatching target list are distinct.

## Process permissions policy attributes

Source: index.bs lines 991–1009, complete algorithm; lines 535–558 explain allowfullscreen.

1. If the element is not an iframe, return an empty directive.
2. Parse its allow attribute value, with its node Document's origin as container origin and its declared origin as target origin.
3. If allowfullscreen is specified and the resulting map does not contain fullscreen, set fullscreen's entry to `*`.
4. Return the container policy.

The separate allowfullscreen description gives explicit allow/fullscreen declarations precedence. Bare `allow="fullscreen"` is different from allowfullscreen: the former uses the declared src origin, while the latter supplies `*` when no fullscreen declaration exists.

## Create a Permissions Policy for a navigable

Source: index.bs lines 1012–1031, complete algorithm.

Inputs: null or an Element container, and an origin.

1. Assert that a non-null container is a navigable container.
2. Start an empty ordered inherited-policy map.
3. For every supported feature, define its inherited policy in this container at this origin, and store that value in the map.
4. Create a policy with that inherited map and an empty declarations map and empty reporting-configuration map.
5. Return the policy.

## Define inherited policy for feature in container at origin

Source: index.bs lines 1055–1088, complete algorithm. The optional report-only argument defaults to false.

1. If the container is null, return `Enabled`.
2. Get the feature value for the container's node Document at that Document's own origin, using the report-only argument. If disabled, return `Disabled`.
3. Get the feature value for that same Document at the supplied child origin, using the report-only argument. If disabled, return `Disabled`.
4. Process the container's permissions-policy attributes.
5. If the resulting container policy has an entry for the feature, return `Enabled` if that allowlist matches the supplied origin, otherwise `Disabled`.
6. If the feature's default allowlist is `*`, return `Enabled`.
7. If its default allowlist is `'self'` and the supplied origin is same origin with the container's node Document's origin, return `Enabled`.
8. Return `Disabled`.

## Get feature value for origin

Source: index.bs lines 1089–1111, complete algorithm.

Inputs: feature, Document, origin, report-only Boolean.

1. Select the Document's report-only policy when report-only is true, otherwise its permissions policy.
2. If that policy's inherited value for the feature is `Disabled`, return `Disabled`.
3. If the feature is present in the declared policy, return `Enabled` when its declaration allowlist matches the origin, otherwise `Disabled`.
4. Return `Enabled`.

Editorial scope note: this algorithm does not add the feature's default allowlist to an otherwise empty declared policy. The child-inheritance algorithm applies the default at its own later step.

## HTML opaque-origin equality

Source: HTML raw lines 64314–64328 and 64370–64397, as identified above.

An opaque origin is an internal value that cannot be reconstructed from serialization. Its serialization is `null`; the meaningful operation on it is equality. Origins can be shared among Documents. Tuple origins carry scheme, host, port and domain, and are a separate alternative.

Two origins are same origin when they are the same opaque origin, or when both are tuple origins with identical schemes and ports and equal hosts; otherwise false.

Two origins are same origin-domain when they are the same opaque origin, or when both are tuple origins satisfying either identical scheme and identical non-null domain, or same origin with both domains null; otherwise false.

Editorial consequence: for opaque targets the two comparison relations reduce to actual identity equality. Two separately created opaque origins do not compare equal because they both serialize to `null`.
