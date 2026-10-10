# Blink CSSOM wrapper deficiency source witness

This reference retains 7 complete source files from [Blink](https://www.chromium.org/blink/). The preferred immutable revision is `7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2`. It provides bounded CSSOM implementation evidence, independently of the normative specifications and any Surgeist implementation decision.

Retrieved on 2026-10-10 from official Chromium Gitiles immutable TEXT responses, decoded from base64. Each complete file below retains its original path, exact byte hash, and copyright/license header. Added provenance, headings, and evidence labels are documentation formatting; the source bytes are unchanged and were not compiled or executed.

File-specific legal material is recorded in the [local attribution](../licenses/chromium/NOTICE.md). The original notices govern each source file.

These fallback witnesses distinguish a working descriptor getter from incomplete or absent rule surfaces. The counter-style `speakAs` getter serializes its stored value. The custom-media wrapper exists, but its `cssText` omits the name and semicolon. Font-feature interface, storage, and serialization cover six category maps without `historicalForms`; the wrapper emits no top-level `font-display` descriptor. Native integer storage does not define conversion from authored exact integers outside its representable range. The dispatch tables do not recognize named-support, when, else, or color-profile rules at this revision.

The [WebKit wrapper witness](webkit-cssom-wrapper-rules--73aa6c89e2cb.md) retains the preferred wrapper evidence and its bounded deficiencies. Existing [Blink Page/margin evidence](blink-cssom-page-rules--7984f9d11800.md) remains separate. No universal wrapper format or product policy is inferred from these files.

| Source | Evidence boundary | Original file SHA-256 |
| --- | --- | --- |
| [css_counter_style_rule.cc](#css-counter-style-rule-cc) | speakAs at line 96 serializes the stored descriptor value; cssText includes the speak-as slot. | `5faace57530648266bfd1047ce0e9d9b6ff74df1e6d196257fc95a85262fd916` |
| [css_custom_media_rule.cc](#css-custom-media-rule-cc) | cssText at line 47 emits the boolean/media query but omits the rule name and final semicolon. | `b28abd1bd36ce7b3ffed80c6de4aeb6c35c915ce1161362c90f54b3232960c07` |
| [css_font_feature_values_rule.idl](#css-font-feature-values-rule-idl) | The complete interface exposes six category maps and no historicalForms map. | `31bb25dcd20eba06921b199488dbadbf6e44909e1ff925c973690c3500488d57` |
| [css_font_feature_values_rule.cc](#css-font-feature-values-rule-cc) | The getters and cssText at line 92 cover the same six categories; there is no historicalForms block or font-display descriptor emission. | `d86bb924fe302ecdc4cd48b6341e9566f73ee7ac5a85b1df1785c70d3f5b638c` |
| [style_rule_font_feature_values.h](#style-rule-font-feature-values-h) | FontFeatureAliases uses HashMap; category enum, storage, and accessors contain the same six categories. | `9a2a3b81e4728c3dc73ad2876fd3bb2492b9e380bd121d6697ba9eedf0a9a180` |
| [css_at_rule_id.h](#css-at-rule-id-h) | The complete at-rule enum includes supports and custom-media but no supports-condition, when, else, or color-profile. | `37d6f1295711ac698e6b415854b9082e25bb0eb31c3fc90a904555abf667c560` |
| [css_at_rule_id.cc](#css-at-rule-id-cc) | The complete regular/flagged dispatch tables and invalid-name fallback establish the inspected named-support absence. | `a829fe085e381f3120ef1e41ea25c27af5354d5eb3d328ac2fb6f9f0c4bdab88` |

## <a id="css-counter-style-rule-cc"></a>css_counter_style_rule.cc

Original: [complete immutable source](https://chromium.googlesource.com/chromium/src/+/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/css/css_counter_style_rule.cc). Path: `third_party/blink/renderer/core/css/css_counter_style_rule.cc`. Revision: `7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2`. Source bytes: 8745; source lines: 225; SHA-256: `5faace57530648266bfd1047ce0e9d9b6ff74df1e6d196257fc95a85262fd916`.

````cpp
// Copyright 2020 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#include "third_party/blink/renderer/core/css/css_counter_style_rule.h"

#include "third_party/blink/renderer/core/css/css_markup.h"
#include "third_party/blink/renderer/core/css/css_style_sheet.h"
#include "third_party/blink/renderer/core/css/parser/at_rule_descriptor_parser.h"
#include "third_party/blink/renderer/core/css/parser/css_parser_context.h"
#include "third_party/blink/renderer/core/css/parser/css_tokenizer.h"
#include "third_party/blink/renderer/core/css/properties/css_parsing_utils.h"
#include "third_party/blink/renderer/core/css/style_engine.h"
#include "third_party/blink/renderer/core/css/style_rule_counter_style.h"
#include "third_party/blink/renderer/core/css/style_rule_css_style_declaration.h"
#include "third_party/blink/renderer/core/dom/document.h"
#include "third_party/blink/renderer/core/execution_context/execution_context.h"
#include "third_party/blink/renderer/platform/wtf/text/string_builder.h"

namespace blink {

CSSCounterStyleRule::CSSCounterStyleRule(
    StyleRuleCounterStyle* counter_style_rule,
    CSSStyleSheet* sheet)
    : CSSRule(sheet), counter_style_rule_(counter_style_rule) {}

CSSCounterStyleRule::~CSSCounterStyleRule() = default;

String CSSCounterStyleRule::cssText() const {
  StringBuilder result;
  result.Append("@counter-style ");
  SerializeIdentifier(name(), result);
  result.Append(" {");

  // Note: The exact serialization isn't well specified.
  AppendDescriptorIfNotEmpty(result, "system", system());
  AppendDescriptorIfNotEmpty(result, "symbols", symbols());
  AppendDescriptorIfNotEmpty(result, "additive-symbols", additiveSymbols());
  AppendDescriptorIfNotEmpty(result, "negative", negative());
  AppendDescriptorIfNotEmpty(result, "prefix", prefix());
  AppendDescriptorIfNotEmpty(result, "suffix", suffix());
  AppendDescriptorIfNotEmpty(result, "pad", pad());
  AppendDescriptorIfNotEmpty(result, "range", range());
  AppendDescriptorIfNotEmpty(result, "fallback", fallback());
  AppendDescriptorIfNotEmpty(result, "speak-as", speakAs());

  result.Append(" }");
  return result.ReleaseString();
}

void CSSCounterStyleRule::Reattach(StyleRuleBase* rule) {
  DCHECK(rule);
  counter_style_rule_ = To<StyleRuleCounterStyle>(rule);
  if (counter_style_cssom_wrapper_) {
    counter_style_cssom_wrapper_->Reattach(counter_style_rule_->Properties());
  }
}

String CSSCounterStyleRule::name() const {
  return counter_style_rule_->GetName();
}

String CSSCounterStyleRule::system() const {
  return CSSValue::CssTextOrEmptyString(counter_style_rule_->GetSystem());
}

String CSSCounterStyleRule::symbols() const {
  return CSSValue::CssTextOrEmptyString(counter_style_rule_->GetSymbols());
}

String CSSCounterStyleRule::additiveSymbols() const {
  return CSSValue::CssTextOrEmptyString(
      counter_style_rule_->GetAdditiveSymbols());
}

String CSSCounterStyleRule::negative() const {
  return CSSValue::CssTextOrEmptyString(counter_style_rule_->GetNegative());
}

String CSSCounterStyleRule::prefix() const {
  return CSSValue::CssTextOrEmptyString(counter_style_rule_->GetPrefix());
}

String CSSCounterStyleRule::suffix() const {
  return CSSValue::CssTextOrEmptyString(counter_style_rule_->GetSuffix());
}

String CSSCounterStyleRule::range() const {
  return CSSValue::CssTextOrEmptyString(counter_style_rule_->GetRange());
}

String CSSCounterStyleRule::pad() const {
  return CSSValue::CssTextOrEmptyString(counter_style_rule_->GetPad());
}

String CSSCounterStyleRule::speakAs() const {
  return CSSValue::CssTextOrEmptyString(counter_style_rule_->GetSpeakAs());
}

String CSSCounterStyleRule::fallback() const {
  return CSSValue::CssTextOrEmptyString(counter_style_rule_->GetFallback());
}

void CSSCounterStyleRule::SetterInternal(
    const ExecutionContext* execution_context,
    AtRuleDescriptorID descriptor_id,
    const String& text) {
  CSSStyleSheet* style_sheet = parentStyleSheet();
  auto& context = *MakeGarbageCollected<CSSParserContext>(
      ParserContext(execution_context->GetSecureContextMode()), style_sheet);
  CSSParserTokenStream stream(text);
  CSSValue* new_value = AtRuleDescriptorParser::ParseAtCounterStyleDescriptor(
      descriptor_id, stream, context);
  if (!new_value ||
      !counter_style_rule_->NewValueInvalidOrEqual(descriptor_id, new_value)) {
    return;
  }

  // TODO(xiaochengh): RuleMutationScope causes all rules of the tree scope to
  // be re-collected and the entire CounterStyleMap rebuilt, while we only need
  // to dirty one CounterStyle. Try to improve.
  CSSStyleSheet::RuleMutationScope rule_mutation_scope(this);

  counter_style_rule_->SetDescriptorValue(descriptor_id, new_value);
  if (Document* document = style_sheet->OwnerDocument()) {
    document->GetStyleEngine().MarkCounterStylesNeedUpdate();
  }
}

void CSSCounterStyleRule::setName(const ExecutionContext* execution_context,
                                  const String& text) {
  CSSStyleSheet* style_sheet = parentStyleSheet();
  auto& context = *MakeGarbageCollected<CSSParserContext>(
      ParserContext(execution_context->GetSecureContextMode()), style_sheet);
  CSSParserTokenStream stream(text);
  AtomicString name =
      css_parsing_utils::ConsumeCounterStyleNameInPrelude(stream, context);
  if (!name || name == counter_style_rule_->GetName() || !stream.AtEnd()) {
    return;
  }

  // Changing name may affect cascade result, which requires re-collecting all
  // the rules and re-constructing the CounterStyleMap to handle.
  CSSStyleSheet::RuleMutationScope rule_mutation_scope(this);

  counter_style_rule_->SetName(name);
  if (Document* document = style_sheet->OwnerDocument()) {
    document->GetStyleEngine().MarkCounterStylesNeedUpdate();
  }
}

void CSSCounterStyleRule::setSystem(const ExecutionContext* execution_context,
                                    const String& text) {
  SetterInternal(execution_context, AtRuleDescriptorID::System, text);
}

void CSSCounterStyleRule::setSymbols(const ExecutionContext* execution_context,
                                     const String& text) {
  SetterInternal(execution_context, AtRuleDescriptorID::Symbols, text);
}

void CSSCounterStyleRule::setAdditiveSymbols(
    const ExecutionContext* execution_context,
    const String& text) {
  SetterInternal(execution_context, AtRuleDescriptorID::AdditiveSymbols, text);
}

void CSSCounterStyleRule::setNegative(const ExecutionContext* execution_context,
                                      const String& text) {
  SetterInternal(execution_context, AtRuleDescriptorID::Negative, text);
}

void CSSCounterStyleRule::setPrefix(const ExecutionContext* execution_context,
                                    const String& text) {
  SetterInternal(execution_context, AtRuleDescriptorID::Prefix, text);
}

void CSSCounterStyleRule::setSuffix(const ExecutionContext* execution_context,
                                    const String& text) {
  SetterInternal(execution_context, AtRuleDescriptorID::Suffix, text);
}

void CSSCounterStyleRule::setRange(const ExecutionContext* execution_context,
                                   const String& text) {
  SetterInternal(execution_context, AtRuleDescriptorID::Range, text);
}

void CSSCounterStyleRule::setPad(const ExecutionContext* execution_context,
                                 const String& text) {
  SetterInternal(execution_context, AtRuleDescriptorID::Pad, text);
}

void CSSCounterStyleRule::setSpeakAs(const ExecutionContext* execution_context,
                                     const String& text) {
  SetterInternal(execution_context, AtRuleDescriptorID::SpeakAs, text);
}

void CSSCounterStyleRule::setFallback(const ExecutionContext* execution_context,
                                      const String& text) {
  SetterInternal(execution_context, AtRuleDescriptorID::Fallback, text);
}

CSSStyleDeclaration* CSSCounterStyleRule::Style() {
  if (!counter_style_cssom_wrapper_) {
    counter_style_cssom_wrapper_ =
        MakeGarbageCollected<StyleRuleCSSStyleDeclaration>(
            counter_style_rule_->Properties(), this);
  }
  return counter_style_cssom_wrapper_;
}

CSSStyleDeclaration* CSSCounterStyleRule::MutableStyleForInspector() {
  // We cannot keep this wrapper around, because we need to request a new one
  // so that the inner style can invalidate layout.
  return MakeGarbageCollected<StyleRuleCSSStyleDeclaration>(
      counter_style_rule_->MutableStyleForInspector(), this);
}

void CSSCounterStyleRule::Trace(Visitor* visitor) const {
  visitor->Trace(counter_style_rule_);
  visitor->Trace(counter_style_cssom_wrapper_);
  CSSRule::Trace(visitor);
}

}  // namespace blink
````

## <a id="css-custom-media-rule-cc"></a>css_custom_media_rule.cc

Original: [complete immutable source](https://chromium.googlesource.com/chromium/src/+/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/css/css_custom_media_rule.cc). Path: `third_party/blink/renderer/core/css/css_custom_media_rule.cc`. Revision: `7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2`. Source bytes: 2372; source lines: 78; SHA-256: `b28abd1bd36ce7b3ffed80c6de4aeb6c35c915ce1161362c90f54b3232960c07`.

````cpp
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
````

## <a id="css-font-feature-values-rule-idl"></a>css_font_feature_values_rule.idl

Original: [complete immutable source](https://chromium.googlesource.com/chromium/src/+/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/css/css_font_feature_values_rule.idl). Path: `third_party/blink/renderer/core/css/css_font_feature_values_rule.idl`. Revision: `7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2`. Source bytes: 953; source lines: 22; SHA-256: `31bb25dcd20eba06921b199488dbadbf6e44909e1ff925c973690c3500488d57`.

````webidl
// Copyright 2022 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// https://drafts.csswg.org/css-fonts-4/#om-fontfeaturevalues
[Exposed(Window ExposeCSSFontFeatureValuesRule)]
interface CSSFontFeatureValuesRule : CSSRule {
  attribute CSSOMString fontFamily;

  [SameObject] readonly attribute CSSFontFeatureValuesMap annotation;
  [SameObject] readonly attribute CSSFontFeatureValuesMap ornaments;
  [SameObject] readonly attribute CSSFontFeatureValuesMap stylistic;
  [SameObject] readonly attribute CSSFontFeatureValuesMap swash;
  [SameObject] readonly attribute CSSFontFeatureValuesMap characterVariant;
  [SameObject] readonly attribute CSSFontFeatureValuesMap styleset;
};

interface CSSFontFeatureValuesMap {
   maplike<CSSOMString, sequence<unsigned long>>;
   undefined set(CSSOMString featureValueName,
     (unsigned long or sequence<unsigned long>) values);
};
````

## <a id="css-font-feature-values-rule-cc"></a>css_font_feature_values_rule.cc

Original: [complete immutable source](https://chromium.googlesource.com/chromium/src/+/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/css/css_font_feature_values_rule.cc). Path: `third_party/blink/renderer/core/css/css_font_feature_values_rule.cc`. Revision: `7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2`. Source bytes: 6451; source lines: 179; SHA-256: `d86bb924fe302ecdc4cd48b6341e9566f73ee7ac5a85b1df1785c70d3f5b638c`.

````cpp
// Copyright 2022 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#include "third_party/blink/renderer/core/css/css_font_feature_values_rule.h"

#include "third_party/blink/renderer/core/css/css_identifier_value.h"
#include "third_party/blink/renderer/core/css/css_markup.h"
#include "third_party/blink/renderer/core/css/css_style_sheet.h"
#include "third_party/blink/renderer/core/css/css_value_list.h"
#include "third_party/blink/renderer/core/css/style_rule.h"
#include "third_party/blink/renderer/platform/wtf/text/string_builder.h"

namespace blink {

CSSFontFeatureValuesRule::CSSFontFeatureValuesRule(
    StyleRuleFontFeatureValues* font_feature_values_rule,
    CSSStyleSheet* parent)
    : CSSRule(parent), font_feature_values_rule_(font_feature_values_rule) {}

CSSFontFeatureValuesRule::~CSSFontFeatureValuesRule() = default;

void CSSFontFeatureValuesRule::setFontFamily(const String& font_family) {
  CSSStyleSheet::RuleMutationScope mutation_scope(this);

  Vector<StringView> families = StringView(font_family).SplitSkippingEmpty(',');

  Vector<AtomicString> filtered_families;
  filtered_families.ReserveInitialCapacity(families.size());
  for (const auto& family : families) {
    StringView stripped = family.StripWhiteSpace();
    if (!stripped.empty()) {
      filtered_families.push_back(AtomicString(stripped));
    }
  }

  font_feature_values_rule_->SetFamilies(std::move(filtered_families));
}

String CSSFontFeatureValuesRule::fontFamily() {
  return font_feature_values_rule_->FamilyAsString();
}

CSSFontFeatureValuesMap* CSSFontFeatureValuesRule::annotation() {
  if (!annotation_) {
    annotation_ = MakeGarbageCollected<CSSFontFeatureValuesMap>(
        this, font_feature_values_rule_,
        font_feature_values_rule_->GetAnnotation());
  }
  return annotation_.Get();
}
CSSFontFeatureValuesMap* CSSFontFeatureValuesRule::ornaments() {
  if (!ornaments_) {
    ornaments_ = MakeGarbageCollected<CSSFontFeatureValuesMap>(
        this, font_feature_values_rule_,
        font_feature_values_rule_->GetOrnaments());
  }
  return ornaments_.Get();
}
CSSFontFeatureValuesMap* CSSFontFeatureValuesRule::stylistic() {
  if (!stylistic_) {
    stylistic_ = MakeGarbageCollected<CSSFontFeatureValuesMap>(
        this, font_feature_values_rule_,
        font_feature_values_rule_->GetStylistic());
  }
  return stylistic_.Get();
}
CSSFontFeatureValuesMap* CSSFontFeatureValuesRule::swash() {
  if (!swash_) {
    swash_ = MakeGarbageCollected<CSSFontFeatureValuesMap>(
        this, font_feature_values_rule_, font_feature_values_rule_->GetSwash());
  }
  return swash_.Get();
}
CSSFontFeatureValuesMap* CSSFontFeatureValuesRule::characterVariant() {
  if (!character_variant_) {
    character_variant_ = MakeGarbageCollected<CSSFontFeatureValuesMap>(
        this, font_feature_values_rule_,
        font_feature_values_rule_->GetCharacterVariant());
  }
  return character_variant_.Get();
}
CSSFontFeatureValuesMap* CSSFontFeatureValuesRule::styleset() {
  if (!styleset_) {
    styleset_ = MakeGarbageCollected<CSSFontFeatureValuesMap>(
        this, font_feature_values_rule_,
        font_feature_values_rule_->GetStyleset());
  }
  return styleset_.Get();
}

String CSSFontFeatureValuesRule::cssText() const {
  StringBuilder result;
  result.Append("@font-feature-values ");
  DCHECK(font_feature_values_rule_);
  result.Append(font_feature_values_rule_->FamilyAsString());
  result.Append(" { ");
  auto append_category = [&result](String rule_name,
                                   FontFeatureAliases* aliases) {
    DCHECK(aliases);
    if (aliases->size()) {
      result.Append("@");
      result.Append(rule_name);
      result.Append(" { ");
      for (auto& alias : *aliases) {
        // In CSS parsing of @font-feature-values an alias is only
        // appended if numbers are specified. In CSSOM
        // (CSSFontFeatureValuesMap::set) an empty or type-incompatible
        // argument is coerced into a number 0 and appended.
        DCHECK_GT(alias.value.indices.size(), 0u);
        SerializeIdentifier(alias.key, result);
        result.Append(":");
        for (uint32_t value : alias.value.indices) {
          result.Append(' ');
          result.AppendNumber(value);
        }
        result.Append("; ");
      }
      result.Append("} ");
    }
  };
  append_category("annotation", font_feature_values_rule_->GetAnnotation());
  append_category("ornaments", font_feature_values_rule_->GetOrnaments());
  append_category("stylistic", font_feature_values_rule_->GetStylistic());
  append_category("swash", font_feature_values_rule_->GetSwash());
  append_category("character-variant",
                  font_feature_values_rule_->GetCharacterVariant());
  append_category("styleset", font_feature_values_rule_->GetStyleset());
  result.Append("}");
  return result.ToString();
}

void CSSFontFeatureValuesRule::Reattach(StyleRuleBase* rule) {
  DCHECK(rule);
  font_feature_values_rule_ = To<StyleRuleFontFeatureValues>(rule);
  if (annotation_) {
    annotation_->Reattach(font_feature_values_rule_.Get(),
                          font_feature_values_rule_->GetAnnotation());
  }
  if (ornaments_) {
    ornaments_->Reattach(font_feature_values_rule_.Get(),
                         font_feature_values_rule_->GetOrnaments());
  }
  if (stylistic_) {
    stylistic_->Reattach(font_feature_values_rule_.Get(),
                         font_feature_values_rule_->GetStylistic());
  }
  if (swash_) {
    swash_->Reattach(font_feature_values_rule_.Get(),
                     font_feature_values_rule_->GetSwash());
  }
  if (character_variant_) {
    character_variant_->Reattach(
        font_feature_values_rule_.Get(),
        font_feature_values_rule_->GetCharacterVariant());
  }
  if (styleset_) {
    styleset_->Reattach(font_feature_values_rule_.Get(),
                        font_feature_values_rule_->GetStyleset());
  }
}

void CSSFontFeatureValuesRule::Trace(blink::Visitor* visitor) const {
  visitor->Trace(font_feature_values_rule_);
  visitor->Trace(annotation_);
  visitor->Trace(ornaments_);
  visitor->Trace(stylistic_);
  visitor->Trace(swash_);
  visitor->Trace(character_variant_);
  visitor->Trace(styleset_);
  CSSRule::Trace(visitor);
}

const StyleRuleFontFeatureValues*
CSSFontFeatureValuesRule::FontFeatureValues() {
  return font_feature_values_rule_.Get();
}

}  // namespace blink
````

## <a id="style-rule-font-feature-values-h"></a>style_rule_font_feature_values.h

Original: [complete immutable source](https://chromium.googlesource.com/chromium/src/+/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/css/style_rule_font_feature_values.h). Path: `third_party/blink/renderer/core/css/style_rule_font_feature_values.h`. Revision: `7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2`. Source bytes: 5697; source lines: 161; SHA-256: `9a2a3b81e4728c3dc73ad2876fd3bb2492b9e380bd121d6697ba9eedf0a9a180`.

````cpp
// Copyright 2022 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#ifndef THIRD_PARTY_BLINK_RENDERER_CORE_CSS_STYLE_RULE_FONT_FEATURE_VALUES_H_
#define THIRD_PARTY_BLINK_RENDERER_CORE_CSS_STYLE_RULE_FONT_FEATURE_VALUES_H_

#include "third_party/blink/renderer/core/core_export.h"
#include "third_party/blink/renderer/core/css/style_rule.h"

namespace blink {

struct FeatureIndicesWithPriority {
  Vector<uint32_t> indices;
  uint16_t layer_order = std::numeric_limits<uint16_t>::max();
};

using FontFeatureAliases = HashMap<AtomicString, FeatureIndicesWithPriority>;

class CORE_EXPORT StyleRuleFontFeature : public StyleRuleBase {
 public:
  enum class FeatureType {
    kStylistic,
    kStyleset,
    kCharacterVariant,
    kSwash,
    kOrnaments,
    kAnnotation
  };

  explicit StyleRuleFontFeature(FeatureType);
  StyleRuleFontFeature(const StyleRuleFontFeature&);
  ~StyleRuleFontFeature();

  void UpdateAlias(AtomicString alias, Vector<uint32_t> features);
  void OverrideAliasesIn(FontFeatureAliases& destination);

  FeatureType GetFeatureType() { return type_; }

  void TraceAfterDispatch(blink::Visitor*) const;

 private:
  FeatureType type_;
  FontFeatureAliases feature_aliases_;
};

template <>
struct DowncastTraits<StyleRuleFontFeature> {
  static bool AllowFrom(const StyleRuleBase& rule) {
    return rule.IsFontFeatureRule();
  }
};

class CORE_EXPORT FontFeatureValuesStorage {
 public:
  FontFeatureValuesStorage(FontFeatureAliases stylistic,
                           FontFeatureAliases styleset,
                           FontFeatureAliases character_variant,
                           FontFeatureAliases swash,
                           FontFeatureAliases ornaments,
                           FontFeatureAliases annotation);

  FontFeatureValuesStorage() = default;
  FontFeatureValuesStorage(const FontFeatureValuesStorage& other) = default;

  FontFeatureValuesStorage& operator=(const FontFeatureValuesStorage& other) =
      default;

  Vector<uint32_t> ResolveStylistic(const AtomicString&) const;
  Vector<uint32_t> ResolveStyleset(const AtomicString&) const;
  Vector<uint32_t> ResolveCharacterVariant(const AtomicString&) const;
  Vector<uint32_t> ResolveSwash(const AtomicString&) const;
  Vector<uint32_t> ResolveOrnaments(const AtomicString&) const;
  Vector<uint32_t> ResolveAnnotation(const AtomicString&) const;

  void SetLayerOrder(uint16_t layer_order);

  // Update and extend this FontFeatureValuesStorage with information from
  // `other`. Intended to be used for fusing multiple at-rules in a document and
  // across cascade layers so that their maps became unified, compare
  // https://drafts.csswg.org/css-fonts-4/#font-feature-values-syntax: If
  // multiple @font-feature-values rules are defined for a given family, the
  // resulting values definitions are the union of the definitions contained
  // within these rules. If `other` is passed in with a higher `layer_order`,
  // existing alias keys are overridden with the values from `other`.
  void FuseUpdate(const FontFeatureValuesStorage& other, unsigned layer_order);

 private:
  // TODO(https://crbug.com/716567): Only styleset and character variant take
  // two values for each alias, the others take 1 value. Consider reducing
  // storage here.
  FontFeatureAliases stylistic_;
  FontFeatureAliases styleset_;
  FontFeatureAliases character_variant_;
  FontFeatureAliases swash_;
  FontFeatureAliases ornaments_;
  FontFeatureAliases annotation_;
  static Vector<uint32_t> ResolveInternal(const FontFeatureAliases&,
                                          const AtomicString&);

  friend class StyleRuleFontFeatureValues;
};

class CORE_EXPORT StyleRuleFontFeatureValues : public StyleRuleBase {
 public:
  StyleRuleFontFeatureValues(Vector<AtomicString> families,
                             FontFeatureAliases stylistic,
                             FontFeatureAliases styleset,
                             FontFeatureAliases character_variant,
                             FontFeatureAliases swash,
                             FontFeatureAliases ornaments,
                             FontFeatureAliases annotation);
  StyleRuleFontFeatureValues(const StyleRuleFontFeatureValues&);
  ~StyleRuleFontFeatureValues();

  const Vector<AtomicString>& GetFamilies() const { return families_; }
  String FamilyAsString() const;

  void SetFamilies(Vector<AtomicString>);

  StyleRuleFontFeatureValues* Copy() const {
    return MakeGarbageCollected<StyleRuleFontFeatureValues>(*this);
  }

  const FontFeatureValuesStorage& Storage() { return feature_values_storage_; }

  // Accessors needed for cssom implementation.
  FontFeatureAliases* GetStylistic() {
    return &feature_values_storage_.stylistic_;
  }
  FontFeatureAliases* GetStyleset() {
    return &feature_values_storage_.styleset_;
  }
  FontFeatureAliases* GetCharacterVariant() {
    return &feature_values_storage_.character_variant_;
  }
  FontFeatureAliases* GetSwash() { return &feature_values_storage_.swash_; }
  FontFeatureAliases* GetOrnaments() {
    return &feature_values_storage_.ornaments_;
  }
  FontFeatureAliases* GetAnnotation() {
    return &feature_values_storage_.annotation_;
  }

  void TraceAfterDispatch(blink::Visitor*) const;

 private:
  Vector<AtomicString> families_;
  FontFeatureValuesStorage feature_values_storage_;
};

template <>
struct DowncastTraits<StyleRuleFontFeatureValues> {
  static bool AllowFrom(const StyleRuleBase& rule) {
    return rule.IsFontFeatureValuesRule();
  }
};

}  // namespace blink

#endif  // THIRD_PARTY_BLINK_RENDERER_CORE_CSS_STYLE_RULE_FONT_FEATURE_VALUES_H_
````

## <a id="css-at-rule-id-h"></a>css_at_rule_id.h

Original: [complete immutable source](https://chromium.googlesource.com/chromium/src/+/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/css/parser/css_at_rule_id.h). Path: `third_party/blink/renderer/core/css/parser/css_at_rule_id.h`. Revision: `7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2`. Source bytes: 2047; source lines: 80; SHA-256: `37d6f1295711ac698e6b415854b9082e25bb0eb31c3fc90a904555abf667c560`.

````cpp
// Copyright 2015 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#ifndef THIRD_PARTY_BLINK_RENDERER_CORE_CSS_PARSER_CSS_AT_RULE_ID_H_
#define THIRD_PARTY_BLINK_RENDERER_CORE_CSS_PARSER_CSS_AT_RULE_ID_H_

#include "third_party/blink/renderer/platform/wtf/text/string_view.h"

namespace blink {

class CSSParserContext;

enum class CSSAtRuleID {
  kCSSAtRuleInvalid,
  kCSSAtRuleViewTransition,
  kCSSAtRuleCharset,
  kCSSAtRuleFontFace,
  kCSSAtRuleFontPaletteValues,
  kCSSAtRuleImport,
  kCSSAtRuleKeyframes,
  kCSSAtRuleLayer,
  kCSSAtRuleMedia,
  kCSSAtRuleNamespace,
  kCSSAtRulePage,
  kCSSAtRulePositionTry,
  kCSSAtRuleProperty,
  kCSSAtRuleLocation,
  kCSSAtRuleNavigation,
  kCSSAtRuleContainer,
  kCSSAtRuleCounterStyle,
  kCSSAtRuleScope,
  kCSSAtRuleStartingStyle,
  kCSSAtRuleSupports,
  kCSSAtRuleWebkitKeyframes,
  // Font-feature-values related at-rule ids below:
  kCSSAtRuleAnnotation,
  kCSSAtRuleCharacterVariant,
  kCSSAtRuleFontFeatureValues,
  kCSSAtRuleOrnaments,
  kCSSAtRuleStylistic,
  kCSSAtRuleStyleset,
  kCSSAtRuleSwash,
  // https://www.w3.org/TR/css-page-3/#syntax-page-selector
  kCSSAtRuleTopLeftCorner,
  kCSSAtRuleTopLeft,
  kCSSAtRuleTopCenter,
  kCSSAtRuleTopRight,
  kCSSAtRuleTopRightCorner,
  kCSSAtRuleBottomLeftCorner,
  kCSSAtRuleBottomLeft,
  kCSSAtRuleBottomCenter,
  kCSSAtRuleBottomRight,
  kCSSAtRuleBottomRightCorner,
  kCSSAtRuleLeftTop,
  kCSSAtRuleLeftMiddle,
  kCSSAtRuleLeftBottom,
  kCSSAtRuleRightTop,
  kCSSAtRuleRightMiddle,
  kCSSAtRuleRightBottom,
  // CSS Functions and Mixins
  kCSSAtRuleFunction,
  kCSSAtRuleMixin,
  kCSSAtRuleApplyMixin,
  kCSSAtRuleContents,
  kCSSAtRuleResult,
  kCSSAtRulePrivate,
  kCSSAtRuleCustomMedia,

  kCount  // Must go last.
};

CSSAtRuleID CssAtRuleID(StringView name);
StringView CssAtRuleIDToString(CSSAtRuleID id);

void CountAtRule(const CSSParserContext*, CSSAtRuleID);

}  // namespace blink

#endif  // THIRD_PARTY_BLINK_RENDERER_CORE_CSS_PARSER_CSS_AT_RULE_ID_H_
````

## <a id="css-at-rule-id-cc"></a>css_at_rule_id.cc

Original: [complete immutable source](https://chromium.googlesource.com/chromium/src/+/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/css/parser/css_at_rule_id.cc). Path: `third_party/blink/renderer/core/css/parser/css_at_rule_id.cc`. Revision: `7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2`. Source bytes: 8146; source lines: 179; SHA-256: `a829fe085e381f3120ef1e41ea25c27af5354d5eb3d328ac2fb6f9f0c4bdab88`.

````cpp
// Copyright 2015 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#include "third_party/blink/renderer/core/css/parser/css_at_rule_id.h"

#include <algorithm>
#include <iterator>
#include <optional>
#include <string_view>

#include "third_party/blink/renderer/core/css/parser/css_parser_context.h"
#include "third_party/blink/renderer/core/frame/web_feature.h"
#include "third_party/blink/renderer/platform/instrumentation/use_counter.h"
#include "third_party/blink/renderer/platform/runtime_enabled_features.h"

namespace blink {

namespace {

// Metadata for at-rules. Sorted by name for binary search.
struct AtRuleEntry {
  const char* name;
  CSSAtRuleID id;
  WebFeature feature;
};

// clang-format off
constexpr AtRuleEntry kAtRuleEntries[] = {
    {"-webkit-keyframes", CSSAtRuleID::kCSSAtRuleWebkitKeyframes, WebFeature::kCSSAtRuleWebkitKeyframes},
    {"annotation", CSSAtRuleID::kCSSAtRuleAnnotation, WebFeature::kCSSAtRuleAnnotation},
    {"bottom-center", CSSAtRuleID::kCSSAtRuleBottomCenter, WebFeature::kCSSAtRulePageMargin},
    {"bottom-left", CSSAtRuleID::kCSSAtRuleBottomLeft, WebFeature::kCSSAtRulePageMargin},
    {"bottom-left-corner", CSSAtRuleID::kCSSAtRuleBottomLeftCorner, WebFeature::kCSSAtRulePageMargin},
    {"bottom-right", CSSAtRuleID::kCSSAtRuleBottomRight, WebFeature::kCSSAtRulePageMargin},
    {"bottom-right-corner", CSSAtRuleID::kCSSAtRuleBottomRightCorner, WebFeature::kCSSAtRulePageMargin},
    {"character-variant", CSSAtRuleID::kCSSAtRuleCharacterVariant, WebFeature::kCSSAtRuleCharacterVariant},
    {"charset", CSSAtRuleID::kCSSAtRuleCharset, WebFeature::kCSSAtRuleCharset},
    {"container", CSSAtRuleID::kCSSAtRuleContainer, WebFeature::kCSSAtRuleContainer},
    {"counter-style", CSSAtRuleID::kCSSAtRuleCounterStyle, WebFeature::kCSSAtRuleCounterStyle},
    {"font-face", CSSAtRuleID::kCSSAtRuleFontFace, WebFeature::kCSSAtRuleFontFace},
    {"font-feature-values", CSSAtRuleID::kCSSAtRuleFontFeatureValues, WebFeature::kCSSAtRuleFontFeatureValues},
    {"font-palette-values", CSSAtRuleID::kCSSAtRuleFontPaletteValues, WebFeature::kCSSAtRuleFontPaletteValues},
    {"import", CSSAtRuleID::kCSSAtRuleImport, WebFeature::kCSSAtRuleImport},
    {"keyframes", CSSAtRuleID::kCSSAtRuleKeyframes, WebFeature::kCSSAtRuleKeyframes},
    {"layer", CSSAtRuleID::kCSSAtRuleLayer, WebFeature::kCSSCascadeLayers},
    {"left-bottom", CSSAtRuleID::kCSSAtRuleLeftBottom, WebFeature::kCSSAtRulePageMargin},
    {"left-middle", CSSAtRuleID::kCSSAtRuleLeftMiddle, WebFeature::kCSSAtRulePageMargin},
    {"left-top", CSSAtRuleID::kCSSAtRuleLeftTop, WebFeature::kCSSAtRulePageMargin},
    {"media", CSSAtRuleID::kCSSAtRuleMedia, WebFeature::kCSSAtRuleMedia},
    {"namespace", CSSAtRuleID::kCSSAtRuleNamespace, WebFeature::kCSSAtRuleNamespace},
    {"ornaments", CSSAtRuleID::kCSSAtRuleOrnaments, WebFeature::kCSSAtRuleOrnaments},
    {"page", CSSAtRuleID::kCSSAtRulePage, WebFeature::kCSSAtRulePage},
    {"position-try", CSSAtRuleID::kCSSAtRulePositionTry, WebFeature::kCSSAnchorPositioning},
    {"property", CSSAtRuleID::kCSSAtRuleProperty, WebFeature::kCSSAtRuleProperty},
    {"right-bottom", CSSAtRuleID::kCSSAtRuleRightBottom, WebFeature::kCSSAtRulePageMargin},
    {"right-middle", CSSAtRuleID::kCSSAtRuleRightMiddle, WebFeature::kCSSAtRulePageMargin},
    {"right-top", CSSAtRuleID::kCSSAtRuleRightTop, WebFeature::kCSSAtRulePageMargin},
    {"scope", CSSAtRuleID::kCSSAtRuleScope, WebFeature::kCSSAtRuleScope},
    {"starting-style", CSSAtRuleID::kCSSAtRuleStartingStyle, WebFeature::kCSSAtRuleStartingStyle},
    {"styleset", CSSAtRuleID::kCSSAtRuleStyleset, WebFeature::kCSSAtRuleStylistic},
    {"stylistic", CSSAtRuleID::kCSSAtRuleStylistic, WebFeature::kCSSAtRuleStylistic},
    {"supports", CSSAtRuleID::kCSSAtRuleSupports, WebFeature::kCSSAtRuleSupports},
    {"swash", CSSAtRuleID::kCSSAtRuleSwash, WebFeature::kCSSAtRuleSwash},
    {"top-center", CSSAtRuleID::kCSSAtRuleTopCenter, WebFeature::kCSSAtRulePageMargin},
    {"top-left", CSSAtRuleID::kCSSAtRuleTopLeft, WebFeature::kCSSAtRulePageMargin},
    {"top-left-corner", CSSAtRuleID::kCSSAtRuleTopLeftCorner, WebFeature::kCSSAtRulePageMargin},
    {"top-right", CSSAtRuleID::kCSSAtRuleTopRight, WebFeature::kCSSAtRulePageMargin},
    {"top-right-corner", CSSAtRuleID::kCSSAtRuleTopRightCorner, WebFeature::kCSSAtRulePageMargin},
    {"view-transition", CSSAtRuleID::kCSSAtRuleViewTransition, WebFeature::kCSSAtRuleViewTransition},
};
// clang-format on

// At-rules gated behind runtime flags.
// Sorted by name for consistency with kAtRuleEntries.
struct FlaggedAtRuleEntry {
  const char* name;
  CSSAtRuleID id;
  WebFeature feature;
  bool (*is_enabled)();
};

// clang-format off
constexpr FlaggedAtRuleEntry kFlaggedAtRuleEntries[] = {
    {"apply", CSSAtRuleID::kCSSAtRuleApplyMixin, WebFeature::kCSSMixins,
     &RuntimeEnabledFeatures::CSSMixinsEnabled},
    {"contents", CSSAtRuleID::kCSSAtRuleContents, WebFeature::kCSSMixins,
     &RuntimeEnabledFeatures::CSSMixinsEnabled},
    {"custom-media", CSSAtRuleID::kCSSAtRuleCustomMedia, WebFeature::kCSSCustomMedia,
     &RuntimeEnabledFeatures::CSSCustomMediaEnabled},
    {"function", CSSAtRuleID::kCSSAtRuleFunction, WebFeature::kCSSFunctions,
     &RuntimeEnabledFeatures::CSSFunctionsEnabled},
    {"location", CSSAtRuleID::kCSSAtRuleLocation, WebFeature::kCSSAtRuleRoute,
     &RuntimeEnabledFeatures::RouteMatchingEnabled},
    {"mixin", CSSAtRuleID::kCSSAtRuleMixin, WebFeature::kCSSMixins,
     &RuntimeEnabledFeatures::CSSMixinsEnabled},
    {"navigation", CSSAtRuleID::kCSSAtRuleNavigation, WebFeature::kCSSAtRuleRoute,
     &RuntimeEnabledFeatures::RouteMatchingEnabled},
    {"private", CSSAtRuleID::kCSSAtRulePrivate, WebFeature::kCSSAtPrivate,
     &RuntimeEnabledFeatures::CSSPrivateEnabled},
    {"result", CSSAtRuleID::kCSSAtRuleResult, WebFeature::kCSSMixins,
     &RuntimeEnabledFeatures::CSSMixinsEnabled},
};
// clang-format on

// Compile-time validation that tables are sorted for binary search.
constexpr auto AtRuleNameProjection = [](const auto& entry) {
  return std::string_view(entry.name);
};

static_assert(std::ranges::is_sorted(kAtRuleEntries, {}, AtRuleNameProjection),
              "kAtRuleEntries must be sorted by name for binary search");
static_assert(std::ranges::is_sorted(kFlaggedAtRuleEntries,
                                     {},
                                     AtRuleNameProjection),
              "kFlaggedAtRuleEntries must be sorted by name");

}  // namespace

CSSAtRuleID CssAtRuleID(StringView name) {
  // Binary search the main table.
  const auto* it = std::lower_bound(
      std::begin(kAtRuleEntries), std::end(kAtRuleEntries), name,
      [](const AtRuleEntry& entry, const StringView& target) {
        return CodeUnitCompareIgnoringAsciiCase(StringView(entry.name),
                                                target) < 0;
      });
  if (it != std::end(kAtRuleEntries) &&
      EqualIgnoringAsciiCase(name, it->name)) {
    return it->id;
  }
  // Linear search the smaller flagged entries table.
  for (const auto& entry : kFlaggedAtRuleEntries) {
    if (entry.is_enabled() && EqualIgnoringAsciiCase(name, entry.name)) {
      return entry.id;
    }
  }
  return CSSAtRuleID::kCSSAtRuleInvalid;
}

StringView CssAtRuleIDToString(CSSAtRuleID id) {
  for (const auto& entry : kAtRuleEntries) {
    if (entry.id == id) {
      return entry.name;
    }
  }
  for (const auto& entry : kFlaggedAtRuleEntries) {
    if (entry.id == id) {
      return entry.name;
    }
  }
  NOTREACHED();
}

namespace {

std::optional<WebFeature> AtRuleFeature(CSSAtRuleID rule_id) {
  for (const auto& entry : kAtRuleEntries) {
    if (entry.id == rule_id) {
      return entry.feature;
    }
  }
  for (const auto& entry : kFlaggedAtRuleEntries) {
    if (entry.id == rule_id) {
      return entry.feature;
    }
  }
  NOTREACHED();
}

}  // namespace

void CountAtRule(const CSSParserContext* context, CSSAtRuleID rule_id) {
  if (std::optional<WebFeature> feature = AtRuleFeature(rule_id)) {
    context->Count(*feature);
  }
}

}  // namespace blink
````
