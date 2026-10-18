# Blink layered DOM sharing witness

Bounded implementation evidence for investigation of sparse state, attribute strings and small collections. This does not change the pinned HTML language, expand #1264, or select a measured optimum.

Revision: `389904a08e57a6725bb7e1bdf6d567e9eead3585`. Retrieved 2026-10-18. Complete source and original legal headers are retained verbatim in Markdown code blocks; no source is compiled.

## third_party/blink/renderer/core/dom/node_rare_data.h

[Immutable original](https://chromium.googlesource.com/chromium/src/+/389904a08e57a6725bb7e1bdf6d567e9eead3585/third_party/blink/renderer/core/dom/node_rare_data.h); 31391 bytes; SHA-256 `1cf7d054c88507419d0d959fe5ceaa61b9e9c80ddeb93e7a59c886165529aaad`.

```cpp
// Copyright 2022 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#ifndef THIRD_PARTY_BLINK_RENDERER_CORE_DOM_NODE_RARE_DATA_H_
#define THIRD_PARTY_BLINK_RENDERER_CORE_DOM_NODE_RARE_DATA_H_

#include "third_party/blink/renderer/core/core_export.h"
#include "third_party/blink/renderer/core/dom/focusgroup_flags.h"
#include "third_party/blink/renderer/core/dom/node_rare_data_field.h"
#include "third_party/blink/renderer/core/dom/pseudo_element_data.h"
#include "third_party/blink/renderer/core/dom/rare_data_update.h"
#include "third_party/blink/renderer/platform/graphics/paint/tracked_element_data.h"
#include "third_party/blink/renderer/platform/heap/garbage_collected.h"
#include "third_party/blink/renderer/platform/heap/trace_traits.h"
#include "third_party/blink/renderer/platform/region_capture_crop_id.h"
#include "third_party/blink/renderer/platform/restriction_target_id.h"
#include "third_party/blink/renderer/platform/wtf/text/atomic_string.h"

namespace viz {
enum class TrackedElementFeature;
}  // namespace viz

namespace gfx {
class Rect;
class Transform;
}  // namespace gfx

namespace blink {

class Attr;
class CSSPseudoElement;
class CSSStyleDeclaration;
class ColumnPseudoElement;
class ContainerQueryData;
class ContainerQueryEvaluator;
class ContentData;
class CustomElementDefinition;
class DOMTokenList;
class DatasetDOMStringMap;
class DisplayAdElementMonitor;
class DisplayLockContext;
class EditContext;
class Element;
class ElementAnimationTriggerData;
class ElementAnimations;
class ElementInternals;
class ElementIntersectionObserverData;
class ExplicitlySetAttrElementsMap;
class FlatTreeNodeData;
class FloatClipRect;
class HTMLElement;
class InlineStylePropertyMap;
class InterestInvokerTargetData;
class InvokerData;
class MutationObserverRegistration;
class NamedNodeMap;
class NodeListsNodeData;
class OutOfFlowData;
class OverscrollAreaTracker;
class PopoverData;
class PseudoElement;
class ResizeObservation;
class ResizeObserver;
class ScrollTimeline;
class ShadowRoot;
class UnboundedEventData;
class StyleScopeData;

enum class DynamicRestyleFlags;
enum class ElementFlags;

typedef HeapVector<Member<Attr>> AttrNodeList;

class NodeMutationObserverData final
    : public GarbageCollected<NodeMutationObserverData>,
      public NodeRareDataField {
 public:
  NodeMutationObserverData() = default;
  NodeMutationObserverData(const NodeMutationObserverData&) = delete;
  NodeMutationObserverData& operator=(const NodeMutationObserverData&) = delete;

  const HeapVector<Member<MutationObserverRegistration>>& Registry() {
    return registry_;
  }

  const HeapHashSet<Member<MutationObserverRegistration>>& TransientRegistry() {
    return transient_registry_;
  }

  void AddTransientRegistration(MutationObserverRegistration* registration);
  void RemoveTransientRegistration(MutationObserverRegistration* registration);
  void AddRegistration(MutationObserverRegistration* registration);
  void RemoveRegistration(MutationObserverRegistration* registration);

  void Trace(Visitor* visitor) const override;

 private:
  HeapVector<Member<MutationObserverRegistration>> registry_;
  HeapHashSet<Member<MutationObserverRegistration>> transient_registry_;
};

class ScrollTimelineHashSet final
    : public GarbageCollected<ScrollTimelineHashSet>,
      public NodeRareDataField {
 public:
  HeapHashSet<Member<ScrollTimeline>> set_;

  void Trace(Visitor* visitor) const override;
};

// NodeRareData provides sparse storage of fields for Node and Element (most
// Elements set only 0–3 of the 50+ possible fields). It consists of two parts:
//  * Fixed bit-fields that are always present (as long as the NodeRareData is
//    allocated at all), and
//  * A variable number of Member<NodeRareDataField>s (“slots”).
//
// The storage scheme for the latter is very similar to SparseVector (a bit set
// saying which values exist, and then the actual Members sorted by ID), but
// stored in AdditionalBytes. This conserves memory and avoids a pointer
// indirection to get to the members (although the actual values are still one
// step away); we don't need to store a pointer, the size can be calculated from
// the bitmap and the capacity is implicit from the size since we never delete
// elements (it is always the size rounded up to the nearest power of two,
// except that there is a minimum size).
//
// However, this means that any Set*() or Ensure*() member function can end up
// having to reallocate the NodeRareData to get more data into AdditionalBytes.
// Thus, every such function will return the new pointer for NodeRareData (which
// may be the same as the previous one, or an entirely new object), which means
// that the caller must update their pointer in case it changed. These are all
// marked by [[nodiscard]] so that you do not accidentally forget to do so.
class CORE_EXPORT NodeRareData final : public GarbageCollected<NodeRareData> {
  friend class NodeRareDataTest;

 public:
  using PassKey = base::PassKey<NodeRareData>;

  enum {
    kConnectedFrameCountBits = 10,  // Must fit Page::maxNumberOfFrames.
    kNumberOfElementFlags = 9,
    kNumberOfDynamicRestyleFlags = 14
  };

  static NodeRareData* Create() {
    return MakeGarbageCollected<NodeRareData>(
        AdditionalBytes(kMinimumVectorSize * kSlotSizeBytes), PassKey());
  }

  ~NodeRareData();
  NodeRareData(const NodeRareData&) = delete;
  NodeRareData& operator=(const NodeRareData&) = delete;

  void ClearNodeLists() { SetFieldToNullIfExists(FieldId::kNodeLists); }
  NodeListsNodeData* NodeLists() const;
  RareDataUpdate<NodeListsNodeData> EnsureNodeLists();

  FlatTreeNodeData* GetFlatTreeNodeData() const;
  RareDataUpdate<FlatTreeNodeData> EnsureFlatTreeNodeData();

  NodeMutationObserverData* MutationObserverData();
  RareDataUpdate<NodeMutationObserverData> EnsureMutationObserverData();

  uint16_t ConnectedSubframeCount() const {
    return flags_.connected_frame_count_;
  }
  void IncrementConnectedSubframeCount();
  void DecrementConnectedSubframeCount() {
    DCHECK(flags_.connected_frame_count_);
    --flags_.connected_frame_count_;
  }

  bool HasRestyleFlag(DynamicRestyleFlags mask) const {
    return flags_.restyle_flags_ & static_cast<uint16_t>(mask);
  }
  void SetRestyleFlag(DynamicRestyleFlags mask) {
    flags_.restyle_flags_ |= static_cast<uint16_t>(mask);
  }
  bool HasRestyleFlags() const { return flags_.restyle_flags_; }
  void ClearRestyleFlags() { flags_.restyle_flags_ = 0u; }

  [[nodiscard]] RareDataUpdate<void> RegisterScrollTimeline(ScrollTimeline*);
  [[nodiscard]] RareDataUpdate<void> UnregisterScrollTimeline(ScrollTimeline*);

  // Mostly for accessibility.
  DOMNodeId NodeId() const {
    auto* value = GetWrappedField<DOMNodeId>(FieldId::kDOMNodeId);
    return value ? *value : 0;
  }
  [[nodiscard]] RareDataUpdate<DOMNodeId> NodeId() {
    return EnsureWrappedField<DOMNodeId>(FieldId::kDOMNodeId);
  }

  [[nodiscard]] RareDataUpdate<void> SetPseudoElement(
      PseudoId,
      PseudoElement*,
      const AtomicString& document_transition_tag = g_null_atom);
  PseudoElement* GetPseudoElement(
      PseudoId,
      const AtomicString& document_transition_tag = g_null_atom) const;
  bool HasAnyPseudos() const;
  bool HasScrollButtonOrMarkerGroupPseudos() const;
  PseudoElementData::PseudoElementVector GetPseudoElements() const;

  [[nodiscard]] RareDataUpdate<void> AddColumnPseudoElement(
      ColumnPseudoElement&);
  const ColumnPseudoElementsVector* GetColumnPseudoElements() const;
  ColumnPseudoElement* GetColumnPseudoElement(wtf_size_t idx) const;
  void ClearColumnPseudoElements(wtf_size_t to_keep);

  RareDataUpdate<CSSStyleDeclaration> EnsureInlineCSSStyleDeclaration(
      Element* owner_element);

  ShadowRoot* GetShadowRoot() const;
  [[nodiscard]] RareDataUpdate<void> SetShadowRoot(ShadowRoot& shadow_root);

  bool HasAttributeMap() const;
  RareDataUpdate<NamedNodeMap> EnsureAttributeMap(Element& owner_element);

  DOMTokenList* GetClassList() const;
  RareDataUpdate<DOMTokenList> EnsureClassList(Element& element);

  DOMTokenList* GetFocusgroupTokenList() const;
  RareDataUpdate<DOMTokenList> EnsureFocusgroupTokenList(Element& element);

  RareDataUpdate<DatasetDOMStringMap> EnsureDataset(Element& element);

  ScrollOffset SavedLayerScrollOffset() const;
  [[nodiscard]] RareDataUpdate<void> SetSavedLayerScrollOffset(
      ScrollOffset offset);

  ElementAnimations* GetElementAnimations();
  RareDataUpdate<ElementAnimations> EnsureElementAnimations();

  bool HasPseudoElements() const;
  void ClearPseudoElements();

  RareDataUpdate<AttrNodeList> EnsureAttrNodeList();
  AttrNodeList* GetAttrNodeList();
  void RemoveAttrNodeList();
  [[nodiscard]] RareDataUpdate<void> AddAttr(Attr* attr);

  ElementIntersectionObserverData* IntersectionObserverData() const;
  RareDataUpdate<ElementIntersectionObserverData>
  EnsureIntersectionObserverData();

  ContainerQueryEvaluator* GetContainerQueryEvaluator() const;
  [[nodiscard]] RareDataUpdate<void> SetContainerQueryEvaluator(
      ContainerQueryEvaluator* evaluator);

  const AtomicString& GetNonce() const;
  [[nodiscard]] RareDataUpdate<void> SetNonce(const AtomicString& nonce);

  const AtomicString& IsValue() const;
  [[nodiscard]] RareDataUpdate<void> SetIsValue(const AtomicString& is_value);

  EditContext* GetEditContext() const;
  [[nodiscard]] RareDataUpdate<void> SetEditContext(EditContext* edit_context);

  RareDataUpdate<DOMTokenList> EnsurePart(Element& element);
  DOMTokenList* GetPart() const;

  [[nodiscard]] RareDataUpdate<void> SetPartNamesMap(
      const AtomicString part_names);
  const NamesMap* PartNamesMap() const;

  RareDataUpdate<InlineStylePropertyMap> EnsureInlineStylePropertyMap(
      Element* owner_element);
  InlineStylePropertyMap* GetInlineStylePropertyMap();

  const ElementInternals* GetElementInternals() const;
  RareDataUpdate<ElementInternals> EnsureElementInternals(HTMLElement& target);

  RareDataUpdate<DisplayLockContext> EnsureDisplayLockContext(Element* element);
  DisplayLockContext* GetDisplayLockContext() const;

  RareDataUpdate<ContainerQueryData> EnsureContainerQueryData();
  ContainerQueryData* GetContainerQueryData() const;
  void ClearContainerQueryData();

  RareDataUpdate<StyleScopeData> EnsureStyleScopeData();
  StyleScopeData* GetStyleScopeData() const;

  RareDataUpdate<OutOfFlowData> EnsureOutOfFlowData();
  OutOfFlowData* GetOutOfFlowData() const;
  void ClearOutOfFlowData();

  // Returns the crop-ID if one was set, or nullptr otherwise.
  const RegionCaptureCropId* GetRegionCaptureCropId() const;
  // Sets a crop-ID on the item. Must be called at most once. Cannot be used
  // to unset a previously set crop-ID.
  [[nodiscard]] RareDataUpdate<void> SetRegionCaptureCropId(
      std::unique_ptr<RegionCaptureCropId> crop_id);

  const TrackedElementSubRect* GetTrackedElementSubRect(
      viz::TrackedElementFeature feature) const;
  [[nodiscard]] RareDataUpdate<void> SetTrackedElementSubRect(
      viz::TrackedElementFeature feature,
      const TrackedElementSubRect& rect);
  void ClearTrackedElementSubRect(viz::TrackedElementFeature feature);

  const TrackedElementSubRects* GetTrackedElementSubRects() const;

  // Returns the ID backing a RestrictionTarget if one was set on the Element,
  // or nullptr otherwise.
  const RestrictionTargetId* GetRestrictionTargetId() const;
  // Returns the ID backing a RestrictionTarget if one was set on the Element,
  // or nullptr otherwise.
  // Sets an ID backing a RestrictionTarget associated with the Element.
  // Must be called at most once. Cannot be used to unset a previously set IDs.
  [[nodiscard]] RareDataUpdate<void> SetRestrictionTargetId(
      std::unique_ptr<RestrictionTargetId> id);

  using ResizeObserverDataMap =
      HeapHashMap<Member<ResizeObserver>, Member<ResizeObservation>>;
  ResizeObserverDataMap* ResizeObserverData() const;
  RareDataUpdate<ResizeObserverDataMap> EnsureResizeObserverData();

  [[nodiscard]] RareDataUpdate<void> SetCustomElementDefinition(
      CustomElementDefinition* definition);
  CustomElementDefinition* GetCustomElementDefinition() const;

  [[nodiscard]] RareDataUpdate<void> SetLastRememberedBlockSize(
      std::optional<LayoutUnit> size);
  [[nodiscard]] RareDataUpdate<void> SetLastRememberedInlineSize(
      std::optional<LayoutUnit> size);
  std::optional<LayoutUnit> LastRememberedBlockSize() const;
  std::optional<LayoutUnit> LastRememberedInlineSize() const;

  gfx::Rect LastSentUnboundedBounds() const;
  [[nodiscard]] RareDataUpdate<void> SetLastSentUnboundedBounds(
      const gfx::Rect& bounds);
  UnboundedEventData* GetUnboundedEventData() const;
  RareDataUpdate<UnboundedEventData> EnsureUnboundedEventData();

  PopoverData* GetPopoverData() const;
  RareDataUpdate<PopoverData> EnsurePopoverData();
  void RemovePopoverData();

  InvokerData* GetInvokerData() const;
  RareDataUpdate<InvokerData> EnsureInvokerData();

  InterestInvokerTargetData* GetInterestInvokerTargetData() const;
  RareDataUpdate<InterestInvokerTargetData> EnsureInterestInvokerTargetData();
  void RemoveInterestInvokerTargetData();

  bool HasElementFlag(ElementFlags mask) const {
    return flags_.element_flags_ & static_cast<uint16_t>(mask);
  }
  void SetElementFlag(ElementFlags mask, bool value) {
    flags_.element_flags_ =
        (flags_.element_flags_ & ~static_cast<uint16_t>(mask)) |
        (-static_cast<uint16_t>(value) & static_cast<uint16_t>(mask));
  }
  void ClearElementFlag(ElementFlags mask) {
    flags_.element_flags_ &= ~static_cast<uint16_t>(mask);
  }

  void SetTabIndexExplicitly() {
    SetElementFlag(ElementFlags::kTabIndexWasSetExplicitly, true);
  }
  void ClearTabIndexExplicitly() {
    ClearElementFlag(ElementFlags::kTabIndexWasSetExplicitly);
  }

  ScrollMarkerGroupData* GetScrollMarkerGroupData() const;
  void RemoveScrollMarkerGroupData();
  RareDataUpdate<ScrollMarkerGroupData> EnsureScrollMarkerGroupData(Element*);

  [[nodiscard]] RareDataUpdate<void> SetScrollMarkerGroupContainerData(
      ScrollMarkerGroupData*);
  ScrollMarkerGroupData* GetScrollMarkerGroupContainerData() const;

  [[nodiscard]] RareDataUpdate<void> CacheCSSPseudoElement(PseudoId,
                                                           const AtomicString&,
                                                           CSSPseudoElement&);
  CSSPseudoElement* GetCSSPseudoElement(PseudoId, const AtomicString&) const;

  ExplicitlySetAttrElementsMap* GetExplicitlySetElementsForAttr() const;
  RareDataUpdate<ExplicitlySetAttrElementsMap>
  EnsureExplicitlySetElementsForAttr();

  AnchorPositionScrollData* GetAnchorPositionScrollData() const;
  void RemoveAnchorPositionScrollData();
  RareDataUpdate<AnchorPositionScrollData> EnsureAnchorPositionScrollData(
      Element*);

  bool HasCustomElementRegistrySet() const;
  CustomElementRegistry* GetCustomElementRegistry() const;
  [[nodiscard]] RareDataUpdate<void> SetCustomElementRegistry(
      CustomElementRegistry* registry);
  void ClearCustomElementRegistry();

  ElementAnimationTriggerData* AnimationTriggerData();
  RareDataUpdate<ElementAnimationTriggerData> EnsureAnimationTriggerData();

  DisplayAdElementMonitor* GetDisplayAdElementMonitor() const;
  RareDataUpdate<DisplayAdElementMonitor> EnsureDisplayAdElementMonitor(
      Element*,
      AdProvenance);

  void SetDidAttachInternals() { flags_.did_attach_internals = true; }
  bool DidAttachInternals() const { return flags_.did_attach_internals; }
  bool HasUndoStack() const { return flags_.has_undo_stack; }
  void SetHasUndoStack(bool value) { flags_.has_undo_stack = value; }
  void SetPseudoElementStylesChangeCounters(bool value) {
    flags_.has_counters_styles = value;
  }
  bool PseudoElementStylesAffectCounters() const {
    return flags_.has_counters_styles;
  }
  bool ScrollbarPseudoElementStylesDependOnFontMetrics() const {
    return flags_.scrollbar_pseudo_element_styles_depend_on_font_metrics;
  }
  void SetScrollbarPseudoElementStylesDependOnFontMetrics(bool value) {
    flags_.scrollbar_pseudo_element_styles_depend_on_font_metrics = value;
  }
  void SetHasBeenExplicitlyScrolled() {
    flags_.has_been_explicitly_scrolled = true;
  }
  bool HasBeenExplicitlyScrolled() const {
    return flags_.has_been_explicitly_scrolled;
  }
  bool MayBeImplicitAnchor() const { return flags_.may_be_implicit_anchor; }
  void SetMayBeImplicitAnchor() { flags_.may_be_implicit_anchor = true; }

  FocusgroupData GetFocusgroupData() const;
  [[nodiscard]] RareDataUpdate<void> SetFocusgroupData(FocusgroupData data);
  void ClearFocusgroupData();
  [[nodiscard]] RareDataUpdate<void> SetFocusgroupLastFocused(Element* element);
  Element* GetFocusgroupLastFocused() const;
  void ClearFocusgroupLastFocused() {
    SetFieldToNullIfExists(FieldId::kFocusgroupLastFocused);
  }

  [[nodiscard]] RareDataUpdate<void> SetOverscrollContainer(Element* element);
  Element* GetOverscrollContainer() const;
  void ClearOverscrollContainer() {
    SetFieldToNullIfExists(FieldId::kOverscrollContainer);
  }

  void SetAffectedByStartingStyles() {
    flags_.affected_by_starting_styles = true;
  }
  bool AffectedByStartingStyles() const {
    return flags_.affected_by_starting_styles;
  }
  bool AffectedBySubjectHas() const { return flags_.affected_by_subject_has_; }
  void SetAffectedBySubjectHas() { flags_.affected_by_subject_has_ = true; }
  bool AffectedByNonSubjectHas() const {
    return flags_.affected_by_non_subject_has_;
  }
  void SetAffectedByNonSubjectHas() {
    flags_.affected_by_non_subject_has_ = true;
  }
  bool AncestorsOrAncestorSiblingsAffectedByHas() const {
    return flags_.ancestors_or_ancestor_siblings_affected_by_has_;
  }
  void SetAncestorsOrAncestorSiblingsAffectedByHas() {
    flags_.ancestors_or_ancestor_siblings_affected_by_has_ = true;
  }
  unsigned GetSiblingsAffectedByHasFlags() const {
    return flags_.siblings_affected_by_has_;
  }
  bool HasSiblingsAffectedByHasFlags(unsigned flags) const {
    return flags_.siblings_affected_by_has_ & flags;
  }
  void SetSiblingsAffectedByHasFlags(unsigned flags) {
    flags_.siblings_affected_by_has_ |= flags;
  }
  bool AffectedByPseudoInHas() const {
    return flags_.affected_by_pseudos_in_has_;
  }
  void SetAffectedByPseudoInHas() { flags_.affected_by_pseudos_in_has_ = true; }
  bool AncestorsOrSiblingsAffectedByHoverInHas() const {
    return flags_.ancestors_or_siblings_affected_by_hover_in_has_;
  }
  void SetAncestorsOrSiblingsAffectedByHoverInHas() {
    flags_.ancestors_or_siblings_affected_by_hover_in_has_ = true;
  }
  bool AncestorsOrSiblingsAffectedByActiveInHas() const {
    return flags_.ancestors_or_siblings_affected_by_active_in_has_;
  }
  void SetAncestorsOrSiblingsAffectedByActiveInHas() {
    flags_.ancestors_or_siblings_affected_by_active_in_has_ = true;
  }
  bool AncestorsOrSiblingsAffectedByFocusInHas() const {
    return flags_.ancestors_or_siblings_affected_by_focus_in_has_;
  }
  void SetAncestorsOrSiblingsAffectedByFocusInHas() {
    flags_.ancestors_or_siblings_affected_by_focus_in_has_ = true;
  }
  bool AncestorsOrSiblingsAffectedByFocusVisibleInHas() const {
    return flags_.ancestors_or_siblings_affected_by_focus_visible_in_has_;
  }
  void SetAncestorsOrSiblingsAffectedByFocusVisibleInHas() {
    flags_.ancestors_or_siblings_affected_by_focus_visible_in_has_ = true;
  }
  bool AffectedByLogicalCombinationsInHas() const {
    return flags_.affected_by_logical_combinations_in_has_;
  }
  void SetAffectedByLogicalCombinationsInHas() {
    flags_.affected_by_logical_combinations_in_has_ = true;
  }
  bool AffectedByMultipleHas() const {
    return flags_.affected_by_multiple_has_;
  }
  void SetAffectedByMultipleHas() { flags_.affected_by_multiple_has_ = true; }

  bool HasBeenHeuristicCustomPasswordCSS() const {
    return flags_.has_been_heuristic_custom_password_css_;
  }
  void SetHasBeenHeuristicCustomPasswordCSS() {
    flags_.has_been_heuristic_custom_password_css_ = true;
  }

  ContentData* GetAltContentData() const;
  [[nodiscard]] RareDataUpdate<void> SetAltContentData(
      ContentData* content_data);

  const gfx::Transform* GetCanvasTransform() const;
  [[nodiscard]] RareDataUpdate<void> SetCanvasTransform(
      const gfx::Transform& transform);
  void ClearCanvasTransform();

  const FloatClipRect* GetCanvasClip() const;
  [[nodiscard]] RareDataUpdate<void> SetCanvasClip(const FloatClipRect& clip);
  void ClearCanvasClip();

  bool WasLastFocusFromUserGesture() const {
    return flags_.was_last_focus_from_user_gesture;
  }
  void SetWasLastFocusFromUserGesture(bool value) {
    flags_.was_last_focus_from_user_gesture = value;
  }

  RareDataUpdate<OverscrollAreaTracker> EnsureOverscrollAreaTracker(Element*);
  OverscrollAreaTracker* OverscrollAreaTracker() const;

  void Trace(Visitor*) const;

  explicit NodeRareData(PassKey) {}
  NodeRareData(PassKey, NodeRareData&& other)
      : flags_(other.flags_), fields_bitfield_(other.fields_bitfield_.load()) {
    UNSAFE_BUFFERS(
        VectorTypeOperations<Member<NodeRareDataField>, HeapAllocator>::Move(
            other.ArrayBase(), other.ArrayBase() + other.size(), ArrayBase(),
            VectorOperationOrigin::kConstruction));

#if DCHECK_IS_ON()
    // Clear out the Members so that a) they are not inadvertently
    // used on errors, and b) the DCHECK in the destructor does not fire.
    for (unsigned i = 0; i < other.size(); ++i) {
      UNSAFE_BUFFERS(other.ArrayBase()[i] = nullptr);
    }
#endif
  }

 private:
  enum class FieldId : unsigned {
    kDataset = 0,
    kShadowRoot = 1,
    kClassList = 2,
    kAttributeMap = 3,
    kAttrNodeList = 4,
    kCssomWrapper = 5,
    kElementAnimations = 6,
    kIntersectionObserverData = 7,
    kPseudoElementData = 8,
    kEditContext = 9,
    kPart = 10,
    kCssomMapWrapper = 11,
    kElementInternals = 12,
    kDisplayLockContext = 13,
    kContainerQueryData = 14,
    kRegionCaptureCropId = 15,
    kResizeObserverData = 16,
    kCustomElementDefinition = 17,
    kPopoverData = 18,
    kPartNamesMap = 19,
    kNonce = 20,
    kIsValue = 21,
    kSavedLayerScrollOffset = 22,
    kAnchorPositionScrollData = 23,
    kMayBeImplicitAnchor = 24,
    kLastRememberedBlockSize = 25,
    kLastRememberedInlineSize = 26,
    kRestrictionTargetId = 27,
    kStyleScopeData = 28,
    kOutOfFlowData = 29,
    kInvokerData = 30,
    kInterestInvokerTargetData = 31,
    kScrollMarkerGroupData = 32,
    kScrollMarkerGroupContainerData = 33,
    kExplicitlySetElementsForAttr = 34,
    kCSSPseudoElementData = 35,
    kCustomElementRegistry = 36,
    kAnimationTriggerData = 37,
    kFocusgroupLastFocused = 38,
    kDisplayAdElementMonitor = 39,
    kOverscrollAreaTracker = 40,
    kAltContentData = 41,
    kOverscrollContainer = 42,
    kTrackedElementRect = 43,
    kNodeLists = 44,
    kMutationObserverData = 45,
    kFlatTreeNodeData = 46,
    kScrollTimelines = 47,
    kFocusgroupData = 48,
    kDOMNodeId = 49,
    kFocusgroupTokenList = 50,
    kLastSentUnboundedBounds = 51,
    kUnboundedEventTask = 52,
    kCanvasTransform = 53,
    kCanvasClip = 54,
    kNumFields = 55,
  };

  inline const Member<NodeRareDataField>* ArrayBase() const {
    static_assert(sizeof(*this) % alignof(Member<NodeRareDataField>) == 0,
                  "ValueArray may be improperly aligned");
    // SAFETY: By funneling all allocation of NodeRareData through Create(), we
    // guarantee that the array will exist where we expect it.
    return UNSAFE_BUFFERS(
        reinterpret_cast<const Member<NodeRareDataField>*>(this + 1));
  }
  inline Member<NodeRareDataField>* ArrayBase() {
    return const_cast<Member<NodeRareDataField>*>(
        const_cast<const NodeRareData*>(this)->ArrayBase());
  }

  const Member<NodeRareDataField>& ArraySlot(FieldId field_id) const {
    // SAFETY: Every modification of fields_bitfield_ goes through
    // SetField(), which makes sure there's always enough room
    // in AdditionalBytes.
    return UNSAFE_BUFFERS(ArrayBase()[GetFieldIndex(field_id)]);
  }

  Member<NodeRareDataField>& ArraySlot(FieldId field_id) {
    // SAFETY: Every modification of fields_bitfield_ goes through
    // SetField(), which makes sure there's always enough room
    // in AdditionalBytes.
    return UNSAFE_BUFFERS(ArrayBase()[GetFieldIndex(field_id)]);
  }

  NodeRareDataField* GetField(FieldId field_id) const;
  [[nodiscard]] NodeRareData* SetField(FieldId field_id,
                                       NodeRareDataField* field);
  void SetFieldToNullIfExists(FieldId field_id);

  template <typename T>
  class DataFieldWrapper final : public GarbageCollected<DataFieldWrapper<T>>,
                                 public NodeRareDataField {
   public:
    T& Get() { return data_; }
    void Trace(Visitor* visitor) const override {
      NodeRareDataField::Trace(visitor);
      TraceIfNeeded<T>::Trace(visitor, data_);
    }

   private:
    T data_;
  };

  template <typename T, typename... Args>
  [[nodiscard]] RareDataUpdate<T> EnsureField(FieldId field_id,
                                              Args&&... args) {
    T* field = static_cast<T*>(GetField(field_id));
    NodeRareData* vec = this;
    if (!field) {
      field = MakeGarbageCollected<T>(std::forward<Args>(args)...);
      vec = SetField(field_id, field);
    }
    return RareDataUpdate<T>(base::PassKey<NodeRareData>(), *field, vec);
  }

  template <typename T>
  [[nodiscard]] RareDataUpdate<T> EnsureWrappedField(FieldId field_id) {
    auto update = EnsureField<DataFieldWrapper<T>>(field_id);
    return RareDataUpdate<T>(base::PassKey<NodeRareData>(),
                             update.field_->Get(), update.rare_data_);
  }

  template <typename T, typename U>
  [[nodiscard]] RareDataUpdate<void> SetWrappedField(FieldId field_id, U data) {
    auto update = EnsureField<DataFieldWrapper<T>>(field_id);
    update.field_->Get() = std::move(data);
    return RareDataUpdate<void>(std::move(update));
  }

  template <typename T>
  T* GetWrappedField(FieldId field_id) const {
    auto* wrapper = static_cast<DataFieldWrapper<T>*>(GetField(field_id));
    return wrapper ? &wrapper->Get() : nullptr;
  }

  template <typename T>
  [[nodiscard]] RareDataUpdate<void> SetOptionalField(FieldId field_id,
                                                      std::optional<T> data) {
    if (data) {
      return SetWrappedField<T>(field_id, *data);
    } else {
      return RareDataUpdate<void>(SetField(field_id, nullptr));
    }
  }

  template <typename T>
  std::optional<T> GetOptionalField(FieldId field_id) const {
    if (auto* value = GetWrappedField<T>(field_id)) {
      return *value;
    }
    return std::nullopt;
  }

 private:
  using BitfieldType = uint64_t;
  static constexpr size_t kMaxSize = 64;

  using Slot = Member<NodeRareDataField>;
  static constexpr size_t kSlotSizeBytes = sizeof(Slot);

  // Most RareData vectors seem to have at least one element,
  // and due to Oilpan alignment, there's no point in having
  // an odd number of elements, so 2 is a reasonable default.
  //
  // This must be a power of two.
  static constexpr unsigned kMinimumVectorSize = 2;

  wtf_size_t size() const { return std::popcount(fields_bitfield_.load()); }

  // Returns whether the field exists. Time complexity is O(1).
  bool HasField(FieldId field_id) const {
    return fields_bitfield_ & FieldIdMask(field_id);
  }

  static BitfieldType FieldIdMask(FieldId field_id) {
    CHECK_LT(static_cast<wtf_size_t>(field_id), kMaxSize);
    return static_cast<BitfieldType>(1) << static_cast<wtf_size_t>(field_id);
  }

  // Returns a mask that has entries for all field IDs lower than `upper`.
  static BitfieldType LowerFieldIdsMask(FieldId upper) {
    return FieldIdMask(upper) - 1;
  }

  // Returns the index in the trailing array that `field_id` is stored in. If
  // the array isn't storing a field for `field_id`, then this returns the index
  // which the data for `field_id` should be inserted into.
  wtf_size_t GetFieldIndex(FieldId field_id) const {
    // Then count the total population of field IDs lower than that one we
    // are looking for. The target field ID should be located at the index of
    // of the total population.
    return std::popcount(fields_bitfield_ & LowerFieldIdsMask(field_id));
  }

  struct Flags {
    uint32_t restyle_flags_ : kNumberOfDynamicRestyleFlags = 0u;
    uint32_t connected_frame_count_ : kConnectedFrameCountBits = 0u;
    uint32_t element_flags_ : kNumberOfElementFlags = 0u;
    // 8 bits remaining in first word of Flags.

    unsigned did_attach_internals : 1 = false;
    unsigned has_undo_stack : 1 = false;
    unsigned scrollbar_pseudo_element_styles_depend_on_font_metrics : 1 = false;
    // This never gets reset, since we would have to keep track for
    // every pseudo-element whether it has counter style or not.
    // But since situations when counter style if removed from
    // pseudo-element are rare, we are fine with it, since
    // it doesn't hurt performance much.
    unsigned has_counters_styles : 1 = false;
    unsigned has_been_explicitly_scrolled : 1 = false;
    unsigned may_be_implicit_anchor : 1 = false;
    unsigned affected_by_starting_styles : 1 = false;
    // This records the last type of a focus on this element via `SetFocused`
    // (or more accurately, the only derived value we need from that).
    // For more see:
    // https://explainers-by-googlers.github.io/user-dictionary-leaks/
    unsigned was_last_focus_from_user_gesture : 1 = false;

    // :has() invalidation flags. See has_invalidation_flags.h for description.
    unsigned affected_by_subject_has_ : 1 = false;
    unsigned affected_by_non_subject_has_ : 1 = false;
    unsigned affected_by_pseudos_in_has_ : 1 = false;
    unsigned siblings_affected_by_has_ : 2 = 0;
    unsigned ancestors_or_ancestor_siblings_affected_by_has_ : 1 = false;
    unsigned ancestors_or_siblings_affected_by_hover_in_has_ : 1 = false;
    unsigned ancestors_or_siblings_affected_by_active_in_has_ : 1 = false;
    unsigned ancestors_or_siblings_affected_by_focus_in_has_ : 1 = false;
    unsigned ancestors_or_siblings_affected_by_focus_visible_in_has_ : 1 =
        false;
    unsigned affected_by_logical_combinations_in_has_ : 1 = false;
    unsigned affected_by_multiple_has_ : 1 = false;

    // We need to be able to distinguish between unset CustomElementRegistry
    // and explicitly nullptr CustomElementRegistry.
    unsigned has_custom_element_registry_ : 1 = false;

    // Whether this element is or has ever been identified as a custom
    // password field via CSS -webkit-text-security heuristics.
    // This is distinct from native passwords (<input type=password>).
    unsigned has_been_heuristic_custom_password_css_ : 1 = false;

    // 1 free bit in the second word (9 free bits overall).
  };
  static_assert(sizeof(Flags) <= 8, "Flags struct should not exceed 64 bits");

  Flags flags_;

  // Atomic because we might get traced from another thread.
  std::atomic<BitfieldType> fields_bitfield_ = 0;
};

template <>
struct ThreadingTrait<blink::NodeRareData> {
  static constexpr ThreadAffinity kAffinity = kMainThreadOnly;
};

}  // namespace blink

#endif  // THIRD_PARTY_BLINK_RENDERER_CORE_DOM_NODE_RARE_DATA_H_
```

## third_party/blink/renderer/core/dom/attribute.h

[Immutable original](https://chromium.googlesource.com/chromium/src/+/389904a08e57a6725bb7e1bdf6d567e9eead3585/third_party/blink/renderer/core/dom/attribute.h); 4957 bytes; SHA-256 `c776337b433632a2598245c088bfec29cfb66ff1eea1e73826cc55fa93895e0d`.

```cpp
/*
 * Copyright (C) 1999 Lars Knoll (knoll@kde.org)
 *           (C) 1999 Antti Koivisto (koivisto@kde.org)
 *           (C) 2001 Peter Kelly (pmk@post.com)
 *           (C) 2001 Dirk Mueller (mueller@kde.org)
 * Copyright (C) 2003, 2004, 2005, 2006, 2008, 2012 Apple Inc. All rights
 * reserved.
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
 *
 */

#ifndef THIRD_PARTY_BLINK_RENDERER_CORE_DOM_ATTRIBUTE_H_
#define THIRD_PARTY_BLINK_RENDERER_CORE_DOM_ATTRIBUTE_H_

#include "base/containers/span.h"
#include "build/build_config.h"
#include "third_party/blink/renderer/core/dom/qualified_name.h"
#include "third_party/blink/renderer/platform/wtf/allocator/allocator.h"

namespace blink {
class Attribute;
}

namespace base {
template <>
inline constexpr bool kCanSafelyConvertToByteSpan<::blink::Attribute> =
    kCanSafelyConvertToByteSpan<::blink::QualifiedName> &&
    kCanSafelyConvertToByteSpan<::blink::AtomicString>;
}

namespace blink {

// This value is set fairly arbitrarily, to get above what we expect to be
// the maximum number of attributes on a normal element. It is used for
// preallocation in Vectors holding Attributes, e.g. to avoid allocations
// in HTMLAtomicToken (which is short-lived, and thus does not need to worry
// much about extra memory usage).
//
// Many places that use this constant don't actually care directly about
// preallocation, but the value tends to propagate out through APIs.
static constexpr int kAttributePrealloc = 10;

// This is the internal representation of an attribute, consisting of a name and
// value. It is distinct from the web-exposed Attr, which also knows of the
// element to which it attached, if any.
class Attribute {
  DISALLOW_NEW();

 public:
  Attribute(const QualifiedName& name, const AtomicString& value)
      : name_(name), value_(value) {}
  Attribute(const QualifiedName& name, AtomicString&& value)
      : name_(name), value_(std::move(value)) {}
  Attribute(QualifiedName&& name, AtomicString&& value)
      : name_(std::move(name)), value_(std::move(value)) {}

  // NOTE: The references returned by these functions are only valid for as long
  // as the Attribute stays in place. For example, calling a function that
  // mutates an Element's internal attribute storage may invalidate them.
  const AtomicString& Value() const { return value_; }
  const AtomicString& Prefix() const { return name_.Prefix(); }
  const AtomicString& LocalName() const { return name_.LocalName(); }
  const AtomicString& NamespaceURI() const { return name_.NamespaceURI(); }

  const QualifiedName& GetName() const { return name_; }

  uint32_t BloomFilter() const { return name_.BloomFilter(); }

  bool IsEmpty() const { return value_.empty(); }
  bool Matches(const QualifiedName&) const;
  bool MatchesCaseInsensitive(const QualifiedName&) const;

  void SetValue(const AtomicString& value) { value_ = value; }

  // Note: This API is only for HTMLTreeBuilder.  It is not safe to change the
  // name of an attribute once parseAttribute has been called as DOM
  // elements may have placed the Attribute in a hash by name.
  void ParserSetName(const QualifiedName& name) { name_ = name; }

#if defined(COMPILER_MSVC)
  // NOTE: This constructor is not actually implemented, it's just defined so
  // MSVC will let us use a zero-length array of Attributes.
  Attribute();
#endif

  bool operator==(const Attribute& other) const = default;

 private:
  QualifiedName name_;
  AtomicString value_;
};
static_assert(sizeof(Attribute) == sizeof(QualifiedName) + sizeof(AtomicString),
              "AttributeHash() assumes Attribute has no padding");

inline bool Attribute::Matches(const QualifiedName& qualified_name) const {
  return (qualified_name.LocalName() == LocalName()) &&
         (qualified_name.NamespaceURI() == NamespaceURI() ||
          qualified_name.Prefix() == g_star_atom);
}

inline bool Attribute::MatchesCaseInsensitive(
    const QualifiedName& qualified_name) const {
  return qualified_name.LocalNameUpper() == name_.LocalNameUpper() &&
         (qualified_name.NamespaceURI() == NamespaceURI() ||
          qualified_name.Prefix() == g_star_atom);
}

}  // namespace blink

#endif  // THIRD_PARTY_BLINK_RENDERER_CORE_DOM_ATTRIBUTE_H_
```
