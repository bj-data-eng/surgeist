# Pinned browser source witnesses for media and collapsed table border images

Captured 2026-10-11 UTC (2026-10-10 America/New_York) from official immutable sources. This is bounded implementation evidence, not normative CSS or an engine-wide conformance claim. Original source segments and complete file-level legal headers are retained exactly. No captured code was compiled or executed or incorporated into the product. [WebKit legal material](../licenses/webkit/NOTICE.md) and [Chromium BSD license](../licenses/chromium/LICENSE.txt) accompany these excerpts.

Blink conditionally emits the extra clear event only for equal effective values and completes valid request calls. Its invalid-value category is TypeMismatchError, whereas the selected Media Queries draft names TypeError; Surgeist retains the draft semantic category. Blink stores the preference overlay on Page and invalidates media/color-scheme consumers; that is evidence of overlay/invalidation mechanics, not authority to replace the draft’s origin-scoped application. WebKit table decoration skips BorderPainter in collapsed mode. Blink’s table background delegates to BoxFragmentPainter, whose border gate is fed by BoxDecorationData’s `!fragment.HasCollapsedBorders()` condition. These witnesses support suppressing image borders while retaining collapsed border geometry.

## <a id="blink-table-painters-cc"></a>Blink: third_party/blink/renderer/core/paint/table_painters.cc

Immutable source: [original](https://chromium.googlesource.com/chromium/src/+/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/paint/table_painters.cc). Full revision `7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2`. Original SHA-256 `3950a9f48b2945db36f0908e8722168284951df400c9cfed1304bbcc0d5c815f` (34808 bytes). Unlisted source is omitted.

Original lines 1–4: complete file-level legal header and original leading comments. Segment SHA-256 `162316597b246776d11c316168bbbb2e3768765b1d2c513412a61947b1915606`.

````cpp
// Copyright 2020 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

````

Original lines 421–464: complete void TablePainter::PaintBoxDecorationBackground. Segment SHA-256 `354fd0ca604db15d27c65204615d9e91a1368c4d84fb199e015b48af664a2eb6`.

````cpp
void TablePainter::PaintBoxDecorationBackground(
    const PaintInfo& paint_info,
    const PhysicalRect& paint_rect,
    const BoxDecorationData& box_decoration_data) {
  WritingModeConverter converter(fragment_.Style().GetWritingDirection(),
                                 fragment_.Size());
  PhysicalRect grid_paint_rect =
      converter.ToPhysical(fragment_.TableGridRect());
  grid_paint_rect.offset += paint_rect.offset;

  // Paint the table background on the grid-rect.
  if (box_decoration_data.ShouldPaint()) {
    BoxFragmentPainter(fragment_).PaintBoxDecorationBackgroundWithRectImpl(
        paint_info, grid_paint_rect, box_decoration_data);
  }

  // Optimization: only traverse colgroups with backgrounds.
  const GCedTableColumnGeometries* column_geometries_original =
      fragment_.TableColumnGeometries();
  TableColumnGeometries column_geometries_with_background;
  if (column_geometries_original) {
    for (const auto& column_geometry : *column_geometries_original) {
      if (column_geometry.node.Style().HasBoxDecorationBackground()) {
        column_geometries_with_background.push_back(column_geometry);
      }
    }
  }

  if (column_geometries_with_background.empty())
    return;

  // Paint <colgroup>/<col> backgrounds.
  PhysicalRect columns_paint_rect = ComputeColumnsRect(fragment_);
  columns_paint_rect.offset += paint_rect.offset;
  for (const PhysicalFragmentLink& child : fragment_.Children()) {
    if (!child.fragment->IsTableSection()) {
      continue;
    }
    TableSectionPainter(To<PhysicalBoxFragment>(*child.fragment))
        .PaintColumnsBackground(paint_info, paint_rect.offset + child.offset,
                                columns_paint_rect,
                                column_geometries_with_background);
  }
}
````

File-specific license: [Chromium BSD-style license](../licenses/chromium/LICENSE.txt).

## <a id="blink-preference-object-cc"></a>Blink: third_party/blink/renderer/core/preferences/preference_object.cc

Immutable source: [original](https://chromium.googlesource.com/chromium/src/+/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/preferences/preference_object.cc). Full revision `7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2`. Original SHA-256 `65ca95325570e41a3e05cfcadf1a530b3ddb5dcbccbe8415b055701e166da371` (17971 bytes). Unlisted source is omitted.

Original lines 1–479: complete source with legal header. Segment SHA-256 `65ca95325570e41a3e05cfcadf1a530b3ddb5dcbccbe8415b055701e166da371`.

````cpp
// Copyright 2023 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#include "third_party/blink/renderer/core/preferences/preference_object.h"

#include "third_party/blink/renderer/bindings/core/v8/frozen_array.h"
#include "third_party/blink/renderer/bindings/core/v8/script_promise.h"
#include "third_party/blink/renderer/core/css/media_feature_names.h"
#include "third_party/blink/renderer/core/css/media_values.h"
#include "third_party/blink/renderer/core/css/media_values_cached.h"
#include "third_party/blink/renderer/core/css/media_values_dynamic.h"
#include "third_party/blink/renderer/core/dom/dom_exception.h"
#include "third_party/blink/renderer/core/dom/events/event.h"
#include "third_party/blink/renderer/core/event_target_names.h"
#include "third_party/blink/renderer/core/frame/local_dom_window.h"
#include "third_party/blink/renderer/core/frame/local_frame.h"
#include "third_party/blink/renderer/core/frame/settings.h"
#include "third_party/blink/renderer/core/page/page.h"
#include "third_party/blink/renderer/core/preferences/preference_names.h"
#include "third_party/blink/renderer/core/preferences/preference_overrides.h"
#include "third_party/blink/renderer/core/preferences/preference_values.h"
#include "third_party/blink/renderer/platform/bindings/script_state.h"
#include "third_party/blink/renderer/platform/heap/garbage_collected.h"
#include "third_party/blink/renderer/platform/runtime_enabled_features.h"

namespace blink {

AtomicString ColorSchemeToString(
    mojom::blink::PreferredColorScheme colorScheme) {
  switch (colorScheme) {
    case mojom::PreferredColorScheme::kLight:
      return preference_values::kLight;
    case mojom::PreferredColorScheme::kDark:
      return preference_values::kDark;
    default:
      NOTREACHED();
  }
}

AtomicString ContrastToString(mojom::blink::PreferredContrast contrast) {
  switch (contrast) {
    case mojom::PreferredContrast::kMore:
      return preference_values::kMore;
    case mojom::PreferredContrast::kLess:
      return preference_values::kLess;
    case mojom::PreferredContrast::kCustom:
      return preference_values::kCustom;
    case mojom::PreferredContrast::kNoPreference:
      return preference_values::kNoPreference;
    default:
      NOTREACHED();
  }
}

PreferenceObject::PreferenceObject(ExecutionContext* execution_context,
                                   AtomicString name)
    : ExecutionContextLifecycleObserver(execution_context), name_(name) {
  LocalFrame* frame = nullptr;
  if (execution_context && !execution_context->IsContextDestroyed()) {
    frame = To<LocalDOMWindow>(*execution_context).GetFrame();
  }
  media_values_ = MediaValues::CreateDynamicIfFrameExists(frame);
  preferred_color_scheme_ = media_values_->GetPreferredColorScheme();
  preferred_contrast_ = media_values_->GetPreferredContrast();
  prefers_reduced_data_ = media_values_->PrefersReducedData();
  prefers_reduced_motion_ = media_values_->PrefersReducedMotion();
  prefers_reduced_transparency_ = media_values_->PrefersReducedTransparency();
}

PreferenceObject::~PreferenceObject() = default;

AtomicString PreferenceObject::override(ScriptState* script_state) {
  CHECK(RuntimeEnabledFeatures::WebPreferencesEnabled());
  if (!script_state || !script_state->ContextIsValid()) {
    return g_null_atom;
  }
  auto* execution_context = ExecutionContext::From(script_state);
  if (!execution_context || execution_context->IsContextDestroyed()) {
    return g_null_atom;
  }
  auto* window = DynamicTo<LocalDOMWindow>(execution_context);
  if (!window) {
    return g_null_atom;
  }

  const PreferenceOverrides* overrides =
      window->GetFrame()->GetPage()->GetPreferenceOverrides();

  if (!overrides) {
    return g_null_atom;
  }

  if (name_ == preference_names::kColorScheme) {
    std::optional<mojom::blink::PreferredColorScheme> color_scheme =
        overrides->GetPreferredColorScheme();
    if (!color_scheme.has_value()) {
      return g_null_atom;
    }

    return ColorSchemeToString(color_scheme.value());
  } else if (name_ == preference_names::kContrast) {
    std::optional<mojom::blink::PreferredContrast> contrast =
        overrides->GetPreferredContrast();
    if (!contrast.has_value()) {
      return g_null_atom;
    }

    return ContrastToString(contrast.value());
  } else if (name_ == preference_names::kReducedMotion) {
    std::optional<bool> reduced_motion = overrides->GetPrefersReducedMotion();
    if (!reduced_motion.has_value()) {
      return g_null_atom;
    }

    return reduced_motion.value() ? preference_values::kReduce
                                  : preference_values::kNoPreference;
  } else if (name_ == preference_names::kReducedTransparency) {
    std::optional<bool> reduced_transparency =
        overrides->GetPrefersReducedTransparency();
    if (!reduced_transparency.has_value()) {
      return g_null_atom;
    }

    return reduced_transparency.value() ? preference_values::kReduce
                                        : preference_values::kNoPreference;
  } else if (name_ == preference_names::kReducedData) {
    std::optional<bool> reduced_data = overrides->GetPrefersReducedData();
    if (!reduced_data.has_value()) {
      return g_null_atom;
    }

    return reduced_data ? preference_values::kReduce
                        : preference_values::kNoPreference;
  } else {
    NOTREACHED();
  }
}

AtomicString PreferenceObject::value(ScriptState* script_state) {
  CHECK(RuntimeEnabledFeatures::WebPreferencesEnabled());
  if (!script_state || !script_state->ContextIsValid()) {
    return g_empty_atom;
  }
  auto* execution_context = ExecutionContext::From(script_state);
  if (!execution_context || execution_context->IsContextDestroyed()) {
    return g_empty_atom;
  }
  auto* window = DynamicTo<LocalDOMWindow>(execution_context);
  if (!window) {
    return g_empty_atom;
  }

  if (name_ == preference_names::kColorScheme) {
    return ColorSchemeToString(preferred_color_scheme_);
  } else if (name_ == preference_names::kContrast) {
    return ContrastToString(preferred_contrast_);
  } else if (name_ == preference_names::kReducedMotion) {
    return prefers_reduced_motion_ ? preference_values::kReduce
                                   : preference_values::kNoPreference;
  } else if (name_ == preference_names::kReducedTransparency) {
    return prefers_reduced_transparency_ ? preference_values::kReduce
                                         : preference_values::kNoPreference;
  } else if (name_ == preference_names::kReducedData) {
    return prefers_reduced_data_ ? preference_values::kReduce
                                 : preference_values::kNoPreference;
  } else {
    NOTREACHED();
  }
}

void PreferenceObject::clearOverride(ScriptState* script_state) {
  CHECK(RuntimeEnabledFeatures::WebPreferencesEnabled());
  if (!script_state || !script_state->ContextIsValid()) {
    return;
  }
  auto* execution_context = ExecutionContext::From(script_state);
  if (!execution_context || execution_context->IsContextDestroyed()) {
    return;
  }
  auto* window = DynamicTo<LocalDOMWindow>(execution_context);
  if (!window) {
    return;
  }

  const PreferenceOverrides* overrides =
      window->GetFrame()->GetPage()->GetPreferenceOverrides();

  if (!overrides) {
    return;
  }

  bool value_unchanged;
  if (name_ == preference_names::kColorScheme) {
    std::optional<mojom::blink::PreferredColorScheme> color_scheme =
        overrides->GetPreferredColorScheme();
    if (!color_scheme.has_value()) {
      return;
    }

    window->GetFrame()->GetPage()->SetPreferenceOverride(
        media_feature_names::kPrefersColorSchemeMediaFeature, String());
    value_unchanged =
        (color_scheme.value() == media_values_->GetPreferredColorScheme());
  } else if (name_ == preference_names::kContrast) {
    std::optional<mojom::blink::PreferredContrast> contrast =
        overrides->GetPreferredContrast();
    if (!contrast.has_value()) {
      return;
    }

    window->GetFrame()->GetPage()->SetPreferenceOverride(
        media_feature_names::kPrefersContrastMediaFeature, String());
    value_unchanged =
        (contrast.value() == media_values_->GetPreferredContrast());
  } else if (name_ == preference_names::kReducedMotion) {
    std::optional<bool> reduced_motion = overrides->GetPrefersReducedMotion();
    if (!reduced_motion.has_value()) {
      return;
    }

    window->GetFrame()->GetPage()->SetPreferenceOverride(
        media_feature_names::kPrefersReducedMotionMediaFeature, String());
    value_unchanged =
        (reduced_motion.value() == media_values_->PrefersReducedMotion());
  } else if (name_ == preference_names::kReducedTransparency) {
    std::optional<bool> reduced_transparency =
        overrides->GetPrefersReducedTransparency();
    if (!reduced_transparency.has_value()) {
      return;
    }

    window->GetFrame()->GetPage()->SetPreferenceOverride(
        media_feature_names::kPrefersReducedTransparencyMediaFeature, String());
    value_unchanged = (reduced_transparency.value() ==
                       media_values_->PrefersReducedTransparency());
  } else if (name_ == preference_names::kReducedData) {
    std::optional<bool> reduced_data = overrides->GetPrefersReducedData();
    if (!reduced_data.has_value()) {
      return;
    }

    window->GetFrame()->GetPage()->SetPreferenceOverride(
        media_feature_names::kPrefersReducedDataMediaFeature, String());
    value_unchanged =
        (reduced_data.value() == media_values_->PrefersReducedData());
  } else {
    NOTREACHED();
  }
  if (value_unchanged) {
    DispatchEvent(*Event::Create(event_type_names::kChange));
  }
}

ScriptPromise<IDLUndefined> PreferenceObject::requestOverride(
    ScriptState* script_state,
    std::optional<AtomicString> value) {
  CHECK(RuntimeEnabledFeatures::WebPreferencesEnabled());
  if (!script_state || !script_state->ContextIsValid()) {
    return EmptyPromise();
  }
  auto* execution_context = ExecutionContext::From(script_state);
  if (!execution_context || execution_context->IsContextDestroyed()) {
    return EmptyPromise();
  }
  auto* window = DynamicTo<LocalDOMWindow>(execution_context);
  if (!window) {
    return EmptyPromise();
  }

  if (!value.has_value() || value.value().empty()) {
    clearOverride(script_state);
    return ToResolvedUndefinedPromise(script_state);
  }

  AtomicString feature_name;
  AtomicString new_value;
  bool has_existing_override = false;
  bool value_same_as_existing_override = false;

  AtomicString existing_value;

  if (validValues().AsVector().Contains(value.value())) {
    new_value = value.value();
  }

  const PreferenceOverrides* overrides =
      window->GetFrame()->GetPage()->GetPreferenceOverrides();
  if (name_ == preference_names::kColorScheme) {
    feature_name = media_feature_names::kPrefersColorSchemeMediaFeature;

    if (overrides) {
      auto override = overrides->GetPreferredColorScheme();
      if (override.has_value()) {
        has_existing_override = true;
        if (new_value == ColorSchemeToString(override.value()).GetString()) {
          value_same_as_existing_override = true;
        }
      }
    }
    existing_value = ColorSchemeToString(preferred_color_scheme_);
  } else if (name_ == preference_names::kContrast) {
    feature_name = media_feature_names::kPrefersContrastMediaFeature;

    if (overrides) {
      auto override = overrides->GetPreferredContrast();
      if (override.has_value()) {
        has_existing_override = true;
        if (new_value == ContrastToString(override.value()).GetString()) {
          value_same_as_existing_override = true;
        }
      }
    }
    existing_value = ContrastToString(preferred_contrast_);
  } else if (name_ == preference_names::kReducedMotion) {
    feature_name = media_feature_names::kPrefersReducedMotionMediaFeature;

    if (overrides) {
      auto override = overrides->GetPrefersReducedMotion();
      if (override.has_value()) {
        has_existing_override = true;
        if ((new_value == preference_values::kReduce && override.value()) ||
            (new_value == preference_values::kNoPreference &&
             !override.value())) {
          value_same_as_existing_override = true;
        }
      }
    }
    existing_value = prefers_reduced_motion_ ? preference_values::kReduce
                                             : preference_values::kNoPreference;
  } else if (name_ == preference_names::kReducedTransparency) {
    feature_name = media_feature_names::kPrefersReducedTransparencyMediaFeature;

    if (overrides) {
      auto override = overrides->GetPrefersReducedTransparency();
      if (override.has_value()) {
        has_existing_override = true;
        if ((new_value == preference_values::kReduce && override.value()) ||
            (new_value == preference_values::kNoPreference &&
             !override.value())) {
          value_same_as_existing_override = true;
        }
      }
    }
    existing_value = prefers_reduced_transparency_
                         ? preference_values::kReduce
                         : preference_values::kNoPreference;
  } else if (name_ == preference_names::kReducedData) {
    feature_name = media_feature_names::kPrefersReducedDataMediaFeature;

    if (overrides) {
      auto override = overrides->GetPrefersReducedData();
      if (override.has_value()) {
        has_existing_override = true;
        if ((new_value == preference_values::kReduce && override.value()) ||
            (new_value == preference_values::kNoPreference &&
             !override.value())) {
          value_same_as_existing_override = true;
        }
      }
    }
    existing_value = prefers_reduced_data_ ? preference_values::kReduce
                                           : preference_values::kNoPreference;
  } else {
    NOTREACHED();
  }

  if (new_value.empty()) {
    return ScriptPromise<IDLUndefined>::RejectWithDOMException(
        script_state, MakeGarbageCollected<DOMException>(
                          DOMExceptionCode::kTypeMismatchError,
                          StrCat({value.value(), " is not a valid value."})));
  }

  if (!value_same_as_existing_override) {
    window->GetFrame()->GetPage()->SetPreferenceOverride(feature_name,
                                                         new_value.GetString());
  }

  if (!has_existing_override && new_value == existing_value) {
    DispatchEvent(*Event::Create(event_type_names::kChange));
  }

  return ToResolvedUndefinedPromise(script_state);
}

const FrozenArray<IDLString>& PreferenceObject::validValues() {
  CHECK(RuntimeEnabledFeatures::WebPreferencesEnabled());
  if (valid_values_) [[likely]] {
    return *valid_values_.Get();
  }

  FrozenArray<IDLString>::VectorType valid_values;
  if (name_ == preference_names::kColorScheme) {
    valid_values.push_back(preference_values::kLight);
    valid_values.push_back(preference_values::kDark);
  } else if (name_ == preference_names::kContrast) {
    valid_values.push_back(preference_values::kMore);
    valid_values.push_back(preference_values::kLess);
    valid_values.push_back(preference_values::kNoPreference);
  } else if (name_ == preference_names::kReducedMotion) {
    valid_values.push_back(preference_values::kReduce);
    valid_values.push_back(preference_values::kNoPreference);
  } else if (name_ == preference_names::kReducedTransparency) {
    valid_values.push_back(preference_values::kReduce);
    valid_values.push_back(preference_values::kNoPreference);
  } else if (name_ == preference_names::kReducedData) {
    valid_values.push_back(preference_values::kReduce);
    valid_values.push_back(preference_values::kNoPreference);
  } else {
    NOTREACHED();
  }
  valid_values_ =
      MakeGarbageCollected<FrozenArray<IDLString>>(std::move(valid_values));
  return *valid_values_.Get();
}

void PreferenceObject::PreferenceMaybeChanged() {
  CHECK(RuntimeEnabledFeatures::WebPreferencesEnabled());
  if (!GetExecutionContext() || GetExecutionContext()->IsContextDestroyed()) {
    return;
  }
  auto* window = DynamicTo<LocalDOMWindow>(GetExecutionContext());
  if (!window) {
    return;
  }

  if (name_ == preference_names::kColorScheme) {
    if (preferred_color_scheme_ == media_values_->GetPreferredColorScheme()) {
      return;
    }
  } else if (name_ == preference_names::kContrast) {
    if (preferred_contrast_ == media_values_->GetPreferredContrast()) {
      return;
    }
  } else if (name_ == preference_names::kReducedMotion) {
    if (prefers_reduced_motion_ == media_values_->PrefersReducedMotion()) {
      return;
    }
  } else if (name_ == preference_names::kReducedTransparency) {
    if (prefers_reduced_transparency_ ==
        media_values_->PrefersReducedTransparency()) {
      return;
    }
  } else if (name_ == preference_names::kReducedData) {
    if (prefers_reduced_data_ == media_values_->PrefersReducedData()) {
      return;
    }
  } else {
    NOTREACHED();
  }
  preferred_color_scheme_ = media_values_->GetPreferredColorScheme();
  preferred_contrast_ = media_values_->GetPreferredContrast();
  prefers_reduced_data_ = media_values_->PrefersReducedData();
  prefers_reduced_motion_ = media_values_->PrefersReducedMotion();
  prefers_reduced_transparency_ = media_values_->PrefersReducedTransparency();
  DispatchEvent(*Event::Create(event_type_names::kChange));
}

void PreferenceObject::Trace(Visitor* visitor) const {
  EventTarget::Trace(visitor);
  ExecutionContextLifecycleObserver::Trace(visitor);
  visitor->Trace(valid_values_);
  visitor->Trace(media_values_);
}

const AtomicString& PreferenceObject::InterfaceName() const {
  return event_target_names::kPreferenceObject;
}

void PreferenceObject::ContextDestroyed() {
  RemoveAllEventListeners();
}

ExecutionContext* PreferenceObject::GetExecutionContext() const {
  return ExecutionContextLifecycleObserver::GetExecutionContext();
}

}  // namespace blink
````

File-specific license: [Chromium BSD-style license](../licenses/chromium/LICENSE.txt).

## <a id="blink-preference-manager-cc"></a>Blink: third_party/blink/renderer/core/preferences/preference_manager.cc

Immutable source: [original](https://chromium.googlesource.com/chromium/src/+/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/preferences/preference_manager.cc). Full revision `7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2`. Original SHA-256 `137cb47dd664e3cc562a791c3af25eb53ad02acd18cd03115af521a2e9de3d0f` (2334 bytes). Unlisted source is omitted.

Original lines 1–69: complete source with legal header. Segment SHA-256 `137cb47dd664e3cc562a791c3af25eb53ad02acd18cd03115af521a2e9de3d0f`.

````cpp
// Copyright 2023 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#include "third_party/blink/renderer/core/preferences/preference_manager.h"
#include "third_party/blink/renderer/core/preferences/preference_names.h"
#include "third_party/blink/renderer/core/preferences/preference_object.h"
#include "third_party/blink/renderer/platform/heap/garbage_collected.h"
#include "third_party/blink/renderer/platform/runtime_enabled_features.h"

namespace blink {

PreferenceManager::PreferenceManager(ExecutionContext* executionContext) {
  color_scheme_ = MakeGarbageCollected<PreferenceObject>(
      executionContext, preference_names::kColorScheme);
  contrast_ = MakeGarbageCollected<PreferenceObject>(
      executionContext, preference_names::kContrast);
  reduced_motion_ = MakeGarbageCollected<PreferenceObject>(
      executionContext, preference_names::kReducedMotion);
  reduced_transparency_ = MakeGarbageCollected<PreferenceObject>(
      executionContext, preference_names::kReducedTransparency);
  reduced_data_ = MakeGarbageCollected<PreferenceObject>(
      executionContext, preference_names::kReducedData);
}

PreferenceManager::~PreferenceManager() = default;

void PreferenceManager::Trace(Visitor* visitor) const {
  visitor->Trace(color_scheme_);
  visitor->Trace(contrast_);
  visitor->Trace(reduced_motion_);
  visitor->Trace(reduced_transparency_);
  visitor->Trace(reduced_data_);
  ScriptWrappable::Trace(visitor);
}

PreferenceObject* PreferenceManager::colorScheme() {
  return color_scheme_.Get();
}

PreferenceObject* PreferenceManager::contrast() {
  return contrast_.Get();
}

PreferenceObject* PreferenceManager::reducedMotion() {
  return reduced_motion_.Get();
}

PreferenceObject* PreferenceManager::reducedTransparency() {
  return reduced_transparency_.Get();
}

PreferenceObject* PreferenceManager::reducedData() {
  return reduced_data_.Get();
}

void PreferenceManager::PreferenceMaybeChanged() {
  if (!RuntimeEnabledFeatures::WebPreferencesEnabled()) {
    return;
  }

  colorScheme()->PreferenceMaybeChanged();
  contrast()->PreferenceMaybeChanged();
  reducedMotion()->PreferenceMaybeChanged();
  reducedTransparency()->PreferenceMaybeChanged();
  reducedData()->PreferenceMaybeChanged();
}

}  // namespace blink
````

File-specific license: [Chromium BSD-style license](../licenses/chromium/LICENSE.txt).

## <a id="blink-page-cc"></a>Blink: third_party/blink/renderer/core/page/page.cc

Immutable source: [original](https://chromium.googlesource.com/chromium/src/+/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/page/page.cc). Full revision `7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2`. Original SHA-256 `cce4f691e8b3b9b75285a0ee003ba7f035ee8a0e2f9bfcd5428c4623ec4fd4ef` (59624 bytes). Unlisted source is omitted.

Original lines 1–21: complete file-level legal header and original leading comments. Segment SHA-256 `93e4a0485c501eaedf7304b1dac20aed09bb9654be680048ecd3201ce40b0da8`.

````cpp
/*
 * Copyright (C) 2006, 2007, 2008, 2009, 2010, 2011, 2012, 2013 Apple Inc. All
 * Rights Reserved.
 * Copyright (C) 2008 Torch Mobile Inc. All rights reserved.
 * (http://www.torchmobile.com/)
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Library General Public
 * License as published by the Free Software Foundation; either
 * version 2 of the License, or (at your option) any later version.
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

````

Original lines 1483–1503: complete void Page::SetPreferenceOverride. Segment SHA-256 `3d8b73174aaa8bd2dab72cf36b221413e99907b9b0fc2366d602d23a4369b02b`.

````cpp
void Page::SetPreferenceOverride(const AtomicString& media_feature,
                                 const String& value) {
  if (!preference_overrides_) {
    if (value.empty()) {
      return;
    }
    preference_overrides_ = std::make_unique<PreferenceOverrides>();
  }

  const Document* document = nullptr;
  if (auto* local_frame = DynamicTo<LocalFrame>(MainFrame())) {
    document = local_frame->GetDocument();
  }

  preference_overrides_->SetOverride(media_feature, value, document);
  if (media_feature == "prefers-color-scheme") {
    SettingsChanged(ChangeType::kColorScheme);
  } else {
    SettingsChanged(ChangeType::kMediaQuery);
  }
}
````

File-specific license: [GNU Library General Public License version 2 or later](../licenses/chromium/blink-LICENSE_FOR_ABOUT_CREDITS.txt).

## <a id="webkit-rendertable-cpp"></a>WebKit: Source/WebCore/rendering/RenderTable.cpp

Immutable source: [original](https://raw.githubusercontent.com/WebKit/WebKit/437139e30888c5e1b4c35f889f32e1d0c39d018e/Source/WebCore/rendering/RenderTable.cpp). Full revision `437139e30888c5e1b4c35f889f32e1d0c39d018e`. Original SHA-256 `ddd270f990dd9fd9d0cd095cf4ac69e03672b4740ad7cdc92bbd7f5f814e37cd` (78048 bytes). Unlisted source is omitted.

Original lines 1–26: complete file-level legal header and original leading comments. Segment SHA-256 `f56d045b4fc2d1d17e19acba6faf04bf981b7bffc2100e9defb93facfe256401`.

````cpp
/*
 * Copyright (C) 1997 Martin Jones (mjones@kde.org)
 *           (C) 1997 Torben Weis (weis@kde.org)
 *           (C) 1998 Waldo Bastian (bastian@kde.org)
 *           (C) 1999 Lars Knoll (knoll@kde.org)
 *           (C) 1999 Antti Koivisto (koivisto@kde.org)
 * Copyright (C) 2003-2026 Apple Inc. All rights reserved.
 * Copyright (C) 2014-2019 Google Inc. All rights reserved.
 * Copyright (C) 2006 Alexey Proskuryakov (ap@nypop.com)
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

````

Original lines 955–988: complete void RenderTable::paintBoxDecorations. Segment SHA-256 `d014faf1d55e104bb647a6e631766df894c93b5395cd75c004c63942852c5e45`.

````cpp
void RenderTable::paintBoxDecorations(PaintInfo& paintInfo, const LayoutPoint& paintOffset)
{
    if (!paintInfo.shouldPaintWithinRoot(*this))
        return;

    LayoutRect rect(paintOffset, borderBoxSize());
    adjustBorderBoxRectForPainting(rect);

    BackgroundPainter backgroundPainter { *this, paintInfo };

    auto bleedAvoidance = determineBleedAvoidance(paintInfo.context());
    if (!BackgroundPainter::boxShadowShouldBeAppliedToBackground(*this, rect.location(), bleedAvoidance, { }))
        backgroundPainter.paintBoxShadow(rect, style(), Style::ShadowStyle::Normal);

    GraphicsContextStateSaver stateSaver(paintInfo.context(), false);
    if (bleedAvoidance == BleedAvoidance::UseTransparencyLayer) {
        // To avoid the background color bleeding out behind the border, we'll render background and border
        // into a transparency layer, and then clip that in one go (which requires setting up the clip before
        // beginning the layer).
        stateSaver.save();
        auto borderShape = BorderShape::shapeForBorderRect(style(), rect);
        borderShape.clipToOuterShape(paintInfo.context(), style().deviceScaleFactor());
        paintInfo.context().beginTransparencyLayer(1);
    }

    backgroundPainter.paintBackground(rect, bleedAvoidance);
    backgroundPainter.paintBoxShadow(rect, style(), Style::ShadowStyle::Inset);

    if (style().border().hasVisibleBorderDecoration() && !collapseBorders())
        BorderPainter { *this, paintInfo }.paintBorder(rect, style());

    if (bleedAvoidance == BleedAvoidance::UseTransparencyLayer)
        paintInfo.context().endTransparencyLayer();
}
````

File-specific license: [GNU Library General Public License version 2 or later](../licenses/webkit/LICENSE-LGPL-2.txt).

## <a id="blink-box-fragment-painter-cc"></a>Blink: third_party/blink/renderer/core/paint/box_fragment_painter.cc

Immutable source: [original](https://chromium.googlesource.com/chromium/src/+/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/paint/box_fragment_painter.cc). Full revision `7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2`. Original SHA-256 `dd1db46d0fc18a2a60c7baa7caa814aa92dc392e07aaba36e6d0db52b6d74486` (129071 bytes). Unlisted source is omitted.

Original lines 1–4: complete file-level legal header and original leading comments. Segment SHA-256 `01b7411973374ce9c7e0892d434ccffd04d7fb76545d1b470ef267fe2fa6da07`.

````cpp
// Copyright 2017 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

````

Original lines 1675–1773: complete void BoxFragmentPainter::PaintBoxDecorationBackgroundWithRectImpl. Segment SHA-256 `dc66faeb5dda310891294f4ec8cf933ff44590cf0c3da16133992fe8de3cc8aa`.

````cpp
void BoxFragmentPainter::PaintBoxDecorationBackgroundWithRectImpl(
    const PaintInfo& paint_info,
    const PhysicalRect& paint_rect,
    const BoxDecorationData& box_decoration_data) {
  const LayoutObject& layout_object = *box_fragment_.GetLayoutObject();
  const LayoutBox& layout_box = To<LayoutBox>(layout_object);

  const ComputedStyle& style = box_fragment_.Style();

  GraphicsContextStateSaver state_saver(paint_info.context, false);

  if (box_decoration_data.ShouldPaintShadow()) {
    std::optional<BorderShapeReferenceRects> border_shape_rects =
        ComputeBorderShapeReferenceRects(paint_rect, style, layout_object);
    PaintNormalBoxShadow(paint_info, paint_rect, style, border_shape_rects,
                         box_fragment_.SidesToInclude(),
                         !box_decoration_data.ShouldPaintBackground());
  }

  bool needs_end_layer = false;
  if (!box_decoration_data.IsPaintingBackgroundInContentsSpace() &&
      BleedAvoidanceIsClipping(
          box_decoration_data.GetBackgroundBleedAvoidance())) {
    state_saver.Save();

    ContouredRect border = ContouredBorderGeometry::PixelSnappedContouredBorder(
        style, paint_rect, box_fragment_.SidesToInclude());
    paint_info.context.ClipContouredRect(border);

    if (box_decoration_data.GetBackgroundBleedAvoidance() ==
        kBackgroundBleedClipLayer) {
      paint_info.context.BeginLayer();
      needs_end_layer = true;
    }
  }

  gfx::Rect snapped_paint_rect = ToPixelSnappedRect(paint_rect);
  ThemePainter& theme_painter = LayoutTheme::GetTheme().Painter();
  bool theme_painted =
      box_decoration_data.HasAppearance() &&
      !theme_painter.Paint(layout_box, paint_info, snapped_paint_rect);
  if (!theme_painted) {
    if (box_decoration_data.ShouldPaintBackground()) {
      PaintBackground(paint_info, paint_rect,
                      box_decoration_data.BackgroundColor(),
                      box_decoration_data.GetBackgroundBleedAvoidance());
    }
    if (box_decoration_data.HasAppearance()) {
      theme_painter.PaintDecorations(layout_box.GetNode(),
                                     layout_box.GetDocument(), style,
                                     paint_info, snapped_paint_rect);
    }
  }

  if (box_decoration_data.ShouldPaintShadow()) {
    if (layout_box.IsTableCell()) {
      PhysicalRect inner_rect = paint_rect;
      inner_rect.Contract(layout_box.BorderOutsets());
      // PaintInsetBoxShadowWithInnerRect doesn't subtract borders before
      // painting. We have to use it here after subtracting collapsed borders
      // above. PaintInsetBoxShadowWithBorderRect below subtracts the borders
      // specified on the style object, which doesn't account for border
      // collapsing.
      BoxPainterBase::PaintInsetBoxShadowWithInnerRect(paint_info, inner_rect,
                                                       style);
    } else {
      std::optional<BorderShapeReferenceRects> border_shape_rects =
          ComputeBorderShapeReferenceRects(paint_rect, style, layout_object);
      PaintInsetBoxShadowWithBorderRect(paint_info, paint_rect, style,
                                        border_shape_rects,
                                        box_fragment_.SidesToInclude());
    }
  }

  // The theme will tell us whether or not we should also paint the CSS
  // border.
  if (box_decoration_data.ShouldPaintBorder()) {
    if (!theme_painted) {
      theme_painted =
          box_decoration_data.HasAppearance() &&
          !LayoutTheme::GetTheme().Painter().PaintBorderOnly(
              layout_box.GetNode(), style, paint_info, snapped_paint_rect);
    }
    if (!theme_painted) {
      Node* generating_node = layout_object.GeneratingNode();
      const Document& document = layout_object.GetDocument();
      std::optional<BorderShapeReferenceRects> border_shape_rects =
          ComputeBorderShapeReferenceRects(paint_rect, style, layout_object);
      PaintBorder(*box_fragment_.GetLayoutObject(), document, generating_node,
                  paint_info, paint_rect, style,
                  box_decoration_data.GetBackgroundBleedAvoidance(),
                  box_fragment_.SidesToInclude(),
                  border_shape_rects ? &*border_shape_rects : nullptr);
    }
  }

  if (needs_end_layer)
    paint_info.context.EndLayer();
}
````

File-specific license: [Chromium BSD-style license](../licenses/chromium/LICENSE.txt).

## <a id="blink-box-decoration-data-h"></a>Blink: third_party/blink/renderer/core/paint/box_decoration_data.h

Immutable source: [original](https://chromium.googlesource.com/chromium/src/+/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/paint/box_decoration_data.h). Full revision `7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2`. Original SHA-256 `ed610afe99ce2f8fa46b062eebcfce9b29e26fd55b60fbf24bd254351cb770d9` (6855 bytes). Unlisted source is omitted.

Original lines 1–177: complete source with legal header. Segment SHA-256 `ed610afe99ce2f8fa46b062eebcfce9b29e26fd55b60fbf24bd254351cb770d9`.

````cpp
// Copyright 2014 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#ifndef THIRD_PARTY_BLINK_RENDERER_CORE_PAINT_BOX_DECORATION_DATA_H_
#define THIRD_PARTY_BLINK_RENDERER_CORE_PAINT_BOX_DECORATION_DATA_H_

#include <optional>

#include "third_party/blink/renderer/core/css/properties/longhands.h"
#include "third_party/blink/renderer/core/layout/background_bleed_avoidance.h"
#include "third_party/blink/renderer/core/layout/layout_box.h"
#include "third_party/blink/renderer/core/layout/layout_replaced.h"
#include "third_party/blink/renderer/core/layout/physical_box_fragment.h"
#include "third_party/blink/renderer/core/paint/paint_info.h"
#include "third_party/blink/renderer/core/style/computed_style.h"
#include "third_party/blink/renderer/platform/graphics/color.h"

namespace blink {

// Data for box decoration painting.
class BoxDecorationData {
  STACK_ALLOCATED();

 public:
  BoxDecorationData(const PaintInfo& paint_info,
                    const LayoutReplaced& layout_replaced)
      : BoxDecorationData(paint_info,
                          layout_replaced,
                          layout_replaced.StyleRef(),
                          layout_replaced.StyleRef().HasBorderDecoration()) {}

  BoxDecorationData(const PaintInfo& paint_info,
                    const PhysicalFragment& fragment,
                    const ComputedStyle& style)
      : BoxDecorationData(
            paint_info,
            To<LayoutBox>(*fragment.GetLayoutObject()),
            style,
            !fragment.HasCollapsedBorders() && style.HasBorderDecoration()) {}

  BoxDecorationData(const PaintInfo& paint_info,
                    const PhysicalFragment& fragment)
      : BoxDecorationData(paint_info, fragment, fragment.Style()) {}

  BoxDecorationData BackgroundOnly() const {
    DCHECK(should_paint_background_);
    return BoxDecorationData(*this, /*should_paint_background=*/true,
                             /*should_paint_border=*/false);
  }
  BoxDecorationData BorderOnly() const {
    DCHECK(should_paint_border_);
    return BoxDecorationData(*this, /*should_paint_background=*/false,
                             /*should_paint_border=*/true);
  }

  bool IsPaintingBackgroundInContentsSpace() const {
    return paint_info_.IsPaintingBackgroundInContentsSpace();
  }
  bool HasAppearance() const { return has_appearance_; }
  bool ShouldPaintBackground() const { return should_paint_background_; }
  bool ShouldPaintBorder() const { return should_paint_border_; }
  bool ShouldPaintShadow() const { return should_paint_shadow_; }
  bool ShouldPaintGapDecorations() const {
    return should_paint_gap_decorations_;
  }

  BackgroundBleedAvoidance GetBackgroundBleedAvoidance() const {
    if (!bleed_avoidance_)
      bleed_avoidance_ = ComputeBleedAvoidance();
    return *bleed_avoidance_;
  }

  bool ShouldPaint() const {
    return HasAppearance() || ShouldPaintBackground() || ShouldPaintBorder() ||
           ShouldPaintShadow() || ShouldPaintGapDecorations();
  }

  // This is not cached because the caller is unlikely to call this repeatedly.
  Color BackgroundColor() const {
    return style_.VisitedDependentColor(GetCSSPropertyBackgroundColor());
  }

 private:
  BoxDecorationData(const PaintInfo& paint_info,
                    const LayoutBox& layout_box,
                    const ComputedStyle& style,
                    const bool has_non_collapsed_border_decoration)
      : paint_info_(paint_info),
        layout_box_(layout_box),
        style_(style),
        has_appearance_(style.HasEffectiveAppearance()),
        should_paint_background_(ComputeShouldPaintBackground()),
        should_paint_border_(
            ComputeShouldPaintBorder(has_non_collapsed_border_decoration)),
        should_paint_shadow_(ComputeShouldPaintShadow()),
        should_paint_gap_decorations_(ComputeShouldPaintGapDecorations()) {}

  // For BackgroundOnly() and BorderOnly().
  BoxDecorationData(const BoxDecorationData& data,
                    bool should_paint_background,
                    bool should_paint_border)
      : paint_info_(data.paint_info_),
        layout_box_(data.layout_box_),
        style_(data.style_),
        has_appearance_(false),
        should_paint_background_(should_paint_background),
        should_paint_border_(should_paint_border),
        should_paint_shadow_(false),
        should_paint_gap_decorations_(false) {
    DCHECK(!data.has_appearance_);
    DCHECK(!data.should_paint_shadow_);
    DCHECK(!data.should_paint_gap_decorations_);
  }

  bool ComputeShouldPaintBackground() const {
    // The page border box fragment paints the document background, so we cannot
    // trust its computed style when it comes to background properties.
    //
    // See https://drafts.csswg.org/css-page-3/#painting
    //
    // When not compositing a backdrop filter needs to have a display item for
    // the background even if it is transparent or otherwise paints nothing.
    //
    // TODO(crbug.com/40286153): This is a false positive. We should be able to
    // remove this once we have a better way to determine whether there is a
    // background.
    bool has_background =
        style_.HasBackground() ||
        GetBoxFragmentType() == PhysicalFragment::kPageBorderBox ||
        (paint_info_.ShouldOmitCompositingInfo() && style_.HasBackdropFilter());
    return has_background && !layout_box_.BackgroundTransfersToView() &&
           !paint_info_.ShouldSkipBackground();
  }

  bool ComputeShouldPaintBorder(
      bool has_non_collapsed_border_decoration) const {
    if (paint_info_.IsPaintingBackgroundInContentsSpace())
      return false;
    return has_non_collapsed_border_decoration;
  }

  bool ComputeShouldPaintShadow() const {
    return !paint_info_.IsPaintingBackgroundInContentsSpace() &&
           style_.BoxShadow();
  }

  bool ComputeShouldPaintGapDecorations() const { return style_.HasGapRule(); }

  bool BorderObscuresBackgroundEdge() const;
  BackgroundBleedAvoidance ComputeBleedAvoidance() const;

  PhysicalFragment::BoxType GetBoxFragmentType() const {
    if (!layout_box_.PhysicalFragmentCount()) {
      return PhysicalFragment::kNormalBox;
    }
    return layout_box_.GetPhysicalFragment(0)->GetBoxType();
  }

  // Inputs.
  const PaintInfo& paint_info_;
  const LayoutBox& layout_box_;
  const ComputedStyle& style_;

  // Outputs that are initialized in the constructor.
  const bool has_appearance_;
  const bool should_paint_background_;
  const bool should_paint_border_;
  const bool should_paint_shadow_;
  const bool should_paint_gap_decorations_;
  // This is lazily initialized.
  mutable std::optional<BackgroundBleedAvoidance> bleed_avoidance_;
};

}  // namespace blink

#endif  // THIRD_PARTY_BLINK_RENDERER_CORE_PAINT_BOX_DECORATION_DATA_H_
````

File-specific license: [Chromium BSD-style license](../licenses/chromium/LICENSE.txt).
