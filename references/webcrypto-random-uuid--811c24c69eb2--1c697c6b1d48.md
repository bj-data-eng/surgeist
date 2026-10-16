# Web Cryptography: random UUID generation clauses

Bounded source retention for native hosted session-history entry identifiers.
The original W3C source remains authoritative. This reference supplies the
complete selected generation algorithm and its hexadecimal spelling; it does
not implement Web Crypto, the exposed Crypto API, JavaScript, a cryptographic
random provider or a new RFC contract.

## Source basis and attribution

Source: W3C Web Application Security Working Group, *Web Cryptography API*,
Level 2 Editor's Draft (source configuration), editor Daniel Huigens; former
editors Mark Watson and Ryan Sleevi. Immutable
[spec/Overview.html](https://github.com/w3c/webcrypto/blob/811c24c69eb22d477af5f1678cb70dbc611c7c40/spec/Overview.html),
commit `811c24c69eb22d477af5f1678cb70dbc611c7c40` (11 August 2026), preceding
HTML877d (9 October 2026). Retrieved as inert reference bytes on 16 October 2026.
No ReSpec, source script, stylesheet, dependency or executable was acquired or run.

Complete original source: 674,900 bytes, SHA-256
`00a094ac4298f0c7a5a4afe599281827c98fefbe72b09756f1f5859746f52c97`;
Git blob `d568caa38f4f1fe7b1b8034a578b99d2008b7477`.

Copyright © 2026 World Wide Web Consortium.
[W3C Document License, 2023](../licenses/w3c/document-license-2023.txt).
Copyright © 2023 W3C®. This software or document includes material copied from or
derived from [Web Cryptography API](https://github.com/w3c/webcrypto/blob/811c24c69eb22d477af5f1678cb70dbc611c7c40/spec/Overview.html).
This bounded copy accompanies Surgeist as implementation-support material; it
is not independently published for use as a technical specification. W3C is not
responsible for added material or formatting errors. The original document
and its status remain authoritative; no endorsement is implied.

The exact upstream [repository LICENSE.md at the pin](https://github.com/w3c/webcrypto/blob/811c24c69eb22d477af5f1678cb70dbc611c7c40/LICENSE.md)
licenses repository documents by contributors under the W3C Document License.
Its named official link resolves to the
[2023 Document License](https://www.w3.org/copyright/document-license-2023/).
The complete local license, SHA-256
`3affd8e454379fc3acdc7a350043acd64358700fba95fa8d480b75685bb6d93d`,
contains the full notice, derivative implementation-support permission,
disclaimers and trademark conditions. Its separate Code Components provision
covers the explicitly marked Web IDL under the
[W3C Software and Document License](../licenses/w3c/software-license-2023.txt),
SHA-256 `bd0e4ada87e6f70c3daceada3401c2f018e3333600b5a50fd57d3da6cea82da4`.
This is not an alternative prose license. The exact 155-byte upstream declaration
is retained with the source capture, SHA-256
`7a3ad7d36b8855bc301276279769da4aff648ea5d7b92f3f023c0823ee948764`.

Changes by Surgeist on 16 October 2026: offline bounded extraction, explanatory
headings/provenance and editorial scope notes. Source fragments are preserved
**byte for byte** in fenced text below, including original HTML/ReSpec syntax,
variables, conditions, step order, notes and citation markers. No source links
or RFC citations are rewritten. The wrapper links exact immutable line ranges.
No renderer or script is executed and no RFC edition is acquired/adopted.

The selected original fragments in table order, joined by one extra LF, have
SHA-256 `1c697c6b1d484b25225d711dc68666d05a3aceced4d2f5e454eb2d8fad6d9b24`.
The filename suffix identifies those original fragments, not the final file.

| Original source lines | Complete retained unit | Original fragment SHA-256 |
| --- | --- | --- |
| [805–840](https://github.com/w3c/webcrypto/blob/811c24c69eb22d477af5f1678cb70dbc611c7c40/spec/Overview.html#L805-L840) | Crypto definition, complete IDL and entropy notes | `956136a0226b4fcaa1bf2120321ddbc188f68f4282b17968ee606db546be61af` |
| [901–979](https://github.com/w3c/webcrypto/blob/811c24c69eb22d477af5f1678cb70dbc611c7c40/spec/Overview.html#L901-L979) | Complete randomUUID method, generate-a-random-UUID algorithm and hexadecimal definition | `568f22062051714a08df4c3267f3d3459e6ecd8141315dd8c6c048d5567447d5` |

## Editorial boundary between algorithm and exposed method

The retained IDL places `[SecureContext]` on the exposed `Crypto.randomUUID()`
method. The following source text separately names the algorithm “generate a
random UUID.” That placement and separation remain literal below. The source
reference does not turn the exposed method's context attribute into an invented
condition on every internal caller, nor remove the actual attribute from the
method. Architecture's native identifier/failure determination belongs to
[#136](https://github.com/bj-data-eng/surgeist/issues/136), not a normative edit here.

The complete algorithm retains its 16-byte cryptographically secure random fill,
byte 6 version and byte 8 variant bit operations, every concatenation item and the
lowercase, two-character, zero-padded hexadecimal definition. Its original
RFC4122 version/variant/representation and RFC2141 citation markers remain
unchanged. This source retention adopts no newer UUID RFC or general UUID API.
The adjacent entropy/PRNG notes are complete and informative; they do not
substitute a counter, timestamp, weak random source or caller assertion for
cryptographically secure generation. Provider availability, admission,
cancellation, retry and canonical identity custody remain actual native-owner
contracts; no successful production sample or Web Crypto completion is claimed.

## Crypto definition, complete IDL and entropy notes

<!-- selected-source-fragment:805-840 -->
```text
        <h2>Crypto interface</h2>
        <p>
          The <dfn data-idl id="dfn-Crypto">Crypto</dfn> interface represents an interface to
          general purpose cryptographic functionality including a
          cryptographically strong pseudo-random number generator seeded with truly random values.
        </p>
        <pre class=idl>
partial interface mixin WindowOrWorkerGlobalScope {
  [SameObject] readonly attribute Crypto crypto;
};

[Exposed=(Window,Worker)]
interface Crypto {
  [SecureContext] readonly attribute SubtleCrypto subtle;
  ArrayBufferView getRandomValues(ArrayBufferView array);
  [SecureContext] DOMString randomUUID();
};
        </pre>

        <div class=note>
          Implementations should generate cryptographically strong random values using
          well-established cryptographic pseudo-random number generators seeded with high-quality
          entropy, such as from an operating-system entropy source (e.g., "/dev/urandom"). This
          specification provides no lower-bound on the information theoretic entropy present in
          cryptographically strong random values, but implementations should make a best effort to provide
          as much entropy as practicable.
        </div>
        <div class=note>
          This interface defines a synchronous method for obtaining cryptographically strong random
          values. While some devices and implementations may support truly random cryptographic
          number generators or provide interfaces that block when there is insufficient entropy,
          implementations are discouraged from using these sources when implementing
          getRandomValues, both for performance and to avoid depleting the system of entropy.
          Instead, these sources should be used to seed a cryptographic pseudo-random number
          generator that can then return suitable values efficiently.
        </div>
```

## Complete randomUUID method, generate-a-random-UUID algorithm and hexadecimal definition

<!-- selected-source-fragment:901-979 -->
```text
          <section id="Crypto-method-randomUUID">
            <h4>The randomUUID method</h4>
            <p>
              The <dfn id="dfn-Crypto-method-randomUUID" data-dfn-for=Crypto>randomUUID</dfn>
              method generates a new <a data-cite="RFC4122#section-4.4">version 4 UUID</a>
              and returns its <a data-cite="RFC2141#section-2.2">namespace specific string representation</a>
              as described in <a data-cite="RFC4122#section-3">section 3</a> of [[RFC4122]].
              To <dfn data-export="">generate a random UUID</dfn>:
            </p>
            <ol>
              <li>
                Let |bytes| be a [= byte sequence =] of length 16.
              </li>
              <li>
                Fill |bytes| with cryptographically secure random bytes.
              </li>
              <li>
                Set the 4 most significant bits of |bytes|[6], which represent the
                UUID <a data-cite="RFC4122#section-4.1.3">version</a>, to `0100`.
              </li>
              <li>
                Set the 2 most significant bits of |bytes|[8], which represent the
                UUID <a data-cite="RFC4122#section-4.1.1">variant</a>, to `10`.
              </li>
              <li>
                <p>
                  Return the [= string/concatenate | string concatenation =] of «
                </p>
                <ol style="list-style-type: none">
                  <li>
                    [= hexadecimal representation =] of |bytes|[0], [= hexadecimal
                    representation =] of |bytes|[1], [= hexadecimal representation =] of
                    |bytes|[2], [= hexadecimal representation =] of |bytes|[3],
                  </li>
                  <li>
                    "`-`",
                  </li>
                  <li>
                    [= hexadecimal representation =] of |bytes|[4], [= hexadecimal
                    representation =] of |bytes|[5],
                  </li>
                  <li>
                    "`-`",
                  </li>
                  <li>
                    [= hexadecimal representation =] of |bytes|[6], [= hexadecimal
                    representation =] of |bytes|[7],
                  </li>
                  <li>
                    "`-`",
                  </li>
                  <li>
                    [= hexadecimal representation =] of |bytes|[8], [= hexadecimal
                    representation =] of |bytes|[9],
                  </li>
                  <li>
                    "`-`",
                  </li>
                  <li>
                    [= hexadecimal representation =] of |bytes|[10], [= hexadecimal
                    representation =] of |bytes|[11], [= hexadecimal representation =] of
                    |bytes|[12], [= hexadecimal representation =] of |bytes|[13],
                    [= hexadecimal representation =] of |bytes|[14], [= hexadecimal
                    representation =] of |bytes|[15]
                  </li>
                </ol>
                <p>
                  ».
                </p>
              </li>
            </ol>
            <p>
              For the steps described in the algorithm to [= generate a random UUID =],
              the <dfn>hexadecimal representation</dfn> of a byte |value| is the
              two-character string created by expressing |value| in hexadecimal using
              [= ASCII lower hex digits =], left-padded with "`0`" to reach two
              [= ASCII lower hex digits =].
            </p>
          </section>
```
