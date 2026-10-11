# Blink Css Generated Alternative

This bounded source excerpt supports Surgeist implementation decisions. It is browser evidence, not a CSS standard or a claim of complete browser behavior. No retained source code has been modified; bounded excerpts do not imply complete surrounding functions. Original source: [third_party/blink/renderer/modules/accessibility/ax_node_object.cc](https://chromium.googlesource.com/chromium/src/+/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/modules/accessibility/ax_node_object.cc?format=TEXT); immutable commit `7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2`. Retrieved 2026-10-11 using verified TLS. Full source SHA-256 `7c47126cefe3238acbe14c10db1e50fd307927f1b2af60de993e82461b057465`; bounded source SHA-256 `1fb5fe3f70d1bdf5557c8d09fba47664d2b9eee188b26d8008a3568b571edcd4`. Retained exact original line ranges: 1583–1619, 5373–5409. Added headings and provenance notes are Surgeist material.

Upstream homepage: [upstream project](https://source.chromium.org/chromium/chromium/src). The exact file-specific redistribution notice follows; it governs the excerpts below.

## Original file notice

```text
/*
 * Copyright (C) 2012, Google Inc. All rights reserved.
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
 * 3.  Neither the name of Apple Computer, Inc. ("Apple") nor the names of
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

## Original lines 1583–1619

```cpp
// static
std::optional<String> AXNodeObject::GetCSSAltText(const Element* element) {
  // CSS alt text rules allow text to be assigned to ::before/::after content.
  // For example, the following CSS assigns "bullet" text to bullet.png:
  // .something::before {
  //   content: url(bullet.png) / "bullet";
  // }

  if (!element) {
    return std::nullopt;
  }
  const ComputedStyle* style = element->GetComputedStyle();
  if (!style || style->ContentBehavesAsNormal()) {
    return std::nullopt;
  }

  if (auto* pseudo_element = DynamicTo<PseudoElement>(element)) {
    for (const ContentData* content_data = pseudo_element->GetContentData();
         content_data; content_data = content_data->Next()) {
      if (content_data->IsAlt()) {
        return ContentData::ConcatenateAltText(*content_data);
      }
    }
    return std::nullopt;
  }

  // If the content property is used on a non-pseudo-element, match the
  // behaviour of LayoutObject::CreateObject and only honour the style if
  // there is exactly one piece of content, which is an image.
  const ContentData* content_data = style->GetContentData();
  if (content_data && content_data->IsImage() && content_data->Next() &&
      content_data->Next()->IsAlt()) {
    return ContentData::ConcatenateAltText(*content_data->Next());
  }

  return std::nullopt;
}
```

## Original lines 5373–5409

```cpp
String AXNodeObject::TextAlternative(
    bool recursive,
    const AXObject* aria_label_or_description_root,
    AXObjectSet& visited,
    ax::mojom::blink::NameFrom& name_from,
    AXRelatedObjectVector* related_objects,
    NameSources* name_sources) const {
  // If nameSources is non-null, relatedObjects is used in filling it in, so it
  // must be non-null as well.
  DCHECK(!name_sources || related_objects);

  bool found_text_alternative = false;
  Node* node = GetNode();

  name_from = ax::mojom::blink::NameFrom::kNone;
  if (!node && !GetLayoutObject()) {
    return String();
  }

  if (IsA<HTMLSlotElement>(node) && node->IsInUserAgentShadowRoot() &&
      !recursive) {
    // User agent slots do not have their own name, but their subtrees can
    // contribute to ancestor names (where recursive == true).
    return String();
  }

  if (GetLayoutObject()) {
    std::optional<String> text_alternative = GetCSSAltText(GetElement());
    if (text_alternative) {
      name_from = ax::mojom::blink::NameFrom::kCssAltText;
      if (name_sources) {
        name_sources->push_back(NameSource(false));
        name_sources->back().type = name_from;
        name_sources->back().text = text_alternative.value();
      }
      return text_alternative.value();
    }
```
