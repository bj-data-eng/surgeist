// These authored cases exercise explicit public property-parser behavior.
// Stable IDs, canonical names, and property-specific values are recorded with
// each case so its public metadata and parser observables can be asserted.

#[derive(Clone, Copy, Debug)]
pub struct PropertyVector {
    pub id: &'static str,
    pub canonical_name: &'static str,
    pub authored_value: &'static str,
}

macro_rules! vector {
    ($id:literal, $canonical_name:literal, $authored_value:literal) => {
        PropertyVector {
            id: $id,
            canonical_name: $canonical_name,
            authored_value: $authored_value,
        }
    };
}

// Every non-`all` value is an ordinary typed value that must reach generated
// property dispatch. The `all` row deliberately exercises its global contract.
pub const PROPERTY_POSITIVE_VECTORS: &[PropertyVector] = &[
    vector!("baseline.property.all", "all", "inherit"),
    vector!("baseline.property.display", "display", "block"),
    vector!(
        "official.property.border-collapse",
        "border-collapse",
        "collapse"
    ),
    vector!(
        "official.property.border-spacing",
        "border-spacing",
        "2px 3px"
    ),
    vector!("official.property.caption-side", "caption-side", "bottom"),
    vector!(
        "official.property.clip",
        "clip",
        "rect(auto, 10px, 20px, -1px)"
    ),
    vector!("official.property.empty-cells", "empty-cells", "hide"),
    vector!("official.property.orphans", "orphans", "3"),
    vector!(
        "official.property.page-break-after",
        "page-break-after",
        "right"
    ),
    vector!(
        "official.property.page-break-before",
        "page-break-before",
        "always"
    ),
    vector!(
        "official.property.page-break-inside",
        "page-break-inside",
        "avoid"
    ),
    vector!("official.property.quotes", "quotes", "\"open\" \"close\""),
    vector!("official.property.table-layout", "table-layout", "fixed"),
    vector!("official.property.widows", "widows", "4"),
    vector!("official.property.word-spacing", "word-spacing", "-0.25em"),
    vector!(
        "official.property.text-combine-upright",
        "text-combine-upright",
        "all"
    ),
    vector!(
        "official.property.text-orientation",
        "text-orientation",
        "sideways"
    ),
    vector!(
        "official.property.unicode-bidi",
        "unicode-bidi",
        "isolate-override"
    ),
    vector!(
        "official.property.caret-color",
        "caret-color",
        "rebeccapurple"
    ),
    vector!("official.property.outline-offset", "outline-offset", "-2px"),
    vector!("official.property.resize", "resize", "horizontal"),
    vector!("official.property.contain", "contain", "paint size"),
    vector!(
        "official.property.transform-box",
        "transform-box",
        "view-box"
    ),
    vector!(
        "official.property.background-blend-mode",
        "background-blend-mode",
        "multiply, luminosity"
    ),
    vector!("official.property.isolation", "isolation", "isolate"),
    vector!(
        "official.property.mix-blend-mode",
        "mix-blend-mode",
        "soft-light"
    ),
    vector!("baseline.property.box-sizing", "box-sizing", "border-box"),
    vector!("baseline.property.position", "position", "sticky"),
    vector!("baseline.property.direction", "direction", "rtl"),
    vector!("baseline.property.overflow", "overflow", "hidden scroll"),
    vector!("baseline.property.overflow-x", "overflow-x", "clip"),
    vector!("baseline.property.overflow-y", "overflow-y", "visible"),
    vector!(
        "baseline.property.flex-direction",
        "flex-direction",
        "column-reverse"
    ),
    vector!("baseline.property.flex-wrap", "flex-wrap", "wrap-reverse"),
    vector!("baseline.property.float", "float", "left"),
    vector!("baseline.property.clear", "clear", "both"),
    vector!(
        "baseline.property.align-content",
        "align-content",
        "space-between"
    ),
    vector!(
        "baseline.property.justify-content",
        "justify-content",
        "safe center"
    ),
    vector!(
        "baseline.property.align-items",
        "align-items",
        "first baseline"
    ),
    vector!(
        "baseline.property.align-self",
        "align-self",
        "safe flex-end"
    ),
    vector!(
        "baseline.property.justify-items",
        "justify-items",
        "stretch"
    ),
    vector!("baseline.property.justify-self", "justify-self", "center"),
    vector!(
        "baseline.property.place-content",
        "place-content",
        "center end"
    ),
    vector!("baseline.property.place-items", "place-items", "stretch"),
    vector!("baseline.property.place-self", "place-self", "end center"),
    vector!("baseline.property.visibility", "visibility", "collapse"),
    vector!("baseline.property.content", "content", "\"Chapter \""),
    vector!(
        "baseline.property.content-visibility",
        "content-visibility",
        "auto"
    ),
    vector!(
        "baseline.property.list-style-type",
        "list-style-type",
        "square"
    ),
    vector!(
        "baseline.property.list-style-position",
        "list-style-position",
        "inside"
    ),
    vector!(
        "baseline.property.list-style-image",
        "list-style-image",
        "url(marker.svg)"
    ),
    vector!(
        "baseline.property.list-style",
        "list-style",
        "url(marker.svg) inside square"
    ),
    vector!(
        "baseline.property.counter-reset",
        "counter-reset",
        "section 2"
    ),
    vector!(
        "baseline.property.counter-increment",
        "counter-increment",
        "section 1"
    ),
    vector!("baseline.property.counter-set", "counter-set", "section 3"),
    vector!("baseline.property.width", "width", "calc(100% - 12px)"),
    vector!("baseline.property.height", "height", "auto"),
    vector!("baseline.property.min-width", "min-width", "0"),
    vector!("baseline.property.min-height", "min-height", "min-content"),
    vector!("baseline.property.max-width", "max-width", "max-content"),
    vector!("baseline.property.max-height", "max-height", "fit-content"),
    vector!("baseline.property.flex-basis", "flex-basis", "10rem"),
    vector!("baseline.property.gap", "gap", "12px"),
    vector!("baseline.property.row-gap", "row-gap", "normal"),
    vector!("baseline.property.column-gap", "column-gap", "5%"),
    vector!(
        "baseline.property.grid-flow-tolerance",
        "grid-flow-tolerance",
        "infinite"
    ),
    vector!(
        "baseline.property.grid-template-rows",
        "grid-template-rows",
        "[top] 100px 1fr"
    ),
    vector!(
        "baseline.property.grid-template-columns",
        "grid-template-columns",
        "repeat(2, minmax(10px, 1fr))"
    ),
    vector!(
        "baseline.property.grid-template-areas",
        "grid-template-areas",
        "\"header header\" \"nav main\""
    ),
    vector!(
        "baseline.property.grid-template",
        "grid-template",
        "100px 1fr / repeat(2, minmax(10px, 1fr))"
    ),
    vector!(
        "baseline.property.grid-auto-rows",
        "grid-auto-rows",
        "minmax(10px, auto)"
    ),
    vector!(
        "baseline.property.grid-auto-columns",
        "grid-auto-columns",
        "fit-content(20%)"
    ),
    vector!(
        "baseline.property.grid-auto-flow",
        "grid-auto-flow",
        "column dense"
    ),
    vector!(
        "baseline.property.grid-row-start",
        "grid-row-start",
        "span 2 main"
    ),
    vector!("baseline.property.grid-row-end", "grid-row-end", "auto"),
    vector!(
        "baseline.property.grid-column-start",
        "grid-column-start",
        "nav"
    ),
    vector!("baseline.property.grid-column-end", "grid-column-end", "4"),
    vector!("baseline.property.grid-row", "grid-row", "1 / span 2"),
    vector!("baseline.property.grid-column", "grid-column", "nav / main"),
    vector!(
        "baseline.property.grid-area",
        "grid-area",
        "header / 1 / span 2 / main"
    ),
    vector!(
        "baseline.property.grid",
        "grid",
        "auto-flow dense 12px / repeat(auto-fit, 10px)"
    ),
    vector!("baseline.property.font-size", "font-size", "16px"),
    vector!("baseline.property.line-height", "line-height", "normal"),
    vector!(
        "baseline.property.writing-mode",
        "writing-mode",
        "vertical-rl"
    ),
    vector!("baseline.property.text-align", "text-align", "start"),
    vector!(
        "baseline.property.text-align-last",
        "text-align-last",
        "justify"
    ),
    vector!(
        "baseline.property.text-indent",
        "text-indent",
        "1rem hanging each-line"
    ),
    vector!(
        "baseline.property.vertical-align",
        "vertical-align",
        "super"
    ),
    vector!(
        "baseline.property.font-family",
        "font-family",
        "\"Avenir Next\", sans-serif"
    ),
    vector!(
        "baseline.property.font",
        "font",
        "italic small-caps 700 condensed 16px/normal \"Avenir Next\", sans-serif"
    ),
    vector!("baseline.property.font-weight", "font-weight", "725"),
    vector!("baseline.property.font-style", "font-style", "italic"),
    vector!(
        "baseline.property.font-stretch",
        "font-stretch",
        "semi-condensed"
    ),
    vector!(
        "baseline.property.font-variant",
        "font-variant",
        "small-caps"
    ),
    vector!(
        "baseline.property.font-feature-settings",
        "font-feature-settings",
        "\"kern\" on, \"liga\" 0"
    ),
    vector!(
        "baseline.property.letter-spacing",
        "letter-spacing",
        "0.1em"
    ),
    vector!("baseline.property.text-wrap", "text-wrap", "balance"),
    vector!("baseline.property.white-space", "white-space", "pre-wrap"),
    vector!("baseline.property.word-break", "word-break", "keep-all"),
    vector!(
        "baseline.property.overflow-wrap",
        "overflow-wrap",
        "anywhere"
    ),
    vector!(
        "baseline.property.text-overflow",
        "text-overflow",
        "ellipsis"
    ),
    vector!(
        "baseline.property.text-decoration",
        "text-decoration",
        "underline dotted white 3px"
    ),
    vector!(
        "baseline.property.text-decoration-line",
        "text-decoration-line",
        "underline overline"
    ),
    vector!(
        "baseline.property.text-decoration-color",
        "text-decoration-color",
        "black"
    ),
    vector!(
        "baseline.property.text-decoration-style",
        "text-decoration-style",
        "wavy"
    ),
    vector!(
        "baseline.property.text-decoration-thickness",
        "text-decoration-thickness",
        "2px"
    ),
    vector!(
        "baseline.property.text-transform",
        "text-transform",
        "uppercase"
    ),
    vector!("baseline.property.inset", "inset", "auto 10px 5%"),
    vector!("baseline.property.top", "top", "auto"),
    vector!("baseline.property.right", "right", "10px"),
    vector!("baseline.property.bottom", "bottom", "5%"),
    vector!("baseline.property.left", "left", "calc(3px + 4%)"),
    vector!("baseline.property.z-index", "z-index", "-2"),
    vector!(
        "baseline.property.box-decoration-break",
        "box-decoration-break",
        "clone"
    ),
    vector!("baseline.property.margin", "margin", "auto 10px 5%"),
    vector!("baseline.property.margin-top", "margin-top", "auto"),
    vector!("baseline.property.margin-right", "margin-right", "10px"),
    vector!("baseline.property.margin-bottom", "margin-bottom", "5%"),
    vector!(
        "baseline.property.margin-left",
        "margin-left",
        "calc(3px + 4%)"
    ),
    vector!(
        "baseline.property.padding",
        "padding",
        "1px 2% calc(3px + 4%) 0"
    ),
    vector!("baseline.property.padding-top", "padding-top", "12px"),
    vector!("baseline.property.padding-right", "padding-right", "2%"),
    vector!(
        "baseline.property.padding-bottom",
        "padding-bottom",
        "calc(3px + 4%)"
    ),
    vector!("baseline.property.padding-left", "padding-left", "0"),
    vector!("baseline.property.border", "border", "solid 2px #fff"),
    vector!("baseline.property.border-top", "border-top", "black dotted"),
    vector!("baseline.property.border-right", "border-right", "1px"),
    vector!("baseline.property.border-bottom", "border-bottom", "#fff"),
    vector!(
        "baseline.property.border-left",
        "border-left",
        "dashed black"
    ),
    vector!(
        "baseline.property.border-width",
        "border-width",
        "1px 2px 3px 4px"
    ),
    vector!(
        "baseline.property.border-top-width",
        "border-top-width",
        "1px"
    ),
    vector!(
        "baseline.property.border-right-width",
        "border-right-width",
        "2px"
    ),
    vector!(
        "baseline.property.border-bottom-width",
        "border-bottom-width",
        "3px"
    ),
    vector!(
        "baseline.property.border-left-width",
        "border-left-width",
        "4px"
    ),
    vector!(
        "official.property.border-image",
        "border-image",
        "url(frame.png) 10 fill / 2 / 1 round"
    ),
    vector!(
        "official.property.border-image-outset",
        "border-image-outset",
        "1 2px 3 4px"
    ),
    vector!(
        "official.property.border-image-repeat",
        "border-image-repeat",
        "round space"
    ),
    vector!(
        "official.property.border-image-slice",
        "border-image-slice",
        "10 fill"
    ),
    vector!(
        "official.property.border-image-source",
        "border-image-source",
        "linear-gradient(red, blue)"
    ),
    vector!(
        "official.property.border-image-width",
        "border-image-width",
        "1 auto 25% 4px"
    ),
    vector!("baseline.property.color", "color", "black"),
    vector!("baseline.property.background", "background", "#fff"),
    vector!(
        "baseline.property.background-color",
        "background-color",
        "transparent"
    ),
    vector!("baseline.property.border-color", "border-color", "black"),
    vector!(
        "baseline.property.border-top-color",
        "border-top-color",
        "black"
    ),
    vector!(
        "baseline.property.border-right-color",
        "border-right-color",
        "white"
    ),
    vector!(
        "baseline.property.border-bottom-color",
        "border-bottom-color",
        "transparent"
    ),
    vector!(
        "baseline.property.border-left-color",
        "border-left-color",
        "#fff"
    ),
    vector!(
        "baseline.property.background-image",
        "background-image",
        "url(\"hero.png\"), none"
    ),
    vector!(
        "baseline.property.background-position",
        "background-position",
        "left 10px top 20%"
    ),
    vector!(
        "baseline.property.background-size",
        "background-size",
        "cover, 10px auto"
    ),
    vector!(
        "baseline.property.background-repeat",
        "background-repeat",
        "repeat-x, no-repeat round"
    ),
    vector!(
        "baseline.property.background-origin",
        "background-origin",
        "content-box"
    ),
    vector!(
        "baseline.property.background-clip",
        "background-clip",
        "padding-box"
    ),
    vector!(
        "baseline.property.background-attachment",
        "background-attachment",
        "fixed, local"
    ),
    vector!(
        "baseline.property.border-style",
        "border-style",
        "none hidden dotted dashed"
    ),
    vector!(
        "baseline.property.border-top-style",
        "border-top-style",
        "solid"
    ),
    vector!(
        "baseline.property.border-right-style",
        "border-right-style",
        "double"
    ),
    vector!(
        "baseline.property.border-bottom-style",
        "border-bottom-style",
        "ridge"
    ),
    vector!(
        "baseline.property.border-left-style",
        "border-left-style",
        "outset"
    ),
    vector!(
        "baseline.property.border-radius",
        "border-radius",
        "1px 2px 3px / 4px 5px"
    ),
    vector!(
        "baseline.property.border-top-left-radius",
        "border-top-left-radius",
        "4px 10%"
    ),
    vector!(
        "baseline.property.border-top-right-radius",
        "border-top-right-radius",
        "1px"
    ),
    vector!(
        "baseline.property.border-bottom-right-radius",
        "border-bottom-right-radius",
        "10%"
    ),
    vector!(
        "baseline.property.border-bottom-left-radius",
        "border-bottom-left-radius",
        "calc(1px + 2%)"
    ),
    vector!(
        "baseline.property.box-shadow",
        "box-shadow",
        "inset 1px 2px 3px 4px black"
    ),
    vector!(
        "official.property.image-orientation",
        "image-orientation",
        "90deg flip"
    ),
    vector!(
        "official.property.image-rendering",
        "image-rendering",
        "crisp-edges"
    ),
    vector!("official.property.object-fit", "object-fit", "scale-down"),
    vector!("baseline.property.opacity", "opacity", "0.5"),
    vector!("baseline.property.flex-grow", "flex-grow", "2"),
    vector!("baseline.property.flex-shrink", "flex-shrink", "0"),
    vector!("baseline.property.order", "order", "-2"),
    vector!("baseline.property.flex", "flex", "2 0 10rem"),
    vector!(
        "baseline.property.justify-tracks",
        "justify-tracks",
        "space-evenly"
    ),
    vector!("baseline.property.align-tracks", "align-tracks", "center"),
    vector!("baseline.property.aspect-ratio", "aspect-ratio", "1.5"),
    vector!(
        "baseline.property.scrollbar-width",
        "scrollbar-width",
        "thin"
    ),
    vector!("baseline.property.cursor", "cursor", "grab"),
    vector!("baseline.property.pointer-events", "pointer-events", "none"),
    vector!("baseline.property.user-select", "user-select", "text"),
    vector!("baseline.property.outline", "outline", "thick dotted white"),
    vector!("baseline.property.outline-color", "outline-color", "black"),
    vector!("baseline.property.outline-style", "outline-style", "auto"),
    vector!("baseline.property.outline-width", "outline-width", "2px"),
    vector!(
        "baseline.property.transform",
        "transform",
        "translate(10px, 20px) rotate(45deg) scale(1.5)"
    ),
    vector!(
        "baseline.property.transform-origin",
        "transform-origin",
        "center top"
    ),
    vector!("baseline.property.translate", "translate", "10px 20px"),
    vector!("baseline.property.rotate", "rotate", "45deg"),
    vector!("baseline.property.scale", "scale", "1.5 2"),
    vector!(
        "baseline.property.filter",
        "filter",
        "blur(4px) opacity(50%)"
    ),
    vector!(
        "baseline.property.backdrop-filter",
        "backdrop-filter",
        "none"
    ),
    vector!(
        "baseline.property.clip-path",
        "clip-path",
        "circle(50% at center)"
    ),
    vector!(
        "baseline.property.mask",
        "mask",
        "url(mask.png) center / contain no-repeat"
    ),
    vector!(
        "baseline.property.mask-image",
        "mask-image",
        "url(mask.png), none"
    ),
    vector!("baseline.property.mask-size", "mask-size", "contain"),
    vector!("baseline.property.mask-position", "mask-position", "center"),
    vector!("baseline.property.mask-repeat", "mask-repeat", "repeat"),
    vector!(
        "baseline.property.transition-property",
        "transition-property",
        "opacity, transform"
    ),
    vector!(
        "baseline.property.transition-duration",
        "transition-duration",
        "150ms, 2s"
    ),
    vector!(
        "baseline.property.transition-delay",
        "transition-delay",
        "20ms"
    ),
    vector!(
        "baseline.property.transition-timing-function",
        "transition-timing-function",
        "ease-in, cubic-bezier(0.1, 0.2, 0.3, 1)"
    ),
    vector!(
        "baseline.property.transition",
        "transition",
        "opacity 150ms ease-in 20ms, transform 2s linear"
    ),
    vector!(
        "baseline.property.animation-name",
        "animation-name",
        "fade, none"
    ),
    vector!(
        "baseline.property.animation-duration",
        "animation-duration",
        "1s"
    ),
    vector!(
        "baseline.property.animation-delay",
        "animation-delay",
        "200ms"
    ),
    vector!(
        "baseline.property.animation-timing-function",
        "animation-timing-function",
        "ease-out"
    ),
    vector!(
        "baseline.property.animation-iteration-count",
        "animation-iteration-count",
        "2, infinite"
    ),
    vector!(
        "baseline.property.animation-direction",
        "animation-direction",
        "alternate"
    ),
    vector!(
        "baseline.property.animation-fill-mode",
        "animation-fill-mode",
        "both"
    ),
    vector!(
        "baseline.property.animation-play-state",
        "animation-play-state",
        "running, paused"
    ),
    vector!(
        "baseline.property.animation",
        "animation",
        "fade 1s ease-in 200ms 3 alternate both running"
    ),
];

// Every rejection is a property-specific wrong keyword, cross-family value,
// cardinality error, or numeric/range boundary. The `all` row deliberately uses
// ordinary typed syntax so its generated dispatch arm rejects it.
pub const PROPERTY_NEGATIVE_VECTORS: &[PropertyVector] = &[
    vector!("baseline.property.all", "all", "block"),
    vector!("baseline.property.display", "display", "inline"),
    vector!(
        "official.property.border-collapse",
        "border-collapse",
        "auto"
    ),
    vector!("official.property.border-spacing", "border-spacing", "-1px"),
    vector!("official.property.caption-side", "caption-side", "left"),
    vector!("official.property.clip", "clip", "rect(1px, 2px, 3px)"),
    vector!("official.property.empty-cells", "empty-cells", "auto"),
    vector!("official.property.orphans", "orphans", "0"),
    vector!(
        "official.property.page-break-after",
        "page-break-after",
        "page"
    ),
    vector!(
        "official.property.page-break-before",
        "page-break-before",
        "recto"
    ),
    vector!(
        "official.property.page-break-inside",
        "page-break-inside",
        "left"
    ),
    vector!("official.property.quotes", "quotes", "\"open\""),
    vector!("official.property.table-layout", "table-layout", "collapse"),
    vector!("official.property.widows", "widows", "0"),
    vector!("official.property.word-spacing", "word-spacing", "auto"),
    vector!(
        "official.property.text-combine-upright",
        "text-combine-upright",
        "sideways"
    ),
    vector!(
        "official.property.text-orientation",
        "text-orientation",
        "auto"
    ),
    vector!(
        "official.property.unicode-bidi",
        "unicode-bidi",
        "isolate isolate"
    ),
    vector!("official.property.caret-color", "caret-color", "auto auto"),
    vector!("official.property.outline-offset", "outline-offset", "auto"),
    vector!("official.property.resize", "resize", "inline"),
    vector!("official.property.contain", "contain", "size size"),
    vector!(
        "official.property.transform-box",
        "transform-box",
        "padding-box"
    ),
    vector!(
        "official.property.background-blend-mode",
        "background-blend-mode",
        "multiply luminosity"
    ),
    vector!("official.property.isolation", "isolation", "none"),
    vector!(
        "official.property.mix-blend-mode",
        "mix-blend-mode",
        "multiply, screen"
    ),
    vector!("baseline.property.box-sizing", "box-sizing", "padding-box"),
    vector!("baseline.property.position", "position", "running"),
    vector!("baseline.property.direction", "direction", "block"),
    vector!("baseline.property.overflow", "overflow", "auto"),
    vector!("baseline.property.overflow-x", "overflow-x", "auto"),
    vector!("baseline.property.overflow-y", "overflow-y", "auto"),
    vector!("baseline.property.flex-direction", "flex-direction", "wrap"),
    vector!("baseline.property.flex-wrap", "flex-wrap", "column"),
    vector!("baseline.property.float", "float", "center"),
    vector!("baseline.property.clear", "clear", "start"),
    vector!("baseline.property.align-content", "align-content", "auto"),
    vector!(
        "baseline.property.justify-content",
        "justify-content",
        "auto"
    ),
    vector!(
        "baseline.property.align-items",
        "align-items",
        "space-between"
    ),
    vector!(
        "baseline.property.align-self",
        "align-self",
        "space-between"
    ),
    vector!(
        "baseline.property.justify-items",
        "justify-items",
        "space-between"
    ),
    vector!(
        "baseline.property.justify-self",
        "justify-self",
        "space-between"
    ),
    vector!("baseline.property.place-content", "place-content", "auto"),
    vector!(
        "baseline.property.place-items",
        "place-items",
        "space-between"
    ),
    vector!(
        "baseline.property.place-self",
        "place-self",
        "space-between"
    ),
    vector!("baseline.property.visibility", "visibility", "auto"),
    vector!("baseline.property.content", "content", "contents"),
    vector!(
        "baseline.property.content-visibility",
        "content-visibility",
        "collapse"
    ),
    vector!(
        "baseline.property.list-style-type",
        "list-style-type",
        "symbols(cyclic \"*\" \"+\")"
    ),
    vector!(
        "baseline.property.list-style-position",
        "list-style-position",
        "center"
    ),
    vector!(
        "baseline.property.list-style-image",
        "list-style-image",
        "red"
    ),
    vector!(
        "baseline.property.list-style",
        "list-style",
        "inside outside"
    ),
    vector!(
        "baseline.property.counter-reset",
        "counter-reset",
        "none item"
    ),
    vector!(
        "baseline.property.counter-increment",
        "counter-increment",
        "1"
    ),
    vector!("baseline.property.counter-set", "counter-set", "none item"),
    vector!("baseline.property.width", "width", "solid"),
    vector!("baseline.property.height", "height", "solid"),
    vector!("baseline.property.min-width", "min-width", "solid"),
    vector!("baseline.property.min-height", "min-height", "solid"),
    vector!("baseline.property.max-width", "max-width", "solid"),
    vector!("baseline.property.max-height", "max-height", "solid"),
    vector!("baseline.property.flex-basis", "flex-basis", "solid"),
    vector!("baseline.property.gap", "gap", "auto"),
    vector!("baseline.property.row-gap", "row-gap", "auto"),
    vector!("baseline.property.column-gap", "column-gap", "auto"),
    vector!(
        "baseline.property.grid-flow-tolerance",
        "grid-flow-tolerance",
        "solid"
    ),
    vector!(
        "baseline.property.grid-template-rows",
        "grid-template-rows",
        "solid"
    ),
    vector!(
        "baseline.property.grid-template-columns",
        "grid-template-columns",
        "solid"
    ),
    vector!(
        "baseline.property.grid-template-areas",
        "grid-template-areas",
        "\"a a\" \"a .\""
    ),
    vector!("baseline.property.grid-template", "grid-template", "solid"),
    vector!(
        "baseline.property.grid-auto-rows",
        "grid-auto-rows",
        "solid"
    ),
    vector!(
        "baseline.property.grid-auto-columns",
        "grid-auto-columns",
        "solid"
    ),
    vector!("baseline.property.grid-auto-flow", "grid-auto-flow", "left"),
    vector!("baseline.property.grid-row-start", "grid-row-start", "0"),
    vector!("baseline.property.grid-row-end", "grid-row-end", "0"),
    vector!(
        "baseline.property.grid-column-start",
        "grid-column-start",
        "0"
    ),
    vector!("baseline.property.grid-column-end", "grid-column-end", "0"),
    vector!("baseline.property.grid-row", "grid-row", "0"),
    vector!("baseline.property.grid-column", "grid-column", "0"),
    vector!("baseline.property.grid-area", "grid-area", "0"),
    vector!("baseline.property.grid", "grid", "auto-flow"),
    vector!("baseline.property.font-size", "font-size", "auto"),
    vector!("baseline.property.line-height", "line-height", "auto"),
    vector!("baseline.property.writing-mode", "writing-mode", "lr"),
    vector!("baseline.property.text-align", "text-align", "auto"),
    vector!(
        "baseline.property.text-align-last",
        "text-align-last",
        "match-parent"
    ),
    vector!("baseline.property.text-indent", "text-indent", "auto"),
    vector!("baseline.property.vertical-align", "vertical-align", "auto"),
    vector!(
        "baseline.property.font-family",
        "font-family",
        "sans-serif,"
    ),
    vector!("baseline.property.font", "font", "bold sans-serif"),
    vector!("baseline.property.font-weight", "font-weight", "1001"),
    vector!("baseline.property.font-style", "font-style", "bold"),
    vector!("baseline.property.font-stretch", "font-stretch", "wide"),
    vector!("baseline.property.font-variant", "font-variant", "italic"),
    vector!(
        "baseline.property.font-feature-settings",
        "font-feature-settings",
        "\"abc\" on"
    ),
    vector!("baseline.property.letter-spacing", "letter-spacing", "auto"),
    vector!("baseline.property.text-wrap", "text-wrap", "auto"),
    vector!("baseline.property.white-space", "white-space", "balance"),
    vector!("baseline.property.word-break", "word-break", "nowrap"),
    vector!(
        "baseline.property.overflow-wrap",
        "overflow-wrap",
        "ellipsis"
    ),
    vector!("baseline.property.text-overflow", "text-overflow", "wrap"),
    vector!(
        "baseline.property.text-decoration",
        "text-decoration",
        "underline underline"
    ),
    vector!(
        "baseline.property.text-decoration-line",
        "text-decoration-line",
        "underline underline"
    ),
    vector!(
        "baseline.property.text-decoration-color",
        "text-decoration-color",
        "black white"
    ),
    vector!(
        "baseline.property.text-decoration-style",
        "text-decoration-style",
        "auto"
    ),
    vector!(
        "baseline.property.text-decoration-thickness",
        "text-decoration-thickness",
        "-1px"
    ),
    vector!("baseline.property.text-transform", "text-transform", "wrap"),
    vector!("baseline.property.inset", "inset", "solid"),
    vector!("baseline.property.top", "top", "solid"),
    vector!("baseline.property.right", "right", "solid"),
    vector!("baseline.property.bottom", "bottom", "solid"),
    vector!("baseline.property.left", "left", "solid"),
    vector!("baseline.property.z-index", "z-index", "1.5"),
    vector!(
        "baseline.property.box-decoration-break",
        "box-decoration-break",
        "auto"
    ),
    vector!("baseline.property.margin", "margin", "solid"),
    vector!("baseline.property.margin-top", "margin-top", "solid"),
    vector!("baseline.property.margin-right", "margin-right", "solid"),
    vector!("baseline.property.margin-bottom", "margin-bottom", "solid"),
    vector!("baseline.property.margin-left", "margin-left", "solid"),
    vector!("baseline.property.padding", "padding", "auto"),
    vector!("baseline.property.padding-top", "padding-top", "auto"),
    vector!("baseline.property.padding-right", "padding-right", "auto"),
    vector!("baseline.property.padding-bottom", "padding-bottom", "auto"),
    vector!("baseline.property.padding-left", "padding-left", "auto"),
    vector!("baseline.property.border", "border", "solid dotted"),
    vector!("baseline.property.border-top", "border-top", "solid dotted"),
    vector!(
        "baseline.property.border-right",
        "border-right",
        "solid dotted"
    ),
    vector!(
        "baseline.property.border-bottom",
        "border-bottom",
        "solid dotted"
    ),
    vector!(
        "baseline.property.border-left",
        "border-left",
        "solid dotted"
    ),
    vector!("baseline.property.border-width", "border-width", "10%"),
    vector!(
        "baseline.property.border-top-width",
        "border-top-width",
        "10%"
    ),
    vector!(
        "baseline.property.border-right-width",
        "border-right-width",
        "10%"
    ),
    vector!(
        "baseline.property.border-bottom-width",
        "border-bottom-width",
        "10%"
    ),
    vector!(
        "baseline.property.border-left-width",
        "border-left-width",
        "10%"
    ),
    vector!(
        "official.property.border-image",
        "border-image",
        "url(frame.png) / 2"
    ),
    vector!(
        "official.property.border-image-outset",
        "border-image-outset",
        "-1"
    ),
    vector!(
        "official.property.border-image-repeat",
        "border-image-repeat",
        "repeat round stretch"
    ),
    vector!(
        "official.property.border-image-slice",
        "border-image-slice",
        "10 20 30 40 50"
    ),
    vector!(
        "official.property.border-image-source",
        "border-image-source",
        "cover"
    ),
    vector!(
        "official.property.border-image-width",
        "border-image-width",
        "-1"
    ),
    vector!("baseline.property.color", "color", "black white"),
    vector!("baseline.property.background", "background", "#fff #000"),
    vector!(
        "baseline.property.background-color",
        "background-color",
        "black white"
    ),
    vector!(
        "baseline.property.border-color",
        "border-color",
        "black white black white black"
    ),
    vector!(
        "baseline.property.border-top-color",
        "border-top-color",
        "black white"
    ),
    vector!(
        "baseline.property.border-right-color",
        "border-right-color",
        "black white"
    ),
    vector!(
        "baseline.property.border-bottom-color",
        "border-bottom-color",
        "black white"
    ),
    vector!(
        "baseline.property.border-left-color",
        "border-left-color",
        "black white"
    ),
    vector!(
        "baseline.property.background-image",
        "background-image",
        "url(\"\")"
    ),
    vector!(
        "baseline.property.background-position",
        "background-position",
        "left right"
    ),
    vector!(
        "baseline.property.background-size",
        "background-size",
        "solid"
    ),
    vector!(
        "baseline.property.background-repeat",
        "background-repeat",
        "solid"
    ),
    vector!(
        "baseline.property.background-origin",
        "background-origin",
        "margin-box"
    ),
    vector!(
        "baseline.property.background-clip",
        "background-clip",
        "margin-box"
    ),
    vector!(
        "baseline.property.background-attachment",
        "background-attachment",
        "sticky"
    ),
    vector!("baseline.property.border-style", "border-style", "auto"),
    vector!(
        "baseline.property.border-top-style",
        "border-top-style",
        "auto"
    ),
    vector!(
        "baseline.property.border-right-style",
        "border-right-style",
        "auto"
    ),
    vector!(
        "baseline.property.border-bottom-style",
        "border-bottom-style",
        "auto"
    ),
    vector!(
        "baseline.property.border-left-style",
        "border-left-style",
        "auto"
    ),
    vector!("baseline.property.border-radius", "border-radius", "-1px"),
    vector!(
        "baseline.property.border-top-left-radius",
        "border-top-left-radius",
        "-1px"
    ),
    vector!(
        "baseline.property.border-top-right-radius",
        "border-top-right-radius",
        "-1px"
    ),
    vector!(
        "baseline.property.border-bottom-right-radius",
        "border-bottom-right-radius",
        "-1px"
    ),
    vector!(
        "baseline.property.border-bottom-left-radius",
        "border-bottom-left-radius",
        "-1px"
    ),
    vector!("baseline.property.box-shadow", "box-shadow", "1px 2px -3px"),
    vector!(
        "official.property.image-orientation",
        "image-orientation",
        "flip 90deg"
    ),
    vector!(
        "official.property.image-rendering",
        "image-rendering",
        "smooth"
    ),
    vector!(
        "official.property.object-fit",
        "object-fit",
        "cover contain"
    ),
    vector!("baseline.property.opacity", "opacity", "solid"),
    vector!("baseline.property.flex-grow", "flex-grow", "solid"),
    vector!("baseline.property.flex-shrink", "flex-shrink", "solid"),
    vector!("baseline.property.order", "order", "1.5"),
    vector!("baseline.property.flex", "flex", "-1"),
    vector!("baseline.property.justify-tracks", "justify-tracks", "auto"),
    vector!("baseline.property.align-tracks", "align-tracks", "auto"),
    vector!("baseline.property.aspect-ratio", "aspect-ratio", "solid"),
    vector!(
        "baseline.property.scrollbar-width",
        "scrollbar-width",
        "solid"
    ),
    vector!("baseline.property.cursor", "cursor", "10px"),
    vector!("baseline.property.pointer-events", "pointer-events", "grab"),
    vector!("baseline.property.user-select", "user-select", "grab"),
    vector!("baseline.property.outline", "outline", "solid dotted"),
    vector!(
        "baseline.property.outline-color",
        "outline-color",
        "black white"
    ),
    vector!("baseline.property.outline-style", "outline-style", "10px"),
    vector!("baseline.property.outline-width", "outline-width", "10%"),
    vector!("baseline.property.transform", "transform", "translate(red)"),
    vector!(
        "baseline.property.transform-origin",
        "transform-origin",
        "left right"
    ),
    vector!("baseline.property.translate", "translate", "red"),
    vector!("baseline.property.rotate", "rotate", "45px"),
    vector!("baseline.property.scale", "scale", "solid"),
    vector!("baseline.property.filter", "filter", "opacity(red)"),
    vector!(
        "baseline.property.backdrop-filter",
        "backdrop-filter",
        "opacity(red)"
    ),
    vector!("baseline.property.clip-path", "clip-path", "circle(red)"),
    vector!("baseline.property.mask", "mask", "solid"),
    vector!("baseline.property.mask-image", "mask-image", "url(\"\")"),
    vector!("baseline.property.mask-size", "mask-size", "solid"),
    vector!(
        "baseline.property.mask-position",
        "mask-position",
        "left right"
    ),
    vector!("baseline.property.mask-repeat", "mask-repeat", "solid"),
    vector!(
        "baseline.property.transition-property",
        "transition-property",
        "auto"
    ),
    vector!(
        "baseline.property.transition-duration",
        "transition-duration",
        "10px"
    ),
    vector!(
        "baseline.property.transition-delay",
        "transition-delay",
        "10px"
    ),
    vector!(
        "baseline.property.transition-timing-function",
        "transition-timing-function",
        "bounce"
    ),
    vector!(
        "baseline.property.transition",
        "transition",
        "opacity 1s 2s 3s"
    ),
    vector!("baseline.property.animation-name", "animation-name", "auto"),
    vector!(
        "baseline.property.animation-duration",
        "animation-duration",
        "10px"
    ),
    vector!(
        "baseline.property.animation-delay",
        "animation-delay",
        "10px"
    ),
    vector!(
        "baseline.property.animation-timing-function",
        "animation-timing-function",
        "bounce"
    ),
    vector!(
        "baseline.property.animation-iteration-count",
        "animation-iteration-count",
        "-1"
    ),
    vector!(
        "baseline.property.animation-direction",
        "animation-direction",
        "running"
    ),
    vector!(
        "baseline.property.animation-fill-mode",
        "animation-fill-mode",
        "running"
    ),
    vector!(
        "baseline.property.animation-play-state",
        "animation-play-state",
        "alternate"
    ),
    vector!("baseline.property.animation", "animation", "fade 1s 2s 3s"),
];
