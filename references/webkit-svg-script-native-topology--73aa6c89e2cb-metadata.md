# WebKit native SVG Script topology witness

Inspected 2026-10-15 at revision `73aa6c89e2cb77c46184a81aec944e4ab99d114d`.
This is metadata and a behavioral summary, not a source-code transcription.
The original files retain their copyright and LGPL notices in private research
at `tmp/work/native-svg-script-topology/`. No browser implementation is ingested.

| Original file | Complete SHA-256 | One-based source lines |
| --- | --- | --- |
| [ScriptElement.cpp](https://raw.githubusercontent.com/WebKit/WebKit/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/dom/ScriptElement.cpp) | `d7dcd262d444dc9cf4d94ddc8431e88432bef926dd9c8ee7c28595bc64619570` | 104–108, 245–249 |
| [Node.h](https://raw.githubusercontent.com/WebKit/WebKit/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/dom/Node.h) | `21f557d8cb2f7db135ba6dedb4d8538d62ec814814678c050a5b95e8576bbc33` | 426 |
| [Node.cpp](https://raw.githubusercontent.com/WebKit/WebKit/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/dom/Node.cpp) | `00cc2c7c7cdb932e6a2a4105b75323d53dd4601a9591738158eb4f17f1f8914e` | 1477–1498 |
| [ContainerNodeAlgorithms.cpp](https://raw.githubusercontent.com/WebKit/WebKit/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/dom/ContainerNodeAlgorithms.cpp) | `e72c0cf2ebaee615e5f28185935b6c7db9e40e1cb64c21178f14f8bb569ef4d8` | 47–91 |
| [ContainerNode.cpp](https://raw.githubusercontent.com/WebKit/WebKit/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/dom/ContainerNode.cpp) | `c3678be009bc51cb9cd4ccbc86ca7949c98423f9de74c26a1ab30acfede18e2c` | 405–430 |

The existing [SVGScriptElement witness](webkit-svg-script-clone--73aa6c89e2cb-metadata.md)
also covers that file's lines 83–94: SVG insertion and post-connection delegate
to ScriptElement. ScriptElement's non-parser post-connection invokes preparation;
preparation checks connectedness. Node stores connectedness separately from
shadow-tree membership. Its shadow-including root follows actual ShadowRoot hosts.
ContainerNodeAlgorithms propagates connected insertion into hosted shadow roots,
visiting each root immediately after its host. ContainerNode then runs collected
post-connection targets in that order. Detached insertion takes a separate path.

This witnesses a bounded native topology correction, not adoption of WebKit's
shared HTML/SVG preparation algorithm, type handling, empty-content guards,
resource loading, scripting policy or notification behavior. Those remain outside
this witness's selected scope.

The authoritative native determination is
[#1200](https://github.com/bj-data-eng/surgeist/issues/1200), section
“Native SVG Script topology and closed processing”. It combines the existing
[Tiny processing operation](svg-tiny12-script-processing--REC-20081222--2f36eb76440c.md#ScriptContentProcessing)
with the selected [DOM source](dom-foundation--editor-capture-20261010--08dff0a4827f.md):
connectedness at line 255, insertion's static post-connection list at 586–596,
and shadow-including order/root at 2779–2783. It does not change the parser's
ordinary-tree domain or select SVG2 wholesale. SVG use-instance shadow trees
are outside this canonical author-created ShadowRoot determination.

The newer SVG2 script chapter was checked before selecting the browser witness;
its HTML-equivalence statement does not supply the missing complete native
algorithm. The existing [SVG2 observation record](svg-script-observations--capture-20261012-metadata.md)
retains that distinction. No new SVG2 normative behavior is selected here.
