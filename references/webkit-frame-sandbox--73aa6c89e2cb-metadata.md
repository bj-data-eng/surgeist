# Legacy Frame creation sandbox witness

Inspected on 16 October 2026 at WebKit revision
`73aa6c89e2cb77c46184a81aec944e4ab99d114d`.
This Markdown reference contains provenance and behavioral summaries, not
transcribed browser source. No browser implementation is ingested. Original
copyright and licensing remain in the linked files and the
[WebKit license inventory](https://webkit.org/licensing-webkit/).

## Immutable implementation evidence

Git blob identifiers identify the complete fetched UTF-8 files. Source locations
are one-based lines in those exact files.

| File | Git blob | Lines | Bounded observation |
| --- | --- | --- | --- |
| [HTMLFrameOwnerElement.h](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/html/HTMLFrameOwnerElement.h) | `8bbb16b4e91e2437ab27d41e03a21cc51e3476f5` | 60, 81 | Common frame-owner storage exposes its sandbox flags. |
| [HTMLFrameOwnerElement.cpp](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/html/HTMLFrameOwnerElement.cpp) | `79a67c264f678d7ee0a434e42d516d1aaf24acd7` | 47–50, 112–116 | Construction leaves the flag set at its default; the setter replaces it. |
| [Frame.h](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/page/Frame.h) | `2815188c644cc66d5ce398a202b0e8b60ce3f2c4` | 70 | SandboxFlags is an OptionSet of SandboxFlag. |
| [OptionSet.h](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WTF/wtf/OptionSet.h) | `ec0b932fd2c98dabb99f08db98a50c3e024aff62` | 86, 215 | Default construction leaves the underlying storage zero. |
| [HTMLFrameElement.cpp](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/html/HTMLFrameElement.cpp) | `12a6b5592f2bca8a57f8281a1a66f7aad3b3c16f` | 43–47, 84–95 | Legacy Frame construction and attribute handling add no sandbox parsing. |
| [HTMLFrameElementBase.cpp](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/html/HTMLFrameElementBase.cpp) | `a68868092882872caa59458fb4cc057b788c3697` | 61–64, 130–153 | The shared Frame base does not parse the sandbox attribute. |
| [HTMLIFrameElement.cpp](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/html/HTMLIFrameElement.cpp) | `d260a8f498d6cca1a96535ca2718931d78e9c497` | 132–140 | Iframe handles sandbox changes and installs parsed flags, or empty flags when absent. |
| [WebFrame.cpp](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebKit/WebProcess/WebPage/WebFrame.cpp) | `b8650e94ff3c139271a14be70bbfd0e26d23f48c` | 165–179 | Subframe creation combines the owner-element flags with the local parent's effective flags. |
| [LocalFrame.cpp](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/page/LocalFrame.cpp) | `96c8e2a3b0be12b61642d30483a6c2b81d1d831a` | 1450–1455 | The parent's effective flags include its current Document sandbox flags. |

Together these paths witness a legacy Frame with no element-specific sandbox
restrictions while preserving inherited parent restrictions. The parent union
is essential: an empty owner contribution does not make the child unsandboxed.
This is a static source witness, not an executed browser conformance test.

## Standard gap and source precedence

The selected [HTML sandboxing reference](html-sandboxing--877d6146487f.md)
retains revision `877d6146487f15b15bc8dd0e3c625063a37b456a`.
Its complete original HTML has SHA-256
`877d6cc00dae6c18c16d952fdcbf944acc58919579621b4257fa0ed1888a871f`.
Original raw lines 65758–65761 define the iframe-owned flag set, while
65785–65794 unconditionally refer to that slot for an element embedder.
The [Frame post-connection clauses](html-language-obsolete--877d6146487f--66870e9cdabe.md)
at raw103188–103194 invoke child creation. The
[constructor clauses](html-hosted-global-agent-settings--877d6146487f--03a314e0aa55.md)
at raw71653–71719 consume those creation flags.

The [current HTML sandboxing section](https://html.spec.whatwg.org/multipage/browsers.html#determining-the-creation-sandboxing-flags),
checked on 16 October 2026 before choosing the browser witness, still defines
the slot for iframe while referencing it for an element embedder. This dated
observation adopts no newer edition or additional behavior.

The authoritative bounded qualification is in
[#1124](https://github.com/bj-data-eng/surgeist/issues/1124), section
“Legacy Frame: creation sandbox contribution”. The literal retained standard
is unchanged. That qualification restricts the iframe-slot contribution to
actual Iframe and gives legacy Frame an empty element-specific contribution,
while retaining the actual current node Document's active flags. A raw sandbox
attribute on Frame does not acquire Iframe semantics.

This evidence does not select WebKit's loader, remote-frame behavior, referrer
policy, CSP implementation, navigation, IPC or arbitrary frame-owner families.
Those paths remain governed by their own selected standards and capabilities.
