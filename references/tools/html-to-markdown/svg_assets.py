"""Extract inline SVG diagrams from a captured body as passive local assets.

Each inline `<svg>` is serialized as a standalone SVG document with its source
geometry, IDs, classes and text, with SVG attribute and element names restored
to their exact case (the HTML parser lowercases them). Scripts, event-handler
attributes and `javascript:` references are dropped, so assets stay passive.
Railroad diagrams receive the capture's own railroad stylesheet with its
custom properties resolved to their light-theme values, so the asset renders
without the page. The body keeps an image reference and the diagram's visible
text in source order.
"""
import re
from lxml import etree

SVG_NS = 'http://www.w3.org/2000/svg'
XLINK_NS = 'http://www.w3.org/1999/xlink'
ASSET_SCHEME = 'surgeist-asset:'

# The HTML "adjust SVG attributes" and "adjust SVG tag names" tables.
_ATTRIBUTES = '''attributeName attributeType baseFrequency baseProfile calcMode clipPathUnits
diffuseConstant edgeMode filterUnits glyphRef gradientTransform gradientUnits kernelMatrix
kernelUnitLength keyPoints keySplines keyTimes lengthAdjust limitingConeAngle markerHeight
markerUnits markerWidth maskContentUnits maskUnits numOctaves pathLength patternContentUnits
patternTransform patternUnits pointsAtX pointsAtY pointsAtZ preserveAlpha preserveAspectRatio
primitiveUnits refX refY repeatCount repeatDur requiredExtensions requiredFeatures
specularConstant specularExponent spreadMethod startOffset stdDeviation stitchTiles
surfaceScale systemLanguage tableValues targetX targetY textLength viewBox viewTarget
xChannelSelector yChannelSelector zoomAndPan'''.split()
_TAGS = '''altGlyph altGlyphDef altGlyphItem animateColor animateMotion animateTransform
clipPath feBlend feColorMatrix feComponentTransfer feComposite feConvolveMatrix
feDiffuseLighting feDisplacementMap feDistantLight feDropShadow feFlood feFuncA feFuncB
feFuncG feFuncR feGaussianBlur feImage feMerge feMergeNode feMorphology feOffset
fePointLight feSpecularLighting feSpotLight feTile feTurbulence foreignObject glyphRef
linearGradient radialGradient textPath'''.split()
ATTRIBUTE_CASE = {name.lower(): name for name in _ATTRIBUTES}
TAG_CASE = {name.lower(): name for name in _TAGS}


def railroad_stylesheet(page):
    """The capture's railroad CSS with light-theme custom properties resolved."""
    for style in page.iter('style'):
        text = style.text or ''
        if 'svg.railroad-diagram' not in text:
            continue
        start = text.find('svg.railroad-diagram')
        end = text.find('@media', start)
        rules = text[start:end if end != -1 else len(text)]
        root = re.search(r':root\s*\{([^}]*)\}', text)
        values = dict(re.findall(r'(--[\w-]+)\s*:\s*([^;]+);', root.group(1))) if root else {}

        def resolve(match):
            name, fallback = match.group(1), match.group(2)
            if name in values:
                return values[name].strip()
            return fallback.strip() if fallback else match.group(0)

        rules = re.sub(r'var\((--[\w-]+)(?:\s*,\s*([^)]+))?\)', resolve, rules)
        return '\n'.join(line.strip() for line in rules.strip().splitlines() if line.strip())
    return None


def _copy(node):
    tag = TAG_CASE.get(node.tag, node.tag)
    element = etree.Element('{%s}%s' % (SVG_NS, tag), nsmap={None: SVG_NS, 'xlink': XLINK_NS})
    for name, value in node.attrib.items():
        if name.startswith('on') or value.strip().lower().startswith('javascript:'):
            continue
        if name == 'xlink:href':
            element.set('{%s}href' % XLINK_NS, value)
        else:
            element.set(ATTRIBUTE_CASE.get(name, name), value)
    element.text = node.text
    for child in node:
        if not isinstance(child.tag, str) or child.tag == 'script':
            continue
        copy = _copy(child)
        copy.tail = child.tail
        element.append(copy)
    return element


def shape(node):
    """Element names, attributes and text in document order, in exact case.

    Works for both an HTML-parsed inline SVG and a namespaced asset, so the
    two compare equal when the asset reproduces its source.
    """
    out = []
    for element in node.iter():
        if not isinstance(element.tag, str):
            continue
        tag = element.tag.split('}', 1)[1] if '}' in element.tag else TAG_CASE.get(element.tag, element.tag)
        if tag == 'script':
            continue
        attributes = {}
        for name, value in element.attrib.items():
            name = 'xlink:' + name.split('}', 1)[1] if '}' in name else ATTRIBUTE_CASE.get(name, name)
            if name.startswith('on') or value.strip().lower().startswith('javascript:'):
                continue
            attributes[name] = value
        out.append((tag, attributes, (element.text or '').strip()))
    return out


def asset_shape(asset, source):
    """The asset's shape without the stylesheet and color this module adds."""
    out = shape(asset)
    if 'color' not in source.attrib:
        out[0][1].pop('color', None)
    source_has_style = len(source) and source[0].tag == 'style'
    if len(out) > 1 and out[1][0] == 'style' and not source_has_style:
        del out[1]
    return out


def diagram_text(svg):
    return re.sub(r'\s+', ' ', ' '.join(t.text_content() for t in svg.iter('text'))).strip()


def extract(body, page, stem):
    """Replace each inline SVG in `body`; return (asset name, SVG bytes) pairs."""
    assets = []
    stylesheet = railroad_stylesheet(page)
    for index, svg in enumerate(body.xpath('.//svg'), 1):
        name = f'{stem}--diagram-{index:02}.svg'
        asset = _copy(svg)
        if stylesheet and 'railroad-diagram' in svg.get('class', '').split():
            asset.set('color', 'black')
            style = etree.Element('{%s}style' % SVG_NS)
            style.set('type', 'text/css')
            style.text = stylesheet
            asset.insert(0, style)
        assets.append((name, etree.tostring(asset, encoding='utf-8') + b'\n'))
        image = etree.Element('img', src=ASSET_SCHEME + name, alt=f'Source diagram {index}')
        holder = etree.Element('div')
        holder.append(image)
        text = diagram_text(svg)
        if text:
            etree.SubElement(holder, 'p').text = 'Diagram text: ' + text
        holder.tail = svg.tail
        svg.getparent().replace(svg, holder)
    return assets
