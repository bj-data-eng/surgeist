# SVG and MathML inclusion of HTMLOrSVGOrMathMLElement

The following excerpts retain the Web IDL inclusion declarations selected
alongside the [HTML mixin rename](https://github.com/whatwg/html/commit/0965f590737610ebb0273d9bfb77aa7fd88cb195).
Each excerpt contains one exact statement from the named upstream source.
They record which interfaces include the HTML mixin.

## SVGElement

[SVG Working Group, revision fd140018a189e6bf2fbaa56f35655635824bfb3f, master/types.html lines 696–706](https://github.com/w3c/svgwg/blob/fd140018a189e6bf2fbaa56f35655635824bfb3f/master/types.html#L696-L706).

<a id="svg-element-includes"></a>

``` idl
SVGElement includes HTMLOrSVGOrMathMLElement;
```

## MathMLElement

[Math Working Group, revision 1681f190a88d2ddd109c23f356c7bc24085c705f, webidl/MathMLElement.idl lines 1–4](https://github.com/w3c/mathml-core/blob/1681f190a88d2ddd109c23f356c7bc24085c705f/webidl/MathMLElement.idl#L1-L4).

<a id="mathml-element-includes"></a>

``` idl
MathMLElement includes HTMLOrSVGOrMathMLElement;
```
