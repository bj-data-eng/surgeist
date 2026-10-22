# Pinned WebKit extracted-path synthetic-bold witness

This reference retains exact bounded excerpts from immutable WebKit source, retrieved on 2026-10-22. It captures implementation evidence, not normative CSS or native CoreText raster parity. Labels and interpretation are Surgeist-authored. No copied source is compiled or linked by this document. The complete original legal header is retained below and in the accompanying [license material](../licenses/webkit/font-coretext-bsd-2-clause.txt).

## Source identity

* Upstream: [WebKit](https://webkit.org/).
* Immutable revision: `73aa6c89e2cb77c46184a81aec944e4ab99d114d`.
* Exact source path: `Source/WebCore/platform/graphics/coretext/FontCoreText.cpp`.
* [Pinned source](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/platform/graphics/coretext/FontCoreText.cpp); [exact raw source](https://raw.githubusercontent.com/WebKit/WebKit/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/platform/graphics/coretext/FontCoreText.cpp).
* Complete original source SHA-256: `33f8d89a2582f3bc0cc484f2264700b6b401882623e67e024a25b11697544cba`.
* Acquisition: exact official raw source over TLS, bounded curl, with complete-source hash checked before extracting. Source text and line endings inside fenced sections are unchanged. Only the identified ranges are retained, not the rest of the file.

## Complete original copyright, conditions and disclaimer

Original physical lines 1–25. This is the complete original file header, including both source-declared copyright notices. Its two redistribution conditions and disclaimer apply to the quoted source; no generic WebKit license substitutes for them.

```cpp
/*
 * Copyright (C) 2005-2026 Apple Inc. All rights reserved.
 * Copyright (C) 2006 Alexey Proskuryakov
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY APPLE INC. AND ITS CONTRIBUTORS ``AS IS''
 * AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO,
 * THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
 * PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL APPLE INC. OR ITS CONTRIBUTORS
 * BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
 * CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
 * SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
 * INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
 * CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
 * ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF
 * THE POSSIBILITY OF SUCH DAMAGE.
 */
```

## Actual size and synthetic-bold displacement

Original physical lines 101–112. This is an exact excerpt ending inside the function, not its complete body.

```cpp
void FontBase::platformInit()
{
    auto constexpr syntheticBoldScaleFactor = 36.0f;
    m_syntheticBoldOffset = m_platformData.syntheticBold() ? (m_platformData.size() / syntheticBoldScaleFactor) : 0.f;

    RetainPtr ctFont = this->ctFont();
    unsigned unitsPerEm = CTFontGetUnitsPerEm(ctFont.get());
    float pointSize = m_platformData.size();
    CGFloat capHeight = pointSize ? CTFontGetCapHeight(ctFont.get()) : 0;
    CGFloat lineGap = pointSize ? CTFontGetLeading(ctFont.get()) : 0;
    CGFloat ascent = pointSize ? CTFontGetAscent(ctFont.get()) : 0;
    CGFloat descent = pointSize ? CTFontGetDescent(ctFont.get()) : 0;
```

## Actual advance without added bold displacement

Original physical lines 581–590. This is an exact excerpt ending before the function's closing brace.

```cpp
float Font::platformWidthForGlyph(Glyph glyph) const
{
    CGSize advance = CGSizeZero;

    if (platformData().size()) {
        bool horizontal = platformData().orientation() == FontOrientation::Horizontal;
        CTFontOrientation orientation = horizontal || m_isBrokenIdeographFallback ? kCTFontOrientationHorizontal : kCTFontOrientationVertical;
        CTFontGetAdvancesForGlyphs(protect(ctFont()).get(), orientation, &glyph, &advance, 1);
    }
    return advance.width;
```

## Original and translated contours in one returned path

Original physical lines 787–802. This is an exact excerpt ending before the function's closing brace.

```cpp
Path Font::platformPathForGlyph(Glyph glyph) const
{
    auto result = adoptCF(CTFontCreatePathForGlyph(protect(ctFont()).get(), glyph, nullptr));
    if (!result)
        return { };

    auto syntheticBoldOffset = this->syntheticBoldOffset();
    if (syntheticBoldOffset) {
        auto newPath = adoptCF(CGPathCreateMutable());
        CGPathAddPath(newPath.get(), nullptr, result.get());
        auto translation = CGAffineTransformMakeTranslation(syntheticBoldOffset, 0);
        CGPathAddPath(newPath.get(), &translation, result.get());
        return { PathCG::create(WTF::move(newPath)) };
    }

    return { PathCG::create(adoptCF(CGPathCreateMutableCopy(result.get()))) };
```

## Source interpretation and coordinate scope

`platformInit` stores a float offset selected by the actual synthetic-bold flag: platform font size divided by `36.0f`, otherwise zero. `platformWidthForGlyph` returns the actual CoreText advance without adding this offset. The glyph-path operation extracts the actual CTFont outline with no additional extraction transform, then appends the original path and an x-translated copy to one mutable path. It does not compute an expansion radius, perform a boolean union, inspect the drawing CTM, apply a synthetic-oblique transform, change a glyph advance or draw twice. Its null-path branch is not a Surgeist proof that an unsupported representation is empty.

[Apple's CTFontCreatePathForGlyph API documentation](https://developer.apple.com/documentation/coretext/ctfontcreatepathforglyph(_:_:_:)) describes the output as incorporating actual font point size, followed by the font matrix and finally the optional extraction transform. This paraphrases the current API documentation; no Apple documentation text is quoted. Consequently WebKit's additional translation acts on the already size/matrix-transformed extracted path. It is not a raw design-unit distance to scale again. This current API documentation is supplemental evidence, not a pinned historical specification.

The actual platform font can incorporate font-matrix and width-variant effects. [FontPlatformDataCoreText.cpp:72–98](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/platform/graphics/coretext/FontPlatformDataCoreText.cpp#L72) keeps actual CTFont and synthesis metadata distinct. [FontCacheCoreText.cpp:740–745](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/platform/graphics/cocoa/FontCacheCoreText.cpp#L740) applies font-size-adjust to platform data. The size basis is the actual effective run font size, including its selected fallback, not an unrelated authored size. These linked files are not copied here; their separate notices are not replaced by the header above.

## Selected product policy

The [Text/Render contract](../docs/text-layout-contract.md#concrete-outline-synthesis) owns the selected operation and numerical, identity and publication rules. This witness supplies its source evidence; it does not establish implemented capability.

## Native paint is a separate operation

No native paint source is copied here. At the same pin, [FontCascadeCoreText.cpp:332–340](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/platform/graphics/coretext/FontCascadeCoreText.cpp#L332) can reduce displacement according to the drawing CTM's transformed unit-x magnitude; [lines 380–389](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/platform/graphics/coretext/FontCascadeCoreText.cpp#L380) issue two foreground glyph draws. Text matrices and vertical-position handling are at lines 65–265. Those operations are not the selected extracted-path rule. A single filled path and two paints can differ under translucency, antialiasing, clipping, shadow and transforms. At full overlap, two source-over paints of alpha `a` give alpha `2a-a²`, while one filled path gives `a`. No native raster, alpha or CTM equivalence is asserted. The linked native paint file carries its own original notice; the FontCoreText header above does not cover it.

## Attribution

The exact header above and its [standalone copy](../licenses/webkit/font-coretext-bsd-2-clause.txt) retain both source-declared copyright notices, redistribution conditions and disclaimer. The [WebKit attribution inventory](../licenses/webkit/NOTICE.md#synthetic-bold-outline-witness) records this bounded capture. The header does not apply to separately linked files.
