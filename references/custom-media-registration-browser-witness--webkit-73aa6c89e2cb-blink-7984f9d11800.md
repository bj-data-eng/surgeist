# Pinned browser evidence for custom-media registration

Captured 2026-10-12. Bounded source evidence for Surgeist #456; neither a whole-engine absence proof nor adoption of all behavior in these files. Original headers and source bytes within each block are retained. Added prose is Surgeist analysis. Source code is documentation only, not compiled or linked.

WebKit was inspected first. A case-insensitive `git grep custommedia` at the pinned revision under Source/WebCore/css and Source/WebCore/dom returned no matches (exit 1). Its CSS namespace below has no registry. Blink supplies authored CSSCustomMediaRule with boolean or MediaList query values and a rule-backed evaluator, not the draft's number/string registration map. The inspected CSS namespace likewise has no customMedia member. This bounded fallback does not establish missing registry lifetime, precedence, update/observer rules or range-value semantics.

The chosen native API uses ordinary stylesheet rules and their CSSOM operations. Do not copy the Blink cssText implementation as a serialization determination; that is outside this witness's scope.

Attribution: [WebKit](https://webkit.org/) and [Chromium/Blink](https://www.chromium.org/blink/). The two namespace files retain complete Motorola BSD notices. Chromium-authored rule files retain their original notices and use the retained [Chromium BSD license](../licenses/chromium/LICENSE.txt). The evaluator excerpt retains its original full copyright, BSD redistribution conditions and disclaimer in its header.

## WebKit: Source/WebCore/css/DOMCSSNamespace.idl

[73aa6c89e2cb77c46184a81aec944e4ab99d114d](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/DOMCSSNamespace.idl); full-source SHA-256 `b94e02062debbaffb9c5e3f109a627e4189eeac6852d9d742adb1fe76858a9d6`. Complete file.

```cpp
/*
 * Copyright (C) 2012 Motorola Mobility Inc. All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are
 * met:
 *
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above
 *    copyright notice, this list of conditions and the following disclaimer in
 *    the documentation and/or other materials provided with the distribution.
 * 3. Neither the name of Motorola Mobility Inc. nor the names of its
 *    contributors may be used to endorse or promote products derived from this
 *    software without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
 * "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
 * LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
 * A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
 * OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
 * SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
 * LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
 * DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
 * THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */

[
    InterfaceName=CSS,
    Exposed=Window
] namespace DOMCSSNamespace {
    [CallWith=CurrentDocument] boolean supports(DOMString property, DOMString value);
    [CallWith=CurrentDocument] boolean supports(DOMString conditionText);
    DOMString escape(DOMString ident);
    [CallWith=CurrentDocument] readonly attribute HighlightRegistry highlights;
};
```

## Blink: third_party/blink/renderer/core/css/css.idl

[7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2](https://chromium.googlesource.com/chromium/src/+/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/css/css.idl); full-source SHA-256 `e7477b31e8badc313d923fb42cd214928454369869dc88cee15f01e72a0bbb1b`. Complete file.

```cpp
/*
 * Copyright (C) 2012 Motorola Mobility Inc. All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are
 * met:
 *
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above
 *    copyright notice, this list of conditions and the following disclaimer in
 *    the documentation and/or other materials provided with the distribution.
 * 3. Neither the name of Motorola Mobility Inc. nor the names of its
 *    contributors may be used to endorse or promote products derived from this
 *    software without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
 * "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
 * LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
 * A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
 * OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
 * SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
 * LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
 * DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
 * THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */

// https://drafts.csswg.org/css-conditional/#the-css-interface

[
    Exposed=Window,
    ImplementedAs=DOMWindowCSS
] namespace CSS {
    [CallWith=ExecutionContext] boolean supports(DOMString property, DOMString value);
    [CallWith=ExecutionContext] boolean supports(DOMString conditionText);
    DOMString escape(DOMString ident);
};
```

## Blink: third_party/blink/renderer/core/css/css_custom_media_rule.idl

[7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2](https://chromium.googlesource.com/chromium/src/+/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/css/css_custom_media_rule.idl); full-source SHA-256 `f59934e65ae23986bfac07593a537fa08f068f94e41093e8a9bb99b5bd78077a`. Complete file.

```cpp
// Copyright 2025 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
typedef (MediaList or boolean) CustomMediaQuery;

[Exposed=Window, RuntimeEnabled=CSSCustomMedia]
interface CSSCustomMediaRule : CSSRule {
    readonly attribute CSSOMString name;
    readonly attribute CustomMediaQuery query;
};```

## Blink: third_party/blink/renderer/core/css/css_custom_media_rule.cc

[7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2](https://chromium.googlesource.com/chromium/src/+/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/css/css_custom_media_rule.cc); full-source SHA-256 `b28abd1bd36ce7b3ffed80c6de4aeb6c35c915ce1161362c90f54b3232960c07`. Complete file.

```cpp
// Copyright 2025 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#include "third_party/blink/renderer/core/css/css_custom_media_rule.h"

#include "third_party/blink/renderer/core/css/css_markup.h"
#include "third_party/blink/renderer/core/css/css_style_sheet.h"
#include "third_party/blink/renderer/core/css/style_rule.h"

namespace blink {

class MediaQuerySetOwner;

CSSCustomMediaRule::CSSCustomMediaRule(StyleRuleCustomMedia* custom_media_rule,
                                       CSSStyleSheet* parent)
    : CSSRule(parent), custom_media_rule_(custom_media_rule) {}

CSSCustomMediaRule::~CSSCustomMediaRule() = default;

String CSSCustomMediaRule::name() const {
  StringBuilder result;
  String name = custom_media_rule_->GetName();
  if (!name.empty()) {
    SerializeIdentifier(name, result);
  }
  return result.ReleaseString();
}

V8CustomMediaQuery* CSSCustomMediaRule::query() {
  if (custom_media_rule_->IsBooleanValue()) {
    return MakeGarbageCollected<V8CustomMediaQuery>(
        custom_media_rule_->GetBooleanValue());
  }
  if (custom_media_rule_->IsMediaQueryValue()) {
    return MakeGarbageCollected<V8CustomMediaQuery>(
        MakeGarbageCollected<MediaList>(this));
  }
  return nullptr;
}

void CSSCustomMediaRule::Reattach(StyleRuleBase* rule) {
  DCHECK(rule);
  custom_media_rule_ = To<StyleRuleCustomMedia>(rule);
}

String CSSCustomMediaRule::cssText() const {
  StringBuilder result;
  result.Append("@custom-media");
  if (custom_media_rule_->IsBooleanValue()) {
    result.Append(' ');
    StringView str = custom_media_rule_->GetBooleanValue() ? "true" : "false";
    result.Append(str);
  }
  if (custom_media_rule_->IsMediaQueryValue()) {
    result.Append(' ');
    result.Append(custom_media_rule_->GetMediaQueryValue()->MediaText());
  }
  return result.ReleaseString();
}

const MediaQuerySet* CSSCustomMediaRule::MediaQueries() const {
  if (custom_media_rule_->IsMediaQueryValue()) {
    return custom_media_rule_->GetMediaQueryValue();
  }
  return nullptr;
}

void CSSCustomMediaRule::SetMediaQueries(const MediaQuerySet* media_queries) {
  custom_media_rule_->SetMediaQueries(media_queries);
}

void CSSCustomMediaRule::Trace(Visitor* visitor) const {
  visitor->Trace(custom_media_rule_);
  CSSRule::Trace(visitor);
}

}  // namespace blink
```

## Blink: third_party/blink/renderer/core/css/media_query_evaluator.cc

[7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2](https://chromium.googlesource.com/chromium/src/+/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/css/media_query_evaluator.cc); full-source SHA-256 `dd70f362f9735aec6e7fcbbae8e6a78420a91149fdb5f8d0af97368fa06a1aaa`. Original license header and lines 1719–1743.

```cpp
/*
 * CSS Media Query Evaluator
 *
 * Copyright (C) 2006 Kimmo Kinnunen <kimmo.t.kinnunen@nokia.com>.
 * Copyright (C) 2013 Apple Inc. All rights reserved.
 * Copyright (C) 2013 Intel Corporation. All rights reserved.
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
 * THIS SOFTWARE IS PROVIDED BY THE AUTHOR ``AS IS'' AND ANY
 * EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
 * PURPOSE ARE DISCLAIMED.  IN NO EVENT SHALL APPLE COMPUTER, INC. OR
 * CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL,
 * EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
 * PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
 * PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY
 * OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */

KleeneValue MediaQueryEvaluator::EvalCustomMedia(
    const StyleRuleCustomMedia* custom_media_rule,
    MediaQueryResultFlags* result_flags,
    const CustomMediaRulesMap* custom_medias) const {
  if (custom_media_rule->IsBooleanValue()) {
    return custom_media_rule->GetBooleanValue() ? KleeneValue::kTrue
                                                : KleeneValue::kFalse;
  }
  CHECK(custom_media_rule->IsMediaQueryValue());
  const MediaQuerySet* query_set = custom_media_rule->GetMediaQueryValue();

  const HeapVector<Member<const MediaQuery>>& queries =
      query_set->QueryVector();
  if (queries.empty()) {
    return KleeneValue::kTrue;  // An empty query list evaluates to true.
  }

  KleeneValue result = KleeneValue::kFalse;
  for (wtf_size_t i = 0; i < queries.size() && result != KleeneValue::kTrue;
       ++i) {
    result = KleeneOr(result, Eval(*queries[i], result_flags, custom_medias));
  }

  return result;
}
```
