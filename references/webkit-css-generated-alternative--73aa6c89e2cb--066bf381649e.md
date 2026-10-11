# Webkit Css Generated Alternative

This bounded source excerpt supports Surgeist implementation decisions. It is browser evidence, not a CSS standard or a claim of complete browser behavior. No retained source code has been modified; bounded excerpts do not imply complete surrounding functions. Original source: [Source/WebCore/accessibility/AccessibilityNodeObject.cpp](https://raw.githubusercontent.com/WebKit/WebKit/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/accessibility/AccessibilityNodeObject.cpp); immutable commit `73aa6c89e2cb77c46184a81aec944e4ab99d114d`. Retrieved 2026-10-11 using verified TLS. Full source SHA-256 `bd80013c47f474fff6eb4a0fa60119534ccd23f071aac975dfa9d98b0daa414b`; bounded source SHA-256 `066bf381649e3ef5d917c9237196d657dd1db163be6145ec698daf0b93a855a9`. Retained exact original line ranges: 3425–3539. Added headings and provenance notes are Surgeist material.

Upstream homepage: [upstream project](https://webkit.org/). The exact file-specific redistribution notice follows; it governs the excerpts below.

## Original file notice

```text
/*
* Copyright (C) 2012 Google Inc. All rights reserved.
*
* Redistribution and use in source and binary forms, with or without
* modification, are permitted provided that the following conditions
* are met:
*
* 1.  Redistributions of source code must retain the above copyright
*     notice, this list of conditions and the following disclaimer.
* 2.  Redistributions in binary form must reproduce the above copyright
*     notice, this list of conditions and the following disclaimer in the
*     documentation and/or other materials provided with the distribution.
* 3.  Neither the name of Apple Inc. ("Apple") nor the names of
*     its contributors may be used to endorse or promote products derived
*     from this software without specific prior written permission.
*
* THIS SOFTWARE IS PROVIDED BY APPLE AND ITS CONTRIBUTORS "AS IS" AND ANY
* EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
* WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
* DISCLAIMED. IN NO EVENT SHALL APPLE OR ITS CONTRIBUTORS BE LIABLE FOR ANY
* DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES
* (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
* LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND
* ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
* (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF
* THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
*/
```

## Original lines 3425–3539

```cpp
void AccessibilityNodeObject::alternativeText(Vector<AccessibilityText>& textOrder) const
{
    if (isWebArea()) {
        String webAreaText = alternativeTextForWebArea();
        if (!webAreaText.isEmpty())
            textOrder.append(AccessibilityText(WTF::move(webAreaText), AccessibilityTextSource::Alternative));
        return;
    }

    ariaLabeledByText(textOrder);

    bool hasValidAriaLabel = false;
    {
        // Scoped since we potentially move |ariaLabel| here. The scope prevents accidental use-after-move later.
        auto ariaLabel = getAttributeTrimmed(aria_labelAttr);
        if (!ariaLabel.isEmpty()) {
            hasValidAriaLabel = true;
            textOrder.append(AccessibilityText(WTF::move(ariaLabel), AccessibilityTextSource::Alternative));
        }
    }

    if (usesAltForTextComputation()) {
        if (auto* renderImage = dynamicDowncast<RenderImage>(renderer())) {
            String renderAltText = renderImage->altText();

            // RenderImage will return title as a fallback from altText, but we don't want title here because we consider that in helpText.
            if (!renderAltText.isEmpty() && renderAltText != getAttribute(titleAttr)) {
                textOrder.append(AccessibilityText(WTF::move(renderAltText), AccessibilityTextSource::Alternative));
                return;
            }
        }
        // Images should use alt as long as the attribute is present, even if empty.
        // Otherwise, it should fallback to other methods, like the title attribute.
        if (String alt = altTextFromAttributeOrStyle(); !alt.isNull())
            textOrder.append(AccessibilityText(WTF::move(alt), AccessibilityTextSource::Alternative));
    }

    RefPtr node = this->node();
    if (!node)
        return;

    CheckedPtr objectCache = axObjectCache();
    // The fieldset element derives its alternative text from the first associated legend element if one is available.
    if (RefPtr fieldset = dynamicDowncast<HTMLFieldSetElement>(*node); fieldset && objectCache) {
        RefPtr object = objectCache->getOrCreate(protect(fieldset->legend()));
        if (object && !object->isHidden())
            textOrder.append(AccessibilityText(accessibleNameForNode(*object->node()), AccessibilityTextSource::Alternative));
    }

    if (RefPtr image = dynamicDowncast<HTMLImageElement>(*node)) {
        // https://github.com/w3c/aria/pull/2224
        // Per html-aam, <img> elements that are unlabeled (e.g., alt attribute, ARIA, title) derive accname
        // from an ancestor figure's <figcaption> if and only if the <figure> does not contain other flow content (besides the <figcaption>).
        const AtomString& alt = image->attributeWithoutSynchronization(altAttr);

        if (alt.isEmpty() && image->attributeWithoutSynchronization(titleAttr).isEmpty()) {
            for (RefPtr ancestor = node->parentNode(); ancestor; ancestor = ancestor->parentNode()) {
                if (auto* figure = dynamicDowncast<HTMLElement>(ancestor.get()); figure && figure->hasTagName(figureTag)) {
                    bool figureHasFlowContent = false;
                    // Iterate over the direct children of the <img>'s ancestor <figure> for any common
                    // flow content, including non-whitespace text nodes.
                    for (auto* figureNodeChild = figure->firstChild(); figureNodeChild; figureNodeChild = figureNodeChild->nextSibling()) {
                        if (isFlowContent(*figureNodeChild)) {
                            figureHasFlowContent = true;
                            break;
                        }
                    }
                    // If no flow content is present in the <figure>, the <img> derives accname from its <figcaption>.
                    if (!figureHasFlowContent) {
                        RefPtr figureObject = objectCache ? objectCache->getOrCreate(*figure) : nullptr;
                        RefPtr caption = figureObject && figureObject->isFigureElement() ? downcast<AccessibilityNodeObject>(figureObject)->captionForFigure() : nullptr;
                        if (caption && !caption->isHidden()) {
                            RefPtr captionNode = caption->node();
                            if (String captionAccname = captionNode ? accessibleNameForNode(*captionNode) : emptyString(); !captionAccname.isEmpty())
                                textOrder.append(AccessibilityText(WTF::move(captionAccname), AccessibilityTextSource::Alternative));
                        }
                    }
                    break;
                }
            }
        }
    }

    // Tree items missing a label are labeled by all child elements.
    if (isTreeItem() && !hasValidAriaLabel && ariaLabeledByAttribute().isEmpty())
        textOrder.append(AccessibilityText(accessibleNameForNode(*node), AccessibilityTextSource::Alternative));

    if (accessibleNameDerivesFromHeading()) {
        CheckedPtr cache = axObjectCache();
        // Where an element supports nameFrom: heading and no nameFrom: content/author is supplied, its accname may be
        // derived from the first descendant node that is a heading (depth-first search, preorder traversal).
        if (RefPtr containerNode = dynamicDowncast<ContainerNode>(node); containerNode && cache) {
            for (Ref element : descendantsOfType<Element>(*containerNode)) {
                if (RefPtr descendantObject = cache->getOrCreate(element.get()); descendantObject && descendantObject->isHeading()) {
                    TextUnderElementMode mode;
                    mode.includeFocusableContent = true;
                    String nameFromHeading = descendantObject->textUnderElement(mode);
                    if (!nameFromHeading.isEmpty())
                        textOrder.append(AccessibilityText(nameFromHeading, AccessibilityTextSource::Heading));
                }
            }
        }
    }

#if ENABLE(MATHML)
    if (node->isMathMLElement())
        textOrder.append(AccessibilityText(getAttribute(MathMLNames::alttextAttr), AccessibilityTextSource::Alternative));
#endif

    if (CheckedPtr style = this->style()) {
        String altText = style->content().altText();
        if (!altText.isEmpty())
            textOrder.append(AccessibilityText(WTF::move(altText), AccessibilityTextSource::Alternative));
    }
}
```
