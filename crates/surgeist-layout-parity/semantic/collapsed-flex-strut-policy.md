# Collapsed Flex strut qualification

Policy `flexbox-collapse-two-round-struts`, edition
`css-flexbox-1-20251014-native-1`, applies to the original no-text source
[`fri07_collapsed_strut_wrapping.html`](../tests/layout/browser_parity/html/flex/fri07_collapsed_strut_wrapping.html)
(SHA-256 `b6b600bc58f098069f88ceca277602cc1ceccb33a6fd5e3570938e600be93f44`).
It covers both box modes and directions under a genuine Normal,
absolute-root-local request. It grants no authority for labeled text baselines,
Range geometry or unsupported source/output channels.

The pinned [Flexbox §4.4 and collapse algorithm](../../../references/css-flexbox-1--CRD-css-flexbox-1-20251014--b0e3b097bc6d.md)
(SHA-256 `33a9f6f67200b90d163f7d24960d04fe64ea195c747fcf1254b5a57faf157d2d`)
require first-round cross struts followed by line collection with zero collapsed
main size. Preserve source margins and gaps during that collection, then exclude
collapsed items, margins and their gaps from committed placement. The normalized
Native contract is documented in [Layout reference](../docs/reference.md).
The captured browser retains different collapsed geometry; its observations
remain raw assertions, separately pinned by each specification.

The source root has inner width130, column gap10 and row gap8. Source children
have fixed width/height50×20,80×35,50×30,100×25,50×15. Children1 and3 collapse;
their inline-start margins are5 and7, and child1 has inline-end auto margin.
First-round collection puts each item on its own line, recording struts35 and25.
Second-round source lines are `[0,1,2]` and `[3,4]`: collection advances125
and67, while adding child3 to the first line would require142>130.
Committed visible main advance is110 on line1 and50 on line2. Cross sizes35
and25 yield root height68 and second-line start43. Definite visible child heights
remain20,30,15. RTL mirrors their border intervals within width130.

| Source path | Index | LTR position | RTL position | Size/content size |
| --- | ---: | --- | --- | --- |
| `/` | 0 | (0,0) | (0,0) | 130×68 |
| `/0` | 0 | (0,0) | (80,0) | 50×20 |
| `/1` | 1 | (0,0) | (0,0) | 0×0 |
| `/2` | 2 | (60,0) | (20,0) | 50×30 |
| `/3` | 3 | (0,0) | (0,0) | 0×0 |
| `/4` | 4 | (0,43) | (80,43) | 50×15 |

All six nodes publish both unrounded and final packets; integral arithmetic makes
their phase values equal. Every edge is zero. Collapsed nodes publish the complete
hidden packet: named facts Unknown and no ScrollGeometry. Visible empty Flex
children actually perform an empty eligible-item collection, establishing all
four named facts Absent. Root named facts remain Unknown; generic physical
baseline synthesis provides no named authority. See the independent owning
[empty-collection tests](../src/flex_tests/named_empty_absence.rs) and
[collapse tests](../src/flex_tests/lines_distribution_tests.rs).

Each noncollapsed node has complete local canonical geometry. Its border,
padding, content, scrollport, overflow and optimal-viewing rectangles are
`(0,0,width,height)` from the table. Visible overflow on both axes gives no clips,
no gutters, no scrollbar reservation and zero signed physical ranges. Initial
scroll-padding auto resolves to the owning Native zero policy; resolved padding
and target scroll margins are zero. Target axes match the actual node direction,
target border rect is local, both snap alignments are None and stop is Normal;
container snap type is None. Content size measures the canonical overflow
envelope unioned with the content-origin anchor, including the padding seed,
not the visible child hull or zero range span. See the public
[scroll contract](../docs/explanation.md) and independent
[geometry/content-size tests](../src/flex_tests/intrinsic_absolute_scroll_tests.rs).

Qualification first checks this complete source-derived Native oracle and both
packet slots, including fields without browser observations. It then checks
bijection of the complete requested raw difference set with exact declared
browser/Native pairs. Ordinary browser tolerance is unchanged; pair drift is
exactly rejected after signed-zero normalization. A new, removed or changed
pair, unexpected pass, identity change, missing packet, late failure or nonempty
Text/fragment/native-inline output refuses qualification. Raw comparison still
reports every difference. The verdict expresses this named Native policy;
it does not assert browser identity, browser conformance or whole-corpus success.
