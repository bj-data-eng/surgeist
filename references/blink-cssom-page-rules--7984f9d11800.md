# Blink Page and margin CSSOM source witness

This reference retains 6 complete source files from [Blink](https://www.chromium.org/blink/) at immutable revision `7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2`. It supplies bounded implementation evidence for the stated CSSOM planning contribution. It is not a CSS specification, a whole-engine compatibility promise, or evidence about other revisions.

Retrieved on 2026-10-09 from the official Chromium Gitiles immutable TEXT responses, decoded from base64. Only the exact revision’s file bytes are included; working-tree changes are excluded. Each full file has its original path, byte hash and license header below. Added headings, provenance and the evidence map are documentation formatting; the source bytes are unchanged. The captured code is reference material and was not compiled or executed.

Original file-specific copyright notices, license conditions and disclaimers remain in the source blocks. The [local legal entry](../licenses/chromium/NOTICE.md) identifies applicable full license texts and their provenance. File-specific LGPL notices govern the named LGPL files; a project’s BSD license is not applied to them by inference.

The source boundaries below identify the relied-on behavior. Retaining complete files supplies surrounding context without selecting unrelated engine behavior.

| Source | Evidence boundaries | Original file SHA-256 |
| --- | --- | --- |
| [css_page_rule.cc](#css-page-rule-cc) | [lines 36–37](https://chromium.googlesource.com/chromium/src/+/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/css/css_page_rule.cc#L36), [lines 50–103](https://chromium.googlesource.com/chromium/src/+/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/css/css_page_rule.cc#L50) | `3eb0b099f52529d16a8fd0972b641cf93bb8a1b1d28bac2d51889e2ccfe6aeda` |
| [css_margin_rule.cc](#css-margin-rule-cc) | [lines 17–44](https://chromium.googlesource.com/chromium/src/+/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/css/css_margin_rule.cc#L17) | `c6e0f32d36201ba01569cacd29feeb65e00fbe2343696801398217d16f007ba2` |
| [css_margin_rule.idl](#css-margin-rule-idl) | [lines 9–12](https://chromium.googlesource.com/chromium/src/+/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/css/css_margin_rule.idl#L9) | `b88df00da153c30bff57628e349943fd8fed035c274292bdbcb03bc707975380` |
| [css_parser.cc](#css-parser-cc) | [lines 72–92](https://chromium.googlesource.com/chromium/src/+/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/css/parser/css_parser.cc#L72) | `d85601be5dd8c1f8cce59e3886126823cbe31d0fe0a673a0b02965972854a556` |
| [css_parser_impl.cc](#css-parser-impl-cc) | [lines 547–592](https://chromium.googlesource.com/chromium/src/+/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/css/parser/css_parser_impl.cc#L547), [lines 1845–1883](https://chromium.googlesource.com/chromium/src/+/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/css/parser/css_parser_impl.cc#L1845) | `3f9ca3b42eb1bada314a3d832bc6ea91a285aaeee039ced016e2b0d665fda674` |
| [css_selector.cc](#css-selector-cc) | [lines 1026–1035](https://chromium.googlesource.com/chromium/src/+/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/css/css_selector.cc#L1026), [lines 1373–1380](https://chromium.googlesource.com/chromium/src/+/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/css/css_selector.cc#L1373) | `0836181aebb7b9138734aab0ea9990197acbd3cc8ecd0871c739ebc3c0842f56` |

## <a id="css-page-rule-cc"></a>css_page_rule.cc

Original: [complete immutable source](https://chromium.googlesource.com/chromium/src/+/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/css/css_page_rule.cc). Path: `third_party/blink/renderer/core/css/css_page_rule.cc`. Source bytes: 4243; source lines: 121; SHA-256: `3eb0b099f52529d16a8fd0972b641cf93bb8a1b1d28bac2d51889e2ccfe6aeda`.

```cpp
/*
 * (C) 1999-2003 Lars Knoll (knoll@kde.org)
 * (C) 2002-2003 Dirk Mueller (mueller@kde.org)
 * Copyright (C) 2002, 2005, 2006, 2008, 2012 Apple Inc. All rights reserved.
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Library General Public
 * License as published by the Free Software Foundation; either
 * version 2 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Library General Public License for more details.
 *
 * You should have received a copy of the GNU Library General Public License
 * along with this library; see the file COPYING.LIB.  If not, write to
 * the Free Software Foundation, Inc., 51 Franklin Street, Fifth Floor,
 * Boston, MA 02110-1301, USA.
 */

#include "third_party/blink/renderer/core/css/css_page_rule.h"

#include "third_party/blink/renderer/core/css/css_property_value_set.h"
#include "third_party/blink/renderer/core/css/css_selector.h"
#include "third_party/blink/renderer/core/css/css_style_sheet.h"
#include "third_party/blink/renderer/core/css/parser/css_parser.h"
#include "third_party/blink/renderer/core/css/style_rule.h"
#include "third_party/blink/renderer/core/css/style_rule_css_style_declaration.h"
#include "third_party/blink/renderer/core/execution_context/execution_context.h"
#include "third_party/blink/renderer/platform/heap/garbage_collected.h"
#include "third_party/blink/renderer/platform/wtf/text/string_builder.h"

namespace blink {

CSSPageRule::CSSPageRule(StyleRulePage* page_rule, CSSStyleSheet* parent)
    : CSSGroupingRule(page_rule, parent), page_rule_(page_rule) {}

CSSPageRule::~CSSPageRule() = default;

CSSStyleDeclaration* CSSPageRule::style() const {
  if (!properties_cssom_wrapper_) {
    properties_cssom_wrapper_ =
        MakeGarbageCollected<StyleRuleCSSStyleDeclaration>(
            page_rule_->MutableProperties(), const_cast<CSSPageRule*>(this));
  }
  return properties_cssom_wrapper_.Get();
}

String CSSPageRule::selectorText() const {
  StringBuilder text;
  const CSSSelector* selector = page_rule_->Selector();
  if (selector) {
    String page_specification = selector->SelectorText();
    if (!page_specification.empty()) {
      text.Append(page_specification);
    }
  }
  return text.ReleaseString();
}

void CSSPageRule::setSelectorText(const ExecutionContext* execution_context,
                                  const String& selector_text) {
  auto* context = MakeGarbageCollected<CSSParserContext>(
      ParserContext(execution_context->GetSecureContextMode()));
  DCHECK(context);
  CSSSelectorList* selector_list = CSSParser::ParsePageSelector(
      *context, parentStyleSheet() ? parentStyleSheet()->Contents() : nullptr,
      selector_text);
  if (!selector_list || !selector_list->IsValid()) {
    return;
  }

  CSSStyleSheet::RuleMutationScope mutation_scope(this);

  page_rule_->WrapperAdoptSelectorList(selector_list);
}

String CSSPageRule::cssText() const {
  // TODO(mstensho): Serialization needs to be specced:
  // https://github.com/w3c/csswg-drafts/issues/9953
  StringBuilder result;
  result.Append("@page ");
  String page_selectors = selectorText();
  result.Append(page_selectors);
  if (!page_selectors.empty()) {
    result.Append(' ');
  }
  result.Append("{ ");
  String decls = page_rule_->Properties().AsText();
  result.Append(decls);
  if (!decls.empty()) {
    result.Append(' ');
  }

  unsigned size = length();
  for (unsigned i = 0; i < size; i++) {
    result.Append(ItemInternal(i)->cssText());
    result.Append(" ");
  }

  result.Append('}');
  return result.ReleaseString();
}

void CSSPageRule::Reattach(StyleRuleBase* rule) {
  DCHECK(rule);
  page_rule_ = To<StyleRulePage>(rule);
  if (properties_cssom_wrapper_) {
    properties_cssom_wrapper_->Reattach(page_rule_->MutableProperties());
  }
  CSSGroupingRule::Reattach(rule);
}

void CSSPageRule::Trace(Visitor* visitor) const {
  visitor->Trace(page_rule_);
  visitor->Trace(properties_cssom_wrapper_);
  CSSGroupingRule::Trace(visitor);
}

}  // namespace blink

```

## <a id="css-margin-rule-cc"></a>css_margin_rule.cc

Original: [complete immutable source](https://chromium.googlesource.com/chromium/src/+/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/css/css_margin_rule.cc). Path: `third_party/blink/renderer/core/css/css_margin_rule.cc`. Source bytes: 1917; source lines: 61; SHA-256: `c6e0f32d36201ba01569cacd29feeb65e00fbe2343696801398217d16f007ba2`.

```cpp
// Copyright 2024 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#include "third_party/blink/renderer/core/css/css_margin_rule.h"

#include "third_party/blink/renderer/core/css/style_rule.h"
#include "third_party/blink/renderer/core/css/style_rule_css_style_declaration.h"
#include "third_party/blink/renderer/platform/wtf/text/string_builder.h"

namespace blink {

CSSMarginRule::CSSMarginRule(StyleRulePageMargin* margin_rule,
                             CSSStyleSheet* parent)
    : CSSRule(parent), margin_rule_(margin_rule) {}

CSSStyleDeclaration* CSSMarginRule::style() const {
  if (!properties_cssom_wrapper_) {
    properties_cssom_wrapper_ =
        MakeGarbageCollected<StyleRuleCSSStyleDeclaration>(
            margin_rule_->MutableProperties(),
            const_cast<CSSMarginRule*>(this));
  }
  return properties_cssom_wrapper_.Get();
}

String CSSMarginRule::name() const {
  return CssAtRuleIDToString(margin_rule_->ID()).ToString();
}

String CSSMarginRule::cssText() const {
  // TODO(mstensho): Serialization needs to be specced:
  // https://github.com/w3c/csswg-drafts/issues/9952
  StringBuilder result;
  result.Append('@');
  result.Append(CssAtRuleIDToString(margin_rule_->ID()));
  result.Append(" { ");
  String decls = margin_rule_->Properties().AsText();
  result.Append(decls);
  if (!decls.empty()) {
    result.Append(' ');
  }
  result.Append("}");
  return result.ReleaseString();
}

void CSSMarginRule::Reattach(StyleRuleBase* rule) {
  DCHECK(rule);
  margin_rule_ = To<StyleRulePageMargin>(rule);
  if (properties_cssom_wrapper_) {
    properties_cssom_wrapper_->Reattach(margin_rule_->MutableProperties());
  }
}

void CSSMarginRule::Trace(Visitor* visitor) const {
  visitor->Trace(margin_rule_);
  visitor->Trace(properties_cssom_wrapper_);
  CSSRule::Trace(visitor);
}

}  // namespace blink

```

## <a id="css-margin-rule-idl"></a>css_margin_rule.idl

Original: [complete immutable source](https://chromium.googlesource.com/chromium/src/+/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/css/css_margin_rule.idl). Path: `third_party/blink/renderer/core/css/css_margin_rule.idl`. Source bytes: 393; source lines: 12; SHA-256: `b88df00da153c30bff57628e349943fd8fed035c274292bdbcb03bc707975380`.

```webidl
// Copyright 2024 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// https://drafts.csswg.org/cssom/#the-cssmarginrule-interface

[
    Exposed=Window
] interface CSSMarginRule : CSSRule {
    readonly attribute DOMString name;
    [SameObject, PutForwards=cssText] readonly attribute CSSStyleDeclaration style;
};

```

## <a id="css-parser-cc"></a>css_parser.cc

Original: [complete immutable source](https://chromium.googlesource.com/chromium/src/+/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/css/parser/css_parser.cc). Path: `third_party/blink/renderer/core/css/parser/css_parser.cc`. Source bytes: 20401; source lines: 508; SHA-256: `d85601be5dd8c1f8cce59e3886126823cbe31d0fe0a673a0b02965972854a556`.

```cpp
// Copyright 2014 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#include "third_party/blink/renderer/core/css/parser/css_parser.h"

#include <memory>

#include "third_party/blink/renderer/core/css/css_color.h"
#include "third_party/blink/renderer/core/css/css_keyframe_rule.h"
#include "third_party/blink/renderer/core/css/parser/css_parser_fast_paths.h"
#include "third_party/blink/renderer/core/css/parser/css_parser_impl.h"
#include "third_party/blink/renderer/core/css/parser/css_parser_token_stream.h"
#include "third_party/blink/renderer/core/css/parser/css_property_parser.h"
#include "third_party/blink/renderer/core/css/parser/css_selector_parser.h"
#include "third_party/blink/renderer/core/css/parser/css_supports_parser.h"
#include "third_party/blink/renderer/core/css/parser/css_tokenizer.h"
#include "third_party/blink/renderer/core/css/parser/css_variable_parser.h"
#include "third_party/blink/renderer/core/css/properties/css_parsing_utils.h"
#include "third_party/blink/renderer/core/css/properties/shorthands.h"
#include "third_party/blink/renderer/core/css/style_color.h"
#include "third_party/blink/renderer/core/css/style_rule.h"
#include "third_party/blink/renderer/core/css/style_rule_keyframe.h"
#include "third_party/blink/renderer/core/css/style_sheet_contents.h"
#include "third_party/blink/renderer/core/execution_context/security_context.h"
#include "third_party/blink/renderer/core/frame/local_dom_window.h"
#include "third_party/blink/renderer/core/layout/layout_theme.h"
#include "third_party/blink/renderer/platform/heap/garbage_collected.h"
#include "third_party/blink/renderer/platform/heap/thread_state.h"
#include "third_party/blink/renderer/platform/wtf/text/strcat.h"

namespace blink {

bool CSSParser::ParseDeclarationList(const CSSParserContext* context,
                                     MutableCSSPropertyValueSet* property_set,
                                     const String& declaration) {
  return CSSParserImpl::ParseDeclarationList(property_set, declaration,
                                             context);
}

StyleRuleBase* CSSParser::ParseNestedDeclarationsRule(
    const CSSParserContext* context,
    CSSNestingType nesting_type,
    StyleRule* parent_rule_for_nesting,
    StringView text) {
  return CSSParserImpl::ParseNestedDeclarationsRule(
      context, nesting_type, parent_rule_for_nesting, text);
}

void CSSParser::ParseDeclarationListForInspector(
    const CSSParserContext* context,
    const String& declaration,
    CSSParserObserver& observer) {
  CSSParserImpl::ParseDeclarationListForInspector(declaration, context,
                                                  observer);
}

base::span<CSSSelector> CSSParser::ParseSelector(
    const CSSParserContext* context,
    CSSNestingType nesting_type,
    StyleRule* parent_rule_for_nesting,
    StyleSheetContents* style_sheet_contents,
    const String& selector,
    HeapVector<CSSSelector>& arena) {
  CSSParserTokenStream stream(selector);
  return CSSSelectorParser::ParseSelector(
      stream, context, nesting_type, parent_rule_for_nesting,
      /* semicolon_aborts_nested_selector */ false, style_sheet_contents,
      arena);
}

CSSSelectorList* CSSParser::ParsePageSelector(
    const CSSParserContext& context,
    StyleSheetContents* style_sheet_contents,
    const String& selector) {
  CSSParserTokenStream stream(selector);
  CSSSelectorList* selector_list =
      CSSParserImpl::ParsePageSelector(stream, style_sheet_contents, context);
  if (!stream.AtEnd()) {
    // Extra tokens at end of selector.
    return nullptr;
  }
  return selector_list;
}

StyleRuleBase* CSSParser::ParseMarginRule(const CSSParserContext* context,
                                          StyleSheetContents* style_sheet,
                                          const String& rule) {
  return CSSParserImpl::ParseRule(rule, context, CSSNestingType::kNone,
                                  /*parent_rule_for_nesting=*/nullptr,
                                  style_sheet, CSSParserImpl::kPageMarginRules);
}

StyleRuleBase* CSSParser::ParseRule(const CSSParserContext* context,
                                    StyleSheetContents* style_sheet,
                                    CSSNestingType nesting_type,
                                    StyleRule* parent_rule_for_nesting,
                                    const String& rule) {
  AllowedRules allowed_rules = CSSParserImpl::kTopLevelRules;
  allowed_rules.Remove(CSSAtRuleID::kCSSAtRuleCharset);
  if (parent_rule_for_nesting) {
    allowed_rules = allowed_rules | CSSParserImpl::kNestedGroupRules;
  }
  return CSSParserImpl::ParseRule(rule, context, nesting_type,
                                  parent_rule_for_nesting, style_sheet,
                                  allowed_rules);
}

ParseSheetResult CSSParser::ParseSheet(
    const CSSParserContext* context,
    StyleSheetContents* style_sheet,
    const String& text,
    CSSDeferPropertyParsing defer_property_parsing,
    bool allow_import_rules) {
  return CSSParserImpl::ParseStyleSheet(
      text, context, style_sheet, defer_property_parsing, allow_import_rules);
}

void CSSParser::ParseSheetForInspector(const CSSParserContext* context,
                                       StyleSheetContents* style_sheet,
                                       const String& text,
                                       CSSParserObserver& observer) {
  return CSSParserImpl::ParseStyleSheetForInspector(text, context, style_sheet,
                                                    observer);
}

MutableCSSPropertyValueSet::SetResult CSSParser::ParseValue(
    MutableCSSPropertyValueSet* declaration,
    CSSPropertyID unresolved_property,
    StringView string,
    bool important,
    const ExecutionContext* execution_context) {
  return ParseValue(
      declaration, unresolved_property, string, important,
      execution_context ? execution_context->GetSecureContextMode()
                        : SecureContextMode::kInsecureContext,
      static_cast<StyleSheetContents*>(nullptr), execution_context);
}

namespace {

// Prepares a CSSParserContext based on the specified parameters.
class LocalCSSParserContext {
  STACK_ALLOCATED();

 public:
  LocalCSSParserContext(SecureContextMode secure_context_mode,
                        StyleSheetContents* style_sheet,
                        const ExecutionContext* execution_context,
                        CSSParserMode parser_mode) {
    if (style_sheet) {
      context_ = style_sheet->ParserContext();
      if (context_->GetMode() != parser_mode) {
        // This can happen when parsing e.g. SVG attributes in the context of
        // an HTML document.
        overriding_scope_.emplace(*context_, parser_mode);
      }
    } else if (auto* window = DynamicTo<LocalDOMWindow>(execution_context)) {
      // Create a parser context using document if it exists so it can check
      // for origin trial enabled property/value.
      auto* mutable_context =
          MakeGarbageCollected<CSSParserContext>(*window->document());
      mutable_context->SetMode(parser_mode);
      context_ = mutable_context;
    } else {
      context_ = MakeGarbageCollected<CSSParserContext>(parser_mode,
                                                        secure_context_mode);
    }
  }

  const CSSParserContext* GetParserContext() const { return context_; }

 private:
  std::optional<CSSParserContext::ParserModeOverridingScope> overriding_scope_;
  const CSSParserContext* context_;
};

}  // namespace

MutableCSSPropertyValueSet::SetResult CSSParser::ParseValue(
    MutableCSSPropertyValueSet* declaration,
    CSSPropertyID unresolved_property,
    StringView string,
    bool important,
    SecureContextMode secure_context_mode,
    StyleSheetContents* style_sheet,
    const ExecutionContext* execution_context) {
  DCHECK(ThreadState::Current()->IsAllocationAllowed());
  if (string.empty()) {
    return MutableCSSPropertyValueSet::kParseError;
  }

  CSSParserMode parser_mode = declaration->CssParserMode();
  const LocalCSSParserContext local_parser_context(
      secure_context_mode, style_sheet, execution_context, parser_mode);
  const CSSParserContext* context = local_parser_context.GetParserContext();

  CSSPropertyID resolved_property = ResolveCSSPropertyID(unresolved_property);

  // See if this property has a specific fast-path parser.
  const CSSValue* value =
      CSSParserFastPaths::MaybeParseValue(resolved_property, string, context);
  if (value) {
    context->Count(unresolved_property);
    return declaration->SetLonghandProperty(CSSPropertyValue(
        CSSPropertyName(resolved_property), *value, important));
  }

  // OK, that didn't work (either the property doesn't have a fast path,
  // or the string is on some form that the fast-path parser doesn't support,
  // e.g. a parse error). See if the value we are looking for is a longhand;
  // if so, we can use a faster parsing function. In particular, we don't need
  // to set up a vector for the results, since there will be only one.
  //
  // We only allow this path in standards mode, which rules out situations
  // like @font-face parsing etc. (which have their own rules).
  const CSSProperty& property = CSSProperty::Get(resolved_property);
  if (parser_mode == kHTMLStandardMode && property.IsProperty() &&
      !property.IsShorthand()) {
    CSSParserTokenStream stream(string);
    value = CSSPropertyParser::ParseSingleValue(unresolved_property, stream,
                                                context);
    if (value != nullptr) {
      context->Count(unresolved_property);
      return declaration->SetLonghandProperty(CSSPropertyValue(
          CSSPropertyName(resolved_property), *value, important));
    }
  }

  // OK, that didn't work either, so we'll need the full-blown parser.
  return ParseValue(declaration, unresolved_property, string, important,
                    context);
}

// NOTE: This follows pretty much the exact same structure as ParseValue(),
// above.
unsigned CSSParser::ParseForPresentationStyle(
    HeapVector<CSSPropertyValue, 8>& result,
    CSSPropertyID resolved_property,
    StringView string,
    CSSParserMode parser_mode,
    StyleSheetContents* context_sheet,
    const ExecutionContext* execution_context) {
  DCHECK(ThreadState::Current()->IsAllocationAllowed());
  if (string.empty()) {
    return 0;
  }

  SecureContextMode secure_context_mode =
      execution_context ? execution_context->GetSecureContextMode()
                        : SecureContextMode::kInsecureContext;

  const LocalCSSParserContext local_parser_context(
      secure_context_mode, context_sheet, execution_context, parser_mode);
  const CSSParserContext* context = local_parser_context.GetParserContext();

  // Fast-path parser.
  const CSSValue* value =
      CSSParserFastPaths::MaybeParseValue(resolved_property, string, context);
  if (value) {
    result.emplace_back(CSSPropertyName(resolved_property), *value);
    return 1;
  }

  // Longhand parsing.
  const CSSProperty& property = CSSProperty::Get(resolved_property);
  if (parser_mode == kHTMLStandardMode && property.IsProperty() &&
      !property.IsShorthand()) {
    CSSParserTokenStream stream(string);
    value =
        CSSPropertyParser::ParseSingleValue(resolved_property, stream, context);
    if (value) {
      result.emplace_back(CSSPropertyName(resolved_property), *value);
      return 1;
    } else {
      return 0;
    }
  }

  // Full-blown parser, for shorthands and SVG.
  return CSSParserImpl::ParseValue(result, resolved_property, string, context);
}

MutableCSSPropertyValueSet::SetResult CSSParser::ParseValueForCustomProperty(
    MutableCSSPropertyValueSet* declaration,
    const AtomicString& property_name,
    StringView value,
    bool important,
    SecureContextMode secure_context_mode,
    StyleSheetContents* style_sheet,
    bool is_animation_tainted) {
  DCHECK(ThreadState::Current()->IsAllocationAllowed());
  DCHECK(CSSVariableParser::IsValidVariableName(property_name));
  if (value.empty()) {
    return MutableCSSPropertyValueSet::kParseError;
  }
  CSSParserMode parser_mode = declaration->CssParserMode();
  CSSParserContext* context;
  if (style_sheet) {
    context =
        MakeGarbageCollected<CSSParserContext>(style_sheet->ParserContext());
    context->SetMode(parser_mode);
  } else {
    context = MakeGarbageCollected<CSSParserContext>(parser_mode,
                                                     secure_context_mode);
  }
  return CSSParserImpl::ParseVariableValue(declaration, property_name, value,
                                           important, context,
                                           is_animation_tainted);
}

MutableCSSPropertyValueSet::SetResult CSSParser::ParseValue(
    MutableCSSPropertyValueSet* declaration,
    CSSPropertyID unresolved_property,
    StringView string,
    bool important,
    const CSSParserContext* context) {
  DCHECK(ThreadState::Current()->IsAllocationAllowed());
  return CSSParserImpl::ParseValue(declaration, unresolved_property, string,
                                   important, context);
}

const CSSValue* CSSParser::ParseSingleValue(CSSPropertyID unresolved_property,
                                            const String& string,
                                            const CSSParserContext* context) {
  DCHECK(ThreadState::Current()->IsAllocationAllowed());
  if (string.empty()) {
    return nullptr;
  }
  if (CSSValue* value = CSSParserFastPaths::MaybeParseValue(unresolved_property,
                                                            string, context)) {
    return value;
  }
  CSSParserTokenStream stream(string);
  return CSSPropertyParser::ParseSingleValue(unresolved_property, stream,
                                             context);
}

ImmutableCSSPropertyValueSet* CSSParser::ParseInlineStyleDeclaration(
    const String& style_string,
    Element* element) {
  return CSSParserImpl::ParseInlineStyleDeclaration(style_string, element);
}

ImmutableCSSPropertyValueSet* CSSParser::ParseInlineStyleDeclaration(
    const String& style_string,
    CSSParserMode parser_mode,
    SecureContextMode secure_context_mode,
    const Document* document) {
  return CSSParserImpl::ParseInlineStyleDeclaration(
      style_string, parser_mode, secure_context_mode, document);
}

std::unique_ptr<Vector<KeyframeOffset>> CSSParser::ParseKeyframeKeyList(
    const CSSParserContext* context,
    const String& key_list) {
  return CSSParserImpl::ParseKeyframeKeyList(context, key_list);
}

StyleRuleKeyframe* CSSParser::ParseKeyframeRule(const CSSParserContext* context,
                                                const String& rule) {
  StyleRuleBase* keyframe = CSSParserImpl::ParseRule(
      rule, context, CSSNestingType::kNone, /*parent_rule_for_nesting=*/nullptr,
      nullptr, CSSParserImpl::kKeyframeRules);
  return To<StyleRuleKeyframe>(keyframe);
}

String CSSParser::ParseCustomPropertyName(const String& name_text) {
  return CSSParserImpl::ParseCustomPropertyName(name_text);
}

bool CSSParser::ParseSupportsCondition(
    const String& condition,
    const ExecutionContext* execution_context) {
  // window.CSS.supports requires to parse as-if it was wrapped in parenthesis.
  String wrapped_condition = StrCat({"(", condition, ")"});
  CSSParserTokenStream stream(wrapped_condition);
  DCHECK(execution_context);
  // Create parser context using document so it can check for origin trial
  // enabled property/value.
  CSSParserContext* context = MakeGarbageCollected<CSSParserContext>(
      *To<LocalDOMWindow>(execution_context)->document());
  // Override the parser mode interpreted from the document as the spec
  // https://quirks.spec.whatwg.org/#css requires quirky values and colors
  // must not be supported in CSS.supports() method.
  context->SetMode(kHTMLStandardMode);
  CSSParserImpl parser(context);
  CSSSupportsParser::Result result =
      CSSSupportsParser::ConsumeSupportsCondition(stream, parser);
  if (!stream.AtEnd()) {
    result = CSSSupportsParser::Result::kParseFailure;
  }

  return result == CSSSupportsParser::Result::kSupported;
}

bool CSSParser::ParseColor(Color& color, const String& string, bool strict) {
  DCHECK(ThreadState::Current()->IsAllocationAllowed());
  if (string.empty()) {
    return false;
  }

  // The regular color parsers don't resolve named colors, so explicitly
  // handle these first.
  Color named_color;
  if (named_color.SetNamedColor(string)) {
    color = named_color;
    return true;
  }

  switch (CSSParserFastPaths::ParseColor(
      string, strict ? kHTMLStandardMode : kHTMLQuirksMode, color)) {
    case ParseColorResult::kFailure:
      break;
    case ParseColorResult::kKeyword:
      return false;
    case ParseColorResult::kColor:
      return true;
  }

  // TODO(timloh): Why is this always strict mode?
  // NOTE(ikilpatrick): We will always parse color value in the insecure
  // context mode. If a function/unit/etc will require a secure context check
  // in the future, plumbing will need to be added.
  const CSSValue* value = ParseSingleValue(
      CSSPropertyID::kColor, string,
      StrictCSSParserContext(SecureContextMode::kInsecureContext));
  auto* color_value = DynamicTo<cssvalue::CSSColor>(value);
  if (!color_value) {
    return false;
  }

  color = color_value->Value();
  return true;
}

bool CSSParser::ParseSystemColor(Color& color,
                                 const String& color_string,
                                 mojom::blink::ColorScheme color_scheme,
                                 const ui::ColorProvider* color_provider,
                                 bool can_expose_accent_color) {
  CSSValueID id = CssValueKeywordID(color_string);
  if (!StyleColor::IsSystemColorIncludingDeprecated(id)) {
    return false;
  }

  color = LayoutTheme::GetTheme().SystemColor(id, color_scheme, color_provider,
                                              can_expose_accent_color);
  return true;
}

const CSSValue* CSSParser::ParseFontFaceDescriptor(
    CSSPropertyID property_id,
    const String& property_value,
    const CSSParserContext* context) {
  auto* style =
      MakeGarbageCollected<MutableCSSPropertyValueSet>(kCSSFontFaceRuleMode);
  CSSParser::ParseValue(style, property_id, property_value, true, context);
  const CSSValue* value = style->GetPropertyCSSValue(property_id);

  return value;
}

CSSPrimitiveValue* CSSParser::ParseLengthPercentage(
    const String& string,
    const CSSParserContext* context,
    CSSParserLocalContext& local_context,
    CSSPrimitiveValue::ValueRange value_range) {
  if (string.empty() || !context) {
    return nullptr;
  }
  CSSParserTokenStream stream(string);
  // Trim whitespace from the string. It's only necessary to consume leading
  // whitespaces, since ConsumeLengthOrPercent always consumes trailing ones.
  stream.ConsumeWhitespace();
  CSSPrimitiveValue* parsed_value = css_parsing_utils::ConsumeLengthOrPercent(
      stream, *context, local_context, value_range);
  return stream.AtEnd() ? parsed_value : nullptr;
}

MutableCSSPropertyValueSet* CSSParser::ParseFont(
    const String& string,
    const ExecutionContext* execution_context) {
  DCHECK(ThreadState::Current()->IsAllocationAllowed());

  LocalCSSParserContext context(execution_context
                                    ? execution_context->GetSecureContextMode()
                                    : SecureContextMode::kInsecureContext,
                                static_cast<StyleSheetContents*>(nullptr),
                                execution_context, kHTMLStandardMode);
  CSSParserLocalContext local_context(CSSPropertyName(CSSPropertyID::kFont),
                                      CSSPropertyID::kFont);
  CSSParserTokenStream stream(string);
  HeapVector<CSSPropertyValue, 64> parsed_properties;

  if (GetCSSPropertyFont().ParseShorthand(/*important=*/true, stream,
                                          *context.GetParserContext(),
                                          local_context, parsed_properties) &&
      stream.AtEnd()) {
    auto* set =
        MakeGarbageCollected<MutableCSSPropertyValueSet>(kHTMLStandardMode);
    set->AddParsedProperties(parsed_properties);
    return set;
  }
  return nullptr;
}

}  // namespace blink

```

## <a id="css-parser-impl-cc"></a>css_parser_impl.cc

Original: [complete immutable source](https://chromium.googlesource.com/chromium/src/+/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/css/parser/css_parser_impl.cc). Path: `third_party/blink/renderer/core/css/parser/css_parser_impl.cc`. Source bytes: 140330; source lines: 3651; SHA-256: `3f9ca3b42eb1bada314a3d832bc6ea91a285aaeee039ced016e2b0d665fda674`.

```cpp
// Copyright 2014 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#include "third_party/blink/renderer/core/css/parser/css_parser_impl.h"

#include <bitset>
#include <limits>
#include <memory>
#include <optional>
#include <utility>

#include "base/auto_reset.h"
#include "base/compiler_specific.h"
#include "base/cpu.h"
#include "base/types/optional_ref.h"
#include "third_party/blink/renderer/core/animation/timeline_offset.h"
#include "third_party/blink/renderer/core/core_probes_inl.h"
#include "third_party/blink/renderer/core/css/css_custom_ident_value.h"
#include "third_party/blink/renderer/core/css/css_font_family_value.h"
#include "third_party/blink/renderer/core/css/css_identifier_value_mappings.h"
#include "third_party/blink/renderer/core/css/css_keyframes_rule.h"
#include "third_party/blink/renderer/core/css/css_position_try_rule.h"
#include "third_party/blink/renderer/core/css/css_selector.h"
#include "third_party/blink/renderer/core/css/css_style_sheet.h"
#include "third_party/blink/renderer/core/css/css_syntax_string_parser.h"
#include "third_party/blink/renderer/core/css/css_unparsed_declaration_value.h"
#include "third_party/blink/renderer/core/css/css_url_data.h"
#include "third_party/blink/renderer/core/css/navigation_query.h"
#include "third_party/blink/renderer/core/css/parser/at_rule_descriptor_parser.h"
#include "third_party/blink/renderer/core/css/parser/container_query_parser.h"
#include "third_party/blink/renderer/core/css/parser/css_at_rule_id.h"
#include "third_party/blink/renderer/core/css/parser/css_lazy_parsing_state.h"
#include "third_party/blink/renderer/core/css/parser/css_parser_observer.h"
#include "third_party/blink/renderer/core/css/parser/css_parser_token.h"
#include "third_party/blink/renderer/core/css/parser/css_parser_token_stream.h"
#include "third_party/blink/renderer/core/css/parser/css_property_parser.h"
#include "third_party/blink/renderer/core/css/parser/css_selector_parser.h"
#include "third_party/blink/renderer/core/css/parser/css_supports_parser.h"
#include "third_party/blink/renderer/core/css/parser/css_tokenizer.h"
#include "third_party/blink/renderer/core/css/parser/css_variable_parser.h"
#include "third_party/blink/renderer/core/css/parser/find_length_of_declaration_list-inl.h"
#include "third_party/blink/renderer/core/css/parser/media_query_parser.h"
#include "third_party/blink/renderer/core/css/parser/navigation_parser.h"
#include "third_party/blink/renderer/core/css/properties/css_parsing_utils.h"
#include "third_party/blink/renderer/core/css/property_registry.h"
#include "third_party/blink/renderer/core/css/style_rule.h"
#include "third_party/blink/renderer/core/css/style_rule_counter_style.h"
#include "third_party/blink/renderer/core/css/style_rule_font_feature_values.h"
#include "third_party/blink/renderer/core/css/style_rule_font_palette_values.h"
#include "third_party/blink/renderer/core/css/style_rule_function_declarations.h"
#include "third_party/blink/renderer/core/css/style_rule_import.h"
#include "third_party/blink/renderer/core/css/style_rule_keyframe.h"
#include "third_party/blink/renderer/core/css/style_rule_location.h"
#include "third_party/blink/renderer/core/css/style_rule_namespace.h"
#include "third_party/blink/renderer/core/css/style_rule_nested_declarations.h"
#include "third_party/blink/renderer/core/css/style_rule_view_transition.h"
#include "third_party/blink/renderer/core/css/style_scope.h"
#include "third_party/blink/renderer/core/css/style_sheet_contents.h"
#include "third_party/blink/renderer/core/dom/document.h"
#include "third_party/blink/renderer/core/dom/element.h"
#include "third_party/blink/renderer/core/frame/local_frame_metrics_aggregator.h"
#include "third_party/blink/renderer/core/frame/local_frame_view.h"
#include "third_party/blink/renderer/core/frame/web_feature.h"
#include "third_party/blink/renderer/platform/heap/garbage_collected.h"
#include "third_party/blink/renderer/platform/instrumentation/tracing/trace_event.h"
#include "third_party/blink/renderer/platform/instrumentation/use_counter.h"
#include "third_party/blink/renderer/platform/runtime_enabled_features.h"
#include "third_party/blink/renderer/platform/wtf/text/atomic_string.h"
#include "third_party/blink/renderer/platform/wtf/text/text_position.h"

using std::swap;

namespace blink {

namespace {

// This may still consume tokens if it fails
AtomicString ConsumeStringOrURI(
    CSSParserTokenStream& stream,
    const CSSParserContext& context,
    base::optional_ref<CSSUrlRequestModifiers> modifiers) {
  const CSSParserToken& token = stream.Peek();

  if (token.GetType() == kStringToken || token.GetType() == kUrlToken) {
    return stream.ConsumeIncludingWhitespace().Value().ToAtomicString();
  }

  if (token.GetType() != kFunctionToken ||
      !EqualIgnoringAsciiCase(token.Value(), "url")) {
    return AtomicString();
  }

  AtomicString result;
  {
    CSSParserTokenStream::BlockGuard guard(stream);
    stream.ConsumeWhitespace();
    // If the block doesn't start with a quote, then the tokenizer
    // would return a kUrlToken or kBadUrlToken instead of a
    // kFunctionToken. Note also that this Peek() placates the
    // DCHECK that we Peek() before Consume().
    DCHECK(stream.Peek().GetType() == kStringToken ||
           stream.Peek().GetType() == kBadStringToken)
        << "Got unexpected token " << stream.Peek();
    const CSSParserToken& uri = stream.ConsumeIncludingWhitespace();
    if (uri.GetType() != kBadStringToken) {
      const bool should_consume_modifiers =
          RuntimeEnabledFeatures::CSSURLRequestModifiersEnabled() &&
          modifiers.has_value();
      const bool consumed_modifiers =
          should_consume_modifiers &&
          css_parsing_utils::ConsumeUrlRequestModifiers(stream, context,
                                                        *modifiers);
      if ((!should_consume_modifiers || consumed_modifiers) &&
          stream.UncheckedAtEnd()) {
        DCHECK_EQ(uri.GetType(), kStringToken);
        result = uri.Value().ToAtomicString();
      }
    }
  }
  stream.ConsumeWhitespace();
  return result;
}

// Finds the longest prefix of |stream| that matches a <layer-name> and parses
// it. Returns an empty result with |stream| unmodified if parsing fails.
StyleRuleBase::LayerName ConsumeCascadeLayerName(CSSParserTokenStream& stream) {
  CSSParserTokenStream::State savepoint = stream.Save();
  StyleRuleBase::LayerName name;
  while (!stream.AtEnd() && stream.Peek().GetType() == kIdentToken) {
    const CSSParserToken& name_part = stream.Consume();
    name.emplace_back(name_part.Value().ToString());

    // Check if we have a next part.
    if (stream.Peek().GetType() != kDelimiterToken ||
        stream.Peek().Delimiter() != '.') {
      break;
    }
    CSSParserTokenStream::State inner_savepoint = stream.Save();
    stream.Consume();
    if (stream.Peek().GetType() != kIdentToken) {
      stream.Restore(inner_savepoint);
      break;
    }
  }

  if (!name.size()) {
    stream.Restore(savepoint);
  } else {
    stream.ConsumeWhitespace();
  }

  return name;
}

StyleRule::RuleType RuleTypeForMutableDeclaration(
    MutableCSSPropertyValueSet* declaration) {
  switch (declaration->CssParserMode()) {
    case kCSSFontFaceRuleMode:
      return StyleRule::kFontFace;
    case kCSSKeyframeRuleMode:
      return StyleRule::kKeyframe;
    case kCSSPropertyRuleMode:
      return StyleRule::kProperty;
    case kCSSFontPaletteValuesRuleMode:
      return StyleRule::kFontPaletteValues;
    case kCSSPositionTryRuleMode:
      return StyleRule::kPositionTry;
    case kCSSFunctionDescriptorsMode:
      return StyleRule::kFunction;
    case kCSSCounterStyleRuleMode:
      return StyleRule::kCounterStyle;
    default:
      return StyleRule::kStyle;
  }
}

std::optional<StyleRuleFontFeature::FeatureType> ToStyleRuleFontFeatureType(
    CSSAtRuleID rule_id) {
  switch (rule_id) {
    case CSSAtRuleID::kCSSAtRuleStylistic:
      return StyleRuleFontFeature::FeatureType::kStylistic;
    case CSSAtRuleID::kCSSAtRuleStyleset:
      return StyleRuleFontFeature::FeatureType::kStyleset;
    case CSSAtRuleID::kCSSAtRuleCharacterVariant:
      return StyleRuleFontFeature::FeatureType::kCharacterVariant;
    case CSSAtRuleID::kCSSAtRuleSwash:
      return StyleRuleFontFeature::FeatureType::kSwash;
    case CSSAtRuleID::kCSSAtRuleOrnaments:
      return StyleRuleFontFeature::FeatureType::kOrnaments;
    case CSSAtRuleID::kCSSAtRuleAnnotation:
      return StyleRuleFontFeature::FeatureType::kAnnotation;
    default:
      NOTREACHED();
  }
}

}  // namespace

CSSParserImpl::CSSParserImpl(const CSSParserContext* context,
                             StyleSheetContents* style_sheet)
    : context_(context),
      style_sheet_(style_sheet),
      observer_(nullptr),
      lazy_state_(nullptr) {}

MutableCSSPropertyValueSet::SetResult CSSParserImpl::ParseValue(
    MutableCSSPropertyValueSet* declaration,
    CSSPropertyID unresolved_property,
    StringView string,
    bool important,
    const CSSParserContext* context) {
  STACK_UNINITIALIZED CSSParserImpl parser(context);
  StyleRule::RuleType rule_type = RuleTypeForMutableDeclaration(declaration);
  CSSParserTokenStream stream(string);
  parser.ConsumeDeclarationValue(stream, unresolved_property,
                                 /*is_in_declaration_list=*/false, rule_type);
  if (parser.parsed_properties_.empty()) {
    return MutableCSSPropertyValueSet::kParseError;
  }
  if (important) {
    for (CSSPropertyValue& property : parser.parsed_properties_) {
      property.SetImportant();
    }
  }
  return declaration->AddParsedProperties(parser.parsed_properties_);
}

unsigned CSSParserImpl::ParseValue(HeapVector<CSSPropertyValue, 8>& result,
                                   CSSPropertyID unresolved_property,
                                   StringView string,
                                   const CSSParserContext* context) {
  STACK_UNINITIALIZED CSSParserImpl parser(context);
  CSSParserTokenStream stream(string);
  parser.ConsumeDeclarationValue(stream, unresolved_property,
                                 /*is_in_declaration_list=*/false,
                                 StyleRule::kStyle);
  result.append_range(parser.parsed_properties_);
  return parser.parsed_properties_.size();
}

MutableCSSPropertyValueSet::SetResult CSSParserImpl::ParseVariableValue(
    MutableCSSPropertyValueSet* declaration,
    const AtomicString& property_name,
    StringView value,
    bool important,
    const CSSParserContext* context,
    bool is_animation_tainted) {
  STACK_UNINITIALIZED CSSParserImpl parser(context);
  CSSParserTokenStream stream(value);
  if (!parser.ConsumeVariableValue(stream, property_name,
                                   /*allow_important_annotation=*/false,
                                   is_animation_tainted)) {
    return MutableCSSPropertyValueSet::kParseError;
  }
  if (important) {
    parser.parsed_properties_.back().SetImportant();
  }
  return declaration->AddParsedProperties(parser.parsed_properties_);
}

static inline void FilterProperties(
    HeapVector<CSSPropertyValue, 64>& values,
    wtf_size_t& unused_entries,
    std::bitset<kNumCSSProperties>& seen_properties,
    HashSet<AtomicString>& seen_custom_properties) {
  // Move !important declarations last, using a simple insertion sort.
  // This is O(n²), but n is typically small, and std::stable_partition
  // wants to allocate memory to get to O(n), which is overkill here.
  // Moreover, this is O(n) if there are no !important properties
  // (the common case) or only !important properties.
  wtf_size_t last_nonimportant_idx = values.size() - 1;
  for (wtf_size_t i = values.size(); i--;) {
    if (values[i].IsImportant()) {
      if (i != last_nonimportant_idx) {
        // Move this element to the end, preserving the order
        // of the other elements.
        CSSPropertyValue tmp = std::move(values[i]);
        for (unsigned j = i; j < last_nonimportant_idx; ++j) {
          values[j] = std::move(values[j + 1]);
        }
        values[last_nonimportant_idx] = std::move(tmp);
      }
      --last_nonimportant_idx;
    }
  }

  // Add properties in reverse order so that highest priority definitions are
  // reached first. Duplicate definitions can then be ignored when found.
  for (wtf_size_t i = values.size(); i--;) {
    const CSSPropertyValue& property = values[i];
    if (property.PropertyID() == CSSPropertyID::kVariable) {
      const AtomicString& name = property.CustomPropertyName();
      if (!seen_custom_properties.insert(name).is_new_entry) {
        continue;
      }
    } else {
      const unsigned property_id_index =
          GetCSSPropertyIDIndex(property.PropertyID());
      if (seen_properties.test(property_id_index)) {
        continue;
      }
      seen_properties.set(property_id_index);
    }
    values[--unused_entries] = property;
  }
}

static ImmutableCSSPropertyValueSet* CreateCSSPropertyValueSet(
    HeapVector<CSSPropertyValue, 64>& parsed_properties,
    CSSParserMode mode,
    const Document* document) {
  if (mode != kHTMLQuirksMode && (parsed_properties.size() < 2 ||
                                  (parsed_properties.size() == 2 &&
                                   parsed_properties[0].PropertyID() !=
                                       parsed_properties[1].PropertyID()))) {
    // Fast path for the situations where we can trivially detect that there can
    // be no collision between properties, and don't need to reorder, make
    // bitsets, or similar.
    ImmutableCSSPropertyValueSet* result =
        ImmutableCSSPropertyValueSet::Create(parsed_properties, mode);
    parsed_properties.resize(0);  // clear() deallocates the backing.
    return result;
  }

  std::bitset<kNumCSSProperties> seen_properties;
  wtf_size_t unused_entries = parsed_properties.size();
  HashSet<AtomicString> seen_custom_properties;

  FilterProperties(parsed_properties, unused_entries, seen_properties,
                   seen_custom_properties);

  // TODO: When we remove this use counter, we can move seen_properties
  // into FilterProperties().
  bool count_cursor_hand = false;
  if (document && mode == kHTMLQuirksMode &&
      seen_properties.test(GetCSSPropertyIDIndex(CSSPropertyID::kCursor))) {
    // See if the properties contain “cursor: hand” without also containing
    // “cursor: pointer”. This is a reasonable approximation for whether
    // removing support for the former would actually matter. (Of course,
    // we don't check whether “cursor: hand” could lose in the cascade
    // due to properties coming from other declarations, but that would be
    // much more complicated)
    bool contains_cursor_hand = false;
    bool contains_cursor_pointer = false;
    for (const CSSPropertyValue& property : parsed_properties) {
      const CSSIdentifierValue* value =
          DynamicTo<CSSIdentifierValue>(property.Value());
      if (value) {
        if (value->WasQuirky()) {
          contains_cursor_hand = true;
        } else if (value->GetValueID() == CSSValueID::kPointer) {
          contains_cursor_pointer = true;
        }
      }
    }
    if (contains_cursor_hand && !contains_cursor_pointer) {
      document->CountUse(WebFeature::kQuirksModeCursorHand);
      count_cursor_hand = true;
    }
  }

  ImmutableCSSPropertyValueSet* result = ImmutableCSSPropertyValueSet::Create(
      base::span(parsed_properties).subspan(unused_entries), mode,
      count_cursor_hand);
  parsed_properties.resize(0);  // clear() deallocates the backing.
  return result;
}

ImmutableCSSPropertyValueSet* CSSParserImpl::ParseInlineStyleDeclaration(
    const String& string,
    Element* element) {
  Document& document = element->GetDocument();
  auto* context = MakeGarbageCollected<CSSParserContext>(
      document.ElementSheet().Contents()->ParserContext(), &document);
  CSSParserMode mode = element->IsHTMLElement() && !document.InQuirksMode()
                           ? kHTMLStandardMode
                           : kHTMLQuirksMode;
  context->SetMode(mode);
  CSSParserImpl parser(context, document.ElementSheet().Contents());
  CSSParserTokenStream stream(string);
  parser.ConsumeBlockContents(stream, StyleRule::kStyle, CSSNestingType::kNone,
                              /*parent_rule_for_nesting=*/nullptr,
                              /*nested_declarations_start_index=*/kNotFound,
                              /*child_rules=*/nullptr);
  return CreateCSSPropertyValueSet(parser.parsed_properties_, mode, &document);
}

ImmutableCSSPropertyValueSet* CSSParserImpl::ParseInlineStyleDeclaration(
    const String& string,
    CSSParserMode parser_mode,
    SecureContextMode secure_context_mode,
    const Document* document) {
  auto* context =
      MakeGarbageCollected<CSSParserContext>(parser_mode, secure_context_mode);
  CSSParserImpl parser(context);
  CSSParserTokenStream stream(string);
  parser.ConsumeBlockContents(stream, StyleRule::kStyle, CSSNestingType::kNone,
                              /*parent_rule_for_nesting=*/nullptr,
                              /*nested_declarations_start_index=*/kNotFound,
                              /*child_rules=*/nullptr);
  return CreateCSSPropertyValueSet(parser.parsed_properties_, parser_mode,
                                   document);
}

bool CSSParserImpl::ParseDeclarationList(
    MutableCSSPropertyValueSet* declaration,
    const String& string,
    const CSSParserContext* context) {
  CSSParserImpl parser(context);
  StyleRule::RuleType rule_type = RuleTypeForMutableDeclaration(declaration);
  CSSParserTokenStream stream(string);
  // See function declaration comment for why parent_rule_for_nesting ==
  // nullptr.
  parser.ConsumeBlockContents(stream, rule_type, CSSNestingType::kNone,
                              /*parent_rule_for_nesting=*/nullptr,
                              /*nested_declarations_start_index=*/kNotFound,
                              /*child_rules=*/nullptr);
  if (parser.parsed_properties_.empty()) {
    return false;
  }

  std::bitset<kNumCSSProperties> seen_properties;
  wtf_size_t unused_entries = parser.parsed_properties_.size();
  HashSet<AtomicString> seen_custom_properties;
  FilterProperties(parser.parsed_properties_, unused_entries, seen_properties,
                   seen_custom_properties);
  return declaration->AddParsedProperties(
      base::span(parser.parsed_properties_).subspan(unused_entries));
}

StyleRuleBase* CSSParserImpl::ParseNestedDeclarationsRule(
    const CSSParserContext* context,
    CSSNestingType nesting_type,
    StyleRule* parent_rule_for_nesting,
    StringView text) {
  CSSParserImpl parser(context);
  CSSParserTokenStream stream(text);

  HeapVector<Member<StyleRuleBase>, 4> child_rules;

  // Using nested_declarations_start_index=0u causes the leading block
  // of declarations (the only block) to be wrapped in a CSSNestedDeclarations
  // rule.
  //
  // See comment above CSSParserImpl::ConsumeBlockContents (definition)
  // for more on nested_declarations_start_index.
  parser.ConsumeBlockContents(stream, StyleRule::RuleType::kStyle, nesting_type,
                              parent_rule_for_nesting,
                              /*nested_declarations_start_index=*/0u,
                              &child_rules);

  return child_rules.size() == 1u ? child_rules.back().Get() : nullptr;
}

StyleRuleBase* CSSParserImpl::ParseRule(const String& string,
                                        const CSSParserContext* context,
                                        CSSNestingType nesting_type,
                                        StyleRule* parent_rule_for_nesting,
                                        StyleSheetContents* style_sheet,
                                        AllowedRules allowed_rules) {
  CSSParserImpl parser(context, style_sheet);
  CSSParserTokenStream stream(string);
  stream.ConsumeWhitespace();
  if (stream.UncheckedAtEnd()) {
    return nullptr;  // Parse error, empty rule
  }
  StyleRuleBase* rule;
  if (stream.UncheckedPeek().GetType() == kAtKeywordToken) {
    rule = parser.ConsumeAtRule(stream, allowed_rules, nesting_type,
                                parent_rule_for_nesting);
  } else {
    rule = parser.ConsumeQualifiedRule(stream, allowed_rules, nesting_type,
                                       parent_rule_for_nesting);
  }
  if (!rule) {
    return nullptr;  // Parse error, failed to consume rule
  }
  stream.ConsumeWhitespace();
  if (!rule || !stream.UncheckedAtEnd()) {
    return nullptr;  // Parse error, trailing garbage
  }
  return rule;
}

ParseSheetResult CSSParserImpl::ParseStyleSheet(
    const String& string,
    const CSSParserContext* context,
    StyleSheetContents* style_sheet,
    CSSDeferPropertyParsing defer_property_parsing,
    bool allow_import_rules) {
  std::optional<LocalFrameMetricsAggregator::ScopedUkmHierarchicalTimer> timer;
  if (context->GetDocument() && context->GetDocument()->View()) {
    if (auto* metrics_aggregator =
            context->GetDocument()->View()->GetMetricsAggregator()) {
      timer.emplace(metrics_aggregator->GetScopedTimer(
          static_cast<size_t>(LocalFrameMetricsAggregator::kParseStyleSheet)));
    }
  }
  TRACE_EVENT_BEGIN("blink,blink_style", "CSSParserImpl::parseStyleSheet",
                    "baseUrl", context->BaseURL().GetString().Utf8(), "mode",
                    context->Mode());

  TRACE_EVENT_BEGIN("blink,blink_style",
                    "CSSParserImpl::parseStyleSheet.parse");
  CSSParserTokenStream stream(string);
  CSSParserImpl parser(context, style_sheet);
  if (defer_property_parsing == CSSDeferPropertyParsing::kYes) {
    parser.lazy_state_ = MakeGarbageCollected<CSSLazyParsingState>(
        context, string, parser.style_sheet_);
  }
  ParseSheetResult result = ParseSheetResult::kSucceeded;
  bool first_rule_valid = parser.ConsumeRuleList(
      stream, kTopLevelRules, /*allow_cdo_cdc_tokens=*/true,
      CSSNestingType::kNone,
      /*parent_rule_for_nesting=*/nullptr,
      [&style_sheet, &result, &string, allow_import_rules, context](
          StyleRuleBase* rule, wtf_size_t offset) {
        if (rule->IsCharsetRule()) {
          return;
        }
        if (rule->IsImportRule()) {
          if (!allow_import_rules || context->IsForMarkupSanitization()) {
            result = ParseSheetResult::kHasUnallowedImportRule;
            return;
          }

          Document* document = style_sheet->AnyOwnerDocument();
          if (document) {
            TextPosition position = TextPosition::MinimumPosition();
            probe::GetTextPosition(document, offset, &string, &position);
            To<StyleRuleImport>(rule)->SetPositionHint(position);
          }
        }

        style_sheet->ParserAppendRule(rule);
      });
  style_sheet->SetHasSyntacticallyValidCSSHeader(first_rule_valid);
  TRACE_EVENT_END("blink,blink_style");

  TRACE_EVENT_END("blink,blink_style", "tokenCount", stream.TokenCount(),
                  "length", string.length());
  return result;
}

// static
CSSSelectorList* CSSParserImpl::ParsePageSelector(
    CSSParserTokenStream& stream,
    StyleSheetContents* style_sheet,
    const CSSParserContext& context) {
  // We only support a small subset of the css-page spec.
  stream.ConsumeWhitespace();
  AtomicString type_selector;
  if (stream.Peek().GetType() == kIdentToken) {
    type_selector = stream.Consume().Value().ToAtomicString();
  }

  AtomicString pseudo;
  if (stream.Peek().GetType() == kColonToken) {
    stream.Consume();
    if (stream.Peek().GetType() != kIdentToken) {
      return nullptr;
    }
    pseudo = stream.Consume().Value().ToAtomicString();
  }

  stream.ConsumeWhitespace();

  HeapVector<CSSSelector> selectors;
  if (!type_selector.IsNull()) {
    selectors.push_back(
        CSSSelector(QualifiedName(g_null_atom, type_selector, g_star_atom)));
  }
  if (!pseudo.IsNull()) {
    CSSSelector selector;
    selector.SetMatch(CSSSelector::kPagePseudoClass);
    selector.UpdatePseudoPage(pseudo.ToAsciiLower(), context.GetDocument());
    if (selector.GetPseudoType() == CSSSelector::kPseudoUnknown) {
      return nullptr;
    }
    if (selectors.size() != 0) {
      selectors[0].SetLastInComplexSelector(false);
    }
    selectors.push_back(selector);
  }
  if (selectors.empty()) {
    selectors.push_back(CSSSelector());
  }
  selectors[0].SetForPage();
  selectors.back().SetLastInComplexSelector(true);
  return CSSSelectorList::AdoptSelectorVector(
      base::span<CSSSelector>(selectors));
}

std::unique_ptr<Vector<KeyframeOffset>> CSSParserImpl::ParseKeyframeKeyList(
    const CSSParserContext* context,
    const String& key_list) {
  CSSParserTokenStream stream(key_list);
  std::unique_ptr<Vector<KeyframeOffset>> result =
      ConsumeKeyframeKeyList(context, stream);
  if (stream.AtEnd()) {
    return result;
  } else {
    return nullptr;
  }
}

String CSSParserImpl::ParseCustomPropertyName(StringView name_text) {
  CSSParserTokenStream stream(name_text);
  const CSSParserToken name_token = stream.Peek();
  if (!CSSVariableParser::IsValidVariableName(name_token)) {
    return {};
  }
  stream.ConsumeIncludingWhitespace();
  if (!stream.AtEnd()) {
    return {};
  }
  return name_token.Value().ToString();
}

bool CSSParserImpl::ConsumeSupportsDeclaration(CSSParserTokenStream& stream) {
  DCHECK(parsed_properties_.empty());
  // Even though we might use an observer here, this is just to test if we
  // successfully parse the stream, so we can temporarily remove the observer.
  CSSParserObserver* observer_copy = observer_;
  observer_ = nullptr;
  ConsumeDeclaration(stream, StyleRule::kStyle);
  observer_ = observer_copy;

  bool result = !parsed_properties_.empty();
  parsed_properties_.resize(0);  // clear() deallocates the backing.
  return result;
}

void CSSParserImpl::ParseDeclarationListForInspector(
    const String& declaration,
    const CSSParserContext* context,
    CSSParserObserver& observer) {
  CSSParserImpl parser(context);
  parser.observer_ = &observer;
  observer.StartRuleHeader(StyleRule::kStyle, 0);
  observer.EndRuleHeader(1);
  CSSParserTokenStream stream(declaration);
  observer.StartRuleBody(stream.Offset());
  parser.ConsumeBlockContents(stream, StyleRule::kStyle, CSSNestingType::kNone,
                              /*parent_rule_for_nesting=*/nullptr,
                              /*nested_declarations_start_index=*/kNotFound,
                              /*child_rules=*/nullptr);
  observer.EndRuleBody(stream.LookAheadOffset());
}

void CSSParserImpl::ParseStyleSheetForInspector(const String& string,
                                                const CSSParserContext* context,
                                                StyleSheetContents* style_sheet,
                                                CSSParserObserver& observer) {
  CSSParserImpl parser(context, style_sheet);
  parser.observer_ = &observer;
  CSSParserTokenStream stream(string);
  bool first_rule_valid = parser.ConsumeRuleList(
      stream, kTopLevelRules, /*allow_cdo_cdc_tokens=*/true,
      CSSNestingType::kNone,
      /*parent_rule_for_nesting=*/nullptr,
      [&style_sheet](StyleRuleBase* rule, wtf_size_t) {
        if (rule->IsCharsetRule()) {
          return;
        }
        style_sheet->ParserAppendRule(rule);
      });
  style_sheet->SetHasSyntacticallyValidCSSHeader(first_rule_valid);
}

CSSPropertyValueSet* CSSParserImpl::ParseDeclarationListForLazyStyle(
    const String& string,
    wtf_size_t offset,
    const CSSParserContext* context) {
  // NOTE: Lazy parsing does not support nested rules (it happens
  // only after matching, which means that we cannot insert child rules
  // we encounter during parsing -- we never match against them),
  // so parent_rule_for_nesting is always nullptr here. The parser
  // explicitly makes sure we do not invoke lazy parsing for rules
  // with child rules in them.
  CSSParserTokenStream stream(string, offset);
  CSSParserTokenStream::BlockGuard guard(stream);
  CSSParserImpl parser(context);
  parser.ConsumeBlockContents(stream, StyleRule::kStyle, CSSNestingType::kNone,
                              /*parent_rule_for_nesting=*/nullptr,
                              /*nested_declarations_start_index=*/kNotFound,
                              /*child_rules=*/nullptr);
  return CreateCSSPropertyValueSet(parser.parsed_properties_, context->Mode(),
                                   context->GetDocument());
}

static AllowedRules ComputeNewAllowedRules(
    AllowedRules old_allowed_rules,
    StyleRuleBase* rule,
    bool& seen_import_or_namespace_rule) {
  if (!rule) {
    return old_allowed_rules;
  }
  // Certain rules have ordering restrictions; we expect to see them
  // in this order:
  //
  // - @charset
  // - [ @layer (statement) ]
  // - @import
  // - @namespace
  //
  // The restrictions are applied by disallowing certain rule types once
  // a "later" rule has been seen, for example: once @import (or @namespace,
  // or any later regular rule) has been seen, it's too late to parse @charset.
  //
  // @layer statement rules are in brackets above because they are special:
  // they can be used before @import/namespace rules (without causing them
  // to become disallowed), but can *also* be used as a regular rule
  // (i.e. where @layer block rules are allowed).
  //
  // https://drafts.csswg.org/css-cascade-5/#layer-empty
  AllowedRules new_allowed_rules = old_allowed_rules;
  if (rule->IsCharsetRule()) {
    // @charset is only allowed once.
    new_allowed_rules.Remove(CSSAtRuleID::kCSSAtRuleCharset);
  } else if (rule->IsLayerStatementRule() && !seen_import_or_namespace_rule) {
    // Any number of @layer statements may appear before @import rules.
    new_allowed_rules.Remove(CSSAtRuleID::kCSSAtRuleCharset);
  } else if (rule->IsImportRule()) {
    // @layer statements are still allowed once @import rules have been seen,
    // but they are treated as regular rules ("else" branch).
    seen_import_or_namespace_rule = true;
    new_allowed_rules.Remove(CSSAtRuleID::kCSSAtRuleCharset);
  } else if (rule->IsNamespaceRule()) {
    // @layer statements are still allowed once @namespace rules have been seen,
    // but they are treated as regular rules ("else" branch).
    seen_import_or_namespace_rule = true;
    new_allowed_rules.Remove(CSSAtRuleID::kCSSAtRuleCharset);
    new_allowed_rules.Remove(CSSAtRuleID::kCSSAtRuleImport);
  } else {
    // Any regular rule must come after @charset/@import/@namespace.
    new_allowed_rules.Remove(CSSAtRuleID::kCSSAtRuleCharset);
    new_allowed_rules.Remove(CSSAtRuleID::kCSSAtRuleImport);
    new_allowed_rules.Remove(CSSAtRuleID::kCSSAtRuleNamespace);
  }
  return new_allowed_rules;
}

template <typename T>
bool CSSParserImpl::ConsumeRuleList(CSSParserTokenStream& stream,
                                    AllowedRules allowed_rules,
                                    bool allow_cdo_cdc_tokens,
                                    CSSNestingType nesting_type,
                                    StyleRule* parent_rule_for_nesting,
                                    const T callback) {
  bool seen_rule = false;
  bool seen_import_or_namespace_rule = false;
  bool first_rule_valid = false;
  while (!stream.AtEnd()) {
    wtf_size_t offset = stream.Offset();
    StyleRuleBase* rule = nullptr;
    switch (stream.UncheckedPeek().GetType()) {
      case kWhitespaceToken:
        stream.UncheckedConsume();
        continue;
      case kAtKeywordToken:
        rule = ConsumeAtRule(stream, allowed_rules, nesting_type,
                             parent_rule_for_nesting);
        break;
      case kCDOToken:
      case kCDCToken:
        if (allow_cdo_cdc_tokens) {
          stream.UncheckedConsume();
          continue;
        }
        [[fallthrough]];
      default:
        rule = ConsumeQualifiedRule(stream, allowed_rules, nesting_type,
                                    parent_rule_for_nesting);
        break;
    }
    if (!seen_rule) {
      seen_rule = true;
      first_rule_valid = rule;
    }
    if (rule) {
      allowed_rules = ComputeNewAllowedRules(allowed_rules, rule,
                                             seen_import_or_namespace_rule);
      callback(rule, offset);
    }
    DCHECK_GT(stream.Offset(), offset);
  }

  return first_rule_valid;
}

// Same as ConsumeEndOfPreludeForAtRuleWithBlock() below, but for at-rules
// that don't have a block and are terminated only by semicolon.
bool CSSParserImpl::ConsumeEndOfPreludeForAtRuleWithoutBlock(
    CSSParserTokenStream& stream,
    CSSAtRuleID id) {
  stream.ConsumeWhitespace();
  if (stream.AtEnd()) {
    return true;
  }
  if (stream.UncheckedPeek().GetType() == kSemicolonToken) {
    stream.UncheckedConsume();  // kSemicolonToken
    return true;
  }

  if (observer_) {
    observer_->ObserveErroneousAtRule(stream.Offset(), id);
  }

  // Consume the erroneous block.
  ConsumeErroneousAtRule(stream, id);
  return false;  // Parse error, we expected no block.
}

// Call this after parsing the prelude of an at-rule that takes a block
// (i.e. @foo-rule <prelude> /* call here */ { ... }). It will check
// that there is no junk after the prelude, and that there is indeed
// a block starting. If either of these are false, then it will consume
// until the end of the declaration (any junk after the prelude,
// and the block if one exists), notify the observer, and return false.
bool CSSParserImpl::ConsumeEndOfPreludeForAtRuleWithBlock(
    CSSParserTokenStream& stream,
    CSSAtRuleID id) {
  stream.ConsumeWhitespace();

  if (stream.AtEnd()) {
    // Parse error, we expected a block.
    if (observer_) {
      observer_->ObserveErroneousAtRule(stream.Offset(), id);
    }
    return false;
  }
  if (stream.UncheckedPeek().GetType() == kLeftBraceToken) {
    return true;
  }

  // We have a parse error, so we need to return an error, but before that,
  // we need to consume until the end of the declaration.
  ConsumeErroneousAtRule(stream, id);
  return false;
}

void CSSParserImpl::ConsumeErroneousAtRule(CSSParserTokenStream& stream,
                                           CSSAtRuleID id) {
  if (observer_) {
    observer_->ObserveErroneousAtRule(stream.Offset(), id);
  }
  // Consume the prelude and block if present.
  stream.SkipUntilPeekedTypeIs<kLeftBraceToken, kSemicolonToken>();
  if (!stream.AtEnd()) {
    if (stream.UncheckedPeek().GetType() == kLeftBraceToken) {
      CSSParserTokenStream::BlockGuard guard(stream);
    } else {
      stream.UncheckedConsume();  // kSemicolonToken
    }
  }
}

StyleRuleBase* CSSParserImpl::ConsumeAtRule(
    CSSParserTokenStream& stream,
    AllowedRules allowed_rules,
    CSSNestingType nesting_type,
    StyleRule* parent_rule_for_nesting) {
  DCHECK_EQ(stream.Peek().GetType(), kAtKeywordToken);
  CSSParserToken name_token =
      stream.ConsumeIncludingWhitespace();  // Must live until CssAtRuleID().
  const StringView name = name_token.Value();
  const CSSAtRuleID id = CssAtRuleID(name);
  return ConsumeAtRuleContents(id, stream, allowed_rules, nesting_type,
                               parent_rule_for_nesting);
}

StyleRuleBase* CSSParserImpl::ConsumeAtRuleContents(
    CSSAtRuleID id,
    CSSParserTokenStream& stream,
    AllowedRules allowed_rules,
    CSSNestingType nesting_type,
    StyleRule* parent_rule_for_nesting) {
  if (!allowed_rules.Has(id)) {
    ConsumeErroneousAtRule(stream, id);
    return nullptr;
  }

  if (id != CSSAtRuleID::kCSSAtRuleInvalid &&
      context_->IsUseCounterRecordingEnabled()) {
    CountAtRule(context_, id);
  }

  stream.EnsureLookAhead();
  switch (id) {
    case CSSAtRuleID::kCSSAtRuleViewTransition:
      return ConsumeViewTransitionRule(stream);
    case CSSAtRuleID::kCSSAtRuleContainer:
      return ConsumeContainerRule(stream, nesting_type,
                                  parent_rule_for_nesting);
    case CSSAtRuleID::kCSSAtRuleMedia:
      return ConsumeMediaRule(stream, nesting_type, parent_rule_for_nesting);
    case CSSAtRuleID::kCSSAtRuleSupports:
      return ConsumeSupportsRule(stream, nesting_type, parent_rule_for_nesting);
    case CSSAtRuleID::kCSSAtRuleStartingStyle:
      return ConsumeStartingStyleRule(stream, nesting_type,
                                      parent_rule_for_nesting);
    case CSSAtRuleID::kCSSAtRuleFontFace:
      return ConsumeFontFaceRule(stream);
    case CSSAtRuleID::kCSSAtRuleFontPaletteValues:
      return ConsumeFontPaletteValuesRule(stream);
    case CSSAtRuleID::kCSSAtRuleFontFeatureValues:
      return ConsumeFontFeatureValuesRule(stream);
    case CSSAtRuleID::kCSSAtRuleWebkitKeyframes:
      return ConsumeKeyframesRule(true, stream);
    case CSSAtRuleID::kCSSAtRuleKeyframes:
      return ConsumeKeyframesRule(false, stream);
    case CSSAtRuleID::kCSSAtRuleLayer:
      return ConsumeLayerRule(stream, nesting_type, parent_rule_for_nesting);
    case CSSAtRuleID::kCSSAtRulePage:
      return ConsumePageRule(stream);
    case CSSAtRuleID::kCSSAtRuleProperty:
      return ConsumePropertyRule(stream);
    case CSSAtRuleID::kCSSAtRuleLocation:
      return ConsumeLocationRule(stream);
    case CSSAtRuleID::kCSSAtRuleNavigation:
      return ConsumeNavigationRule(stream, nesting_type,
                                   parent_rule_for_nesting);
    case CSSAtRuleID::kCSSAtRuleScope:
      return ConsumeScopeRule(stream, nesting_type, parent_rule_for_nesting);
    case CSSAtRuleID::kCSSAtRuleCounterStyle:
      return ConsumeCounterStyleRule(stream);
    case CSSAtRuleID::kCSSAtRuleFunction:
      return ConsumeFunctionRule(stream);
    case CSSAtRuleID::kCSSAtRuleMixin:
      return ConsumeMixinRule(stream);
    case CSSAtRuleID::kCSSAtRuleApplyMixin:
      return ConsumeApplyMixinRule(stream);
    case CSSAtRuleID::kCSSAtRuleContents:
      return ConsumeContentsRule(stream);
    case CSSAtRuleID::kCSSAtRuleResult:
      return ConsumeResultRule(stream);
    case CSSAtRuleID::kCSSAtRulePrivate:
      return ConsumePrivateRule(stream);
    case CSSAtRuleID::kCSSAtRulePositionTry:
      return ConsumePositionTryRule(stream);
    case CSSAtRuleID::kCSSAtRuleCharset:
      return ConsumeCharsetRule(stream);
    case CSSAtRuleID::kCSSAtRuleImport: {
      // @import rules have a URI component that is not technically part of the
      // prelude.
      CSSUrlRequestModifiers modifiers;
      AtomicString uri = ConsumeStringOrURI(stream, *context_, modifiers);
      stream.EnsureLookAhead();
      return ConsumeImportRule(std::move(uri), stream, modifiers);
    }
    case CSSAtRuleID::kCSSAtRuleNamespace:
      return ConsumeNamespaceRule(stream);
    case CSSAtRuleID::kCSSAtRuleStylistic:
    case CSSAtRuleID::kCSSAtRuleStyleset:
    case CSSAtRuleID::kCSSAtRuleCharacterVariant:
    case CSSAtRuleID::kCSSAtRuleSwash:
    case CSSAtRuleID::kCSSAtRuleOrnaments:
    case CSSAtRuleID::kCSSAtRuleAnnotation:
      return ConsumeFontFeatureRule(id, stream);
    case CSSAtRuleID::kCSSAtRuleTopLeftCorner:
    case CSSAtRuleID::kCSSAtRuleTopLeft:
    case CSSAtRuleID::kCSSAtRuleTopCenter:
    case CSSAtRuleID::kCSSAtRuleTopRight:
    case CSSAtRuleID::kCSSAtRuleTopRightCorner:
    case CSSAtRuleID::kCSSAtRuleBottomLeftCorner:
    case CSSAtRuleID::kCSSAtRuleBottomLeft:
    case CSSAtRuleID::kCSSAtRuleBottomCenter:
    case CSSAtRuleID::kCSSAtRuleBottomRight:
    case CSSAtRuleID::kCSSAtRuleBottomRightCorner:
    case CSSAtRuleID::kCSSAtRuleLeftTop:
    case CSSAtRuleID::kCSSAtRuleLeftMiddle:
    case CSSAtRuleID::kCSSAtRuleLeftBottom:
    case CSSAtRuleID::kCSSAtRuleRightTop:
    case CSSAtRuleID::kCSSAtRuleRightMiddle:
    case CSSAtRuleID::kCSSAtRuleRightBottom:
      return ConsumePageMarginRule(id, stream);
    case CSSAtRuleID::kCSSAtRuleCustomMedia:
      return ConsumeCustomMediaRule(stream);
    case CSSAtRuleID::kCSSAtRuleInvalid:
    case CSSAtRuleID::kCount:
      ConsumeErroneousAtRule(stream, id);
      return nullptr;  // Parse error, unrecognised or not-allowed at-rule
  }
}

StyleRuleBase* CSSParserImpl::ConsumeQualifiedRule(
    CSSParserTokenStream& stream,
    AllowedRules allowed_rules,
    CSSNestingType nesting_type,
    StyleRule* parent_rule_for_nesting) {
  // TODO(andruud): This function assumes 'nested=false', even though
  // a CSSNestingType and parent rule is provided. This means error recovery
  // always works as if non-nested, which is fragile.

  if (allowed_rules.Has(QualifiedRuleType::kStyle)) {
    bool invalid_rule_error_ignored = false;  // Only relevant when nested.
    return ConsumeStyleRule(stream, nesting_type, parent_rule_for_nesting,
                            /* nested */ false, invalid_rule_error_ignored);
  }

  if (allowed_rules.Has(QualifiedRuleType::kKeyframe)) {
    stream.EnsureLookAhead();
    const wtf_size_t prelude_offset_start = stream.LookAheadOffset();
    std::unique_ptr<Vector<KeyframeOffset>> key_list =
        ConsumeKeyframeKeyList(context_, stream);
    stream.ConsumeWhitespace();
    const RangeOffset prelude_offset(prelude_offset_start,
                                     stream.LookAheadOffset());

    if (stream.Peek().GetType() != kLeftBraceToken) {
      key_list = nullptr;  // Parse error, junk after prelude
      stream.SkipUntilPeekedTypeIs<kLeftBraceToken>();
    }
    if (stream.AtEnd()) {
      return nullptr;  // Parse error, EOF instead of qualified rule block
    }

    CSSParserTokenStream::BlockGuard guard(stream);
    return ConsumeKeyframeStyleRule(std::move(key_list), prelude_offset,
                                    stream);
  }

  // We still consume a qualified rule per css-syntax even when no rule
  // is allowed. This "error recovery" allows ConsumeRuleList to use
  // this function as the default branch.
  //
  // https://drafts.csswg.org/css-syntax/#consume-qualified-rule

  // Discard prelude and block.
  stream.SkipUntilPeekedTypeIs<kLeftBraceToken>();
  if (stream.Peek().GetType() == kLeftBraceToken) {
    CSSParserTokenStream::BlockGuard guard(stream);
  }

  return nullptr;
}

StyleRulePageMargin* CSSParserImpl::ConsumePageMarginRule(
    CSSAtRuleID rule_id,
    CSSParserTokenStream& stream) {
  wtf_size_t header_start = stream.LookAheadOffset();
  // NOTE: @page-margin prelude should be empty.
  if (!ConsumeEndOfPreludeForAtRuleWithBlock(stream, rule_id)) {
    return nullptr;
  }
  wtf_size_t header_end = stream.LookAheadOffset();

  CSSParserTokenStream::BlockGuard guard(stream);

  if (observer_) {
    observer_->StartRuleHeader(StyleRule::kPageMargin, header_start);
    observer_->EndRuleHeader(header_end);
    observer_->StartRuleBody(stream.Offset());
  }

  ConsumeBlockContents(stream, StyleRule::kPageMargin, CSSNestingType::kNone,
                       /*parent_rule_for_nesting=*/nullptr,
                       /*nested_declarations_start_index=*/kNotFound,
                       /*child_rules=*/nullptr);

  if (observer_) {
    observer_->EndRuleBody(stream.LookAheadOffset());
  }

  return MakeGarbageCollected<StyleRulePageMargin>(
      rule_id, CreateCSSPropertyValueSet(parsed_properties_, context_->Mode(),
                                         context_->GetDocument()));
}

StyleRuleCharset* CSSParserImpl::ConsumeCharsetRule(
    CSSParserTokenStream& stream) {
  const CSSParserToken& string = stream.Peek();
  if (string.GetType() != kStringToken || !stream.AtEnd()) {
    // Parse error, expected a single string.
    ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleCharset);
    return nullptr;
  }
  stream.ConsumeIncludingWhitespace();
  if (!ConsumeEndOfPreludeForAtRuleWithoutBlock(
          stream, CSSAtRuleID::kCSSAtRuleCharset)) {
    return nullptr;
  }

  return MakeGarbageCollected<StyleRuleCharset>();
}

StyleRuleImport* CSSParserImpl::ConsumeImportRule(
    const AtomicString& uri,
    CSSParserTokenStream& stream,
    const CSSUrlRequestModifiers& modifiers) {
  wtf_size_t prelude_offset_start = stream.LookAheadOffset();

  if (uri.IsNull()) {
    // Parse error, expected string or URI.
    ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleImport);
    return nullptr;
  }

  StyleRuleBase::LayerName layer;
  if (stream.Peek().GetType() == kIdentToken &&
      stream.Peek().Id() == CSSValueID::kLayer) {
    stream.ConsumeIncludingWhitespace();
    layer = StyleRuleBase::LayerName({g_empty_atom});
  } else if (stream.Peek().GetType() == kFunctionToken &&
             stream.Peek().FunctionId() == CSSValueID::kLayer) {
    CSSParserTokenStream::RestoringBlockGuard guard(stream);
    stream.ConsumeWhitespace();
    StyleRuleBase::LayerName name = ConsumeCascadeLayerName(stream);
    if (name.size() && stream.AtEnd()) {
      layer = std::move(name);
      guard.Release();
    } else {
      // Invalid layer() function can still be parsed as <general-enclosed>
    }
  }
  if (layer.size()) {
    context_->Count(WebFeature::kCSSCascadeLayers);
  }

  stream.ConsumeWhitespace();

  // https://drafts.csswg.org/css-cascade-5/#at-import
  //
  // <import-conditions> =
  //     [ supports([ <supports-condition> | <declaration> ]) ]?
  //     <media-query-list>?
  StringView supports_string = g_null_atom;
  CSSSupportsParser::Result supported = CSSSupportsParser::Result::kSupported;
  if (RuntimeEnabledFeatures::CSSSupportsForImportRulesEnabled() &&
      stream.Peek().GetType() == kFunctionToken &&
      stream.Peek().FunctionId() == CSSValueID::kSupports) {
    {
      CSSParserTokenStream::BlockGuard guard(stream);
      stream.ConsumeWhitespace();
      wtf_size_t supports_offset_start = stream.Offset();

      // First, try parsing as <declaration>.
      CSSParserTokenStream::State savepoint = stream.Save();
      if (stream.Peek().GetType() == kIdentToken &&
          CSSParserImpl::ConsumeSupportsDeclaration(stream)) {
        supported = CSSSupportsParser::Result::kSupported;
      } else {
        // Rewind and try parsing as <supports-condition>.
        stream.Restore(savepoint);
        supported = CSSSupportsParser::ConsumeSupportsCondition(stream, *this);
      }
      wtf_size_t supports_offset_end = stream.Offset();
      supports_string = stream.StringRangeAt(
          supports_offset_start, supports_offset_end - supports_offset_start);
    }
    if (supported == CSSSupportsParser::Result::kParseFailure) {
      ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleImport);
      return nullptr;
    }
  }
  stream.ConsumeWhitespace();

  const StyleScope* style_scope = nullptr;
  if (RuntimeEnabledFeatures::CSSScopeImportEnabled() &&
      stream.Peek().FunctionId() == CSSValueID::kScope) {
    {
      CSSParserTokenStream::RestoringBlockGuard guard(stream);
      stream.ConsumeWhitespace();
      style_scope = StyleScope::Consume(stream, context_, CSSNestingType::kNone,
                                        /*parent_rule_for_nesting=*/nullptr,
                                        style_sheet_);
      if (!guard.Release()) {
        style_scope = nullptr;
      }
    }
  }
  stream.ConsumeWhitespace();

  // Parse the rest of the prelude as a media query.
  // TODO(sesse): When the media query parser becomes streaming,
  // we can just parse media queries here instead.
  wtf_size_t media_query_offset_start = stream.Offset();
  stream.SkipUntilPeekedTypeIs<kLeftBraceToken, kSemicolonToken>();
  wtf_size_t prelude_offset_end = stream.LookAheadOffset();
  StringView media_query_string = stream.StringRangeAt(
      media_query_offset_start, prelude_offset_end - media_query_offset_start);

  MediaQuerySet* media_query_set = MediaQueryParser::ParseMediaQuerySet(
      media_query_string.ToString(), context_->GetExecutionContext());

  if (!ConsumeEndOfPreludeForAtRuleWithoutBlock(
          stream, CSSAtRuleID::kCSSAtRuleImport)) {
    return nullptr;
  }

  if (observer_) {
    observer_->StartRuleHeader(StyleRule::kImport, prelude_offset_start);
    observer_->EndRuleHeader(prelude_offset_end);
    observer_->StartRuleBody(prelude_offset_end);
    observer_->EndRuleBody(prelude_offset_end);
  }

  return MakeGarbageCollected<StyleRuleImport>(
      uri, std::move(layer), style_scope,
      supported == CSSSupportsParser::Result::kSupported,
      supports_string.ToString(), media_query_set,
      context_->IsOriginClean() ? OriginClean::kTrue : OriginClean::kFalse,
      modifiers);
}

StyleRuleNamespace* CSSParserImpl::ConsumeNamespaceRule(
    CSSParserTokenStream& stream) {
  AtomicString namespace_prefix;
  if (stream.Peek().GetType() == kIdentToken) {
    namespace_prefix =
        stream.ConsumeIncludingWhitespace().Value().ToAtomicString();
  }

  AtomicString uri(
      ConsumeStringOrURI(stream, *context_, /*modifiers=*/std::nullopt));
  if (uri.IsNull()) {
    // Parse error, expected string or URI.
    ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleNamespace);
    return nullptr;
  }
  if (!ConsumeEndOfPreludeForAtRuleWithoutBlock(
          stream, CSSAtRuleID::kCSSAtRuleNamespace)) {
    return nullptr;
  }

  return MakeGarbageCollected<StyleRuleNamespace>(namespace_prefix, uri);
}

namespace {

// Returns a :where(:scope) selector.
//
// Nested declaration rules within @scope behave as :where(:scope) rules.
//
// https://github.com/w3c/csswg-drafts/issues/10431
HeapVector<CSSSelector> WhereScopeSelector() {
  HeapVector<CSSSelector> selectors;

  CSSSelector inner[1] = {
      CSSSelector(AtomicString("scope"), /* implicit */ false)};
  inner[0].SetLastInComplexSelector(true);
  inner[0].SetLastInSelectorList(true);
  CSSSelectorList* inner_list =
      CSSSelectorList::AdoptSelectorVector(base::span<CSSSelector>(inner));

  CSSSelector where;
  where.SetWhere(inner_list);
  where.SetScopeContaining(true);
  selectors.push_back(where);

  selectors.back().SetLastInComplexSelector(true);
  selectors.back().SetLastInSelectorList(true);

  return selectors;
}

// https://drafts.csswg.org/css-nesting-1/#nested-declarations-rule
StyleRuleNestedDeclarations* CreateNestedDeclarationsRule(
    CSSNestingType nesting_type,
    const CSSParserContext& context,
    HeapVector<CSSSelector> selectors,
    HeapVector<CSSPropertyValue, 64>& declarations) {
  return MakeGarbageCollected<StyleRuleNestedDeclarations>(
      nesting_type,
      StyleRule::Create(selectors,
                        CreateCSSPropertyValueSet(declarations, context.Mode(),
                                                  context.GetDocument())));
}

}  // namespace

StyleRuleBase* CSSParserImpl::CreateDeclarationsRule(
    CSSNestingType nesting_type,
    const CSSSelector* selector_list,
    wtf_size_t start_index) {
  DCHECK(selector_list || (nesting_type != CSSNestingType::kNesting));

  // Create a nested declarations rule containing all declarations from
  // start_index to the end.
  HeapVector<CSSPropertyValue, 64> declarations(
      base::span(parsed_properties_).subspan(start_index));

  // Create the selector for StyleRuleNestedDeclarations's inner StyleRule.

  switch (nesting_type) {
    case CSSNestingType::kNone:
      break;
    case CSSNestingType::kNesting:
      // For regular nesting, the nested declarations rule should match
      // exactly what the parent rule matches, with top-level specificity
      // behavior. This means the selector list is copied rather than just
      // being referenced with '&'.
      return blink::CreateNestedDeclarationsRule(
          nesting_type, *context_,
          /*selectors=*/CSSSelectorList::Copy(selector_list), declarations);
    case CSSNestingType::kScope:
      // For direct nesting within @scope
      // (e.g. .foo { @scope (...) { color:green } }),
      // the nested declarations rule should match like a :where(:scope) rule.
      //
      // https://github.com/w3c/csswg-drafts/issues/10431
      return blink::CreateNestedDeclarationsRule(
          nesting_type, *context_,
          /*selectors=*/WhereScopeSelector(), declarations);
    case CSSNestingType::kFunction:
    case CSSNestingType::kMixin:
      // For descriptors within @function or @mixin, e.g.:
      //
      //  @function --x() {
      //    --local: 1px;
      //    result: var(--local);
      //  }
      //
      return MakeGarbageCollected<StyleRuleFunctionDeclarations>(
          *CreateCSSPropertyValueSet(declarations, kCSSFunctionDescriptorsMode,
                                     context_->GetDocument()));
  }

  NOTREACHED();
}

void CSSParserImpl::EmitDeclarationsRuleIfNeeded(
    StyleRule::RuleType rule_type,
    CSSNestingType nesting_type,
    StyleRule* parent_rule_for_nesting,
    wtf_size_t start_index,
    HeapVector<Member<StyleRuleBase>, 4>& child_rules) {
  if (rule_type == StyleRule::kPage) {
    // @page does not keep interleaved declarations "in place" by means of
    // CSSNestedDeclarations; they are effectively shifted to the top instead.
    return;
  }
  if (start_index == kNotFound) {
    return;
  }
  // The spec only allows creating non-empty rules, however, the inspector needs
  // empty rules to appear as well. This has no effect on the styles seen by
  // the page (the styles parsed with an `observer_` are for local use in the
  // inspector only).
  const bool emit_empty_rule = observer_;
  if (start_index >= parsed_properties_.size() && !emit_empty_rule) {
    return;
  }

  StyleRuleBase* nested_declarations_rule = CreateDeclarationsRule(
      nesting_type,
      parent_rule_for_nesting ? parent_rule_for_nesting->FirstSelector()
                              : nullptr,
      start_index);
  DCHECK(nested_declarations_rule);
  child_rules.push_back(nested_declarations_rule);

  if (observer_) {
    observer_->ObserveNestedDeclarations(
        /* insert_rule_index */ child_rules.size() - 1);
  }

  // The declarations held by the nested declarations rule
  // should not *also* appear in the main style declarations of the parent rule.
  parsed_properties_.resize(start_index);
}

StyleRuleMedia* CSSParserImpl::ConsumeMediaRule(
    CSSParserTokenStream& stream,
    CSSNestingType nesting_type,
    StyleRule* parent_rule_for_nesting) {
  // Consume the prelude.

  // First just get the string for the prelude to see if we've got a cached
  // version of this. (This is mainly to save memory in certain page with
  // lots of duplicate media queries.)
  CSSParserTokenStream::State savepoint = stream.Save();
  wtf_size_t prelude_offset_start = stream.LookAheadOffset();
  stream.SkipUntilPeekedTypeIs<kLeftBraceToken, kSemicolonToken>();
  wtf_size_t prelude_offset_end = stream.LookAheadOffset();

  String prelude_string =
      stream
          .StringRangeAt(prelude_offset_start,
                         prelude_offset_end - prelude_offset_start)
          .ToString();
  const MediaQuerySet* media;
  Member<const MediaQuerySet>& cached_media =
      media_query_cache_.insert(prelude_string, nullptr).stored_value->value;
  if (cached_media) {
    media = cached_media.Get();
  } else {
    // Not in the cache, so we'll have to rewind and actually parse it.
    // Note that the media query set grammar doesn't really have an idea
    // of when the stream should end; if it sees something it doesn't
    // understand (which includes a left brace), it will just forward to
    // the next comma, skipping over the entire stylesheet until the end.
    // The grammar is generally written in the understanding that the prelude
    // is extracted as a string and only then parsed, whereas we do fully
    // streaming prelude parsing. Thus, we need to set some boundaries
    // here ourselves to make sure we end when the prelude does; the alternative
    // would be to teach the media query set parser to stop there itself.
    stream.Restore(savepoint);
    CSSParserTokenStream::Boundary boundary(stream, kLeftBraceToken);
    CSSParserTokenStream::Boundary boundary2(stream, kSemicolonToken);
    media = MediaQueryParser::ParseMediaQuerySet(
        stream, context_->GetExecutionContext());
  }
  DCHECK(media);

  if (!ConsumeEndOfPreludeForAtRuleWithBlock(stream,
                                             CSSAtRuleID::kCSSAtRuleMedia)) {
    return nullptr;
  }

  cached_media = media;

  // Consume the actual block.
  CSSParserTokenStream::BlockGuard guard(stream);

  if (observer_) {
    observer_->StartRuleHeader(StyleRule::kMedia, prelude_offset_start);
    observer_->EndRuleHeader(prelude_offset_end);
    observer_->StartRuleBody(stream.Offset());
  }

  if (style_sheet_) {
    style_sheet_->SetHasMediaQueries();
  }

  HeapVector<Member<StyleRuleBase>, 4> rules;
  ConsumeRuleListOrNestedDeclarationList(stream, nesting_type,
                                         parent_rule_for_nesting, &rules);

  if (observer_) {
    observer_->EndRuleBody(stream.Offset());
  }

  // NOTE: There will be a copy of rules here, to deal with the different inline
  // size.
  return MakeGarbageCollected<StyleRuleMedia>(media, std::move(rules));
}

StyleRuleSupports* CSSParserImpl::ConsumeSupportsRule(
    CSSParserTokenStream& stream,
    CSSNestingType nesting_type,
    StyleRule* parent_rule_for_nesting) {
  wtf_size_t prelude_offset_start = stream.LookAheadOffset();
  CSSSupportsParser::Result supported =
      CSSSupportsParser::ConsumeSupportsCondition(stream, *this);
  if (supported == CSSSupportsParser::Result::kParseFailure) {
    ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleSupports);
    return nullptr;
  }
  wtf_size_t prelude_offset_end = stream.LookAheadOffset();
  if (!ConsumeEndOfPreludeForAtRuleWithBlock(stream,
                                             CSSAtRuleID::kCSSAtRuleSupports)) {
    return nullptr;
  }
  CSSParserTokenStream::BlockGuard guard(stream);

  if (observer_) {
    observer_->StartRuleHeader(StyleRule::kSupports, prelude_offset_start);
    observer_->EndRuleHeader(prelude_offset_end);
    observer_->StartRuleBody(stream.Offset());
  }

  const auto prelude_serialized =
      stream
          .StringRangeAt(prelude_offset_start,
                         prelude_offset_end - prelude_offset_start)
          .ToString()
          .SimplifyWhiteSpace();

  HeapVector<Member<StyleRuleBase>, 4> rules;
  ConsumeRuleListOrNestedDeclarationList(stream, nesting_type,
                                         parent_rule_for_nesting, &rules);

  if (observer_) {
    observer_->EndRuleBody(stream.Offset());
  }

  // NOTE: There will be a copy of rules here, to deal with the different inline
  // size.
  return MakeGarbageCollected<StyleRuleSupports>(
      prelude_serialized, supported == CSSSupportsParser::Result::kSupported,
      std::move(rules));
}

StyleRuleStartingStyle* CSSParserImpl::ConsumeStartingStyleRule(
    CSSParserTokenStream& stream,
    CSSNestingType nesting_type,
    StyleRule* parent_rule_for_nesting) {
  // NOTE: @starting-style prelude should be empty.
  wtf_size_t prelude_offset_start = stream.LookAheadOffset();
  wtf_size_t prelude_offset_end = stream.LookAheadOffset();
  if (!ConsumeEndOfPreludeForAtRuleWithBlock(
          stream, CSSAtRuleID::kCSSAtRuleStartingStyle)) {
    return nullptr;
  }
  CSSParserTokenStream::BlockGuard guard(stream);

  if (observer_) {
    observer_->StartRuleHeader(StyleRule::kStartingStyle, prelude_offset_start);
    observer_->EndRuleHeader(prelude_offset_end);
    observer_->StartRuleBody(stream.Offset());
  }

  HeapVector<Member<StyleRuleBase>, 4> rules;
  ConsumeRuleListOrNestedDeclarationList(stream, nesting_type,
                                         parent_rule_for_nesting, &rules);

  if (observer_) {
    observer_->EndRuleBody(stream.Offset());
  }

  // NOTE: There will be a copy of rules here, to deal with the different inline
  // size.
  return MakeGarbageCollected<StyleRuleStartingStyle>(std::move(rules));
}

StyleRuleFontFace* CSSParserImpl::ConsumeFontFaceRule(
    CSSParserTokenStream& stream) {
  // Consume the prelude.
  // NOTE: @font-face prelude should be empty.
  wtf_size_t prelude_offset_start = stream.LookAheadOffset();
  wtf_size_t prelude_offset_end = stream.LookAheadOffset();
  if (!ConsumeEndOfPreludeForAtRuleWithBlock(stream,
                                             CSSAtRuleID::kCSSAtRuleFontFace)) {
    return nullptr;
  }

  // Consume the actual block.
  CSSParserTokenStream::BlockGuard guard(stream);
  if (observer_) {
    observer_->StartRuleHeader(StyleRule::kFontFace, prelude_offset_start);
    observer_->EndRuleHeader(prelude_offset_end);
    observer_->StartRuleBody(stream.Offset());
  }

  if (style_sheet_) {
    style_sheet_->SetHasFontFaceRule();
  }

  ConsumeBlockContents(stream, StyleRule::kFontFace, CSSNestingType::kNone,
                       /*parent_rule_for_nesting=*/nullptr,
                       /*nested_declarations_start_index=*/kNotFound,
                       /*child_rules=*/nullptr);

  if (observer_) {
    observer_->EndRuleBody(stream.LookAheadOffset());
  }

  return MakeGarbageCollected<StyleRuleFontFace>(CreateCSSPropertyValueSet(
      parsed_properties_, kCSSFontFaceRuleMode, context_->GetDocument()));
}

StyleRuleKeyframes* CSSParserImpl::ConsumeKeyframesRule(
    bool webkit_prefixed,
    CSSParserTokenStream& stream) {
  // Parse the prelude, expecting a single non-whitespace token.
  wtf_size_t prelude_offset_start = stream.LookAheadOffset();
  const CSSParserToken& name_token = stream.Peek();
  String name;
  if (name_token.GetType() == kIdentToken &&
      css_parsing_utils::IsValidIdentAnimationName(
          name_token.Value().ToAtomicString())) {
    name = name_token.Value().ToString();
  } else if (name_token.GetType() == kStringToken &&
             !name_token.Value().empty()) {
    context_->Count(WebFeature::kOBSOLETE_QuotedKeyframesRule);
    name = name_token.Value().ToString();
  } else {
    ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleKeyframes);
    return nullptr;  // Parse error; expected ident token in @keyframes header
  }
  stream.ConsumeIncludingWhitespace();
  wtf_size_t prelude_offset_end = stream.LookAheadOffset();
  if (!ConsumeEndOfPreludeForAtRuleWithBlock(
          stream, CSSAtRuleID::kCSSAtRuleKeyframes)) {
    return nullptr;
  }

  // Parse the body.
  CSSParserTokenStream::BlockGuard guard(stream);

  if (observer_) {
    observer_->StartRuleHeader(StyleRule::kKeyframes, prelude_offset_start);
    observer_->EndRuleHeader(prelude_offset_end);
    observer_->StartRuleBody(stream.Offset());
  }

  auto* keyframe_rule = MakeGarbageCollected<StyleRuleKeyframes>();
  ConsumeRuleList(
      stream, kKeyframeRules, /*allow_cdo_cdc_tokens=*/false,
      CSSNestingType::kNone,
      /*parent_rule_for_nesting=*/nullptr,
      [keyframe_rule](StyleRuleBase* keyframe, wtf_size_t) {
        keyframe_rule->ParserAppendKeyframe(To<StyleRuleKeyframe>(keyframe));
      });
  keyframe_rule->SetName(name);
  keyframe_rule->SetVendorPrefixed(webkit_prefixed);

  if (observer_) {
    observer_->EndRuleBody(stream.Offset());
  }

  return keyframe_rule;
}

StyleRuleFontFeature* CSSParserImpl::ConsumeFontFeatureRuleBlock(
    StyleRuleFontFeature::FeatureType feature_type,
    CSSParserTokenStream& stream) {
  wtf_size_t max_allowed_values = 1;
  if (feature_type == StyleRuleFontFeature::FeatureType::kCharacterVariant) {
    max_allowed_values = 2;
  }
  if (feature_type == StyleRuleFontFeature::FeatureType::kStyleset) {
    max_allowed_values = std::numeric_limits<wtf_size_t>::max();
  }
  auto* font_feature_rule =
      MakeGarbageCollected<StyleRuleFontFeature>(feature_type);

  while (!stream.AtEnd()) {
    const CSSParserToken& alias_token = stream.Peek();

    wtf_size_t decl_offset_start = stream.Offset();
    if (alias_token.GetType() != kIdentToken) {
      return nullptr;
    }
    AtomicString alias =
        stream.ConsumeIncludingWhitespace().Value().ToAtomicString();

    const CSSParserToken& colon_token = stream.Peek();

    if (colon_token.GetType() != kColonToken) {
      return nullptr;
    }

    stream.UncheckedConsume();
    stream.ConsumeWhitespace();

    CSSValueList* numbers = CSSValueList::CreateSpaceSeparated();

    stream.ConsumeWhitespace();

    do {
      if (numbers->length() == max_allowed_values) {
        return nullptr;
      }
      CSSParserLocalContext local_context =
          CSSParserLocalContext::CreateWithoutPropertyForAtRules();
      CSSPrimitiveValue* parsed_number =
          css_parsing_utils::ConsumeIntegerOrNumberCalc(
              stream, *context_, local_context,
              CSSPrimitiveValue::ValueRange::kNonNegativeInteger);
      if (!parsed_number) {
        return nullptr;
      }
      numbers->Append(*parsed_number);
    } while (stream.Peek().GetType() != kSemicolonToken && !stream.AtEnd());

    if (!stream.AtEnd()) {
      stream.ConsumeIncludingWhitespace();  // kSemicolonToken
    }

    if (!numbers->length()) {
      return nullptr;
    }

    Vector<uint32_t> parsed_numbers;
    for (auto value : *numbers) {
      const CSSPrimitiveValue* number_value =
          DynamicTo<CSSPrimitiveValue>(*value);
      if (!number_value) {
        return nullptr;
      }
      std::optional<double> number = number_value->GetValueIfKnown();
      if (!number.has_value()) {
        return nullptr;
      }
      parsed_numbers.push_back(ClampTo<int>(number.value()));
    }

    if (observer_) {
      observer_->ObserveProperty(decl_offset_start, stream.LookAheadOffset(),
                                 /*is_important=*/false, /*is_parsed=*/true);
    }
    const CSSParserToken& expected_semicolon = stream.Peek();
    if (expected_semicolon.GetType() == kSemicolonToken) {
      stream.UncheckedConsume();
    }
    stream.ConsumeWhitespace();

    font_feature_rule->UpdateAlias(alias, std::move(parsed_numbers));
  }

  return font_feature_rule;
}

StyleRuleFontFeature* CSSParserImpl::ConsumeFontFeatureRule(
    CSSAtRuleID rule_id,
    CSSParserTokenStream& stream) {
  wtf_size_t prelude_offset_start = stream.LookAheadOffset();

  std::optional<StyleRuleFontFeature::FeatureType> feature_type =
      ToStyleRuleFontFeatureType(rule_id);
  if (!feature_type) {
    return nullptr;
  }

  stream.ConsumeWhitespace();
  wtf_size_t prelude_offset_end = stream.LookAheadOffset();

  if (stream.Peek().GetType() != kLeftBraceToken) {
    return nullptr;
  }

  CSSParserTokenStream::BlockGuard guard(stream);

  if (observer_) {
    observer_->StartRuleHeader(StyleRule::kFontFeature, prelude_offset_start);
    observer_->ObserveFontFeatureType(*feature_type);
    observer_->EndRuleHeader(prelude_offset_end);
    observer_->StartRuleBody(stream.Offset());
  }

  stream.ConsumeWhitespace();
  auto* font_feature_rule = ConsumeFontFeatureRuleBlock(*feature_type, stream);

  if (observer_) {
    observer_->EndRuleBody(stream.Offset());
    if (!font_feature_rule) {
      observer_->ObserveErroneousAtRule(prelude_offset_start, rule_id, {});
    }
  }

  return font_feature_rule;
}

StyleRuleFontFeatureValues* CSSParserImpl::ConsumeFontFeatureValuesRule(
    CSSParserTokenStream& stream) {
  // Parse the prelude.
  wtf_size_t prelude_offset_start = stream.LookAheadOffset();
  CSSValueList* family_list =
      css_parsing_utils::ConsumeFontFamily(stream, *context_);
  if (!family_list || !family_list->length()) {
    ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleFontFeatureValues);
    return nullptr;
  }

  Vector<AtomicString> families;
  families.ReserveInitialCapacity(family_list->length());
  for (const auto& family_entry : *family_list) {
    const CSSFontFamilyValue* family_value =
        DynamicTo<CSSFontFamilyValue>(*family_entry);
    if (!family_value) {
      ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleFontFeatureValues);
      return nullptr;
    }
    families.push_back(family_value->Value());
  }
  wtf_size_t prelude_offset_end = stream.LookAheadOffset();
  if (!ConsumeEndOfPreludeForAtRuleWithBlock(
          stream, CSSAtRuleID::kCSSAtRuleFontFeatureValues)) {
    return nullptr;
  }
  CSSParserTokenStream::BlockGuard guard(stream);

  if (observer_) {
    observer_->StartRuleHeader(StyleRule::kFontFeatureValues,
                               prelude_offset_start);
    observer_->EndRuleHeader(prelude_offset_end);
    observer_->StartRuleBody(stream.Offset());
  }

  // Parse the actual block.

  // The nesting logic for parsing @font-feature-values looks as follow:
  // 1) ConsumeRuleList, calls ConsumeAtRule, and in turn ConsumeAtRuleContents
  // 2) ConsumeAtRuleContents uses new ids for inner at-rules, for swash,
  // styleset etc.
  // 3) ConsumeFeatureRule (with type) consumes the inner mappings from aliases
  // to number lists.

  FontFeatureAliases stylistic;
  FontFeatureAliases styleset;
  FontFeatureAliases character_variant;
  FontFeatureAliases swash;
  FontFeatureAliases ornaments;
  FontFeatureAliases annotation;

  HeapVector<Member<StyleRuleFontFeature>> feature_rules;
  bool had_valid_rules = false;
  // ConsumeRuleList returns true only if the first rule is true, but we need to
  // be more generous with the internals of what's inside a font feature value
  // declaration, e.g. inside a @stylsitic, @styleset, etc.
  if (ConsumeRuleList(
          stream, kFontFeatureRules, /*allow_cdo_cdc_tokens=*/false,
          CSSNestingType::kNone,
          /*parent_rule_for_nesting=*/nullptr,
          [&feature_rules, &had_valid_rules](StyleRuleBase* rule, wtf_size_t) {
            if (rule) {
              had_valid_rules = true;
            }
            feature_rules.push_back(To<StyleRuleFontFeature>(rule));
          }) ||
      had_valid_rules) {
    // https://drafts.csswg.org/css-fonts-4/#font-feature-values-syntax
    // "Specifying the same <font-feature-value-type> more than once is valid;
    // their contents are cascaded together."
    for (auto& feature_rule : feature_rules) {
      switch (feature_rule->GetFeatureType()) {
        case StyleRuleFontFeature::FeatureType::kStylistic:
          feature_rule->OverrideAliasesIn(stylistic);
          break;
        case StyleRuleFontFeature::FeatureType::kStyleset:
          feature_rule->OverrideAliasesIn(styleset);
          break;
        case StyleRuleFontFeature::FeatureType::kCharacterVariant:
          feature_rule->OverrideAliasesIn(character_variant);
          break;
        case StyleRuleFontFeature::FeatureType::kSwash:
          feature_rule->OverrideAliasesIn(swash);
          break;
        case StyleRuleFontFeature::FeatureType::kOrnaments:
          feature_rule->OverrideAliasesIn(ornaments);
          break;
        case StyleRuleFontFeature::FeatureType::kAnnotation:
          feature_rule->OverrideAliasesIn(annotation);
          break;
      }
    }
  }

  auto* feature_values_rule = MakeGarbageCollected<StyleRuleFontFeatureValues>(
      std::move(families), stylistic, styleset, character_variant, swash,
      ornaments, annotation);

  if (observer_) {
    observer_->EndRuleBody(stream.Offset());
  }

  return feature_values_rule;
}

// Parse an @page rule, with contents.
StyleRulePage* CSSParserImpl::ConsumePageRule(CSSParserTokenStream& stream) {
  // Parse the prelude.
  wtf_size_t prelude_offset_start = stream.LookAheadOffset();
  CSSSelectorList* selector_list =
      ParsePageSelector(stream, style_sheet_, *context_);
  if (!selector_list || !selector_list->IsValid()) {
    ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRulePage);
    return nullptr;  // Parse error, invalid @page selector
  }
  wtf_size_t prelude_offset_end = stream.LookAheadOffset();
  if (!ConsumeEndOfPreludeForAtRuleWithBlock(stream,
                                             CSSAtRuleID::kCSSAtRulePage)) {
    return nullptr;
  }

  // Parse the actual block.
  CSSParserTokenStream::BlockGuard guard(stream);

  if (observer_) {
    observer_->StartRuleHeader(StyleRule::kPage, prelude_offset_start);
    observer_->EndRuleHeader(prelude_offset_end);
    observer_->StartRuleBody(stream.Offset());
  }

  HeapVector<Member<StyleRuleBase>, 4> child_rules;
  ConsumeBlockContents(stream, StyleRule::kPage, CSSNestingType::kNone,
                       /*parent_rule_for_nesting=*/nullptr,
                       /*nested_declarations_start_index=*/kNotFound,
                       &child_rules);

  if (observer_) {
    observer_->EndRuleBody(stream.LookAheadOffset());
  }

  return MakeGarbageCollected<StyleRulePage>(
      selector_list,
      CreateCSSPropertyValueSet(parsed_properties_, context_->Mode(),
                                context_->GetDocument()),
      child_rules);
}

StyleRuleProperty* CSSParserImpl::ConsumePropertyRule(
    CSSParserTokenStream& stream) {
  // Parse the prelude.
  wtf_size_t prelude_offset_start = stream.LookAheadOffset();
  const CSSParserToken& name_token = stream.Peek();
  if (!CSSVariableParser::IsValidVariableName(name_token)) {
    ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleProperty);
    return nullptr;
  }
  String name = name_token.Value().ToString();
  stream.ConsumeIncludingWhitespace();
  wtf_size_t prelude_offset_end = stream.LookAheadOffset();
  if (!ConsumeEndOfPreludeForAtRuleWithBlock(stream,
                                             CSSAtRuleID::kCSSAtRuleProperty)) {
    return nullptr;
  }

  // Parse the body.
  CSSParserTokenStream::BlockGuard guard(stream);

  if (observer_) {
    observer_->StartRuleHeader(StyleRule::kProperty, prelude_offset_start);
    observer_->EndRuleHeader(prelude_offset_end);
    observer_->StartRuleBody(stream.Offset());
  }

  ConsumeBlockContents(stream, StyleRule::kProperty, CSSNestingType::kNone,
                       /*parent_rule_for_nesting=*/nullptr,
                       /*nested_declarations_start_index=*/kNotFound,
                       /*child_rules=*/nullptr);

  if (observer_) {
    observer_->EndRuleBody(stream.LookAheadOffset());
  }

  StyleRuleProperty* rule = MakeGarbageCollected<StyleRuleProperty>(
      name, CreateCSSPropertyValueSet(parsed_properties_, kCSSPropertyRuleMode,
                                      context_->GetDocument()));

  std::optional<CSSSyntaxDefinition> syntax =
      PropertyRegistration::ConvertSyntax(rule->GetSyntax());
  std::optional<bool> inherits =
      PropertyRegistration::ConvertInherits(rule->Inherits());

  // Since random() might be element dependent, we should disallow random()
  // values inside initial value of registered custom properties. Use
  // CSSParserLocalContext with custom property name just to keep it consistent
  // in case we need it in the future.
  CSSParserLocalContext local_context(CSSPropertyName(AtomicString(name)),
                                      CSSPropertyID::kInvalid);
  std::optional<const CSSValue*> initial =
      syntax.has_value()
          ? PropertyRegistration::ConvertInitial(
                rule->GetInitialValue(), *syntax, *context_, local_context)
          : std::nullopt;

  bool invalid_rule =
      !syntax.has_value() || !inherits.has_value() || !initial.has_value();

  if (observer_ && invalid_rule) {
    Vector<CSSPropertyID, 2> failed_properties;
    if (!syntax.has_value()) {
      failed_properties.push_back(CSSPropertyID::kSyntax);
    }
    if (!inherits.has_value()) {
      failed_properties.push_back(CSSPropertyID::kInherits);
    }
    if (!initial.has_value() && syntax.has_value()) {
      failed_properties.push_back(CSSPropertyID::kInitialValue);
    }
    DCHECK(!failed_properties.empty());
    observer_->ObserveErroneousAtRule(prelude_offset_start,
                                      CSSAtRuleID::kCSSAtRuleProperty,
                                      failed_properties);
  }
  if (invalid_rule) {
    return nullptr;
  }
  return rule;
}

StyleRuleLocation* CSSParserImpl::ConsumeLocationRule(
    CSSParserTokenStream& stream) {
  // Parse the prelude.
  wtf_size_t prelude_offset_start = stream.LookAheadOffset();
  const CSSParserToken& name_token = stream.Peek();
  // <dashed-ident>
  AtomicString name;
  if (name_token.GetType() == kIdentToken) {
    name = name_token.Value().ToAtomicString();
    if (!name.starts_with("--")) {
      ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleLocation);
      return nullptr;
    }
  } else {
    ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleLocation);
    return nullptr;
  }
  stream.ConsumeIncludingWhitespace();
  wtf_size_t prelude_offset_end = stream.LookAheadOffset();
  if (!ConsumeEndOfPreludeForAtRuleWithBlock(stream,
                                             CSSAtRuleID::kCSSAtRuleLocation)) {
    return nullptr;
  }

  // Parse the actual block.
  CSSParserTokenStream::BlockGuard guard(stream);
  if (observer_) {
    observer_->StartRuleHeader(StyleRule::kLocation, prelude_offset_start);
    observer_->EndRuleHeader(prelude_offset_end);
    observer_->StartRuleBody(stream.Offset());
  }

  ConsumeBlockContents(stream, StyleRule::kLocation, CSSNestingType::kNone,
                       /*parent_rule_for_nesting=*/nullptr,
                       /*nested_declarations_start_index=*/kNotFound,
                       /*child_rules=*/nullptr);

  if (observer_) {
    observer_->EndRuleBody(stream.LookAheadOffset());
  }

  // TODO(crbug.com/436805487): Honor [ <pattern-descriptors> |
  // <init-descriptors> ] (it should either be a URLPattern, OR init
  // descriptors, not a combination).
  return MakeGarbageCollected<StyleRuleLocation>(
      name, CreateCSSPropertyValueSet(parsed_properties_, context_->Mode(),
                                      context_->GetDocument()));
}

StyleRuleNavigation* CSSParserImpl::ConsumeNavigationRule(
    CSSParserTokenStream& stream,
    CSSNestingType nesting_type,
    StyleRule* parent_rule_for_nesting) {
  // Parse the prelude.
  wtf_size_t header_start_offset = stream.LookAheadOffset();
  NavigationQuery* query = NavigationParser::ParseQuery(stream);
  if (!query) {
    ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleNavigation);
    return nullptr;
  }
  if (!ConsumeEndOfPreludeForAtRuleWithBlock(
          stream, CSSAtRuleID::kCSSAtRuleNavigation)) {
    return nullptr;
  }
  wtf_size_t header_end_offset = stream.LookAheadOffset();

  // Parse the actual block.
  CSSParserTokenStream::BlockGuard body_guard(stream);
  if (observer_) {
    observer_->StartRuleHeader(StyleRule::kNavigation, header_start_offset);
    observer_->EndRuleHeader(header_end_offset);
    observer_->StartRuleBody(stream.Offset());
  }

  HeapVector<Member<StyleRuleBase>, 4> rules;
  ConsumeRuleListOrNestedDeclarationList(stream, nesting_type,
                                         parent_rule_for_nesting, &rules);

  if (observer_) {
    observer_->EndRuleBody(stream.Offset());
  }

  return MakeGarbageCollected<StyleRuleNavigation>(query, std::move(rules));
}

StyleRuleCounterStyle* CSSParserImpl::ConsumeCounterStyleRule(
    CSSParserTokenStream& stream) {
  // Parse the prelude.
  wtf_size_t prelude_offset_start = stream.LookAheadOffset();
  AtomicString name = css_parsing_utils::ConsumeCounterStyleNameInPrelude(
      stream, *GetContext());
  if (!name) {
    ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleCounterStyle);
    return nullptr;
  }
  wtf_size_t prelude_offset_end = stream.LookAheadOffset();
  if (!ConsumeEndOfPreludeForAtRuleWithBlock(
          stream, CSSAtRuleID::kCSSAtRuleCounterStyle)) {
    return nullptr;
  }

  // Parse the actual block.
  CSSParserTokenStream::BlockGuard guard(stream);
  if (observer_) {
    observer_->StartRuleHeader(StyleRule::kCounterStyle, prelude_offset_start);
    observer_->EndRuleHeader(prelude_offset_end);
    observer_->StartRuleBody(stream.Offset());
  }

  ConsumeBlockContents(stream, StyleRule::kCounterStyle, CSSNestingType::kNone,
                       /*parent_rule_for_nesting=*/nullptr,
                       /*nested_declarations_start_index=*/kNotFound,
                       /*child_rules=*/nullptr);

  if (observer_) {
    observer_->EndRuleBody(stream.LookAheadOffset());
  }

  return MakeGarbageCollected<StyleRuleCounterStyle>(
      name,
      CreateCSSPropertyValueSet(parsed_properties_, kCSSCounterStyleRuleMode,
                                context_->GetDocument()));
}

StyleRuleFontPaletteValues* CSSParserImpl::ConsumeFontPaletteValuesRule(
    CSSParserTokenStream& stream) {
  // Parse the prelude.
  wtf_size_t prelude_offset_start = stream.LookAheadOffset();
  const CSSParserToken& name_token = stream.Peek();
  if (!css_parsing_utils::IsDashedIdent(name_token)) {
    ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleFontPaletteValues);
    return nullptr;
  }
  AtomicString name = name_token.Value().ToAtomicString();
  if (!name) {
    ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleFontPaletteValues);
    return nullptr;
  }
  stream.ConsumeIncludingWhitespace();
  wtf_size_t prelude_offset_end = stream.LookAheadOffset();
  if (!ConsumeEndOfPreludeForAtRuleWithBlock(
          stream, CSSAtRuleID::kCSSAtRuleFontPaletteValues)) {
    return nullptr;
  }

  // Parse the actual block.
  CSSParserTokenStream::BlockGuard guard(stream);
  if (observer_) {
    observer_->StartRuleHeader(StyleRule::kFontPaletteValues,
                               prelude_offset_start);
    observer_->EndRuleHeader(prelude_offset_end);
    observer_->StartRuleBody(stream.Offset());
  }

  ConsumeBlockContents(stream, StyleRule::kFontPaletteValues,
                       CSSNestingType::kNone,
                       /*parent_rule_for_nesting=*/nullptr,
                       /*nested_declarations_start_index=*/kNotFound,
                       /*child_rules=*/nullptr);

  if (observer_) {
    observer_->EndRuleBody(stream.LookAheadOffset());
  }

  return MakeGarbageCollected<StyleRuleFontPaletteValues>(
      name, CreateCSSPropertyValueSet(parsed_properties_,
                                      kCSSFontPaletteValuesRuleMode,
                                      context_->GetDocument()));
}

StyleRuleBase* CSSParserImpl::ConsumeScopeRule(
    CSSParserTokenStream& stream,
    CSSNestingType nesting_type,
    StyleRule* parent_rule_for_nesting) {
  // Parse the prelude.
  wtf_size_t prelude_offset_start = stream.LookAheadOffset();
  auto* style_scope = StyleScope::Consume(
      stream, context_, nesting_type, parent_rule_for_nesting, style_sheet_);
  wtf_size_t prelude_offset_end = stream.LookAheadOffset();
  if (!ConsumeEndOfPreludeForAtRuleWithBlock(stream,
                                             CSSAtRuleID::kCSSAtRuleScope)) {
    return nullptr;
  }
  if (!style_scope) {
    // The prelude was empty. This represents an implicit scope
    // rooted at the parent element of the stylesheet's owner node.
    style_scope = StyleScope::CreateImplicit();
  }

  if (observer_) {
    observer_->StartRuleHeader(StyleRule::kScope, prelude_offset_start);
    observer_->EndRuleHeader(prelude_offset_end);
    observer_->StartRuleBody(stream.Offset());
  }

  // Parse the actual block.
  CSSParserTokenStream::BlockGuard guard(stream);

  HeapVector<Member<StyleRuleBase>, 4> rules;
  ConsumeBlockContents(
      stream, StyleRule::kScope, CSSNestingType::kScope,
      /*parent_rule_for_nesting=*/style_scope->RuleForNesting(),
      /*nested_declarations_start_index=*/0, &rules);

  if (observer_) {
    observer_->EndRuleBody(stream.Offset());
  }

  return MakeGarbageCollected<StyleRuleScope>(*style_scope, std::move(rules));
}

StyleRuleViewTransition* CSSParserImpl::ConsumeViewTransitionRule(
    CSSParserTokenStream& stream) {
  // NOTE: @view-transition prelude should be empty.
  wtf_size_t prelude_offset_start = stream.LookAheadOffset();
  wtf_size_t prelude_offset_end = stream.LookAheadOffset();
  if (!ConsumeEndOfPreludeForAtRuleWithBlock(
          stream, CSSAtRuleID::kCSSAtRuleViewTransition)) {
    return nullptr;
  }

  CSSParserTokenStream::BlockGuard guard(stream);
  if (observer_) {
    observer_->StartRuleHeader(StyleRule::kViewTransition,
                               prelude_offset_start);
    observer_->EndRuleHeader(prelude_offset_end);
    observer_->StartRuleBody(stream.Offset());
  }
  ConsumeBlockContents(stream, StyleRule::kViewTransition,
                       CSSNestingType::kNone,
                       /*parent_rule_for_nesting=*/nullptr,
                       /*nested_declarations_start_index=*/kNotFound,
                       /*child_rules=*/nullptr);

  if (observer_) {
    observer_->EndRuleBody(stream.LookAheadOffset());
  }

  return MakeGarbageCollected<StyleRuleViewTransition>(
      *CreateCSSPropertyValueSet(parsed_properties_, context_->Mode(),
                                 context_->GetDocument()));
}

StyleRuleContainer* CSSParserImpl::ConsumeContainerRule(
    CSSParserTokenStream& stream,
    CSSNestingType nesting_type,
    StyleRule* parent_rule_for_nesting) {
  // Consume the prelude.
  wtf_size_t prelude_offset_start = stream.LookAheadOffset();

  const ContainerQuerySet* container_query_set =
      ContainerQueryParser::ParseContainerQuerySet(stream, *context_);
  if (!container_query_set) {
    ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleContainer);
    return nullptr;
  }

  wtf_size_t prelude_offset_end = stream.LookAheadOffset();

  if (!ConsumeEndOfPreludeForAtRuleWithBlock(
          stream, CSSAtRuleID::kCSSAtRuleContainer)) {
    return nullptr;
  }

  // Consume the actual block.
  CSSParserTokenStream::BlockGuard guard(stream);

  if (observer_) {
    observer_->StartRuleHeader(StyleRule::kContainer, prelude_offset_start);
    observer_->EndRuleHeader(prelude_offset_end);
    observer_->StartRuleBody(stream.Offset());
  }

  HeapVector<Member<StyleRuleBase>, 4> rules;
  ConsumeRuleListOrNestedDeclarationList(stream, nesting_type,
                                         parent_rule_for_nesting, &rules);

  if (observer_) {
    observer_->EndRuleBody(stream.Offset());
  }

  // NOTE: There will be a copy of rules here, to deal with the different inline
  // size.
  return MakeGarbageCollected<StyleRuleContainer>(*container_query_set,
                                                  std::move(rules));
}

StyleRuleBase* CSSParserImpl::ConsumeLayerRule(
    CSSParserTokenStream& stream,
    CSSNestingType nesting_type,
    StyleRule* parent_rule_for_nesting) {
  // Consume the prelude.
  wtf_size_t prelude_offset_start = stream.LookAheadOffset();

  Vector<StyleRuleBase::LayerName> names;
  while (!stream.AtEnd() && stream.Peek().GetType() != kLeftBraceToken &&
         stream.Peek().GetType() != kSemicolonToken) {
    if (names.size()) {
      if (!css_parsing_utils::ConsumeCommaIncludingWhitespace(stream)) {
        ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleLayer);
        return nullptr;
      }
    }
    StyleRuleBase::LayerName name = ConsumeCascadeLayerName(stream);
    if (!name.size()) {
      ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleLayer);
      return nullptr;
    }
    names.push_back(std::move(name));
  }

  // @layer statement rule without style declarations.
  if (stream.AtEnd() || stream.UncheckedPeek().GetType() == kSemicolonToken) {
    if (!names.size()) {
      ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleLayer);
      return nullptr;
    }

    if (nesting_type == CSSNestingType::kNesting) {
      // @layer statement rules are not group rules, and can therefore
      // not be nested.
      //
      // https://drafts.csswg.org/css-nesting-1/#nested-group-rules
      ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleLayer);
      return nullptr;
    }

    wtf_size_t prelude_offset_end = stream.LookAheadOffset();
    if (!ConsumeEndOfPreludeForAtRuleWithoutBlock(
            stream, CSSAtRuleID::kCSSAtRuleLayer)) {
      return nullptr;
    }

    if (observer_) {
      observer_->StartRuleHeader(StyleRule::kLayerStatement,
                                 prelude_offset_start);
      observer_->EndRuleHeader(prelude_offset_end);
      observer_->StartRuleBody(prelude_offset_end);
      observer_->EndRuleBody(prelude_offset_end);
    }

    return MakeGarbageCollected<StyleRuleLayerStatement>(std::move(names));
  }

  // @layer block rule with style declarations.
  StyleRuleBase::LayerName name;
  if (names.empty()) {
    name.push_back(g_empty_atom);
  } else if (names.size() > 1) {
    ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleLayer);
    return nullptr;
  } else {
    name = std::move(names[0]);
  }

  wtf_size_t prelude_offset_end = stream.LookAheadOffset();

  if (!ConsumeEndOfPreludeForAtRuleWithBlock(stream,
                                             CSSAtRuleID::kCSSAtRuleLayer)) {
    return nullptr;
  }

  // Consume the actual block.
  CSSParserTokenStream::BlockGuard guard(stream);

  if (observer_) {
    observer_->StartRuleHeader(StyleRule::kLayerBlock, prelude_offset_start);
    observer_->EndRuleHeader(prelude_offset_end);
    observer_->StartRuleBody(stream.Offset());
  }

  HeapVector<Member<StyleRuleBase>, 4> rules;
  ConsumeRuleListOrNestedDeclarationList(stream, nesting_type,
                                         parent_rule_for_nesting, &rules);

  if (observer_) {
    observer_->EndRuleBody(stream.Offset());
  }

  return MakeGarbageCollected<StyleRuleLayerBlock>(std::move(name),
                                                   std::move(rules));
}

StyleRulePositionTry* CSSParserImpl::ConsumePositionTryRule(
    CSSParserTokenStream& stream) {
  // Parse the prelude.
  wtf_size_t prelude_offset_start = stream.LookAheadOffset();
  const CSSParserToken& name_token = stream.Peek();
  // <dashed-ident>, and -internal-* for UA sheets only.
  String name;
  if (name_token.GetType() == kIdentToken) {
    name = name_token.Value().ToString();
    if (!name.starts_with("--") &&
        !(context_->Mode() == kUASheetMode && name.starts_with("-internal-"))) {
      ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRulePositionTry);
      return nullptr;
    }
  } else {
    ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRulePositionTry);
    return nullptr;
  }
  stream.ConsumeIncludingWhitespace();
  wtf_size_t prelude_offset_end = stream.LookAheadOffset();
  if (!ConsumeEndOfPreludeForAtRuleWithBlock(
          stream, CSSAtRuleID::kCSSAtRulePositionTry)) {
    return nullptr;
  }

  // Parse the actual block.
  CSSParserTokenStream::BlockGuard guard(stream);
  if (observer_) {
    observer_->StartRuleHeader(StyleRule::kPositionTry, prelude_offset_start);
    observer_->EndRuleHeader(prelude_offset_end);
    observer_->StartRuleBody(stream.Offset());
  }

  ConsumeBlockContents(stream, StyleRule::kPositionTry, CSSNestingType::kNone,
                       /*parent_rule_for_nesting=*/nullptr,
                       /*nested_declarations_start_index=*/kNotFound,
                       /*child_rules=*/nullptr);

  if (observer_) {
    observer_->EndRuleBody(stream.LookAheadOffset());
  }

  return MakeGarbageCollected<StyleRulePositionTry>(
      AtomicString(name),
      CreateCSSPropertyValueSet(parsed_properties_, kCSSPositionTryRuleMode,
                                context_->GetDocument()));
}

// Consume a type for CSS Functions; e.g. <length>, <color>, etc..
//
// https://drafts.csswg.org/css-mixins-1/#typedef-css-type
static std::optional<CSSSyntaxDefinition> ConsumeFunctionType(
    CSSParserTokenStream& stream) {
  // The <syntax> must generally be wrapped in type().
  if (stream.Peek().FunctionId() == CSSValueID::kType) {
    CSSParserTokenStream::RestoringBlockGuard guard(stream);
    stream.ConsumeWhitespace();
    std::optional<CSSSyntaxDefinition> type =
        CSSSyntaxDefinition::Consume(stream);
    if (type.has_value() && guard.Release()) {
      stream.ConsumeWhitespace();
      return type;
    }
  }
  // However, a lone <syntax-component> may appear unwrapped.
  return CSSSyntaxDefinition::ConsumeComponent(stream);
}

StyleRuleFunction* CSSParserImpl::ConsumeFunctionRule(
    CSSParserTokenStream& stream) {
  wtf_size_t prelude_offset_start = stream.LookAheadOffset();

  // Parse the prelude; first a function token (the name), then parameters,
  // then return type.
  if (stream.Peek().GetType() != kFunctionToken) {
    ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleFunction);
    return nullptr;  // Parse error.
  }
  AtomicString name =
      stream.Peek()
          .Value()
          .ToAtomicString();  // Includes the opening parenthesis.
  std::optional<HeapVector<StyleRuleFunction::Parameter>> parameters;
  {
    CSSParserTokenStream::BlockGuard guard(stream);
    stream.ConsumeWhitespace();
    parameters = ConsumeFunctionParameters(stream);
  }
  if (!parameters.has_value()) {
    ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleFunction);
    return nullptr;
  }
  stream.ConsumeWhitespace();

  std::optional<CSSSyntaxDefinition> return_type;
  if (stream.Peek().Id() == CSSValueID::kReturns) {
    stream.ConsumeIncludingWhitespace();  // kReturns
    return_type = ConsumeFunctionType(stream);
    if (!return_type.has_value()) {
      ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleFunction);
      return nullptr;
    }
  } else {
    return_type = CSSSyntaxDefinition::CreateUniversal();
  }

  wtf_size_t prelude_offset_end = stream.LookAheadOffset();
  if (!ConsumeEndOfPreludeForAtRuleWithBlock(stream,
                                             CSSAtRuleID::kCSSAtRuleFunction)) {
    return nullptr;
  }

  // Parse the actual block.
  CSSParserTokenStream::BlockGuard guard(stream);

  if (observer_) {
    observer_->StartRuleHeader(StyleRule::kFunction, prelude_offset_start);
    observer_->EndRuleHeader(prelude_offset_end);
    observer_->StartRuleBody(stream.Offset());
  }

  HeapVector<Member<StyleRuleBase>, 4> child_rules;
  ConsumeBlockContents(stream, StyleRule::kFunction, CSSNestingType::kFunction,
                       /*parent_rule_for_nesting=*/nullptr,
                       /*nested_declarations_start_index=*/0, &child_rules,
                       /*has_visited_pseudo=*/false);

  if (observer_) {
    observer_->EndRuleBody(stream.LookAheadOffset());
  }

  return MakeGarbageCollected<StyleRuleFunction>(
      name, std::move(*parameters),
      HeapVector<Member<StyleRuleBase>>(child_rules), std::move(*return_type));
}

StyleRuleMixin* CSSParserImpl::ConsumeMixinRule(CSSParserTokenStream& stream) {
  wtf_size_t header_start = stream.LookAheadOffset();

  // @mixin must be top-level, and as such, we need to clear the arena
  // after we're done parsing it (like ConsumeStyleRule() does).
  if (in_nested_style_rule_) {
    return nullptr;
  }
  auto func_clear_arena = [&](HeapVector<CSSSelector>* arena) {
    arena->resize(0);  // See class comment on CSSSelectorParser.
  };
  std::unique_ptr<HeapVector<CSSSelector>, decltype(func_clear_arena)>
      scope_guard(&arena_, std::move(func_clear_arena));

  // Parse the prelude: a dashed ident with an optional parameter list.
  if (stream.Peek().GetType() != kIdentToken &&
      stream.Peek().GetType() != kFunctionToken) {
    ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleMixin);
    return nullptr;  // Parse error.
  }
  AtomicString name = stream.Peek().Value().ToAtomicString();
  if (!name.starts_with("--")) {
    ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleMixin);
    return nullptr;
  }

  // Parse the parameter list (which may be empty).
  std::optional<HeapVector<StyleRuleFunction::Parameter>> parameters;
  if (stream.Peek().GetType() == kIdentToken) {
    stream.ConsumeIncludingWhitespace();
    parameters.emplace();
  } else {
    CSSParserTokenStream::BlockGuard guard(stream);
    stream.ConsumeWhitespace();
    parameters = ConsumeFunctionParameters(stream);
  }
  if (!parameters.has_value()) {
    ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleMixin);
    return nullptr;
  }
  stream.ConsumeWhitespace();

  // After the name or parameter list, there should be nothing (there's no
  // return value, unlike with functions).
  if (!ConsumeEndOfPreludeForAtRuleWithBlock(stream,
                                             CSSAtRuleID::kCSSAtRuleMixin)) {
    return nullptr;
  }
  wtf_size_t header_end = stream.LookAheadOffset();

  if (observer_) {
    observer_->StartRuleHeader(StyleRule::kMixin, header_start);
    observer_->EndRuleHeader(header_end);
    observer_->StartRuleBody(stream.Offset());
  }

  // Parse the actual block.
  CSSParserTokenStream::BlockGuard guard(stream);
  HeapVector<Member<StyleRuleBase>, 4> child_rules;
  ConsumeBlockContents(stream, StyleRule::kMixin, CSSNestingType::kMixin,
                       /*parent_rule_for_nesting=*/nullptr,
                       /*nested_declarations_start_index=*/0, &child_rules,
                       /*has_visited_pseudo=*/false);

  if (observer_) {
    observer_->EndRuleBody(stream.LookAheadOffset());
  }

  return MakeGarbageCollected<StyleRuleMixin>(name, std::move(*parameters),
                                              std::move(child_rules));
}

StyleRuleResult* CSSParserImpl::ConsumeResultRule(
    CSSParserTokenStream& stream) {
  wtf_size_t header_start = stream.LookAheadOffset();

  // The prelude should be empty.
  if (!ConsumeEndOfPreludeForAtRuleWithBlock(stream,
                                             CSSAtRuleID::kCSSAtRuleResult)) {
    return nullptr;
  }

  stream.EnsureLookAhead();
  wtf_size_t header_end = stream.LookAheadOffset();
  if (observer_) {
    observer_->StartRuleHeader(StyleRule::kResult, header_start);
    observer_->EndRuleHeader(header_end);
  }

  // Parse the actual block.
  StyleRule* fake_parent_rule;
  {
    base::AutoReset<bool> reset_in_mixin(&in_mixin_, true);
    fake_parent_rule = ConsumeDeclarationListForMixins(stream);
  }

  // ConsumeDeclarationListForMixins() must have a fake parent rule in case
  // there are any rules containing parent selectors (including raw
  // declarations, as they are wrapped in an implicit nested block); however,
  // StyleRuleResult is a StyleRuleGroup and expects to own its rules itself.
  // This means that even though fake_parent_rule is the parent pointed to by
  // the selectors (and will be kept alive by them), it doesn't actually
  // contain the child rules and isn't used for anything anymore. Once
  // a mixin is actually used (in @apply), we clone all the rules and call
  // Clone(), which changes all the parent references to @apply's parent.
  fake_parent_rule->EnsureChildRules();
  return MakeGarbageCollected<StyleRuleResult>(
      HeapVector{std::move(*fake_parent_rule->ChildRules())});
}

// Parses one variable declared inside a @private block e.g. `--foo: 10px` or
// `--foo <length>: 10px`. Returns nullptr if the declaration is invalid.
static CSSPrivateVariable* ConsumePrivateVariable(
    CSSParserTokenStream& stream,
    const CSSParserContext& context) {
  CSSParserTokenStream::Boundary boundary(stream, kSemicolonToken);

  const CSSParserToken& name_token = stream.Peek();
  if (!CSSVariableParser::IsValidVariableName(name_token)) {
    return nullptr;
  }
  AtomicString name = name_token.Value().ToAtomicString();
  stream.ConsumeIncludingWhitespace();

  std::optional<CSSSyntaxDefinition> type = ConsumeFunctionType(stream);

  if (stream.Peek().GetType() != kColonToken) {
    return nullptr;
  }
  stream.ConsumeIncludingWhitespace();
  // TODO(crbug.com/549237151): !important is invalid in a @private block per
  // CSSWG resolution and needs followup to ensure it is fully rejected.
  bool important_ignored;
  CSSVariableData* default_value =
      CSSVariableParser::ConsumeUnparsedDeclaration(
          stream, /*allow_important_annotation=*/false,
          /*is_animation_tainted=*/false,
          /*must_contain_variable_reference=*/false,
          /*restricted_value=*/false,
          /*comma_ends_declaration=*/false, important_ignored, context);
  if (!default_value) {
    return nullptr;
  }

  // If a type and a default are both provided, the default must parse
  // successfully according to that type.
  CSSSyntaxDefinition syntax =
      type.value_or(CSSSyntaxDefinition::CreateUniversal());
  if (default_value && !default_value->NeedsVariableResolution() &&
      !syntax.IsUniversal()) {
    CSSParserLocalContext local_context =
        CSSParserLocalContext::CreateWithoutPropertyForSyntaxParsing();
    if (!syntax.Parse(default_value->OriginalText(), context, local_context,
                      /*is_animation_tainted=*/false,
                      /*is_attr_tainted=*/false)) {
      return nullptr;
    }
  }

  // Reject any tokens left over before the ';' (e.g. a stray type after the
  // value, or other trailing garbage).
  if (!stream.AtEnd()) {
    return nullptr;
  }

  return MakeGarbageCollected<CSSPrivateVariable>(name, std::move(syntax),
                                                  default_value, &context);
}

StyleRulePrivate* CSSParserImpl::ConsumePrivateRule(
    CSSParserTokenStream& stream) {
  wtf_size_t header_start = stream.LookAheadOffset();

  if (!ConsumeEndOfPreludeForAtRuleWithBlock(stream,
                                             CSSAtRuleID::kCSSAtRulePrivate)) {
    return nullptr;
  }

  stream.EnsureLookAhead();
  wtf_size_t header_end = stream.LookAheadOffset();
  CSSParserTokenStream::BlockGuard guard(stream);
  if (observer_) {
    observer_->StartRuleHeader(StyleRule::kPrivate, header_start);
    observer_->EndRuleHeader(header_end);
    observer_->StartRuleBody(stream.Offset());
  }

  // Parse the variables declared in the block. Invalid declarations are skipped
  // so that the valid ones are still collected.
  HeapVector<Member<const CSSPrivateVariable>> private_variables;
  stream.ConsumeWhitespace();
  while (!stream.AtEnd()) {
    if (CSSPrivateVariable* variable =
            ConsumePrivateVariable(stream, *context_)) {
      private_variables.push_back(variable);
    } else {
      stream.SkipUntilPeekedTypeIs<kSemicolonToken>();
    }
    if (!stream.AtEnd()) {
      stream.ConsumeIncludingWhitespace();
    }
  }

  if (observer_) {
    observer_->EndRuleBody(stream.LookAheadOffset());
  }

  return MakeGarbageCollected<StyleRulePrivate>(std::move(private_variables));
}

StyleRule* CSSParserImpl::ConsumeDeclarationListForMixins(
    CSSParserTokenStream& stream) {
  CSSParserTokenStream::BlockGuard guard(stream);

  if (observer_) {
    observer_->StartRuleBody(stream.Offset());
  }

  // When we encounter a declaration list, the selector of our fake parent rule
  // will be _copied_, so it needs to be something sane; the implicit @nest rule
  // gives us the behavior that we want.
  CSSSelector dummy(/*parent_rule=*/nullptr, /*is_implicit=*/true);
  dummy.SetLastInSelectorList(true);
  dummy.SetLastInComplexSelector(true);

  // We do not use the properties for anything, but we need a valid pointer
  // or we will have a crash when we try to clone the rule during apply.
  ImmutableCSSPropertyValueSet* empty_properties =
      ImmutableCSSPropertyValueSet::Create({},
                                           CSSParserMode::kHTMLStandardMode);

  StyleRule* fake_parent_rule =
      StyleRule::Create(base::span_from_ref(dummy), empty_properties);
  HeapVector<Member<StyleRuleBase>, 4> child_rules;
  ConsumeRuleListOrNestedDeclarationList(stream, CSSNestingType::kNesting,
                                         fake_parent_rule, &child_rules);
  for (StyleRuleBase* child_rule : child_rules) {
    fake_parent_rule->AddChildRule(child_rule);
  }

  if (observer_) {
    observer_->EndRuleBody(stream.Offset());
  }

  return fake_parent_rule;
}

StyleRuleApplyMixin* CSSParserImpl::ConsumeApplyMixinRule(
    CSSParserTokenStream& stream) {
  wtf_size_t header_start = stream.LookAheadOffset();
  if (stream.Peek().GetType() != kIdentToken &&
      stream.Peek().GetType() != kFunctionToken) {
    ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleApplyMixin);
    return nullptr;  // Parse error.
  }
  AtomicString name = stream.Peek().Value().ToAtomicString();
  if (!name.starts_with("--")) {
    ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleApplyMixin);
    return nullptr;
  }

  // Parse arguments, if any.
  HeapVector<Member<CSSVariableData>> arguments;
  if (stream.Peek().GetType() == kIdentToken) {
    // @apply --name ...
    stream.ConsumeIncludingWhitespace();
  } else {
    // @apply --name( ...
    if (!CSSVariableParser::ConsumeMixinArguments(stream, *context_,
                                                  arguments)) {
      ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleApplyMixin);
      return nullptr;
    }
  }

  stream.EnsureLookAhead();
  wtf_size_t header_end = stream.LookAheadOffset();
  if (observer_) {
    observer_->StartRuleHeader(StyleRule::kApplyMixin, header_start);
    observer_->EndRuleHeader(header_end);
  }

  if (stream.AtEnd() || stream.UncheckedPeek().GetType() == kSemicolonToken) {
    // No declarations block, just a semicolon (possibly implicit.
    if (!stream.AtEnd()) {
      stream.UncheckedConsume();  // kSemicolonToken
    }
    if (observer_) {
      // Devtools expects to see a rule body for every rule; it is
      // the trigger for actually inserting the rule. So we need to
      // include a fake empty body here, or the indexing will be
      // messed up when the NestedDeclarations rules arrive.
      observer_->StartRuleBody(stream.Offset());
      observer_->EndRuleBody(stream.Offset());
    }
    return MakeGarbageCollected<StyleRuleApplyMixin>(name,
                                                     std::move(arguments));
  }

  if (stream.UncheckedPeek().GetType() != kLeftBraceToken) {
    ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleApplyMixin);
    return nullptr;  // Parse error.
  }

  // Parse the @contents block.
  StyleRule* fake_parent_rule = ConsumeDeclarationListForMixins(stream);
  fake_parent_rule->EnsureChildRules();
  return MakeGarbageCollected<StyleRuleApplyMixin>(
      name, std::move(arguments),
      HeapVector{std::move(*fake_parent_rule->ChildRules())});
}

StyleRuleContentsStatement* CSSParserImpl::ConsumeContentsRule(
    CSSParserTokenStream& stream) {
  wtf_size_t header_start = stream.LookAheadOffset();
  stream.ConsumeWhitespace();
  if (stream.AtEnd() || stream.UncheckedPeek().GetType() == kSemicolonToken) {
    // No block, just a semicolon (possibly implicit).
    if (observer_) {
      observer_->StartRuleHeader(StyleRule::kContents, header_start);
      observer_->EndRuleHeader(stream.Offset());
      observer_->StartRuleBody(stream.Offset());
      observer_->EndRuleBody(stream.Offset());
    }
    if (!stream.AtEnd()) {
      stream.UncheckedConsume();  // kSemicolonToken
    }
    return MakeGarbageCollected<StyleRuleContentsStatement>(
        HeapVector<Member<StyleRuleBase>>{});
  }

  if (stream.UncheckedPeek().GetType() != kLeftBraceToken) {
    ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleContents);
    return nullptr;  // Parse error.
  }
  wtf_size_t header_end = stream.LookAheadOffset();

  if (observer_) {
    observer_->StartRuleHeader(StyleRule::kContents, header_start);
    observer_->EndRuleHeader(header_end);
  }

  // Parse the actual block.
  StyleRule* fake_parent_rule = ConsumeDeclarationListForMixins(stream);
  fake_parent_rule->EnsureChildRules();
  return MakeGarbageCollected<StyleRuleContentsStatement>(
      HeapVector{std::move(*fake_parent_rule->ChildRules())});
}

// Parse the parameters of a CSS function: Zero or more comma-separated
// instances of [ <name> <type>? [ : <default-value> ]? ].
// Returns the empty value on parse error.
std::optional<HeapVector<StyleRuleFunction::Parameter>>
CSSParserImpl::ConsumeFunctionParameters(CSSParserTokenStream& stream) {
  HeapVector<StyleRuleFunction::Parameter> parameters;
  bool first_parameter = true;
  for (;;) {
    stream.ConsumeWhitespace();

    if (first_parameter && stream.Peek().GetType() == kRightParenthesisToken) {
      // No arguments.
      break;
    }
    if (stream.Peek().GetType() != kIdentToken) {
      return {};  // Parse error.
    }
    String parameter_name = stream.Peek().Value().ToString();
    if (!CSSVariableParser::IsValidVariableName(parameter_name)) {
      return {};
    }
    stream.ConsumeIncludingWhitespace();

    std::optional<CSSSyntaxDefinition> type = ConsumeFunctionType(stream);

    CSSVariableData* default_value = nullptr;
    if (stream.Peek().GetType() == kColonToken) {
      stream.ConsumeIncludingWhitespace();

      // Note that this is a comma-containing production [1], and therefore
      // the value may not contain commas until we support the {} wrapper
      // defined by the spec.
      // [1] https://drafts.csswg.org/css-values-5/#component-function-commas
      bool important_ignored;
      default_value = CSSVariableParser::ConsumeUnparsedDeclaration(
          stream, /*allow_important_annotation=*/false,
          /*is_animation_tainted=*/false,
          /*must_contain_variable_reference=*/false,
          /*restricted_value=*/false,
          /*comma_ends_declaration=*/true, important_ignored, *context_);
    }

    // We just check the syntax here, we don't actually parse calc()
    // expressions, so we don't need property context for random().
    CSSParserLocalContext local_context =
        CSSParserLocalContext::CreateWithoutPropertyForSyntaxParsing();
    // If a type and a default are both provided, the default must
    // parse successfully according to that type.
    //
    // https://drafts.csswg.org/css-mixins-1/#function-rule
    if (type.has_value() && default_value) {
      if (!default_value->NeedsVariableResolution() &&
          !type->Parse(default_value->OriginalText(), *context_, local_context,
                       /*is_animation_tainted=*/false,
                       /*is_attr_tainted=*/false)) {
        return std::nullopt;
      }
    }

    parameters.push_back(StyleRuleFunction::Parameter{
        parameter_name, type.value_or(CSSSyntaxDefinition::CreateUniversal()),
        default_value});
    if (stream.Peek().GetType() == kRightParenthesisToken) {
      // No more arguments.
      break;
    }
    if (stream.Peek().GetType() != kCommaToken) {
      return {};  // Expected more parameters, or end of argument list.
    }
    stream.ConsumeIncludingWhitespace();
    first_parameter = false;
  }
  return parameters;
}

StyleRuleKeyframe* CSSParserImpl::ConsumeKeyframeStyleRule(
    std::unique_ptr<Vector<KeyframeOffset>> key_list,
    const RangeOffset& prelude_offset,
    CSSParserTokenStream& block) {
  if (!key_list) {
    return nullptr;
  }

  if (observer_) {
    observer_->StartRuleHeader(StyleRule::kKeyframe, prelude_offset.start);
    observer_->EndRuleHeader(prelude_offset.end);
    observer_->StartRuleBody(block.Offset());
  }

  ConsumeBlockContents(block, StyleRule::kKeyframe, CSSNestingType::kNone,
                       /*parent_rule_for_nesting=*/nullptr,
                       /*nested_declarations_start_index=*/kNotFound,
                       /*child_rules=*/nullptr);

  if (observer_) {
    observer_->EndRuleBody(block.LookAheadOffset());
  }

  return MakeGarbageCollected<StyleRuleKeyframe>(
      std::move(key_list),
      CreateCSSPropertyValueSet(parsed_properties_, kCSSKeyframeRuleMode,
                                context_->GetDocument()));
}

namespace {

// https://drafts.csswg.org/css-extensions-1/#typedef-extension-name
bool IsValidExtensionName(const CSSParserToken& token) {
  if (token.GetType() != kIdentToken) {
    return false;
  }
  StringView value = token.Value();
  // SAFETY: length checked and &&-expression short circuit.
  return value.length() >= 2 && UNSAFE_BUFFERS(value[0]) == '-' &&
         UNSAFE_BUFFERS(value[1]) == '-';
}

std::optional<bool> GetBooleanValue(const CSSParserToken& token) {
  if (token.GetType() != kIdentToken) {
    return std::nullopt;
  }
  if (token.Value() == "true") {
    return true;
  }
  if (token.Value() == "false") {
    return false;
  }
  return std::nullopt;
}

}  // namespace

StyleRuleCustomMedia* CSSParserImpl::ConsumeCustomMediaRule(
    CSSParserTokenStream& stream) {
  const CSSParserToken& name_token = stream.Peek();
  if (!IsValidExtensionName(name_token)) {
    ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleCustomMedia);
    return nullptr;
  }
  String name = name_token.Value().ToString();
  stream.ConsumeIncludingWhitespace();

  std::optional<bool> bool_val = GetBooleanValue(stream.Peek());
  if (bool_val.has_value()) {
    stream.ConsumeIncludingWhitespace();
    if (!ConsumeEndOfPreludeForAtRuleWithoutBlock(
            stream, CSSAtRuleID::kCSSAtRuleCustomMedia)) {
      return nullptr;
    }
    return MakeGarbageCollected<StyleRuleCustomMedia>(
        StyleRuleCustomMedia(AtomicString(name), *bool_val));
  }

  MediaQuerySet* media_query_set = MediaQueryParser::ParseCustomMediaDefinition(
      stream, context_->GetExecutionContext());
  if (!media_query_set) {
    ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleCustomMedia);
    return nullptr;
  }
  if (!ConsumeEndOfPreludeForAtRuleWithoutBlock(
          stream, CSSAtRuleID::kCSSAtRuleCustomMedia)) {
    return nullptr;
  }
  return MakeGarbageCollected<StyleRuleCustomMedia>(
      StyleRuleCustomMedia(AtomicString(name), media_query_set));
}

StyleRule* CSSParserImpl::ConsumeStyleRule(CSSParserTokenStream& stream,
                                           CSSNestingType nesting_type,
                                           StyleRule* parent_rule_for_nesting,
                                           bool nested,
                                           bool& invalid_rule_error) {
  if (!in_nested_style_rule_) {
    DCHECK_EQ(0u, arena_.size());
  }
  auto func_clear_arena = [&](HeapVector<CSSSelector>* arena) {
    if (!in_nested_style_rule_) {
      arena->resize(0);  // See class comment on CSSSelectorParser.
    }
  };
  std::unique_ptr<HeapVector<CSSSelector>, decltype(func_clear_arena)>
      scope_guard(&arena_, std::move(func_clear_arena));

  if (observer_) {
    observer_->StartRuleHeader(StyleRule::kStyle, stream.LookAheadOffset());
  }

  // Style rules that look like custom property declarations
  // are not allowed by css-syntax.
  //
  // https://drafts.csswg.org/css-syntax/#consume-qualified-rule
  bool custom_property_ambiguity =
      CSSVariableParser::StartsCustomPropertyDeclaration(stream);

  bool has_visited_pseudo = false;
  // Parse the prelude of the style rule
  base::span<CSSSelector> selector_vector = CSSSelectorParser::ConsumeSelector(
      stream, context_, nesting_type, parent_rule_for_nesting,
      /* semicolon_aborts_nested_selector*/ nested, style_sheet_, observer_,
      arena_, &has_visited_pseudo);

  if (selector_vector.empty()) {
    // Read the rest of the prelude if there was an error
    stream.EnsureLookAhead();
    if (nested) {
      stream.SkipUntilPeekedTypeIs<kLeftBraceToken, kSemicolonToken>();
    } else {
      stream.SkipUntilPeekedTypeIs<kLeftBraceToken>();
    }
  }

  if (observer_) {
    observer_->EndRuleHeader(stream.LookAheadOffset());
  }

  if (stream.Peek().GetType() != kLeftBraceToken) {
    // Parse error, EOF instead of qualified rule block
    // (or we went into error recovery above).
    // NOTE: If we aborted due to a semicolon, don't consume it here;
    // the caller will do that for us.
    return nullptr;
  }

  if (custom_property_ambiguity) {
    if (nested) {
      // https://drafts.csswg.org/css-syntax/#consume-the-remnants-of-a-bad-declaration
      // Note that the caller consumes the bad declaration remnants
      // (see ConsumeBlockContents).
      return nullptr;
    }
    // "If nested is false, consume a block from input, and return nothing."
    // https://drafts.csswg.org/css-syntax/#consume-qualified-rule
    CSSParserTokenStream::BlockGuard guard(stream);
    return nullptr;
  }
  // Check if rule is "valid in current context".
  // https://drafts.csswg.org/css-syntax/#consume-qualified-rule
  //
  // This means checking if the selector parsed successfully.
  if (selector_vector.empty()) {
    CSSParserTokenStream::BlockGuard guard(stream);
    invalid_rule_error = true;
    return nullptr;
  }

  // TODO(csharrison): How should we lazily parse css that needs the observer?
  if (!observer_ && lazy_state_) {
    DCHECK(style_sheet_);

    StringView text(stream.RemainingText(), 1);
#ifdef ARCH_CPU_X86_FAMILY
    static const bool kHasAVX2AndPCLMUL =
        base::CPU::GetInstanceNoAllocation().has_avx2() &&
        base::CPU::GetInstanceNoAllocation().has_pclmul();
    wtf_size_t len;
    if (kHasAVX2AndPCLMUL) {
      len = static_cast<wtf_size_t>(FindLengthOfDeclarationListAVX2(text));
    } else {
      len = static_cast<wtf_size_t>(FindLengthOfDeclarationList(text));
    }
#else
    wtf_size_t len = static_cast<wtf_size_t>(FindLengthOfDeclarationList(text));
#endif
    if (len != 0) {
      wtf_size_t block_start_offset = stream.Offset();
      stream.SkipToEndOfBlock(len + 2);  // +2 for { and }.
      return StyleRule::Create(selector_vector, lazy_state_,
                               block_start_offset);
    }
  }
  CSSParserTokenStream::BlockGuard guard(stream);
  return ConsumeStyleRuleContents(selector_vector, stream, has_visited_pseudo);
}

StyleRule* CSSParserImpl::ConsumeStyleRuleContents(
    base::span<CSSSelector> selector_vector,
    CSSParserTokenStream& stream,
    bool has_visited_pseudo) {
  StyleRule* style_rule = StyleRule::Create(selector_vector);
  HeapVector<Member<StyleRuleBase>, 4> child_rules;
  if (observer_) {
    observer_->StartRuleBody(stream.Offset());
  }
  ConsumeBlockContents(stream, StyleRule::kStyle, CSSNestingType::kNesting,
                       /*parent_rule_for_nesting=*/style_rule,
                       /*nested_declarations_start_index=*/kNotFound,
                       &child_rules, has_visited_pseudo);
  if (observer_) {
    observer_->EndRuleBody(stream.LookAheadOffset());
  }
  for (StyleRuleBase* child_rule : child_rules) {
    style_rule->AddChildRule(child_rule);
  }
  style_rule->SetProperties(CreateCSSPropertyValueSet(
      parsed_properties_, context_->Mode(), context_->GetDocument()));
  return style_rule;
}

// https://drafts.csswg.org/css-syntax/#consume-block-contents
//
// Consumes declarations and/or child rules from the block of a style rule
// or an at-rule (e.g. @media).
//
// The `nested_declarations_start_index` parameter controls how this function
// emits "nested declaration" rules for the leading block of declarations.
// For regular style rules (which can hold declarations directly), this should
// be kNotFound, which will prevent a wrapper rule for the leading block.
// (Subsequent declarations "interleaved" with child rules will still be
// wrapped). For nested group rules, or generally rules that cannot hold
// declarations directly (e.g. @media), the parameter value should be 0u,
// causing the leading declarations to get wrapped as well.
void CSSParserImpl::ConsumeBlockContents(
    CSSParserTokenStream& stream,
    StyleRule::RuleType rule_type,
    CSSNestingType nesting_type,
    StyleRule* parent_rule_for_nesting,
    wtf_size_t nested_declarations_start_index,
    HeapVector<Member<StyleRuleBase>, 4>* child_rules,
    bool has_visited_pseudo) {
  DCHECK(parsed_properties_.empty());

  while (true) {
    // Having a lookahead may skip comments, which are used by the observer.
    DCHECK(!stream.HasLookAhead() || stream.AtEnd());

    if (observer_ && !stream.HasLookAhead()) {
      while (true) {
        wtf_size_t start_offset = stream.Offset();
        if (!stream.ConsumeCommentOrNothing()) {
          break;
        }
        observer_->ObserveComment(start_offset, stream.Offset());
      }
    }

    if (stream.AtEnd()) {
      break;
    }

    switch (stream.UncheckedPeek().GetType()) {
      case kWhitespaceToken:
      case kSemicolonToken:
        stream.UncheckedConsume();
        break;
      case kAtKeywordToken: {
        CSSParserToken name_token = stream.ConsumeIncludingWhitespace();
        const StringView name = name_token.Value();
        const CSSAtRuleID id = CssAtRuleID(name);
        bool invalid_rule_error_ignored = false;
        StyleRuleBase* child = ConsumeNestedRule(
            id, rule_type, stream, nesting_type, parent_rule_for_nesting,
            invalid_rule_error_ignored);
        // "Consume an at-rule" can't return invalid-rule-error.
        // https://drafts.csswg.org/css-syntax/#consume-at-rule
        DCHECK(!invalid_rule_error_ignored);
        if (child && child_rules) {
          EmitDeclarationsRuleIfNeeded(
              rule_type, nesting_type, parent_rule_for_nesting,
              nested_declarations_start_index, *child_rules);
          nested_declarations_start_index = parsed_properties_.size();
          child_rules->push_back(child);
        }
        break;
      }
      case kIdentToken: {
        CSSParserTokenStream::State state = stream.Save();
        bool consumed_declaration = false;
        {
          CSSParserTokenStream::Boundary boundary(stream, kSemicolonToken);
          consumed_declaration =
              ConsumeDeclaration(stream, rule_type, has_visited_pseudo);
        }
        if (consumed_declaration) {
          if (!stream.AtEnd()) {
            DCHECK_EQ(stream.UncheckedPeek().GetType(), kSemicolonToken);
            stream.UncheckedConsume();  // kSemicolonToken
          }
          break;
        } else if (stream.Peek().GetType() == kSemicolonToken) {
          // As an optimization, we avoid the restart below (retrying as a
          // nested style rule) if we ended on a kSemicolonToken, as this
          // situation can't produce a valid rule.
          stream.UncheckedConsume();  // kSemicolonToken
          break;
        }
        // Retry as nested rule.
        stream.Restore(state);
        [[fallthrough]];
      }
      default:
        if (nesting_type != CSSNestingType::kNone &&
            nesting_type != CSSNestingType::kFunction &&
            nesting_type != CSSNestingType::kMixin) {
          bool invalid_rule_error = false;
          StyleRuleBase* child =
              ConsumeNestedRule(std::nullopt, rule_type, stream, nesting_type,
                                parent_rule_for_nesting, invalid_rule_error);
          if (child) {
            if (child_rules) {
              EmitDeclarationsRuleIfNeeded(
                  rule_type, nesting_type, parent_rule_for_nesting,
                  nested_declarations_start_index, *child_rules);
              nested_declarations_start_index = parsed_properties_.size();
              child_rules->push_back(child);
            }
            break;
          } else if (invalid_rule_error) {
            // https://drafts.csswg.org/css-syntax/#invalid-rule-error
            //
            // This means the rule was valid per the "core" grammar of
            // css-syntax, but the prelude (i.e. selector list) didn't parse.
            // We should not fall through to error recovery in this case,
            // because we should continue parsing immediately after
            // the {}-block.
            break;
          }
          // Fall through to error recovery.
          stream.EnsureLookAhead();
        }

        [[fallthrough]];
        // Function tokens should start parsing a declaration
        // (which then immediately goes into error recovery mode).
      case CSSParserTokenType::kFunctionToken:
        stream.SkipUntilPeekedTypeIs<kSemicolonToken>();
        if (!stream.UncheckedAtEnd()) {
          stream.UncheckedConsume();  // kSemicolonToken
        }

        break;
    }
  }

  // We need a final call to EmitDeclarationsRuleIfNeeded in case there
  // are trailing bare declarations. If no child rule has been observed,
  // nested_declarations_start_index is still kNotFound (UINT_MAX),
  // which causes EmitDeclarationsRuleIfNeeded to have no effect.
  if (child_rules) {
    EmitDeclarationsRuleIfNeeded(rule_type, nesting_type,
                                 parent_rule_for_nesting,
                                 nested_declarations_start_index, *child_rules);
  }
}

// Consumes a list of style rules and stores the result in `child_rules`,
// or (for nested group rules) consumes the interior of a nested group rule [1].
// Nested group rules allow a list of declarations to appear
// directly in place of where a list of rules would normally go.
//
// [1] https://drafts.csswg.org/css-nesting-1/#nested-group-rules
void CSSParserImpl::ConsumeRuleListOrNestedDeclarationList(
    CSSParserTokenStream& stream,
    CSSNestingType nesting_type,
    StyleRule* parent_rule_for_nesting,
    HeapVector<Member<StyleRuleBase>, 4>* child_rules) {
  DCHECK(child_rules);

  bool is_nested_group_rule = nesting_type == CSSNestingType::kNesting ||
                              nesting_type == CSSNestingType::kFunction ||
                              nesting_type == CSSNestingType::kMixin;
  if (is_nested_group_rule) {
    // This is a nested group rule, which (in addition to rules) allows
    // *declarations* to appear directly within the body of the rule, e.g.:
    //
    // .foo {
    //    @media (width > 800px) {
    //      color: green;
    //    }
    //  }
    //
    // Using nested_declarations_start_index=0u here means that the leading
    // declarations will be wrapped in a CSSNestedDeclarations rule.
    // Unlike regular style rules, the leading declarations must be wrapped
    // in something that can hold them, because group rules (e.g. @media)
    // can not hold properties directly.
    //
    // RuleType determines which declarations are valid within the rule.
    // Within @function rules, only local variables and the 'result' descriptor
    // are allowed. All other cases accept regular properties without special
    // restrictions.
    StyleRule::RuleType rule_type;
    if (nesting_type == CSSNestingType::kFunction) {
      rule_type = StyleRule::kFunction;
    } else if (nesting_type == CSSNestingType::kMixin) {
      rule_type = StyleRule::kMixin;
    } else {
      rule_type = StyleRule::kStyle;
    }
    ConsumeBlockContents(stream, rule_type, nesting_type,
                         parent_rule_for_nesting,
                         /* nested_declarations_start_index */ 0u, child_rules);
  } else {
    ConsumeRuleList(stream, kRegularRules,
                    /*allow_cdo_cdc_tokens=*/false, nesting_type,
                    parent_rule_for_nesting,
                    [child_rules](StyleRuleBase* rule, wtf_size_t) {
                      child_rules->push_back(rule);
                    });
  }
}

namespace {

AllowedRules AllowedNestedRules(StyleRule::RuleType parent_rule_type,
                                bool in_nested_style_rule,
                                bool in_mixin) {
  switch (parent_rule_type) {
    case StyleRule::kScope:
      if (!in_nested_style_rule) {
        return CSSParserImpl::kRegularRules;
      }
      [[fallthrough]];
    case StyleRule::kStyle: {
      if (in_mixin) {
        AllowedRules allowed = CSSParserImpl::kNestedGroupRules |
                               AllowedRules{CSSAtRuleID::kCSSAtRuleContents,
                                            CSSAtRuleID::kCSSAtRulePrivate};
        allowed.Remove(CSSAtRuleID::kCSSAtRuleLayer);
        return allowed;
      } else {
        // TODO(crbug.com/549226765): This also allows @private inside a
        // @media/@supports rule nested in a style rule, but needs a resolution
        // from the CSSWG.
        return CSSParserImpl::kNestedGroupRules |
               AllowedRules{CSSAtRuleID::kCSSAtRulePrivate};
      }
    }
    case StyleRule::kMixin:
      return CSSParserImpl::kConditionalRules |
             AllowedRules{CSSAtRuleID::kCSSAtRulePrivate,
                          CSSAtRuleID::kCSSAtRuleResult};
    case StyleRule::kPage:
      return CSSParserImpl::kPageMarginRules;
    case StyleRule::kFunction:
      return CSSParserImpl::kConditionalRules;
    default:
      break;
  }
  return AllowedRules();
}

}  // namespace

StyleRuleBase* CSSParserImpl::ConsumeNestedRule(
    std::optional<CSSAtRuleID> id,
    StyleRule::RuleType parent_rule_type,
    CSSParserTokenStream& stream,
    CSSNestingType nesting_type,
    StyleRule* parent_rule_for_nesting,
    bool& invalid_rule_error) {
  // A nested style rule. Recurse into the parser; we need to move the parsed
  // properties out of the way while we're parsing the child rule, though.
  HeapVector<CSSPropertyValue, 64> outer_parsed_properties;
  swap(parsed_properties_, outer_parsed_properties);
  StyleRuleBase* child;
  base::AutoReset<bool> reset_in_nested_style_rule(
      &in_nested_style_rule_,
      in_nested_style_rule_ || parent_rule_type == StyleRule::kStyle);
  if (!id.has_value()) {
    child = ConsumeStyleRule(stream, nesting_type, parent_rule_for_nesting,
                             /* nested */ true, invalid_rule_error);
  } else {
    child = ConsumeAtRuleContents(
        *id, stream,
        AllowedNestedRules(parent_rule_type, in_nested_style_rule_, in_mixin_),
        nesting_type, parent_rule_for_nesting);
  }
  parsed_properties_ = std::move(outer_parsed_properties);
  if (child && parent_rule_type != StyleRule::kPage &&
      parent_rule_type != StyleRule::kScope &&
      parent_rule_type != StyleRule::kFunction) {
    context_->Count(WebFeature::kCSSNesting);
  }
  return child;
}

// This function can leave the stream in one of the following states:
//
//  1) If the ident token is not immediately followed by kColonToken,
//     then the stream is left at the token where kColonToken was expected.
//  2) If the ident token is not a recognized property/descriptor,
//     then the stream is left at the token immediately after kColonToken.
//  3) Otherwise the stream is is left AtEnd(), regardless of whether or
//     not the value was valid.
//
// Leaving the stream in an awkward states is normally not desirable for
// Consume functions, but declarations are sometimes parsed speculatively,
// which may cause a restart at the call site (see ConsumeBlockContents,
// kIdentToken branch). If we are anyway going to restart, any work we do
// to leave the stream in a more consistent state is just wasted.
bool CSSParserImpl::ConsumeDeclaration(CSSParserTokenStream& stream,
                                       StyleRule::RuleType rule_type,
                                       bool has_visited_pseudo) {
  const wtf_size_t decl_offset_start = stream.Offset();

  DCHECK_EQ(stream.Peek().GetType(), kIdentToken);
  const CSSParserToken& lhs = stream.ConsumeIncludingWhitespace();
  if (stream.Peek().GetType() != kColonToken) {
    return false;  // Parse error.
  }

  stream.UncheckedConsume();  // kColonToken
  stream.EnsureLookAhead();

  size_t properties_count = parsed_properties_.size();

  bool parsing_descriptor =
      rule_type == StyleRule::kFontFace ||
      rule_type == StyleRule::kFontPaletteValues ||
      rule_type == StyleRule::kProperty || rule_type == StyleRule::kLocation ||
      rule_type == StyleRule::kCounterStyle ||
      rule_type == StyleRule::kViewTransition ||
      rule_type == StyleRule::kFunction || rule_type == StyleRule::kMixin;

  uint64_t id = parsing_descriptor
                    ? static_cast<uint64_t>(lhs.ParseAsAtRuleDescriptorID())
                    : static_cast<uint64_t>(lhs.ParseAsUnresolvedCSSPropertyID(
                          context_->GetExecutionContext(), context_->Mode()));

  bool important = false;

  static_assert(static_cast<uint64_t>(AtRuleDescriptorID::Invalid) == 0u);
  static_assert(static_cast<uint64_t>(CSSPropertyID::kInvalid) == 0u);

  stream.ConsumeWhitespace();

  if (id) {
    if (parsing_descriptor) {
      const AtRuleDescriptorID atrule_id = static_cast<AtRuleDescriptorID>(id);
      const AtomicString& variable_name =
          (atrule_id == AtRuleDescriptorID::Variable
               ? lhs.Value().ToAtomicString()
               : g_null_atom);
      AtRuleDescriptorParser::ParseDescriptorValue(
          rule_type, atrule_id, variable_name, stream, *context_,
          parsed_properties_);
    } else {
      const CSSPropertyID unresolved_property = static_cast<CSSPropertyID>(id);
      if (unresolved_property == CSSPropertyID::kVariable) {
        if (rule_type != StyleRule::kStyle && rule_type != StyleRule::kScope &&
            rule_type != StyleRule::kKeyframe) {
          return false;
        }
        AtomicString variable_name = lhs.Value().ToAtomicString();
        bool allow_important_annotation = (rule_type != StyleRule::kKeyframe);
        bool is_animation_tainted = rule_type == StyleRule::kKeyframe;
        if (!ConsumeVariableValue(stream, variable_name,
                                  allow_important_annotation,
                                  is_animation_tainted)) {
          return false;
        }
      } else if (unresolved_property != CSSPropertyID::kInvalid) {
        if (observer_) {
          CSSParserTokenStream::State savepoint = stream.Save();
          ConsumeDeclarationValue(stream, unresolved_property,
                                  /*is_in_declaration_list=*/true, rule_type);

          // The observer would like to know (below) whether this declaration
          // was !important or not. If our parse succeeded, we can just pick it
          // out from the list of properties. If not, we'll need to look at the
          // tokens ourselves.
          if (parsed_properties_.size() != properties_count) {
            important = parsed_properties_.back().IsImportant();
          } else {
            stream.Restore(savepoint);
            // NOTE: This call is solely to update “important”.
            CSSVariableParser::ConsumeUnparsedDeclaration(
                stream, /*allow_important_annotation=*/true,
                /*is_animation_tainted=*/false,
                /*must_contain_variable_reference=*/false,
                /*restricted_value=*/true, /*comma_ends_declaration=*/false,
                important, *context_);
          }
        } else {
          if (context_->IsUseCounterRecordingEnabled() && has_visited_pseudo &&
              unresolved_property == CSSPropertyID::kColumnRuleColor) {
            context_->Count(WebFeature::kVisitedColumnRuleColor);
          }
          ConsumeDeclarationValue(stream, unresolved_property,
                                  /*is_in_declaration_list=*/true, rule_type);
        }
      }
    }
  }
  if (observer_ &&
      (rule_type == StyleRule::kStyle || rule_type == StyleRule::kScope ||
       rule_type == StyleRule::kKeyframe || rule_type == StyleRule::kProperty ||
       rule_type == StyleRule::kPositionTry ||
       rule_type == StyleRule::kFontFace || rule_type == StyleRule::kFunction ||
       rule_type == StyleRule::kCounterStyle ||
       rule_type == StyleRule::kFontPaletteValues)) {
    if (!id) {
      // If we skipped the relevant Consume*() calls above due to an invalid
      // property/descriptor, the inspector still needs to know the offset
      // where the would-be declaration ends.
      CSSVariableParser::ConsumeUnparsedDeclaration(
          stream, /*allow_important_annotation=*/true,
          /*is_animation_tainted=*/false,
          /*must_contain_variable_reference=*/false,
          /*restricted_value=*/true, /*comma_ends_declaration=*/false,
          important, *context_);
    }

    // There could be remnants of a broken !important declaration,
    // that neither ConsumeUnparsedDeclaration() nor MaybeConsumeImportant()
    // would consume, but which Devtools wants us to include.
    stream.SkipUntilPeekedTypeIs<kLeftBraceToken, kSemicolonToken>();

    // The end offset is the offset of the terminating token, which is peeked
    // but not yet consumed.
    observer_->ObserveProperty(decl_offset_start, stream.LookAheadOffset(),
                               important,
                               parsed_properties_.size() != properties_count);
  }

  return parsed_properties_.size() != properties_count;
}

bool CSSParserImpl::ConsumeVariableValue(CSSParserTokenStream& stream,
                                         const AtomicString& variable_name,
                                         bool allow_important_annotation,
                                         bool is_animation_tainted) {
  stream.EnsureLookAhead();

  // First, see if this is (only) a CSS-wide keyword.
  bool important;
  const CSSValue* value = CSSPropertyParser::ConsumeCSSWideKeyword(
      stream, *context_, allow_important_annotation, important);
  if (!value) {
    // It was not, so try to parse it as an unparsed declaration value
    // (which is pretty free-form).
    CSSVariableData* variable_data =
        CSSVariableParser::ConsumeUnparsedDeclaration(
            stream, allow_important_annotation, is_animation_tainted,
            /*must_contain_variable_reference=*/false,
            /*restricted_value=*/false, /*comma_ends_declaration=*/false,
            important, *context_);
    if (!variable_data) {
      return false;
    }

    value = MakeGarbageCollected<CSSUnparsedDeclarationValue>(variable_data,
                                                              context_);
  }
  parsed_properties_.push_back(
      CSSPropertyValue(CSSPropertyName(variable_name), *value, important));
  context_->Count(CSSPropertyID::kVariable);
  return true;
}

// NOTE: Leading whitespace must be stripped from the stream, since
// ParseValue() has the same requirement.
void CSSParserImpl::ConsumeDeclarationValue(CSSParserTokenStream& stream,
                                            CSSPropertyID unresolved_property,
                                            bool is_in_declaration_list,
                                            StyleRule::RuleType rule_type) {
  const bool allow_important_annotation = is_in_declaration_list &&
                                          rule_type != StyleRule::kKeyframe &&
                                          rule_type != StyleRule::kPositionTry;
  CSSPropertyParser::ParseValue(unresolved_property, allow_important_annotation,
                                stream, context_, parsed_properties_,
                                rule_type);
}

std::unique_ptr<Vector<KeyframeOffset>> CSSParserImpl::ConsumeKeyframeKeyList(
    const CSSParserContext* context,
    CSSParserTokenStream& stream) {
  std::unique_ptr<Vector<KeyframeOffset>> result =
      std::make_unique<Vector<KeyframeOffset>>();
  while (true) {
    stream.ConsumeWhitespace();
    const CSSParserToken& token = stream.Peek();
    if (token.GetType() == kPercentageToken && token.NumericValue() >= 0 &&
        token.NumericValue() <= 100) {
      result->push_back(KeyframeOffset(TimelineOffset::NamedRange::kNone,
                                       token.NumericValue() / 100));
      stream.ConsumeIncludingWhitespace();
    } else if (token.GetType() == kIdentToken) {
      if (EqualIgnoringAsciiCase(token.Value(), "from")) {
        result->push_back(KeyframeOffset(TimelineOffset::NamedRange::kNone, 0));
        stream.ConsumeIncludingWhitespace();
      } else if (EqualIgnoringAsciiCase(token.Value(), "to")) {
        result->push_back(KeyframeOffset(TimelineOffset::NamedRange::kNone, 1));
        stream.ConsumeIncludingWhitespace();
      } else {
        CSSParserLocalContext local_context =
            CSSParserLocalContext::CreateWithoutPropertyForAtRules();
        auto* stream_name_percent = To<CSSValueList>(
            css_parsing_utils::ConsumeTimelineRangeNameAndPercent(
                stream, *context, local_context));
        if (!stream_name_percent) {
          return nullptr;
        }

        auto stream_name = To<CSSIdentifierValue>(stream_name_percent->Item(0))
                               .ConvertTo<TimelineOffset::NamedRange>();
        double percent =
            To<CSSNumericLiteralValue>(stream_name_percent->Item(1))
                .ClampedDoubleValue();
        result->push_back(KeyframeOffset(stream_name, percent / 100.0));
      }
    } else {
      return nullptr;
    }

    if (stream.Peek().GetType() != kCommaToken) {
      return result;
    }
    stream.Consume();
  }
}

CSSParserMode CSSParserImpl::GetMode() const {
  return context_->Mode();
}

}  // namespace blink

```

## <a id="css-selector-cc"></a>css_selector.cc

Original: [complete immutable source](https://chromium.googlesource.com/chromium/src/+/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/css/css_selector.cc). Path: `third_party/blink/renderer/core/css/css_selector.cc`. Source bytes: 84107; source lines: 2423; SHA-256: `0836181aebb7b9138734aab0ea9990197acbd3cc8ecd0871c739ebc3c0842f56`.

```cpp
/*
 * Copyright (C) 1999-2003 Lars Knoll (knoll@kde.org)
 *               1999 Waldo Bastian (bastian@kde.org)
 *               2001 Andreas Schlapbach (schlpbch@iam.unibe.ch)
 *               2001-2003 Dirk Mueller (mueller@kde.org)
 * Copyright (C) 2002, 2006, 2007, 2008, 2009, 2010 Apple Inc. All rights
 * reserved.
 * Copyright (C) 2008 David Smith (catfish.man@gmail.com)
 * Copyright (C) 2010 Google Inc. All rights reserved.
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Library General Public
 * License as published by the Free Software Foundation; either
 * version 2 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Library General Public License for more details.
 *
 * You should have received a copy of the GNU Library General Public License
 * along with this library; see the file COPYING.LIB.  If not, write to
 * the Free Software Foundation, Inc., 51 Franklin Street, Fifth Floor,
 * Boston, MA 02110-1301, USA.
 */

#include "third_party/blink/renderer/core/css/css_selector.h"

#include <algorithm>
#include <iterator>
#include <memory>
#include <new>

#include "base/containers/span.h"
#include "base/strings/string_view_util.h"
#include "style_rule.h"
#include "third_party/blink/renderer/core/css/css_markup.h"
#include "third_party/blink/renderer/core/css/css_selector_list.h"
#include "third_party/blink/renderer/core/css/navigation_query.h"
#include "third_party/blink/renderer/core/css/parser/css_parser_context.h"
#include "third_party/blink/renderer/core/css/parser/css_selector_parser.h"
#include "third_party/blink/renderer/core/css/parser/css_tokenizer.h"
#include "third_party/blink/renderer/core/dom/document.h"
#include "third_party/blink/renderer/core/dom/pseudo_element.h"
#include "third_party/blink/renderer/core/execution_context/execution_context.h"
#include "third_party/blink/renderer/core/html/forms/html_select_element.h"
#include "third_party/blink/renderer/core/html/html_document.h"
#include "third_party/blink/renderer/core/html/html_install_element.h"
#include "third_party/blink/renderer/core/html_names.h"
#include "third_party/blink/renderer/core/script_tools/model_context.h"
#include "third_party/blink/renderer/platform/runtime_enabled_features.h"
#include "third_party/blink/renderer/platform/wtf/hash_map.h"
#include "third_party/blink/renderer/platform/wtf/size_assertions.h"
#include "third_party/blink/renderer/platform/wtf/std_lib_extras.h"
#include "third_party/blink/renderer/platform/wtf/text/string_builder.h"

#if DCHECK_IS_ON()
#include <stdio.h>
#endif  // DCHECK_IS_ON()

namespace blink {

namespace {

constexpr bool kExpandPseudoReferences = true;

unsigned MaximumSpecificity(const CSSSelectorList* list) {
  if (!list) {
    return 0;
  }
  return list->MaximumSpecificity();
}

}  // namespace

// Returns the maximum specificity across a selector list, only including
// the (complex) selectors for which the `predicate` returns true.
template <typename Predicate>
unsigned MaximumSpecificity(
    const CSSSelector* first_selector,
    Predicate predicate = [](const CSSSelector*) { return true; }) {
  unsigned specificity = 0;
  for (const CSSSelector* s = first_selector; s;
       s = CSSSelectorList::Next(*s)) {
    if (predicate(s)) {
      specificity = std::max(specificity, s->Specificity());
    }
  }
  return specificity;
}

struct SameSizeAsCSSSelector {
  unsigned bitfields;
  union {
    AtomicString value_;
    QualifiedName tag_q_name_or_attribute_;
    Member<void*> rare_data_;
  } pointers;
};

ASSERT_SIZE(CSSSelector, SameSizeAsCSSSelector);

CSSSelector::CSSSelector(MatchType match_type,
                         const QualifiedName& attribute,
                         AttributeMatchType case_sensitivity)
    : bits_(
          RelationField::encode(kSubSelector) | MatchField::encode(match_type) |
          PseudoTypeField::encode(kPseudoUnknown) |
          IsLastInSelectorListField::encode(false) |
          IsLastInComplexSelectorField::encode(false) |
          HasRareDataField::encode(false) | IsForPageField::encode(false) |
          IsImplicitlyAddedField::encode(false) |
          IsCoveredByBucketingField::encode(false) |
          AttributeMatchField::encode(static_cast<unsigned>(case_sensitivity)) |
          LegacyCaseInsensitiveMatchField::encode(
              !HTMLDocument::IsCaseSensitiveAttribute(attribute) &&
              case_sensitivity != AttributeMatchType::kCaseSensitiveAlways) |
          IsScopeContainingField::encode(false)),
      data_(attribute) {
  DCHECK_EQ(match_type, kAttributeSet);
}

CSSSelector::CSSSelector(MatchType match_type,
                         const QualifiedName& attribute,
                         AttributeMatchType case_sensitivity,
                         const AtomicString& value)
    : bits_(
          RelationField::encode(kSubSelector) |
          MatchField::encode(static_cast<unsigned>(match_type)) |
          PseudoTypeField::encode(kPseudoUnknown) |
          IsLastInSelectorListField::encode(false) |
          IsLastInComplexSelectorField::encode(false) |
          HasRareDataField::encode(true) | IsForPageField::encode(false) |
          IsImplicitlyAddedField::encode(false) |
          IsCoveredByBucketingField::encode(false) |
          AttributeMatchField::encode(static_cast<unsigned>(case_sensitivity)) |
          LegacyCaseInsensitiveMatchField::encode(
              !HTMLDocument::IsCaseSensitiveAttribute(attribute) &&
              case_sensitivity != AttributeMatchType::kCaseSensitiveAlways) |
          IsScopeContainingField::encode(false)),
      data_(MakeGarbageCollected<RareData>(value)) {
  DCHECK(IsAttributeSelector());
  data_.rare_data_->attribute_ = attribute;
}

// static
const AtomicString& CSSSelector::NameForInlineSelectorListPseudo(
    PseudoType pseudo_type) {
  DEFINE_STATIC_LOCAL(const AtomicString, is_atom, ("is"));
  DEFINE_STATIC_LOCAL(const AtomicString, where_atom, ("where"));
  DEFINE_STATIC_LOCAL(const AtomicString, not_atom, ("not"));
  DEFINE_STATIC_LOCAL(const AtomicString, has_atom, ("has"));
  switch (pseudo_type) {
    case kPseudoIs:
      return is_atom;
    case kPseudoWhere:
      return where_atom;
    case kPseudoNot:
      return not_atom;
    case kPseudoHas:
      return has_atom;
    default:
      NOTREACHED();
  }
}

void CSSSelector::CreateRareData() {
  DCHECK_NE(Match(), kTag);
  DCHECK_NE(Match(), kUniversalTag);
  if (HasRareData()) {
    return;
  }
  // This transitions the DataUnion from |value_| (or |selector_list_|) to
  // |rare_data_| and thus needs to be careful to correctly manage explicit
  // destruction of the old member followed by placement new of |rare_data_|.
  // A straight-assignment will compile and may kinda work, but will be
  // undefined behavior.
  if (HasInlineSelectorList()) {
    CSSSelectorList* selector_list = data_.selector_list_.Get();
    auto* rare_data = MakeGarbageCollected<RareData>(
        NameForInlineSelectorListPseudo(GetPseudoType()));
    rare_data->selector_list_ = selector_list;
    // Clear the tag bit _before_ touching the union, so that a concurrent
    // Oilpan marker (which dispatches on the tag bits, see Trace()) never
    // sees the RareData pointer as a CSSSelectorList. While neither bit is
    // set nothing is traced; both objects are kept alive by the stack
    // (conservative scanning) meanwhile.
    bits_.set<HasInlineSelectorListField>(false);
    data_.selector_list_.~Member<CSSSelectorList>();
    new (&data_.rare_data_) Member<RareData>(rare_data);
    bits_.set<HasRareDataField>(true);
    return;
  }
  auto* rare_data = MakeGarbageCollected<RareData>(data_.value_);
  data_.value_.~AtomicString();
  new (&data_.rare_data_) Member<RareData>(rare_data);
  bits_.set<HasRareDataField>(true);
}

unsigned CSSSelector::Specificity() const {
  if (IsForPage()) {
    return SpecificityForPage() & CSSSelector::kMaxValueMask;
  }

  unsigned total = 0;
  unsigned temp = 0;

  for (const CSSSelector* selector = this; selector;
       selector = selector->NextSimpleSelector()) {
    temp = total + selector->SpecificityForOneSelector();
    // Clamp each component to its max in the case of overflow.
    if ((temp & kIdMask) < (total & kIdMask)) {
      total |= kIdMask;
    } else if ((temp & kClassMask) < (total & kClassMask)) {
      total |= kClassMask;
    } else if ((temp & kElementMask) < (total & kElementMask)) {
      total |= kElementMask;
    } else {
      total = temp;
    }
  }
  return total;
}

std::array<uint8_t, 3> CSSSelector::SpecificityTuple() const {
  unsigned specificity = Specificity();

  uint8_t a = (specificity & kIdMask) >> 16;
  uint8_t b = (specificity & kClassMask) >> 8;
  uint8_t c = (specificity & kElementMask);

  return {a, b, c};
}

inline unsigned CSSSelector::SpecificityForOneSelector() const {
  // FIXME: Pseudo-elements and pseudo-classes do not have the same specificity.
  // This function isn't quite correct.
  // http://www.w3.org/TR/selectors/#specificity
  switch (Match()) {
    case kId:
      return kIdSpecificity;
    case kPseudoClass:
      switch (GetPseudoType()) {
        case kPseudoWhere:
          return 0;
        case kPseudoHost:
          if (!SelectorList()) {
            return kClassLikeSpecificity;
          }
          [[fallthrough]];
        case kPseudoHostContext:
          DCHECK(SelectorList()->IsSingleComplexSelector());
          return kClassLikeSpecificity + SelectorList()->First()->Specificity();
        case kPseudoNot:
          DCHECK(SelectorList());
          [[fallthrough]];
        case kPseudoIs:
          return MaximumSpecificity(SelectorList());
        case kPseudoHas:
          return MaximumSpecificity(SelectorList());
        case kPseudoParent:
          if (data_.parent_rule_ == nullptr) {
            // & in a non-nesting context matches nothing.
            return 0;
          }
          return MaximumSpecificity(
              data_.parent_rule_->FirstSelector(),
              [](const CSSSelector* selector) {
                return selector->IsAllowedInParentPseudo();
              });
        case kPseudoNthChild:
        case kPseudoNthLastChild:
          if (SelectorList()) {
            return kClassLikeSpecificity + MaximumSpecificity(SelectorList());
          } else {
            return kClassLikeSpecificity;
          }
        case kPseudoRelativeAnchor:
          return 0;
        case kPseudoScope:
          if (IsImplicit()) {
            // Implicit :scope pseudo-classes are added to selectors
            // within @scope. Such pseudo-classes must not have any effect
            // on the specificity of the scoped selector.
            //
            // https://drafts.csswg.org/css-cascade-6/#scope-effects
            return 0;
          }
          break;
        // FIXME: PseudoAny should base the specificity on the sub-selectors.
        // See http://lists.w3.org/Archives/Public/www-style/2010Sep/0530.html
        case kPseudoAny:
        default:
          break;
      }
      return kClassLikeSpecificity;
    case kPseudoElement:
      switch (GetPseudoType()) {
        case kPseudoSlotted:
          DCHECK(SelectorList()->IsSingleComplexSelector());
          return kTagSpecificity + SelectorList()->First()->Specificity();
        case kPseudoViewTransitionGroup:
        case kPseudoViewTransitionGroupChildren:
        case kPseudoViewTransitionImagePair:
        case kPseudoViewTransitionOld:
        case kPseudoViewTransitionNew: {
          CHECK(!IdentList().empty());
          return (IdentList().size() == 1u && IdentList()[0].IsNull())
                     ? 0
                     : kTagSpecificity;
        }
        case kPseudoHighlight:
          if (Argument() == UniversalSelectorAtom()) {
            return 0;
          }
          [[fallthrough]];
        default:
          return kTagSpecificity;
      }
    case kClass:
    case kAttributeExact:
    case kAttributeSet:
    case kAttributeList:
    case kAttributeHyphen:
    case kAttributeContain:
    case kAttributeBegin:
    case kAttributeEnd:
      return kClassLikeSpecificity;
    case kTag:
      return kTagSpecificity;
    case kUniversalTag:
      return 0;
    case kInvalidList:
    case kPagePseudoClass:
      NOTREACHED();
    case kUnknown:
      return 0;
  }
  NOTREACHED();
}

std::array<uint8_t, 3> CSSSelector::SimpleSelectorSpecificityTuple() const {
  unsigned specificity = SpecificityForOneSelector();

  uint8_t a = (specificity & kIdMask) >> 16;
  uint8_t b = (specificity & kClassMask) >> 8;
  uint8_t c = (specificity & kElementMask);

  return {a, b, c};
}

unsigned CSSSelector::SpecificityForPage() const {
  // See https://drafts.csswg.org/css-page/#cascading-and-page-context
  unsigned s = 0;

  for (const CSSSelector* component = this; component;
       component = component->NextSimpleSelector()) {
    switch (component->Match()) {
      case kTag:
        s += 4;
        break;
      case kPagePseudoClass:
        switch (component->GetPseudoType()) {
          case kPseudoFirstPage:
            s += 2;
            break;
          case kPseudoLeftPage:
          case kPseudoRightPage:
            s += 1;
            break;
          default:
            NOTREACHED();
        }
        break;
      default:
        break;
    }
  }
  return s;
}

PseudoId CSSSelector::GetPseudoId(PseudoType type) {
  switch (type) {
    case kPseudoFirstLine:
      return kPseudoIdFirstLine;
    case kPseudoFirstLetter:
      return kPseudoIdFirstLetter;
    case kPseudoSelection:
      return kPseudoIdSelection;
    case kPseudoCheckMark:
      return kPseudoIdCheckMark;
    case kPseudoBefore:
      return kPseudoIdBefore;
    case kPseudoAfter:
      return kPseudoIdAfter;
    case kPseudoExpandIcon:
      return kPseudoIdExpandIcon;
    case kPseudoPickerIcon:
      return kPseudoIdPickerIcon;
    case kPseudoInterestButton:
      return kPseudoIdInterestButton;
    case kPseudoMarker:
      return kPseudoIdMarker;
    case kPseudoBackdrop:
      return kPseudoIdBackdrop;
    case kPseudoScrollbar:
      return kPseudoIdScrollbar;
    case kPseudoScrollMarker:
      return kPseudoIdScrollMarker;
    case kPseudoScrollMarkerGroup:
      return kPseudoIdScrollMarkerGroup;
    case kPseudoScrollButton:
      return kPseudoIdScrollButton;
    case kPseudoColumn:
      return kPseudoIdColumn;
    case kPseudoScrollbarButton:
      return kPseudoIdScrollbarButton;
    case kPseudoScrollbarCorner:
      return kPseudoIdScrollbarCorner;
    case kPseudoScrollbarThumb:
      return kPseudoIdScrollbarThumb;
    case kPseudoScrollbarTrack:
      return kPseudoIdScrollbarTrack;
    case kPseudoScrollbarTrackPiece:
      return kPseudoIdScrollbarTrackPiece;
    case kPseudoResizer:
      return kPseudoIdResizer;
    case kPseudoSearchText:
      return kPseudoIdSearchText;
    case kPseudoTargetText:
      return kPseudoIdTargetText;
    case kPseudoHighlight:
      return kPseudoIdHighlight;
    case kPseudoSpellingError:
      return kPseudoIdSpellingError;
    case kPseudoGrammarError:
      return kPseudoIdGrammarError;
    case kPseudoPlaceholder:
      return kPseudoIdPlaceholder;
    case kPseudoFileSelectorButton:
      return kPseudoIdFileSelectorButton;
    case kPseudoDetailsContent:
      return kPseudoIdDetailsContent;
    case kPseudoPermissionIcon:
      return kPseudoIdPermissionIcon;
    case kPseudoPicker:
      // NOTE: When we support more than one argument to ::picker() we will
      // need to refactor something here (possibly the callers of this method)
      // to account for this.
      return kPseudoIdPickerSelect;
    case kPseudoSelectListbox:
      return kPseudoIdSelectListbox;
    case kPseudoViewTransition:
      return kPseudoIdViewTransition;
    case kPseudoViewTransitionGroup:
      return kPseudoIdViewTransitionGroup;
    case kPseudoViewTransitionGroupChildren:
      return kPseudoIdViewTransitionGroupChildren;
    case kPseudoViewTransitionImagePair:
      return kPseudoIdViewTransitionImagePair;
    case kPseudoViewTransitionOld:
      return kPseudoIdViewTransitionOld;
    case kPseudoViewTransitionNew:
      return kPseudoIdViewTransitionNew;
    case kPseudoOverscrollAreaParent:
      return kPseudoIdOverscrollAreaParent;
    case kPseudoOverscrollBackdrop:
      return kPseudoIdOverscrollBackdrop;
    case kPseudoSkeleton:
      return kPseudoIdSkeleton;
    case kPseudoAnimatedImage:
    case kPseudoActive:
    case kPseudoActiveOption:
    case kPseudoActiveViewTransition:
    case kPseudoActiveViewTransitionType:
    case kPseudoAny:
    case kPseudoAnyLink:
    case kPseudoAutofill:
    case kPseudoAutofillPreviewed:
    case kPseudoAutofillSelected:
    case kPseudoBlinkInternalElement:
    case kPseudoBuffering:
    case kPseudoChecked:
    case kPseudoCornerPresent:
    case kPseudoCue:
    case kPseudoCurrent:
    case kPseudoDecrement:
    case kPseudoDefault:
    case kPseudoDefined:
    case kPseudoDialogInTopLayer:
    case kPseudoDir:
    case kPseudoDisabled:
    case kPseudoDoubleButton:
    case kPseudoDrag:
    case kPseudoEmpty:
    case kPseudoEnabled:
    case kPseudoEnd:
    case kPseudoFiltered:
    case kPseudoFirstChild:
    case kPseudoFirstOfType:
    case kPseudoFirstPage:
    case kPseudoFocus:
    case kPseudoFocusVisible:
    case kPseudoFocusWithin:
    case kPseudoFullPageMedia:
    case kPseudoFullScreen:
    case kPseudoFullScreenAncestor:
    case kPseudoFullscreen:
    case kPseudoFutureCue:
    case kPseudoHas:
    case kPseudoHasSlotted:
    case kPseudoHasDatalist:
    case kPseudoHasOpenMenuitem:
    case kPseudoHorizontal:
    case kPseudoHost:
    case kPseudoHostContext:
    case kPseudoHostHasNonAutoAppearance:
    case kPseudoHover:
    case kPseudoInRange:
    case kPseudoIncrement:
    case kPseudoIndeterminate:
    case kPseudoInterestSource:
    case kPseudoInterestTarget:
    case kPseudoInvalid:
    case kPseudoIs:
    case kPseudoIsHtml:
    case kPseudoLang:
    case kPseudoLastChild:
    case kPseudoLastOfType:
    case kPseudoLeftPage:
    case kPseudoLink:
    case kPseudoLinkTo:
    case kPseudoListBox:
    case kPseudoMenulistPopoverWithMenulistAnchor:
    case kPseudoModal:
    case kPseudoMultiSelectFocus:
    case kPseudoMuted:
    case kPseudoNoButton:
    case kPseudoNot:
    case kPseudoNthChild:
    case kPseudoNthLastChild:
    case kPseudoNthLastOfType:
    case kPseudoNthOfType:
    case kPseudoOnlyChild:
    case kPseudoOnlyOfType:
    case kPseudoOpen:
    case kPseudoOptional:
    case kPseudoOutOfRange:
    case kPseudoOverscrollClosed:
    case kPseudoOverscrollOpen:
    case kPseudoParent:
    case kPseudoPart:
    case kPseudoPastCue:
    case kPseudoPaused:
    case kPseudoPermissionGranted:
    case kPseudoPictureInPicture:
    case kPseudoPlaceholderShown:
    case kPseudoPlaying:
    case kPseudoPopoverInTopLayer:
    case kPseudoPopoverOpen:
    case kPseudoReadOnly:
    case kPseudoReadWrite:
    case kPseudoRelativeAnchor:
    case kPseudoRequired:
    case kPseudoRightPage:
    case kPseudoRoot:
    case kPseudoScope:
    case kPseudoSeeking:
    case kPseudoSelectContainsInput:
    case kPseudoSelectHasSlottedButton:
    case kPseudoSingleButton:
    case kPseudoSlotted:
    case kPseudoSpatialNavigationFocus:
    case kPseudoStalled:
    case kPseudoStart:
    case kPseudoState:
    case kPseudoTarget:
    case kPseudoTargetCurrent:
    case kPseudoTargetBefore:
    case kPseudoTargetAfter:
    case kPseudoTextField:
    case kPseudoToolFormActive:
    case kPseudoToolSubmitActive:
    case kPseudoNavigationSource:
    case kPseudoUnknown:
    case kPseudoUnbounded:
    case kPseudoUnparsed:
    case kPseudoUserInvalid:
    case kPseudoUserValid:
    case kPseudoValid:
    case kPseudoVertical:
    case kPseudoVideoPersistent:
    case kPseudoVideoPersistentAncestor:
    case kPseudoVisited:
    case kPseudoVolumeLocked:
    case kPseudoWebKitAutofill:
    case kPseudoWebKitCustomElement:
    case kPseudoWebkitAnyLink:
    case kPseudoWhere:
    case kPseudoWindowInactive:
    case kPseudoXrOverlay:
      return kPseudoIdNone;
  }

  NOTREACHED();
}

std::optional<CSSSelector> CSSSelector::Renest(StyleRule* new_parent) const {
  if (GetPseudoType() == CSSSelector::kPseudoParent &&
      data_.parent_rule_ != new_parent) {
    CSSSelector selector(*this);
    selector.data_.parent_rule_ = new_parent;
    return selector;
  } else if (HasRareData()) {
    // Handles cases where simple selectors hold an inner selector list,
    // e.g. :is(), :where(), :not().
    RareData* old_rare_data = data_.rare_data_;
    RareData* new_rare_data = old_rare_data->Renest(new_parent);
    if (old_rare_data != new_rare_data) {
      CSSSelector selector(*this);
      selector.data_.rare_data_ = new_rare_data;
      return selector;
    }
  } else if (HasInlineSelectorList()) {
    CSSSelectorList* old_list = data_.selector_list_.Get();
    CSSSelectorList* new_list = old_list->Renest(new_parent);
    if (old_list != new_list) {
      CSSSelector selector(*this);
      selector.data_.selector_list_ = new_list;
      return selector;
    }
  }
  // Note that :scope (which isn't handled by any of the branches above)
  // does not need re-nesting, because it does not contain any reference
  // to a parent rule. The relationship between :scope, and the elements
  // matched by it, are instead handled dynamically at selector-matching time.

  return std::nullopt;
}

// Could be made smaller and faster by replacing pointer with an
// offset into a string buffer and making the bit fields smaller but
// that could not be maintained by hand.
struct NameToPseudoStruct {
  const char* string;
  unsigned type : 8;
};

// These tables must be kept sorted.
constexpr static NameToPseudoStruct kPseudoTypeWithoutArgumentsMap[] = {
    {"-internal-autofill-previewed", CSSSelector::kPseudoAutofillPreviewed},
    {"-internal-autofill-selected", CSSSelector::kPseudoAutofillSelected},
    {"-internal-dialog-in-top-layer", CSSSelector::kPseudoDialogInTopLayer},
    {"-internal-has-datalist", CSSSelector::kPseudoHasDatalist},
    {"-internal-has-open-menuitem", CSSSelector::kPseudoHasOpenMenuitem},
    {"-internal-is-html", CSSSelector::kPseudoIsHtml},
    {"-internal-list-box", CSSSelector::kPseudoListBox},
    {"-internal-media-controls-overlay-cast-button",
     CSSSelector::kPseudoWebKitCustomElement},
    {"-internal-menulist-popover-with-menulist-anchor",
     CSSSelector::kPseudoMenulistPopoverWithMenulistAnchor},
    {"-internal-multi-select-focus", CSSSelector::kPseudoMultiSelectFocus},
    {"-internal-popover-in-top-layer", CSSSelector::kPseudoPopoverInTopLayer},
    {"-internal-relative-anchor", CSSSelector::kPseudoRelativeAnchor},
    {"-internal-select-contains-input",
     CSSSelector::kPseudoSelectContainsInput},
    {"-internal-select-has-slotted-button",
     CSSSelector::kPseudoSelectHasSlottedButton},
    {"-internal-shadow-host-has-non-auto-appearance",
     CSSSelector::kPseudoHostHasNonAutoAppearance},
    {"-internal-spatial-navigation-focus",
     CSSSelector::kPseudoSpatialNavigationFocus},
    {"-internal-text-field", CSSSelector::kPseudoTextField},
    {"-internal-video-persistent", CSSSelector::kPseudoVideoPersistent},
    {"-internal-video-persistent-ancestor",
     CSSSelector::kPseudoVideoPersistentAncestor},
    {"-webkit-any-link", CSSSelector::kPseudoWebkitAnyLink},
    {"-webkit-autofill", CSSSelector::kPseudoWebKitAutofill},
    {"-webkit-drag", CSSSelector::kPseudoDrag},
    {"-webkit-full-page-media", CSSSelector::kPseudoFullPageMedia},
    {"-webkit-full-screen", CSSSelector::kPseudoFullScreen},
    {"-webkit-full-screen-ancestor", CSSSelector::kPseudoFullScreenAncestor},
    {"-webkit-resizer", CSSSelector::kPseudoResizer},
    {"-webkit-scrollbar", CSSSelector::kPseudoScrollbar},
    {"-webkit-scrollbar-button", CSSSelector::kPseudoScrollbarButton},
    {"-webkit-scrollbar-corner", CSSSelector::kPseudoScrollbarCorner},
    {"-webkit-scrollbar-thumb", CSSSelector::kPseudoScrollbarThumb},
    {"-webkit-scrollbar-track", CSSSelector::kPseudoScrollbarTrack},
    {"-webkit-scrollbar-track-piece", CSSSelector::kPseudoScrollbarTrackPiece},
    {"active", CSSSelector::kPseudoActive},
    {"active-option", CSSSelector::kPseudoActiveOption},
    {"active-view-transition", CSSSelector::kPseudoActiveViewTransition},
    {"after", CSSSelector::kPseudoAfter},
    {"animated-image", CSSSelector::kPseudoAnimatedImage},
    {"any-link", CSSSelector::kPseudoAnyLink},
    {"autofill", CSSSelector::kPseudoAutofill},
    {"backdrop", CSSSelector::kPseudoBackdrop},
    {"before", CSSSelector::kPseudoBefore},
    {"buffering", CSSSelector::kPseudoBuffering},
    {"checked", CSSSelector::kPseudoChecked},
    {"checkmark", CSSSelector::kPseudoCheckMark},
    {"column", CSSSelector::kPseudoColumn},
    {"corner-present", CSSSelector::kPseudoCornerPresent},
    {"cue", CSSSelector::kPseudoWebKitCustomElement},
    {"current", CSSSelector::kPseudoCurrent},
    {"decrement", CSSSelector::kPseudoDecrement},
    {"default", CSSSelector::kPseudoDefault},
    {"defined", CSSSelector::kPseudoDefined},
    {"details-content", CSSSelector::kPseudoDetailsContent},
    {"disabled", CSSSelector::kPseudoDisabled},
    {"double-button", CSSSelector::kPseudoDoubleButton},
    {"empty", CSSSelector::kPseudoEmpty},
    {"enabled", CSSSelector::kPseudoEnabled},
    {"end", CSSSelector::kPseudoEnd},
    {"expand-icon", CSSSelector::kPseudoExpandIcon},
    {"file-selector-button", CSSSelector::kPseudoFileSelectorButton},
    {"filtered", CSSSelector::kPseudoFiltered},
    {"first", CSSSelector::kPseudoFirstPage},
    {"first-child", CSSSelector::kPseudoFirstChild},
    {"first-letter", CSSSelector::kPseudoFirstLetter},
    {"first-line", CSSSelector::kPseudoFirstLine},
    {"first-of-type", CSSSelector::kPseudoFirstOfType},
    {"focus", CSSSelector::kPseudoFocus},
    {"focus-visible", CSSSelector::kPseudoFocusVisible},
    {"focus-within", CSSSelector::kPseudoFocusWithin},
    {"fullscreen", CSSSelector::kPseudoFullscreen},
    {"future", CSSSelector::kPseudoFutureCue},
    {"grammar-error", CSSSelector::kPseudoGrammarError},
    {"granted", CSSSelector::kPseudoPermissionGranted},
    {"has-slotted", CSSSelector::kPseudoHasSlotted},
    {"horizontal", CSSSelector::kPseudoHorizontal},
    {"host", CSSSelector::kPseudoHost},
    {"hover", CSSSelector::kPseudoHover},
    {"in-range", CSSSelector::kPseudoInRange},
    {"increment", CSSSelector::kPseudoIncrement},
    {"indeterminate", CSSSelector::kPseudoIndeterminate},
    {"interest-button", CSSSelector::kPseudoInterestButton},
    {"interest-source", CSSSelector::kPseudoInterestSource},
    {"interest-target", CSSSelector::kPseudoInterestTarget},
    {"invalid", CSSSelector::kPseudoInvalid},
    {"last-child", CSSSelector::kPseudoLastChild},
    {"last-of-type", CSSSelector::kPseudoLastOfType},
    {"left", CSSSelector::kPseudoLeftPage},
    {"link", CSSSelector::kPseudoLink},
    {"marker", CSSSelector::kPseudoMarker},
    {"modal", CSSSelector::kPseudoModal},
    {"muted", CSSSelector::kPseudoMuted},
    {"navigation-source", CSSSelector::kPseudoNavigationSource},
    {"no-button", CSSSelector::kPseudoNoButton},
    {"only-child", CSSSelector::kPseudoOnlyChild},
    {"only-of-type", CSSSelector::kPseudoOnlyOfType},
    {"open", CSSSelector::kPseudoOpen},
    {"optional", CSSSelector::kPseudoOptional},
    {"out-of-range", CSSSelector::kPseudoOutOfRange},
    {"overscroll-backdrop", CSSSelector::kPseudoOverscrollBackdrop},
    {"overscroll-closed", CSSSelector::kPseudoOverscrollClosed},
    {"overscroll-open", CSSSelector::kPseudoOverscrollOpen},
    {"past", CSSSelector::kPseudoPastCue},
    {"paused", CSSSelector::kPseudoPaused},
    {"permission-icon", CSSSelector::kPseudoPermissionIcon},
    {"picker-icon", CSSSelector::kPseudoPickerIcon},
    {"picture-in-picture", CSSSelector::kPseudoPictureInPicture},
    {"placeholder", CSSSelector::kPseudoPlaceholder},
    {"placeholder-shown", CSSSelector::kPseudoPlaceholderShown},
    {"playing", CSSSelector::kPseudoPlaying},
    {"popover-open", CSSSelector::kPseudoPopoverOpen},
    {"read-only", CSSSelector::kPseudoReadOnly},
    {"read-write", CSSSelector::kPseudoReadWrite},
    {"required", CSSSelector::kPseudoRequired},
    {"right", CSSSelector::kPseudoRightPage},
    {"root", CSSSelector::kPseudoRoot},
    {"scope", CSSSelector::kPseudoScope},
    {"scroll-marker", CSSSelector::kPseudoScrollMarker},
    {"scroll-marker-group", CSSSelector::kPseudoScrollMarkerGroup},
    {"search-text", CSSSelector::kPseudoSearchText},
    {"seeking", CSSSelector::kPseudoSeeking},
    {"select-listbox", CSSSelector::kPseudoSelectListbox},
    {"selection", CSSSelector::kPseudoSelection},
    {"single-button", CSSSelector::kPseudoSingleButton},
    {"skeleton", CSSSelector::kPseudoSkeleton},
    {"spelling-error", CSSSelector::kPseudoSpellingError},
    {"stalled", CSSSelector::kPseudoStalled},
    {"start", CSSSelector::kPseudoStart},
    {"target", CSSSelector::kPseudoTarget},
    {"target-after", CSSSelector::kPseudoTargetAfter},
    {"target-before", CSSSelector::kPseudoTargetBefore},
    {"target-current", CSSSelector::kPseudoTargetCurrent},
    {"target-text", CSSSelector::kPseudoTargetText},
    {"tool-form-active", CSSSelector::kPseudoToolFormActive},
    {"tool-submit-active", CSSSelector::kPseudoToolSubmitActive},
    {"unbounded", CSSSelector::kPseudoUnbounded},
    {"user-invalid", CSSSelector::kPseudoUserInvalid},
    {"user-valid", CSSSelector::kPseudoUserValid},
    {"valid", CSSSelector::kPseudoValid},
    {"vertical", CSSSelector::kPseudoVertical},
    {"view-transition", CSSSelector::kPseudoViewTransition},
    {"visited", CSSSelector::kPseudoVisited},
    {"volume-locked", CSSSelector::kPseudoVolumeLocked},
    {"window-inactive", CSSSelector::kPseudoWindowInactive},
    {"xr-overlay", CSSSelector::kPseudoXrOverlay},
};

constexpr static NameToPseudoStruct kPseudoTypeWithArgumentsMap[] = {
    {"-internal-overscroll-area-parent",
     CSSSelector::kPseudoOverscrollAreaParent},
    {"-webkit-any", CSSSelector::kPseudoAny},
    {"active-view-transition-type",
     CSSSelector::kPseudoActiveViewTransitionType},
    {"cue", CSSSelector::kPseudoCue},
    {"dir", CSSSelector::kPseudoDir},
    {"has", CSSSelector::kPseudoHas},
    {"highlight", CSSSelector::kPseudoHighlight},
    {"host", CSSSelector::kPseudoHost},
    {"host-context", CSSSelector::kPseudoHostContext},
    {"is", CSSSelector::kPseudoIs},
    {"lang", CSSSelector::kPseudoLang},
    {"link-to", CSSSelector::kPseudoLinkTo},
    {"not", CSSSelector::kPseudoNot},
    {"nth-child", CSSSelector::kPseudoNthChild},
    {"nth-last-child", CSSSelector::kPseudoNthLastChild},
    {"nth-last-of-type", CSSSelector::kPseudoNthLastOfType},
    {"nth-of-type", CSSSelector::kPseudoNthOfType},
    {"part", CSSSelector::kPseudoPart},
    {"picker", CSSSelector::kPseudoPicker},
    {"scroll-button", CSSSelector::kPseudoScrollButton},
    {"slotted", CSSSelector::kPseudoSlotted},
    {"state", CSSSelector::kPseudoState},
    {"view-transition-group", CSSSelector::kPseudoViewTransitionGroup},
    {"view-transition-group-children",
     CSSSelector::kPseudoViewTransitionGroupChildren},
    {"view-transition-image-pair", CSSSelector::kPseudoViewTransitionImagePair},
    {"view-transition-new", CSSSelector::kPseudoViewTransitionNew},
    {"view-transition-old", CSSSelector::kPseudoViewTransitionOld},
    {"where", CSSSelector::kPseudoWhere},
};

// TODO(sesse): This function should probably be gperf-generated, like
// everything else converting strings to enums, instead of hand-coded.
CSSSelector::PseudoType CSSSelector::NameToPseudoType(
    StringView name,
    bool has_arguments,
    const Document* document) {
  if (name.IsNull()) {
    return CSSSelector::kPseudoUnknown;
  }
  if (!name.Is8Bit()) {
    Vector<LChar, 50> latin1_name;
    for (UChar ch : name.Span16()) {
      if (ch > 0xFF) {
        return CSSSelector::kPseudoUnknown;
      }
      latin1_name.push_back(ch);
    }
    return NameToPseudoType(StringView(base::span(latin1_name)), has_arguments,
                            document);
  }

  const NameToPseudoStruct* pseudo_type_map;
  const NameToPseudoStruct* pseudo_type_map_end;
  if (has_arguments) {
    pseudo_type_map = std::begin(kPseudoTypeWithArgumentsMap);
    pseudo_type_map_end = std::end(kPseudoTypeWithArgumentsMap);
  } else {
    pseudo_type_map = std::begin(kPseudoTypeWithoutArgumentsMap);
    pseudo_type_map_end = std::end(kPseudoTypeWithoutArgumentsMap);
  }
  DCHECK(name.Is8Bit());
  std::string_view latin1_name = base::as_string_view(name.Span8());
  const NameToPseudoStruct* match =
      std::lower_bound(pseudo_type_map, pseudo_type_map_end, latin1_name,
                       [](const NameToPseudoStruct& entry,
                          const std::string_view& latin1_name) -> bool {
                         DCHECK(entry.string);
                         return std::string_view(entry.string) < latin1_name;
                       });
  if (match == pseudo_type_map_end ||
      std::string_view(match->string) != latin1_name) {
    return CSSSelector::kPseudoUnknown;
  }

  if ((match->type == CSSSelector::kPseudoPlaying ||
       match->type == CSSSelector::kPseudoPaused ||
       match->type == CSSSelector::kPseudoSeeking ||
       match->type == CSSSelector::kPseudoBuffering ||
       match->type == CSSSelector::kPseudoStalled ||
       match->type == CSSSelector::kPseudoMuted ||
       match->type == CSSSelector::kPseudoVolumeLocked) &&
      !RuntimeEnabledFeatures::CSSMediaElementPseudosEnabled()) {
    return CSSSelector::kPseudoUnknown;
  }

  if (match->type == CSSSelector::kPseudoPermissionGranted &&
      !RuntimeEnabledFeatures::GeolocationElementEnabled(
          document ? document->GetExecutionContext() : nullptr) &&
      !RuntimeEnabledFeatures::UserMediaElementEnabled(
          document ? document->GetExecutionContext() : nullptr) &&
      !RuntimeEnabledFeatures::InstallElementEnabled(
          document ? document->GetExecutionContext() : nullptr)) {
    return CSSSelector::kPseudoUnknown;
  }

  if (match->type == CSSSelector::kPseudoTargetCurrent &&
      !RuntimeEnabledFeatures::CSSPseudoScrollMarkersEnabled()) {
    return CSSSelector::kPseudoUnknown;
  }

  if ((match->type == CSSSelector::kPseudoTargetBefore ||
       match->type == CSSSelector::kPseudoTargetAfter) &&
      !RuntimeEnabledFeatures::CSSScrollMarkerTargetBeforeAfterEnabled()) {
    return CSSSelector::kPseudoUnknown;
  }

  if ((match->type == CSSSelector::kPseudoScrollMarker ||
       match->type == CSSSelector::kPseudoScrollMarkerGroup) &&
      !RuntimeEnabledFeatures::CSSPseudoScrollMarkersEnabled()) {
    return CSSSelector::kPseudoUnknown;
  }

  if (match->type == CSSSelector::kPseudoScrollButton &&
      !RuntimeEnabledFeatures::CSSPseudoScrollButtonsEnabled()) {
    return CSSSelector::kPseudoUnknown;
  }

  if (match->type == CSSSelector::kPseudoColumn &&
      !RuntimeEnabledFeatures::CSSPseudoColumnEnabled()) {
    return CSSSelector::kPseudoUnknown;
  }

  if ((match->type == CSSSelector::kPseudoSearchText ||
       match->type == CSSSelector::kPseudoCurrent) &&
      !RuntimeEnabledFeatures::SearchTextHighlightPseudoEnabled()) {
    return CSSSelector::kPseudoUnknown;
  }

  if ((match->type == CSSSelector::kPseudoToolFormActive ||
       match->type == CSSSelector::kPseudoToolSubmitActive) &&
      document &&
      !ModelContext::IsDeclarativeWebMCPEnabled(
          document->GetExecutionContext())) {
    return CSSSelector::kPseudoUnknown;
  }

  if (match->type == CSSSelector::kPseudoUnbounded &&
      !RuntimeEnabledFeatures::UnboundedElementEnabled()) {
    return CSSSelector::kPseudoUnknown;
  }

  if (match->type == CSSSelector::kPseudoHasSlotted &&
      !RuntimeEnabledFeatures::CSSPseudoHasSlottedEnabled()) {
    return CSSSelector::kPseudoUnknown;
  }

  if ((match->type == CSSSelector::kPseudoOverscrollAreaParent ||
       match->type == CSSSelector::kPseudoOverscrollBackdrop ||
       match->type == CSSSelector::kPseudoOverscrollClosed ||
       match->type == CSSSelector::kPseudoOverscrollOpen) &&
      !RuntimeEnabledFeatures::OverscrollAreasEnabled(
          document ? document->GetExecutionContext() : nullptr)) {
    return CSSSelector::kPseudoUnknown;
  }

  if ((match->type == CSSSelector::kPseudoActiveOption ||
       match->type == CSSSelector::kPseudoFiltered) &&
      !RuntimeEnabledFeatures::CustomizableComboboxEnabled() &&
      !RuntimeEnabledFeatures::FilterableSelectEnabled()) {
    return CSSSelector::kPseudoUnknown;
  }
  if (match->type == CSSSelector::kPseudoAnimatedImage &&
      !RuntimeEnabledFeatures::CSSImageAnimationEnabled()) {
    return CSSSelector::kPseudoUnknown;
  }

  if (match->type == CSSSelector::kPseudoExpandIcon &&
      !RuntimeEnabledFeatures::MenuElementsEnabled()) {
    return CSSSelector::kPseudoUnknown;
  }

  if (match->type == CSSSelector::kPseudoNavigationSource &&
      !RuntimeEnabledFeatures::NavigationSourcePseudoClassEnabled()) {
    return CSSSelector::kPseudoUnknown;
  }

  if (match->type == CSSSelector::kPseudoSkeleton &&
      !RuntimeEnabledFeatures::DeclarativeSkeletonsEnabled()) {
    return CSSSelector::kPseudoUnknown;
  }

  return static_cast<CSSSelector::PseudoType>(match->type);
}

#if DCHECK_IS_ON()
void CSSSelector::Show(int indent) const {
  printf("%*sSelectorText(): %s\n", indent, "", SelectorText().Ascii().c_str());
  printf("%*smatch_: %d\n", indent, "", Match());
  if (Match() != kTag && Match() != kUniversalTag && !IsPseudoParent()) {
    printf("%*sValue(): %s\n", indent, "", Value().Ascii().c_str());
  }
  printf("%*sGetPseudoType(): %d\n", indent, "", GetPseudoType());
  if (Match() == kTag && Match() != kUniversalTag) {
    printf("%*sTagQName().LocalName(): %s\n", indent, "",
           TagQName().LocalName().Ascii().c_str());
  }
  printf("%*sIsAttributeSelector(): %d\n", indent, "", IsAttributeSelector());
  if (IsAttributeSelector()) {
    printf("%*sAttribute(): %s\n", indent, "",
           Attribute().LocalName().Ascii().c_str());
  }
  printf("%*sArgument(): %s\n", indent, "", Argument().Ascii().c_str());
  printf("%*sSpecificity(): %u\n", indent, "", Specificity());
  if (NextSimpleSelector()) {
    printf("\n%*s--> (Relation() == %d)\n", indent, "", Relation());
    NextSimpleSelector()->Show(indent + 2);
  } else {
    printf("\n%*s--> (Relation() == %d)\n", indent, "", Relation());
  }
}

void CSSSelector::Show() const {
  printf("\n******* CSSSelector::Show(\"%s\") *******\n",
         SelectorText().Ascii().c_str());
  Show(2);
  printf("******* end *******\n");
}
#endif  // DCHECK_IS_ON()

void CSSSelector::UpdatePseudoPage(const AtomicString& value,
                                   const Document* document) {
  DCHECK_EQ(Match(), kPagePseudoClass);
  SetValue(value);
  PseudoType type = CSSSelectorParser::ParsePseudoType(value, false, document);
  if (type != kPseudoFirstPage && type != kPseudoLeftPage &&
      type != kPseudoRightPage) {
    type = kPseudoUnknown;
  }
  bits_.set<PseudoTypeField>(type);
}

void CSSSelector::UpdatePseudoType(AtomicString value,
                                   const CSSParserContext& context,
                                   bool has_arguments,
                                   CSSParserMode mode) {
  DCHECK(Match() == kPseudoClass || Match() == kPseudoElement);
  if (!value.ContainsNoAsciiUpper()) [[unlikely]] {
    value = value.ToAsciiLower();
  }
  PseudoType pseudo_type = CSSSelectorParser::ParsePseudoType(
      value, has_arguments, context.GetDocument());
  SetPseudoType(pseudo_type);
  SetValue(std::move(value));

  switch (GetPseudoType()) {
    case kPseudoAfter:
    case kPseudoBefore:
    case kPseudoFirstLetter:
    case kPseudoFirstLine:
      // The spec says some pseudos allow both single and double colons like
      // :before for backwards compatibility. Single colon becomes PseudoClass,
      // but should be PseudoElement like double colon.
      if (Match() == kPseudoClass) {
        bits_.set<MatchField>(kPseudoElement);
      }
      [[fallthrough]];
    // For pseudo-elements
    case kPseudoExpandIcon:
    case kPseudoPickerIcon:
    case kPseudoInterestButton:
    case kPseudoCheckMark:
    case kPseudoBackdrop:
    case kPseudoOverscrollBackdrop:
    case kPseudoCue:
    case kPseudoMarker:
    case kPseudoPart:
    case kPseudoPlaceholder:
    case kPseudoFileSelectorButton:
    case kPseudoResizer:
    case kPseudoScrollbar:
    case kPseudoScrollbarCorner:
    case kPseudoScrollbarButton:
    case kPseudoScrollbarThumb:
    case kPseudoScrollbarTrack:
    case kPseudoScrollbarTrackPiece:
    case kPseudoScrollMarker:
    case kPseudoScrollMarkerGroup:
    case kPseudoScrollButton:
    case kPseudoColumn:
    case kPseudoPicker:
    case kPseudoSelectListbox:
    case kPseudoSelection:
    case kPseudoWebKitCustomElement:
    case kPseudoSlotted:
    case kPseudoSearchText:
    case kPseudoTargetText:
    case kPseudoHighlight:
    case kPseudoSpellingError:
    case kPseudoGrammarError:
    case kPseudoViewTransition:
    case kPseudoViewTransitionGroup:
    case kPseudoViewTransitionGroupChildren:
    case kPseudoViewTransitionImagePair:
    case kPseudoViewTransitionOld:
    case kPseudoViewTransitionNew:
    case kPseudoDetailsContent:
      if (Match() != kPseudoElement) {
        bits_.set<PseudoTypeField>(kPseudoUnknown);
      }
      break;
    case kPseudoPermissionIcon:
      if (Match() != kPseudoElement) {
        bits_.set<PseudoTypeField>(kPseudoUnknown);
      }
      break;
    case kPseudoOverscrollAreaParent:
    case kPseudoBlinkInternalElement:
      if (Match() != kPseudoElement || mode != kUASheetMode) {
        bits_.set<PseudoTypeField>(kPseudoUnknown);
      }
      break;
    case kPseudoSkeleton:
      if (Match() != kPseudoElement) {
        bits_.set<PseudoTypeField>(kPseudoUnknown);
      }
      break;
    case kPseudoHasDatalist:
    case kPseudoHasOpenMenuitem:
    case kPseudoHostHasNonAutoAppearance:
    case kPseudoIsHtml:
    case kPseudoListBox:
    case kPseudoMultiSelectFocus:
    case kPseudoSelectContainsInput:
    case kPseudoSpatialNavigationFocus:
    case kPseudoVideoPersistent:
    case kPseudoVideoPersistentAncestor:
      if (mode != kUASheetMode) {
        bits_.set<PseudoTypeField>(kPseudoUnknown);
        break;
      }
      [[fallthrough]];
    // For pseudo-classes
    case kPseudoActive:
    case kPseudoActiveOption:
    case kPseudoActiveViewTransition:
    case kPseudoActiveViewTransitionType:
    case kPseudoAnimatedImage:
    case kPseudoAny:
    case kPseudoAnyLink:
    case kPseudoAutofill:
    case kPseudoAutofillPreviewed:
    case kPseudoAutofillSelected:
    case kPseudoBuffering:
    case kPseudoChecked:
    case kPseudoCornerPresent:
    case kPseudoCurrent:
    case kPseudoDecrement:
    case kPseudoDefault:
    case kPseudoDefined:
    case kPseudoDialogInTopLayer:
    case kPseudoDir:
    case kPseudoDisabled:
    case kPseudoDoubleButton:
    case kPseudoDrag:
    case kPseudoEmpty:
    case kPseudoEnabled:
    case kPseudoEnd:
    case kPseudoFiltered:
    case kPseudoFirstChild:
    case kPseudoFirstOfType:
    case kPseudoFocus:
    case kPseudoFocusVisible:
    case kPseudoFocusWithin:
    case kPseudoFullPageMedia:
    case kPseudoFullScreen:
    case kPseudoFullScreenAncestor:
    case kPseudoFullscreen:
    case kPseudoFutureCue:
    case kPseudoHas:
    case kPseudoHasSlotted:
    case kPseudoHorizontal:
    case kPseudoHost:
    case kPseudoHostContext:
    case kPseudoHover:
    case kPseudoInRange:
    case kPseudoIncrement:
    case kPseudoIndeterminate:
    case kPseudoInterestSource:
    case kPseudoInterestTarget:
    case kPseudoInvalid:
    case kPseudoIs:
    case kPseudoLang:
    case kPseudoLastChild:
    case kPseudoLastOfType:
    case kPseudoLink:
    case kPseudoLinkTo:
    case kPseudoMenulistPopoverWithMenulistAnchor:
    case kPseudoModal:
    case kPseudoMuted:
    case kPseudoNavigationSource:
    case kPseudoNoButton:
    case kPseudoNot:
    case kPseudoNthChild:
    case kPseudoNthLastChild:
    case kPseudoNthLastOfType:
    case kPseudoNthOfType:
    case kPseudoOnlyChild:
    case kPseudoOnlyOfType:
    case kPseudoOpen:
    case kPseudoOptional:
    case kPseudoOutOfRange:
    case kPseudoOverscrollClosed:
    case kPseudoOverscrollOpen:
    case kPseudoParent:
    case kPseudoPastCue:
    case kPseudoPaused:
    case kPseudoPermissionGranted:
    case kPseudoPictureInPicture:
    case kPseudoPlaceholderShown:
    case kPseudoPlaying:
    case kPseudoPopoverInTopLayer:
    case kPseudoPopoverOpen:
    case kPseudoReadOnly:
    case kPseudoReadWrite:
    case kPseudoRelativeAnchor:
    case kPseudoRequired:
    case kPseudoRoot:
    case kPseudoScope:
    case kPseudoSeeking:
    case kPseudoSelectHasSlottedButton:
    case kPseudoSingleButton:
    case kPseudoStalled:
    case kPseudoStart:
    case kPseudoState:
    case kPseudoTarget:
    case kPseudoTargetCurrent:
    case kPseudoTargetBefore:
    case kPseudoTargetAfter:
    case kPseudoTextField:
    case kPseudoUnknown:
    case kPseudoUnbounded:
    case kPseudoUnparsed:
    case kPseudoUserInvalid:
    case kPseudoUserValid:
    case kPseudoValid:
    case kPseudoVertical:
    case kPseudoVisited:
    case kPseudoVolumeLocked:
    case kPseudoWebKitAutofill:
    case kPseudoWebkitAnyLink:
    case kPseudoWhere:
    case kPseudoWindowInactive:
    case kPseudoXrOverlay:
    case kPseudoToolFormActive:
    case kPseudoToolSubmitActive:
      if (Match() != kPseudoClass) {
        bits_.set<PseudoTypeField>(kPseudoUnknown);
      }
      break;
    case kPseudoFirstPage:
    case kPseudoLeftPage:
    case kPseudoRightPage:
      bits_.set<PseudoTypeField>(kPseudoUnknown);
      break;
  }
}

void CSSSelector::SetUnparsedPlaceholder(CSSNestingType unparsed_nesting_type,
                                         const AtomicString& value) {
  DCHECK(Match() == kPseudoClass);
  SetPseudoType(kPseudoUnparsed);
  CreateRareData();
  SetValue(value);
  data_.rare_data_->bits_.unparsed_nesting_type_ = unparsed_nesting_type;
}

CSSNestingType CSSSelector::GetNestingType() const {
  switch (GetPseudoType()) {
    case CSSSelector::kPseudoParent:
      return CSSNestingType::kNesting;
    case CSSSelector::kPseudoUnparsed:
      return data_.rare_data_->bits_.unparsed_nesting_type_;
    case CSSSelector::kPseudoScope:
      // TODO(crbug.com/1280240): Handle unparsed :scope.
      return CSSNestingType::kScope;
    default:
      return CSSNestingType::kNone;
  }
}

void CSSSelector::SetWhere(CSSSelectorList* selector_list) {
  SetMatch(kPseudoClass);
  SetPseudoType(kPseudoWhere);
  SetSelectorList(selector_list);
}

static void SerializeIdentifierOrAny(const AtomicString& identifier,
                                     const AtomicString& any,
                                     StringBuilder& builder) {
  if (identifier != any) {
    SerializeIdentifier(identifier, builder);
  } else {
    builder.Append(g_star_atom);
  }
}

static void SerializeNamespacePrefixIfNeeded(const AtomicString& prefix,
                                             const AtomicString& any,
                                             StringBuilder& builder,
                                             bool is_attribute_selector) {
  if (prefix.IsNull() || (prefix.empty() && is_attribute_selector)) {
    return;
  }
  SerializeIdentifierOrAny(prefix, any, builder);
  builder.Append('|');
}

template <typename ListType>
static void SerializeIdentifierList(StringBuilder& builder,
                                    const ListType& list) {
  bool is_first = true;
  for (const AtomicString& item : list) {
    if (!is_first) {
      builder.Append(", ");
    }
    SerializeIdentifier(item, builder);
    is_first = false;
  }
}

// static
template <bool expand_pseudo_references>
void CSSSelector::SerializeSelectorList(const CSSSelectorList* selector_list,
                                        StringBuilder& builder,
                                        uintptr_t scope_id) {
  const CSSSelector* first_sub_selector =
      selector_list->FirstIncludingUnparsedInvalid();
  for (const CSSSelector* sub_selector = first_sub_selector; sub_selector;
       sub_selector =
           CSSSelectorList::NextIncludingUnparsedInvalid(*sub_selector)) {
    if (sub_selector != first_sub_selector) {
      builder.Append(", ");
    }
    builder.Append(
        sub_selector->SelectorTextInternal<expand_pseudo_references>(scope_id));
  }
}

String CSSSelector::SelectorText() const {
  // The value of `scope_id` does not matter when
  // `expand_pseudo_references` is `false`.
  return SelectorTextInternal<!kExpandPseudoReferences>(/*scope_id=*/0);
}

String CSSSelector::SelectorTextExpandingPseudoReferences(
    uintptr_t scope_id) const {
  return SelectorTextInternal<kExpandPseudoReferences>(scope_id);
}

template <bool expand_pseudo_references>
void CSSSelector::SerializeSimpleSelector(StringBuilder& builder,
                                          uintptr_t scope_id) const {
  bool suppress_selector_list = false;
  if ((Match() == kTag || Match() == kUniversalTag) && !IsImplicit()) {
    SerializeNamespacePrefixIfNeeded(TagQName().Prefix(), g_star_atom, builder,
                                     IsAttributeSelector());
    SerializeIdentifierOrAny(TagQName().LocalName(), UniversalSelectorAtom(),
                             builder);
  } else if (Match() == kId) {
    builder.Append('#');
    SerializeIdentifier(SerializingValue(), builder);
  } else if (Match() == kClass) {
    builder.Append('.');
    SerializeIdentifier(SerializingValue(), builder);
  } else if (Match() == kInvalidList && IsUnparsedInvalid()) {
    builder.Append(Value());
  } else if (Match() == kPseudoClass || Match() == kPagePseudoClass) {
    if (GetPseudoType() == kPseudoUnparsed) {
      builder.Append(Value());
    } else if (GetPseudoType() != kPseudoParent &&
               GetPseudoType() != kPseudoScope) {
      builder.Append(':');
      builder.Append(SerializingValue());
    }

    switch (GetPseudoType()) {
      case kPseudoNthChild:
      case kPseudoNthLastChild:
      case kPseudoNthOfType:
      case kPseudoNthLastOfType: {
        builder.Append('(');

        // https://drafts.csswg.org/css-syntax/#serializing-anb
        int a = data_.rare_data_->NthAValue();
        int b = data_.rare_data_->NthBValue();
        if (a == 0) {
          builder.Append(String::Number(b));
        } else {
          if (a == 1) {
            builder.Append('n');
          } else if (a == -1) {
            builder.Append("-n");
          } else {
            builder.AppendNumber(a);
            builder.Append('n');
          }

          if (b < 0) {
            builder.Append(String::Number(b));
          } else if (b > 0) {
            builder.Append('+');
            builder.AppendNumber(b);
          }
        }

        // Only relevant for :nth-child, not :nth-of-type.
        if (data_.rare_data_->selector_list_ != nullptr) {
          builder.Append(" of ");
          SerializeSelectorList<expand_pseudo_references>(
              data_.rare_data_->selector_list_, builder, scope_id);
          suppress_selector_list = true;
        }

        builder.Append(')');
        break;
      }
      case kPseudoDir:
      case kPseudoState:
        builder.Append('(');
        SerializeIdentifier(Argument(), builder);
        builder.Append(')');
        break;
      case kPseudoLang: {
        builder.Append('(');
        SerializeIdentifierList(builder, *ArgumentList());
        builder.Append(')');
        break;
      }
      case kPseudoHas:
      case kPseudoNot:
        DCHECK(SelectorList());
        break;
      case kPseudoHost:
      case kPseudoHostContext:
      case kPseudoAny:
      case kPseudoIs:
      case kPseudoWhere:
        break;
      case kPseudoParent:
        if constexpr (expand_pseudo_references) {
          // Replace parent pseudo with equivalent :is() pseudo.
          builder.Append(":is");
          if (auto* parent = SelectorListOrParent()) {
            builder.Append('(');
            builder.Append(
                parent->SelectorTextExpandingPseudoReferences(scope_id));
            builder.Append(')');
          }
        } else {
          builder.Append('&');
        }
        break;
      case kPseudoScope:
        if constexpr (expand_pseudo_references) {
          builder.Append(":-internal-scope-");
          builder.AppendNumber(scope_id);
        } else {
          builder.Append(':');
          builder.Append(SerializingValue());
        }
        break;
      case kPseudoRelativeAnchor:
        NOTREACHED();
      case kPseudoActiveViewTransitionType: {
        CHECK(!IdentList().empty());
        builder.Append('(');
        SerializeIdentifierList(builder, IdentList());
        builder.Append(')');
        break;
      }
      case kPseudoLinkTo: {
        DCHECK(GetNavigationLocation());
        builder.Append("(");
        GetNavigationLocation()->SerializeTo(builder);
        builder.Append(")");
        break;
      }
      default:
        break;
    }
  } else if (Match() == kPseudoElement) {
    builder.Append("::");
    SerializeIdentifier(SerializingValue(), builder);
    switch (GetPseudoType()) {
      case kPseudoPart: {
        char separator = '(';
        for (AtomicString part : IdentList()) {
          builder.Append(separator);
          if (separator == '(') {
            separator = ' ';
          }
          SerializeIdentifier(part, builder);
        }
        builder.Append(')');
        break;
      }
      case kPseudoPicker:
      case kPseudoHighlight: {
        builder.Append('(');
        if (GetPseudoType() == kPseudoHighlight &&
            Argument() == UniversalSelectorAtom()) {
          builder.Append('*');
        } else {
          SerializeIdentifier(Argument(), builder);
        }
        builder.Append(')');
        break;
      }
      case kPseudoOverscrollAreaParent:
      case kPseudoScrollButton: {
        builder.Append('(');
        // These accept only fixed arguments that do not require escaping (in
        // some cases including "*" which should not be escaped).
        builder.Append(Argument());
        builder.Append(')');
        break;
      }
      case kPseudoViewTransitionGroup:
      case kPseudoViewTransitionGroupChildren:
      case kPseudoViewTransitionImagePair:
      case kPseudoViewTransitionNew:
      case kPseudoViewTransitionOld: {
        builder.Append('(');
        bool first = true;
        for (const AtomicString& name_or_class : IdentList()) {
          if (!first) {
            builder.Append('.');
          }

          first = false;
          if (name_or_class == UniversalSelectorAtom()) {
            builder.Append(g_star_atom);
          } else {
            SerializeIdentifier(name_or_class, builder);
          }
        }
        builder.Append(')');
        break;
      }
      default:
        break;
    }
  } else if (IsAttributeSelector()) {
    builder.Append('[');
    SerializeNamespacePrefixIfNeeded(Attribute().Prefix(), g_star_atom, builder,
                                     IsAttributeSelector());
    SerializeIdentifier(Attribute().LocalName(), builder);
    switch (Match()) {
      case kAttributeExact:
        builder.Append('=');
        break;
      case kAttributeSet:
        // set has no operator or value, just the attrName
        builder.Append(']');
        break;
      case kAttributeList:
        builder.Append("~=");
        break;
      case kAttributeHyphen:
        builder.Append("|=");
        break;
      case kAttributeBegin:
        builder.Append("^=");
        break;
      case kAttributeEnd:
        builder.Append("$=");
        break;
      case kAttributeContain:
        builder.Append("*=");
        break;
      default:
        break;
    }
    if (Match() != kAttributeSet) {
      SerializeString(SerializingValue(), builder);
      if (AttributeMatch() == AttributeMatchType::kCaseInsensitive) {
        builder.Append(" i");
      } else if (AttributeMatch() == AttributeMatchType::kCaseSensitiveAlways) {
        DCHECK(RuntimeEnabledFeatures::CSSCaseSensitiveSelectorEnabled());
        builder.Append(" s");
      }
      builder.Append(']');
    }
  }

  if (SelectorList() && !suppress_selector_list) {
    builder.Append('(');
    SerializeSelectorList<expand_pseudo_references>(SelectorList(), builder,
                                                    scope_id);
    builder.Append(')');
  }
}

template <bool expand_pseudo_references>
const CSSSelector* CSSSelector::SerializeCompound(StringBuilder& builder,
                                                  uintptr_t scope_id) const {
  for (const CSSSelector* simple_selector = this; simple_selector;
       simple_selector = simple_selector->NextSimpleSelector()) {
    simple_selector->SerializeSimpleSelector<expand_pseudo_references>(
        builder, scope_id);
    if (simple_selector->Relation() != kSubSelector) {
      return simple_selector;
    }
  }
  return nullptr;
}

template <bool expand_pseudo_references>
String CSSSelector::SelectorTextInternal(uintptr_t scope_id) const {
  String result;
  for (const CSSSelector* compound = this; compound;
       compound = compound->NextSimpleSelector()) {
    StringBuilder builder;
    compound = compound->SerializeCompound<expand_pseudo_references>(builder,
                                                                     scope_id);
    if (!compound) {
      return StrCat({builder.ReleaseString(), result});
    }

    RelationType relation = compound->Relation();
    DCHECK_NE(relation, kSubSelector);

    const CSSSelector* next_compound = compound->NextSimpleSelector();
    DCHECK(next_compound);

    // If we are combining with an implicit :scope, it is as if we
    // used a relative combinator. However, when expand_pseudo_references=true,
    // we must serialize the implicit compound anyway, since the :has() cache
    // needs this to be a part of the key.
    bool implicit = next_compound->IsImplicit() && !expand_pseudo_references;
    if (!next_compound ||
        (next_compound->Match() == kPseudoClass &&
         next_compound->GetPseudoType() == kPseudoScope && implicit)) {
      relation = ConvertRelationToRelative(relation);
    }

    switch (relation) {
      case kDescendant:
        result = StrCat({" ", builder.ReleaseString(), result});
        break;
      case kChild:
        result = StrCat({" > ", builder.ReleaseString(), result});
        break;
      case kDirectAdjacent:
        result = StrCat({" + ", builder.ReleaseString(), result});
        break;
      case kIndirectAdjacent:
        result = StrCat({" ~ ", builder.ReleaseString(), result});
        break;
      case kSubSelector:
      case kPseudoChild:
      case kShadowPart:
      case kUAShadow:
      case kShadowSlot:
        result = StrCat({builder.ReleaseString(), result});
        break;
      case kRelativeDescendant:
        return StrCat({builder.ReleaseString(), result});
      case kRelativeChild:
        return StrCat({"> ", builder.ReleaseString(), result});
      case kRelativeDirectAdjacent:
        return StrCat({"+ ", builder.ReleaseString(), result});
      case kRelativeIndirectAdjacent:
        return StrCat({"~ ", builder.ReleaseString(), result});
    }
  }
  NOTREACHED();
}

String CSSSelector::SimpleSelectorTextForDebug() const {
  StringBuilder builder;
  // `scope_id` is ignored when `expand_pseudo_references` is false.
  SerializeSimpleSelector<!kExpandPseudoReferences>(builder, /*scope_id=*/0);
  return builder.ToString();
}

void CSSSelector::SetArgument(const AtomicString& value) {
  CreateRareData();
  data_.rare_data_->argument_ = value;
}

void CSSSelector::SetArgumentList(
    std::unique_ptr<Vector<AtomicString>> arguments) {
  CreateRareData();
  data_.rare_data_->argument_list_ = std::move(arguments);
}

void CSSSelector::SetSelectorList(CSSSelectorList* selector_list) {
  if (HasInlineSelectorList()) {
    data_.selector_list_ = selector_list;
    return;
  }
  if (!HasRareData() && Match() == kPseudoClass &&
      CanStoreSelectorListInline(GetPseudoType()) &&
      data_.value_ == NameForInlineSelectorListPseudo(GetPseudoType())) {
    // Same care as in CreateRareData(): switch the active union member.
    data_.value_.~AtomicString();
    new (&data_.selector_list_) Member<CSSSelectorList>(selector_list);
    bits_.set<HasInlineSelectorListField>(true);
    return;
  }
  CreateRareData();
  data_.rare_data_->selector_list_ = selector_list;
}

void CSSSelector::SetNavigationLocation(NavigationLocation* location) {
  CreateRareData();
  data_.rare_data_->navigation_location_ = location;
}

void CSSSelector::SetContainsPseudoInsideHasPseudoClass() {
  CreateRareData();
  data_.rare_data_->bits_.has_.contains_pseudo_ = true;
}

void CSSSelector::SetContainsComplexLogicalCombinationsInsideHasPseudoClass() {
  CreateRareData();
  data_.rare_data_->bits_.has_.contains_complex_logical_combinations_ = true;
}

void CSSSelector::SetHasArgumentMatchInShadowTree() {
  CreateRareData();
  data_.rare_data_->bits_.has_.argument_match_in_shadow_tree_ = true;
}

namespace {

bool IsSubSelectorCompound(const CSSSelector* selector) {
  switch (selector->Match()) {
    case CSSSelector::kTag:
    case CSSSelector::kUniversalTag:
    case CSSSelector::kId:
    case CSSSelector::kClass:
    case CSSSelector::kAttributeExact:
    case CSSSelector::kAttributeSet:
    case CSSSelector::kAttributeList:
    case CSSSelector::kAttributeHyphen:
    case CSSSelector::kAttributeContain:
    case CSSSelector::kAttributeBegin:
    case CSSSelector::kAttributeEnd:
      return true;
    case CSSSelector::kPseudoElement:
    case CSSSelector::kUnknown:
      return false;
    case CSSSelector::kPagePseudoClass:
    case CSSSelector::kPseudoClass:
      if (const CSSSelectorList* sublist = selector->SelectorList()) {
        for (const CSSSelector* subselector = sublist->First(); subselector;
             subselector = CSSSelectorList::Next(*subselector)) {
          if (!subselector->IsFullyCompound()) {
            return false;
          }
        }
      }
      return true;
    case CSSSelector::kInvalidList:
      NOTREACHED();
  }
}

}  // namespace

bool CSSSelector::IsFullyCompound() const {
  if (!IsSubSelectorCompound(this)) {
    return false;
  }

  const CSSSelector* prev_sub_selector = this;
  const CSSSelector* sub_selector = NextSimpleSelector();

  while (sub_selector) {
    if (prev_sub_selector->Relation() != kSubSelector) {
      return false;
    }
    if (!IsSubSelectorCompound(sub_selector)) {
      return false;
    }

    prev_sub_selector = sub_selector;
    sub_selector = sub_selector->NextSimpleSelector();
  }

  return true;
}

bool CSSSelector::HasLinkOrVisited() const {
  for (const CSSSelector* current = this; current;
       current = current->NextSimpleSelector()) {
    CSSSelector::PseudoType pseudo = current->GetPseudoType();
    if (pseudo == CSSSelector::kPseudoLink ||
        pseudo == CSSSelector::kPseudoVisited) {
      return true;
    }
    for (const CSSSelector* sub_selector = current->SelectorListOrParent();
         sub_selector; sub_selector = CSSSelectorList::Next(*sub_selector)) {
      if (sub_selector->HasLinkOrVisited()) {
        return true;
      }
    }
  }
  return false;
}

bool CSSSelector::HasVisited() const {
  for (const CSSSelector* current = this; current;
       current = current->NextSimpleSelector()) {
    CSSSelector::PseudoType pseudo = current->GetPseudoType();
    if (pseudo == CSSSelector::kPseudoVisited) {
      return true;
    }
    for (const CSSSelector* sub_selector = current->SelectorListOrParent();
         sub_selector; sub_selector = CSSSelectorList::Next(*sub_selector)) {
      if (sub_selector->HasVisited()) {
        return true;
      }
    }
  }
  return false;
}

void CSSSelector::SetNth(int a, int b, CSSSelectorList* sub_selectors) {
  CreateRareData();
  data_.rare_data_->bits_.nth_.a_ = a;
  data_.rare_data_->bits_.nth_.b_ = b;
  data_.rare_data_->selector_list_ = sub_selectors;
}

bool CSSSelector::MatchNth(unsigned count) const {
  DCHECK(HasRareData());
  return data_.rare_data_->MatchNth(count);
}

bool CSSSelector::MatchesPseudoElement() const {
  for (const CSSSelector* current = this; current;
       current = current->NextSimpleSelector()) {
    if (current->Match() == kPseudoElement) {
      return true;
    }
    if (current->Relation() != kSubSelector) {
      return false;
    }
  }
  return false;
}

bool CSSSelector::IsAllowedInParentPseudo() const {
  // Pseudo-elements are not allowed (parse-time) within :is(), but using
  // nesting you can still get effectively that same situation using
  // e.g. "div, ::before { & {} }". Since '::before' is "contextually invalid",
  // it should not contribute to specificity.
  //
  // https://github.com/w3c/csswg-drafts/issues/9600
  return !MatchesPseudoElement();
}

bool CSSSelector::IsTreeAbidingPseudoElement() const {
  return Match() == CSSSelector::kPseudoElement &&
         (GetPseudoType() == kPseudoCheckMark ||
          GetPseudoType() == kPseudoBefore || GetPseudoType() == kPseudoAfter ||
          GetPseudoType() == kPseudoExpandIcon ||
          GetPseudoType() == kPseudoPickerIcon ||
          GetPseudoType() == kPseudoInterestButton ||
          GetPseudoType() == kPseudoMarker ||
          GetPseudoType() == kPseudoPlaceholder ||
          GetPseudoType() == kPseudoFileSelectorButton ||
          GetPseudoType() == kPseudoBackdrop ||
          GetPseudoType() == kPseudoOverscrollBackdrop ||
          GetPseudoType() == kPseudoViewTransition ||
          GetPseudoType() == kPseudoViewTransitionGroup ||
          GetPseudoType() == kPseudoViewTransitionGroupChildren ||
          GetPseudoType() == kPseudoViewTransitionImagePair ||
          GetPseudoType() == kPseudoViewTransitionOld ||
          GetPseudoType() == kPseudoViewTransitionNew ||
          GetPseudoType() == kPseudoOverscrollAreaParent ||
          GetPseudoType() == kPseudoSkeleton ||
          GetPseudoType() == kPseudoScrollMarkerGroup ||
          GetPseudoType() == kPseudoScrollMarker ||
          GetPseudoType() == kPseudoScrollButton ||
          IsElementBackedPseudoElement(GetPseudoType()));
}

/* static */ bool CSSSelector::IsElementBackedPseudoElement(
    CSSSelector::PseudoType pseudo) {
  return pseudo == kPseudoDetailsContent || pseudo == kPseudoPicker ||
         pseudo == kPseudoPermissionIcon || pseudo == kPseudoSelectListbox;
}

bool CSSSelector::IsElementBackedPseudoElement() const {
  return Match() == CSSSelector::kPseudoElement &&
         IsElementBackedPseudoElement(GetPseudoType());
}

bool CSSSelector::IsAllowedAfterPart() const {
  if (Match() != CSSSelector::kPseudoElement &&
      Match() != CSSSelector::kPseudoClass) {
    return false;
  }
  switch (GetPseudoType()) {
    // Pseudo-elements
    //
    // All pseudo-elements other than ::part() should be allowed after
    // ::part().
    case kPseudoCheckMark:
    case kPseudoBefore:
    case kPseudoAfter:
    case kPseudoExpandIcon:
    case kPseudoPickerIcon:
    case kPseudoInterestButton:
    case kPseudoPlaceholder:
    case kPseudoFileSelectorButton:
    case kPseudoFirstLine:
    case kPseudoFirstLetter:
    case kPseudoPicker:
    case kPseudoSelectListbox:
    case kPseudoSelection:
    case kPseudoSearchText:
    case kPseudoTargetText:
    case kPseudoHighlight:
    case kPseudoSpellingError:
    case kPseudoGrammarError:
    case kPseudoBackdrop:
    case kPseudoOverscrollBackdrop:
    case kPseudoCue:
    case kPseudoMarker:
    case kPseudoResizer:
    case kPseudoScrollbar:
    case kPseudoScrollbarButton:
    case kPseudoScrollbarCorner:
    case kPseudoScrollbarThumb:
    case kPseudoScrollbarTrack:
    case kPseudoScrollbarTrackPiece:
    case kPseudoScrollMarker:
    case kPseudoScrollMarkerGroup:
    case kPseudoScrollButton:
    case kPseudoColumn:
    case kPseudoWebKitCustomElement:
    case kPseudoBlinkInternalElement:
    case kPseudoDetailsContent:
    case kPseudoPermissionIcon:
    case kPseudoViewTransition:
    case kPseudoViewTransitionGroup:
    case kPseudoViewTransitionGroupChildren:
    case kPseudoViewTransitionImagePair:
    case kPseudoViewTransitionNew:
    case kPseudoViewTransitionOld:
    case kPseudoOverscrollAreaParent:
    case kPseudoSkeleton:
      return true;

    // It's possible that we should support ::slotted() after ::part().
    // (WebKit accepts it at parse time but it doesn't appear to work;
    // Gecko doesn't accept it.)  However, making it work isn't trivial.
    // https://github.com/w3c/csswg-drafts/issues/10807
    case kPseudoSlotted:
      return false;

    case kPseudoPart:
      return false;

    // Pseudo-classes
    //
    // All non-structural pseudo-classes should be allowed, and structural
    // pseudo-classes should be forbidden.
    case kPseudoAnimatedImage:
    case kPseudoAutofill:
    case kPseudoAutofillPreviewed:
    case kPseudoAutofillSelected:
    case kPseudoWebKitAutofill:
    case kPseudoActive:
    case kPseudoActiveOption:
    case kPseudoActiveViewTransition:
    case kPseudoActiveViewTransitionType:
    case kPseudoAnyLink:
    case kPseudoBuffering:
    case kPseudoChecked:
    case kPseudoDefault:
    case kPseudoDialogInTopLayer:
    case kPseudoDisabled:
    case kPseudoDrag:
    case kPseudoEnabled:
    case kPseudoFiltered:
    case kPseudoFocus:
    case kPseudoFocusVisible:
    case kPseudoFocusWithin:
    case kPseudoFullPageMedia:
    case kPseudoHasSlotted:
    case kPseudoHover:
    case kPseudoIndeterminate:
    case kPseudoInterestSource:
    case kPseudoInterestTarget:
    case kPseudoInvalid:
    case kPseudoLang:
    case kPseudoLink:
    case kPseudoLinkTo:
    case kPseudoMenulistPopoverWithMenulistAnchor:
    case kPseudoModal:
    case kPseudoMuted:
    case kPseudoOptional:
    case kPseudoOverscrollClosed:
    case kPseudoOverscrollOpen:
    case kPseudoPermissionGranted:
    case kPseudoPlaceholderShown:
    case kPseudoReadOnly:
    case kPseudoReadWrite:
    case kPseudoRequired:
    case kPseudoSeeking:
    case kPseudoSelectContainsInput:
    case kPseudoSelectHasSlottedButton:
    case kPseudoStalled:
    case kPseudoState:
    case kPseudoTarget:
    case kPseudoUserInvalid:
    case kPseudoUserValid:
    case kPseudoValid:
    case kPseudoVisited:
    case kPseudoVolumeLocked:
    case kPseudoWebkitAnyLink:
    case kPseudoWindowInactive:
    case kPseudoFullScreen:
    case kPseudoFullScreenAncestor:
    case kPseudoFullscreen:
    case kPseudoInRange:
    case kPseudoOutOfRange:
    case kPseudoPaused:
    case kPseudoPictureInPicture:
    case kPseudoPlaying:
    case kPseudoXrOverlay:
    case kPseudoDefined:
    case kPseudoDir:
    case kPseudoFutureCue:
    case kPseudoIsHtml:
    case kPseudoListBox:
    case kPseudoMultiSelectFocus:
    case kPseudoNavigationSource:
    case kPseudoOpen:
    case kPseudoPastCue:
    case kPseudoPopoverInTopLayer:
    case kPseudoPopoverOpen:
    case kPseudoRelativeAnchor:
    case kPseudoSpatialNavigationFocus:
    case kPseudoTargetCurrent:
    case kPseudoTargetBefore:
    case kPseudoTargetAfter:
    case kPseudoTextField:
    case kPseudoToolFormActive:
    case kPseudoToolSubmitActive:
    case kPseudoUnbounded:
    case kPseudoVideoPersistent:
    case kPseudoVideoPersistentAncestor:
      return true;

    // IsSimpleSelectorValidAfterPseudoElement allows these selectors after
    // ::part() regardless of what we do here.  However, since they are in
    // fact allowed, tell the truth here.
    case kPseudoIs:
    case kPseudoNot:
    case kPseudoWhere:
      return true;

    // :-webkit-any() should in theory be allowed too like :is() and :where(),
    // but it's a legacy feature so just leave it disallowed.
    case kPseudoAny:
      return false;

    // TODO(https://crbug.com/40623497): Figure out what to do with this.
    case kPseudoParent:
      return false;

    // These are supported only after ::webkit-scrollbar, which *maybe* makes
    // them structural?  Leave them unsupported for now
    case kPseudoHorizontal:
    case kPseudoVertical:
    case kPseudoDecrement:
    case kPseudoIncrement:
    case kPseudoStart:
    case kPseudoEnd:
    case kPseudoDoubleButton:
    case kPseudoSingleButton:
    case kPseudoNoButton:
    case kPseudoCornerPresent:
    // Likewise, this matches only after ::search-text.
    case kPseudoCurrent:
      return false;

    // These are supported only on @page, so not allowed after ::part().
    case kPseudoFirstPage:
    case kPseudoLeftPage:
    case kPseudoRightPage:
      return false;

    // These are structural pseudo-classes, which should not be allowed.
    case kPseudoEmpty:
    case kPseudoFirstChild:
    case kPseudoFirstOfType:
    case kPseudoLastChild:
    case kPseudoLastOfType:
    case kPseudoNthChild:
    case kPseudoNthLastChild:
    case kPseudoNthLastOfType:
    case kPseudoNthOfType:
    case kPseudoOnlyChild:
    case kPseudoOnlyOfType:
    case kPseudoRoot:
      return false;

    // These are other pseudo-classes that match based on tree information
    // rather than local element information, which should not be allowed.
    case kPseudoHas:
    case kPseudoHasDatalist:
    case kPseudoHasOpenMenuitem:
    case kPseudoHost:
    case kPseudoHostContext:
    case kPseudoHostHasNonAutoAppearance:
    case kPseudoScope:
      return false;

    case kPseudoUnparsed:
    case kPseudoUnknown:
      return false;
  }
}

bool CSSSelector::IsOrContainsHostPseudoClass() const {
  if (IsHostPseudoClass()) {
    return true;
  }
  // Accept selector lists like :is(:host, .foo).
  for (const CSSSelector* sub_selector = SelectorListOrParent(); sub_selector;
       sub_selector = CSSSelectorList::Next(*sub_selector)) {
    if (sub_selector->IsOrContainsHostPseudoClass()) {
      return true;
    }
  }
  return false;
}

bool CSSSelector::IsDeeplyHostPseudoClass() const {
  if ((GetPseudoType() == kPseudoIs || GetPseudoType() == kPseudoWhere ||
       GetPseudoType() == kPseudoParent) &&
      SelectorListOrParent() &&
      CSSSelectorList::IsSingleComplexSelector(*SelectorListOrParent())) {
    return SelectorListOrParent()->IsDeeplyHostPseudoClass();
  }
  return IsHostPseudoClass();
}

template <typename Functor>
static bool ForAnyInComplexSelector(const Functor& functor,
                                    const CSSSelector& selector) {
  for (const CSSSelector* current = &selector; current;
       current = current->NextSimpleSelector()) {
    if (functor(*current)) {
      return true;
    }
    if (const CSSSelectorList* selector_list = current->SelectorList()) {
      for (const CSSSelector* sub_selector = selector_list->First();
           sub_selector; sub_selector = CSSSelectorList::Next(*sub_selector)) {
        if (ForAnyInComplexSelector(functor, *sub_selector)) {
          return true;
        }
      }
    }
  }

  return false;
}

bool CSSSelector::CrossesTreeScopes() const {
  for (const CSSSelector* s = this; s; s = s->NextSimpleSelector()) {
    switch (s->Relation()) {
      case kShadowPart:
      case kUAShadow:
      case kShadowSlot:
        return true;
      default:
        break;
    }
  }
  return false;
}

String CSSSelector::FormatPseudoTypeForDebugging(PseudoType type) {
  for (const auto& s : kPseudoTypeWithoutArgumentsMap) {
    if (s.type == type) {
      return s.string;
    }
  }
  for (const auto& s : kPseudoTypeWithArgumentsMap) {
    if (s.type == type) {
      return s.string;
    }
  }
  StringBuilder builder;
  builder.Append("pseudo-");
  builder.AppendNumber(static_cast<int>(type));
  return builder.ReleaseString();
}

CSSSelector::RareData::RareData(const AtomicString& value)
    : matching_value_(value),
      serializing_value_(value),
      bits_(),
      attribute_(AnyQName()),
      argument_(g_null_atom) {}

CSSSelector::RareData::RareData(const RareData& other)
    : matching_value_(other.matching_value_),
      serializing_value_(other.serializing_value_),
      bits_(other.bits_),
      attribute_(other.attribute_),
      argument_(other.argument_),
      selector_list_(other.selector_list_),
      ident_list_(other.ident_list_ ? std::make_unique<Vector<AtomicString>>(
                                          *other.ident_list_)
                                    : nullptr) {}

CSSSelector::RareData::~RareData() = default;

// a helper function for checking nth-arguments
bool CSSSelector::RareData::MatchNth(unsigned unsigned_count) {
  return CSSSelector::MatchNth(NthAValue(), NthBValue(), unsigned_count);
}

CSSSelector::RareData* CSSSelector::RareData::Renest(StyleRule* new_parent) {
  CSSSelectorList* old_list = selector_list_.Get();
  CSSSelectorList* new_list = old_list ? old_list->Renest(new_parent) : nullptr;
  if (old_list == new_list) {
    return this;
  }
  auto* rare_data = MakeGarbageCollected<RareData>(*this);
  rare_data->selector_list_ = new_list;
  return rare_data;
}

void CSSSelector::SetIdentList(
    std::unique_ptr<Vector<AtomicString>> ident_list) {
  CreateRareData();
  data_.rare_data_->ident_list_ = std::move(ident_list);
}

void CSSSelector::Trace(Visitor* visitor) const {
  if (MatchForOilpan() == kPseudoClass && GetPseudoType() == kPseudoParent) {
    visitor->Trace(data_.parent_rule_);
  } else if (HasRareDataForOilpan()) {
    visitor->Trace(data_.rare_data_);
  } else if (HasInlineSelectorListForOilpan()) {
    visitor->Trace(data_.selector_list_);
  }
}

void CSSSelector::RareData::Trace(Visitor* visitor) const {
  visitor->Trace(selector_list_);
  visitor->Trace(navigation_location_);
}

const CSSSelector* CSSSelector::SelectorListOrParent() const {
  if (Match() == kPseudoClass && GetPseudoType() == kPseudoParent) {
    if (ParentRule()) {
      return ParentRule()->FirstSelector();
    } else {
      return nullptr;
    }
  } else if (const CSSSelectorList* selector_list = SelectorList()) {
    return selector_list->First();
  } else {
    return nullptr;
  }
}

bool CSSSelector::IsChildIndexedSelector() const {
  switch (GetPseudoType()) {
    case kPseudoFirstChild:
    case kPseudoFirstOfType:
    case kPseudoLastChild:
    case kPseudoLastOfType:
    case kPseudoNthChild:
    case kPseudoNthLastChild:
    case kPseudoNthLastOfType:
    case kPseudoNthOfType:
    case kPseudoOnlyChild:
    case kPseudoOnlyOfType:
      return true;
    default:
      return false;
  }
}

CSSSelector::RelationType ConvertRelationToRelative(
    CSSSelector::RelationType relation) {
  switch (relation) {
    case CSSSelector::kSubSelector:
    case CSSSelector::kDescendant:
      return CSSSelector::kRelativeDescendant;
    case CSSSelector::kChild:
      return CSSSelector::kRelativeChild;
    case CSSSelector::kDirectAdjacent:
      return CSSSelector::kRelativeDirectAdjacent;
    case CSSSelector::kIndirectAdjacent:
      return CSSSelector::kRelativeIndirectAdjacent;
    default:
      NOTREACHED();
  }
}

// static
bool CSSSelector::SupportsPseudoStateChange(PseudoType type) {
  switch (type) {
    case CSSSelector::kPseudoAnimatedImage:
    case CSSSelector::kPseudoActive:
    case CSSSelector::kPseudoActiveOption:
    case CSSSelector::kPseudoActiveViewTransition:
    case CSSSelector::kPseudoActiveViewTransitionType:
    case CSSSelector::kPseudoAnyLink:
    case CSSSelector::kPseudoAutofill:
    case CSSSelector::kPseudoAutofillPreviewed:
    case CSSSelector::kPseudoAutofillSelected:
    case CSSSelector::kPseudoBuffering:
    case CSSSelector::kPseudoChecked:
    case CSSSelector::kPseudoDefault:
    case CSSSelector::kPseudoDefined:
    case CSSSelector::kPseudoDialogInTopLayer:
    case CSSSelector::kPseudoDir:
    case CSSSelector::kPseudoDisabled:
    case CSSSelector::kPseudoDrag:
    case CSSSelector::kPseudoEmpty:
    case CSSSelector::kPseudoEnabled:
    case CSSSelector::kPseudoFiltered:
    case CSSSelector::kPseudoFirstChild:
    case CSSSelector::kPseudoFirstOfType:
    case CSSSelector::kPseudoFocus:
    case CSSSelector::kPseudoFocusVisible:
    case CSSSelector::kPseudoFocusWithin:
    case CSSSelector::kPseudoFullScreen:
    case CSSSelector::kPseudoFullScreenAncestor:
    case CSSSelector::kPseudoFullscreen:
    case CSSSelector::kPseudoHas:
    case CSSSelector::kPseudoHasDatalist:
    case CSSSelector::kPseudoHasOpenMenuitem:
    case CSSSelector::kPseudoHasSlotted:
    case CSSSelector::kPseudoHover:
    case CSSSelector::kPseudoInRange:
    case CSSSelector::kPseudoIndeterminate:
    case CSSSelector::kPseudoInterestSource:
    case CSSSelector::kPseudoInterestTarget:
    case CSSSelector::kPseudoInvalid:
    case CSSSelector::kPseudoLang:
    case CSSSelector::kPseudoLastChild:
    case CSSSelector::kPseudoLastOfType:
    case CSSSelector::kPseudoLink:
    case CSSSelector::kPseudoLinkTo:
    case CSSSelector::kPseudoListBox:
    case CSSSelector::kPseudoModal:
    case CSSSelector::kPseudoMultiSelectFocus:
    case CSSSelector::kPseudoMuted:
    case CSSSelector::kPseudoNavigationSource:
    case CSSSelector::kPseudoNthChild:
    case CSSSelector::kPseudoNthLastChild:
    case CSSSelector::kPseudoNthLastOfType:
    case CSSSelector::kPseudoNthOfType:
    case CSSSelector::kPseudoOnlyChild:
    case CSSSelector::kPseudoOnlyOfType:
    case CSSSelector::kPseudoOpen:
    case CSSSelector::kPseudoOptional:
    case CSSSelector::kPseudoOutOfRange:
    case CSSSelector::kPseudoPaused:
    case CSSSelector::kPseudoPermissionGranted:
    case CSSSelector::kPseudoPictureInPicture:
    case CSSSelector::kPseudoPlaceholderShown:
    case CSSSelector::kPseudoPlaying:
    case CSSSelector::kPseudoPopoverInTopLayer:
    case CSSSelector::kPseudoPopoverOpen:
    case CSSSelector::kPseudoReadOnly:
    case CSSSelector::kPseudoReadWrite:
    case CSSSelector::kPseudoRequired:
    case CSSSelector::kPseudoSeeking:
    case CSSSelector::kPseudoSelectContainsInput:
    case CSSSelector::kPseudoSelectHasSlottedButton:
    case CSSSelector::kPseudoSelection:
    case CSSSelector::kPseudoStalled:
    case CSSSelector::kPseudoState:
    case CSSSelector::kPseudoTarget:
    case CSSSelector::kPseudoTargetAfter:
    case CSSSelector::kPseudoTargetBefore:
    case CSSSelector::kPseudoTargetCurrent:
    case CSSSelector::kPseudoTextField:
    case CSSSelector::kPseudoToolFormActive:
    case CSSSelector::kPseudoToolSubmitActive:
    case CSSSelector::kPseudoUnbounded:
    case CSSSelector::kPseudoUserInvalid:
    case CSSSelector::kPseudoUserValid:
    case CSSSelector::kPseudoValid:
    case CSSSelector::kPseudoVideoPersistent:
    case CSSSelector::kPseudoVideoPersistentAncestor:
    case CSSSelector::kPseudoVisited:
    case CSSSelector::kPseudoWebKitAutofill:
    case CSSSelector::kPseudoWebkitAnyLink:
    case CSSSelector::kPseudoXrOverlay:
      return true;
    default:
      return false;
  }
}

constexpr auto PseudoNameProjection = [](const NameToPseudoStruct& entry) {
  return std::string_view(entry.string);
};

static_assert(std::ranges::is_sorted(kPseudoTypeWithoutArgumentsMap,
                                     {},
                                     PseudoNameProjection),
              "kPseudoTypeWithoutArgumentsMap must be sorted.");
static_assert(std::ranges::is_sorted(kPseudoTypeWithArgumentsMap,
                                     {},
                                     PseudoNameProjection),
              "kPseudoTypeWithArgumentsMap must be sorted.");

}  // namespace blink

```
