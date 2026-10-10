# Blink: bounded text source witnesses

This reference retains complete source files from Blink at immutable revision `7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2`, including original file-specific license headers. It supplies bounded evidence for whitespace value support, character-alignment support. It is not a CSS specification or a whole-engine conformance claim. These source files are reference material only.

License material: [Chromium BSD license](../licenses/chromium/LICENSE.txt).

## css_properties.json5

Original: [complete immutable source](https://chromium.googlesource.com/chromium/src/+/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/css/css_properties.json5). Path: `third_party/blink/renderer/core/css/css_properties.json5`. Source bytes: 390281; source lines: 11140; SHA-256: `d8e8f3205ca241d76ab736ebc2b21a89aa5275c0fd8b7be32c09bea6317b8aba`.

```cpp
// Copyright 2017 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

{
// This file specifies all the CSS properties we support and the necessary
// information for our code generation. The various supported arguments
// are described below with example usage

  parameters: {
    // - alias_for: "other-property"
    // Properties specifying alias_for should be virtually identical to the
    // properties they alias. Minor parsing differences are allowed as long as
    // the CSSValues created are of the same format of the aliased property.
    alias_for: {
    },

    // - alternative_of: "other-property"
    //
    // Makes the the property an "alternative" of another property.
    // An alternative property has a separate CSSProperty class which (based on
    // runtime flags) is used internally in place of the main CSSProperty class.
    // This makes it possible to e.g. switch a property definition from a
    // longhand to a shorthand at runtime.
    //
    // When parsing text (e.g. "animation") into a CSSPropertyID, the
    // alternative will be chosen if it is enabled. Otherwise, the main
    // property will be chosen.
    //
    // A main property may only have a single alternative property. It is
    // however possible to have an alternative of an alternative, in which case
    // the chain is followed. In other words, we choose the "innermost"
    // alternative that's enabled.
    //
    // Note that an alternative property ignores any runtime_flag on the main
    // property.
    alternative_of: {
      valid_type: "str",
    },

    // - longhands: ["property", "other-property"]
    // The property is a shorthand for several other properties.
    longhands: {
    },

    // - property_methods: ["method1", "method2"]
    // List of methods that are implemented in the CSSProperty for this
    // property.
    property_methods: {
      default: [],
      valid_type: "list",
      valid_values: [
        "ParseSingleValue",
        "ParseShorthand",
        "CSSValueFromComputedStyleInternal",
        "ColorIncludingFallback",
        "InitialValue"
      ],
    },

    // - parse_helper: "function-name"
    // Specifies a css_parsing_utils helper function to use for ParseSingleValue,
    // allowing the method to be generated instead of hand-written.
    parse_helper: {
      default: null,
      valid_type: "str",
    },

    // Suppresses code generation for the specified style builder functions.
    // This allows us to provide hand-written style builder functions in cases
    // where it's needed.
    style_builder_custom_functions: {
      default: [],
      valid_type: "list",
      valid_values: [
        "initial",
        "inherit",
        "value",
      ],
    },

    // Affects how the style building functions are generated.
    //
    // Several property groups (e.g. color properties) deviate from the default
    // style builder application, yet there are enough of these properties that
    // we want to generate code for them rather than having manually written
    // style builder functions.
    style_builder_template: {
      valid_values: [
        "animation",
        "auto",
        "background_layer",
        "border_image",
        "color",
        "counter",
        "empty",
        "grid",
        // The legacy template means that regular code generation should not be
        // be performed, and that the property is hard-coded in
        // style_builder_functions.cc.tmpl.
        "legacy",
        "mask_box",
        "mask_layer",
        "transition",
        "visited_color",
      ],
    },

    // Additional arguments to 'style_builder_template' may be provided here.
    style_builder_template_args: {
      default: {},
      valid_type: "dict"
    },

    // - is_descriptor
    // Whether it is a CSS descriptor. Descriptors define the characteristics of
    // an at-rule. E.g. @font-face is an at-rule, and src is a valid descriptor
    // for @font-face. Descriptors and CSS properties with the same name are
    // handled together in this file.
    // TODO(crbug.com/752745): Don't use CSSPropertyID for descriptors.
    // - is_property
    // Whether it is a CSS property. If this is false then is_descriptor must be
    // true.
    is_descriptor: {
      default: false,
      valid_type: "bool",
    },
    is_property: {
      default: true,
      valid_type: "bool",
    },

    // - independent
    // This property affects only one field on ComputedStyle, and can be set
    // directly during inheritance instead of forcing a recalc.
    // StyleResolver and StyleAdjuster are not invoked when these properties
    // are changed on a parent. Recalcs only happen if at least one
    // non-independent inherited property is changed in the parent.
    independent: {
      default: false,
      valid_type: "bool",
    },

    // - semi_independent_variable
    // This property affects to the {Inherited, NonInherited}Variable data fields so that we
    // can assume that the custom properties might not depend on any other property. We can
    // handle these properties so that they are excluded from the shared Inherited/NohInherited
    // logic, like the Equal and inheritance functions.
    semi_independent_variable: {
      default: false,
      valid_type: "bool",
    },

    // - affected_by_all
    // The affected_by_all flag indicates whether a change to the CSS property
    // "all" affects this property.
    // c.f. https://drafts.csswg.org/css-cascade/#all-shorthand
    // Descriptors (is_property: false) are never affected by changes to the
    // all property.
    affected_by_all: {
      default: true,
      valid_type: "bool",
    },

    // - interpolable
    // The interpolable flag indicates whether a property can be animated
    // smoothly. If this flag is set, the property should also be added to the
    // switch statements in CSSPropertyEquality and InterpolationTypesMap.
    interpolable: {
      default: false,
      valid_type: "bool",
    },

    // - inherited
    // The property will inherit by default if no value is specified, typically
    // mentioned in specifications as "Inherited: yes"
    inherited: {
      default: false,
      valid_type: "bool",
    },

    // - compositable
    // The property can be animated by the compositor
    compositable: {
      default: false,
      valid_type: "bool",
    },

    // - computable
    //
    // Whether or not a property appears on CSSStyleDeclaration.
    //
    // By default a property is computable if it's all of the following:
    //
    // - Not an alias
    // - A property (as opposed to a descriptor)
    // - A longhand
    //
    // Otherwise the property is (by default) _not_ computable.
    //
    // If an explicit true/false value is provided, this overrides the default,
    // and the property unconditionally becomes computable/not-computable
    // according to the value specified.
    //
    // Internal properties (-internal-*) are never computable, and using this
    // flag on internal properties is an error.
    computable: {
      valid_type: "bool",
    },

    // - includes_currentcolor
    //
    // Whether or not a property's value potentially includes currentcolor.
    //
    // This is used to determine if we need to invalidate the property when the
    // currentcolor is changed.
    includes_currentcolor: {
      valid_type: "bool",
      default: false,
    },

    // - runtime_flag
    // The name of the flag on RuntimeEnabledFeatures
    // (e.g. "CSSOverscrollBehavior") that conditionally enables the
    // property.
    // This doesn't currently work with alias_for.
    runtime_flag: {
      valid_type: "str",
    },

    // - field_group
    // Name of the group that this field belongs to. Fields in the same group
    // are stored together as a nested class inside ComputedStyle and
    // dynamically allocated on use.
    // Leave this out if the field is stored directly on ComputedStyle.
    // If you want to auto group this property use: field_group: "*[->subgroup]"
    // If you use the auto grouping function check if your property is in
    // css_properties_ranking.json5
    // -  If yes, only provide: field_group: "*"
    // -  If no, you can specify a subgroup following the asterisk:
    //    field_group: "*[->subgroup]"
    field_group: {
      valid_type: "str",
    },

    // - stored_on_extra_field
    // If this property doesn't correspond directly to a field (field_template
    // is unset), this allows you to specify that the property's data is stored
    // somewhere within the given extra field (which must exist in
    // computed_style_extra_fields.json5).
    //
    // This is useful for skipping over it during diffing.
    //
    // If field_template is set, then the property is assumed to have split
    // storage between the field_group _and_ this extra field's group,
    // i.e., a change in either could mean the property changed (and must be
    // tested for). You do not need to do this for visited color properties,
    // which are handled separately.
    stored_on_extra_field: {
      valid_type: "list",
    },

    // - field_size
    // Number of bits needed to store this field.
    field_size: {
      valid_type: "int",
    },

    // - field_template
    // Affects how the interface to this field is generated.
    // TODO(sashab, meade): Remove this once TypedOM types are specified for
    // every property, since this value can be inferred from that.
    field_template: {
      valid_values: [
        // Field is stored as an enum and has a initial/getter/setter/resetter.
        // If include_paths is empty, we would also generate the corresponding
        // enum definition in ComputedStyleConstants.h.
        // "keyword" properties are required to be parsed in
        // CSSParserFastPaths.
        // Use "keyword_custom" if a custom ParseSingleValue implementation is
        // necessary.
        "keyword",
        // Same as "keyword" except the implementation is allowed to implement a
        // custom ParseSingleValue. See Cursor and RubyPosition.
        "keyword_custom",
        // Field can take on any subset of values from a list of keywords.
        "multi_keyword",
        // Semantically equivalent to keyword, but the type is represented as a
        // bit flag field as with multi_keyword as a performance optimization
        // for matching multiple values.
        "bitset_keyword",
        // Field stores a primitive value like int/bool. The type is specified
        // by type_name. The interface has a initial/getter/setter/resetter.
        "primitive",
        // Field is stored as a bool, whose default value is false
        // and can only be set to true. Has a initial/getter/setter.
        "monotonic_flag",
        // A derived flag is derived from other information on ComputedStyle.
        // It has no setters, and is instead calculated on first access by
        // the function specified by 'derived_from'.
        //
        // Derived flags must be marked as 'mutable', and can not have a
        // 'field_group' (i.e. must exist on the top level of ComputedStyle).
        //
        // See computed_style_extra_fields.json5 for examples of derived flags.
        "derived_flag",
        // Field has type specified at type_name and has a getter/setter.
        // Also has a setter taking an rvalue reference. Cannot be packed.
        "external",
        // Field is stored as a wrapper_pointer_name to a class.
        "pointer",
        // Preset "length" for external and Length class
        // This preset represents alias templates that will be replace by
        // entries in CSSFieldAlias.json5.
        "<[a-z]+>"
      ],
    },

    // - anchor_mode
    // Determines whether or not anchor() / anchor-size() queries are allowed
    // in the relevant property.
    //
    // If omitted, no anchor queries are allowed.
    //
    // See also AnchorScope::Mode.
    anchor_mode: {
      valid_values: [
        // anchor() / anchor-size()
        "left",
        "right",
        "top",
        "bottom",

        // anchor-size()
        "width",
        "height",
      ]
    },

    // When specified on a property/field, this will generate code within
    // ComputedStyleBase::FieldInvalidationDiff to check if the property/field
    // has changed, and if so set a flag indicating this.
    //
    // Example usage:
    // if (field_diff & kBorderRadius) {
    //   diff.SetBorderRadiusChanged();
    // }
    //
    // The diff can also be used to "guard" against more expensive checks, e.g:
    // if ((field_diff & kOutline) && !OutlineVisuallyEqual(other)) {
    //   return true;
    // }
    //
    // This is to be **only** used within ComputedStyle::VisualInvalidationDiff
    // and will generally be more efficient than comparing fields directly.
    invalidate: {
      default: [],
      valid_type: "list",
      valid_values: [
        "accent-color",
        "ax-style",
        "background",
        "background-color",
        "blend-mode",
        "border-image",
        "border-outline-visited-color",
        "border-radius",
        "border-shape",
        "border-visual",
        "border-width",
        "box-paint-property",
        "clip",
        "clip-path",
        "color",
        "compositing",
        "corner-shape",
        "currentcolor",
        "filter-data",
        "gap-decorations",
        "has-transform",
        "inert",
        "inset",
        "layout",
        "mask",
        "opacity",
        "outline",
        "paint",
        "reshape",
        "scroll-anchor",
        "scrollbar-color",
        "scrollbar-style",
        "stroke",
        "text-decoration",
        "transform-data",
        "transform-other",
        "transform-property",
        "visibility",
        "visual-overflow",
        "z-index",
      ],
    },

    // Valid for field_template:derived_flag only. This specifies the function
    // on ComputedStyle used to calculate the flag.
    derived_from: {
      valid_type: "str",
    },

    // - include_paths: ["path/to/file1.h", "path/to/file2.h"]
    // List of files containing the definitions of types in 'type_name'. Each of
    // these files will appear as a #include in ComputedStyleBase.h. For
    // example, if the type_name is 'Vector<String>', include_paths should be
    // ["third_party/blink/renderer/platform/wtf/vector.h",
    //  "third_party/blink/renderer/platform/wtf/text/wtf_string.h"]
    include_paths: {
      default: [],
    },

    // Name of the pointer type that wraps this field (e.g. scoped_refptr).
    wrapper_pointer_name: {
      valid_type: "str",
      valid_values: ["scoped_refptr", "Member", "std::unique_ptr"],
    },

    // - keywords: ["keyword1", "keyword2"]
    // This specifies all valid keyword values for the property.
    // TODO(sashab): Once all properties are represented here, delete
    // CSSValueKeywords.in and use this list instead.
    keywords: {
      default: [],
    },

    // - default_value: "keyword-value"
    // This specifies the default value for this field.
    // - for keyword fields, this is the initial keyword
    // - for other fields, this is a string containg the C++ expression
    //   that is used to initialise the field.
    default_value: {
    },

    // Flags which go into CSSOMTypes:
    // - typedom_types: ["Keyword", "Type", "OtherType"]
    // The property can take types specified in typedom_types for CSS Typed OM.
    // - separator
    // The property supports a list of values, and when there is more than one,
    // it is separated with this character.
    typedom_types: {
      default: [],
      valid_type: "list",
      valid_values: [
        "Angle",
        "Flex",
        "Frequency",
        "Keyword",
        "Length",
        "Number",
        "Percentage",
        "Position",
        "Resolution",
        "Time",
        "Transform",
        "Unparsed",
        "Image"
      ],
    },
    // If typedom_types include "Keyword", these keywords are the ones reified
    // and accepted as CSSKeywordValue objects when set on StylePropertyMap.
    // Defaults to the same as 'keywords'.
    typedom_keywords: {
    },

    // If true, the property accepts an arbitrary <custom-ident> as a
    // CSSKeywordValue when set on StylePropertyMap (in addition to any
    // enumerated typedom_keywords).
    typedom_custom_ident: {
      default: false,
      valid_type: "bool",
    },

    separator: {
      valid_values: [",", " ", "/"],
    },

    // The remaining arguments are used for the StyleBuilder and allow us to
    // succinctly describe how to apply properties. When default handlers are
    // not sufficient, we should prefer to use converter, and failing that
    // define custom property handlers in CSSProperty subclasses. We should only
    // use style_builder_functions.tmpl to define handlers when there are
    // multiple properties requiring the same handling, but converter doesn't
    // suffice.

    // - font
    // The default property handlers call into the FontBuilder instead of
    // setting values directly onto the ComputedStyle
    font: {
      default: false,
      valid_type: "bool",
    },

    // - name_for_methods: "BlendMode"
    // Tweaks how we choose defaults for getter, setter, initial and type_name.
    // For example, setting this to BlendMode will make us use a setter of
    // SetBlendMode. Note that 'name_for_methods' also determines the name
    // of the generated field on ComputedStyle.
    // - initial
    // The static function to invoke on ComputedStyleInitialValues
    // or FontBuilder to retrieve the initial value.
    // Defaults to e.g. InitialBorderBottomLeft.
    // - getter
    // The ComputedStyle getter, defaults to e.g. BorderBottomLeft
    // - setter
    // The ComputedStyle setter, defaults to e.g. GetBorderBottomLeft
    // - type_name
    // The computed type for the property. Only required for the default value
    // application, defaults to e.g. EDisplay
    name_for_methods: {
    },
    initial: {
    },
    getter: {
    },
    setter: {
    },
    type_name: {
    },

    // - computed_style_protected_functions
    //
    // Any function specified in the list will be generated with protected
    // visibility. This is useful if the default-generated getter function is
    // typically not what clients want to use.
    //
    // For example, the Clear getter is protected to force clients to take
    // TextDirection into account.
    computed_style_protected_functions: {
      default: [],
      valid_type: "list",
      valid_values: ["getter", "setter", "resetter"],
    },

    // - computed_style_custom_functions
    //
    // Any function specified in the list will be generated with protected
    // visibility and an "Internal" suffix. A custom accessor (with the suffix-
    // less name) must be manually provided on ComputedStyle. This is useful for
    // e.g. properties that have special behavior that affects the computed
    // value of the property.
    //
    // For example, the computed value of border-left-width magically becomes
    // zero if border-left-style is none or hidden. The generated code can not
    // express this, hence a custom one is specified.
    //
    // Any custom function automatically gets protected visiblity, and therefore
    // it is not valid to specify a function as both custom and explicitly
    // protected (using computed_style_protected_functions).
    computed_style_custom_functions: {
      default: [],
      valid_type: "list",
      valid_values: ["initial", "getter", "setter", "resetter"],
    },

    // - converter: "ConvertRadius"
    // The StyleBuilder will call the specified function on
    // StyleBuilderConverter to convert a CSSValue to an appropriate platform
    // value
    converter: {
    },

    // - logical_property_group: used for properties that depend on writing-mode
    // and/or text-direction (e.g. css-logical), and for their physical counterparts.
    // Represents the "logical property group" described by css-logical
    // (https://drafts.csswg.org/css-logical/#logical-property-group).
    logical_property_group: {
      // A name identifying the logical property group. All logical and physical
      // properties in the same group should have the same name.
      //
      // In terms of code generation, each value corresponds to 2 functions in
      // CSSDirectionAwareResolver. E.g. a value of "foo-bar" would correspond to:
      // - CSSDirectionAwareResolver::LogicalFooBarMapping(), containing the
      //   properties of the group with a flow-relative mapping logic.
      // - CSSDirectionAwareResolver::PhysicalFooBarMapping(), containing the
      //   properties of the group with a physical mapping logic.
      name: {
        valid_type: "str",
        valid_values: ["border", "border-color", "border-radius",
                       "border-style", "border-width", "contain-intrinsic-size",
                       "inset", "margin", "max-size", "min-size", "overflow",
                       "padding", "scroll-margin", "scroll-padding", "size",
                       "visited-border-color"],
      },
      // The name of the mapping function used to convert between equivalent
      // logical and physical properties within the same group. Corresponds to
      // a function in CSSDirectionAwareResolver. E.g. a value of "baz"
      // corresponds to CSSDirectionAwareResolver::ResolveBaz(...).
      //
      // Also identifies the mapping logic of the group
      // (https://drafts.csswg.org/css-logical-1/#mapping-logic)
      resolver: {
        valid_type: "str",
        valid_values: [
          // Mapping logic: flow-relative (logical)
          "block", "inline",
          "block-start", "block-end", "inline-start", "inline-end",
          "start-start", "start-end", "end-start", "end-end",
          // Mapping logic: physical
          "vertical", "horizontal",
          "top", "bottom", "left", "right",
          "top-left", "top-right", "bottom-right", "bottom-left",
        ],
      },
    },

    // - surrogate_for: "other-property"
    //
    // A surrogate is a property which acts like another property. Unlike an
    // alias (which is resolved as parse-time), a surrogate exists alongside
    // the original in the parsed rule, and in the cascade.
    //
    // However, surrogates modify the same fields on ComputedStyle. Examples of
    // surrogates are:
    //
    //  * -webkit-writing-mode (surrogate of writing-mode)
    //  * inline-size (surrogate for width, or height)
    //  * All css-logical properties in general
    //
    // Note that for properties that use logical_property_group,
    // 'surrogate_for' should not be set, as the mapping is determined at
    // run-time (depending og e.g. 'direction').
    surrogate_for: {
      valid_type: "str",
    },

    // - priority: 1
    // The priority level for computing the property. Properties with the same
    // priority level are grouped and computed in alphabetical order.
    // Anything above zero are designated "high priority" and done before
    // certain operations, like updating fonts. (Most high-priority properties
    // are 1; 2 and higher are used only in special circumstances.) This mechanism
    // is primarily useful for properties that influence other properties,
    // like line-height influencing lh units. Negative values can be used to
    // to define ordered groups within the low-priority properties.
    priority: {
      default: 0,
      valid_type: "int",
    },

    // - layout_dependent
    // The resolved value used for getComputedStyle() depends on layout for this
    // property, which means we may need to update layout to return the correct
    // value from getComputedStyle(). Setting this to true will override
    // IsLayoutDependentProperty() to return true and require a custom
    // IsLayoutDependent() which typically checks for LayoutObject existence and
    // type.
    layout_dependent: {
      default: false,
      valid_type: "bool",
    },

    // - visited_property_for: "other-property"
    // CSS properties that are allowed in :visited selectors each have an
    // internal "companion" property with the visited value. For privacy reasons
    // CSSOM APIs must return computed values as if links aren't visited, but
    // for rendering purposes we need the value with the :visited rules applied.
    //
    // This means that the regular property (e.g. background-color) represents
    // the value as seen by CSSOM, and the -internal-visited counterpart (e.g.
    // -internal-visited-background-color) represents the same property as seen
    // by painting.
    visited_property_for: {
      valid_type: "str",
    },

    // - valid_for_first_letter: true
    //
    // https://drafts.csswg.org/css-pseudo-4/#first-letter-styling
    valid_for_first_letter: {
      default: false,
      valid_type: "bool",
    },

    // - valid_for_first_line: true
    //
    // https://drafts.csswg.org/css-pseudo-4/#first-line-styling
    valid_for_first_line: {
      default: false,
      valid_type: "bool",
    },

    // - valid_for_cue: true
    //
    // https://w3c.github.io/webvtt/#the-cue-pseudo-element
    valid_for_cue: {
      default: false,
      valid_type: "bool",
    },

    // - valid_for_marker: true
    //
    // https://drafts.csswg.org/css-pseudo-4/#marker-pseudo
    valid_for_marker: {
      default: false,
      valid_type: "bool",
    },

    // - valid_for_highlight: true
    //
    // https://drafts.csswg.org/css-pseudo-4/#highlight-styling
    valid_for_highlight: {
      default: false,
      valid_type: "bool",
    },

    // Applicable @page properties and descriptors.
    valid_for_page_context: {
      default: false,
      valid_type: "bool",
    },

    // - is_border
    // The property, when used by the author, will disable any native
    // appearance on UI elements.
    is_border: {
      default: false,
      valid_type: "bool",
    },

    // - is_background
    // The property, when used by the author, will disable any native
    // appearance on UI elements.
    is_background: {
      default: false,
      valid_type: "bool",
    },

    // - is_border_radius
    // The property, when used by the author, will disable any native
    // appearance on UI elements.
    is_border_radius: {
      default: false,
      valid_type: "bool",
    },

    // - is_highlight_colors
    // The property participates in paired cascade, such that when encountered
    // in highlight styles, we make all other highlight color properties default
    // to initial, rather than the UA default.
    // https://drafts.csswg.org/css-pseudo-4/#highlight-cascade
    is_highlight_colors: {
      default: false,
      valid_type: "bool",
    },
    // - is_visited_highlight_colors
    // Like the previous one but for visited internal properties.
    is_visited_highlight_colors: {
      default: false,
      valid_type: "bool",
    },

    // - is_animation_property
    // The property is a longhand of the 'animation' or 'transition' shorthands.
    is_animation_property: {
      default: false,
      valid_type: "bool",
    },

    // - is_animation_affecting
    //
    // Properties that affect animations are not allowed to be affected by
    // animations. Animation properties must always be animation-affecting.
    // Internal properties ignore this field; can_be_animated() always
    // returns true.
    //
    // https://w3.org/TR/web-animations-1/#animating-properties
    is_animation_affecting: {
      default: false,
      valid_type: "bool",
    },

    // Whether changes to an existing inline style declaration or SVG
    // presentation attribute can be applied incrementally on top of the old
    // style. Also selects which CSS values from SVG presentation attributes are
    // applied before inline style; values from other presentation attributes
    // remain in the cloned style.
    // See CanApplyStyleIncrementally() for eligibility.
    // Conceptually, this could be a blocklist, but being conservative, we have
    // chosen to make it an allowlist.
    // The properties with known issue are explicitly marked as false,
    // so changing the default from false to true _should_ have no ill effects,
    // but bugs are of course possible. (There is a DCHECK verifying that we
    // computed the correct style when this optimization is in effect.)
    //
    // Since animations can affect pretty much anything else, and we don't
    // support their interactions anyway (see CanApplyStyleIncrementally()),
    // animation properties are also never marked as supporting incremental style.
    // This is verified in validate_property().
    supports_incremental_style: {
      default: false,
      valid_type: "bool",
    },

    // If false, specifying this property in inline style or through an SVG
    // presentation attribute makes incremental style unsafe.
    // StyleAdjuster runs again on the cloned style, even for property values
    // retained from presentation attributes that are not reapplied.
    //
    // NOTE: Setting false here is probably indicative of a bug. Long-term,
    // we should fix all of these and remove the flag.
    idempotent: {
      default: true,
      valid_type: "bool",
    },

    // If true, this property will accept a CSSNumericLiteralValue
    // (created by a fast-path parser), with no restrictions on range.
    // (NaN and infinities will be sent through the normal ParseSingleValue
    // path.) A typical case is the properties that can accept an alpha value;
    // percent values will take the slow paths, but simple numbers will be sent
    // directly through.
    accepts_numeric_literal: {
      default: false,
      valid_type: "bool",
    },

    // If true, then the ComputedStyle field for this property overlaps with
    // another property.
    //
    // Overlapping properties are *partially* overlapping, or do otherwise not
    // have compatible or interchangeable values with each other.
    overlapping: {
      default: false,
      valid_type: "bool",
    },

    // Like 'overlapping', but set on -webkit-prefixed properties that should
    // ultimately be removed.
    //
    // Note that properties that are legacy_overlapping are also overlapping
    // (i.e. legacy_overlapping:true implies overlapping:true).
    legacy_overlapping: {
      default: false,
      valid_type: "bool",
    },

    // - valid_for_keyframe: true
    //
    // Whether the property can be used in @keyframes.
    // https://www.w3.org/TR/css-animations-1/#typedef-keyframe-block
    valid_for_keyframe: {
      default: true,
      valid_type: "bool",
    },

    // Whether the property can be applied to <permission> elements.
    // See https://github.com/WICG/PEPC/blob/main/explainer.md#locking-the-pepc-style
    valid_for_permission_element: {
      default: false,
      valid_type: "bool",
    },

    // Whether the property can be applied to ::permission-icon elements.
    // See https://github.com/WICG/PEPC/blob/main/explainer.md#locking-the-pepc-style
    valid_for_permission_icon: {
      default: false,
      valid_type: "bool",
    },

    // - valid_for_position_try: true
    //
    // Whether the property can be used in a @position-try rule
    // https://drafts.csswg.org/css-anchor-1/#fallback-rule
    valid_for_position_try: {
      default: false,
      valid_type: "bool",
    },

    // - affected_by_zoom: true
    //
    // Whether or not the computed value of this property is affected by
    // the effective zoom factor. Generally, all computed values that contain
    // a blink::Length are affected by zoom.
    //
    // Setting this flag to 'true' will change the inheritance behavior
    // (Longhand::ApplyInherit) to effectively "rezoom" the inherited value.
    //
    // https://github.com/w3c/csswg-drafts/issues/9397
    affected_by_zoom: {
      default: false,
      valid_type: "bool",
    },

    // - highlight_style_comes_from_originating_element: true
    //
    // ::highlight pseudo styles have different inheritance rules, where almost
    // all properties (even non-inherited ones) come from the parent style.
    // A couple of styles and fields are different in that they come from the
    // originating element's style instead, and those are marked as 'true' here.
    highlight_style_comes_from_originating_element: {
      default: false,
      valid_type: "bool",
    },

    // - percentages_depend_on_used_value
    //
    // Whether or not percentages for the given property depend on used value
    // and can or cannot be resolved before layout time.
    //
    // Should only be set on properties which accept percentage values. Setting
    // this flag to true will disallow simplifying calc expressions with
    // percentages which resolve differently for positive and negative
    // reference values, i.e. min()/max()/clamp()/random(), at computed value
    // time.
    percentages_depend_on_used_value: {
      valid_type: "bool",
    },

    // - tracks_animated_source: true
    //
    // Whether to track the animating element that this property's value
    // depends on through style inheritance and property references.
    tracks_animated_source: {
      default: false,
      valid_type: "bool",
    },

    // - devtools_keywords: ["keyword1", "keyword2"]
    //
    // Keywords that are valid for devtools. This would act as an override to
    // the keywords list.
    // This is useful for cases where
    // 1. Keywords are not accepted on their own in devtools but might be valid
    // in other contexts. For example, 'jump-both' is not accepted on its own in
    // devtools, but is valid as part of the animation-timing-function property.
    // 2. Shorthand properties do not accept all keywords from the longhands
    // as it is. For example, 'columns' does not accept 'wrap' or 'nowrap'
    // (which are valid keywords for 'column-wrap').
    // This field is used by the following script in the devtools-frontend repo to
    // generate the supported CSS properties:
    // third_party/devtools-frontend/src/scripts/build/generate_supported_css.py
    devtools_keywords: {
      valid_type: "list",
    },
  },

  // Members in the data objects should appear in the same order as in the
  // parameters object above
  data: [
    // Properties with StyleBuilder handling

    // Animation Priority properties
    {
      name: "animation-composition",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal", "InitialValue"],
      keywords: ["replace", "add", "accumulate"],
      style_builder_template: "animation",
      style_builder_template_args: {
        attribute: "Composition",
      },
      typedom_types: ["Keyword"],
      separator: ",",
      include_paths: ["third_party/blink/renderer/core/animation/effect_model.h"],
      default_value: "EffectModel::kCompositeReplace",
      type_name: "EffectModel::CompositeOperation",
      valid_for_marker: true,
      is_animation_affecting: true,
    },
    {
      name: "animation-delay",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal", "InitialValue"],
      style_builder_template: "animation",
      style_builder_template_args: {
        attribute: "DelayStart",
      },
      typedom_types: ["Time"],
      separator: ",",
      valid_for_marker: true,
      is_animation_property: true,
      is_animation_affecting: true,
      // Animation properites are never incremental.
      supports_incremental_style: false,
      valid_for_keyframe: false,
    },
    {
      name: "animation-direction",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal", "InitialValue"],
      keywords: ["normal", "reverse", "alternate", "alternate-reverse"],
      typedom_types: ["Keyword"],
      separator: ",",
      style_builder_template: "animation",
      style_builder_template_args: {
        attribute: "Direction",
      },
      separator: ",",
      valid_for_marker: true,
      is_animation_property: true,
      is_animation_affecting: true,
      // Animation properites are never incremental.
      supports_incremental_style: false,
      valid_for_keyframe: false,
    },
    {
      name: "animation-duration",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal", "InitialValue"],
      separator: ",",
      style_builder_template: "animation",
      style_builder_template_args: {
        attribute: "Duration",
      },
      typedom_types: ["Time"],
      separator: ",",
      valid_for_marker: true,
      is_animation_property: true,
      is_animation_affecting: true,
      // Animation properites are never incremental.
      supports_incremental_style: false,
      valid_for_keyframe: false,
      keywords: ["auto"],
    },
    {
      name: "animation-fill-mode",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal", "InitialValue"],
      style_builder_template: "animation",
      style_builder_template_args: {
        attribute: "FillMode",
      },
      keywords: ["none", "forwards", "backwards", "both"],
      typedom_types: ["Keyword"],
      separator: ",",
      valid_for_marker: true,
      is_animation_property: true,
      is_animation_affecting: true,
      // Animation properites are never incremental.
      supports_incremental_style: false,
      valid_for_keyframe: false,
    },
    {
      name: "animation-iteration-count",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal", "InitialValue"],
      keywords: ["infinite"],
      separator: ",",
      style_builder_template: "animation",
      style_builder_template_args: {
        attribute: "IterationCount",
      },
      keywords: ["infinite"],
      typedom_types: ["Keyword", "Number"],
      separator: ",",
      valid_for_marker: true,
      is_animation_property: true,
      is_animation_affecting: true,
      // Animation properites are never incremental.
      supports_incremental_style: false,
      valid_for_keyframe: false,
    },
    {
      name: "animation-name",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal", "InitialValue"],
      style_builder_template: "animation",
      style_builder_template_args: {
        attribute: "Name",
      },
      keywords: ["none"],
      typedom_types: ["Keyword"],
      typedom_custom_ident: true,
      separator: ",",
      valid_for_marker: true,
      is_animation_property: true,
      is_animation_affecting: true,
      // Animation properites are never incremental.
      supports_incremental_style: false,
      valid_for_keyframe: false,
    },
    {
      name: "animation-play-state",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal", "InitialValue"],
      style_builder_template: "animation",
      style_builder_template_args: {
        attribute: "PlayState",
      },
      keywords: ["running", "paused"],
      typedom_types: ["Keyword"],
      separator: ",",
      valid_for_marker: true,
      is_animation_property: true,
      is_animation_affecting: true,
      // Animation properites are never incremental.
      supports_incremental_style: false,
      valid_for_keyframe: false,
    },
    {
      name: "animation-range-start",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal", "InitialValue"],
      style_builder_template: "animation",
      style_builder_template_args: {
        attribute: "RangeStart",
      },
      separator: ",",
      valid_for_marker: true,
      is_animation_property: true,
      is_animation_affecting: true,
      // Animation properites are never incremental.
      supports_incremental_style: false,
      valid_for_keyframe: false,
      percentages_depend_on_used_value: false,
      affected_by_zoom: true,
    },
    {
      name: "animation-range-end",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal", "InitialValue"],
      style_builder_template: "animation",
      style_builder_template_args: {
        attribute: "RangeEnd",
      },
      separator: ",",
      valid_for_marker: true,
      is_animation_property: true,
      is_animation_affecting: true,
      // Animation properites are never incremental.
      supports_incremental_style: false,
      valid_for_keyframe: false,
      percentages_depend_on_used_value: false,
      affected_by_zoom: true,
    },
    {
      name: "animation-timeline",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal", "InitialValue"],
      style_builder_template: "animation",
      style_builder_template_args: {
        attribute: "Timeline",
      },
      keywords: ["none", "auto"],
      typedom_types: ["Keyword"],
      separator: ",",
      valid_for_marker: true,
      is_animation_property: true,
      is_animation_affecting: true,
      affected_by_zoom: true,
      // Animation properites are never incremental.
      supports_incremental_style: false,
    },
    {
      name: "animation-timing-function",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal", "InitialValue"],
      style_builder_template: "animation",
      style_builder_template_args: {
        attribute: "TimingFunction",
      },
      keywords: [
        "linear",
        "ease",
        "ease-in",
        "ease-out",
        "ease-in-out",
        "jump-both",
        "jump-end",
        "jump-none",
        "jump-start",
        "step-start",
        "step-end"
      ],
      typedom_types: ["Keyword"],
      separator: ",",
      valid_for_marker: true,
      is_animation_property: true,
      is_animation_affecting: true,
      // Animation properites are never incremental.
      supports_incremental_style: false,
    },
    {
      name: "animation-trigger",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal", "InitialValue"],
      style_builder_template: "animation",
      style_builder_template_args: {
        attribute: "TriggerAttachments",
      },
      keywords: ["none"],
      separator: ",",
      valid_for_marker: true,
      runtime_flag: "AnimationTrigger",
      is_animation_affecting: true,
    },
    {
      name: "timeline-trigger-name",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal", "InitialValue"],
      // TODO: maybe define an animation_trigger template?
      style_builder_template: "animation",
      style_builder_template_args: {
        attribute: "TimelineTriggerName",
      },
      separator: ",",
      valid_for_marker: true,
      runtime_flag: "TimelineTrigger",
      is_animation_affecting: true,
      keywords: ["none"],
    },
    {
      name: "timeline-trigger-activation-range-start",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal", "InitialValue"],
      style_builder_template: "animation",
      style_builder_template_args: {
        attribute: "TimelineTriggerActivationRangeStart",
      },
      separator: ",",
      valid_for_marker: true,
      runtime_flag: "TimelineTrigger",
      is_animation_affecting: true,
      percentages_depend_on_used_value: false,
      affected_by_zoom: true,
    },
    {
      name: "timeline-trigger-activation-range-end",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal", "InitialValue"],
      style_builder_template: "animation",
      style_builder_template_args: {
        attribute: "TimelineTriggerActivationRangeEnd",
      },
      separator: ",",
      valid_for_marker: true,
      runtime_flag: "TimelineTrigger",
      is_animation_affecting: true,
      percentages_depend_on_used_value: false,
      affected_by_zoom: true,
    },
    {
      name: "timeline-trigger-active-range-start",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal", "InitialValue"],
      style_builder_template: "animation",
      style_builder_template_args: {
        attribute: "TimelineTriggerActiveRangeStart",
      },
      separator: ",",
      valid_for_marker: true,
      runtime_flag: "TimelineTrigger",
      is_animation_affecting: true,
      percentages_depend_on_used_value: false,
      keywords: ["auto", "normal"],
      affected_by_zoom: true,
    },
    {
      name: "timeline-trigger-active-range-end",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal", "InitialValue"],
      style_builder_template: "animation",
      style_builder_template_args: {
        attribute: "TimelineTriggerActiveRangeEnd",
      },
      separator: ",",
      valid_for_marker: true,
      runtime_flag: "TimelineTrigger",
      is_animation_affecting: true,
      percentages_depend_on_used_value: false,
      keywords: ["auto", "normal"],
      affected_by_zoom: true,
    },
    {
      name: "timeline-trigger-source",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal", "InitialValue"],
      style_builder_template: "animation",
      style_builder_template_args: {
        attribute: "TimelineTriggerSource",
      },
      keywords: ["none", "auto"],
      typedom_types: ["Keyword"],
      separator: ",",
      valid_for_marker: true,
      runtime_flag: "TimelineTrigger",
      is_animation_affecting: true,
    },
    {
      name: "transition-delay",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal", "InitialValue"],
      style_builder_template: "transition",
      style_builder_template_args: {
        attribute: "DelayStart",
      },
      typedom_types: ["Time"],
      separator: ",",
      valid_for_marker: true,
      is_animation_property: true,
      is_animation_affecting: true,
      // Animation properites are never incremental.
      supports_incremental_style: false,
    },
    {
      name: "transition-duration",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal", "InitialValue"],
      typedom_types: ["Keyword", "Time"],
      separator: ",",
      style_builder_template: "transition",
      style_builder_template_args: {
        attribute: "Duration",
      },
      valid_for_marker: true,
      is_animation_property: true,
      is_animation_affecting: true,
      // Animation properites are never incremental.
      supports_incremental_style: false,
    },
    {
      name: "transition-property",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal", "InitialValue"],
      style_builder_template: "transition",
      style_builder_template_args: {
        attribute: "Property",
      },
      keywords: ["none"],
      typedom_types: ["Keyword"],
      valid_for_marker: true,
      is_animation_property: true,
      is_animation_affecting: true,
      // Animation properites are never incremental.
      supports_incremental_style: false,
    },
    {
      name: "transition-behavior",
      keywords: ["normal", "allow-discrete"],
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal", "InitialValue"],
      typedom_types: ["Keyword"],
      separator: ",",
      style_builder_template: "transition",
      style_builder_template_args: {
        attribute: "Behavior",
      },
      valid_for_marker: true,
      is_animation_property: true,
      is_animation_affecting: true,
      // Animation properites are never incremental.
      supports_incremental_style: false,
    },
    {
      name: "transition-timing-function",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal", "InitialValue"],
      style_builder_template: "transition",
      style_builder_template_args: {
        attribute: "TimingFunction",
      },
      keywords: [
        "linear",
        "ease",
        "ease-in",
        "ease-out",
        "ease-in-out",
        "jump-both",
        "jump-end",
        "jump-none",
        "jump-start",
        "step-start",
        "step-end"],
      typedom_types: ["Keyword"],
      separator: ",",
      valid_for_marker: true,
      is_animation_property: true,
      is_animation_affecting: true,
      // Animation properites are never incremental.
      supports_incremental_style: false,
    },
    {
      name: "trigger-scope",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ConsumeNameScope",
      include_paths: ["third_party/blink/renderer/core/style/style_trigger_scope.h"],
      type_name: "StyleTriggerScope",
      default_value: "StyleTriggerScope()",
      field_group: "*",
      field_template: "external",
      converter: "ConvertTriggerScope",
      keywords: ["none", "all"],
      runtime_flag: "AnimationTrigger",
      typedom_types: ["Keyword"],
      invalidate: ["layout"],
      valid_for_permission_element: true,
    },
    // High Priority and all other font properties.
    // Other properties can depend upon high priority properties
    // (e.g. font-size / ems)
    {
      name: "color",
      property_methods: ["CSSValueFromComputedStyleInternal", "ColorIncludingFallback"],
      parse_helper: "ConsumeColorMaybeQuirky",
      interpolable: true,
      inherited: true,
      // color isn't strictly independent of all other properties;
      // it determines currentColor, which in turn can affect the used value of
      // other properties (such as border colors, stops in gradients, etc.).
      // However, changes to color generally also trigger paint invalidation,
      // and paint invalidation resolves the color anew. (For the special case
      // of gradient stops, we have logic within ComputedStyle::AdjustDiffForBackgroundVisuallyEqual
      // that forces paint invalidation, recomputing the gradient and repainting
      // the element.)
      independent: true,
      field_group: "inherited",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/css/style_color.h"],
      default_value: "StyleColor(Color::kBlack)",
      type_name: "StyleColor",
      computed_style_protected_functions: ["getter"],
      tracks_animated_source: true,
      style_builder_custom_functions: ["initial", "inherit", "value"],
      priority: 1,
      keywords: ["currentcolor"],
      typedom_types: ["Keyword"],
      valid_for_first_letter: true,
      valid_for_first_line: true,
      valid_for_cue: true,
      valid_for_marker: true,
      valid_for_highlight: true,
      is_highlight_colors: true,
      supports_incremental_style: true,
      valid_for_permission_element: true,
      valid_for_page_context: true,
      invalidate: ["accent-color", "border-visual", "color", "currentcolor", "outline"],
    },
    {
      name: "direction",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      affected_by_all: false,
      inherited: true,
      field_template: "keyword",
      include_paths: ["third_party/blink/renderer/platform/text/text_direction.h"],
      keywords: ["ltr", "rtl"],
      typedom_types: ["Keyword"],
      default_value: "ltr",
      type_name: "TextDirection",
      style_builder_custom_functions: ["value"],
      priority: 1,
      valid_for_marker: true,
      valid_for_page_context: true,
      invalidate: ["ax-style", "reshape"],
      is_animation_affecting: true,
    },
    {
      name: "font-family",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      is_descriptor: true,
      inherited: true,
      font: true,
      name_for_methods: "FamilyDescription",
      type_name: "FontDescription::FamilyDescription",
      style_builder_custom_functions: ["initial", "inherit"],
      converter: "ConvertFontFamily",
      priority: 1,
      valid_for_first_letter: true,
      valid_for_first_line: true,
      valid_for_cue: true,
      valid_for_marker: true,
      valid_for_page_context: true,
      // See comment on font.
      supports_incremental_style: false,
      stored_on_extra_field: ["font"],
    },
    {
      name: "font-kerning",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      inherited: true,
      font: true,
      name_for_methods: "Kerning",
      converter: "ConvertFontKerning",
      type_name: "FontDescription::Kerning",
      priority: 1,
      keywords: ["auto", "normal", "none"],
      typedom_types: ["Keyword"],
      valid_for_first_letter: true,
      valid_for_first_line: true,
      valid_for_marker: true,
      // See comment on font.
      supports_incremental_style: false,
      valid_for_permission_element: true,
      valid_for_page_context: true,
      stored_on_extra_field: ["font"],
    },
    {
      name: "font-optical-sizing",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      inherited: true,
      font: true,
      name_for_methods: "FontOpticalSizing",
      converter: "ConvertFontOpticalSizing",
      type_name: "OpticalSizing",
      priority: 1,
      keywords: ["auto", "none"],
      typedom_types: ["Keyword"],
      valid_for_first_letter: true,
      valid_for_first_line: true,
      valid_for_marker: true,
      // See comment on font.
      supports_incremental_style: false,
      valid_for_permission_element: true,
      valid_for_page_context: true,
      stored_on_extra_field: ["font"],
    },
    {
      name: "font-palette",
      property_methods: ["CSSValueFromComputedStyleInternal" ],
      parse_helper: "ConsumeFontPalette",
      interpolable: true,
      inherited: true,
      font: true,
      converter: "ConvertFontPalette",
      type_name: "FontPalette",
      priority: 1,
      keywords: ["normal", "light", "dark"],
      typedom_types: ["Keyword"],
      valid_for_first_letter: true,
      valid_for_first_line: true,
      valid_for_marker: true,
      valid_for_page_context: true,
      // See comment on font.
      supports_incremental_style: false,
      stored_on_extra_field: ["font"],
    },
    {
      name: "font-size",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      inherited: true,
      font: true,
      name_for_methods: "Size",
      getter: "GetSize",
      converter: "ConvertFontSize",
      priority: 1,
      keywords: ["xx-small", "x-small", "small", "medium", "large", "x-large", "xx-large", "xxx-large", "larger", "smaller", "-webkit-xxx-large"],
      typedom_types: ["Keyword", "Length", "Percentage"],
      valid_for_first_letter: true,
      valid_for_first_line: true,
      valid_for_cue: true,
      valid_for_marker: true,
      // See comment on font.
      supports_incremental_style: false,
      valid_for_permission_element: true,
      valid_for_page_context: true,
      stored_on_extra_field: ["font"],
      percentages_depend_on_used_value: false,
    },
    {
      name: "font-size-adjust",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      inherited: true,
      runtime_flag: "CSSFontSizeAdjust",
      font: true,
      name_for_methods: "SizeAdjust",
      converter: "ConvertFontSizeAdjust",
      priority: 1,
      keywords: ["none", "ex-height", "cap-height", "ch-width", "ic-width", "ic-height", "from-font"],
      typedom_types: ["Keyword", "Number"],
      valid_for_first_letter: true,
      valid_for_first_line: true,
      valid_for_marker: true,
      valid_for_page_context: true,
      // See comment on font.
      supports_incremental_style: false,
      stored_on_extra_field: ["font"],
    },
    {
      name: "font-stretch",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ConsumeFontStretch",
      is_descriptor: true,
      interpolable: true,
      inherited: true,
      font: true,
      name_for_methods: "Stretch",
      converter: "ConvertFontStretch",
      priority: 1,
      keywords: [
        "normal", "ultra-condensed", "extra-condensed", "condensed",
        "semi-condensed", "semi-expanded", "expanded", "extra-expanded", "ultra-expanded"
      ],
      typedom_types: ["Keyword", "Percentage"],
      valid_for_first_letter: true,
      valid_for_first_line: true,
      valid_for_cue: true,
      valid_for_marker: true,
      // See comment on font.
      supports_incremental_style: false,
      valid_for_permission_element: true,
      valid_for_page_context: true,
      stored_on_extra_field: ["font"],
      percentages_depend_on_used_value: false,
    },
    {
      name: "font-style",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ConsumeFontStyle",
      is_descriptor: true,
      interpolable: true,
      inherited: true,
      font: true,
      name_for_methods: "Style",
      style_builder_custom_functions: ["inherit", "value"],
      converter: "ConvertFontStyle",
      priority: 1,
      keywords: ["normal", "italic", "oblique"],
      typedom_types: ["Keyword"],
      valid_for_first_letter: true,
      valid_for_first_line: true,
      valid_for_cue: true,
      valid_for_marker: true,
      // See comment on font.
      supports_incremental_style: false,
      valid_for_permission_element: true,
      valid_for_page_context: true,
      stored_on_extra_field: ["font"],
    },
    {
      name: "font-variant-ligatures",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      inherited: true,
      font: true,
      name_for_methods: "VariantLigatures",
      type_name: "VariantLigatures",
      converter: "ConvertFontVariantLigatures",
      priority: 1,
      keywords: [
         "normal", "none", "common-ligatures", "no-common-ligatures",
         "discretionary-ligatures", "no-discretionary-ligatures",
         "historical-ligatures", "no-historical-ligatures", "contextual",
         "no-contextual"
      ],
      typedom_types: ["Keyword"],
      valid_for_first_letter: true,
      valid_for_first_line: true,
      valid_for_marker: true,
      // See comment on font.
      supports_incremental_style: false,
      stored_on_extra_field: ["font"],
    },
    {
      name: "font-variant-caps",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      inherited: true,
      font: true,
      name_for_methods: "VariantCaps",
      converter: "ConvertFontVariantCaps",
      priority: 1,
      keywords: [
        "normal", "small-caps", "all-small-caps", "petite-caps",
        "all-petite-caps", "unicase", "titling-caps"
      ],
      typedom_types: ["Keyword"],
      valid_for_first_letter: true,
      valid_for_first_line: true,
      valid_for_marker: true,
      // See comment on font.
      supports_incremental_style: false,
      stored_on_extra_field: ["font"],
    },
    {
      name: "font-variant-east-asian",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      inherited: true,
      font: true,
      name_for_methods: "VariantEastAsian",
      converter: "ConvertFontVariantEastAsian",
      priority: 1,
      keywords: [
         "normal", "jis78", "jis83", "jis90", "jis04", "simplified",
         "traditional", "full-width", "proportional-width", "ruby"
      ],
      typedom_types: ["Keyword"],
      valid_for_first_letter: true,
      valid_for_first_line: true,
      valid_for_marker: true,
      // See comment on font.
      supports_incremental_style: false,
      stored_on_extra_field: ["font"],
    },
    {
      name: "font-variant-numeric",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      inherited: true,
      font: true,
      name_for_methods: "VariantNumeric",
      converter: "ConvertFontVariantNumeric",
      priority: 1,
      keywords: [
        "normal", "lining-nums", "oldstyle-nums", "proportional-nums",
        "tabular-nums", "diagonal-fractions", "stacked-fractions", "ordinal",
        "slashed-zero"
      ],
      typedom_types: ["Keyword"],
      valid_for_first_letter: true,
      valid_for_first_line: true,
      valid_for_marker: true,
      // See comment on font.
      supports_incremental_style: false,
      stored_on_extra_field: ["font"],
    },
    {
      name: "font-variant-alternates",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      inherited: true,
      font: true,
      type_name: "FontVariantAlternates",
      name_for_methods: "FontVariantAlternates",
      converter: "ConvertFontVariantAlternates",
      priority: 1,
      keywords: [
        "normal",
      ],
      typedom_types: ["Keyword"],
      valid_for_first_letter: true,
      valid_for_first_line: true,
      valid_for_marker: true,
      // See comment on font.
      supports_incremental_style: false,
      stored_on_extra_field: ["font"],
    },
    {
      name: "font-weight",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ConsumeFontWeight",
      is_descriptor: true,
      interpolable: true,
      inherited: true,
      font: true,
      name_for_methods: "Weight",
      converter: "ConvertFontWeight",
      priority: 1,
      keywords: ["normal", "bold", "bolder", "lighter"],
      typedom_types: ["Keyword", "Number"],
      valid_for_first_letter: true,
      valid_for_first_line: true,
      valid_for_cue: true,
      valid_for_marker: true,
      // See comment on font.
      supports_incremental_style: false,
      valid_for_permission_element: true,
      valid_for_page_context: true,
      stored_on_extra_field: ["font"],
    },
    {
      name: "font-synthesis-weight",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      inherited: true,
      font: true,
      name_for_methods: "FontSynthesisWeight",
      type_name: "FontDescription::FontSynthesisWeight",
      priority: 1,
      keywords: ["auto", "none"],
      typedom_types: ["Keyword"],
      valid_for_first_letter: true,
      valid_for_first_line: true,
      valid_for_marker: true,
      // See comment on font.
      supports_incremental_style: false,
      valid_for_permission_element: true,
      stored_on_extra_field: ["font"],
    },
    {
      name: "font-synthesis-style",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      inherited: true,
      font: true,
      name_for_methods: "FontSynthesisStyle",
      type_name: "FontDescription::FontSynthesisStyle",
      priority: 1,
      keywords: ["auto", "none"],
      typedom_types: ["Keyword"],
      valid_for_first_letter: true,
      valid_for_first_line: true,
      valid_for_marker: true,
      // See comment on font.
      supports_incremental_style: false,
      valid_for_permission_element: true,
      stored_on_extra_field: ["font"],
    },
    {
      name: "font-synthesis-small-caps",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      inherited: true,
      font: true,
      name_for_methods: "FontSynthesisSmallCaps",
      type_name: "FontDescription::FontSynthesisSmallCaps",
      priority: 1,
      keywords: ["auto", "none"],
      typedom_types: ["Keyword"],
      valid_for_first_letter: true,
      valid_for_first_line: true,
      valid_for_marker: true,
      // See comment on font.
      supports_incremental_style: false,
      valid_for_permission_element: true,
      stored_on_extra_field: ["font"],
    },
    {
      name: "font-feature-settings",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ConsumeFontFeatureSettings",
      is_descriptor: true,
      inherited: true,
      font: true,
      name_for_methods: "FeatureSettings",
      converter: "ConvertFontFeatureSettings",
      priority: 1,
      keywords: ["normal"],
      typedom_types: ["Keyword"],
      valid_for_first_letter: true,
      valid_for_first_line: true,
      valid_for_marker: true,
      // See comment on font.
      supports_incremental_style: false,
      valid_for_permission_element: true,
      stored_on_extra_field: ["font"],
    },
    {
      name: "font-variation-settings",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      is_descriptor: true,
      interpolable: true,
      inherited: true,
      font: true,
      name_for_methods: "VariationSettings",
      converter: "ConvertFontVariationSettings",
      priority: 1,
      keywords: ["normal"],
      typedom_types: ["Keyword"],
      valid_for_first_letter: true,
      valid_for_first_line: true,
      valid_for_cue: true,
      valid_for_marker: true,
      // See comment on font.
      supports_incremental_style: false,
      stored_on_extra_field: ["font"],
    },
    {
      name: "font-language-override",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      inherited: true,
      font: true,
      runtime_flag: "FontLanguageOverride",
      priority: 1,
      keywords: ["normal"],
      typedom_types: ["Keyword"],
      converter: "ConvertFontLanguageOverride",
      stored_on_extra_field: ["font"],
    },
    {
      name: "font-variant-emoji",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      inherited: true,
      font: true,
      name_for_methods: "VariantEmoji",
      type_name: "FontDescription::FontVariantEmoji",
      converter: "ConvertFontVariantEmoji",
      priority: 1,
      keywords: ["normal", "text", "emoji", "unicode"],
      typedom_types: ["Keyword"],
      valid_for_first_letter: true,
      valid_for_first_line: true,
      valid_for_marker: true,
      supports_incremental_style: false,
      stored_on_extra_field: ["font"],
    },
    {
      name: "font-variant-position",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      inherited: true,
      font: true,
      name_for_methods: "VariantPosition",
      type_name: "FontDescription::FontVariantPosition",
      converter: "ConvertFontVariantPosition",
      priority: 1,
      keywords: ["normal", "sub", "super"],
      typedom_types: ["Keyword"],
      valid_for_first_letter: true,
      valid_for_first_line: true,
      valid_for_marker: true,
      // See comment on font.
      supports_incremental_style: false,
      stored_on_extra_field: ["font"],
    },
    {
      name: "-webkit-font-smoothing",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      inherited: true,
      font: true,
      type_name: "FontSmoothingMode",
      priority: 1,
      valid_for_first_letter: true,
      valid_for_first_line: true,
      valid_for_permission_element: true,
      stored_on_extra_field: ["font"],
      keywords: ["auto", "none", "antialiased", "subpixel-antialiased"],
    },
    {
      name: "forced-color-adjust",
      field_group: "*",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      inherited: true,
      runtime_flag: "ForcedColors",
      field_template: "keyword",
      // Affects the computed value of color when it is inherited and
      // forced-color- adjust is set to preserve-parent-color.
      priority: 2,
      keywords: ["auto", "none", "preserve-parent-color"],
      typedom_types: ["Keyword"],
      default_value: "auto",
      valid_for_permission_element: true,
      highlight_style_comes_from_originating_element: true,
    },
    {
      name: "field-sizing",
      field_group: "visual",
      field_template: "keyword",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      keywords: ["fixed", "content"],
      default_value: "fixed",
      typedom_types: ["Keyword"],
      invalidate: ["layout"],
    },
    {
      name: "-webkit-locale",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      inherited: true,
      font: true,
      style_builder_custom_functions: ["value"],
      priority: 1,
      stored_on_extra_field: ["font"],
      keywords: ["auto"],
    },
    {
      name: "math-depth",
      default_value: 0,
      field_group: "*",
      field_template: "primitive",
      interpolable: true,
      inherited: true,
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ConsumeMathDepth",
      style_builder_custom_functions: ["value"],
      type_name: "short",
      typedom_types: ["Number"],
      // Affects the computed value of 'font-size', hence needs to happen before
      // high-priority properties.
      priority: 2,
    },
    {
      name: "text-orientation",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      inherited: true,
      field_group: "*",
      field_template: "keyword",
      keywords: ["sideways", "mixed", "upright", "sideways-right"],
      typedom_types: ["Keyword"],
      default_value: "mixed",
      getter: "GetTextOrientation",
      style_builder_custom_functions: ["initial", "inherit", "value"],
      priority: 1,
      invalidate: ["layout", "paint"],
      valid_for_marker: true,
      is_animation_affecting: true,
    },
    {
      name: "-webkit-text-orientation",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      inherited: true,
      type_name: "TextOrientation",
      priority: 1,
      surrogate_for: "text-orientation",
      valid_for_marker: true,
      keywords: ["sideways", "upright", "sideways-right", "vertical-right"],
    },
    {
      name: "writing-mode",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      inherited: true,
      field_template: "keyword",
      include_paths: ["third_party/blink/renderer/platform/text/writing_mode.h"],
      keywords: ["horizontal-tb", "vertical-rl", "vertical-lr",
                 "sideways-rl", "sideways-lr"],
      typedom_types: ["Keyword"],
      default_value: "horizontal-tb",
      type_name: "WritingMode",
      style_builder_custom_functions: ["initial", "inherit", "value"],
      priority: 1,
      valid_for_page_context: true,
      // Incremental code does not call DidChangeWritingMode(), which influences
      // the font.
      supports_incremental_style: false,
      invalidate: ["ax-style", "layout", "paint"],
      highlight_style_comes_from_originating_element: true,
      is_animation_affecting: true,
    },
    {
      name: "-webkit-writing-mode",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      inherited: true,
      type_name: "WritingMode",
      priority: 1,
      surrogate_for: "writing-mode",
      is_animation_affecting: true,
    },
    {
      name: "text-rendering",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      inherited: true,
      font: true,
      type_name: "TextRenderingMode",
      keywords: ["auto", "optimizespeed", "optimizelegibility", "geometricprecision"],
      typedom_types: ["Keyword"],
      priority: 1,
      valid_for_permission_element: true,
      stored_on_extra_field: ["font"],
    },
    {
      name: "zoom",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      field_group: "visual",
      field_template: "primitive",
      default_value: "1.0",
      type_name: "float",
      style_builder_custom_functions: ["initial", "inherit", "value"],
      priority: 1,
      // Setting zoom affects the _EffectiveZoom_, which in turns affects every px value
      // stored on ComputedStyle; see CSSToLengthConversionData::ZoomedComputedPixels.
      supports_incremental_style: false,
      valid_for_permission_element: true,
      percentages_depend_on_used_value: true,
    },
    {
      name: "accent-color",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      inherited: true,
      field_group: "*",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/css/style_auto_color.h"],
      type_name: "StyleAutoColor",
      computed_style_protected_functions: ["getter"],
      keywords: ["auto", "currentcolor"],
      typedom_types: ["Keyword"],
      converter: "ConvertStyleAutoColor",
      default_value: "StyleAutoColor::AutoColor()",
      computable: true,
      invalidate: ["accent-color"],
      includes_currentcolor: true,
    },
    {
      name: "align-content",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      field_group: "*",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/style/style_content_alignment_data.h"],
      default_value: "StyleContentAlignmentData(ContentPosition::kNormal, ContentDistributionType::kDefault, OverflowAlignment::kDefault)",
      type_name: "StyleContentAlignmentData",
      converter: "ConvertContentAlignmentData",
      invalidate: ["layout"],
    },
    {
      name: "align-items",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      field_group: "box",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/style/style_self_alignment_data.h"],
      default_value: "StyleSelfAlignmentData(ItemPosition::kNormal, OverflowAlignment::kDefault)",
      type_name: "StyleSelfAlignmentData",
      converter: "ConvertSelfOrDefaultAlignmentData",
      invalidate: ["layout"],
    },
    {
      name: "alignment-baseline",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      field_group: "svg",
      field_template: "keyword",
      keywords: ["auto", "baseline", "alphabetic", "ideographic", "middle",
                 "central", "mathematical", "before-edge", "text-before-edge",
                 "after-edge", "text-after-edge", "hanging"],
      typedom_types: ["Keyword"],
      default_value: "auto",
      invalidate: ["layout", "paint"],
    },
    {
      name: "align-self",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      field_group: "*",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/style/style_self_alignment_data.h"],
      default_value: "StyleSelfAlignmentData(ItemPosition::kAuto, OverflowAlignment::kDefault)",
      type_name: "StyleSelfAlignmentData",
      converter: "ConvertSelfOrDefaultAlignmentData",
      valid_for_position_try: true,
      valid_for_permission_element: true,
      invalidate: ["layout"],
      keywords: ["auto", "normal", "stretch", "baseline", "center", "start",
                 "end", "flow-start", "flow-end", "self-start", "self-end",
                 "flex-start", "flex-end", "anchor-center"],
    },
    {
      name: "anchor-name",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal" ],
      include_paths: ["third_party/blink/renderer/core/style/scoped_css_name.h"],
      type_name: "ScopedCSSNameList",
      wrapper_pointer_name: "Member",
      default_value: "nullptr",
      field_group: "*",
      field_template: "external",
      converter: "ConvertAnchorName",
      keywords: ["none"],
      typedom_types: ["Keyword"],
      valid_for_permission_element: true,
      invalidate: ["layout"],
    },
    {
      name: "anchor-scope",
      property_methods: ["CSSValueFromComputedStyleInternal" ],
      parse_helper: "ConsumeNameScope",
      include_paths: ["third_party/blink/renderer/core/style/style_anchor_scope.h"],
      type_name: "StyleAnchorScope",
      default_value: "StyleAnchorScope()",
      field_group: "*",
      field_template: "external",
      converter: "ConvertAnchorScope",
      keywords: ["none", "all"],
      typedom_types: ["Keyword"],
      valid_for_permission_element: true,
      invalidate: ["layout"],
    },
    {
      name: "aspect-ratio",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      field_group: "surround",
      field_template: "external",
      keywords: ["auto"],
      default_value: "StyleAspectRatio(EAspectRatioType::kAuto, gfx::SizeF())",
      type_name: "StyleAspectRatio",
      converter: "ConvertAspectRatio",
      include_paths: ["third_party/blink/renderer/core/style/style_aspect_ratio.h"],
      valid_for_permission_element: true,
      invalidate: ["layout"],
    },
    {
      name: "backdrop-filter",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ConsumeFilterFunctionList",
      interpolable: true,
      compositable: true,
      tracks_animated_source: true,
      field_group: "*",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/style/filter_operations.h"],
      default_value: "FilterOperations()",
      type_name: "FilterOperations",
      computed_style_custom_functions: ["initial"],
      style_builder_custom_functions: ["value"],
      keywords: ["none"],
      typedom_types: ["Keyword"],
      invalidate: ["compositing"],
      includes_currentcolor: true,
      affected_by_zoom: true,
    },
    {
      name: "backface-visibility",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      field_group: "*",
      field_template: "keyword",
      keywords: ["visible", "hidden"],
      typedom_types: ["Keyword"],
      default_value: "visible",
      invalidate: ["compositing"],
    },
    {
      name: "background-attachment",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      keywords: ["scroll", "fixed", "local"],
      typedom_types: ["Keyword"],
      separator: " ",
      style_builder_template: "background_layer",
      style_builder_template_args: {
        fill_type: "Attachment",
      },
      valid_for_first_letter: true,
      valid_for_first_line: true,
      valid_for_cue: true,
      is_background: true,
      supports_incremental_style: true,
      valid_for_page_context: true,
      stored_on_extra_field: ["Background"],
    },
    {
      name: "background-blend-mode",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      keywords: [
        "normal", "multiply", "screen", "overlay", "darken", "lighten",
        "color-dodge", "color-burn", "hard-light", "soft-light", "difference",
        "exclusion", "hue", "saturation", "color", "luminosity"
      ],
      typedom_types: ["Keyword"],
      separator: " ",
      style_builder_template: "background_layer",
      style_builder_template_args: {
        fill_type: "BlendMode",
        fill_type_getter: "GetBlendMode",
      },
      valid_for_first_letter: true,
      valid_for_first_line: true,
      is_background: false,
      valid_for_page_context: true,
      stored_on_extra_field: ["Background"],
    },
    {
      name: "background-clip",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      keywords: ["border-box", "padding-box", "content-box", "text", "border-area"],
      typedom_types: ["Keyword"],
      separator: " ",
      style_builder_template: "background_layer",
      style_builder_template_args: {
        fill_type: "Clip",
      },
      style_builder_custom_functions: ["value"],
      valid_for_first_letter: true,
      valid_for_first_line: true,
      valid_for_cue: true,
      is_background: true,
      supports_incremental_style: true,
      valid_for_page_context: true,
      stored_on_extra_field: ["Background"],
    },
    {
      name: "background-color",
      property_methods: ["CSSValueFromComputedStyleInternal", "ColorIncludingFallback"],
      parse_helper: "ConsumeColorMaybeQuirky",
      interpolable: true,
      compositable: true,
      tracks_animated_source: true,
      field_group: "background",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/css/style_color.h"],
      default_value: "StyleColor(Color::kTransparent)",
      type_name: "StyleColor",
      computed_style_protected_functions: ["getter"],
      keywords: ["currentcolor"],
      includes_currentcolor: true,
      typedom_types: ["Keyword"],
      converter: "ConvertStyleColor",
      style_builder_template: "color",
      style_builder_template_args: {
        initial_color: "ComputedStyleInitialValues::InitialBackgroundColor",
      },
      valid_for_first_letter: true,
      valid_for_first_line: true,
      valid_for_cue: true,
      valid_for_highlight: true,
      is_background: true,
      is_highlight_colors: true,
      supports_incremental_style: true,
      valid_for_permission_element: true,
      valid_for_page_context: true,
      invalidate: ["ax-style", "background-color"],
    },
    {
      name: "background-image",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      keywords: ["none"],
      typedom_types: ["Keyword", "Image"],
      separator: " ",
      style_builder_template: "background_layer",
      style_builder_template_args: {
        fill_type: "Image",
        fill_type_getter: "GetImage",
      },
      valid_for_first_letter: true,
      valid_for_first_line: true,
      valid_for_cue: true,
      is_background: true,
      supports_incremental_style: true,
      valid_for_page_context: true,
      includes_currentcolor: true,
      stored_on_extra_field: ["Background"],
    },
    {
      name: "background-origin",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      keywords: ["border-box", "padding-box", "content-box"],
      typedom_types: ["Keyword"],
      separator: " ",
      style_builder_template: "background_layer",
      style_builder_template_args: {
        fill_type: "Origin",
      },
      valid_for_first_letter: true,
      valid_for_first_line: true,
      valid_for_cue: true,
      is_background: true,
      supports_incremental_style: true,
      valid_for_page_context: true,
      stored_on_extra_field: ["Background"],
    },
    {
      name: "background-position-x",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      style_builder_template: "background_layer",
      style_builder_template_args: {
        fill_type: "PositionX",
      },
      valid_for_first_letter: true,
      valid_for_first_line: true,
      valid_for_cue: true,
      is_background: true,
      computable: false,
      supports_incremental_style: true,
      valid_for_page_context: true,
      stored_on_extra_field: ["Background"],
      percentages_depend_on_used_value: true,
      affected_by_zoom: true,
    },
    {
      name: "background-position-y",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      style_builder_template: "background_layer",
      style_builder_template_args: {
        fill_type: "PositionY",
      },
      valid_for_first_letter: true,
      valid_for_first_line: true,
      valid_for_cue: true,
      is_background: true,
      computable: false,
      supports_incremental_style: true,
      valid_for_page_context: true,
      stored_on_extra_field: ["Background"],
      percentages_depend_on_used_value: true,
      affected_by_zoom: true,
    },
    {
      name: "background-repeat",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      style_builder_template: "background_layer",
      style_builder_template_args: {
        fill_type: "Repeat",
      },
      valid_for_first_letter: true,
      valid_for_first_line: true,
      valid_for_cue: true,
      supports_incremental_style: true,
      valid_for_page_context: true,
      stored_on_extra_field: ["Background"],
    },
    {
      name: "background-size",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ParseBackgroundSize",
      interpolable: true,
      keywords: ["auto", "cover", "contain"],
      typedom_types: ["Keyword", "Length", "Percentage"],
      separator: " ",
      style_builder_template: "background_layer",
      style_builder_template_args: {
        fill_type: "Size",
      },
      valid_for_first_letter: true,
      valid_for_first_line: true,
      valid_for_cue: true,
      is_background: true,
      supports_incremental_style: true,
      valid_for_page_context: true,
      stored_on_extra_field: ["Background"],
      percentages_depend_on_used_value: true,
      affected_by_zoom: true,
    },
    {
      name: "baseline-shift",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      field_group: "svg->svgmisc",
      field_template: "external",
      type_name: "Length",
      default_value: "Length::Fixed()",
      style_builder_custom_functions: ["inherit", "value"],
      keywords: ["baseline", "sub", "super"],
      typedom_types: ["Keyword", "Percentage", "Length"],
      invalidate: ["layout", "paint"],
      percentages_depend_on_used_value: false,
      affected_by_zoom: true,
    },
    {
      name: "baseline-source",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      field_group: "box",
      field_template: "keyword",
      default_value: "auto",
      keywords: ["auto", "first", "last"],
      typedom_types: ["Keyword"],
      invalidate: ["layout"],
    },
    {
      name: "border-bottom-color",
      property_methods: ["CSSValueFromComputedStyleInternal", "ColorIncludingFallback", "InitialValue"],
      parse_helper: "ConsumeBorderColorSide",
      interpolable: true,
      field_group: "surround",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/css/style_color.h"],
      default_value: "StyleColor::CurrentColor()",
      type_name: "StyleColor",
      computed_style_protected_functions: ["getter"],
      keywords: ["currentcolor"],
      includes_currentcolor: true,
      typedom_types: ["Keyword"],
      converter: "ConvertStyleColor",
      style_builder_template: "color",
      valid_for_first_letter: true,
      is_border: true,
      logical_property_group: {
        name: "border-color",
        resolver: "bottom",
      },
      supports_incremental_style: true,
      valid_for_permission_element: true,
      valid_for_page_context: true,
      invalidate: ["border-visual"],
    },
    {
      name: "border-bottom-left-radius",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ParseBorderRadiusCorner",
      interpolable: true,
      field_group: "surround",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/platform/geometry/length_size.h"],
      default_value: "LengthSize(Length::Fixed(0), Length::Fixed(0))",
      type_name: "LengthSize",
      converter: "ConvertRadius",
      typedom_types: ["Length", "Percentage"],
      valid_for_first_letter: true,
      is_border: true,
      is_border_radius: true,
      logical_property_group: {
        name: "border-radius",
        resolver: "bottom-left",
      },
      valid_for_permission_element: true,
      valid_for_page_context: true,
      invalidate: ["border-radius", "paint"],
      percentages_depend_on_used_value: true,
      affected_by_zoom: true,
    },
    {
      name: "border-bottom-right-radius",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ParseBorderRadiusCorner",
      interpolable: true,
      field_group: "surround",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/platform/geometry/length_size.h"],
      default_value: "LengthSize(Length::Fixed(0), Length::Fixed(0))",
      type_name: "LengthSize",
      converter: "ConvertRadius",
      typedom_types: ["Length", "Percentage"],
      valid_for_first_letter: true,
      is_border: true,
      is_border_radius: true,
      logical_property_group: {
        name: "border-radius",
        resolver: "bottom-right",
      },
      valid_for_permission_element: true,
      valid_for_page_context: true,
      invalidate: ["border-radius", "paint"],
      percentages_depend_on_used_value: true,
      affected_by_zoom: true,
    },
    {
      name: "border-bottom-style",
      property_methods: ["CSSValueFromComputedStyleInternal", "InitialValue"],
      field_group: "box",
      field_template: "keyword",
      keywords: [
        "none", "hidden", "inset", "groove", "outset", "ridge", "dotted",
        "dashed", "solid", "double"
      ],
      typedom_types: ["Keyword"],
      default_value: "none",
      type_name: "EBorderStyle",
      valid_for_first_letter: true,
      is_border: true,
      logical_property_group: {
        name: "border-style",
        resolver: "bottom",
      },
      valid_for_permission_element: true,
      valid_for_page_context: true,
      invalidate: ["border-width", "border-visual"],
    },
    {
      name: "border-bottom-width",
      property_methods: ["CSSValueFromComputedStyleInternal", "InitialValue"],
      parse_helper: "ParseBorderWidthSide",
      interpolable: true,
      field_group: "box",
      field_template: "external",
      keywords: ["thin", "medium", "thick"],
      default_value: "3",
      typedom_types: ["Keyword", "Length"],
      type_name: "int",
      getter: "SpecifiedBorderBottomWidth",
      style_builder_custom_functions: ["initial", "inherit"],
      converter: "ConvertBorderWidth",
      valid_for_first_letter: true,
      is_border: true,
      logical_property_group: {
        name: "border-width",
        resolver: "bottom",
      },
      // Overlaps with -webkit-border-image.
      overlapping: true,
      valid_for_permission_element: true,
      valid_for_page_context: true,
      invalidate: ["border-width", "border-visual"],
      affected_by_zoom: true,
    },
    {
      name: "border-collapse",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      independent: true,
      inherited: true,
      field_template: "keyword",
      keywords: ["separate", "collapse"],
      typedom_types: ["Keyword"],
      default_value: "separate",
      invalidate: ["layout", "paint"],
    },
    {
      name: "border-image-outset",
      property_methods: ["CSSValueFromComputedStyleInternal", "InitialValue"],
      parse_helper: "ConsumeBorderImageOutset",
      interpolable: true,
      typedom_types: ["Length", "Number"],
      style_builder_template: "border_image",
      style_builder_template_args: {
        modifier_type: "Outset",
      },
      valid_for_first_letter: true,
      is_border: true,
      // Overlaps with -webkit-border-image.
      overlapping: true,
      stored_on_extra_field: ["border-image"],
      affected_by_zoom: true,
    },
    {
      name: "border-image-repeat",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal", "InitialValue"],
      keywords: ["stretch", "repeat", "round", "space"],
      typedom_types: ["Keyword"],
      style_builder_template: "border_image",
      style_builder_template_args: {
        modifier_type: "Repeat",
      },
      valid_for_first_letter: true,
      is_border: true,
      // Overlaps with -webkit-border-image.
      overlapping: true,
      stored_on_extra_field: ["border-image"],
    },
    {
      name: "border-image-slice",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal", "InitialValue"],
      interpolable: true,
      typedom_types: ["Number", "Percentage"],
      style_builder_template: "border_image",
      style_builder_template_args: {
        modifier_type: "Slice",
      },
      valid_for_first_letter: true,
      is_border: true,
      // Overlaps with -webkit-border-image.
      overlapping: true,
      stored_on_extra_field: ["border-image"],
      percentages_depend_on_used_value: true,
    },
    {
      name: "border-image-source",
      property_methods: ["CSSValueFromComputedStyleInternal", "InitialValue"],
      parse_helper: "ConsumeImageOrNone",
      interpolable: true,
      keywords: ["none"],
      typedom_types: ["Keyword", "Image"],
      style_builder_custom_functions: ["value"],
      valid_for_first_letter: true,
      is_border: true,
      // Overlaps with -webkit-border-image.
      overlapping: true,
      includes_currentcolor: true,
      stored_on_extra_field: ["border-image"],
    },
    {
      name: "border-image-width",
      property_methods: ["CSSValueFromComputedStyleInternal", "InitialValue"],
      parse_helper: "ConsumeBorderImageWidth",
      interpolable: true,
      keywords: ["auto"],
      typedom_types: ["Keyword", "Length", "Percentage", "Number"],
      style_builder_template: "border_image",
      style_builder_template_args: {
        modifier_type: "Width",
      },
      valid_for_first_letter: true,
      is_border: true,
      // Overlaps with -webkit-border-image.
      overlapping: true,
      stored_on_extra_field: ["border-image"],
      percentages_depend_on_used_value: true,
      affected_by_zoom: true,
    },
    {
      name: "border-left-color",
      property_methods: ["CSSValueFromComputedStyleInternal", "ColorIncludingFallback", "InitialValue"],
      parse_helper: "ConsumeBorderColorSide",
      interpolable: true,
      field_group: "surround",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/css/style_color.h"],
      default_value: "StyleColor::CurrentColor()",
      type_name: "StyleColor",
      computed_style_protected_functions: ["getter"],
      keywords: ["currentcolor"],
      includes_currentcolor: true,
      typedom_types: ["Keyword"],
      converter: "ConvertStyleColor",
      style_builder_template: "color",
      valid_for_first_letter: true,
      is_border: true,
      logical_property_group: {
        name: "border-color",
        resolver: "left",
      },
      supports_incremental_style: true,
      valid_for_permission_element: true,
      valid_for_page_context: true,
      invalidate: ["border-visual"],
    },
    {
      name: "border-left-style",
      property_methods: ["CSSValueFromComputedStyleInternal", "InitialValue"],
      field_group: "box",
      field_template: "keyword",
      keywords: [
        "none", "hidden", "inset", "groove", "outset", "ridge", "dotted",
        "dashed", "solid", "double"
      ],
      typedom_types: ["Keyword"],
      default_value: "none",
      type_name: "EBorderStyle",
      valid_for_first_letter: true,
      is_border: true,
      logical_property_group: {
        name: "border-style",
        resolver: "left",
      },
      valid_for_permission_element: true,
      valid_for_page_context: true,
      invalidate: ["border-width", "border-visual"],
    },
    {
      name: "border-left-width",
      property_methods: ["CSSValueFromComputedStyleInternal", "InitialValue"],
      parse_helper: "ParseBorderWidthSide",
      interpolable: true,
      field_group: "box",
      field_template: "external",
      keywords: ["thin", "medium", "thick"],
      default_value: "3",
      typedom_types: ["Keyword", "Length"],
      type_name: "int",
      getter: "SpecifiedBorderLeftWidth",
      style_builder_custom_functions: ["initial", "inherit"],
      converter: "ConvertBorderWidth",
      valid_for_first_letter: true,
      is_border: true,
      logical_property_group: {
        name: "border-width",
        resolver: "left",
      },
      // Overlaps with -webkit-border-image.
      overlapping: true,
      valid_for_permission_element: true,
      valid_for_page_context: true,
      invalidate: ["border-width", "border-visual"],
      affected_by_zoom: true,
    },
    {
      name: "border-right-color",
      property_methods: ["CSSValueFromComputedStyleInternal", "ColorIncludingFallback", "InitialValue"],
      parse_helper: "ConsumeBorderColorSide",
      interpolable: true,
      field_group: "surround",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/css/style_color.h"],
      default_value: "StyleColor::CurrentColor()",
      type_name: "StyleColor",
      computed_style_protected_functions: ["getter"],
      keywords: ["currentcolor"],
      includes_currentcolor: true,
      typedom_types: ["Keyword"],
      converter: "ConvertStyleColor",
      style_builder_template: "color",
      valid_for_first_letter: true,
      is_border: true,
      logical_property_group: {
        name: "border-color",
        resolver: "right",
      },
      supports_incremental_style: true,
      valid_for_permission_element: true,
      valid_for_page_context: true,
      invalidate: ["border-visual"],
    },
    {
      name: "border-right-style",
      property_methods: ["CSSValueFromComputedStyleInternal", "InitialValue"],
      field_group: "box",
      field_template: "keyword",
      keywords: [
        "none", "hidden", "inset", "groove", "outset", "ridge", "dotted",
        "dashed", "solid", "double"
      ],
      typedom_types: ["Keyword"],
      default_value: "none",
      type_name: "EBorderStyle",
      valid_for_first_letter: true,
      is_border: true,
      logical_property_group: {
        name: "border-style",
        resolver: "right",
      },
      valid_for_permission_element: true,
      valid_for_page_context: true,
      invalidate: ["border-width", "border-visual"],
    },
    {
      name: "border-right-width",
      property_methods: ["CSSValueFromComputedStyleInternal", "InitialValue"],
      parse_helper: "ParseBorderWidthSide",
      interpolable: true,
      field_group: "box",
      field_template: "external",
      keywords: ["thin", "medium", "thick"],
      default_value: "3",
      typedom_types: ["Keyword", "Length"],
      type_name: "int",
      getter: "SpecifiedBorderRightWidth",
      style_builder_custom_functions: ["initial", "inherit"],
      converter: "ConvertBorderWidth",
      valid_for_first_letter: true,
      is_border: true,
      logical_property_group: {
        name: "border-width",
        resolver: "right",
      },
      // Overlaps with -webkit-border-image.
      overlapping: true,
      valid_for_permission_element: true,
      valid_for_page_context: true,
      invalidate: ["border-width", "border-visual"],
      affected_by_zoom: true,
    },
    {
      name: "border-top-color",
      property_methods: ["CSSValueFromComputedStyleInternal", "ColorIncludingFallback", "InitialValue"],
      parse_helper: "ConsumeBorderColorSide",
      interpolable: true,
      field_group: "surround",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/css/style_color.h"],
      default_value: "StyleColor::CurrentColor()",
      type_name: "StyleColor",
      computed_style_protected_functions: ["getter"],
      keywords: ["currentcolor"],
      includes_currentcolor: true,
      typedom_types: ["Keyword"],
      converter: "ConvertStyleColor",
      style_builder_template: "color",
      valid_for_first_letter: true,
      is_border: true,
      logical_property_group: {
        name: "border-color",
        resolver: "top",
      },
      supports_incremental_style: true,
      valid_for_permission_element: true,
      valid_for_page_context: true,
      invalidate: ["border-visual"],
    },
    {
      name: "border-top-left-radius",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ParseBorderRadiusCorner",
      interpolable: true,
      field_group: "surround",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/platform/geometry/length_size.h"],
      default_value: "LengthSize(Length::Fixed(0), Length::Fixed(0))",
      type_name: "LengthSize",
      converter: "ConvertRadius",
      typedom_types: ["Length", "Percentage"],
      valid_for_first_letter: true,
      is_border: true,
      is_border_radius: true,
      logical_property_group: {
        name: "border-radius",
        resolver: "top-left",
      },
      valid_for_permission_element: true,
      valid_for_page_context: true,
      invalidate: ["border-radius", "paint"],
      percentages_depend_on_used_value: true,
      affected_by_zoom: true,
    },
    {
      name: "border-top-right-radius",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ParseBorderRadiusCorner",
      interpolable: true,
      field_group: "surround",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/platform/geometry/length_size.h"],
      default_value: "LengthSize(Length::Fixed(0), Length::Fixed(0))",
      type_name: "LengthSize",
      converter: "ConvertRadius",
      typedom_types: ["Length", "Percentage"],
      valid_for_first_letter: true,
      is_border: true,
      is_border_radius: true,
      logical_property_group: {
        name: "border-radius",
        resolver: "top-right",
      },
      valid_for_permission_element: true,
      valid_for_page_context: true,
      invalidate: ["border-radius", "paint"],
      percentages_depend_on_used_value: true,
      affected_by_zoom: true,
    },
    {
      name: "border-top-style",
      property_methods: ["CSSValueFromComputedStyleInternal", "InitialValue"],
      field_group: "box",
      field_template: "keyword",
      keywords: [
        "none", "hidden", "inset", "groove", "outset", "ridge", "dotted",
        "dashed", "solid", "double"
      ],
      typedom_types: ["Keyword"],
      default_value: "none",
      type_name: "EBorderStyle",
      valid_for_first_letter: true,
      is_border: true,
      logical_property_group: {
        name: "border-style",
        resolver: "top",
      },
      supports_incremental_style: true,
      valid_for_permission_element: true,
      valid_for_page_context: true,
      invalidate: ["border-width", "border-visual"],
    },
    {
      name: "border-top-width",
      property_methods: ["CSSValueFromComputedStyleInternal", "InitialValue"],
      parse_helper: "ParseBorderWidthSide",
      interpolable: true,
      field_group: "box",
      field_template: "external",
      keywords: ["thin", "medium", "thick"],
      default_value: "3",
      typedom_types: ["Keyword", "Length"],
      type_name: "int",
      getter: "SpecifiedBorderTopWidth",
      style_builder_custom_functions: ["initial", "inherit"],
      converter: "ConvertBorderWidth",
      valid_for_first_letter: true,
      is_border: true,
      logical_property_group: {
        name: "border-width",
        resolver: "top",
      },
      // Overlaps with -webkit-border-image.
      overlapping: true,
      valid_for_permission_element: true,
      valid_for_page_context: true,
      invalidate: ["border-width", "border-visual"],
      affected_by_zoom: true,
    },
    {
      name: "border-shape",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      runtime_flag: "CSSBorderShape",
      interpolable: true,
      compositable: false,
      field_group: "*",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/style/style_border_shape.h"],
      wrapper_pointer_name: "Member",
      default_value: "nullptr",
      type_name: "StyleBorderShape",
      converter: "ConvertBorderShape",
      keywords: ["none"],
      typedom_types: ["Keyword"],
      invalidate: ["border-visual", "paint", "visual-overflow", "border-shape"],
      affected_by_zoom: true,
    },
    {
      name: "bottom",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      layout_dependent: true,
      field_group: "surround",
      field_template: "<length>",
      keywords: ["auto"],
      default_value: "Length()",
      typedom_types: ["Keyword", "Length", "Percentage"],
      converter: "ConvertLengthOrAuto",
      anchor_mode: "bottom",
      logical_property_group: {
        name: "inset",
        resolver: "bottom",
      },
      supports_incremental_style: true,
      valid_for_position_try: true,
      valid_for_permission_element: true,
      invalidate: ["inset", "scroll-anchor"],
      affected_by_zoom: true,
      percentages_depend_on_used_value: true,
    },
    {
      name: "box-decoration-break",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      field_group: "box",
      field_template: "keyword",
      keywords: ["slice", "clone"],
      default_value: "slice",
      invalidate: ["layout", "paint"],
    },
    {
      name: "box-shadow",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      field_group: "*",
      field_template: "pointer",
      include_paths: ["third_party/blink/renderer/core/style/shadow_list.h"],
      wrapper_pointer_name: "Member",
      default_value: "nullptr",
      type_name: "ShadowList",
      converter: "ConvertShadowList",
      keywords: ["none"],
      typedom_types: ["Keyword"],
      valid_for_first_letter: true,
      valid_for_first_line: true,
      valid_for_permission_element: true,
      invalidate: ["paint", "visual-overflow"],
      includes_currentcolor: true,
      affected_by_zoom: true,
    },
    {
      name: "box-sizing",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      // NOTE: Naturally fits into field_group: "box", but is so commonly set
      // that is is better to have it at the root.
      field_template: "keyword",
      keywords: ["content-box", "border-box"],
      typedom_types: ["Keyword"],
      default_value: "content-box",
      valid_for_permission_element: true,
      valid_for_page_context: true,
      invalidate: ["layout"],
    },
    {
      name: "break-after",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      // Storage for this property also covers these legacy properties:
      // page-break-after, -webkit-column-break-after
      field_template: "keyword",
      field_group: "*",
      keywords: [
        "auto", "avoid", "avoid-column", "avoid-page", "column", "left", "page",
        "recto", "right", "verso"
      ],
      typedom_types: ["Keyword"],
      default_value: "auto",
      type_name: "EBreakBetween",
      invalidate: ["layout"],
    },
    {
      name: "break-before",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      // Storage for this property also covers these legacy properties:
      // page-break-before, -webkit-column-break-before
      field_template: "keyword",
      field_group: "*",
      keywords: [
        "auto", "avoid", "avoid-column", "avoid-page", "column", "left", "page",
        "recto", "right", "verso"
      ],
      typedom_types: ["Keyword"],
      default_value: "auto",
      type_name: "EBreakBetween",
      invalidate: ["layout"],
    },
    {
      name: "break-inside",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      // Storage for this property also covers these legacy properties:
      // page-break-inside, -webkit-column-break-inside
      field_template: "keyword",
      field_group: "*",
      keywords: ["auto", "avoid", "avoid-column", "avoid-page"],
      typedom_types: ["Keyword"],
      default_value: "auto",
      invalidate: ["layout"],
    },
    {
      name: "buffered-rendering",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      field_group: "svg",
      field_template: "keyword",
      keywords: ["auto", "dynamic", "static"],
      default_value: "auto",
    },
    {
      name: "caption-side",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      independent: true,
      inherited: true,
      field_template: "keyword",
      keywords: ["top", "bottom"],
      typedom_types: ["Keyword"],
      default_value: "top",
      invalidate: ["layout", "paint"],
    },
    {
      name: "caret-animation",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      inherited: true,
      field_template: "keyword",
      keywords: ["auto", "manual"],
      default_value: "auto",
      typedom_types: ["Keyword"],
      invalidate: ["paint"],
      runtime_flag: "CSSCaretAnimation",
    },
    {
      name: "caret-color",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal", "ColorIncludingFallback"],
      interpolable: true,
      inherited: true,
      field_group: "*",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/css/style_caret_color.h"],
      default_value: "StyleCaretColor()",
      type_name: "StyleCaretColor",
      computed_style_protected_functions: ["getter"],
      converter: "ConvertStyleCaretColor",
      keywords: ["auto", "currentcolor"],
      typedom_types: ["Keyword"],
      invalidate: ["color"],
      includes_currentcolor: true,
    },
    {
      name: "caret-shape",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      inherited: true,
      field_template: "keyword",
      keywords: ["auto", "bar", "block", "underscore"],
      default_value: "auto",
      typedom_types: ["Keyword"],
      invalidate: ["paint"],
      runtime_flag: "CSSCaretShape",
    },
    {
      name: "clear",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      field_template: "keyword",
      computed_style_protected_functions: ["getter"],
      keywords: ["none", "left", "right", "both", "inline-start", "inline-end"],
      typedom_types: ["Keyword"],
      default_value: "none",
      valid_for_permission_element: true,
      invalidate: ["layout", "paint"],
    },
    {
      name: "clip",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      field_group: "visual",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/platform/geometry/length_box.h"],
      default_value: "LengthBox()",
      type_name: "LengthBox",
      computed_style_custom_functions: ["setter"],
      style_builder_template: "auto",
      converter: "ConvertClip",
      keywords: ["auto"],
      typedom_types: ["Keyword"],
      invalidate: ["clip"],
      affected_by_zoom: true,
    },
    {
      name: "clip-path",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      compositable: true,
      tracks_animated_source: true,
      field_group: "*",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/style/clip_path_operation.h"],
      wrapper_pointer_name: "Member",
      default_value: "nullptr",
      type_name: "ClipPathOperation",
      computed_style_custom_functions: ["getter", "setter"],
      converter: "ConvertClipPath",
      keywords: ["border-box", "padding-box", "content-box", "margin-box", "fill-box", "stroke-box", "view-box", "none"],
      typedom_types: ["Keyword"],
      invalidate: ["clip-path"],
      affected_by_zoom: true,
    },
    {
      name: "clip-rule",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      inherited: true,
      field_group: "svginherited",
      // TODO(fs): Convert this to a keyword (requires enum massage).
      field_template: "primitive",
      field_size: 1,
      include_paths: ["third_party/blink/renderer/platform/geometry/path_types.h"],
      type_name: "WindRule",
      keywords: ["nonzero", "evenodd"],
      default_value: "RULE_NONZERO",
      typedom_types: ["Keyword"],
      invalidate: ["paint"],
    },
    {
      name: "color-interpolation",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      inherited: true,
      field_group: "svginherited",
      field_template: "keyword",
      type_name: "EColorInterpolation",
      keywords: ["auto", "srgb", "linearrgb"],
      default_value: "srgb",
      typedom_types: ["Keyword"],
      invalidate: ["paint"],
    },
    {
      name: "color-interpolation-filters",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      inherited: true,
      field_group: "svginherited",
      field_template: "keyword",
      type_name: "EColorInterpolation",
      keywords: ["auto", "srgb", "linearrgb"],
      default_value: "linearrgb",
      typedom_types: ["Keyword"],
      invalidate: ["paint"],
    },
    {
      name: "color-rendering",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      inherited: true,
      field_group: "svginherited",
      field_template: "keyword",
      keywords: ["auto", "optimizespeed", "optimizequality"],
      default_value: "auto",
      typedom_types: ["Keyword"],
    },
    {
      name: "color-scheme",
      field_group: "*",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal", "InitialValue"],
      style_builder_custom_functions: ["initial", "inherit", "value"],
      inherited: true,
      include_paths: ["third_party/blink/public/mojom/frame/color_scheme.mojom-blink.h"],
      type_name: "Vector<AtomicString>",
      default_value: "Vector<AtomicString, 0>()",
      field_template: "external",
      // Affects the computed value of 'color', hence needs to happen before
      // high-priority properties.
      priority: 2,
      valid_for_permission_element: true,
      keywords: ["normal", "light", "dark"],
    },
    {
      name: "column-fill",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      field_group: "*",
      field_template: "keyword",
      keywords: ["balance", "auto"],
      default_value: "balance",
      getter: "GetColumnFill",
      typedom_types: ["Keyword"],
      invalidate: ["layout", "paint"],
    },
    {
      name: "contain",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      field_group: "*",
      field_size: 5,
      field_template: "primitive",
      default_value: "kContainsNone",
      name_for_methods: "Contain",
      type_name: "unsigned",
      converter: "ConvertFlags<Containment>",
      keywords: ["none", "strict", "content", "size", "layout", "style", "paint", "inline-size"],
      typedom_types: ["Keyword"],
      invalidate: ["layout"],
      is_animation_affecting: true,
    },
    {
      name: "contain-intrinsic-width",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ConsumeIntrinsicSizeLonghand",
      interpolable: true,
      field_group: "surround",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/style/style_intrinsic_length.h"],
      keywords: ["none"],
      default_value: "StyleIntrinsicLength()",
      type_name: "StyleIntrinsicLength",
      converter: "ConvertIntrinsicDimension",
      invalidate: ["layout", "scroll-anchor"],
      affected_by_zoom: true,
    },
    {
      name: "contain-intrinsic-height",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ConsumeIntrinsicSizeLonghand",
      interpolable: true,
      field_group: "surround",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/style/style_intrinsic_length.h"],
      keywords: ["none"],
      default_value: "StyleIntrinsicLength()",
      type_name: "StyleIntrinsicLength",
      converter: "ConvertIntrinsicDimension",
      invalidate: ["layout", "scroll-anchor"],
      affected_by_zoom: true,
    },
    {
      name: "container-name",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ConsumeContainerName",
      type_name: "Vector<AtomicString>",
      default_value: "Vector<AtomicString>()",
      field_group: "*",
      field_template: "external",
      converter: "ConvertContainerName",
      keywords: ["none"],
      typedom_types: ["Keyword"],
      typedom_custom_ident: true,
      is_animation_affecting: true,
    },
    {
      name: "container-type",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      keywords: ["normal", "inline-size", "size", "scroll-state", "anchored"],
      field_group: "*",
      field_size: 4,
      field_template: "primitive",
      default_value: "kContainerTypeNormal",
      type_name: "unsigned",
      converter: "ConvertFlags<EContainerType, CSSValueID::kNormal>",
      typedom_types: ["Keyword"],
      invalidate: ["layout"],
      is_animation_affecting: true,
    },
    {
      name: "content",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      field_group: "*",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/style/content_data.h"],
      wrapper_pointer_name: "Member",
      default_value: "nullptr",
      separator: ",",
      type_name: "ContentData",
      computed_style_custom_functions: ["getter"],
      style_builder_custom_functions: ["initial", "inherit", "value"],
      valid_for_marker: true,
      valid_for_page_context: true,
      supports_incremental_style: true,
      keywords: ["none", "normal", "close-quote", "no-close-quote", "no-open-quote", "open-quote"],
    },
    {
      name: "corner-bottom-left-shape",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ConsumeCornerShape",
      interpolable: true,
      field_group: "surround",
      field_template: "external",
      default_value: "Superellipse::Round()",
      include_paths: ["third_party/blink/renderer/core/style/superellipse.h"],
      keywords: ["notch", "scoop", "bevel", "round", "squircle", "square"],
      type_name: "Superellipse",
      converter: "ConvertCornerShape",
      valid_for_first_letter: true,
      logical_property_group: {
        name: "corner-shape",
        resolver: "bottom-left",
      },
      valid_for_page_context: true,
      invalidate: ["border-radius", "paint"],
    },
    {
      name: "corner-bottom-right-shape",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ConsumeCornerShape",
      interpolable: true,
      field_group: "surround",
      field_template: "external",
      default_value: "Superellipse::Round()",
      include_paths: ["third_party/blink/renderer/core/style/superellipse.h"],
      keywords: ["notch", "scoop", "bevel", "round", "squircle", "square"],
      type_name: "Superellipse",
      converter: "ConvertCornerShape",
      valid_for_first_letter: true,
      logical_property_group: {
        name: "corner-shape",
        resolver: "bottom-right",
      },
      valid_for_page_context: true,
      invalidate: ["border-radius", "paint"],
    },
    {
      name: "corner-top-left-shape",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ConsumeCornerShape",
      interpolable: true,
      field_group: "surround",
      field_template: "external",
      default_value: "Superellipse::Round()",
      include_paths: ["third_party/blink/renderer/core/style/superellipse.h"],
      keywords: ["notch", "scoop", "bevel", "round", "squircle", "square"],
      type_name: "Superellipse",
      converter: "ConvertCornerShape",
      valid_for_first_letter: true,
      logical_property_group: {
        name: "corner-shape",
        resolver: "top-left",
      },
      valid_for_page_context: true,
      invalidate: ["border-radius", "paint"],
    },
    {
      name: "corner-top-right-shape",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ConsumeCornerShape",
      interpolable: true,
      field_group: "surround",
      field_template: "external",
      default_value: "Superellipse::Round()",
      include_paths: ["third_party/blink/renderer/core/style/superellipse.h"],
      keywords: ["notch", "scoop", "bevel", "round", "squircle", "square"],
      type_name: "Superellipse",
      converter: "ConvertCornerShape",
      valid_for_first_letter: true,
      logical_property_group: {
        name: "corner-shape",
        resolver: "top-right",
      },
      valid_for_page_context: true,
      invalidate: ["border-radius", "paint"],
    },
    {
      name: "counter-increment",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      style_builder_template: "counter",
      style_builder_template_args: {
        action: "Increment",
      },
      keywords: ["none"],
      typedom_types: ["Keyword"],
      valid_for_permission_element: true,
      valid_for_page_context: true,
      stored_on_extra_field: ["CounterDirectives", "CounterIncrementList"],
    },
    {
      name: "counter-reset",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      style_builder_template: "counter",
      style_builder_template_args: {
        action: "Reset",
      },
      keywords: ["none"],
      typedom_types: ["Keyword"],
      valid_for_permission_element: true,
      valid_for_page_context: true,
      stored_on_extra_field: ["CounterDirectives", "CounterResetList"],
    },
    {
      name: "counter-set",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      style_builder_template: "counter",
      style_builder_template_args: {
        action: "Set",
      },
      keywords: ["none"],
      typedom_types: ["Keyword"],
      valid_for_permission_element: true,
      valid_for_page_context: true,
      stored_on_extra_field: ["CounterDirectives", "CounterSetList"],
    },
    {
      name: "cursor",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      field_group: "inherited",
      inherited: true,
      independent: true,
      field_template: "keyword_custom",
      keywords: [
        "auto", "default", "none", "context-menu", "help", "pointer",
        "progress", "wait", "cell", "crosshair", "text", "vertical-text",
        "alias", "copy", "move", "no-drop", "not-allowed", "e-resize",
        "n-resize", "ne-resize", "nw-resize", "s-resize", "se-resize",
        "sw-resize", "w-resize", "ew-resize", "ns-resize", "nesw-resize",
        "nwse-resize", "col-resize", "row-resize", "all-scroll", "zoom-in",
        "zoom-out", "grab", "grabbing"
      ],
      default_value: "auto",
      style_builder_custom_functions: ["initial", "inherit", "value"],
      typedom_types: ["Keyword"],
      valid_for_marker: true,
      valid_for_permission_element: true,
    },
    {
      name: "cx",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      field_group: "svg->geometry",
      field_template: "<length>",
      default_value: "Length::Fixed()",
      typedom_types: ["Length", "Percentage"],
      converter: "ConvertLength",
      supports_incremental_style: true,
      invalidate: ["layout", "paint"],
      percentages_depend_on_used_value: true,
      affected_by_zoom: true,
    },
    {
      name: "cy",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      field_group: "svg->geometry",
      field_template: "<length>",
      default_value: "Length::Fixed()",
      typedom_types: ["Length", "Percentage"],
      converter: "ConvertLength",
      supports_incremental_style: true,
      invalidate: ["layout", "paint"],
      percentages_depend_on_used_value: true,
      affected_by_zoom: true,
    },
    {
      name: "d",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      field_group: "svg->geometry",
      field_template: "pointer",
      include_paths: ["third_party/blink/renderer/core/style/style_path.h"],
      wrapper_pointer_name: "Member",
      type_name: "StylePath",
      default_value: "nullptr",
      converter: "ConvertPathOrNone",
      keywords: ["none"],
      typedom_types: ["Keyword"],
      supports_incremental_style: true,
      invalidate: ["layout", "paint"],
    },
    {
      name: "display",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      keywords: [
        "inline", "block", "list-item", "inline-block", "table", "inline-table",
        "table-row-group", "table-header-group", "table-footer-group",
        "table-row", "table-column-group", "table-column", "table-cell",
        "table-caption", "-webkit-box", "-webkit-inline-box", "flex",
        "inline-flex", "grid", "inline-grid", "contents", "flow-root", "none",
        "flow", "math", "ruby", "ruby-text", "grid-lanes", "inline-grid-lanes"
      ],
      // `grid-lanes` and `inline-grid-lanes` are experimental keywords and
      // currently behind the CSSGridLanesLayout flag. Since they are not
      // fully supported yet, they are not being exposed to devtools.
      devtools_keywords: [
        "inline", "block", "list-item", "inline-block", "table", "inline-table",
        "table-row-group", "table-header-group", "table-footer-group",
        "table-row", "table-column-group", "table-column", "table-cell",
        "table-caption", "-webkit-box", "-webkit-inline-box", "flex",
        "inline-flex", "grid", "inline-grid", "contents", "flow-root", "none",
        "flow", "math", "ruby", "ruby-text"
      ],
      typedom_types: ["Keyword"],
      style_builder_custom_functions: ["initial", "inherit", "value"],
      // In general many things are tweaked after-the-fact based on display/float/position
      // (e.g. OriginalDisplay is based on display, and setting float can cause blockification),
      // so we turn off incremental style for all them all.
      supports_incremental_style: false,
      valid_for_permission_element: true,
      valid_for_permission_icon: true,
      invalidate: ["layout", "paint"],
      stored_on_extra_field: ["Display"],
    },
    {
      name: "dominant-baseline",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      inherited: true,
      field_group: "svginherited",
      field_template: "keyword",
      keywords: ["auto", "alphabetic", "ideographic", "middle", "central", "mathematical", "hanging",
                 "use-script", "no-change", "reset-size", "text-after-edge", "text-before-edge"],
      default_value: "auto",
      typedom_types: ["Keyword"],
      invalidate: ["layout", "paint"],
    },
    {
      name: "empty-cells",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      independent: true,
      inherited: true,
      field_group: "*",
      field_template: "keyword",
      keywords: ["show", "hide"],
      typedom_types: ["Keyword"],
      default_value: "show",
      invalidate: ["layout", "paint"],
    },
    {
      name: "fill",
      property_methods: ["CSSValueFromComputedStyleInternal", "ColorIncludingFallback"],
      parse_helper: "ConsumeSVGPaint",
      interpolable: true,
      inherited: true,
      includes_currentcolor: true,
      field_group: "svginherited->fill",
      field_template: "external",
      type_name: "SVGPaint",
      include_paths: ["third_party/blink/renderer/core/style/svg_paint.h"],
      default_value: "SVGPaint::CreateInitialBlack()",
      name_for_methods: "FillPaint",
      converter: "ConvertSVGPaint",
      style_builder_template: "color",
      style_builder_template_args: {
        initial_color: "ComputedStyleInitialValues::InitialFillPaint",
      },
      style_builder_custom_functions: ["value"],
      valid_for_highlight: true,
      valid_for_permission_icon: true,
      invalidate: ["paint"],
      keywords: ["none"],
    },
    {
      name: "fill-opacity",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ConsumeAlphaValue",
      interpolable: true,
      inherited: true,
      field_group: "svginherited->fill",
      field_template: "primitive",
      type_name: "float",
      default_value: "1",
      converter: "ConvertAlpha",
      typedom_types: ["Number", "Percentage"],
      accepts_numeric_literal: true,
      invalidate: ["paint"],
      percentages_depend_on_used_value: false,
    },
    {
      name: "fill-rule",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      inherited: true,
      field_group: "svginherited",
      // TODO(fs): Convert this to a keyword (requires enum massage).
      field_template: "primitive",
      field_size: 1,
      include_paths: ["third_party/blink/renderer/platform/geometry/path_types.h"],
      type_name: "WindRule",
      keywords: ["nonzero", "evenodd"],
      default_value: "RULE_NONZERO",
      typedom_types: ["Keyword"],
      invalidate: ["paint"],
    },
    {
      name: "filter",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ConsumeFilterFunctionList",
      interpolable: true,
      compositable: true,
      tracks_animated_source: true,
      field_group: "*",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/style/filter_operations.h"],
      default_value: "FilterOperations()",
      type_name: "FilterOperations",
      computed_style_custom_functions: ["initial"],
      style_builder_custom_functions: ["value"],
      keywords: ["none"],
      typedom_types: ["Keyword"],
      invalidate: ["filter-data"],
      includes_currentcolor: true,
      affected_by_zoom: true,
    },
    {
      name: "flex-basis",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      field_group: "*",
      field_template: "<length>",
      default_value: "Length::Auto()",
      converter: "ConvertLengthSizing",
      typedom_types: ["Keyword", "Length", "Percentage"],
      keywords: ["auto", "fit-content", "min-content", "max-content", "content"],
      valid_for_permission_element: true,
      invalidate: ["layout", "paint"],
      percentages_depend_on_used_value: true,
      affected_by_zoom: true,
    },
    {
      name: "flex-direction",
      property_methods: ["CSSValueFromComputedStyleInternal", "InitialValue"],
      field_group: "*",
      field_template: "keyword",
      typedom_types: ["Keyword"],
      keywords: ["row", "row-reverse", "column", "column-reverse"],
      default_value: "row",
      valid_for_permission_element: true,
      invalidate: ["layout", "paint"],
    },
    {
      name: "flex-grow",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      field_group: "*",
      field_template: "primitive",
      default_value: "0.0f",
      type_name: "float",
      typedom_types: ["Number"],
      valid_for_permission_element: true,
      invalidate: ["layout", "paint"],
    },
    {
      name: "flex-shrink",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      field_group: "*",
      field_template: "primitive",
      default_value: "1.0f",
      type_name: "float",
      typedom_types: ["Number"],
      valid_for_permission_element: true,
      invalidate: ["layout", "paint"],
    },
    {
      name: "flex-wrap",
      property_methods: ["InitialValue", "ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      field_group: "*",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/style/style_flex_wrap_data.h"],
      default_value: "StyleFlexWrapData(FlexWrapMode::kNowrap)",
      type_name: "StyleFlexWrapData",
      converter: "ConvertFlexWrapData",
      typedom_types: ["Keyword"],
      keywords: ["nowrap", "wrap", "wrap-reverse", "balance"],
      valid_for_permission_element: true,
      invalidate: ["layout", "paint"],
    },
    {
      name: "flex-line-count",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      field_group: "*",
      field_template: "primitive",
      default_value: "1u",
      type_name: "uint16_t",
      typedom_types: ["Number"],
      valid_for_permission_element: true,
      invalidate: ["layout", "paint"],
      runtime_flag: "FlexWrapBalance",
    },
    {
      name: "float",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      field_template: "keyword",
      computed_style_protected_functions: ["getter"],
      keywords: ["none", "left", "right", "inline-start", "inline-end"],
      typedom_types: ["Keyword"],
      default_value: "none",
      name_for_methods: "Floating",
      type_name: "EFloat",
      valid_for_first_letter: true,
      // See comment on display.
      supports_incremental_style: false,
      valid_for_permission_element: true,
      invalidate: ["layout", "paint"],
    },
    {
      name: "flood-color",
      property_methods: ["CSSValueFromComputedStyleInternal", "ColorIncludingFallback"],
      parse_helper: "ConsumeColor",
      interpolable: true,
      field_group: "svg->svgmisc",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/css/style_color.h"],
      default_value: "StyleColor(Color::kBlack)",
      type_name: "StyleColor",
      style_builder_template: "color",
      style_builder_template_args: {
        initial_color: "ComputedStyleInitialValues::InitialFloodColor",
      },
      converter: "ConvertStyleColor",
      keywords: ["currentcolor"],
      includes_currentcolor: true,
      typedom_types: ["Keyword"],
      supports_incremental_style: true,
      invalidate: ["paint"],
    },
    {
      name: "flood-opacity",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ConsumeAlphaValue",
      interpolable: true,
      field_group: "svg->svgmisc",
      field_template: "primitive",
      type_name: "float",
      default_value: "1",
      converter: "ConvertAlpha",
      typedom_types: ["Number", "Percentage"],
      supports_incremental_style: true,
      accepts_numeric_literal: true,
      invalidate: ["paint"],
      percentages_depend_on_used_value: false,
    },
    {
      name: "flow-tolerance",
      include_paths: ["third_party/blink/renderer/core/style/flow_tolerance.h"],
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ConsumeFlowTolerance",
      field_group: "*",
      field_template: "external",
      default_value: "FlowTolerance(CSSValueID::kNormal)",
      type_name: "FlowTolerance",
      converter: "ConvertFlowTolerance",
      typedom_types: ["Length", "Percentage", "Keyword"],
      keywords: ["normal", "infinite"],
      invalidate: ["layout", "paint"],
      runtime_flag: "CSSGridLanesLayout",
      interpolable: true,
      percentages_depend_on_used_value: true,
      affected_by_zoom: true,
    },
    {
      name: "frame-sizing",
      field_group: "surround",
      field_template: "keyword",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      keywords: ["auto", "content-width", "content-height", "content-block-size", "content-inline-size"],
      default_value: "auto",
      typedom_types: ["Keyword"],
      invalidate: ["layout", "scroll-anchor"],
      runtime_flag: "ResponsiveIframes",
    },
    {
      name: "grid-auto-columns",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal", "InitialValue"],
      field_group: "*",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/style/grid_track_list.h"],
      default_value: "GridTrackList(GridTrackSize(Length::Auto()))",
      type_name: "GridTrackList",
      converter: "ConvertGridTrackSizeList",
      keywords: ["auto", "min-content", "max-content"],
      typedom_types: ["Keyword", "Length", "Percentage", "Flex"],
      separator: " ",
      invalidate: ["layout", "paint"],
      percentages_depend_on_used_value: true,
      affected_by_zoom: true,
    },
    {
      name: "grid-auto-flow",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal", "InitialValue"],
      field_group: "*",
      field_size: 4, // FIXME: Make this use "kGridAutoFlowBits".
      field_template: "primitive",
      default_value: "kAutoFlowRow",
      type_name: "GridAutoFlow",
      computed_style_custom_functions: ["getter"],
      converter: "ConvertGridAutoFlow",
      keywords: ["row", "column"],
      typedom_types: ["Keyword"],
      invalidate: ["layout", "paint"],
    },
    {
      name: "grid-auto-rows",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal", "InitialValue"],
      field_group: "*",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/style/grid_track_list.h"],
      default_value: "GridTrackList(GridTrackSize(Length::Auto()))",
      type_name: "GridTrackList",
      converter: "ConvertGridTrackSizeList",
      keywords: ["auto", "min-content", "max-content"],
      typedom_types: ["Keyword", "Length", "Percentage", "Flex"],
      separator: " ",
      invalidate: ["layout", "paint"],
      percentages_depend_on_used_value: true,
      affected_by_zoom: true,
    },
    {
      name: "grid-column-end",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ConsumeGridLine",
      field_group: "*",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/style/grid_position.h"],
      default_value: "GridPosition()",
      type_name: "GridPosition",
      keywords: ["auto"],
      typedom_types: ["Keyword"],
      converter: "ConvertGridPosition",
      invalidate: ["layout", "paint"],
    },
    {
      name: "grid-column-start",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ConsumeGridLine",
      field_group: "*",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/style/grid_position.h"],
      default_value: "GridPosition()",
      type_name: "GridPosition",
      keywords: ["auto"],
      typedom_types: ["Keyword"],
      converter: "ConvertGridPosition",
      invalidate: ["layout", "paint"],
    },
    {
      name: "grid-lanes-direction",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      field_group: "*",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/layout/grid_lanes/grid_lanes_direction.h"],
      default_value: "GridLanesDirection()",
      type_name: "GridLanesDirection",
      converter: "ConvertGridLanesDirection",
      getter: "GetGridLanesDirection",
      typedom_types: ["Keyword"],
      keywords: ["normal", "row", "column", "fill-reverse", "track-reverse"],
      typedom_keywords: ["normal"],
      invalidate: ["layout", "paint"],
      runtime_flag: "CSSGridLanesLayout",
    },
    {
      name: "grid-lanes-pack",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      field_group: "*",
      field_template: "keyword",
      typedom_types: ["Keyword"],
      keywords: ["normal", "dense"],
      default_value: "normal",
      invalidate: ["layout", "paint"],
      runtime_flag: "CSSGridLanesLayout",
    },
    {
      name: "grid-row-end",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ConsumeGridLine",
      field_group: "*",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/style/grid_position.h"],
      default_value: "GridPosition()",
      type_name: "GridPosition",
      keywords: ["auto"],
      typedom_types: ["Keyword"],
      converter: "ConvertGridPosition",
      invalidate: ["layout", "paint"],
    },
    {
      name: "grid-row-start",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ConsumeGridLine",
      field_group: "*",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/style/grid_position.h"],
      default_value: "GridPosition()",
      type_name: "GridPosition",
      keywords: ["auto"],
      typedom_types: ["Keyword"],
      converter: "ConvertGridPosition",
      invalidate: ["layout", "paint"],
    },
    {
      name: "grid-template-areas",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal", "InitialValue"],
      field_group: "*",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/style/computed_grid_template_areas.h"],
      type_name: "ComputedGridTemplateAreas",
      wrapper_pointer_name: "Member",
      default_value: "nullptr",
      keywords: ["none"],
      typedom_types: ["Keyword"],
      converter: "ConvertGridTemplateAreas",
      invalidate: ["layout", "paint"],
    },
    {
      name: "grid-template-columns",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ConsumeGridTemplatesRowsOrColumns",
      layout_dependent: true,
      field_group: "*",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/style/computed_grid_track_list.h"],
      interpolable: true,
      default_value: "nullptr",
      wrapper_pointer_name: "Member",
      type_name: "ComputedGridTrackList",
      converter: "ConvertGridTrackList",
      getter: "SpecifiedGridTemplateColumns",
      style_builder_custom_functions: ["inherit"],
      typedom_keywords: ["none"],
      keywords: ["auto", "none", "min-content", "max-content"],
      typedom_types: ["Keyword"],
      invalidate: ["layout", "paint"],
      percentages_depend_on_used_value: true,
      affected_by_zoom: true,
    },
    {
      name: "hanging-punctuation",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      inherited: true,
      field_group: "*",
      field_template: "multi_keyword",
      keywords: ["none", "first", "last", "allow-end"],
      include_paths:["third_party/blink/renderer/platform/text/hanging_punctuation.h"],
      typedom_types: ["Keyword"],
      default_value: "none",
      type_name: "HangingPunctuation",
      getter: "GetHangingPunctuation",
      converter: "ConvertFlags<blink::HangingPunctuation>",
      runtime_flag: "CSSHangingPunctuation",
      invalidate: ["layout", "paint"],
    },
    {
      name: "grid-template-rows",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ConsumeGridTemplatesRowsOrColumns",
      layout_dependent: true,
      field_group: "*",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/style/computed_grid_track_list.h"],
      interpolable: true,
      default_value: "nullptr",
      wrapper_pointer_name: "Member",
      type_name: "ComputedGridTrackList",
      converter: "ConvertGridTrackList",
      getter: "SpecifiedGridTemplateRows",
      style_builder_custom_functions: ["inherit"],
      typedom_keywords: ["none"],
      keywords: ["auto", "none", "min-content", "max-content"],
      typedom_types: ["Keyword"],
      invalidate: ["layout", "paint"],
      percentages_depend_on_used_value: true,
      affected_by_zoom: true,
    },
    {
      name: "height",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      layout_dependent: true,
      field_group: "box",
      field_template: "<length>",
      keywords: ["auto", "fit-content", "min-content", "max-content"],
      default_value: "Length()",
      typedom_types: ["Keyword", "Length", "Percentage"],
      converter: "ConvertLengthSizing",
      anchor_mode: "height",
      logical_property_group: {
        name: "size",
        resolver: "vertical",
      },
      supports_incremental_style: true,
      valid_for_position_try: true,
      valid_for_permission_element: true,
      valid_for_permission_icon: true,
      valid_for_page_context: true,
      invalidate: ["layout", "scroll-anchor"],
      affected_by_zoom: true,
      percentages_depend_on_used_value: true,
    },
    {
      name: "hyphenate-limit-chars",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ConsumeHyphenateLimitChars",
      inherited: true,
      field_group: "*",
      field_template: "external",
      keywords: ["auto"],
      type_name: "StyleHyphenateLimitChars",
      default_value: "StyleHyphenateLimitChars()",
      include_paths: ["third_party/blink/renderer/core/style/style_hyphenate_limit_chars.h"],
      converter: "ConvertHyphenateLimitChars",
      invalidate: ["layout", "paint"],
    },
    {
      name: "interest-delay",
      longhands: [
        "interest-delay-start", "interest-delay-end"
      ],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      supports_incremental_style: true,
    },
    {
      name: "interest-delay-start",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      field_group: "*",
      field_template: "external",
      type_name: "StyleInterestDelay",
      default_value: "StyleInterestDelay()",
      interpolable: true,
      include_paths: ["third_party/blink/renderer/core/style/style_interest_delay.h"],
      converter: "ConvertInterestDelayValue",
      supports_incremental_style: true,
    },
    {
      name: "interest-delay-end",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      field_group: "*",
      field_template: "external",
      type_name: "StyleInterestDelay",
      default_value: "StyleInterestDelay()",
      interpolable: true,
      include_paths: ["third_party/blink/renderer/core/style/style_interest_delay.h"],
      converter: "ConvertInterestDelayValue",
      supports_incremental_style: true,
    },
    {
      name: "hyphens",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      inherited: true,
      field_group: "*",
      field_template: "keyword",
      keywords: ["none", "manual", "auto"],
      default_value: "manual",
      type_name: "Hyphens",
      typedom_types: ["Keyword"],
      valid_for_marker: true,
      invalidate: ["layout", "paint"],
    },
    {
      name: "image-animation",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      inherited: true,
      field_group: "*",
      field_template: "keyword",
      include_paths: [
        "third_party/blink/renderer/platform/graphics/image_node_animation_info.h"
      ],
      type_name: "ImageAnimationEnum",
      keywords: ["normal", "running", "paused", "stopped"],
      default_value: "normal",
      typedom_types: ["Keyword"],
      invalidate: ["paint"],
      runtime_flag: "CSSImageAnimation",
    },
    {
      name: "image-rendering",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      inherited: true,
      field_group: "*",
      field_template: "keyword",
      keywords: [
        "auto", "-webkit-optimize-contrast", "pixelated", "crisp-edges"
      ],
      typedom_types: ["Keyword"],
      default_value: "auto",
      invalidate: ["paint"],
    },
    {
      name: "image-orientation",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      inherited: true,
      field_group: "*",
      field_template: "primitive",
      field_size: 1,
      type_name: "RespectImageOrientationEnum",
      default_value: "kRespectImageOrientation",
      converter: "ConvertImageOrientation",
      include_paths: [
        "third_party/blink/renderer/platform/graphics/image_orientation.h"
      ],
      invalidate: ["layout", "paint"],
      keywords: ["none", "from-image"],
    },
    {
      name: "dynamic-range-limit",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      inherited: true,
      runtime_flag: "CSSDynamicRangeLimit",
      converter: "ConvertDynamicRangeLimit",
      type_name: "DynamicRangeLimit",
      field_group: "*",
      field_template: "external",
      keywords: ["standard", "no-limit", "constrained"],
      typedom_types: ["Keyword"],
      include_paths: ["third_party/blink/renderer/platform/graphics/graphics_context_types.h"],
      default_value: "DynamicRangeLimit(cc::PaintFlags::DynamicRangeLimit::kHigh)",
      invalidate: ["paint"],
    },
    {
      name: "initial-letter",
      converter: "ConvertInitialLetter",
      default_value: "StyleInitialLetter()",
      field_group: "*",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/style/style_initial_letter.h"],
      inherited: false,
      keywords: ["drop", "normal", "raise"],
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ConsumeInitialLetter",
      type_name: "StyleInitialLetter",
      valid_for_first_letter: true,
      invalidate: ["reshape", "layout", "paint"],
    },
    {
      name: "interactivity",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      inherited: true,
      independent: true,
      field_group: "*",
      field_template: "keyword",
      keywords: ["auto", "inert"],
      typedom_types: ["Keyword"],
      default_value: "auto",
      invalidate: [],
    },
    {
      name: "interpolate-size",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      inherited: true,
      field_group: "*",
      field_template: "keyword",
      keywords: ["numeric-only", "allow-keywords"],
      typedom_types: ["Keyword"],
      default_value: "numeric-only",
      invalidate: [],
      is_animation_affecting: true,
    },
    {
      name: "isolation",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      field_group: "*",
      field_template: "keyword",
      keywords: ["auto", "isolate"],
      typedom_types: ["Keyword"],
      default_value: "auto",
      valid_for_permission_element: true,
      invalidate: ["paint"],
    },
    {
      name: "justify-content",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      field_group: "box",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/style/style_content_alignment_data.h"],
      default_value: "StyleContentAlignmentData(ContentPosition::kNormal, ContentDistributionType::kDefault, OverflowAlignment::kDefault)",
      type_name: "StyleContentAlignmentData",
      converter: "ConvertContentAlignmentData",
      invalidate: ["layout"],
    },
    {
      name: "justify-items",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      field_group: "*",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/style/style_self_alignment_data.h"],
      default_value: "StyleSelfAlignmentData(ItemPosition::kLegacy, OverflowAlignment::kDefault)",
      type_name: "StyleSelfAlignmentData",
      converter: "ConvertSelfOrDefaultAlignmentData",
      invalidate: ["layout"],
    },
    {
      name: "justify-self",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      field_group: "*",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/style/style_self_alignment_data.h"],
      default_value: "StyleSelfAlignmentData(ItemPosition::kAuto, OverflowAlignment::kDefault)",
      type_name: "StyleSelfAlignmentData",
      converter: "ConvertSelfOrDefaultAlignmentData",
      valid_for_position_try: true,
      valid_for_permission_element: true,
      invalidate: ["layout"],
      keywords: ["auto", "normal", "stretch", "baseline", "center", "start",
        "end", "flow-start", "flow-end", "self-start", "self-end", "flex-start",
        "flex-end", "left", "right", "anchor-center"],
    },
    {
      name: "left",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      layout_dependent: true,
      field_group: "surround",
      field_template: "<length>",
      keywords: ["auto"],
      default_value: "Length()",
      typedom_types: ["Keyword", "Length", "Percentage"],
      converter: "ConvertLengthOrAuto",
      anchor_mode: "left",
      logical_property_group: {
        name: "inset",
        resolver: "left",
      },
      supports_incremental_style: true,
      valid_for_position_try: true,
      valid_for_permission_element: true,
      invalidate: ["inset", "scroll-anchor"],
      affected_by_zoom: true,
      percentages_depend_on_used_value: true,
    },
    {
      name: "letter-spacing",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ParseSpacing",
      interpolable: true,
      inherited: true,
      keywords: ["normal"],
      field_group: "inherited",
      field_template: "<length>",
      default_value: "Length::Fixed()",
      getter: "ComputedLetterSpacing",
      converter: "ConvertSpacing",
      computed_style_custom_functions: ["getter"],
      typedom_types: ["Keyword", "Length", "Percentage"],
      valid_for_first_letter: true,
      valid_for_first_line: true,
      valid_for_marker: true,
      valid_for_permission_element: true,
      valid_for_page_context: true,
      affected_by_zoom: true,
      // The field isn't actually used; this is currently stored in FontDescription.
      stored_on_extra_field: ["font"],
      percentages_depend_on_used_value: false,
    },
    {
      name: "lighting-color",
      property_methods: ["CSSValueFromComputedStyleInternal", "ColorIncludingFallback"],
      parse_helper: "ConsumeColor",
      interpolable: true,
      field_group: "svg->svgmisc",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/css/style_color.h"],
      default_value: "StyleColor(Color::kWhite)",
      type_name: "StyleColor",
      style_builder_template: "color",
      style_builder_template_args: {
        initial_color: "ComputedStyleInitialValues::InitialLightingColor",
      },
      converter: "ConvertStyleColor",
      keywords: ["currentcolor"],
      includes_currentcolor: true,
      typedom_types: ["Keyword"],
      invalidate: ["paint"],
    },
    {
      name: "line-height",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ConsumeLineHeight",
      interpolable: true,
      inherited: true,
      field_group: "inherited",
      field_template: "<length>",
      default_value: "Length::Auto()",
      converter: "ConvertLineHeight",
      keywords: ["normal"],
      typedom_types: ["Keyword", "Length", "Number", "Percentage"],
      valid_for_first_letter: true,
      valid_for_first_line: true,
      valid_for_cue: true,
      valid_for_marker: true,
      valid_for_page_context: true,
      affected_by_zoom: true,
      invalidate: ["layout", "paint"],
      // The font and line height are necessary to correctly resolve font relative
      // units.
      highlight_style_comes_from_originating_element: true,
      percentages_depend_on_used_value: false,
    },
    {
      name: "link-parameters",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      field_group: "*",
      field_template: "pointer",
      include_paths: ["third_party/blink/renderer/core/style/link_parameter_list.h"],
      wrapper_pointer_name: "Member",
      default_value: "nullptr",
      type_name: "LinkParameterList",
      converter: "ConvertLinkParameters",
      keywords: ["none"],
      runtime_flag: "CSSLinkParametersProperty",
      invalidate: ["paint"],
    },
    {
      name: "list-style-image",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ConsumeImageOrNone",
      interpolable: true,
      inherited: true,
      field_group: "*",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/style/style_image.h"],
      wrapper_pointer_name: "Member",
      default_value: "nullptr",
      typedom_types: ["Keyword", "Image"],
      type_name: "StyleImage",
      style_builder_custom_functions: ["value"],
      keywords: ["none"],
      invalidate: ["layout", "paint"],
      affected_by_zoom: true,
      includes_currentcolor: true,
    },
    {
      name: "list-style-position",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      independent: true,
      inherited: true,
      field_template: "keyword",
      keywords: ["outside", "inside"],
      typedom_types: ["Keyword"],
      default_value: "outside",
      invalidate: ["layout", "paint"],
    },
    {
      name: "list-style-type",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      inherited: true,
      field_group: "*",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/keywords.h",
                      "third_party/blink/renderer/core/style/list_style_type_data.h"],
      wrapper_pointer_name: "Member",
      default_value: "ListStyleTypeData::CreateCounterStyle(keywords::kDisc, nullptr)",
      type_name: "ListStyleTypeData",
      keywords: [
        "disc", "circle", "square", "disclosure-open", "disclosure-closed",
        "decimal", "none"
      ],
      style_builder_custom_functions: ["value"],
      invalidate: ["layout", "paint"],
    },
    {
      name: "margin-bottom",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      layout_dependent: true,
      field_group: "box",
      field_template: "<length>",
      default_value: "Length::Fixed()",
      converter: "ConvertQuirkyLength",
      computed_style_custom_functions: ["setter"],
      keywords: ["auto"],
      typedom_types: ["Keyword", "Length", "Percentage"],
      valid_for_first_letter: true,
      valid_for_position_try: true,
      logical_property_group: {
        name: "margin",
        resolver: "bottom",
      },
      supports_incremental_style: true,
      valid_for_permission_element: true,
      valid_for_page_context: true,
      invalidate: ["layout", "scroll-anchor"],
      anchor_mode: "height",
      percentages_depend_on_used_value: true,
      affected_by_zoom: true,
    },
    {
      name: "margin-left",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      layout_dependent: true,
      field_group: "box",
      field_template: "<length>",
      default_value: "Length::Fixed()",
      converter: "ConvertQuirkyLength",
      computed_style_custom_functions: ["setter"],
      keywords: ["auto"],
      typedom_types: ["Keyword", "Length", "Percentage"],
      valid_for_first_letter: true,
      valid_for_position_try: true,
      logical_property_group: {
        name: "margin",
        resolver: "left",
      },
      supports_incremental_style: true,
      valid_for_permission_element: true,
      valid_for_permission_icon: true,
      valid_for_page_context: true,
      invalidate: ["layout", "scroll-anchor"],
      anchor_mode: "width",
      percentages_depend_on_used_value: true,
      affected_by_zoom: true,
    },
    {
      name: "margin-right",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      layout_dependent: true,
      field_group: "box",
      field_template: "<length>",
      default_value: "Length::Fixed()",
      converter: "ConvertQuirkyLength",
      computed_style_custom_functions: ["setter"],
      keywords: ["auto"],
      typedom_types: ["Keyword", "Length", "Percentage"],
      valid_for_first_letter: true,
      valid_for_position_try: true,
      logical_property_group: {
        name: "margin",
        resolver: "right",
      },
      supports_incremental_style: true,
      valid_for_permission_element: true,
      valid_for_permission_icon: true,
      valid_for_page_context: true,
      invalidate: ["layout", "scroll-anchor"],
      anchor_mode: "width",
      percentages_depend_on_used_value: true,
      affected_by_zoom: true,
    },
    {
      name: "margin-top",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      layout_dependent: true,
      field_group: "box",
      field_template: "<length>",
      default_value: "Length::Fixed()",
      converter: "ConvertQuirkyLength",
      computed_style_custom_functions: ["setter"],
      keywords: ["auto"],
      typedom_types: ["Keyword", "Length", "Percentage"],
      valid_for_first_letter: true,
      valid_for_position_try: true,
      logical_property_group: {
        name: "margin",
        resolver: "top",
      },
      supports_incremental_style: true,
      valid_for_permission_element: true,
      valid_for_page_context: true,
      invalidate: ["layout", "scroll-anchor"],
      anchor_mode: "height",
      percentages_depend_on_used_value: true,
      affected_by_zoom: true,
    },
    {
      name: "margin-trim",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      field_group: "*",
      field_size: 4,
      field_template: "primitive",
      name_for_methods: "MarginTrim",
      type_name: "unsigned",
      converter: "ConvertFlags<EMarginTrim>",
      default_value: "kMarginTrimNone",
      typedom_types: ["Keyword"],
      runtime_flag: "MarginTrim",
      invalidate: ["layout"],
    },
    {
      name: "marker-end",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      inherited: true,
      field_group: "svginherited->resources",
      field_template: "pointer",
      type_name: "StyleSVGResource",
      include_paths: ["third_party/blink/renderer/core/style/style_svg_resource.h"],
      wrapper_pointer_name: "Member",
      default_value: "nullptr",
      name_for_methods: "MarkerEndResource",
      style_builder_custom_functions: ["value"],
      keywords: ["none"],
      typedom_types: ["Keyword"],
      invalidate: ["layout", "paint"],
    },
    {
      name: "marker-mid",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      inherited: true,
      field_group: "svginherited->resources",
      field_template: "pointer",
      type_name: "StyleSVGResource",
      include_paths: ["third_party/blink/renderer/core/style/style_svg_resource.h"],
      wrapper_pointer_name: "Member",
      default_value: "nullptr",
      name_for_methods: "MarkerMidResource",
      style_builder_custom_functions: ["value"],
      keywords: ["none"],
      typedom_types: ["Keyword"],
      invalidate: ["layout", "paint"],
    },
    {
      name: "marker-start",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      inherited: true,
      field_group: "svginherited->resources",
      field_template: "pointer",
      type_name: "StyleSVGResource",
      include_paths: ["third_party/blink/renderer/core/style/style_svg_resource.h"],
      wrapper_pointer_name: "Member",
      default_value: "nullptr",
      name_for_methods: "MarkerStartResource",
      style_builder_custom_functions: ["value"],
      keywords: ["none"],
      typedom_types: ["Keyword"],
      invalidate: ["layout", "paint"],
    },
    {
      name: "mask-type",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      field_group: "svg",
      field_template: "keyword",
      keywords: ["luminance", "alpha"],
      typedom_types: ["Keyword"],
      default_value: "luminance",
      invalidate: ["paint"],
    },
    {
      name: "math-shift",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      field_group: "*",
      field_template: "keyword",
      inherited: true,
      keywords: ["normal", "compact"],
      typedom_types: ["Keyword"],
      default_value: "normal",
    },
    {
      name: "math-style",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      field_group: "*",
      field_template: "keyword",
      inherited: true,
      keywords: ["normal", "compact"],
      typedom_types: ["Keyword"],
      default_value: "normal",
    },
    {
      name: "max-content-sizing",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      field_group: "*",
      field_template: "keyword",
      keywords: ["auto", "shrink-to-fit"],
      default_value: "auto",
      typedom_types: ["Keyword"],
      invalidate: ["layout"],
      runtime_flag: "CssMaxContentSizing",
      style_builder_custom_functions: ["initial", "inherit", "value"],
    },
    {
      name: "max-height",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      field_group: "box",
      field_template: "<length>",
      default_value: "Length::None()",
      converter: "ConvertLengthMaxSizing",
      anchor_mode: "height",
      keywords: ["none"],
      typedom_types: ["Keyword", "Length", "Percentage"],
      logical_property_group: {
        name: "max-size",
        resolver: "vertical",
      },
      supports_incremental_style: true,
      valid_for_position_try: true,
      valid_for_permission_element: true,
      valid_for_permission_icon: true,
      valid_for_page_context: true,
      invalidate: ["layout", "scroll-anchor"],
      affected_by_zoom: true,
      percentages_depend_on_used_value: true,
    },
    {
      name: "max-width",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      field_group: "box",
      field_template: "<length>",
      default_value: "Length::None()",
      converter: "ConvertLengthMaxSizing",
      anchor_mode: "width",
      keywords: ["none"],
      typedom_types: ["Keyword", "Length", "Percentage"],
      logical_property_group: {
        name: "max-size",
        resolver: "horizontal",
      },
      supports_incremental_style: true,
      valid_for_position_try: true,
      valid_for_permission_element: true,
      valid_for_permission_icon: true,
      valid_for_page_context: true,
      invalidate: ["layout", "scroll-anchor"],
      affected_by_zoom: true,
      percentages_depend_on_used_value: true,
    },
    {
      name: "min-height",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      field_group: "box",
      field_template: "<length>",
      default_value: "Length()",
      converter: "ConvertLengthSizing",
      anchor_mode: "height",
      typedom_types: ["Length", "Percentage"],
      logical_property_group: {
        name: "min-size",
        resolver: "vertical",
      },
      supports_incremental_style: true,
      valid_for_position_try: true,
      valid_for_permission_element: true,
      valid_for_permission_icon: true,
      valid_for_page_context: true,
      invalidate: ["layout", "scroll-anchor"],
      affected_by_zoom: true,
      percentages_depend_on_used_value: true,
      keywords: ["auto", "min-content", "max-content", "fit-content", "stretch"],
    },
    {
      name: "min-width",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      field_group: "box",
      field_template: "<length>",
      default_value: "Length()",
      converter: "ConvertLengthSizing",
      anchor_mode: "width",
      typedom_types: ["Length", "Percentage"],
      logical_property_group: {
        name: "min-size",
        resolver: "horizontal",
      },
      supports_incremental_style: true,
      valid_for_position_try: true,
      valid_for_permission_element: true,
      valid_for_permission_icon: true,
      valid_for_page_context: true,
      invalidate: ["layout", "scroll-anchor"],
      affected_by_zoom: true,
      percentages_depend_on_used_value: true,
      keywords: ["auto", "min-content", "max-content", "fit-content", "stretch"],
    },
    {
      name: "mix-blend-mode",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      field_group: "*",
      field_template: "keyword",
      include_paths: ["third_party/blink/renderer/platform/graphics/blend_mode.h"],
      keywords: [
        "normal", "multiply", "screen", "overlay", "darken", "lighten",
        "color-dodge", "color-burn", "hard-light", "soft-light", "difference",
        "exclusion", "hue", "saturation", "color", "luminosity", "plus-lighter"
      ],
      typedom_types: ["Keyword"],
      default_value: "normal",
      name_for_methods: "BlendMode",
      type_name: "BlendMode",
      invalidate: ["blend-mode", "paint"],
    },
    {
      name: "object-fit",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      field_group: "*",
      field_template: "keyword",
      keywords: ["fill", "contain", "cover", "none", "scale-down"],
      typedom_types: ["Keyword"],
      default_value: "fill",
      getter: "GetObjectFit",
      invalidate: ["paint"],
    },
    {
      name: "object-position",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      field_group: "*",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/platform/geometry/length_point.h"],
      default_value: "LengthPoint(Length::Percent(50.0), Length::Percent(50.0))",
      type_name: "LengthPoint",
      converter: "ConvertPosition",
      typedom_types: ["Keyword", "Position"],
      invalidate: ["paint"],
      percentages_depend_on_used_value: true,
      affected_by_zoom: true,
    },
    {
      name: "object-view-box",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      field_group: "*",
      field_template: "pointer",
      include_paths: ["third_party/blink/renderer/core/style/basic_shapes.h"],
      wrapper_pointer_name: "Member",
      default_value: "nullptr",
      type_name: "BasicShape",
      converter: "ConvertObjectViewBox",
      keywords: ["none"],
      typedom_types: ["Keyword"],
      invalidate: ["layout", "paint"],
      affected_by_zoom: true,
    },
    {
      name: "offset-anchor",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      field_group: "*",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/platform/geometry/length_point.h"],
      default_value: "LengthPoint(Length::Auto(), Length::Auto())",
      type_name: "LengthPoint",
      converter: "ConvertPositionOrAuto",
      keywords: ["auto"],
      typedom_types: ["Keyword", "Position"],
      invalidate: ["transform-data", "transform-other"],
      percentages_depend_on_used_value: true,
      affected_by_zoom: true,
    },
    {
      name: "offset-distance",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      field_group: "*",
      field_template: "<length>",
      default_value: "Length::Fixed(0)",
      converter: "ConvertLength",
      typedom_types: ["Length", "Percentage"],
      invalidate: ["transform-data", "transform-other"],
      percentages_depend_on_used_value: true,
      affected_by_zoom: true,
    },
    {
      name: "offset-path",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ConsumeOffsetPath",
      interpolable: true,
      field_group: "*",
      field_template: "pointer",
      include_paths: ["third_party/blink/renderer/core/style/offset_path_operation.h"],
      wrapper_pointer_name: "Member",
      default_value: "nullptr",
      type_name: "OffsetPathOperation",
      converter: "ConvertOffsetPath",
      keywords: ["none"],
      typedom_types: ["Keyword"],
      invalidate: ["has-transform", "transform-data", "transform-other"],
      affected_by_zoom: true,
    },
    {
      name: "offset-position",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      field_group: "*",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/platform/geometry/length_point.h"],
      default_value: "LengthPoint(Length::None(), Length::None())",
      type_name: "LengthPoint",
      converter: "ConvertOffsetPosition",
      keywords: ["auto", "normal"],
      typedom_types: ["Keyword", "Position"],
      invalidate: ["has-transform", "transform-data", "transform-other"],
      percentages_depend_on_used_value: true,
      affected_by_zoom: true,
    },
    {
      name: "offset-rotate",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ConsumeOffsetRotate",
      interpolable: true,
      field_group: "*",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/style/style_offset_rotation.h"],
      default_value: "StyleOffsetRotation(0, OffsetRotationType::kAuto)",
      type_name: "StyleOffsetRotation",
      converter: "ConvertOffsetRotate",
      keywords: ["auto", "reverse"],
      typedom_types: ["Keyword", "Angle"],
      invalidate: ["transform-data", "transform-other"],
    },
    {
      name: "opacity",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ConsumeAlphaValue",
      interpolable: true,
      compositable: true,
      tracks_animated_source: true,
      field_group: "svg",  // Not SVG, but frequently used with it.
      field_template: "primitive",
      default_value: "1.0",
      type_name: "float",
      computed_style_custom_functions: ["setter"],
      typedom_types: ["Number", "Percentage"],
      valid_for_first_letter: true,
      valid_for_first_line: true,
      valid_for_cue: true,
      supports_incremental_style: true,
      accepts_numeric_literal: true,
      invalidate: ["opacity"],
      percentages_depend_on_used_value: false,
    },
    {
      name: "order",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ConsumeInteger",
      interpolable: true,
      field_group: "*",
      field_template: "primitive",
      default_value: "0",
      type_name: "int",
      typedom_types: ["Number"],
      valid_for_permission_element: true,
      invalidate: ["layout", "paint"],
    },
    {
      // This property is used for testing with origin trial intergration only.
      // It should never be web-exposed.
      name: "origin-trial-test-property",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      field_template: "keyword",
      default_value: "normal",
      keywords: ["normal", "none"],
      typedom_types: ["Keyword"],
      runtime_flag: "OriginTrialsSampleAPI",
      computable: false,
    },
    {
      name: "orphans",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ConsumePositiveInteger",
      interpolable: true,
      inherited: true,
      field_group: "*",
      field_template: "primitive",
      computed_style_custom_functions: ["setter"],
      default_value: "2",
      type_name: "short",
      typedom_types: ["Number"],
      valid_for_permission_element: true,
      invalidate: ["layout"],
    },
    {
      name: "outline-color",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal", "ColorIncludingFallback"],
      interpolable: true,
      field_group: "*",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/css/style_color.h"],
      default_value: "StyleColor::CurrentColor()",
      type_name: "StyleColor",
      computed_style_protected_functions: ["getter"],
      keywords: ["currentcolor"],
      includes_currentcolor: true,
      typedom_types: ["Keyword"],
      converter: "ConvertStyleColor",
      style_builder_template: "color",
      valid_for_cue: true,
      valid_for_permission_element: true,
      valid_for_page_context: true,
      invalidate: ["outline"],
    },
    {
      name: "outline-offset",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      field_group: "*",
      field_template: "external",
      default_value: "0",
      type_name: "int",
      converter: "ConvertOutlineOffset",
      typedom_types: ["Length"],
      valid_for_cue: true,
      valid_for_permission_element: true,
      valid_for_page_context: true,
      invalidate: ["outline"],
      affected_by_zoom: true,
    },
    {
      name: "outline-style",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      field_group: "*",
      field_template: "keyword",
      keywords: [
        "none", "hidden", "inset", "groove", "outset", "ridge", "dotted",
        "dashed", "solid", "double"
      ],
      devtools_keywords: [
        "none", "auto", "inset", "groove", "outset", "ridge", "dotted",
        "dashed", "solid", "double"
      ],
      typedom_types: ["Keyword"],
      typedom_keywords: [
        "none", "hidden", "inset", "groove", "outset", "ridge", "dotted",
        "dashed", "solid", "double", "auto"
      ],
      default_value: "none",
      type_name: "EBorderStyle",
      style_builder_custom_functions: ["initial", "inherit", "value"],
      valid_for_cue: true,
      valid_for_permission_element: true,
      valid_for_page_context: true,
      invalidate: ["outline"],
    },
    {
      name: "outline-width",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      field_group: "*",
      field_template: "external",
      default_value: "3",
      type_name: "int",
      style_builder_custom_functions: ["initial", "inherit"],
      converter: "ConvertBorderWidth",
      keywords: ["thin", "medium", "thick"],
      typedom_types: ["Keyword", "Length"],
      valid_for_cue: true,
      valid_for_permission_element: true,
      valid_for_page_context: true,
      invalidate: ["outline"],
      affected_by_zoom: true,
    },
    {
      name: "overflow-anchor",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      inherited: false,
      field_template: "keyword",
      keywords: [
        "visible", "none", "auto"
      ],
      typedom_types: ["Keyword"],
      default_value: "auto",
      valid_for_permission_element: true,
    },
    {
      name: "overflow-wrap",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      inherited: true,
      field_group: "*",
      field_template: "keyword",
      keywords: ["normal", "break-word", "anywhere"],
      default_value: "normal",
      typedom_types: ["Keyword"],
      valid_for_marker: true,
      invalidate: ["layout", "paint"],
    },
    {
      name: "overflow-inline",
      logical_property_group: {
        name: "overflow",
        resolver: "inline",
      },
      // See comment on overflow-x.
      supports_incremental_style: false,
    },
    {
      name: "overflow-block",
      logical_property_group: {
        name: "overflow",
        resolver: "block",
      },
      // See comment on overflow-x.
      supports_incremental_style: false,
    },
    {
      name: "overflow-clip-margin",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      field_template: "external",
      field_group: "box",
      include_paths: ["third_party/blink/renderer/core/style/style_overflow_clip_margin.h"],
      keywords: ["border-box", "content-box", "padding-box"],
      separator: " ",
      default_value: "std::nullopt",
      type_name: "std::optional<StyleOverflowClipMargin>",
      converter: "ConvertOverflowClipMargin",
      invalidate: ["box-paint-property", "layout"],
      affected_by_zoom: true,
    },
    {
      name: "overflow-x",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      field_template: "keyword",
      keywords: [
        "visible", "hidden", "scroll", "auto", "overlay", "clip"
      ],
      typedom_types: ["Keyword"],
      default_value: "visible",
      // This is to allow us to gather metrics when overflow is explicitly set.
      style_builder_custom_functions: ["initial", "inherit", "value"],
      type_name: "EOverflow",
      logical_property_group: {
        name: "overflow",
        resolver: "horizontal",
      },
      // Overflow has special semantics; overflowY can influence overflowX.
      // But StyleAdjuster::AdjustOverflow() only does this properly if overflowX
      // is set to the initial value (it cannot distinguish between an explicitly
      // set overflowX, and one that was already adjusted in a previous pass).
      // Modifying overflow frequently should not be common, so we take it out.
      supports_incremental_style: false,
      idempotent: false,
      invalidate: ["box-paint-property", "layout", "paint"],
    },
    {
      name: "overflow-y",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      field_template: "keyword",
      keywords: [
        "visible", "hidden", "scroll", "auto", "overlay", "clip"
      ],
      typedom_types: ["Keyword"],
      default_value: "visible",
      // This is to allow us to gather metrics when overflow is explicitly set.
      style_builder_custom_functions: ["initial", "inherit", "value"],
      type_name: "EOverflow",
      logical_property_group: {
        name: "overflow",
        resolver: "vertical",
      },
      // See comment on overflow-x.
      supports_incremental_style: false,
      idempotent: false,
      invalidate: ["box-paint-property", "layout", "paint"],
    },
    {
      name: "overscroll-container-type",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      field_group: "*",
      field_template: "keyword",
      keywords: ["none", "auto", "push", "overlay"],
      typedom_types: ["Keyword"],
      default_value: "auto",
      runtime_flag: "OverscrollAreas",
    },
    {
      name: "overscroll-behavior-inline",
      logical_property_group: {
        name: "overscroll-behavior",
        resolver: "inline",
      },
      valid_for_permission_element: true,
    },
    {
      name: "overscroll-behavior-block",
      logical_property_group: {
        name: "overscroll-behavior",
        resolver: "block",
      },
      valid_for_permission_element: true,
    },
    {
      name: "overscroll-behavior-x",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      field_template: "keyword",
      field_group: "*",
      keywords: ["auto", "chain", "contain", "none"],
      default_value: "auto",
      type_name: "EOverscrollBehavior",
      typedom_types: ["Keyword"],
      logical_property_group: {
        name: "overscroll-behavior",
        resolver: "horizontal",
      },
      invalidate: ["box-paint-property"],
      valid_for_permission_element: true,
    },
    {
      name: "overscroll-behavior-y",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      field_template: "keyword",
      field_group: "*",
      keywords: ["auto", "chain", "contain", "none"],
      default_value: "auto",
      type_name: "EOverscrollBehavior",
      typedom_types: ["Keyword"],
      logical_property_group: {
        name: "overscroll-behavior",
        resolver: "vertical",
      },
      invalidate: ["box-paint-property"],
      valid_for_permission_element: true,
    },
    {
      name: "-internal-overscroll-container",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      field_group: "*",
      field_template: "keyword",
      keywords: ["none", "auto"],
      typedom_types: ["Keyword"],
      default_value: "none",
      runtime_flag: "OverscrollAreas",
    },
    {
      name: "-internal-unbounded",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      field_group: "*",
      field_template: "keyword",
      keywords: ["none", "active"],
      typedom_types: ["Keyword"],
      default_value: "none",
      runtime_flag: "UnboundedElement",
      invalidate: ["layout", "paint"],
    },
    {
      name: "padding-bottom",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      layout_dependent: true,
      field_group: "box",
      field_template: "<length>",
      default_value: "Length::Fixed()",
      converter: "ConvertLength",
      computed_style_custom_functions: ["setter"],
      typedom_types: ["Keyword", "Length", "Percentage"],
      valid_for_first_letter: true,
      valid_for_page_context: true,
      logical_property_group: {
        name: "padding",
        resolver: "bottom",
      },
      supports_incremental_style: true,
      valid_for_permission_element: true,
      invalidate: ["layout", "paint", "scroll-anchor"],
      percentages_depend_on_used_value: true,
      affected_by_zoom: true,
    },
    {
      name: "padding-left",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      layout_dependent: true,
      field_group: "box",
      field_template: "<length>",
      default_value: "Length::Fixed()",
      converter: "ConvertLength",
      computed_style_custom_functions: ["setter"],
      typedom_types: ["Keyword", "Length", "Percentage"],
      valid_for_first_letter: true,
      valid_for_page_context: true,
      logical_property_group: {
        name: "padding",
        resolver: "left",
      },
      supports_incremental_style: true,
      valid_for_permission_element: true,
      invalidate: ["layout", "paint", "scroll-anchor"],
      percentages_depend_on_used_value: true,
      affected_by_zoom: true,
    },
    {
      name: "padding-right",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      layout_dependent: true,
      field_group: "box",
      field_template: "<length>",
      default_value: "Length::Fixed()",
      converter: "ConvertLength",
      computed_style_custom_functions: ["setter"],
      typedom_types: ["Keyword", "Length", "Percentage"],
      valid_for_first_letter: true,
      valid_for_page_context: true,
      logical_property_group: {
        name: "padding",
        resolver: "right",
      },
      supports_incremental_style: true,
      valid_for_permission_element: true,
      invalidate: ["layout", "paint", "scroll-anchor"],
      percentages_depend_on_used_value: true,
      affected_by_zoom: true,
    },
    {
      name: "padding-top",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      layout_dependent: true,
      field_group: "box",
      field_template: "<length>",
      default_value: "Length::Fixed()",
      converter: "ConvertLength",
      computed_style_custom_functions: ["setter"],
      typedom_types: ["Keyword", "Length", "Percentage"],
      valid_for_first_letter: true,
      valid_for_page_context: true,
      logical_property_group: {
        name: "padding",
        resolver: "top",
      },
      supports_incremental_style: true,
      valid_for_permission_element: true,
      invalidate: ["layout", "paint", "scroll-anchor"],
      percentages_depend_on_used_value: true,
      affected_by_zoom: true,
    },
    {
      name: "page",
      field_group: "*",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      converter: "ConvertPage",
      type_name: "AtomicString",
      default_value: "AtomicString()",
      field_template: "external",
      keywords: ["auto"],
      typedom_types: ["Keyword"],
      typedom_custom_ident: true,
      computable: false,
      valid_for_permission_element: true,
    },
    {
      name: "page-margin-safety",
      is_descriptor: true,
      field_group: "*",
      field_template: "keyword",
      keywords: ["none", "clamp", "add"],
      default_value: "none",
      getter: "GetPageMarginSafety",
      computed_style_custom_functions: ["setter"],
      computable: false,
      valid_for_page_context: true,
      invalidate: ["layout", "paint"],
      runtime_flag: "CSSSafePrintableInset",
    },
    {
      name: "page-orientation",
      is_descriptor: true,
      field_template: "primitive",
      field_group: "*",
      type_name: "PageOrientation",
      field_size: 2,
      default_value: "PageOrientation::kUpright",
      include_paths: ["third_party/blink/public/common/css/page_orientation.h"],
      computable: false,
      valid_for_page_context: true,
    },
    {
      name: "paint-order",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      inherited: true,
      field_group: "svginherited",
      field_template: "primitive",
      field_size: 3,
      type_name: "EPaintOrder",
      default_value: "kPaintOrderNormal",
      converter: "ConvertPaintOrder",
      keywords: ["normal", "fill", "stroke", "markers"],
      typedom_types: ["Keyword"],
      invalidate: ["paint"],
    },
    {
      name: "path-length",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      runtime_flag: "SvgPathLengthCssProperty",
      field_group: "svg->geometry",
      field_template: "<length>",
      default_value: "Length::None()",
      converter: "ConvertPathLength",
      keywords: ["none"],
      typedom_types: ["Keyword", "Length"],
      supports_incremental_style: true,
      invalidate: ["paint"],
      // <length> never accepts percentages, but the field type requires this.
      percentages_depend_on_used_value: false,
      affected_by_zoom: true,
    },
    {
      name: "perspective",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      field_group: "*",
      field_template: "primitive",
      default_value: "-1.0",
      type_name: "float",
      converter: "ConvertPerspective",
      keywords: ["none"],
      typedom_types: ["Keyword", "Length"],
      invalidate: ["transform-other"],
      affected_by_zoom: true,
    },
    {
      name: "perspective-origin",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      layout_dependent: true,
      field_group: "*",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/platform/geometry/length_point.h"],
      default_value: "LengthPoint(Length::Percent(50.0), Length::Percent(50.0))",
      type_name: "LengthPoint",
      converter: "ConvertPosition",
      typedom_types: ["Position"],
      // Overlaps with -webkit-perspective-origin-[x,y].
      overlapping: true,
      invalidate: ["transform-other"],
      percentages_depend_on_used_value: true,
      affected_by_zoom: true,
    },
    {
      name: "pointer-events",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      computed_style_protected_functions: ["getter"],
      independent: true,
      inherited: true,
      field_template: "keyword",
      keywords: [
        "none", "auto", "stroke", "fill", "painted", "visible", "visiblestroke",
        "visiblefill", "visiblepainted", "bounding-box", "all"
      ],
      typedom_types: ["Keyword"],
      default_value: "auto",
    },
    {
      name: "position",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      field_template: "keyword",
      keywords: [
        "static", "relative", "absolute", "fixed", "sticky"
      ],
      typedom_types: ["Keyword"],
      default_value: "static",
      getter: "GetPosition",
      computed_style_custom_functions: ["getter"],
      // See comment on display.
      supports_incremental_style: false,
      // @position-try-styling rely on this being high-priority, so that
      // declarations from @position-try blocks can be applied conditionally
      // based on whether or not we're out-of-flow positioned. Also, it needs to
      // have a priority higher than position-area which is already at priority:1.
      priority: 2,
      valid_for_permission_element: true,
      invalidate: ["clip", "layout", "scroll-anchor"],
    },
    {
      name: "position-anchor",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal" ],
      style_builder_custom_functions: ["initial", "inherit", "value"],
      include_paths: ["third_party/blink/renderer/core/style/style_position_anchor.h"],
      type_name: "StylePositionAnchor",
      default_value: "StylePositionAnchor::Initial()",
      field_group: "*",
      field_template: "external",
      converter: "ConvertPositionAnchor",
      keywords: ["auto", "none", "normal"],
      typedom_types: ["Keyword"],
      valid_for_permission_element: true,
      valid_for_position_try: true,
      // Needs to be applied before position-area which in turn needs to be applied
      // before inset properties.
      priority: 2,
      invalidate: ["layout", "paint"],
    },
    {
      name: "position-try-fallbacks",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ConsumePositionTryFallbacks",
      style_builder_custom_functions: ["value"],
      field_group: "*",
      field_template: "external",
      keywords: ["none", "flip-block", "flip-inline", "flip-start", "flip-x", "flip-y"],
      typedom_keywords: ["none"],
      typedom_types: ["Keyword"],
      include_paths: ["third_party/blink/renderer/core/style/position_try_fallbacks.h"],
      wrapper_pointer_name: "Member",
      type_name: "PositionTryFallbacks",
      default_value: "nullptr",
      invalidate: ["layout", "paint"],
    },
    {
      name: "position-try-order",
      property_methods: ["CSSValueFromComputedStyleInternal", "InitialValue"],
      field_group: "*",
      field_template: "keyword",
      keywords: ["normal", "most-width", "most-height", "most-block-size", "most-inline-size"],
      typedom_types: ["Keyword"],
      default_value: "normal",
      invalidate: ["layout", "paint"],
    },
    {
      name: "position-visibility",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      field_group: "*",
      field_size: 3,
      field_template: "primitive",
      default_value: "PositionVisibility::kAnchorVisible",
      getter: "GetPositionVisibility",
      type_name: "PositionVisibility",
      converter: "ConvertPositionVisibility",
      keywords: ["always", "anchor-valid", "anchor-visible", "anchors-visible", "no-overflow"],
      typedom_types: ["Keyword"],
      devtools_keywords: ["always", "anchor-valid", "anchor-visible", "no-overflow"],
      invalidate: ["layout", "paint"],
    },
    {
      name: "print-color-adjust",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      independent: false,  // Actually true, but setting it to false saves a precious bit in ComputedStyleBase.
      inherited: true,
      field_template: "keyword",
      keywords: ["economy", "exact"],
      default_value: "economy",
      valid_for_permission_element: true,
      invalidate: ["paint"],
    },
    {
      name: "quotes",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      inherited: true,
      field_group: "*",
      field_template: "pointer",
      include_paths: ["third_party/blink/renderer/platform/text/quotes_data.h"],
      wrapper_pointer_name: "scoped_refptr",
      default_value: "nullptr",
      type_name: "QuotesData",
      converter: "ConvertQuotes",
      keywords: ["auto", "none"],
      typedom_types: ["Keyword"],
      valid_for_marker: true,
      valid_for_page_context: true,
      invalidate: ["layout", "paint"],
    },
    {
      name: "content-visibility",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      field_template: "keyword",
      keywords: ["visible", "auto", "hidden"],
      default_value: "visible",
      typedom_types: ["Keyword"],
      invalidate: ["layout"],
    },
    {
      name: "reading-flow",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      field_group: "*",
      field_template: "keyword",
      keywords: ["normal", "flex-visual", "flex-flow", "grid-rows", "grid-columns", "grid-order", "source-order"],
      typedom_types: ["Keyword"],
      default_value: "normal",
      invalidate: ["layout"],
    },
    {
      name: "reading-order",
      interpolable: true,
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ConsumeInteger",
      field_group: "*",
      field_template: "primitive",
      typedom_types: ["Number"],
      default_value: "0",
      type_name: "int",
      invalidate: ["layout"],
      valid_for_permission_element: true,
    },
    {
      name: "resize",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      field_group: "*",
      field_template: "keyword",
      computed_style_protected_functions: ["getter"],
      style_builder_custom_functions: ["value"],
      keywords: ["none", "both", "horizontal", "vertical", "block", "inline"],
      typedom_types: ["Keyword"],
      default_value: "none",
      invalidate: ["paint"],
    },
    {
      name: "right",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      layout_dependent: true,
      field_group: "surround",
      field_template: "<length>",
      keywords: ["auto"],
      default_value: "Length()",
      typedom_types: ["Keyword", "Length", "Percentage"],
      converter: "ConvertLengthOrAuto",
      anchor_mode: "right",
      logical_property_group: {
        name: "inset",
        resolver: "right",
      },
      supports_incremental_style: true,
      valid_for_position_try: true,
      valid_for_permission_element: true,
      invalidate: ["inset", "scroll-anchor"],
      affected_by_zoom: true,
      percentages_depend_on_used_value: true,
    },
    {
      name: "r",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      field_group: "svg->geometry",
      field_template: "<length>",
      default_value: "Length::Fixed()",
      typedom_types: ["Length", "Percentage"],
      converter: "ConvertLength",
      supports_incremental_style: true,
      invalidate: ["layout", "paint"],
      percentages_depend_on_used_value: true,
      affected_by_zoom: true,
    },
    {
      name: "rx",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      field_group: "svg->geometry",
      field_template: "<length>",
      default_value: "Length::Auto()",
      keywords: ["auto"],
      typedom_types: ["Keyword", "Length", "Percentage"],
      converter: "ConvertLengthOrAuto",
      supports_incremental_style: true,
      invalidate: ["layout", "paint"],
      percentages_depend_on_used_value: true,
      affected_by_zoom: true,
    },
    {
      name: "ry",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      field_group: "svg->geometry",
      field_template: "<length>",
      default_value: "Length::Auto()",
      keywords: ["auto"],
      typedom_types: ["Keyword", "Length", "Percentage"],
      converter: "ConvertLengthOrAuto",
      supports_incremental_style: true,
      invalidate: ["layout", "paint"],
      percentages_depend_on_used_value: true,
      affected_by_zoom: true,
    },
    {
      name: "scroll-axis-lock",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      field_group: "*",
      field_template: "keyword",
      keywords: ["auto", "none"],
      default_value: "auto",
      typedom_types: ["Keyword"],
      runtime_flag: "ScrollAxisLock",
      invalidate: ["compositing"],
    },
    {
      name: "scroll-target-group",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      field_group: "*",
      field_template: "keyword",
      keywords: ["none", "auto"],
      typedom_types: ["Keyword"],
      default_value: "none",
      runtime_flag: "CSSScrollTargetGroup",
    },
    {
      name: "scroll-marker-group",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      field_group: "*",
      field_template: "pointer",
      typedom_types: ["Keyword"],
      default_value: "nullptr",
      runtime_flag: "CSSPseudoScrollMarkers",
      include_paths: ["third_party/blink/renderer/core/style/scroll_marker_group.h"],
      wrapper_pointer_name: "Member",
      type_name: "ScrollMarkerGroup",
      converter: "ConvertScrollMarkerGroup",
      typedom_types: ["Keyword"],
      keywords: ["none", "after", "before"],
    },
    {
      interpolable: true,
      name: "scrollbar-color",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      inherited: true,
      name_for_methods: "ScrollbarColor",
      type_name: "StyleScrollbarColor",
      converter: "ConvertScrollbarColor",
      keywords: ["auto"],
      field_group: "*",
      field_template: "pointer",
      default_value: "nullptr",
      wrapper_pointer_name: "Member",
      include_paths: ["third_party/blink/renderer/core/style/style_scrollbar_color.h"],
      invalidate: ["scrollbar-style", "scrollbar-color"],
      runtime_flag: "ScrollbarColor",
    },
    {
      name: "scrollbar-gutter",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      inherited: false,
      field_size: 4,
      field_template: "primitive",
      default_value: "kScrollbarGutterAuto",
      name_for_methods: "ScrollbarGutter",
      type_name: "unsigned",
      converter: "ConvertScrollbarGutter",
      keywords: [
        "auto", "stable", "both-edges"
      ],
      typedom_types: ["Keyword"],
      invalidate: ["layout"],
      field_group: "*",
    },
    {
      name: "scrollbar-width",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      inherited: false,
      field_group: "*",
      field_template: "keyword",
      keywords: ["auto", "thin", "none"],
      default_value: "auto",
      typedom_types: ["Keyword"],
      invalidate: ["layout", "paint", "scrollbar-style"],
      runtime_flag: "ScrollbarWidth",
    },
    {
      name: "scroll-behavior",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      field_group: "*",
      field_size: 2, // FIXME: Convert this to a keyword field
      field_template: "primitive",
      include_paths: ["third_party/blink/public/mojom/scroll/scroll_enums.mojom-blink.h"],
      default_value: "mojom::blink::ScrollBehavior::kAuto",
      type_name: "mojom::blink::ScrollBehavior",
      keywords: ["auto", "smooth"],
      typedom_types: ["Keyword"],
    },
    {
      name: "scroll-initial-target",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      field_group: "*",
      field_template: "keyword",
      type_name: "EScrollInitialTarget",
      default_value: "none",
      keywords: ["none", "nearest"],
      invalidate: ["layout"],
      runtime_flag: "CSSScrollInitialTarget",
    },
    {
      name: "scroll-margin-block-end",
      property_methods: ["ParseSingleValue"],
      logical_property_group: {
        name: "scroll-margin",
        resolver: "block-end",
      },
      typedom_types: ["Keyword", "Length"],
      valid_for_permission_element: true,
    },
    {
      name: "scroll-margin-block-start",
      property_methods: ["ParseSingleValue"],
      logical_property_group: {
        name: "scroll-margin",
        resolver: "block-start",
      },
      typedom_types: ["Keyword", "Length"],
      valid_for_permission_element: true,
    },
    {
      name: "scroll-margin-bottom",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      field_group: "*",
      field_template: "primitive",
      default_value: "0.0f",
      type_name: "float",
      converter: "ConvertComputedLength<float>",
      typedom_types: ["Keyword", "Length"],
      logical_property_group: {
        name: "scroll-margin",
        resolver: "bottom",
      },
      valid_for_permission_element: true,
      affected_by_zoom: true,
    },
    {
      name: "scroll-margin-inline-end",
      property_methods: ["ParseSingleValue"],
      logical_property_group: {
        name: "scroll-margin",
        resolver: "inline-end",
      },
      typedom_types: ["Keyword", "Length"],
      valid_for_permission_element: true,
    },
    {
      name: "scroll-margin-inline-start",
      property_methods: ["ParseSingleValue"],
      logical_property_group: {
        name: "scroll-margin",
        resolver: "inline-start",
      },
      typedom_types: ["Keyword", "Length"],
      valid_for_permission_element: true,
    },
    {
      name: "scroll-margin-left",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      field_group: "*",
      field_template: "primitive",
      default_value: "0.0f",
      type_name: "float",
      converter: "ConvertComputedLength<float>",
      typedom_types: ["Keyword", "Length"],
      logical_property_group: {
        name: "scroll-margin",
        resolver: "left",
      },
      valid_for_permission_element: true,
      affected_by_zoom: true,
    },
    {
      name: "scroll-margin-right",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      field_group: "*",
      field_template: "primitive",
      default_value: "0.0f",
      type_name: "float",
      converter: "ConvertComputedLength<float>",
      typedom_types: ["Keyword", "Length"],
      logical_property_group: {
        name: "scroll-margin",
        resolver: "right",
      },
      valid_for_permission_element: true,
      affected_by_zoom: true,
    },
    {
      name: "scroll-margin-top",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      field_group: "*",
      field_template: "primitive",
      default_value: "0.0f",
      type_name: "float",
      converter: "ConvertComputedLength<float>",
      typedom_types: ["Keyword", "Length"],
      logical_property_group: {
        name: "scroll-margin",
        resolver: "top",
      },
      valid_for_permission_element: true,
      affected_by_zoom: true,
    },
    {
      name: "scroll-padding-block-end",
      parse_helper: "ConsumeScrollPadding",
      include_paths: ["third_party/blink/renderer/platform/geometry/length.h"],
      keywords: ["auto"],
      typedom_types: ["Keyword", "Length", "Percentage"],
      type_name: "Length",
      converter: "ConvertLengthOrAuto",
      logical_property_group: {
        name: "scroll-padding",
        resolver: "block-end",
      },
      valid_for_permission_element: true,
      percentages_depend_on_used_value: true,
    },
    {
      name: "scroll-padding-block-start",
      parse_helper: "ConsumeScrollPadding",
      include_paths: ["third_party/blink/renderer/platform/geometry/length.h"],
      keywords: ["auto"],
      typedom_types: ["Keyword", "Length", "Percentage"],
      type_name: "Length",
      converter: "ConvertLengthOrAuto",
      logical_property_group: {
        name: "scroll-padding",
        resolver: "block-start",
      },
      valid_for_permission_element: true,
      percentages_depend_on_used_value: true,
    },
    {
      name: "scroll-padding-bottom",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ConsumeScrollPadding",
      field_group: "*",
      field_template: "<length>",
      default_value: "Length()",
      keywords: ["auto"],
      typedom_types: ["Keyword", "Length", "Percentage"],
      converter: "ConvertLengthOrAuto",
      logical_property_group: {
        name: "scroll-padding",
        resolver: "bottom",
      },
      valid_for_permission_element: true,
      percentages_depend_on_used_value: true,
      affected_by_zoom: true,
    },
    {
      name: "scroll-padding-inline-end",
      parse_helper: "ConsumeScrollPadding",
      include_paths: ["third_party/blink/renderer/platform/geometry/length.h"],
      keywords: ["auto"],
      typedom_types: ["Keyword", "Length", "Percentage"],
      type_name: "Length",
      converter: "ConvertLengthOrAuto",
      logical_property_group: {
        name: "scroll-padding",
        resolver: "inline-end",
      },
      valid_for_permission_element: true,
      percentages_depend_on_used_value: true,
    },
    {
      name: "scroll-padding-inline-start",
      parse_helper: "ConsumeScrollPadding",
      include_paths: ["third_party/blink/renderer/platform/geometry/length.h"],
      keywords: ["auto"],
      typedom_types: ["Keyword", "Length", "Percentage"],
      type_name: "Length",
      converter: "ConvertLengthOrAuto",
      logical_property_group: {
        name: "scroll-padding",
        resolver: "inline-start",
      },
      valid_for_permission_element: true,
      percentages_depend_on_used_value: true,
    },
    {
      name: "scroll-padding-left",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ConsumeScrollPadding",
      field_group: "*",
      field_template: "<length>",
      default_value: "Length()",
      keywords: ["auto"],
      typedom_types: ["Keyword", "Length", "Percentage"],
      converter: "ConvertLengthOrAuto",
      logical_property_group: {
        name: "scroll-padding",
        resolver: "left",
      },
      valid_for_permission_element: true,
      percentages_depend_on_used_value: true,
      affected_by_zoom: true,
    },
    {
      name: "scroll-padding-right",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ConsumeScrollPadding",
      field_group: "*",
      field_template: "<length>",
      default_value: "Length()",
      keywords: ["auto"],
      typedom_types: ["Keyword", "Length", "Percentage"],
      converter: "ConvertLengthOrAuto",
      logical_property_group: {
        name: "scroll-padding",
        resolver: "right",
      },
      valid_for_permission_element: true,
      percentages_depend_on_used_value: true,
      affected_by_zoom: true,
    },
    {
      name: "scroll-padding-top",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ConsumeScrollPadding",
      field_group: "*",
      field_template: "<length>",
      default_value: "Length()",
      keywords: ["auto"],
      typedom_types: ["Keyword", "Length", "Percentage"],
      converter: "ConvertLengthOrAuto",
      logical_property_group: {
        name: "scroll-padding",
        resolver: "top",
      },
      valid_for_permission_element: true,
      percentages_depend_on_used_value: true,
      affected_by_zoom: true,
    },
    {
      name: "scroll-snap-align",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      field_group: "*",
      field_template: "external",
      include_paths: ["cc/input/scroll_snap_data.h"],
      default_value: "cc::ScrollSnapAlign()",
      getter: "GetScrollSnapAlign",
      type_name: "cc::ScrollSnapAlign",
      converter: "ConvertSnapAlign",
      keywords: ["none", "start", "end", "center"],
      typedom_types: ["Keyword"],
    },
    {
      name: "scroll-snap-stop",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      field_template: "keyword",
      keywords: ["normal", "always", "before"],
      default_value: "normal",
      typedom_types: ["Keyword"],
    },
    {
      name: "scroll-snap-type",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      field_group: "*",
      field_template: "external",
      include_paths: ["cc/input/scroll_snap_data.h"],
      default_value: "cc::ScrollSnapType()",
      getter: "GetScrollSnapType",
      type_name: "cc::ScrollSnapType",
      converter: "ConvertSnapType",
      keywords: ["none", "x", "y", "block", "inline", "both", "mandatory", "proximity", "pair"],
      typedom_types: ["Keyword"],
    },
    {
      name: "scroll-timeline-axis",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal", "InitialValue"],
      field_group: "misc->timeline",
      field_template: "external",
      default_value: "Vector<TimelineAxis>()",
      type_name: "Vector<TimelineAxis>",
      converter: "ConvertViewTimelineAxis",
      separator: ",",
      is_animation_affecting: true,
    },
    {
      name: "scroll-timeline-name",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal", "InitialValue"],
      field_group: "misc->timeline",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/style/scoped_css_name.h"],
      default_value: "nullptr",
      wrapper_pointer_name: "Member",
      type_name: "ScopedCSSNameList",
      converter: "ConvertScrollTimelineName",
      separator: ",",
      is_animation_affecting: true,
      keywords: ["none"],
    },
    {
      name: "shape-image-threshold",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ConsumeAlphaValue",
      interpolable: true,
      field_group: "*",
      field_template: "primitive",
      default_value: "0.0",
      type_name: "float",
      computed_style_custom_functions: ["setter"],
      typedom_types: ["Number"],
      accepts_numeric_literal: true,
      percentages_depend_on_used_value: false,
    },
    {
      name: "shape-margin",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      field_group: "*",
      field_template: "<length>",
      default_value: "Length::Fixed(0)",
      converter: "ConvertLength",
      typedom_types: ["Length", "Percentage"],
      invalidate: ["layout", "paint"],
      percentages_depend_on_used_value: true,
      affected_by_zoom: true,
    },
    {
      name: "shape-outside",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      field_group: "*",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/style/shape_value.h"],
      wrapper_pointer_name: "Member",
      default_value: "nullptr",
      typedom_types: ["Keyword", "Image"],
      type_name: "ShapeValue",
      computed_style_custom_functions: ["getter"],
      converter: "ConvertShapeValue",
      keywords: ["none"],
      invalidate: ["paint"],
      affected_by_zoom: true,
    },
    {
      name: "shape-rendering",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      inherited: true,
      field_group: "svginherited",
      field_template: "keyword",
      keywords: ["auto", "optimizespeed", "crispedges", "geometricprecision"],
      default_value: "auto",
      typedom_types: ["Keyword"],
      invalidate: ["paint"],
    },
    {
      name: "size",
      property_methods: ["ParseSingleValue"],
      style_builder_custom_functions: ["initial", "inherit", "value"],
      computable: false,
      valid_for_page_context: true,
      stored_on_extra_field: ["PageSize", "PageSizeType"],
      keywords: ["auto", "portrait", "landscape"],
    },
    {
      name: "speak",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      inherited: true,
      field_group: "*",
      field_template: "keyword",
      keywords: [
        "none", "normal", "spell-out", "digits", "literal-punctuation",
        "no-punctuation"
      ],
      default_value: "normal",
    },
    {
      name: "stop-color",
      property_methods: ["CSSValueFromComputedStyleInternal", "ColorIncludingFallback"],
      parse_helper: "ConsumeColor",
      interpolable: true,
      field_group: "svg->stop",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/css/style_color.h"],
      default_value: "StyleColor(Color::kBlack)",
      type_name: "StyleColor",
      computed_style_protected_functions: ["getter"],
      style_builder_template: "color",
      style_builder_template_args: {
        initial_color: "ComputedStyleInitialValues::InitialStopColor",
      },
      converter: "ConvertStyleColor",
      keywords: ["currentcolor"],
      includes_currentcolor: true,
      typedom_types: ["Keyword"],
      invalidate: ["paint"],
    },
    {
      name: "stop-opacity",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ConsumeAlphaValue",
      interpolable: true,
      field_group: "svg->stop",
      field_template: "primitive",
      type_name: "float",
      default_value: "1",
      converter: "ConvertAlpha",
      typedom_types: ["Number", "Percentage"],
      accepts_numeric_literal: true,
      invalidate: ["paint"],
      percentages_depend_on_used_value: false,
    },
    {
      name: "stroke",
      property_methods: ["CSSValueFromComputedStyleInternal", "ColorIncludingFallback"],
      parse_helper: "ConsumeSVGPaint",
      interpolable: true,
      inherited: true,
      includes_currentcolor: true,
      field_group: "svginherited->stroke",
      field_template: "external",
      type_name: "SVGPaint",
      include_paths: ["third_party/blink/renderer/core/style/svg_paint.h"],
      default_value: "SVGPaint::CreateInitial()",
      name_for_methods: "StrokePaint",
      converter: "ConvertSVGPaint",
      style_builder_template: "color",
      style_builder_template_args: {
        initial_color: "ComputedStyleInitialValues::InitialStrokePaint",
      },
      style_builder_custom_functions: ["value"],
      valid_for_highlight: true,
      valid_for_permission_icon: true,
      invalidate: ["paint", "stroke"],
      keywords: ["none", "context-stroke"],
    },
    {
      name: "stroke-dasharray",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      inherited: true,
      field_group: "svginherited->stroke",
      field_template: "pointer",
      type_name: "SVGDashArray",
      include_paths: ["third_party/blink/renderer/core/style/svg_dash_array.h"],
      wrapper_pointer_name: "Member",
      default_value: "nullptr",
      name_for_methods: "StrokeDashArray",
      converter: "ConvertStrokeDasharray",
      keywords: ["none"],
      typedom_types: ["Keyword"],
      invalidate: ["paint", "stroke", "border-shape"],
      affected_by_zoom: true,
      percentages_depend_on_used_value: true,
    },
    {
      name: "stroke-dashoffset",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      inherited: true,
      field_group: "svginherited->stroke",
      field_template: "external",
      type_name: "Length",
      default_value: "Length::Fixed()",
      name_for_methods: "StrokeDashOffset",
      converter: "ConvertLength",
      typedom_types: ["Length", "Percentage"],
      invalidate: ["paint"],
      affected_by_zoom: true,
      percentages_depend_on_used_value: true,
    },
    {
      name: "stroke-linecap",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      inherited: true,
      field_group: "svginherited->stroke",
      field_template: "primitive",
      field_size: 2,
      type_name: "LineCap",
      default_value: "kButtCap",
      name_for_methods: "CapStyle",
      keywords: ["butt", "round", "square"],
      typedom_types: ["Keyword"],
      invalidate: ["layout", "paint", "border-shape"],
    },
    {
      name: "stroke-linejoin",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      inherited: true,
      field_group: "svginherited->stroke",
      field_template: "primitive",
      field_size: 2,
      type_name: "LineJoin",
      default_value: "kMiterJoin",
      name_for_methods: "JoinStyle",
      keywords: ["miter", "bevel", "round"],
      typedom_types: ["Keyword"],
      invalidate: ["layout", "paint", "border-shape"],
    },
    {
      name: "stroke-miterlimit",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      inherited: true,
      field_group: "svginherited->stroke",
      field_template: "primitive",
      type_name: "float",
      default_value: "4",
      name_for_methods: "StrokeMiterLimit",
      typedom_types: ["Number"],
      invalidate: ["layout", "paint", "border-shape"],
    },
    {
      name: "stroke-opacity",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ConsumeAlphaValue",
      interpolable: true,
      inherited: true,
      field_group: "svginherited->stroke",
      field_template: "primitive",
      type_name: "float",
      default_value: "1",
      converter: "ConvertAlpha",
      typedom_types: ["Number", "Percentage"],
      accepts_numeric_literal: true,
      invalidate: ["paint"],
      percentages_depend_on_used_value: false,
    },
    {
      name: "stroke-width",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      inherited: true,
      field_group: "svginherited->stroke",
      field_template: "external",
      type_name: "UnzoomedLength",
      include_paths: ["third_party/blink/renderer/core/style/unzoomed_length.h"],
      default_value: "UnzoomedLength(Length::Fixed(1))",
      converter: "ConvertUnzoomedLength",
      typedom_types: ["Length", "Percentage"],
      valid_for_highlight: true,
      valid_for_permission_icon: true,
      invalidate: ["layout", "paint", "border-shape"],
      affected_by_zoom: true,
      percentages_depend_on_used_value: true,
    },
    {
      name: "table-layout",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      field_template: "keyword",
      keywords: [
        "auto", "fixed"
      ],
      typedom_types: ["Keyword"],
      default_value: "auto",
      invalidate: ["layout", "paint"],
    },
    {
      name: "tab-size",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      inherited: true,
      field_group: "*",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/platform/text/tab_size.h"],
      default_value: "TabSize(8)",
      getter: "GetTabSize",
      type_name: "TabSize",
      converter: "ConvertLengthOrTabSpaces",
      computed_style_custom_functions: ["setter"],
      typedom_types: ["Number", "Length"],
      valid_for_marker: true,
      invalidate: ["layout", "paint"],
      affected_by_zoom: true,
    },
    {
      name: "text-align",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      independent: false,
      inherited: true,
      field_template: "keyword",
      keywords: [
        "left", "right", "center", "justify", "-webkit-left", "-webkit-right",
        "-webkit-center", "start", "end", "match-parent"
      ],
      // `match-parent` is an experimental keyword and currently behind the
      // CSSTextAlignMatchParent flag. Since it is not fully supported yet,
      // it is not being exposed to devtools.
      devtools_keywords: [
        "left", "right", "center", "justify", "-webkit-left", "-webkit-right",
        "-webkit-center", "start", "end"
      ],
      typedom_types: ["Keyword"],
      default_value: "start",
      getter: "GetTextAlign",
      style_builder_custom_functions: ["value"],
      valid_for_page_context: true,
      invalidate: ["ax-style", "layout", "paint"],
    },
    {
      name: "text-align-last",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      inherited: true,
      field_group: "*",
      field_template: "keyword",
      keywords: ["auto", "start", "end", "left", "right", "center", "justify",
        "match-parent"
      ],
      // `match-parent` is an experimental keyword and currently behind the
      // CSSTextAlignMatchParent flag. Since it is not fully supported yet,
      // it is not being exposed to devtools.
      devtools_keywords: [
        "auto", "start", "end", "left", "right", "center", "justify"
      ],
      default_value: "auto",
      typedom_types: ["Keyword"],
      valid_for_page_context: true,
      invalidate: ["layout", "paint"],
    },
    {
      name: "text-anchor",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      inherited: true,
      field_group: "svginherited",
      field_template: "keyword",
      keywords: ["start", "middle", "end"],
      default_value: "start",
      typedom_types: ["Keyword"],
      invalidate: ["layout", "paint"],
    },
    {
      name: "text-autospace",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      inherited: true,
      field_group: "*",
      field_template: "keyword",
      keywords: ["no-autospace", "normal"],
      default_value: "no-autospace",
      typedom_types: ["Keyword"],
      invalidate: ["reshape"],
    },
    {
      name: "text-box",
      longhands: ["text-box-trim", "text-box-edge"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
    },
    {
      name: "text-box-edge",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      inherited: true,
      field_group: "*",
      field_template: "primitive",
      field_size: 6,  // Sync with `TextBoxEdge::kBits`.
      include_paths: ["third_party/blink/renderer/core/style/text_box_edge.h"],
      default_value: "TextBoxEdge()",
      type_name: "TextBoxEdge",
      converter: "ConvertTextBoxEdge",
      invalidate: ["layout"],
      keywords: ["auto", "text", "cap", "ex"],
    },
    {
      name: "text-box-trim",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      field_group: "box",
      field_template: "keyword",
      default_value: "none",
      keywords: ["none", "trim-start", "trim-end", "trim-both"],
      typedom_types: ["Keyword"],
      invalidate: ["layout"],
    },
    {
      name: "text-combine-upright",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      inherited: true,
      field_group: "*",
      field_template: "keyword",
      keywords: ["none", "all"],
      typedom_types: ["Keyword"],
      default_value: "none",
      name_for_methods: "TextCombine",
      valid_for_marker: true,
      invalidate: ["layout", "paint"],
      is_animation_affecting: true,
    },
    {
      name: "text-decoration-color",
      property_methods: ["CSSValueFromComputedStyleInternal", "ColorIncludingFallback"],
      parse_helper: "ConsumeColor",
      interpolable: true,
      field_group: "*",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/css/style_color.h"],
      default_value: "StyleColor::CurrentColor()",
      type_name: "StyleColor",
      computed_style_protected_functions: ["getter"],
      keywords: ["currentcolor"],
      includes_currentcolor: true,
      typedom_types: ["Keyword"],
      converter: "ConvertStyleColor",
      style_builder_template: "color",
      valid_for_first_letter: true,
      valid_for_first_line: true,
      valid_for_cue: true,
      valid_for_highlight: true,
      supports_incremental_style: true,
      valid_for_page_context: true,
      invalidate: ["color"],
    },
    {
      name: "text-decoration-line",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      field_group: "visual",
      field_template: "multi_keyword",
      keywords: ["none", "underline", "overline", "line-through", "blink", "spelling-error", "grammar-error"],
      typedom_types: ["Keyword"],
      default_value: "none",
      type_name: "TextDecorationLine",
      converter: "ConvertFlags<blink::TextDecorationLine>",
      valid_for_first_letter: true,
      valid_for_first_line: true,
      valid_for_cue: true,
      valid_for_highlight: true,
      valid_for_page_context: true,
      invalidate: ["text-decoration"],
    },
    {
      name: "text-decoration-inset",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      include_paths: ["third_party/blink/renderer/core/style/text_decoration_inset.h"],
      inherited: false,
      field_group: "*",
      field_template: "external",
      type_name: "TextDecorationInset",
      default_value: "TextDecorationInset(Length::Fixed(0), Length::Fixed(0))",
      converter: "ConvertTextDecorationInset",
      keywords: ["auto"],
      typedom_types: ["Keyword", "Length", "Percentage"],
      valid_for_first_letter: true,
      valid_for_first_line: true,
      valid_for_highlight: true,
      valid_for_page_context: true,
      runtime_flag: "CSSTextDecorationInset",
      invalidate: ["text-decoration"],
      affected_by_zoom: true,
      percentages_depend_on_used_value: true,
      interpolable: true,
    },
    {
      name: "text-decoration-skip-ink",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      inherited: true,
      field_group: "*",
      field_template: "keyword",
      keywords: ["none", "auto", "all"],
      typedom_types: ["Keyword"],
      default_value: "auto",
      valid_for_first_letter: true,
      valid_for_first_line: true,
      valid_for_cue: true,
      valid_for_marker: true,
      valid_for_highlight: true,
      valid_for_page_context: true,
      invalidate: ["text-decoration"],
    },
    {
      name: "text-decoration-skip-spaces",
      runtime_flag: "CSSTextDecorationSkipSpaces",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      inherited: true,
      field_group: "*",
      field_template: "multi_keyword",
      keywords: ["none", "start", "end", "all"],
      include_paths:["third_party/blink/renderer/core/style/text_decoration_skip_spaces.h"],
      typedom_types: ["Keyword"],
      separator: " ",
      default_value: "start end",
      type_name: "TextDecorationSkipSpaces",
      converter: "ConvertFlags<blink::TextDecorationSkipSpaces>",
      valid_for_first_letter: true,
      valid_for_first_line: true,
      valid_for_cue: true,
      valid_for_marker: true,
      valid_for_highlight: true,
      valid_for_page_context: true,
      invalidate: ["text-decoration"],
    },
    {
      name: "text-decoration-style",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      field_group: "*",
      field_template: "keyword",
      keywords: ["solid", "double", "dotted", "dashed", "wavy"],
      typedom_types: ["Keyword"],
      default_value: "solid",
      valid_for_first_letter: true,
      valid_for_first_line: true,
      valid_for_cue: true,
      valid_for_highlight: true,
      valid_for_page_context: true,
      invalidate: ["text-decoration"],
    },
    {
      name: "text-decoration-thickness",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      include_paths: ["third_party/blink/renderer/core/style/text_decoration_thickness.h"],
      inherited: false,
      field_group: "*",
      field_template: "external",
      type_name: "TextDecorationThickness",
      default_value: "TextDecorationThickness(Length::Auto())",
      converter: "ConvertTextDecorationThickness",
      keywords: ["auto", "from-font"],
      typedom_types: ["Keyword", "Length", "Percentage"],
      valid_for_first_letter: true,
      valid_for_first_line: true,
      valid_for_highlight: true,
      valid_for_page_context: true,
      invalidate: ["text-decoration"],
      percentages_depend_on_used_value: false,
      affected_by_zoom: true,
    },
    {
      name: "text-indent",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      inherited: true,
      field_group: "*",
      field_template: "<length>",
      default_value: "Length::Fixed()",
      style_builder_custom_functions: ["value"],
      typedom_types: ["Length", "Percentage"],
      valid_for_page_context: true,
      invalidate: ["ax-style", "layout", "paint"],
      affected_by_zoom: true,
      percentages_depend_on_used_value: true,
    },
    {
      name: "text-justify",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      field_group: "*",
      field_template: "primitive",
      field_size: 2,
      type_name: "TextJustify",
      default_value: "TextJustify::kAuto",
      include_paths: ["third_party/blink/renderer/platform/text/text_justify.h"],
      inherited: true,
      keywords: ["auto", "none", "inter-word", "inter-character"],
      typedom_types: ["Keyword"],
      valid_for_first_letter: true,
      valid_for_first_line: true,
      invalidate: ["layout"],
    },
    {
      name: "text-overflow",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      field_group: "*",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/style/text_overflow_data.h"],
      default_value: "TextOverflowData(TextOverflowData::Type::kClip)",
      type_name: "TextOverflowData",
      converter: "ConvertTextOverflow",
      keywords: ["clip", "ellipsis"],
      typedom_types: ["Keyword"],
      valid_for_page_context: true,
      invalidate: ["layout", "paint"],
    },
    {
      name: "text-shadow",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      inherited: true,
      field_group: "*",
      field_template: "pointer",
      include_paths: ["third_party/blink/renderer/core/style/shadow_list.h"],
      wrapper_pointer_name: "Member",
      default_value: "nullptr",
      type_name: "ShadowList",
      converter: "ConvertShadowList",
      keywords: ["none"],
      typedom_types: ["Keyword"],
      valid_for_first_letter: true,
      valid_for_first_line: true,
      valid_for_cue: true,
      valid_for_marker: true,
      valid_for_highlight: true,
      valid_for_page_context: true,
      invalidate: ["layout", "paint"],
      affected_by_zoom: true,
      includes_currentcolor: true,
    },
    {
      name: "text-size-adjust",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      inherited: true,
      field_group: "*",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/style/text_size_adjust.h"],
      style_builder_custom_functions: ["initial", "inherit", "value"],
      default_value: "TextSizeAdjust::AdjustAuto()",
      getter: "GetTextSizeAdjust",
      // Affects font-size during style building and needs to apply before it.
      priority: 2,
      type_name: "TextSizeAdjust",
      converter: "ConvertTextSizeAdjust",
      keywords: ["none", "auto"],
      typedom_types: ["Keyword", "Percentage"],
      // Affects font-size during style building. Changes to anything related to
      // fonts require that we call style.UpdateFont(), which the incremental
      // path does not do.
      supports_incremental_style: false,
      invalidate: ["layout", "paint"],
      percentages_depend_on_used_value: false,
    },
    {
      name: "text-spacing",
      longhands: ["text-autospace", "text-spacing-trim"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      runtime_flag: "CSSTextSpacing",
    },
    {
      name: "text-spacing-trim",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      inherited: true,
      font: true,
      priority: 1,
      type_name: "TextSpacingTrim",
      include_paths: ["third_party/blink/renderer/platform/fonts/shaping/text_spacing_trim.h"],
      keywords: ["normal", "space-all", "space-first", "trim-start"],
      default_value: "TextSpacingTrim::kInitial",
      typedom_types: ["Keyword"],
      valid_for_permission_element: true,
      stored_on_extra_field: ["font"],
    },
    {
      name: "text-transform",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      independent: true,
      inherited: true,
      field_group: "inherited",
      field_template: "multi_keyword",
      keywords: ["none", "capitalize", "uppercase", "lowercase", "full-width", "full-size-kana", "math-auto"],
      // `full-width` and `full-size-kana` are experimental keywords and currently
      // behind the CSSTextTransformFullWidth and CSSTextTransformFullSizeKana
      // flags respectively. Since they are not fully supported yet, they are
      // not being exposed to devtools.
      devtools_keywords: ["none", "capitalize", "uppercase", "lowercase", "math-auto"],
      type_name: "ETextTransform",
      converter: "ConvertFlags<blink::ETextTransform>",
      typedom_types: ["Keyword"],
      default_value: "none",
      valid_for_first_letter: true,
      valid_for_first_line: true,
      valid_for_marker: true,
      valid_for_page_context: true,
      invalidate: ["reshape"],
      valid_for_permission_element: true,
    },
    {
      name: "text-underline-offset",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      inherited: true,
      field_group: "*",
      field_template: "<length>",
      default_value: "Length()",
      name_for_methods: "TextUnderlineOffset",
      converter: "ConvertTextUnderlineOffset",
      keywords: ["auto"],
      typedom_types: ["Keyword", "Length", "Percentage"],
      valid_for_first_letter: true,
      valid_for_first_line: true,
      valid_for_highlight: true,
      valid_for_page_context: true,
      invalidate: ["text-decoration"],
      affected_by_zoom: true,
      percentages_depend_on_used_value: false,
    },
    {
      name: "text-underline-position",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      inherited: true,
      field_group: "*",
      field_size: 4,
      field_template: "primitive",
      default_value: "TextUnderlinePosition::kAuto",
      getter: "GetTextUnderlinePosition",
      type_name: "TextUnderlinePosition",
      converter: "ConvertTextUnderlinePosition",
      keywords: ["auto", "from-font", "under", "left", "right"],
      typedom_types: ["Keyword"],
      valid_for_first_letter: true,
      valid_for_first_line: true,
      valid_for_page_context: true,
      invalidate: ["text-decoration"],
    },
    {
      name: "timeline-scope",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      field_group: "misc->timeline",
      field_template: "external",
      default_value: "StyleTimelineScope()",
      include_paths: ["third_party/blink/renderer/core/style/style_timeline_scope.h"],
      type_name: "StyleTimelineScope",
      converter: "ConvertTimelineScope",
      separator: ",",
      is_animation_affecting: true,
      keywords: ["none", "all"],
      // `all` is an experimental keyword and currently behind the
      // CSSTimelineScopeAll flag. Since it is not fully supported yet,
      // it is not being exposed to devtools.
      devtools_keywords: ["none"],
    },
    {
      name: "top",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      layout_dependent: true,
      field_group: "surround",
      field_template: "<length>",
      keywords: ["auto"],
      default_value: "Length()",
      typedom_types: ["Keyword", "Length", "Percentage"],
      converter: "ConvertLengthOrAuto",
      anchor_mode: "top",
      logical_property_group: {
        name: "inset",
        resolver: "top",
      },
      supports_incremental_style: true,
      valid_for_position_try: true,
      valid_for_permission_element: true,
      invalidate: ["inset", "scroll-anchor"],
      affected_by_zoom: true,
      percentages_depend_on_used_value: true,
    },
    {
      name: "overlay",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      field_group: "*",
      field_template: "keyword",
      keywords: ["none", "auto"],
      default_value: "none",
      typedom_types: ["Keyword"],
      runtime_flag: "OverlayProperty",
    },
    {
      name: "touch-action",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      field_group: "*",
      field_size: 8, // FIXME: Make this use "kTouchActionBits".
      field_template: "primitive",
      include_paths: ["third_party/blink/renderer/platform/graphics/touch_action.h"],
      default_value: "TouchAction::kAuto",
      type_name: "TouchAction",
      converter: "ConvertFlags<blink::TouchAction>",
      keywords: ["auto", "none", "pan-x", "pan-left", "pan-right", "pan-y", "pan-up", "pan-down", "pinch-zoom", "manipulation"],
      typedom_types: ["Keyword"],
    },
    {
      name: "transform",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ConsumeTransformList",
      interpolable: true,
      compositable: true,
      layout_dependent: true,
      tracks_animated_source: true,
      field_group: "svg",  // Not SVG, but frequently used with it.
      field_template: "external",
      include_paths: ["third_party/blink/renderer/platform/transforms/transform_operations.h"],
      keywords: ["none"],
      default_value: "EmptyTransformOperations()",
      typedom_types: ["Keyword", "Transform"],
      type_name: "TransformOperations",
      converter: "ConvertTransformOperations",
      supports_incremental_style: true,
      invalidate: ["has-transform", "transform-data", "transform-property"],
      affected_by_zoom: true,
    },
    {
      name: "transform-box",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      field_template: "keyword",
      keywords: ["content-box", "border-box", "fill-box", "stroke-box", "view-box"],
      default_value: "view-box",
      typedom_types: ["Keyword"],
      supports_incremental_style: true,
      invalidate: ["transform-data", "transform-other"],
    },
    {
      name: "transform-origin",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      layout_dependent: true,
      field_group: "svg",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/style/transform_origin.h"],
      default_value: "TransformOrigin(Length::Percent(50.0), Length::Percent(50.0), 0)",
      getter: "GetTransformOrigin",
      type_name: "TransformOrigin",
      converter: "ConvertTransformOrigin",
      supports_incremental_style: true,
      // Overlaps with -webkit-transform-origin-[x,y,z].
      overlapping: true,
      invalidate: ["transform-data", "transform-other"],
      percentages_depend_on_used_value: true,
      affected_by_zoom: true,
    },
    {
      name: "transform-style",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      field_group: "*",
      field_template: "keyword",
      keywords: ["flat", "preserve-3d"],
      default_value: "flat",
      typedom_types: ["Keyword"],
      name_for_methods: "TransformStyle3D",
      supports_incremental_style: true,
    },
    {
      name: "translate",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      compositable: true,
      layout_dependent: true,
      tracks_animated_source: true,
      field_group: "*",
      field_template: "pointer",
      include_paths: ["third_party/blink/renderer/platform/transforms/translate_transform_operation.h"],
      wrapper_pointer_name: "Member",
      default_value: "nullptr",
      type_name: "TranslateTransformOperation",
      converter: "ConvertTranslate",
      supports_incremental_style: true,
      invalidate: ["has-transform", "transform-data", "transform-other"],
      percentages_depend_on_used_value: true,
      keywords: ["none"],
      affected_by_zoom: true,
    },
    {
      name: "rotate",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      compositable: true,
      tracks_animated_source: true,
      field_group: "*",
      field_template: "pointer",
      include_paths: ["third_party/blink/renderer/platform/transforms/rotate_transform_operation.h"],
      wrapper_pointer_name: "Member",
      default_value: "nullptr",
      type_name: "RotateTransformOperation",
      converter: "ConvertRotate",
      supports_incremental_style: true,
      invalidate: ["has-transform", "transform-data", "transform-other"],
      keywords: ["none"],
    },
    {
      name: "scale",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      compositable: true,
      tracks_animated_source: true,
      field_group: "*",
      field_template: "pointer",
      include_paths: ["third_party/blink/renderer/platform/transforms/scale_transform_operation.h"],
      wrapper_pointer_name: "Member",
      default_value: "nullptr",
      type_name: "ScaleTransformOperation",
      converter: "ConvertScale",
      supports_incremental_style: true,
      invalidate: ["has-transform", "transform-data", "transform-other"],
      percentages_depend_on_used_value: true,
      keywords: ["none"],
    },
    {
      name: "unicode-bidi",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      affected_by_all: false,
      field_template: "keyword",
      include_paths: ["third_party/blink/renderer/platform/text/unicode_bidi.h"],
      keywords: [
        "normal", "embed", "bidi-override", "isolate", "plaintext",
        "isolate-override"
      ],
      typedom_types: ["Keyword"],
      default_value: "normal",
      type_name: "UnicodeBidi",
      valid_for_marker: true,
      invalidate: ["reshape"],
      is_animation_affecting: true,
    },
    {
      name: "vector-effect",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      field_group: "svg",
      field_template: "keyword",
      keywords: ["none", "non-scaling-stroke"],
      typedom_types: ["Keyword"],
      default_value: "none",
      invalidate: ["layout", "paint"],
    },
    {
      name: "vertical-align",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      style_builder_custom_functions: ["inherit", "value"],
      typedom_types: ["Keyword", "Length", "Percentage"],
      keywords: ["baseline", "sub", "super", "text-top", "text-bottom", "middle"],
      valid_for_first_letter: true,
      valid_for_first_line: true,
      valid_for_page_context: true,
      valid_for_permission_element: true,
      stored_on_extra_field: ["VerticalAlign", "VerticalAlignLength"],
      percentages_depend_on_used_value: false,
      affected_by_zoom: true,
    },
    {
      name: "view-timeline-axis",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal", "InitialValue"],
      field_group: "misc->timeline",
      field_template: "external",
      default_value: "Vector<TimelineAxis>()",
      type_name: "Vector<TimelineAxis>",
      converter: "ConvertViewTimelineAxis",
      separator: ",",
      is_animation_affecting: true,
    },
    {
      name: "view-timeline-inset",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal", "InitialValue"],
      field_group: "misc->timeline",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/animation/timeline_inset.h"],
      default_value: "Vector<TimelineInset>()",
      type_name: "Vector<TimelineInset>",
      converter: "ConvertViewTimelineInset",
      separator: ",",
      is_animation_affecting: true,
      percentages_depend_on_used_value: true,
      keywords: ["auto"],
      affected_by_zoom: true,
    },
    {
      name: "view-timeline-name",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal", "InitialValue"],
      field_group: "misc->timeline",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/style/scoped_css_name.h"],
      default_value: "nullptr",
      wrapper_pointer_name: "Member",
      type_name: "ScopedCSSNameList",
      converter: "ConvertViewTimelineName",
      separator: ",",
      is_animation_affecting: true,
      keywords: ["none"],
    },
    {
      name: "view-transition-class",
      field_group: "*",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      converter: "ConvertViewTransitionClass",
      type_name: "ScopedCSSNameList",
      default_value: "nullptr",
      wrapper_pointer_name: "Member",
      field_template: "external",
      keywords: ["none"],
      typedom_types: ["Keyword"],
    },
    {
      name: "view-transition-group",
      field_group: "*",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      converter: "ConvertViewTransitionGroup",
      type_name: "StyleViewTransitionGroup",
      default_value: "StyleViewTransitionGroup::Normal()",
      include_paths: ["third_party/blink/renderer/core/style/style_view_transition_group.h"],
      field_template: "external",
      keywords: ["normal", "contain", "nearest"],
      typedom_types: ["Keyword"],
    },
    {
      name: "view-transition-name",
      field_group: "*",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      converter: "ConvertViewTransitionName",
      type_name: "StyleViewTransitionName",
      default_value: "nullptr",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/style/style_view_transition_name.h"],
      keywords: ["none", "auto"],
      // `auto` is an experimental keyword and currently behind the
      // CSSViewTransitionNameAuto flag. Since it is not fully supported yet,
      // it is not being exposed to devtools.
      devtools_keywords: ["none"],
      typedom_types: ["Keyword"],
      wrapper_pointer_name: "Member",
    },
    {
      name: "view-transition-scope",
      field_group: "*",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      inherited: false,
      field_template: "keyword",
      keywords: ["none", "all"],
      typedom_types: ["Keyword"],
      default_value: "none",
    },
    {
      name: "visibility",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      independent: true,
      interpolable: true,
      inherited: true,
      field_template: "keyword",
      keywords: ["visible", "hidden", "collapse"],
      typedom_types: ["Keyword"],
      default_value: "visible",
      valid_for_first_letter: true,
      valid_for_first_line: true,
      valid_for_cue: true,
      valid_for_permission_element: true,
      valid_for_page_context: true,
      invalidate: ["paint", "visibility"],
    },
    {
      name: "x",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      field_group: "svg->geometry",
      field_template: "<length>",
      default_value: "Length::Fixed()",
      typedom_types: ["Length", "Percentage"],
      converter: "ConvertLength",
      supports_incremental_style: true,
      valid_for_permission_element: true,
      invalidate: ["layout", "paint"],
      percentages_depend_on_used_value: true,
      affected_by_zoom: true,
    },
    {
      name: "y",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      field_group: "svg->geometry",
      field_template: "<length>",
      default_value: "Length::Fixed()",
      typedom_types: ["Length", "Percentage"],
      converter: "ConvertLength",
      supports_incremental_style: true,
      valid_for_permission_element: true,
      invalidate: ["layout", "paint"],
      percentages_depend_on_used_value: true,
      affected_by_zoom: true,
    },
    {
      name: "appearance",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      field_group: "*",
      field_size: 5,
      field_template: "primitive",
      include_paths: ["third_party/blink/renderer/platform/theme_types.h"],
      computed_style_protected_functions: ["getter"],
      default_value: "AppearanceValue::kNone",
      type_name: "AppearanceValue",
      // appearance needs to be computed before
      // -internal-auto-base() can be resolved.
      priority: 1,
      // we invalidate for EffectiveAppearance rather than appearance
      keywords: ["auto", "none", "checkbox", "radio", "button", "listbox",
                "menulist", "menulist-button", "meter", "progress-bar",
                "searchfield", "textfield", "textarea", "base-select"]
    },
    {
      name: "-webkit-appearance",
      alias_for: "appearance",
    },
    {
      name: "app-region",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      surrogate_for: "window-drag",
      devtools_keywords: ["none", "move", "no-drag"],
    },
    {
      name: "-webkit-app-region",
      alias_for: "app-region",
    },
    {
      name: "window-drag",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      field_group: "*",
      field_template: "keyword",
      type_name: "EDraggableRegionMode",
      default_value: "none",
      inherited: true,
      // `window-drag` only exposes `none` and `move`; the parser rejects the
      // extra `no-drag` keyword (see css_parser_fast_paths.cc). It is listed
      // here solely so the auto-generated enum includes a `kNoDrag` state for
      // the `app-region` surrogate. `app-region: drag` maps to the shared
      // `kMove` state.
      keywords: ["none", "move", "no-drag"],
      devtools_keywords: ["none", "move"],
      name_for_methods: "DraggableRegionMode",
      style_builder_custom_functions: ["value"],
      invalidate: ["layout"],
      runtime_flag: "CSSWindowDrag",
    },
    {
      name: "-webkit-border-horizontal-spacing",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      inherited: true,
      field_group: "inherited",
      field_template: "primitive",
      default_value: "0",
      name_for_methods: "HorizontalBorderSpacing",
      type_name: "short",
      converter: "ConvertComputedLength<short>",
      valid_for_first_letter: true,
      invalidate: ["layout", "paint"],
      affected_by_zoom: true,
    },
    {
      name: "-webkit-border-image",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ConsumeWebkitBorderImage",
      style_builder_custom_functions: ["value"],
      valid_for_first_letter: true,
      affected_by_all: false,
      // Overlaps with border-image.
      legacy_overlapping: true,
      percentages_depend_on_used_value: true,
      devtools_keywords: ["none", "stretch", "repeat", "space", "round"],
    },
    {
      name: "-webkit-border-vertical-spacing",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      inherited: true,
      field_group: "inherited",
      field_template: "primitive",
      default_value: "0",
      name_for_methods: "VerticalBorderSpacing",
      type_name: "short",
      converter: "ConvertComputedLength<short>",
      valid_for_first_letter: true,
      invalidate: ["layout", "paint"],
      affected_by_zoom: true,
    },
    // For valid values of box-align see
    // http://www.w3.org/TR/2009/WD-css3-flexbox-20090723/#alignment
    {
      name: "-webkit-box-align",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      field_group: "*",
      field_template: "keyword",
      keywords: ["stretch", "start", "center", "end", "baseline"],
      default_value: "stretch",
      type_name: "EBoxAlignment",
      valid_for_permission_element: true,
      invalidate: ["layout", "paint"],
    },
    {
      name: "-webkit-box-decoration-break",
      surrogate_for: "box-decoration-break",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      keywords: ["slice", "clone"],
      stored_on_extra_field: ["crbug.com/40919412"],
    },
    {
      name: "-webkit-box-direction",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      field_template: "keyword",
      keywords: ["normal", "reverse"],
      default_value: "normal",
      computed_style_protected_functions: ["getter"],
      invalidate: ["layout", "paint"],
    },
    {
      name: "-webkit-box-flex",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      field_group: "*",
      field_template: "primitive",
      default_value: "0.0f",
      type_name: "float",
      accepts_numeric_literal: true,
      invalidate: ["layout", "paint"],
    },
    {
      name: "-webkit-box-ordinal-group",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ConsumePositiveInteger",
      field_group: "*",
      field_template: "primitive",
      default_value: "1",
      type_name: "unsigned",
      computed_style_custom_functions: ["setter"],
      valid_for_permission_element: true,
      invalidate: ["layout", "paint"],
    },
    {
      name: "-webkit-box-orient",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      field_group: "*",
      field_template: "keyword",
      keywords: ["horizontal", "vertical"],
      default_value: "horizontal",
      invalidate: ["layout", "paint"],
    },
    {
      name: "-webkit-box-pack",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      field_group: "*",
      field_template: "keyword",
      keywords: ["start", "center", "end", "justify"],
      default_value: "start",
      invalidate: ["layout", "paint"],
    },
    {
      name: "-webkit-box-reflect",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      field_group: "*",
      field_template: "pointer",
      include_paths: ["third_party/blink/renderer/core/style/style_reflection.h"],
      wrapper_pointer_name: "Member",
      default_value: "nullptr",
      type_name: "StyleReflection",
      converter: "ConvertBoxReflect",
      invalidate: ["filter-data"],
      includes_currentcolor: true,
      percentages_depend_on_used_value: true,
      affected_by_zoom: true,
    },
    {
      name: "column-count",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ConsumeColumnCount",
      interpolable: true,
      field_group: "*",
      field_template: "primitive",
      default_value: "1",
      type_name: "unsigned short",
      computed_style_custom_functions: ["setter"],
      style_builder_template: "auto",
      keywords: ["auto"],
      typedom_types: ["Keyword", "Number"],
      invalidate: ["layout", "paint"],
    },
    {
      name: "column-gap",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ConsumeGapLength",
      interpolable: true,
      field_group: "*",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/platform/geometry/length.h"],
      default_value: "std::nullopt",
      type_name: "std::optional<Length>",
      converter: "ConvertGapLength",
      keywords: ["normal"],
      typedom_types: ["Keyword", "Length", "Percentage"],
      invalidate: ["layout", "paint"],
      percentages_depend_on_used_value: true,
      affected_by_zoom: true,
    },
    {
      name: "row-gap",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ConsumeGapLength",
      interpolable: true,
      field_group: "*",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/platform/geometry/length.h"],
      default_value: "std::nullopt",
      type_name: "std::optional<Length>",
      converter: "ConvertGapLength",
      keywords: ["normal"],
      typedom_types: ["Keyword", "Length", "Percentage"],
      invalidate: ["layout", "paint"],
      percentages_depend_on_used_value: true,
      affected_by_zoom: true,
    },
    {
      name: "rule-overlap",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      inherited: false,
      field_group: "*",
      field_template: "keyword",
      keywords: ["row-over-column", "column-over-row"],
      default_value: "row-over-column",
      typedom_types: ["Keyword"],
      invalidate: ["paint"],
    },
    {
      name: "column-rule-break",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      inherited: false,
      field_group: "*",
      field_template: "keyword",
      keywords: ["none", "normal", "intersection"],
      default_value: "normal",
      type_name: "RuleBreak",
      typedom_types: ["Keyword"],
      invalidate: ["paint"],
    },
    {
      name: "row-rule-break",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      inherited: false,
      field_group: "*",
      field_template: "keyword",
      keywords: ["none", "normal", "intersection"],
      default_value: "normal",
      type_name: "RuleBreak",
      typedom_types: ["Keyword"],
      invalidate: ["paint"],
    },
    {
      name: "column-rule-inset-cap-end",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      inherited: false,
      field_group: "*",
      field_template: "<length>",
      default_value: "Length::Fixed(0)",
      keywords: ["overlap-join"],
      typedom_types: ["Keyword", "Length", "Percentage"],
      converter: "ConvertGapDecorationInsetLength",
      invalidate: ["paint"],
      percentages_depend_on_used_value: true,
      affected_by_zoom: true,
    },
    {
      name: "row-rule-inset-cap-end",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      inherited: false,
      field_group: "*",
      field_template: "<length>",
      default_value: "Length::Fixed(0)",
      keywords: ["overlap-join"],
      typedom_types: ["Keyword", "Length", "Percentage"],
      converter: "ConvertGapDecorationInsetLength",
      invalidate: ["paint"],
      percentages_depend_on_used_value: true,
      affected_by_zoom: true,
    },
    {
      name: "column-rule-inset-cap-start",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      inherited: false,
      field_group: "*",
      field_template: "<length>",
      default_value: "Length::Fixed(0)",
      keywords: ["overlap-join"],
      typedom_types: ["Keyword", "Length", "Percentage"],
      converter: "ConvertGapDecorationInsetLength",
      invalidate: ["paint"],
      percentages_depend_on_used_value: true,
      affected_by_zoom: true,
    },
    {
      name: "row-rule-inset-cap-start",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      inherited: false,
      field_group: "*",
      field_template: "<length>",
      default_value: "Length::Fixed(0)",
      keywords: ["overlap-join"],
      typedom_types: ["Keyword", "Length", "Percentage"],
      converter: "ConvertGapDecorationInsetLength",
      invalidate: ["paint"],
      percentages_depend_on_used_value: true,
      affected_by_zoom: true,
    },
    {
      name: "column-rule-inset-junction-end",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      inherited: false,
      field_group: "*",
      field_template: "<length>",
      default_value: "Length::Fixed(0)",
      keywords: ["overlap-join"],
      typedom_types: ["Keyword", "Length", "Percentage"],
      converter: "ConvertGapDecorationInsetLength",
      invalidate: ["paint"],
      percentages_depend_on_used_value: true,
      affected_by_zoom: true,
    },
    {
      name: "row-rule-inset-junction-end",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      inherited: false,
      field_group: "*",
      field_template: "<length>",
      default_value: "Length::Fixed(0)",
      keywords: ["overlap-join"],
      typedom_types: ["Keyword", "Length", "Percentage"],
      converter: "ConvertGapDecorationInsetLength",
      invalidate: ["paint"],
      percentages_depend_on_used_value: true,
      affected_by_zoom: true,
    },
    {
      name: "column-rule-inset-junction-start",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      inherited: false,
      field_group: "*",
      field_template: "<length>",
      default_value: "Length::Fixed(0)",
      keywords: ["overlap-join"],
      typedom_types: ["Keyword", "Length", "Percentage"],
      converter: "ConvertGapDecorationInsetLength",
      invalidate: ["paint"],
      percentages_depend_on_used_value: true,
      affected_by_zoom: true,
    },
    {
      name: "row-rule-inset-junction-start",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      inherited: false,
      field_group: "*",
      field_template: "<length>",
      default_value: "Length::Fixed(0)",
      keywords: ["overlap-join"],
      typedom_types: ["Keyword", "Length", "Percentage"],
      converter: "ConvertGapDecorationInsetLength",
      invalidate: ["paint"],
      percentages_depend_on_used_value: true,
      affected_by_zoom: true,
    },
    {
      name: "column-rule-color",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal", "ColorIncludingFallback"],
      interpolable: true,
      field_group: "*",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/style/gap_data_list.h",
                      "third_party/blink/renderer/core/css/style_color.h"],
      default_value: "GapDataList<StyleColor>::DefaultGapColorDataList()",
      type_name: "GapDataList<StyleColor>",
      keywords: ["currentcolor"],
      computed_style_custom_functions: ["setter"],
      includes_currentcolor: true,
      typedom_types: ["Keyword"],
      converter: "ConvertGapDecorationColorDataList",
      invalidate: ["paint"],
    },
    {
      name: "row-rule-color",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      field_group: "*",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/style/gap_data_list.h",
                      "third_party/blink/renderer/core/css/style_color.h"],
      default_value: "GapDataList<StyleColor>::DefaultGapColorDataList()",
      type_name: "GapDataList<StyleColor>",
      keywords: ["currentcolor"],
      computed_style_custom_functions: ["setter"],
      includes_currentcolor: true,
      typedom_types: ["Keyword"],
      converter: "ConvertGapDecorationColorDataList",
      invalidate: ["paint"],
    },
    {
      name: "column-rule-style",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      field_group: "*",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/style/gap_data_list.h"],
      keywords: [
        "none", "hidden", "inset", "groove", "outset", "ridge", "dotted",
        "dashed", "solid", "double"
      ],
      computed_style_custom_functions: ["setter"],
      default_value: "GapDataList<EBorderStyle>::DefaultGapStyleDataList()",
      type_name: "GapDataList<EBorderStyle>",
      typedom_types: ["Keyword"],
      converter: "ConvertGapDecorationStyleDataList",
      invalidate: ["paint", "gap-decorations"],
    },
    {
      name: "row-rule-style",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      field_group: "*",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/style/gap_data_list.h"],
      keywords: [
        "none", "hidden", "inset", "groove", "outset", "ridge", "dotted",
        "dashed", "solid", "double"
      ],
      computed_style_custom_functions: ["setter"],
      default_value: "GapDataList<EBorderStyle>::DefaultGapStyleDataList()",
      type_name: "GapDataList<EBorderStyle>",
      typedom_types: ["Keyword"],
      converter: "ConvertGapDecorationStyleDataList",
      invalidate: ["paint", "gap-decorations"],
    },
    {
      name: "column-rule-visibility-items",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      inherited: false,
      field_group: "*",
      field_template: "keyword",
      keywords: ["all", "normal", "around", "between"],
      default_value: "normal",
      type_name: "RuleVisibilityItems",
      typedom_types: ["Keyword"],
      invalidate: ["paint"],
    },
    {
      name: "row-rule-visibility-items",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      inherited: false,
      field_group: "*",
      field_template: "keyword",
      keywords: ["all", "normal", "around", "between"],
      default_value: "normal",
      type_name: "RuleVisibilityItems",
      typedom_types: ["Keyword"],
      invalidate: ["paint"],
    },
    {
      name: "column-rule-width",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      field_group: "*",
      field_template: "external",
      default_value: "ComputedStyleInitialValues::InitialColumnRuleWidth()",
      include_paths: ["third_party/blink/renderer/core/style/gap_data_list.h"],
      type_name: "GapDataList<int>",
      computed_style_custom_functions: ["initial", "setter"],
      style_builder_custom_functions: ["initial", "inherit"],
      converter: "ConvertGapDecorationWidthDataList",
      keywords: ["thin", "medium", "thick"],
      typedom_types: ["Keyword", "Length"],
      invalidate: ["paint"],
      affected_by_zoom: true,
    },
    {
      name: "row-rule-width",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      field_group: "*",
      field_template: "external",
      default_value: "ComputedStyleInitialValues::InitialRowRuleWidth()",
      include_paths: ["third_party/blink/renderer/core/style/gap_data_list.h"],
      type_name: "GapDataList<int>",
      computed_style_custom_functions: ["initial", "setter"],
      style_builder_custom_functions: ["initial", "inherit"],
      converter: "ConvertGapDecorationWidthDataList",
      keywords: ["thin", "medium", "thick"],
      typedom_types: ["Keyword", "Length"],
      invalidate: ["paint"],
      affected_by_zoom: true,
    },
    {
      name: "column-span",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      field_group: "*",
      field_template: "keyword",
      keywords: ["none", "all"],
      default_value: "none",
      getter: "GetColumnSpan",
      typedom_types: ["Keyword"],
      invalidate: ["layout", "paint"],
    },
    {
      name: "column-width",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ConsumeColumnLength",
      interpolable: true,
      field_group: "*",
      field_template: "primitive",
      default_value: "0.0f",
      type_name: "float",
      computed_style_custom_functions: ["setter"],
      style_builder_template: "auto",
      converter: "ConvertComputedLength<float>",
      keywords: ["auto"],
      typedom_types: ["Keyword", "Length"],
      invalidate: ["layout", "paint"],
      affected_by_zoom: true,
    },
    {
      name: "column-height",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ConsumeColumnLength",
      interpolable: true,
      field_group: "*",
      field_template: "primitive",
      default_value: "0.0f",
      type_name: "float",
      computed_style_custom_functions: ["setter"],
      style_builder_template: "auto",
      converter: "ConvertComputedLength<float>",
      keywords: ["auto"],
      typedom_types: ["Keyword", "Length"],
      invalidate: ["layout", "paint"],
      affected_by_zoom: true,
    },
    {
      name: "column-wrap",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      field_group: "*",
      field_template: "keyword",
      keywords: ["auto", "nowrap", "wrap"],
      default_value: "auto",
      typedom_types: ["Keyword"],
      invalidate: ["layout", "paint"],
    },
    {
      name: "hyphenate-character",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      inherited: true,
      field_group: "*",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/platform/wtf/text/atomic_string.h"],
      default_value: "AtomicString()",
      name_for_methods: "HyphenationString",
      type_name: "AtomicString",
      converter: "ConvertString<CSSValueID::kAuto>",
      invalidate: ["layout", "paint"],
      keywords: ["auto"],
    },
    {
      name: "-webkit-line-break",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      keywords: ["auto", "loose", "normal", "strict", "after-white-space"],
      surrogate_for: "line-break",
      stored_on_extra_field: ["crbug.com/40919412"],
    },
    {
      name: "line-break",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      inherited: true,
      field_group: "*",
      field_template: "keyword",
      default_value: "auto",
      type_name: "LineBreak",
      keywords: ["auto", "loose", "normal", "strict", "anywhere", "after-white-space"],
      devtools_keywords: ["auto", "loose", "normal", "strict", "anywhere"],
      typedom_types: ["Keyword"],
      valid_for_marker: true,
      invalidate: ["layout", "paint"],
    },
    {
      name: "continue",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      keywords: ["normal", "collapse", "-webkit-legacy"],
      typedom_types: ["Keyword"],
      stored_on_extra_field: ["Continue"],
      runtime_flag: "CSSLineClampAsShorthand",
    },
    {
      name: "max-lines",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      converter: "ConvertMaxLines",
      keywords: ["auto"],
      typedom_types: ["Number", "Keyword"],
      stored_on_extra_field: ["MaxLines"],
      runtime_flag: "CSSLineClampAsShorthand",
    },
    {
      name: "block-ellipsis",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      inherited: true,
      field_group: "*",
      field_template: "keyword",
      keywords: ["ellipsis", "no-ellipsis"],
      default_value: "no-ellipsis",
      typedom_types: ["Keyword"],
      invalidate: ["layout", "paint"],
      runtime_flag: "CSSLineClampAsShorthand",
    },
    {
      // This is a longhand that behaves like the spec'd shorthand, so we store it in extra fields
      // corresponding to each of the longhands, which are also used for those longhands proper with
      // the `CSSLineClampAsShorthand` runtime flag.
      // However, the spec's `block-ellipsis` longhand is inherited, while the rest of longhands
      // aren't; so to avoid issues, we avoid sharing its state with the longhand.
      name: "line-clamp",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      style_builder_custom_functions: ["initial", "inherit", "value"],
      keywords: ["none", "auto", "ellipsis", "no-ellipsis", "-webkit-legacy"],
      devtools_keywords: ["none", "auto", "ellipsis", "no-ellipsis"],
      typedom_types: ["Keyword", "Number"],
      stored_on_extra_field: [
        "Continue",
        "MaxLines",
        "LineClampInternalBlockEllipsis"
      ],
      // overlaps with -alternative-webkit-line-clamp
      overlapping: true,
      runtime_flag: "CSSLineClamp"
    },
    {
      // True shorthand for `line-clamp`
      name: "-alternative-line-clamp-shorthand",
      alternative_of: "line-clamp",
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      longhands: ["max-lines", "block-ellipsis", "continue"],
      runtime_flag: "CSSLineClampAsShorthand",
    },
    {
      // Standalone (legacy) version of the -webkit-line-clamp property, when no
      // related runtime flags are enabled.
      name: "-webkit-line-clamp",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      field_group: "*",
      field_template: "primitive",
      default_value: "0",
      converter: "ConvertIntegerOrNone<0>",
      keywords: ["none"],
      type_name: "int",
      invalidate: ["layout", "paint"],
      name_for_methods: "WebkitLineClamp",
    },
    {
      // Longhand version of -webkit-line-clamp which stores its values in extra
      // fields, used when the CSSLineClamp is enabled (so there *is* a
      // `line-clamp` property) but when CSSLineClampAsShorthand is disabled (so
      // `line-clamp` is a longhand, not a shorthand).
      name: "-alternative-webkit-line-clamp-longhand",
      alternative_of: "-webkit-line-clamp",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      style_builder_custom_functions: ["initial", "inherit", "value"],
      keywords: ["none"],
      typedom_types: ["Keyword", "Number"],
      stored_on_extra_field: [
        "Continue",
        "MaxLines",
        "LineClampInternalBlockEllipsis"
      ],
      // overlaps with line-clamp
      legacy_overlapping: true,
      runtime_flag: "CSSLineClamp",
    },
    {
      // Shorthand version of -webkit-line-clamp, corresponding to the
      // `-alternative-line-clamp-shorthand` definition.
      name: "-alternative-webkit-line-clamp-shorthand",
      alternative_of: "-alternative-webkit-line-clamp-longhand",
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      longhands: ["max-lines", "block-ellipsis", "continue"],
      runtime_flag: "CSSLineClampAsShorthand",
    },
    {
      name: "-webkit-mask-box-image-outset",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ConsumeBorderImageOutset",
      interpolable: true,
      style_builder_template: "mask_box",
      style_builder_template_args: {
        modifier_type: "Outset",
      },
      stored_on_extra_field: ["MaskBoxImage"],
      affected_by_zoom: true,
    },
    {
      name: "-webkit-mask-box-image-repeat",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      style_builder_template: "mask_box",
      style_builder_template_args: {
        modifier_type: "Repeat",
      },
      stored_on_extra_field: ["MaskBoxImage"],
      keywords: ["repeat", "stretch", "space", "round"],
    },
    {
      name: "-webkit-mask-box-image-slice",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      style_builder_template: "mask_box",
      style_builder_template_args: {
        modifier_type: "Slice",
      },
      stored_on_extra_field: ["MaskBoxImage"],
      percentages_depend_on_used_value: true,
    },
    {
      name: "-webkit-mask-box-image-source",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ConsumeImageOrNone",
      interpolable: true,
      style_builder_custom_functions: ["value"],
      includes_currentcolor: true,
      stored_on_extra_field: ["MaskBoxImage"],
      keywords: ["none"],
    },
    {
      name: "-webkit-mask-box-image-width",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ConsumeBorderImageWidth",
      interpolable: true,
      style_builder_template: "mask_box",
      style_builder_template_args: {
        modifier_type: "Width",
      },
      stored_on_extra_field: ["MaskBoxImage"],
      percentages_depend_on_used_value: true,
      keywords: ["auto"],
      affected_by_zoom: true,
    },
    {
      name: "mask-mode",
      keywords: ["alpha", "luminance", "match-source"],
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal", "InitialValue"],
      style_builder_template: "mask_layer",
      style_builder_template_args: {
        fill_type: "MaskMode",
      },
      stored_on_extra_field: ["Mask"],
    },
    {
      name: "mask-clip",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal", "InitialValue"],
      style_builder_template: "mask_layer",
      style_builder_template_args: {
        fill_type: "Clip",
      },
      stored_on_extra_field: ["Mask"],
    },
    {
      name: "-webkit-mask-clip",
      alias_for: "mask-clip",
    },
    {
      name: "mask-composite",
      keywords: ["add", "subtract", "intersect", "exclude"],
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal", "InitialValue"],
      style_builder_template: "mask_layer",
      style_builder_template_args: {
        fill_type: "CompositingOperator",
      },
      // {-webkit-}mask-image needs to be applied before {-webkit-}mask-composite,
      // otherwise {-webkit-}mask-composite has no effect.
      priority: -1,
      stored_on_extra_field: ["Mask"],
    },
    {
      name: "-webkit-mask-composite",
      alias_for: "mask-composite",
    },
    {
      name: "mask-image",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      style_builder_template: "mask_layer",
      style_builder_template_args: {
        fill_type: "Image",
        fill_type_getter: "GetImage"
      },
      includes_currentcolor: true,
      stored_on_extra_field: ["Mask"],
      keywords: ["none"],
    },
    {
      name: "-webkit-mask-image",
      alias_for: "mask-image",
    },
    {
      name: "mask-origin",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal", "InitialValue"],
      style_builder_template: "mask_layer",
      style_builder_template_args: {
        fill_type: "Origin",
      },
      stored_on_extra_field: ["Mask"],
    },
    {
      name: "-webkit-mask-origin",
      alias_for: "mask-origin",
    },
    {
      name: "-webkit-mask-position-x",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal", "InitialValue"],
      interpolable: true,
      style_builder_template: "mask_layer",
      style_builder_template_args: {
        fill_type: "PositionX",
      },
      stored_on_extra_field: ["Mask"],
      percentages_depend_on_used_value: true,
      affected_by_zoom: true,
    },
    {
      name: "-webkit-mask-position-y",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal", "InitialValue"],
      interpolable: true,
      style_builder_template: "mask_layer",
      style_builder_template_args: {
        fill_type: "PositionY",
      },
      stored_on_extra_field: ["Mask"],
      percentages_depend_on_used_value: true,
      affected_by_zoom: true,
    },
    {
      name: "mask-repeat",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal", "InitialValue"],
      style_builder_template: "mask_layer",
      style_builder_template_args: {
        fill_type: "Repeat",
      },
      stored_on_extra_field: ["Mask"],
    },
    {
      name: "-webkit-mask-repeat",
      alias_for: "mask-repeat",
    },
    {
      name: "mask-size",
      property_methods: ["CSSValueFromComputedStyleInternal", "InitialValue"],
      parse_helper: "ParseMaskSize",
      interpolable: true,
      style_builder_template: "mask_layer",
      style_builder_template_args: {
        fill_type: "Size",
      },
      stored_on_extra_field: ["Mask"],
      percentages_depend_on_used_value: true,
      keywords: ["auto", "contain", "cover"],
      affected_by_zoom: true,
    },
    {
      name: "-webkit-mask-size",
      alias_for: "mask-size",
    },
    {
      name: "-webkit-perspective-origin-x",
      property_methods: ["ParseSingleValue"],
      style_builder_custom_functions: ["inherit"],
      interpolable: true,
      converter: "ConvertLength",
      computable: false,
      affected_by_all: false,
      // Overlaps with perspective-origin.
      legacy_overlapping: true,
      percentages_depend_on_used_value: true,
      affected_by_zoom: true,
    },
    {
      name: "-webkit-perspective-origin-y",
      property_methods: ["ParseSingleValue"],
      style_builder_custom_functions: ["inherit"],
      interpolable: true,
      converter: "ConvertLength",
      computable: false,
      affected_by_all: false,
      // Overlaps with perspective-origin.
      legacy_overlapping: true,
      percentages_depend_on_used_value: true,
      affected_by_zoom: true,
    },
    {
      name: "-webkit-rtl-ordering",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      independent: true,
      inherited: true,
      field_template: "keyword",
      keywords: ["logical", "visual"],
      default_value: "logical",
      name_for_methods: "RtlOrdering",
      setter: "SetRtlOrdering",
      type_name: "EOrder",
      invalidate: ["reshape"],
    },
    {
      name: "ruby-align",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      inherited: true,
      field_group: "*",
      field_template: "keyword",
      keywords: ["space-around", "start", "center", "space-between"],
      default_value: "space-around",
      valid_for_permission_element: true,
      invalidate: ["layout", "paint"],
    },
    {
      name: "ruby-overhang",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      inherited: true,
      field_group: "*",
      field_template: "keyword_custom",
      keywords: ["auto", "none", "spaces"],
      default_value: "auto",
      invalidate: ["layout", "paint"],
      runtime_flag: "CSSRubyOverhang",
    },
    {
      name: "-webkit-ruby-position",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      inherited: true,
      type_name: "RubyPosition",
      converter: "ConvertRubyPosition",
      surrogate_for: "ruby-position",
      keywords: ["before", "after"],
      valid_for_permission_element: true,
      stored_on_extra_field: ["crbug.com/40919412"],
    },
    {
      name: "ruby-position",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      inherited: true,
      field_group: "*",
      field_template: "keyword_custom",
      keywords: ["over", "under"],
      default_value: "over",
      type_name: "RubyPosition",
      converter: "ConvertRubyPosition",
      valid_for_first_line: true,
      valid_for_permission_element: true,
      invalidate: ["layout", "paint"],
    },
    {
      name: "-webkit-tap-highlight-color",
      property_methods: ["CSSValueFromComputedStyleInternal", "ColorIncludingFallback"],
      parse_helper: "ConsumeColor",
      inherited: true,
      field_group: "*",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/css/style_color.h",
                      "third_party/blink/renderer/core/layout/layout_theme.h"],
      type_name: "StyleColor",
      default_value: "StyleColor(LayoutTheme::GetTheme().TapHighlightColor())",
      converter: "ConvertStyleColor",
      includes_currentcolor: true,
    },
    {
      name: "-webkit-text-combine",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      inherited: true,
      surrogate_for: "text-combine-upright",
      stored_on_extra_field: ["crbug.com/40919412"],
      keywords: ["none", "horizontal"],
    },
    {
      name: "text-emphasis-color",
      property_methods: ["CSSValueFromComputedStyleInternal", "ColorIncludingFallback"],
      parse_helper: "ConsumeColor",
      interpolable: true,
      inherited: true,
      field_group: "*",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/css/style_color.h"],
      default_value: "StyleColor::CurrentColor()",
      type_name: "StyleColor",
      computed_style_protected_functions: ["getter"],
      keywords: ["currentcolor"],
      includes_currentcolor: true,
      typedom_types: ["Keyword"],
      converter: "ConvertStyleColor",
      style_builder_template: "color",
      valid_for_marker: true,
      valid_for_highlight: true,
      invalidate: ["color"],
    },
    {
      name: "text-emphasis-position",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      inherited: true,
      field_group: "*",
      field_size: 3,
      field_template: "primitive",
      default_value: "ComputedStyleInitialValues::InitialTextEmphasisPosition()",
      type_name: "TextEmphasisPosition",
      converter: "ConvertTextTextEmphasisPosition",
      computed_style_custom_functions: ["initial"],
      valid_for_marker: true,
      invalidate: ["layout", "paint"],
    },
    {
      name: "text-emphasis-style",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      inherited: true,
      style_builder_custom_functions: ["initial", "inherit", "value"],
      valid_for_marker: true,
      stored_on_extra_field: ["TextEmphasisFill", "TextEmphasisMark", "TextEmphasisCustomMark"],
      invalidate: ["reshape"],
    },
    {
      name: "-webkit-text-fill-color",
      property_methods: ["CSSValueFromComputedStyleInternal", "ColorIncludingFallback"],
      parse_helper: "ConsumeColor",
      interpolable: true,
      inherited: true,
      field_group: "*",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/css/style_color.h"],
      default_value: "StyleColor::CurrentColor()",
      type_name: "StyleColor",
      computed_style_protected_functions: ["getter"],
      converter: "ConvertStyleColor",
      style_builder_template: "color",
      invalidate: ["color"],
    },
    {
      name: "text-fit",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ConsumeTextFit",
      inherited: true,
      field_group: "*",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/style/text_fit.h"],
      default_value: "TextFit()",
      type_name: "TextFit",
      converter: "ConvertTextFit",
      invalidate: ["layout"],
      runtime_flag: "CssTextFit",
      percentages_depend_on_used_value: false,
      keywords: ["none", "shrink", "grow"],
    },
    {
      name: "-webkit-text-security",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      inherited: true,
      field_group: "*",
      field_template: "keyword",
      keywords: ["none", "disc", "circle", "square"],
      default_value: "none",
      invalidate: ["layout", "paint"],
    },
    {
      name: "-webkit-text-stroke-color",
      property_methods: ["CSSValueFromComputedStyleInternal", "ColorIncludingFallback"],
      parse_helper: "ConsumeColor",
      interpolable: true,
      inherited: true,
      field_group: "*",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/css/style_color.h"],
      default_value: "StyleColor::CurrentColor()",
      type_name: "StyleColor",
      computed_style_protected_functions: ["getter"],
      converter: "ConvertStyleColor",
      style_builder_template: "color",
      invalidate: ["color"],
      includes_currentcolor: true,
    },
    {
      name: "-webkit-text-stroke-width",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      inherited: true,
      field_group: "*",
      field_template: "primitive",
      default_value: "0",
      type_name: "float",
      converter: "ConvertTextStrokeWidth",
      invalidate: ["layout", "paint"],
      affected_by_zoom: true,
    },
    {
      name: "-webkit-transform-origin-x",
      property_methods: ["ParseSingleValue"],
      style_builder_custom_functions: ["inherit"],
      interpolable: true,
      converter: "ConvertLength",
      computable: false,
      affected_by_all: false,
      // Overlaps with transform-origin.
      legacy_overlapping: true,
      percentages_depend_on_used_value: true,
      affected_by_zoom: true,
    },
    {
      name: "-webkit-transform-origin-y",
      property_methods: ["ParseSingleValue"],
      style_builder_custom_functions: ["inherit"],
      interpolable: true,
      converter: "ConvertLength",
      computable: false,
      affected_by_all: false,
      // Overlaps with transform-origin.
      legacy_overlapping: true,
      percentages_depend_on_used_value: true,
      affected_by_zoom: true,
    },
    {
      name: "-webkit-transform-origin-z",
      property_methods: ["ParseSingleValue"],
      style_builder_custom_functions: ["inherit"],
      interpolable: true,
      converter: "ConvertComputedLength<float>",
      computable: false,
      affected_by_all: false,
      // Overlaps with transform-origin.
      legacy_overlapping: true,
      percentages_depend_on_used_value: true,
      affected_by_zoom: true,
    },
    {
      name: "-webkit-user-drag",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      field_group: "*",
      field_template: "keyword",
      keywords: ["auto", "none", "element"],
      default_value: "auto",
      valid_for_permission_element: true,
      invalidate: ["paint"],
    },
    {
      name: "-webkit-user-modify",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      computed_style_protected_functions: ["getter"],
      inherited: true,
      field_group: "*",
      field_template: "keyword",
      keywords: ["read-only", "read-write", "read-write-plaintext-only"],
      default_value: "read-only",
      affected_by_all: false,
      invalidate: ["paint"],
    },
    {
      name: "user-select",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      computed_style_protected_functions: ["getter"],
      inherited: true,
      field_group: "*",
      field_template: "keyword",
      keywords: ["auto", "none", "text", "all", "contain"],
      // `contain` is an experimental keyword and currenly behind the
      // CSSUserSelectContain flag.
      // Since it is not fully supported yet, we are not exposing it to
      // devtools.
      devtools_keywords: ["auto", "none", "text", "all"],
      typedom_types: ["Keyword"],
      default_value: "auto",
      valid_for_permission_element: true,
      invalidate: ["paint"],
    },
    {
      name: "white-space",
      longhands: [
        "white-space-collapse", "text-wrap-mode"
      ],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      valid_for_page_context: true,
    },
    {
      name: "white-space-collapse",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      inherited: true,
      field_template: "primitive",
      field_size: 2,  // Ensure this is in sync with `kWhiteSpaceCollapseBits`.
      type_name: "WhiteSpaceCollapse",
      keywords: ["collapse", "preserve", "preserve-breaks", "break-spaces"],
      default_value: "WhiteSpaceCollapse::kCollapse",
      include_paths: ["third_party/blink/renderer/core/css/white_space.h"],
      typedom_types: ["Keyword"],
      valid_for_cue: true,
      valid_for_marker: true,
      valid_for_page_context: true,
      invalidate: ["reshape"],
    },
    {
      name: "text-wrap",
      longhands: [
        "text-wrap-mode", "text-wrap-style"
      ],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      valid_for_page_context: true,
    },
    {
      name: "text-wrap-mode",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      inherited: true,
      field_template: "keyword",
      type_name: "TextWrapMode",
      keywords: ["wrap", "nowrap"],
      default_value: "wrap",
      typedom_types: ["Keyword"],
      valid_for_cue: true,
      valid_for_marker: true,
      valid_for_page_context: true,
      invalidate: ["reshape"],
    },
    {
      name: "text-wrap-style",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      inherited: true,
      field_template: "keyword",
      type_name: "TextWrapStyle",
      keywords: ["auto", "balance", "pretty", "stable"],
      default_value: "auto",
      typedom_types: ["Keyword"],
      valid_for_cue: true,
      valid_for_marker: true,
      valid_for_page_context: true,
      invalidate: ["layout"],
    },
    {
      name: "widows",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ConsumePositiveInteger",
      interpolable: true,
      inherited: true,
      field_group: "*",
      field_template: "primitive",
      computed_style_custom_functions: ["setter"],
      default_value: "2",
      type_name: "short",
      typedom_types: ["Number"],
      invalidate: ["layout"],
    },
    {
      name: "width",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      layout_dependent: true,
      field_group: "box",
      field_template: "<length>",
      keywords: ["auto", "fit-content", "min-content", "max-content"],
      default_value: "Length()",
      typedom_types: ["Keyword", "Length", "Percentage"],
      converter: "ConvertLengthSizing",
      anchor_mode: "width",
      logical_property_group: {
        name: "size",
        resolver: "horizontal",
      },
      supports_incremental_style: true,
      valid_for_position_try: true,
      affected_by_zoom: true,
      valid_for_permission_element: true,
      valid_for_permission_icon: true,
      valid_for_page_context: true,
      invalidate: ["layout", "scroll-anchor"],
      percentages_depend_on_used_value: true,
    },
    {
      name: "will-change",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      style_builder_custom_functions: ["initial", "inherit", "value"],
      type_name: "StyleWillChangeData",
      field_template: "pointer",
      field_group: "misc",
      wrapper_pointer_name: "Member",
      default_value: "nullptr",
      include_paths: ["third_party/blink/renderer/core/style/style_will_change_data.h"],
      keywords: ["auto"],
      typedom_types: ["Keyword"],
      invalidate: ["compositing"],
      valid_for_permission_element: true,
      is_animation_affecting: true,
    },
    {
      name: "word-break",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      style_builder_custom_functions: ["value"],
      inherited: true,
      field_group: "*",
      field_template: "keyword",
      // Word Break Values. Matches WinIE and CSS3
      keywords: ["normal", "break-all", "keep-all", "break-word", "auto-phrase"],
      default_value: "normal",
      typedom_types: ["Keyword"],
      valid_for_marker: true,
      invalidate: ["layout", "paint"],
    },
    {
      name: "word-spacing",
      property_methods: ["CSSValueFromComputedStyleInternal"],
      parse_helper: "ParseSpacing",
      interpolable: true,
      inherited: true,
      converter: "ConvertSpacing",
      keywords: ["normal"],
      field_group: "inherited",
      field_template: "<length>",
      default_value: "Length::Fixed()",
      getter: "ComputedWordSpacing",
      computed_style_custom_functions: ["getter"],
      typedom_types: ["Keyword", "Length", "Percentage"],
      valid_for_first_letter: true,
      valid_for_first_line: true,
      valid_for_marker: true,
      valid_for_permission_element: true,
      valid_for_page_context: true,
      affected_by_zoom: true,
      // The field isn't actually used; this is currently stored in FontDescription.
      stored_on_extra_field: ["font"],
      percentages_depend_on_used_value: false,
    },
    {
      name: "z-index",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      interpolable: true,
      field_group: "box",
      field_template: "primitive",
      default_value: "0",
      type_name: "int",
      computed_style_custom_functions: ["setter"],
      style_builder_template: "auto",
      converter: "ConvertInteger",
      keywords: ["auto"],
      typedom_types: ["Keyword", "Number"],
      valid_for_permission_element: true,
      valid_for_page_context: true,
      invalidate: ["z-index"],
      stored_on_extra_field: ["HasAutoZIndex"],
    },

    // CSS logical props
    {
      name: "inline-size",
      parse_helper: "ConsumeWidthOrHeight",
      layout_dependent: true,
      logical_property_group: {
        name: "size",
        resolver: "inline",
      },
      keywords: ["auto"],
      typedom_types: ["Keyword", "Length", "Percentage"],
      valid_for_position_try: true,
      valid_for_permission_element: true,
      valid_for_page_context: true,
      percentages_depend_on_used_value: true,
    },
    {
      name: "block-size",
      parse_helper: "ConsumeWidthOrHeight",
      layout_dependent: true,
      logical_property_group: {
        name: "size",
        resolver: "block",
      },
      keywords: ["auto"],
      typedom_types: ["Keyword", "Length", "Percentage"],
      valid_for_position_try: true,
      valid_for_permission_element: true,
      valid_for_page_context: true,
      percentages_depend_on_used_value: true,
    },
    {
      name: "min-inline-size",
      property_methods: ["ParseSingleValue"],
      logical_property_group: {
        name: "min-size",
        resolver: "inline",
      },
      typedom_types: ["Length", "Percentage"],
      valid_for_position_try: true,
      valid_for_page_context: true,
      percentages_depend_on_used_value: true,
      keywords: ["auto", "min-content", "max-content", "fit-content", "stretch"],
    },
    {
      name: "min-block-size",
      property_methods: ["ParseSingleValue"],
      logical_property_group: {
        name: "min-size",
        resolver: "block",
      },
      typedom_types: ["Length", "Percentage"],
      valid_for_position_try: true,
      valid_for_permission_element: true,
      valid_for_page_context: true,
      percentages_depend_on_used_value: true,
      keywords: ["auto", "min-content", "max-content", "fit-content", "stretch"],
    },
    {
      name: "max-inline-size",
      parse_helper: "ConsumeMaxWidthOrHeight",
      logical_property_group: {
        name: "max-size",
        resolver: "inline",
      },
      keywords: ["none"],
      typedom_types: ["Keyword", "Length", "Percentage"],
      valid_for_position_try: true,
      valid_for_permission_element: true,
      valid_for_page_context: true,
      percentages_depend_on_used_value: true,
    },
    {
      name: "max-block-size",
      parse_helper: "ConsumeMaxWidthOrHeight",
      logical_property_group: {
        name: "max-size",
        resolver: "block",
      },
      keywords: ["none"],
      typedom_types: ["Keyword", "Length", "Percentage"],
      valid_for_position_try: true,
      valid_for_permission_element: true,
      valid_for_page_context: true,
      percentages_depend_on_used_value: true,
    },
    {
      name: "margin-inline-start",
      layout_dependent: true,
      property_methods: ["ParseSingleValue"],
      logical_property_group: {
        name: "margin",
        resolver: "inline-start",
      },
      typedom_types: ["Length", "Percentage"],
      keywords: ["auto"],
      valid_for_first_letter: true,
      valid_for_position_try: true,
      valid_for_permission_element: true,
      valid_for_page_context: true,
      percentages_depend_on_used_value: true,
    },
    {
      name: "margin-inline-end",
      layout_dependent: true,
      property_methods: ["ParseSingleValue"],
      logical_property_group: {
        name: "margin",
        resolver: "inline-end",
      },
      typedom_types: ["Length", "Percentage"],
      keywords: ["auto"],
      valid_for_first_letter: true,
      valid_for_position_try: true,
      valid_for_permission_element: true,
      valid_for_permission_icon: true,
      valid_for_page_context: true,
      percentages_depend_on_used_value: true,
    },
    {
      name: "margin-block-start",
      layout_dependent: true,
      property_methods: ["ParseSingleValue"],
      logical_property_group: {
        name: "margin",
        resolver: "block-start",
      },
      typedom_types: ["Length", "Percentage"],
      keywords: ["auto"],
      valid_for_first_letter: true,
      valid_for_position_try: true,
      valid_for_permission_element: true,
      valid_for_page_context: true,
      percentages_depend_on_used_value: true,
    },
    {
      name: "margin-block-end",
      layout_dependent: true,
      property_methods: ["ParseSingleValue"],
      logical_property_group: {
        name: "margin",
        resolver: "block-end",
      },
      typedom_types: ["Length", "Percentage"],
      keywords: ["auto"],
      valid_for_first_letter: true,
      valid_for_position_try: true,
      valid_for_permission_element: true,
      valid_for_page_context: true,
      percentages_depend_on_used_value: true,
    },
    {
      name: "padding-inline-start",
      layout_dependent: true,
      property_methods: ["ParseSingleValue"],
      logical_property_group: {
        name: "padding",
        resolver: "inline-start",
      },
      typedom_types: ["Length", "Percentage"],
      valid_for_page_context: true,
      percentages_depend_on_used_value: true,
    },
    {
      name: "padding-inline-end",
      layout_dependent: true,
      property_methods: ["ParseSingleValue"],
      logical_property_group: {
        name: "padding",
        resolver: "inline-end",
      },
      typedom_types: ["Length", "Percentage"],
      valid_for_page_context: true,
      percentages_depend_on_used_value: true,
    },
    {
      name: "padding-block-start",
      layout_dependent: true,
      property_methods: ["ParseSingleValue"],
      logical_property_group: {
        name: "padding",
        resolver: "block-start",
      },
      typedom_types: ["Length", "Percentage"],
      valid_for_page_context: true,
      percentages_depend_on_used_value: true,
    },
    {
      name: "padding-block-end",
      layout_dependent: true,
      property_methods: ["ParseSingleValue"],
      logical_property_group: {
        name: "padding",
        resolver: "block-end",
      },
      typedom_types: ["Length", "Percentage"],
      valid_for_page_context: true,
      percentages_depend_on_used_value: true,
    },
    {
      name: "border-inline-start-width",
      property_methods: ["ParseSingleValue"],
      logical_property_group: {
        name: "border-width",
        resolver: "inline-start",
      },
      valid_for_first_letter: true,
      valid_for_permission_element: true,
      valid_for_page_context: true,
      keywords: ["medium", "thick", "thin"],
      typedom_types: ["Keyword", "Length"],
    },
    {
      name: "border-inline-start-style",
      logical_property_group: {
        name: "border-style",
        resolver: "inline-start",
      },
      valid_for_first_letter: true,
      valid_for_permission_element: true,
      valid_for_page_context: true,
      keywords: ["none", "hidden", "inset", "groove", "outset", "ridge",
                "dotted", "dashed", "solid", "double"],
      typedom_types: ["Keyword"],
    },
    {
      name: "border-inline-start-color",
      parse_helper: "ConsumeColor",
      logical_property_group: {
        name: "border-color",
        resolver: "inline-start",
      },
      valid_for_first_letter: true,
      valid_for_permission_element: true,
      valid_for_page_context: true,
      keywords: ["currentcolor"],
      typedom_types: ["Keyword"],
    },
    {
      name: "border-inline-end-width",
      property_methods: ["ParseSingleValue"],
      logical_property_group: {
        name: "border-width",
        resolver: "inline-end",
      },
      valid_for_first_letter: true,
      valid_for_permission_element: true,
      valid_for_page_context: true,
      keywords: ["medium", "thick", "thin"],
      typedom_types: ["Keyword", "Length"],
    },
    {
      name: "border-inline-end-style",
      logical_property_group: {
        name: "border-style",
        resolver: "inline-end",
      },
      valid_for_first_letter: true,
      valid_for_permission_element: true,
      valid_for_page_context: true,
      keywords: ["none", "hidden", "inset", "groove", "outset", "ridge",
                "dotted", "dashed", "solid", "double"],
      typedom_types: ["Keyword"],
    },
    {
      name: "border-inline-end-color",
      parse_helper: "ConsumeColor",
      logical_property_group: {
        name: "border-color",
        resolver: "inline-end",
      },
      valid_for_first_letter: true,
      valid_for_permission_element: true,
      valid_for_page_context: true,
      keywords: ["currentcolor"],
      typedom_types: ["Keyword"],
    },
    {
      name: "border-block-start-width",
      property_methods: ["ParseSingleValue"],
      logical_property_group: {
        name: "border-width",
        resolver: "block-start",
      },
      valid_for_first_letter: true,
      valid_for_permission_element: true,
      valid_for_page_context: true,
      keywords: ["medium", "thick", "thin"],
      typedom_types: ["Keyword", "Length"],
    },
    {
      name: "border-block-start-style",
      logical_property_group: {
        name: "border-style",
        resolver: "block-start",
      },
      valid_for_first_letter: true,
      valid_for_permission_element: true,
      valid_for_page_context: true,
      keywords: ["none", "hidden", "inset", "groove", "outset", "ridge",
                "dotted", "dashed", "solid", "double"],
      typedom_types: ["Keyword"],
    },
    {
      name: "border-block-start-color",
      parse_helper: "ConsumeColor",
      logical_property_group: {
        name: "border-color",
        resolver: "block-start",
      },
      valid_for_first_letter: true,
      valid_for_permission_element: true,
      valid_for_page_context: true,
      keywords: ["currentcolor"],
      typedom_types: ["Keyword"],
    },
    {
      name: "border-block-end-width",
      property_methods: ["ParseSingleValue"],
      logical_property_group: {
        name: "border-width",
        resolver: "block-end",
      },
      valid_for_first_letter: true,
      valid_for_permission_element: true,
      valid_for_page_context: true,
      keywords: ["medium", "thick", "thin"],
      typedom_types: ["Keyword", "Length"],
    },
    {
      name: "border-block-end-style",
      logical_property_group: {
        name: "border-style",
        resolver: "block-end",
      },
      valid_for_first_letter: true,
      valid_for_permission_element: true,
      valid_for_page_context: true,
      keywords: ["none", "hidden", "inset", "groove", "outset", "ridge",
                "dotted", "dashed", "solid", "double"],
      typedom_types: ["Keyword"],
    },
    {
      name: "border-block-end-color",
      parse_helper: "ConsumeColor",
      logical_property_group: {
        name: "border-color",
        resolver: "block-end",
      },
      valid_for_first_letter: true,
      valid_for_permission_element: true,
      valid_for_page_context: true,
      keywords: ["currentcolor"],
      typedom_types: ["Keyword"],
    },
    {
      name: "inset-inline-start",
      property_methods: ["ParseSingleValue"],
      logical_property_group: {
        name: "inset",
        resolver: "inline-start",
      },
      typedom_types: ["Length", "Percentage"],
      valid_for_position_try: true,
      layout_dependent: true,
      valid_for_permission_element: true,
      percentages_depend_on_used_value: true,
      keywords: ["auto"],
    },
    {
      name: "inset-inline-end",
      property_methods: ["ParseSingleValue"],
      logical_property_group: {
        name: "inset",
        resolver: "inline-end",
      },
      typedom_types: ["Length", "Percentage"],
      valid_for_position_try: true,
      layout_dependent: true,
      valid_for_permission_element: true,
      percentages_depend_on_used_value: true,
      keywords: ["auto"],
    },
    {
      name: "inset-block-start",
      property_methods: ["ParseSingleValue"],
      logical_property_group: {
        name: "inset",
        resolver: "block-start",
      },
      typedom_types: ["Length", "Percentage"],
      valid_for_position_try: true,
      layout_dependent: true,
      valid_for_permission_element: true,
      percentages_depend_on_used_value: true,
      keywords: ["auto"],
    },
    {
      name: "inset-block-end",
      property_methods: ["ParseSingleValue"],
      logical_property_group: {
        name: "inset",
        resolver: "block-end",
      },
      typedom_types: ["Length", "Percentage"],
      valid_for_position_try: true,
      layout_dependent: true,
      valid_for_permission_element: true,
      valid_for_page_context: true,
      percentages_depend_on_used_value: true,
      keywords: ["auto"],
    },
    {
      name: "border-start-start-radius",
      parse_helper: "ParseBorderRadiusCorner",
      logical_property_group: {
        name: "border-radius",
        resolver: "start-start",
      },
      typedom_types: ["Length", "Percentage"],
      valid_for_first_letter: true,
      valid_for_permission_element: true,
      valid_for_page_context: true,
      percentages_depend_on_used_value: true,
    },
    {
      name: "border-start-end-radius",
      parse_helper: "ParseBorderRadiusCorner",
      logical_property_group: {
        name: "border-radius",
        resolver: "start-end",
      },
      typedom_types: ["Length", "Percentage"],
      valid_for_first_letter: true,
      valid_for_permission_element: true,
      valid_for_page_context: true,
      percentages_depend_on_used_value: true,
    },
    {
      name: "border-end-start-radius",
      parse_helper: "ParseBorderRadiusCorner",
      logical_property_group: {
        name: "border-radius",
        resolver: "end-start",
      },
      typedom_types: ["Length", "Percentage"],
      valid_for_first_letter: true,
      valid_for_permission_element: true,
      valid_for_page_context: true,
      percentages_depend_on_used_value: true,
    },
    {
      name: "border-end-end-radius",
      parse_helper: "ParseBorderRadiusCorner",
      logical_property_group: {
        name: "border-radius",
        resolver: "end-end",
      },
      typedom_types: ["Length", "Percentage"],
      valid_for_first_letter: true,
      valid_for_permission_element: true,
      valid_for_page_context: true,
      percentages_depend_on_used_value: true,
    },

    {
      name: "corner-start-start-shape",
      parse_helper: "ConsumeCornerShape",
      logical_property_group: {
        name: "corner-shape",
        resolver: "start-start",
      },
      valid_for_first_letter: true,
      valid_for_permission_element: true,
      valid_for_page_context: true,
    },
    {
      name: "corner-start-end-shape",
      parse_helper: "ConsumeCornerShape",
      logical_property_group: {
        name: "corner-shape",
        resolver: "start-end",
      },
      valid_for_first_letter: true,
      valid_for_permission_element: true,
      valid_for_page_context: true,
    },
    {
      name: "corner-end-start-shape",
      parse_helper: "ConsumeCornerShape",
      logical_property_group: {
        name: "corner-shape",
        resolver: "end-start",
      },
      valid_for_first_letter: true,
      valid_for_permission_element: true,
      valid_for_page_context: true,
    },
    {
      name: "corner-end-end-shape",
      parse_helper: "ConsumeCornerShape",
      logical_property_group: {
        name: "corner-shape",
        resolver: "end-end",
      },
      valid_for_first_letter: true,
      valid_for_permission_element: true,
      valid_for_page_context: true,
    },
    // Non-standard direction aware properties

    {
      name: "-webkit-border-end-color",
      alias_for: "border-inline-end-color",
    },
    {
      name: "-webkit-border-end-style",
      alias_for: "border-inline-end-style",
    },
    {
      name: "-webkit-border-end-width",
      alias_for: "border-inline-end-width",
    },
    {
      name: "-webkit-border-start-color",
      alias_for: "border-inline-start-color",
    },
    {
      name: "-webkit-border-start-style",
      alias_for: "border-inline-start-style",
    },
    {
      name: "-webkit-border-start-width",
      alias_for: "border-inline-start-width",
    },
    {
      name: "-webkit-border-before-color",
      alias_for: "border-block-start-color",
    },
    {
      name: "-webkit-border-before-style",
      alias_for: "border-block-start-style",
    },
    {
      name: "-webkit-border-before-width",
      alias_for: "border-block-start-width",
    },
    {
      name: "-webkit-border-after-color",
      alias_for: "border-block-end-color",
    },
    {
      name: "-webkit-border-after-style",
      alias_for: "border-block-end-style",
    },
    {
      name: "-webkit-border-after-width",
      alias_for: "border-block-end-width",
    },
    {
      name: "-webkit-margin-end",
      alias_for: "margin-inline-end",
    },
    {
      name: "-webkit-margin-start",
      alias_for: "margin-inline-start",
    },
    {
      name: "-webkit-margin-before",
      alias_for: "margin-block-start",
    },
    {
      name: "-webkit-margin-after",
      alias_for: "margin-block-end",
    },
    {
      name: "-webkit-padding-end",
      alias_for: "padding-inline-end",
    },
    {
      name: "-webkit-padding-start",
      alias_for: "padding-inline-start",
    },
    {
      name: "-webkit-padding-before",
      alias_for: "padding-block-start",
    },
    {
      name: "-webkit-padding-after",
      alias_for: "padding-block-end",
    },
    {
      name: "-webkit-logical-width",
      alias_for: "inline-size",
    },
    {
      name: "-webkit-logical-height",
      alias_for: "block-size",
    },
    {
      name: "-webkit-min-logical-width",
      alias_for: "min-inline-size",
    },
    {
      name: "-webkit-min-logical-height",
      alias_for: "min-block-size",
    },
    {
      name: "-webkit-max-logical-width",
      alias_for: "max-inline-size",
    },
    {
      name: "-webkit-max-logical-height",
      alias_for: "max-block-size",
    },
    {
      name: "-webkit-print-color-adjust",
      alias_for: "print-color-adjust",
    },

    // Properties that we ignore in the StyleBuilder.
    // TODO(timloh): This seems wrong, most of these shouldn't reach the
    // StyleBuilder
    {
      name: "all",
      affected_by_all: false,
      style_builder_template: "empty",
      computable: false,
      // Don't expand transition: all into all.
      is_animation_affecting: true,
    },
    // TODO(hjkim3323@gmail.com): Remove -internal-font-size-delta.
    // fontSizeDelta execCommand does not need separate CSS property.
    {
      name: "-internal-font-size-delta",
      property_methods: ["ParseSingleValue"],
      style_builder_template: "empty",
    },
    {
      name: "-webkit-text-decorations-in-effect",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      inherited: true,
      style_builder_template: "empty",
      stored_on_extra_field: ["crbug.com/40919412"],
      keywords: ["none", "blink", "line-through", "overline", "underline",
                "spelling-error", "grammar-error"],
    },

    // Descriptor only names
    {
      name: "font-display",
      is_descriptor: true,
      is_property: false,
    },
    {
      name: "src",
      is_descriptor: true,
      is_property: false,
    },
    {
      name: "unicode-range",
      is_descriptor: true,
      is_property: false,
    },
    {
      name: "syntax",
      is_descriptor: true,
      is_property: false,
    },
    {
      name: "initial-value",
      is_descriptor: true,
      is_property: false,
    },
    {
      name: "inherits",
      is_descriptor: true,
      is_property: false,
    },
    {
      name: "ascent-override",
      is_descriptor: true,
      is_property: false,
    },
    {
      name: "descent-override",
      is_descriptor: true,
      is_property: false,
    },
    {
      name: "line-gap-override",
      is_descriptor: true,
      is_property: false,
    },
    {
      name: "system",
      is_descriptor: true,
      is_property: false,
    },
    {
      name: "negative",
      is_descriptor: true,
      is_property: false,
    },
    {
      name: "prefix",
      is_descriptor: true,
      is_property: false,
    },
    {
      name: "suffix",
      is_descriptor: true,
      is_property: false,
    },
    {
      name: "range",
      is_descriptor: true,
      is_property: false,
    },
    {
      name: "pad",
      is_descriptor: true,
      is_property: false,
    },
    {
      name: "fallback",
      is_descriptor: true,
      is_property: false,
    },
    {
      name: "symbols",
      is_descriptor: true,
      is_property: false,
    },
    {
      name: "additive-symbols",
      is_descriptor: true,
      is_property: false,
    },
    {
      name: "speak-as",
      is_descriptor: true,
      is_property: false,
    },
    {
      name: "size-adjust",
      is_descriptor: true,
      is_property: false,
    },
    {
      name: "base-palette",
      is_descriptor: true,
      is_property: false,
      valid_for_permission_element: true,
    },
    {
      name: "override-colors",
      is_descriptor: true,
      is_property: false,
    },
    {
      name: "navigation",
      is_descriptor: true,
      is_property: false,
    },
    {
      name: "types",
      is_descriptor: true,
      is_property: false,
    },
    {
      name: "result",
      is_descriptor: true,
      is_property: false,
      runtime_flag: "CSSFunctions",
    },
    {
      name: "base-url",
      is_descriptor: true,
      is_property: false,
      runtime_flag: "RouteMatching",
    },
    {
      name: "hash",
      is_descriptor: true,
      is_property: false,
      runtime_flag: "RouteMatching",
    },
    {
      name: "hostname",
      is_descriptor: true,
      is_property: false,
      runtime_flag: "RouteMatching",
    },
    {
      name: "pathname",
      is_descriptor: true,
      is_property: false,
      runtime_flag: "RouteMatching",
    },
    {
      name: "pattern",
      is_descriptor: true,
      is_property: false,
      runtime_flag: "RouteMatching",
    },
    {
      name: "port",
      is_descriptor: true,
      is_property: false,
      runtime_flag: "RouteMatching",
    },
    {
      name: "protocol",
      is_descriptor: true,
      is_property: false,
      runtime_flag: "RouteMatching",
    },
    {
      name: "search",
      is_descriptor: true,
      is_property: false,
      runtime_flag: "RouteMatching",
    },
    {
      name: "position-area",
      property_methods: ["ParseSingleValue", "CSSValueFromComputedStyleInternal"],
      style_builder_custom_functions: ["initial", "inherit", "value"],
      field_group: "*",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/style/position_area.h"],
      converter: "ConvertPositionArea",
      default_value: "PositionArea()",
      type_name: "PositionArea",
      keywords: [
        "none", "top", "bottom", "center", "left", "right", "x-start", "x-end",
        "y-start", "y-end", "start", "end", "self-start", "self-end", "span-all",
        "span-left", "span-right", "span-x-start", "span-x-end",
        "self-x-start", "self-x-end", "span-self-x-start", "span-self-x-end",
        "span-top", "span-bottom", "span-y-start", "span-y-end",
        "self-y-start", "self-y-end", "span-self-y-start", "span-self-y-end",
        "block-start", "block-end", "span-block-start", "span-block-end",
        "inline-start", "inline-end", "span-inline-start", "span-inline-end",
        "self-block-start", "self-block-end", "span-self-block-start",
        "span-self-block-end", "self-inline-start", "self-inline-end",
        "span-self-inline-start", "span-self-inline-end", "span-start",
        "span-end", "span-self-start", "span-self-end"
      ],
      typedom_keywords: ["none"],
      typedom_types: ["Keyword"],
      valid_for_position_try: true,
      valid_for_permission_element: true,
      // position-area needs to be applied before inset properties since
      // anchor() functions compute relative to the containing block that is
      // modified by position-area in AnchorEvaluatorImpl.
      priority: 1,
      invalidate: ["box-paint-property", "layout", "paint"],
    },
    // Shorthands
    {
      name: "animation",
      longhands: [
        "animation-duration", "animation-timing-function", "animation-delay",
        "animation-iteration-count", "animation-direction",
        "animation-fill-mode", "animation-play-state", "animation-name",
        "animation-timeline", "animation-range-start", "animation-range-end"
      ],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      supports_incremental_style: false,
      valid_for_keyframe: false,
      is_animation_affecting: true,
    },
    {
      name: "animation-range",
      longhands: [
        "animation-range-start", "animation-range-end",
      ],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      supports_incremental_style: false,
      valid_for_keyframe: false,
      is_animation_affecting: true,
    },
    {
      name: "background",
      longhands: [
        "background-image", "background-position-x", "background-position-y",
        "background-size", "background-repeat",
        "background-attachment", "background-origin", "background-clip",
        "background-color"
      ],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      supports_incremental_style: true,
      valid_for_page_context: true,
      devtools_keywords: ["none", "left", "right", "center", "top", "bottom",
        "repeat", "no-repeat", "repeat-x", "repeat-y", "round", "space",
        "scroll", "fixed", "local", "border-box", "padding-box", "content-box",
         "text", "border-area", "currentcolor"
      ],
    },
    {
      name: "background-position",
      longhands: ["background-position-x", "background-position-y"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      computable: true,
      supports_incremental_style: true,
      valid_for_page_context: true,
    },
    {
      name: "border",
      longhands: [
        "border-top-color", "border-top-style", "border-top-width",
        "border-right-color", "border-right-style", "border-right-width",
        "border-bottom-color", "border-bottom-style", "border-bottom-width",
        "border-left-color", "border-left-style", "border-left-width",
        "border-image-source", "border-image-slice", "border-image-width",
        "border-image-outset", "border-image-repeat"
      ],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      valid_for_page_context: true,
      devtools_keywords: ["none", "hidden", "inset", "groove", "outset", "ridge",
        "dotted", "dashed", "solid", "double", "thin", "medium", "thick",
        "currentcolor"
      ],
    },
    {
      name: "border-block",
      longhands: [
        "border-block-start-color", "border-block-start-style", "border-block-start-width",
        "border-block-end-color", "border-block-end-style", "border-block-end-width"
      ],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      valid_for_page_context: true,
    },
    {
      name: "border-block-color",
      longhands: ["border-block-start-color", "border-block-end-color"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      valid_for_page_context: true,
    },
    {
      name: "border-block-end",
      longhands: [
        "border-block-end-width", "border-block-end-style",
        "border-block-end-color"
      ],
      property_methods: ["ParseShorthand"],
      logical_property_group: {
        name: "border",
        resolver: "block-end",
      },
      valid_for_page_context: true,
    },
    {
      name: "border-block-start",
      longhands: [
        "border-block-start-width", "border-block-start-style",
        "border-block-start-color"
      ],
      property_methods: ["ParseShorthand"],
      logical_property_group: {
        name: "border",
        resolver: "block-start",
      },
      valid_for_page_context: true,
    },
    {
      name: "border-block-style",
      longhands: ["border-block-start-style", "border-block-end-style"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      valid_for_page_context: true,
    },
    {
      name: "border-block-width",
      longhands: ["border-block-start-width", "border-block-end-width"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      valid_for_page_context: true,
    },
    {
      name: "border-bottom",
      longhands: [
        "border-bottom-width", "border-bottom-style", "border-bottom-color"
      ],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      logical_property_group: {
        name: "border",
        resolver: "bottom",
      },
      valid_for_page_context: true,
    },
    {
      name: "border-color",
      longhands: [
        "border-top-color", "border-right-color", "border-bottom-color",
        "border-left-color"
      ],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      supports_incremental_style: true,
      valid_for_page_context: true,
    },
    {
      name: "border-image",
      longhands: [
        "border-image-source", "border-image-slice", "border-image-width",
        "border-image-outset", "border-image-repeat"
      ],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      valid_for_page_context: true,
      keywords: ["none", "repeat", "stretch", "space", "round"],
    },
    {
      name: "border-inline",
      longhands: [
        "border-inline-start-color", "border-inline-start-style", "border-inline-start-width",
        "border-inline-end-color", "border-inline-end-style", "border-inline-end-width"
      ],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      valid_for_page_context: true,
    },
    {
      name: "border-inline-color",
      longhands: ["border-inline-start-color", "border-inline-end-color"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      valid_for_page_context: true,
    },
    {
      name: "border-inline-end",
      longhands: [
        "border-inline-end-width", "border-inline-end-style",
        "border-inline-end-color"
      ],
      property_methods: ["ParseShorthand"],
      logical_property_group: {
        name: "border",
        resolver: "inline-end",
      },
      valid_for_page_context: true,
    },
    {
      name: "border-inline-start",
      longhands: [
        "border-inline-start-width", "border-inline-start-style",
        "border-inline-start-color"
      ],
      property_methods: ["ParseShorthand"],
      logical_property_group: {
        name: "border",
        resolver: "inline-start",
      },
      valid_for_page_context: true,
    },
    {
      name: "border-inline-style",
      longhands: ["border-inline-start-style", "border-inline-end-style"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      valid_for_page_context: true,
    },
    {
      name: "border-inline-width",
      longhands: ["border-inline-start-width", "border-inline-end-width"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      valid_for_page_context: true,
    },
    {
      name: "border-left",
      longhands: [
        "border-left-width", "border-left-style", "border-left-color"
      ],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      logical_property_group: {
        name: "border",
        resolver: "left",
      },
      valid_for_page_context: true,
    },
    {
      name: "border-radius",
      longhands: [
        "border-top-left-radius", "border-top-right-radius",
        "border-bottom-right-radius", "border-bottom-left-radius"
      ],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      valid_for_permission_element: true,
      valid_for_page_context: true,
    },
    {
      name: "border-right",
      longhands: [
        "border-right-width", "border-right-style", "border-right-color"
      ],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      logical_property_group: {
        name: "border",
        resolver: "right",
      },
      valid_for_page_context: true,
    },
    {
      name: "border-spacing",
      longhands: [
        "-webkit-border-horizontal-spacing", "-webkit-border-vertical-spacing"
      ],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
    },
    {
      name: "border-style",
      longhands: [
        "border-top-style", "border-right-style", "border-bottom-style",
        "border-left-style"
      ],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      keywords: ["none"],
      valid_for_page_context: true,
    },
    {
      name: "border-top",
      longhands: ["border-top-width", "border-top-style", "border-top-color"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      logical_property_group: {
        name: "border",
        resolver: "top",
      },
      valid_for_page_context: true,
    },
    {
      name: "border-width",
      longhands: [
        "border-top-width", "border-right-width", "border-bottom-width",
        "border-left-width"
      ],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      valid_for_page_context: true,
    },
    {
      name: "column-rule-inset-cap",
      longhands: ["column-rule-inset-cap-start", "column-rule-inset-cap-end"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
    },
    {
      name: "column-rule-inset-end",
      longhands: ["column-rule-inset-cap-end", "column-rule-inset-junction-end"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
    },
    {
      name: "column-rule-inset-start",
      longhands: ["column-rule-inset-cap-start", "column-rule-inset-junction-start"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
    },
    {
      name: "column-rule-inset-junction",
      longhands: ["column-rule-inset-junction-start", "column-rule-inset-junction-end"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
    },
    {
      name: "contain-intrinsic-inline-size",
      parse_helper: "ConsumeIntrinsicSizeLonghand",
      logical_property_group: {
        name: "contain-intrinsic-size",
        resolver: "inline",
      },
      typedom_types: ["Keyword", "Length"],
      keywords: ["none"],
    },
    {
      name: "contain-intrinsic-block-size",
      parse_helper: "ConsumeIntrinsicSizeLonghand",
      logical_property_group: {
        name: "contain-intrinsic-size",
        resolver: "block",
      },
      typedom_types: ["Keyword", "Length"],
      keywords: ["none"],
    },
    {
      name: "contain-intrinsic-size",
      longhands: [
        "contain-intrinsic-width", "contain-intrinsic-height"
      ],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      computable: true,
    },
    {
      name: "container",
      longhands: [
        "container-name", "container-type"
      ],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      devtools_keywords: ["none"],
    },
    {
      name: "corner-top-shape",
      longhands: [
        "corner-top-left-shape", "corner-top-right-shape"
      ],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      valid_for_permission_element: true,
      valid_for_page_context: true,
    },
    {
      name: "corner-right-shape",
      longhands: [
        "corner-top-right-shape", "corner-bottom-right-shape"
      ],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      valid_for_permission_element: true,
      valid_for_page_context: true,
    },
    {
      name: "corner-bottom-shape",
      longhands: [
        "corner-bottom-left-shape", "corner-bottom-right-shape"
      ],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      valid_for_permission_element: true,
      valid_for_page_context: true,
    },
    {
      name: "corner-left-shape",
      longhands: [
        "corner-top-left-shape", "corner-bottom-left-shape"
      ],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      valid_for_permission_element: true,
      valid_for_page_context: true,
    },
    {
      name: "corner-block-start-shape",
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      longhands: ["corner-start-start-shape", "corner-start-end-shape"],
      valid_for_permission_element: true,
      valid_for_page_context: true,
    },
    {
      name: "corner-block-end-shape",
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      longhands: ["corner-end-start-shape", "corner-end-end-shape"],
      valid_for_permission_element: true,
      valid_for_page_context: true,
    },
    {
      name: "corner-inline-start-shape",
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      longhands: ["corner-start-start-shape", "corner-end-start-shape"],
      valid_for_permission_element: true,
      valid_for_page_context: true,
    },
    {
      name: "corner-inline-end-shape",
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      longhands: ["corner-start-end-shape", "corner-end-end-shape"],
      valid_for_permission_element: true,
      valid_for_page_context: true,
    },
    {
      name: "corner-shape",
      longhands: [
        "corner-top-left-shape", "corner-top-right-shape",
        "corner-bottom-right-shape", "corner-bottom-left-shape"
      ],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      valid_for_permission_element: true,
      valid_for_page_context: true,
    },
    {
      name: "corner-top-left",
      longhands: ["border-top-left-radius", "corner-top-left-shape"],
      keywords: ["normal"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      runtime_flag: "CSSCornersShorthand",
      valid_for_permission_element: true,
      valid_for_page_context: true,
    },
    {
      name: "corner-top-right",
      longhands: ["border-top-right-radius", "corner-top-right-shape"],
      keywords: ["normal"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      runtime_flag: "CSSCornersShorthand",
      valid_for_permission_element: true,
      valid_for_page_context: true,
    },
    {
      name: "corner-bottom-left",
      longhands: ["border-bottom-left-radius", "corner-bottom-left-shape"],
      keywords: ["normal"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      runtime_flag: "CSSCornersShorthand",
      valid_for_permission_element: true,
      valid_for_page_context: true,
    },
    {
      name: "corner-bottom-right",
      longhands: ["border-bottom-right-radius", "corner-bottom-right-shape"],
      keywords: ["normal"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      runtime_flag: "CSSCornersShorthand",
      valid_for_permission_element: true,
      valid_for_page_context: true,
    },
    {
      name: "corner-start-start",
      longhands: ["border-start-start-radius", "corner-start-start-shape"],
      keywords: ["normal"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      runtime_flag: "CSSCornersShorthand",
      valid_for_permission_element: true,
      valid_for_page_context: true,
    },
    {
      name: "corner-start-end",
      longhands: ["border-start-end-radius", "corner-start-end-shape"],
      keywords: ["normal"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      runtime_flag: "CSSCornersShorthand",
      valid_for_permission_element: true,
      valid_for_page_context: true,
    },
    {
      name: "corner-end-start",
      longhands: ["border-end-start-radius", "corner-end-start-shape"],
      keywords: ["normal"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      runtime_flag: "CSSCornersShorthand",
      valid_for_permission_element: true,
      valid_for_page_context: true,
    },
    {
      name: "corner-end-end",
      longhands: ["border-end-end-radius", "corner-end-end-shape"],
      keywords: ["normal"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      runtime_flag: "CSSCornersShorthand",
      valid_for_permission_element: true,
      valid_for_page_context: true,
    },
    {
      name: "corner-top",
      longhands: [
        "border-top-left-radius", "corner-top-left-shape",
        "border-top-right-radius", "corner-top-right-shape"
      ],
      keywords: ["normal"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      runtime_flag: "CSSCornersShorthand",
      valid_for_permission_element: true,
      valid_for_page_context: true,
    },
    {
      name: "corner-right",
      longhands: [
        "border-top-right-radius", "corner-top-right-shape",
        "border-bottom-right-radius", "corner-bottom-right-shape"
      ],
      keywords: ["normal"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      runtime_flag: "CSSCornersShorthand",
      valid_for_permission_element: true,
      valid_for_page_context: true,
    },
    {
      name: "corner-bottom",
      longhands: [
        "border-bottom-left-radius", "corner-bottom-left-shape",
        "border-bottom-right-radius", "corner-bottom-right-shape"
      ],
      keywords: ["normal"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      runtime_flag: "CSSCornersShorthand",
      valid_for_permission_element: true,
      valid_for_page_context: true,
    },
    {
      name: "corner-left",
      longhands: [
        "border-top-left-radius", "corner-top-left-shape",
        "border-bottom-left-radius", "corner-bottom-left-shape"
      ],
      keywords: ["normal"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      runtime_flag: "CSSCornersShorthand",
      valid_for_permission_element: true,
      valid_for_page_context: true,
    },
    {
      name: "corner-block-start",
      longhands: [
        "border-start-start-radius", "corner-start-start-shape",
        "border-start-end-radius", "corner-start-end-shape"
      ],
      keywords: ["normal"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      runtime_flag: "CSSCornersShorthand",
      valid_for_permission_element: true,
      valid_for_page_context: true,
    },
    {
      name: "corner-block-end",
      longhands: [
        "border-end-start-radius", "corner-end-start-shape",
        "border-end-end-radius", "corner-end-end-shape"
      ],
      keywords: ["normal"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      runtime_flag: "CSSCornersShorthand",
      valid_for_permission_element: true,
      valid_for_page_context: true,
    },
    {
      name: "corner-inline-start",
      longhands: [
        "border-start-start-radius", "corner-start-start-shape",
        "border-end-start-radius", "corner-end-start-shape"
      ],
      keywords: ["normal"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      runtime_flag: "CSSCornersShorthand",
      valid_for_permission_element: true,
      valid_for_page_context: true,
    },
    {
      name: "corner-inline-end",
      longhands: [
        "border-start-end-radius", "corner-start-end-shape",
        "border-end-end-radius", "corner-end-end-shape"
      ],
      keywords: ["normal"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      runtime_flag: "CSSCornersShorthand",
      valid_for_permission_element: true,
      valid_for_page_context: true,
    },
    {
      name: "corner",
      longhands: [
        "border-top-left-radius", "corner-top-left-shape",
        "border-top-right-radius", "corner-top-right-shape",
        "border-bottom-right-radius", "corner-bottom-right-shape",
        "border-bottom-left-radius", "corner-bottom-left-shape"
      ],
      keywords: ["normal"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      runtime_flag: "CSSCornersShorthand",
      valid_for_permission_element: true,
      valid_for_page_context: true,
    },
    {
      name: "flex",
      longhands: ["flex-grow", "flex-shrink", "flex-basis"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      devtools_keywords: [
        "none", "auto", "content", "min-content", "max-content", "fit-content",
        "stretch"
      ],
    },
    {
      name: "flex-flow",
      longhands: ["flex-direction", "flex-wrap"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
    },
    {
      name: "font",
      longhands: [
        "font-style", "font-variant-ligatures", "font-variant-caps",
        "font-variant-numeric", "font-variant-east-asian", "font-variant-alternates",
        "font-variant-position", "font-variant-emoji", "font-weight", "font-stretch",
        "font-size", "line-height", "font-family", "font-optical-sizing",
        "font-size-adjust", "font-kerning", "font-feature-settings",
        "font-variation-settings", "font-language-override"
      ],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      // Changes to anything related to fonts require that we call style.UpdateFont(),
      // which the incremental path does not do.
      supports_incremental_style: false,
      devtools_keywords: [],
    },
    {
      name: "font-variant",
      longhands: [
        "font-variant-ligatures", "font-variant-caps", "font-variant-alternates",
        "font-variant-numeric", "font-variant-east-asian", "font-variant-position",
        "font-variant-emoji"
      ],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      is_descriptor: true,
      computable: true,
      // See comment on font.
      supports_incremental_style: false,
    },
    {
      name: "font-synthesis",
      longhands: ["font-synthesis-weight", "font-synthesis-style", "font-synthesis-small-caps"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      // See comment on font.
      supports_incremental_style: false,
      devtools_keywords: ["none"],
    },
    {
      name: "grid",
      longhands: [
        "grid-template-rows", "grid-template-columns", "grid-template-areas",
        "grid-auto-flow", "grid-auto-rows", "grid-auto-columns"
      ],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      layout_dependent: true,
      devtools_keywords: ["none"],
    },
    {
      name: "place-content",
      longhands: ["align-content", "justify-content"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
    },
    {
      name: "place-items",
      longhands: ["align-items", "justify-items"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
    },
    {
      name: "place-self",
      longhands: ["align-self", "justify-self"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      valid_for_position_try: true,
    },
    {
      name: "gap",
      longhands: ["row-gap", "column-gap"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
    },
    {
      name: "grid-area",
      longhands: [
        "grid-row-start", "grid-column-start", "grid-row-end",
        "grid-column-end"
      ],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
    },
    {
      name: "grid-column",
      longhands: ["grid-column-start", "grid-column-end"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
    },
    {
      name: "grid-lanes",
      longhands: [ "grid-template-areas", "grid-template-columns", "grid-template-rows", "grid-lanes-direction"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      layout_dependent: true,
      runtime_flag: "CSSGridLanesLayout",
    },
    {
      name: "grid-row",
      longhands: ["grid-row-start", "grid-row-end"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
    },
    {
      name: "grid-template",
      longhands: [
        "grid-template-rows", "grid-template-columns", "grid-template-areas"
      ],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      layout_dependent: true,
      devtools_keywords: ["none"],
    },
    {
      name: "inset",
      longhands: ["top", "right", "bottom", "left"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      valid_for_position_try: true,
      layout_dependent: true,
    },
    {
      name: "inset-block",
      longhands: ["inset-block-start", "inset-block-end"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      valid_for_position_try: true,
      layout_dependent: true,
    },
    {
      name: "inset-inline",
      longhands: ["inset-inline-start", "inset-inline-end"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      valid_for_position_try: true,
      layout_dependent: true,
    },
    {
      name: "list-style",
      longhands: ["list-style-position", "list-style-image", "list-style-type"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
    },
    {
      name: "margin",
      longhands: ["margin-top", "margin-right", "margin-bottom", "margin-left"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      layout_dependent: true,
      valid_for_position_try: true,
      valid_for_permission_element: true,
      valid_for_page_context: true,
      valid_for_permission_icon: true,
    },
    {
      name: "margin-block",
      longhands: ["margin-block-start", "margin-block-end"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      layout_dependent: true,
      valid_for_position_try: true,
      valid_for_page_context: true,
    },
    {
      name: "margin-inline",
      longhands: ["margin-inline-start", "margin-inline-end"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      layout_dependent: true,
      valid_for_position_try: true,
      valid_for_page_context: true,
    },
    {
      name: "marker",
      longhands: ["marker-start", "marker-mid", "marker-end"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
    },
    {
      name: "offset",
      longhands: [
        "offset-position", "offset-path", "offset-distance", "offset-rotate",
        "offset-anchor"
      ],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
    },
    {
      name: "outline",
      longhands: ["outline-color", "outline-style", "outline-width"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      valid_for_page_context: true,
      keywords: ["auto", "none", "inset", "groove", "ridge", "outset", "dotted",
                "dashed", "solid", "double", "medium", "thick", "thin"],
    },
    {
      name: "overflow",
      longhands: ["overflow-x", "overflow-y"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      // See comment on overflow-x.
      supports_incremental_style: false,
    },
    {
      name: "overscroll-behavior",
      longhands: ["overscroll-behavior-x", "overscroll-behavior-y"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      keywords: ["auto", "none", "contain"],
    },
    {
      name: "padding",
      longhands: [
        "padding-top", "padding-right", "padding-bottom", "padding-left"
      ],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      layout_dependent: true,
      supports_incremental_style: true,
      valid_for_page_context: true,
    },
    {
      name: "padding-block",
      longhands: ["padding-block-start", "padding-block-end"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      valid_for_page_context: true,
    },
    {
      name: "padding-inline",
      longhands: ["padding-inline-start", "padding-inline-end"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      valid_for_page_context: true,
    },
    {
      name: "page-break-after",
      longhands: ["break-after"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
    },
    {
      name: "page-break-before",
      longhands: ["break-before"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      keywords: ["auto", "left", "right", "always", "avoid"],
    },
    {
      name: "page-break-inside",
      longhands: ["break-inside"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
    },
    {
      name: "position-try",
      longhands: ["position-try-order", "position-try-fallbacks"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
    },
    {
      name: "row-rule-inset-cap",
      longhands: ["row-rule-inset-cap-start", "row-rule-inset-cap-end"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
    },
    {
      name: "row-rule-inset-end",
      longhands: ["row-rule-inset-cap-end", "row-rule-inset-junction-end"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
    },
    {
      name: "row-rule-inset-start",
      longhands: ["row-rule-inset-cap-start", "row-rule-inset-junction-start"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
    },
    {
      name: "row-rule-inset-junction",
      longhands: ["row-rule-inset-junction-start", "row-rule-inset-junction-end"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
    },
    {
      name: "rule",
      longhands: [
        "column-rule-width", "column-rule-style", "column-rule-color",
        "row-rule-width", "row-rule-style", "row-rule-color"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
    },
    {
      name: "rule-break",
      longhands: ["row-rule-break", "column-rule-break"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
    },
    {
      name: "rule-color",
      longhands: ["column-rule-color", "row-rule-color"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
    },
    {
      name: "rule-inset-cap",
      longhands: ["row-rule-inset-cap-start", "row-rule-inset-cap-end",
                  "column-rule-inset-cap-start", "column-rule-inset-cap-end"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
    },
    {
      name: "rule-inset-junction",
      longhands: ["row-rule-inset-junction-start", "row-rule-inset-junction-end",
                  "column-rule-inset-junction-start", "column-rule-inset-junction-end"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
    },
    {
      name: "rule-inset-end",
      longhands: ["column-rule-inset-cap-end", "column-rule-inset-junction-end",
                  "row-rule-inset-cap-end", "row-rule-inset-junction-end"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
    },
    {
      name: "rule-inset-start",
      longhands: ["column-rule-inset-cap-start", "column-rule-inset-junction-start",
                  "row-rule-inset-cap-start", "row-rule-inset-junction-start"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
    },
    {
      name: "column-rule-inset",
      longhands: ["column-rule-inset-cap-start", "column-rule-inset-cap-end", "column-rule-inset-junction-start",
                  "column-rule-inset-junction-end"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
    },
    {
      name: "row-rule-inset",
      longhands: ["row-rule-inset-cap-start", "row-rule-inset-cap-end", "row-rule-inset-junction-start",
                  "row-rule-inset-junction-end"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
    },
    {
      name: "rule-inset",
      longhands: ["row-rule-inset-cap-start", "row-rule-inset-cap-end", "row-rule-inset-junction-start",
                  "row-rule-inset-junction-end", "column-rule-inset-cap-start", "column-rule-inset-cap-end",
                  "column-rule-inset-junction-start", "column-rule-inset-junction-end"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
    },
    {
      name: "rule-width",
      longhands: ["column-rule-width", "row-rule-width"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
    },
    {
      name: "rule-style",
      longhands: ["column-rule-style", "row-rule-style"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
    },
    {
      name: "rule-visibility-items",
      longhands: ["column-rule-visibility-items", "row-rule-visibility-items"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
    },
    {
      name: "scroll-margin",
      longhands: ["scroll-margin-top", "scroll-margin-right", "scroll-margin-bottom", "scroll-margin-left"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
    },
    {
      name: "scroll-margin-block",
      longhands: ["scroll-margin-block-start", "scroll-margin-block-end"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
    },
    {
      name: "scroll-margin-inline",
      longhands: ["scroll-margin-inline-start", "scroll-margin-inline-end"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
    },
    {
      name: "scroll-padding",
      longhands: [
        "scroll-padding-top", "scroll-padding-right", "scroll-padding-bottom",
        "scroll-padding-left"
      ],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
    },
    {
      name: "scroll-padding-block",
      longhands: ["scroll-padding-block-start", "scroll-padding-block-end"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
    },
    {
      name: "scroll-padding-inline",
      longhands: ["scroll-padding-inline-start", "scroll-padding-inline-end"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
    },
    {
      name: "scroll-timeline",
      longhands: ["scroll-timeline-name", "scroll-timeline-axis"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
    },
    {
      name: "text-decoration",
      longhands: ["text-decoration-line", "text-decoration-thickness", "text-decoration-style", "text-decoration-color"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      computable: true,
      valid_for_page_context: true,
    },
    {
      name: "transition",
      longhands: [
        "transition-property", "transition-duration",
        "transition-timing-function", "transition-delay",
        "transition-behavior",
      ],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      // Animation properites are never incremental.
      supports_incremental_style: false,
      is_animation_affecting: true,
    },
    {
      name: "view-timeline",
      longhands: ["view-timeline-name", "view-timeline-axis", "view-timeline-inset"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      devtools_keywords: ["none"],
    },
    {
      name: "-webkit-border-after",
      alias_for: "border-block-end",
    },
    {
      name: "-webkit-border-before",
      alias_for: "border-block-start",
    },
    {
      name: "-webkit-border-end",
      alias_for: "border-inline-end",
    },
    {
      name: "-webkit-border-start",
      alias_for: "border-inline-start",
    },
    {
      name: "-webkit-column-break-after",
      longhands: ["break-after"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
    },
    {
      name: "-webkit-column-break-before",
      longhands: ["break-before"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
    },
    {
      name: "-webkit-column-break-inside",
      longhands: ["break-inside"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
    },
    {
      name: "column-rule",
      longhands: [
        "column-rule-width", "column-rule-style", "column-rule-color"
      ],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
    },
    {
      name: "row-rule",
      longhands: [
        "row-rule-width", "row-rule-style", "row-rule-color"
      ],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
    },
    {
      name: "columns",
      longhands: ["column-width", "column-count", "column-height", "column-wrap"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      devtools_keywords: ["auto"],
    },
    {
      name: "mask",
      longhands: [
        "mask-image", "-webkit-mask-position-x",
        "-webkit-mask-position-y", "mask-size", "mask-repeat", "mask-origin",
        "mask-clip", "mask-composite", "mask-mode"
      ],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      devtools_keywords: ["none", "left", "right", "center", "top", "bottom",
        "subtract", "intersect", "exclude", "alpha", "luminance", "match-source"
      ],
    },
    {
      name: "-webkit-mask",
      alias_for: "mask",
    },
    {
      name: "-webkit-mask-box-image",
      longhands: [
        "-webkit-mask-box-image-source", "-webkit-mask-box-image-slice",
        "-webkit-mask-box-image-width", "-webkit-mask-box-image-outset",
        "-webkit-mask-box-image-repeat"
      ],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      computable: true,
      devtools_keywords: ["none", "stretch", "repeat", "space", "round"],
    },
    {
      name: "mask-position",
      longhands: ["-webkit-mask-position-x", "-webkit-mask-position-y"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      computable: true,
    },
    {
      name: "-webkit-mask-position",
      alias_for: "mask-position",
    },
    {
      name: "text-emphasis",
      longhands: ["text-emphasis-style", "text-emphasis-color"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
    },
    {
      name: "timeline-trigger",
      longhands: [
        "timeline-trigger-name", "timeline-trigger-source",
        "timeline-trigger-activation-range-start", "timeline-trigger-activation-range-end",
        "timeline-trigger-active-range-start", "timeline-trigger-active-range-end",
      ],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      supports_incremental_style: false,
      valid_for_keyframe: false,
      runtime_flag: "TimelineTrigger",
    },
    {
      name: "timeline-trigger-activation-range",
      longhands: [
        "timeline-trigger-activation-range-start", "timeline-trigger-activation-range-end",
      ],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      supports_incremental_style: false,
      valid_for_keyframe: false,
      runtime_flag: "TimelineTrigger",
    },
    {
      name: "timeline-trigger-active-range",
      longhands: [
        "timeline-trigger-active-range-start", "timeline-trigger-active-range-end",
      ],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
      supports_incremental_style: false,
      valid_for_keyframe: false,
      runtime_flag: "TimelineTrigger",
    },
    {
      name: "-webkit-text-stroke",
      longhands: ["-webkit-text-stroke-width", "-webkit-text-stroke-color"],
      property_methods: ["ParseShorthand", "CSSValueFromComputedStyleInternal"],
    },

    // Visited properties.
    {
      name: "-internal-visited-color",
      visited_property_for: "color",
      property_methods: ["ColorIncludingFallback"],
      parse_helper: "ConsumeColorMaybeQuirky",
      inherited: true,
      field_group: "inherited",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/css/style_color.h"],
      default_value: "StyleColor(Color::kBlack)",
      type_name: "StyleColor",
      computed_style_protected_functions: ["getter"],
      style_builder_custom_functions: ["initial", "inherit", "value"],
      priority: 1,
      valid_for_first_letter: true,
      valid_for_first_line: true,
      valid_for_cue: true,
      valid_for_marker: true,
      valid_for_highlight: true,
      is_visited_highlight_colors: true,
      invalidate: ["border-visual", "color", "currentcolor"],
    },
    {
      name: "-internal-visited-caret-color",
      visited_property_for: "caret-color",
      property_methods: ["ParseSingleValue", "ColorIncludingFallback"],
      inherited: true,
      field_group: "inherited->inherited_visited",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/css/style_caret_color.h"],
      default_value: "StyleCaretColor()",
      type_name: "StyleCaretColor",
      converter: "ConvertStyleCaretColor",
      computed_style_protected_functions: ["getter"],
      style_builder_template: "visited_color",
      style_builder_template_args: {
        initial_color: "StyleCaretColor",
      },
      invalidate: ["color"],
    },
    {
      name: "-internal-visited-column-rule-color",
      visited_property_for: "column-rule-color",
      property_methods: ["ColorIncludingFallback"],
      parse_helper: "ConsumeColor",
      field_group: "misc->visited",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/style/gap_data_list.h",
                      "third_party/blink/renderer/core/css/style_color.h"],
      default_value: "GapDataList<StyleColor>::DefaultGapColorDataList()",
      type_name: "GapDataList<StyleColor>",
      converter: "ConvertGapDecorationColorDataList",
      invalidate: ["paint"],
    },
    {
      name: "-internal-visited-background-color",
      visited_property_for: "background-color",
      property_methods: ["ColorIncludingFallback"],
      parse_helper: "ConsumeColorMaybeQuirky",
      field_group: "misc->visited",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/css/style_color.h"],
      default_value: "StyleColor(Color::kTransparent)",
      type_name: "StyleColor",
      computed_style_protected_functions: ["getter"],
      converter: "ConvertStyleColor",
      style_builder_template: "visited_color",
      style_builder_template_args: {
        initial_color: "ComputedStyleInitialValues::InitialBackgroundColor",
      },
      valid_for_first_letter: true,
      valid_for_first_line: true,
      valid_for_cue: true,
      valid_for_highlight: true,
      is_visited_highlight_colors: true,
      invalidate: ["background-color"],
    },
    {
      name: "-internal-visited-border-left-color",
      visited_property_for: "border-left-color",
      property_methods: ["ColorIncludingFallback"],
      parse_helper: "ConsumeBorderColorSide",
      field_group: "misc->visited",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/css/style_color.h"],
      default_value: "StyleColor::CurrentColor()",
      type_name: "StyleColor",
      computed_style_protected_functions: ["getter"],
      converter: "ConvertStyleColor",
      style_builder_template: "visited_color",
      valid_for_first_letter: true,
      logical_property_group: {
        name: "visited-border-color",
        resolver: "left",
      },
      invalidate: ["border-outline-visited-color"],
    },
    {
      name: "-internal-visited-border-right-color",
      visited_property_for: "border-right-color",
      property_methods: ["ColorIncludingFallback"],
      parse_helper: "ConsumeBorderColorSide",
      field_group: "misc->visited",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/css/style_color.h"],
      default_value: "StyleColor::CurrentColor()",
      type_name: "StyleColor",
      computed_style_protected_functions: ["getter"],
      converter: "ConvertStyleColor",
      style_builder_template: "visited_color",
      valid_for_first_letter: true,
      logical_property_group: {
        name: "visited-border-color",
        resolver: "right",
      },
      invalidate: ["border-outline-visited-color"],
    },
    {
      name: "-internal-visited-border-top-color",
      visited_property_for: "border-top-color",
      property_methods: ["ColorIncludingFallback"],
      parse_helper: "ConsumeBorderColorSide",
      field_group: "misc->visited",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/css/style_color.h"],
      default_value: "StyleColor::CurrentColor()",
      type_name: "StyleColor",
      computed_style_protected_functions: ["getter"],
      converter: "ConvertStyleColor",
      style_builder_template: "visited_color",
      valid_for_first_letter: true,
      logical_property_group: {
        name: "visited-border-color",
        resolver: "top",
      },
      invalidate: ["border-outline-visited-color"],
    },
    {
      name: "-internal-visited-border-bottom-color",
      visited_property_for: "border-bottom-color",
      property_methods: ["ColorIncludingFallback"],
      parse_helper: "ConsumeBorderColorSide",
      field_group: "misc->visited",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/css/style_color.h"],
      default_value: "StyleColor::CurrentColor()",
      type_name: "StyleColor",
      computed_style_protected_functions: ["getter"],
      converter: "ConvertStyleColor",
      style_builder_template: "visited_color",
      valid_for_first_letter: true,
      logical_property_group: {
        name: "visited-border-color",
        resolver: "bottom",
      },
      invalidate: ["border-outline-visited-color"],
    },
    {
      name: "-internal-visited-border-inline-start-color",
      visited_property_for: "border-inline-start-color",
      parse_helper: "ConsumeBorderColorSide",
      logical_property_group: {
        name: "visited-border-color",
        resolver: "inline-start",
      },
      valid_for_first_letter: true,
    },
    {
      name: "-internal-visited-border-inline-end-color",
      visited_property_for: "border-inline-end-color",
      parse_helper: "ConsumeBorderColorSide",
      logical_property_group: {
        name: "visited-border-color",
        resolver: "inline-end",
      },
      valid_for_first_letter: true,
    },
    {
      name: "-internal-visited-border-block-start-color",
      visited_property_for: "border-block-start-color",
      parse_helper: "ConsumeBorderColorSide",
      logical_property_group: {
        name: "visited-border-color",
        resolver: "block-start",
      },
      valid_for_first_letter: true,
    },
    {
      name: "-internal-visited-border-block-end-color",
      visited_property_for: "border-block-end-color",
      parse_helper: "ConsumeBorderColorSide",
      logical_property_group: {
        name: "visited-border-color",
        resolver: "block-end",
      },
      valid_for_first_letter: true,
    },
    {
      name: "-internal-visited-fill",
      visited_property_for: "fill",
      property_methods: ["ColorIncludingFallback"],
      parse_helper: "ConsumeSVGPaint",
      inherited: true,
      field_group: "svginherited->fill",
      field_template: "external",
      type_name: "SVGPaint",
      include_paths: ["third_party/blink/renderer/core/style/svg_paint.h"],
      default_value: "SVGPaint(Color::kBlack)",
      name_for_methods: "InternalVisitedFillPaint",
      converter: "ConvertSVGPaint",
      style_builder_template: "visited_color",
      style_builder_template_args: {
        initial_color: "ComputedStyleInitialValues::InitialFillPaint",
      },
      valid_for_highlight: true,
    },
    {
      name: "-internal-visited-flood-color",
      visited_property_for: "flood-color",
      property_methods: ["ColorIncludingFallback"],
      parse_helper: "ConsumeColor",
      field_group: "misc->visited",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/css/style_color.h"],
      default_value: "StyleColor(Color::kBlack)",
      type_name: "StyleColor",
      style_builder_template: "visited_color",
      style_builder_template_args: {
        initial_color: "ComputedStyleInitialValues::InitialFloodColor",
      },
      converter: "ConvertStyleColor",
    },
    {
      name: "-internal-visited-lighting-color",
      visited_property_for: "lighting-color",
      property_methods: ["ColorIncludingFallback"],
      parse_helper: "ConsumeColor",
      field_group: "misc->visited",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/css/style_color.h"],
      default_value: "StyleColor(Color::kWhite)",
      type_name: "StyleColor",
      style_builder_template: "visited_color",
      style_builder_template_args: {
        initial_color: "ComputedStyleInitialValues::InitialLightingColor",
      },
      converter: "ConvertStyleColor",
    },
    {
      name: "-internal-visited-outline-color",
      visited_property_for: "outline-color",
      property_methods: ["ParseSingleValue", "ColorIncludingFallback"],
      field_group: "misc->visited",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/css/style_color.h"],
      default_value: "StyleColor::CurrentColor()",
      type_name: "StyleColor",
      computed_style_protected_functions: ["getter"],
      converter: "ConvertStyleColor",
      style_builder_template: "visited_color",
      valid_for_cue: true,
      invalidate: ["border-outline-visited-color"],
    },
    {
      name: "-internal-visited-stop-color",
      visited_property_for: "stop-color",
      property_methods: ["ColorIncludingFallback"],
      parse_helper: "ConsumeColor",
      field_group: "misc->visited",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/css/style_color.h"],
      default_value: "StyleColor(Color::kBlack)",
      type_name: "StyleColor",
      style_builder_template: "visited_color",
      style_builder_template_args: {
        initial_color: "ComputedStyleInitialValues::InitialStopColor",
      },
      converter: "ConvertStyleColor",
    },
    {
      name: "-internal-visited-stroke",
      visited_property_for: "stroke",
      property_methods: ["ColorIncludingFallback"],
      parse_helper: "ConsumeSVGPaint",
      inherited: true,
      field_group: "svginherited->stroke",
      field_template: "external",
      type_name: "SVGPaint",
      include_paths: ["third_party/blink/renderer/core/style/svg_paint.h"],
      default_value: "SVGPaint()",
      name_for_methods: "InternalVisitedStrokePaint",
      converter: "ConvertSVGPaint",
      style_builder_template: "visited_color",
      style_builder_template_args: {
        initial_color: "ComputedStyleInitialValues::InitialStrokePaint",
      },
      valid_for_highlight: true,
      invalidate: ["paint"],
    },
    {
      name: "-internal-visited-text-decoration-color",
      visited_property_for: "text-decoration-color",
      property_methods: ["ColorIncludingFallback"],
      parse_helper: "ConsumeColor",
      field_group: "misc->visited",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/css/style_color.h"],
      default_value: "StyleColor::CurrentColor()",
      type_name: "StyleColor",
      computed_style_protected_functions: ["getter"],
      converter: "ConvertStyleColor",
      style_builder_template: "visited_color",
      valid_for_first_letter: true,
      valid_for_first_line: true,
      valid_for_cue: true,
      valid_for_highlight: true,
      invalidate: ["color"],
    },
    {
      name: "-internal-visited-text-emphasis-color",
      visited_property_for: "text-emphasis-color",
      property_methods: ["ColorIncludingFallback"],
      parse_helper: "ConsumeColor",
      inherited: true,
      field_group: "inherited->inherited_visited",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/css/style_color.h"],
      default_value: "StyleColor::CurrentColor()",
      type_name: "StyleColor",
      computed_style_protected_functions: ["getter"],
      converter: "ConvertStyleColor",
      style_builder_template: "visited_color",
      valid_for_highlight: true,
      invalidate: ["color"],
    },
    {
      name: "-internal-visited-text-fill-color",
      visited_property_for: "-webkit-text-fill-color",
      property_methods: ["ColorIncludingFallback"],
      parse_helper: "ConsumeColor",
      inherited: true,
      field_group: "inherited->inherited_visited",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/css/style_color.h"],
      default_value: "StyleColor::CurrentColor()",
      type_name: "StyleColor",
      computed_style_protected_functions: ["getter"],
      converter: "ConvertStyleColor",
      style_builder_template: "visited_color",
      valid_for_highlight: true,
      invalidate: ["color"],
    },
    {
      name: "-internal-visited-text-stroke-color",
      visited_property_for: "-webkit-text-stroke-color",
      property_methods: ["ColorIncludingFallback"],
      parse_helper: "ConsumeColor",
      inherited: true,
      field_group: "inherited->inherited_visited",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/css/style_color.h"],
      default_value: "StyleColor::CurrentColor()",
      type_name: "StyleColor",
      computed_style_protected_functions: ["getter"],
      converter: "ConvertStyleColor",
      style_builder_template: "visited_color",
      valid_for_highlight: true,
      invalidate: ["color"],
    },

    // Forced colors properties.
    {
      name: "-internal-forced-background-color",
      property_methods: ["CSSValueFromComputedStyleInternal", "ColorIncludingFallback"],
      parse_helper: "ConsumeColorMaybeQuirky",
      field_group: "misc->forced_colors",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/css_value_keywords.h",
                      "third_party/blink/renderer/core/css/style_color.h"],
      default_value: "StyleColor(CSSValueID::kCanvas)",
      type_name: "StyleColor",
      computed_style_protected_functions: ["getter"],
      converter: "ConvertStyleColor",
      style_builder_template: "color",
      style_builder_template_args: {
        initial_color: "ComputedStyleInitialValues::InitialInternalForcedBackgroundColor",
      },
      valid_for_first_letter: true,
      valid_for_first_line: true,
      valid_for_cue: true,
    },
    {
      name: "-internal-forced-border-color",
      property_methods: ["CSSValueFromComputedStyleInternal", "ColorIncludingFallback"],
      parse_helper: "ConsumeColorMaybeQuirky",
      field_group: "misc->forced_colors",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/css/style_color.h"],
      default_value: "StyleColor::CurrentColor()",
      type_name: "StyleColor",
      computed_style_protected_functions: ["getter"],
      converter: "ConvertStyleColor",
      style_builder_template: "color",
      valid_for_first_letter: true,
    },
    {
      name: "-internal-forced-color",
      property_methods: ["CSSValueFromComputedStyleInternal", "ColorIncludingFallback"],
      parse_helper: "ConsumeColorMaybeQuirky",
      inherited: true,
      field_group: "inherited->inherited_forced_colors",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/css_value_keywords.h",
                      "third_party/blink/renderer/core/css/style_color.h"],
      default_value: "StyleColor(CSSValueID::kCanvastext)",
      type_name: "StyleColor",
      computed_style_protected_functions: ["getter"],
      style_builder_custom_functions: ["initial", "inherit", "value"],
      valid_for_first_letter: true,
      valid_for_first_line: true,
      valid_for_cue: true,
      valid_for_marker: true,
    },
    {
      name: "-internal-forced-outline-color",
      property_methods: ["CSSValueFromComputedStyleInternal", "ColorIncludingFallback"],
      parse_helper: "ConsumeColorMaybeQuirky",
      field_group: "misc->forced_colors",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/css/style_color.h"],
      default_value: "StyleColor::CurrentColor()",
      type_name: "StyleColor",
      computed_style_protected_functions: ["getter"],
      converter: "ConvertStyleColor",
      style_builder_template: "color",
      valid_for_cue: true,
    },
    {
      name: "-internal-forced-visited-color",
      visited_property_for: "-internal-forced-color",
      property_methods: ["ColorIncludingFallback"],
      parse_helper: "ConsumeColorMaybeQuirky",
      inherited: true,
      field_group: "inherited->inherited_forced_colors",
      field_template: "external",
      include_paths: ["third_party/blink/renderer/core/css_value_keywords.h",
                      "third_party/blink/renderer/core/css/style_color.h"],
      default_value: "StyleColor(CSSValueID::kCanvastext)",
      type_name: "StyleColor",
      computed_style_protected_functions: ["getter"],
      style_builder_custom_functions: ["initial", "inherit", "value"],
      valid_for_first_letter: true,
      valid_for_first_line: true,
      valid_for_cue: true,
      valid_for_marker: true,
    },

    // Name: -internal-empty-line-height:
    // Value: none | fabricated
    //    If the element is inline or contains visible text, this property has
    //    no effect.
    //
    // 'none'
    //   The box's intrinsic height is 0, and it defines no baseline.
    // 'fabricated'
    //   The box has intrinsic height and baseline, computed from the current
    //   font metrics.
    {
      name: "-internal-empty-line-height",
      property_methods: ["ParseSingleValue" ],
      inherited: true,
      field_group: "*",
      field_template: "primitive",
      type_name: "bool",
      default_value: "false",
      name_for_methods: "HasLineIfEmpty",
      converter: "ConvertInternalEmptyLineHeight",
    },
    {
      name: "-internal-align-content-block",
      property_methods: ["ParseSingleValue"],
      inherited: false,
      field_group: "*",
      field_template: "primitive",
      type_name: "bool",
      default_value: "false",
      name_for_methods: "AlignContentBlockCenter",
      converter: "ConvertInternalAlignContentBlock",
      invalidate: ["layout"],
    },
    // Aliases; these map to the same CSSPropertyID
    {
      name: "-epub-caption-side",
      alias_for: "caption-side",
    },
    {
      name: "-epub-text-combine",
      alias_for: "-webkit-text-combine",
    },
    {
      name: "-epub-text-emphasis",
      alias_for: "text-emphasis",
    },
    {
      name: "-epub-text-emphasis-color",
      alias_for: "text-emphasis-color",
    },
    {
      name: "-epub-text-emphasis-style",
      alias_for: "text-emphasis-style",
    },
    {
      name: "-epub-text-orientation",
      alias_for: "-webkit-text-orientation",
    },
    {
      name: "-epub-text-transform",
      alias_for: "text-transform",
    },
    {
      name: "-epub-word-break",
      alias_for: "word-break",
    },
    {
      name: "-epub-writing-mode",
      alias_for: "-webkit-writing-mode",
    },
    {
      name: "-webkit-align-content",
      alias_for: "align-content",
    },
    {
      name: "-webkit-align-items",
      alias_for: "align-items",
    },
    {
      name: "-webkit-align-self",
      alias_for: "align-self",
    },
    {
      name: "-webkit-animation",
      alias_for: "animation",
    },
    {
      name: "-webkit-animation-delay",
      alias_for: "animation-delay",
    },
    {
      name: "-webkit-animation-direction",
      alias_for: "animation-direction",
    },
    {
      name: "-webkit-animation-duration",
      alias_for: "animation-duration",
    },
    {
      name: "-webkit-animation-fill-mode",
      alias_for: "animation-fill-mode",
    },
    {
      name: "-webkit-animation-iteration-count",
      alias_for: "animation-iteration-count",
    },
    {
      name: "-webkit-animation-name",
      alias_for: "animation-name",
    },
    {
      name: "-webkit-animation-play-state",
      alias_for: "animation-play-state",
    },
    {
      name: "-webkit-animation-timing-function",
      alias_for: "animation-timing-function",
    },
    {
      name: "-webkit-backface-visibility",
      alias_for: "backface-visibility",
    },
    {
      name: "-webkit-background-clip",
      alias_for: "background-clip",
    },
    // -webkit-background-origin accepts "content", "padding", and "border"
    // values. See crbug.com/604023
    {
      name: "-webkit-background-origin",
      alias_for: "background-origin"
    },
    // "-webkit-background-size: 10px" behaves as "background-size: 10px 10px"
    {
      name: "-webkit-background-size",
      alias_for: "background-size",
    },
    {
      name: "-webkit-border-bottom-left-radius",
      alias_for: "border-bottom-left-radius",
    },
    {
      name: "-webkit-border-bottom-right-radius",
      alias_for: "border-bottom-right-radius",
    },
    // "-webkit-border-radius: 1px 2px" behaves as "border-radius: 1px / 2px"
    {
      name: "-webkit-border-radius",
      alias_for: "border-radius",
    },
    {
      name: "-webkit-border-top-left-radius",
      alias_for: "border-top-left-radius",
    },
    {
      name: "-webkit-border-top-right-radius",
      alias_for: "border-top-right-radius",
    },
    {
      name: "-webkit-box-shadow",
      alias_for: "box-shadow",
    },
    {
      name: "-webkit-box-sizing",
      alias_for: "box-sizing",
    },
    {
      name: "-webkit-clip-path",
      alias_for: "clip-path",
    },
    {
      name: "-webkit-column-count",
      alias_for: "column-count",
    },
    {
      name: "-webkit-column-gap",
      alias_for: "column-gap",
    },
    {
      name: "-webkit-column-rule",
      alias_for: "column-rule",
    },
    {
      name: "-webkit-column-rule-color",
      alias_for: "column-rule-color",
    },
    {
      name: "-webkit-column-rule-style",
      alias_for: "column-rule-style",
    },
    {
      name: "-webkit-column-rule-width",
      alias_for: "column-rule-width",
    },
    {
      name: "-webkit-column-span",
      alias_for: "column-span",
    },
    {
      name: "-webkit-column-width",
      alias_for: "column-width",
    },
    {
      name: "-webkit-columns",
      alias_for: "columns",
    },
    {
      name: "-webkit-filter",
      alias_for: "filter",
    },
    {
      name: "-webkit-flex",
      alias_for: "flex",
    },
    {
      name: "-webkit-flex-basis",
      alias_for: "flex-basis",
    },
    {
      name: "-webkit-flex-direction",
      alias_for: "flex-direction",
    },
    {
      name: "-webkit-flex-flow",
      alias_for: "flex-flow",
    },
    {
      name: "-webkit-flex-grow",
      alias_for: "flex-grow",
    },
    {
      name: "-webkit-flex-shrink",
      alias_for: "flex-shrink",
    },
    {
      name: "-webkit-flex-wrap",
      alias_for: "flex-wrap",
    },
    {
      name: "-webkit-font-feature-settings",
      alias_for: "font-feature-settings",
    },
    {
      name: "-webkit-hyphenate-character",
      alias_for: "hyphenate-character",
    },
    {
      name: "-webkit-justify-content",
      alias_for: "justify-content",
    },
    {
      name: "-webkit-opacity",
      alias_for: "opacity",
    },
    {
      name: "-webkit-order",
      alias_for: "order",
    },
    {
      name: "-webkit-perspective",
      alias_for: "perspective",
    },
    {
      name: "-webkit-perspective-origin",
      alias_for: "perspective-origin",
    },
    {
      name: "-webkit-shape-image-threshold",
      alias_for: "shape-image-threshold",
    },
    {
      name: "-webkit-shape-margin",
      alias_for: "shape-margin",
    },
    {
      name: "-webkit-shape-outside",
      alias_for: "shape-outside",
    },
    {
      name: "-webkit-text-emphasis",
      alias_for: "text-emphasis",
    },
    {
      name: "-webkit-text-emphasis-color",
      alias_for: "text-emphasis-color",
    },
    {
      name: "-webkit-text-emphasis-position",
      alias_for: "text-emphasis-position",
    },
    {
      name: "-webkit-text-emphasis-style",
      alias_for: "text-emphasis-style",
    },
    {
      name: "-webkit-text-size-adjust",
      alias_for: "text-size-adjust",
    },
    {
      name: "-webkit-transform",
      alias_for: "transform",
    },
    {
      name: "-webkit-transform-origin",
      alias_for: "transform-origin",
    },
    {
      name: "-webkit-transform-style",
      alias_for: "transform-style",
    },
    {
      name: "-webkit-transition",
      alias_for: "transition",
    },
    {
      name: "-webkit-transition-delay",
      alias_for: "transition-delay",
    },
    {
      name: "-webkit-transition-duration",
      alias_for: "transition-duration",
    },
    {
      name: "-webkit-transition-property",
      alias_for: "transition-property",
    },
    {
      name: "-webkit-transition-timing-function",
      alias_for: "transition-timing-function",
    },
    {
      name: "-webkit-user-select",
      alias_for: "user-select",
    },
    {
      name: "word-wrap",
      alias_for: "overflow-wrap",
    },
    {
      name: "grid-column-gap",
      alias_for: "column-gap",
    },
    {
      name: "grid-row-gap",
      alias_for: "row-gap",
    },
    {
      name: "grid-gap",
      alias_for: "gap",
    },
  ],
}
```

## line_breaker.cc

Original: [complete immutable source](https://chromium.googlesource.com/chromium/src/+/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/layout/inline/line_breaker.cc). Path: `third_party/blink/renderer/core/layout/inline/line_breaker.cc`. Source bytes: 194287; source lines: 4786; SHA-256: `c63cede1ffa6193fe708120e8970a6be9aeca58aaeb05bd2cda93130e3bd1d76`.

```cpp
// Copyright 2016 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#include "third_party/blink/renderer/core/layout/inline/line_breaker.h"

#include <algorithm>
#include <ranges>

#include "third_party/blink/renderer/core/frame/web_feature.h"
#include "third_party/blink/renderer/core/layout/block_break_token.h"
#include "third_party/blink/renderer/core/layout/constraint_space.h"
#include "third_party/blink/renderer/core/layout/constraint_space_builder.h"
#include "third_party/blink/renderer/core/layout/floats_utils.h"
#include "third_party/blink/renderer/core/layout/fragmentation_utils.h"
#include "third_party/blink/renderer/core/layout/inline/inline_break_token.h"
#include "third_party/blink/renderer/core/layout/inline/inline_cursor.h"
#include "third_party/blink/renderer/core/layout/inline/inline_item_result_ruby_column.h"
#include "third_party/blink/renderer/core/layout/inline/inline_item_segment.h"
#include "third_party/blink/renderer/core/layout/inline/inline_node.h"
#include "third_party/blink/renderer/core/layout/inline/line_break_candidate.h"
#include "third_party/blink/renderer/core/layout/inline/line_info.h"
#include "third_party/blink/renderer/core/layout/inline/ruby_utils.h"
#include "third_party/blink/renderer/core/layout/layout_object_inlines.h"
#include "third_party/blink/renderer/core/layout/layout_text_combine.h"
#include "third_party/blink/renderer/core/layout/length_utils.h"
#include "third_party/blink/renderer/core/layout/logical_fragment.h"
#include "third_party/blink/renderer/core/layout/physical_box_fragment.h"
#include "third_party/blink/renderer/core/layout/positioned_float.h"
#include "third_party/blink/renderer/core/layout/space_utils.h"
#include "third_party/blink/renderer/core/layout/svg/resolved_text_layout_attributes_iterator.h"
#include "third_party/blink/renderer/core/layout/unpositioned_float.h"
#include "third_party/blink/renderer/core/style/computed_style.h"
#include "third_party/blink/renderer/core/svg/svg_text_content_element.h"
#include "third_party/blink/renderer/platform/fonts/shaping/shape_result_view.h"
#include "third_party/blink/renderer/platform/fonts/shaping/shaping_line_breaker.h"
#include "third_party/blink/renderer/platform/runtime_enabled_features.h"
#include "third_party/blink/renderer/platform/text/bidi_paragraph.h"
#include "third_party/blink/renderer/platform/text/character.h"

namespace blink {

namespace {

inline bool ShouldApplyTextIndent(const ComputedStyle& style,
                                  bool is_first_formatted_line,
                                  bool is_after_forced_break) {
  if (style.TextIndent().IsZero()) {
    return false;
  }
  const bool is_first_line =
      is_first_formatted_line ||
      (is_after_forced_break && style.IsTextIndentEachLine());
  return !style.IsTextIndentHanging() ? is_first_line : !is_first_line;
}

inline LineBreakStrictness StrictnessFromLineBreak(LineBreak line_break) {
  switch (line_break) {
    case LineBreak::kAuto:
    case LineBreak::kAfterWhiteSpace:
    case LineBreak::kAnywhere:
      return LineBreakStrictness::kDefault;
    case LineBreak::kNormal:
      return LineBreakStrictness::kNormal;
    case LineBreak::kStrict:
      return LineBreakStrictness::kStrict;
    case LineBreak::kLoose:
      return LineBreakStrictness::kLoose;
  }
  NOTREACHED();
}

// Returns smallest negative left and right bearing in `box_fragment`.
// This function is used for calculating side bearing.
LineBoxStrut ComputeNegativeSideBearings(
    const PhysicalBoxFragment& box_fragment) {
  const auto get_shape_result =
      [](const InlineCursor cursor) -> const ShapeResultView* {
    if (!cursor)
      return nullptr;
    const FragmentItem& item = *cursor.CurrentItem();
    if (item.Type() != FragmentItem::kText &&
        item.Type() != FragmentItem::kGeneratedText) {
      return nullptr;
    }
    if (item.IsFlowControl())
      return nullptr;
    return item.TextShapeResult();
  };

  LineBoxStrut side_bearing;

  for (InlineCursor cursor(box_fragment); cursor; cursor.MoveToNextLine()) {
    // Take left/right bearing from the first/last child in the line if it has
    // `ShapeResult`. The first/last child can be non text item, e.g. image.
    // Note: Items in the line are in visual order. So, first=left, last=right.
    //
    // Example: If we have three text item "[", "T", "]", we should take left
    // baring from "[" and right bearing from "]". The text ink bounds of "T"
    // is not involved with side bearing calculation.
    DCHECK(cursor.Current().IsLineBox());

    // `gfx::RectF` returned from `ShapeResult::ComputeInkBounds()` is in
    // text origin coordinate aka baseline. Y-coordinate of points above
    // baseline are negative.
    //
    //  Text Ink Bounds:
    //   * left bearing = text_ink_bounds.X()
    //   * right bearing = width - text_ink_bounds.InlineEndOffset()
    //
    //          <--> left bearing (positive)
    //          ...+---------+
    //          ...|*********|..<
    //          ...|....*....|..<
    //          ...|....*....|<-> right bearing (positive)
    //          ...|....*....|..<
    //          ...|....*....|..<
    //          >..+----*----+..< baseline
    //          ^text origin
    //          <---------------> width/advance
    //
    //            left bearing (negative)
    //          <-->          <--> right bearing (negative)
    //          +----------------+
    //          |... *****..*****|
    //          |......*.....*<..|
    //          |.....*.....*.<..|
    //          |....*******..<..|
    //          |...*.....*...<..|
    //          |..*.....*....<..|
    //          +****..*****..<..+
    //             ^ text origin
    //             <----------> width/advance
    //
    // When `FragmentItem` has `ShapeTesult`, its `rect` is
    //    * `rect.offset.left = X`
    //    * `rect.size.width  = shape_result.SnappedWidth() // advance
    // where `X` is the original item offset.
    // For the initial letter text, its `rect` is[1]
    //    * `rect.offset.left = X - text_ink_bounds.X()`
    //    * `rect.size.width  = text_ink_bounds.Width()`
    // [1] https://drafts.csswg.org/css-inline/#initial-letter-box-size
    // Sizeing the Initial Letter Box
    InlineCursor child_at_left_edge = cursor;
    child_at_left_edge.MoveToFirstChild();
    if (auto* shape_result = get_shape_result(child_at_left_edge)) {
      const LayoutUnit left_bearing =
          LogicalRect::EnclosingRect(shape_result->ComputeInkBounds())
              .offset.inline_offset;
      side_bearing.inline_start =
          std::min(side_bearing.inline_start, left_bearing);
    }

    InlineCursor child_at_right_edge = cursor;
    child_at_right_edge.MoveToLastChild();
    if (auto* shape_result = get_shape_result(child_at_right_edge)) {
      const LayoutUnit width = shape_result->SnappedWidth();
      const LogicalRect text_ink_bounds =
          LogicalRect::EnclosingRect(shape_result->ComputeInkBounds());
      const LayoutUnit right_bearing =
          width - text_ink_bounds.InlineEndOffset();
      side_bearing.inline_end =
          std::min(side_bearing.inline_end, right_bearing);
    }
  }

  return side_bearing;
}

// This rule comes from the spec[1].
// Note: We don't apply inline kerning for vertical writing mode with text
// orientation other than `sideways` because characters are laid out vertically.
// [1] https://drafts.csswg.org/css-inline/#initial-letter-inline-position
bool ShouldApplyInlineKerning(const PhysicalBoxFragment& box_fragment) {
  if (!box_fragment.Borders().IsZero() || !box_fragment.Padding().IsZero())
    return false;
  const ComputedStyle& style = box_fragment.Style();
  return style.IsHorizontalWritingMode() ||
         style.GetTextOrientation() == ETextOrientation::kSideways;
}

// CSS-defined white space characters, excluding the newline character.
// In most cases, the line breaker consider break opportunities are before
// spaces because it handles trailing spaces differently from other normal
// characters, but breaking before newline characters is not desired.
inline bool IsBreakableSpace(UChar c) {
  return c == uchar::kSpace || c == uchar::kTab;
}

inline bool IsBreakableSpaceOrOtherSeparator(UChar c) {
  return IsBreakableSpace(c) || Character::IsOtherSpaceSeparator(c);
}

inline bool IsAllBreakableSpaces(const String& string,
                                 unsigned start,
                                 unsigned end) {
  DCHECK_GE(end, start);
  return StringView(string, start, end - start)
      .IsAllSpecialCharacters<IsBreakableSpace>();
}

inline bool IsBidiTrailingSpace(UChar c) {
  return u_charDirection(c) == UCharDirection::U_WHITE_SPACE_NEUTRAL;
}

inline LayoutUnit HyphenAdvance(const ComputedStyle& style,
                                bool is_ltr,
                                const HyphenResult& hyphen_result,
                                std::optional<LayoutUnit>& cache) {
  if (cache) {
    return *cache;
  }
  const LayoutUnit size = hyphen_result ? hyphen_result.InlineSize()
                                        : HyphenResult(style).InlineSize();
  const LayoutUnit advance = is_ltr ? size : -size;
  cache = advance;
  return advance;
}

// True if the item is "trailable". Trailable items should be included in the
// line if they are after the soft wrap point.
//
// Note that some items are ambiguous; e.g., text is trailable if it has leading
// spaces, and open tags are trailable if spaces follow. This function returns
// true for such cases.
inline bool IsTrailableItemType(InlineItem::InlineItemType type) {
  return type != InlineItem::kAtomicInline &&
         type != InlineItem::kOutOfFlowPositioned &&
         type != InlineItem::kInitialLetterBox &&
         type != InlineItem::kListMarker && type != InlineItem::kOpenRubyColumn;
}

inline bool CanBreakAfterLast(const InlineItemResults& item_results) {
  return !item_results.empty() && item_results.back().can_break_after;
}

inline bool ShouldCreateLineBox(const InlineItemResults& item_results) {
  return !item_results.empty() && item_results.back().should_create_line_box;
}

inline bool HasUnpositionedFloats(const InlineItemResults& item_results) {
  return !item_results.empty() && item_results.back().has_unpositioned_floats;
}

LayoutUnit ComputeInlineEndSize(const ConstraintSpace& space,
                                const ComputedStyle* style) {
  DCHECK(style);
  BoxStrut margins = ComputeMarginsForSelf(space, *style);
  BoxStrut borders = ComputeBordersForInline(*style);
  BoxStrut paddings = ComputePadding(space, *style);

  return margins.inline_end + borders.inline_end + paddings.inline_end;
}

bool NeedsAccurateEndPosition(const InlineItem& line_end_item) {
  DCHECK(line_end_item.Type() == InlineItem::kText ||
         line_end_item.Type() == InlineItem::kControl);
  DCHECK(line_end_item.Style());
  const ComputedStyle& line_end_style = *line_end_item.Style();
  return line_end_style.HasBoxDecorationBackground() ||
         line_end_style.HasAppliedTextDecorations();
}

inline bool NeedsAccurateEndPosition(const LineInfo& line_info,
                                     const InlineItem& line_end_item) {
  return line_info.NeedsAccurateEndPosition() ||
         NeedsAccurateEndPosition(line_end_item);
}

inline void ComputeCanBreakAfter(InlineItemResult* item_result,
                                 bool auto_wrap,
                                 const LazyLineBreakIterator& break_iterator) {
  item_result->can_break_after =
      auto_wrap && break_iterator.IsBreakable(item_result->EndOffset());
}

inline void RemoveLastItem(LineInfo* line_info) {
  InlineItemResults* item_results = line_info->MutableResults();
  DCHECK_GT(item_results->size(), 0u);
  item_results->Shrink(item_results->size() - 1);
}

// To correctly determine if a float is allowed to be on the same line as its
// content, we need to determine if it has any ancestors with inline-end
// padding, border, or margin.
// The inline-end size from all of these ancestors contribute to the "used
// size" of the float, and may cause the float to be pushed down.
LayoutUnit ComputeFloatAncestorInlineEndSize(const ConstraintSpace& space,
                                             const InlineItems& items,
                                             wtf_size_t item_index) {
  LayoutUnit inline_end_size;
  for (const Member<InlineItem>& item_ptr :
       base::span(items).subspan(item_index)) {
    const InlineItem& item = *item_ptr;
    if (item.Type() == InlineItem::kCloseTag) {
      inline_end_size += ComputeInlineEndSize(space, item.Style());
      continue;
    }

    // For this calculation, any open tag (even if its empty) stops this
    // calculation, and allows the float to appear on the same line. E.g.
    // <span style="padding-right: 20px;"><f></f><span></span></span>
    //
    // Any non-empty item also allows the float to be on the same line.
    if (item.Type() == InlineItem::kOpenTag || !item.IsEmptyItem()) {
      break;
    }
  }
  return inline_end_size;
}

// See LineBreaker::SplitTextIntoSegments().
void CollectCharIndex(void* context,
                      unsigned char_index,
                      Glyph,
                      gfx::Vector2dF,
                      float,
                      bool,
                      CanvasRotationInVertical,
                      const SimpleFontData*) {
  auto* index_list = static_cast<Vector<unsigned>*>(context);
  wtf_size_t size = index_list->size();
  if (size > 0 && index_list->at(size - 1) == char_index)
    return;
  index_list->push_back(char_index);
}

inline LayoutTextCombine* MayBeTextCombine(const InlineItem* item) {
  if (!item)
    return nullptr;
  return DynamicTo<LayoutTextCombine>(item->GetLayoutObject());
}

LayoutUnit MaxLineWidth(const LineInfo& base_line,
                        const HeapVector<LineInfo, 1>& annotation_lines) {
  LayoutUnit max = base_line.Width();
  for (const auto& line : annotation_lines) {
    max = std::max(max, line.Width());
  }
  return max;
}

// Represents data associated with an `InlineItemResult`.
class FastMinTextContext {
  STACK_ALLOCATED();

 public:
  LayoutUnit MinInlineSize() const { return min_inline_size_; }

  LayoutUnit HyphenInlineSize(InlineItemResult& item_result) const {
    if (!hyphen_inline_size_) {
      if (!item_result.hyphen) {
        item_result.ShapeHyphen();
      }
      hyphen_inline_size_ = item_result.hyphen.InlineSize();
    }
    return *hyphen_inline_size_;
  }

  void Add(LayoutUnit width) {
    min_inline_size_ = std::max(width, min_inline_size_);
  }

  // Add the width between the `start_offset` and the `end_offset`.
  void Add(const ShapeResult& shape_result,
           unsigned start_offset,
           unsigned end_offset,
           bool has_hyphen,
           InlineItemResult& item_result) {
    LayoutUnit width = shape_result.CachedWidth(start_offset, end_offset);
    if (has_hyphen) [[unlikely]] {
      const LayoutUnit hyphen_inline_size = HyphenInlineSize(item_result);
      width += hyphen_inline_size;
    }
    Add(width);
  }

  // Hyphenate the `word` and add all parts.
  void AddHyphenated(const ShapeResult& shape_result,
                     unsigned start_offset,
                     unsigned end_offset,
                     bool has_hyphen,
                     InlineItemResult& item_result,
                     const Hyphenation& hyphenation,
                     const StringView& word) {
    Vector<wtf_size_t, 8> locations = hyphenation.HyphenLocations(word);
    // |locations| is a list of hyphenation points in the descending order.
#if EXPENSIVE_DCHECKS_ARE_ON()
    DCHECK_EQ(word.length(), end_offset - start_offset);
    DCHECK(std::is_sorted(locations.rbegin(), locations.rend()));
    DCHECK(!locations.Contains(0u));
    DCHECK(!locations.Contains(word.length()));
#endif  // EXPENSIVE_DCHECKS_ARE_ON()
        // Append 0 to process all parts the same way.
    locations.push_back(0);
    const LayoutUnit hyphen_inline_size = HyphenInlineSize(item_result);
    LayoutUnit max_part_width;
    for (const wtf_size_t location : locations) {
      const unsigned part_start_offset = start_offset + location;
      LayoutUnit part_width =
          shape_result.CachedWidth(part_start_offset, end_offset);
      if (has_hyphen) {
        part_width += hyphen_inline_size;
      }
      max_part_width = std::max(part_width, max_part_width);
      end_offset = part_start_offset;
      has_hyphen = true;
    }
    Add(max_part_width);
  }

 private:
  LayoutUnit min_inline_size_;
  mutable std::optional<LayoutUnit> hyphen_inline_size_;
};

}  // namespace

bool LineBreaker::IsComputingContentSize() const {
  return constraint_space_.AvailableSize().inline_size == kIndefiniteSize;
}

inline bool LineBreaker::ShouldAutoWrap(const ComputedStyle& style) const {
  if (disallow_auto_wrap_) [[unlikely]] {
    return false;
  }
  return style.ShouldWrapLine();
}

void LineBreaker::UpdateAvailableWidth() {
  LayoutUnit available_width;
  if (override_available_width_) [[unlikely]] {
    // If we have an overridden width (e.g. because of text-wrap), the
    // line-clamp ellipsis should only cut into it when the overridden available
    // width plus the ellipsis width overflow the available inline size.
    if (line_clamp_ellipsis_width_) {
      available_width = std::min(
          line_opportunity_.AvailableInlineSize() - line_clamp_ellipsis_width_,
          override_available_width_);
    } else {
      available_width = override_available_width_;
    }
  } else {
    available_width =
        line_opportunity_.AvailableInlineSize() - line_clamp_ellipsis_width_;
  }
  // Make sure it's at least the initial size, which is usually 0 but not so
  // when `box-decoration-break: clone`.
  available_width =
      std::max(available_width, cloned_box_decorations_initial_size_);
  // Available width must be smaller than |LayoutUnit::Max()| so that the
  // position can be larger.
  available_width = std::min(available_width, LayoutUnit::NearlyMax());
  base_available_width_ = available_width;
  UpdateAvailableWidthFromBaseAvailableWidth();
}

inline void LineBreaker::UpdateAvailableWidthFromBaseAvailableWidth() {
  if (RuntimeEnabledFeatures::BoxDecorationBreakCloneLineBreakingEnabled() &&
      cloned_box_decorations_end_size_) [[unlikely]] {
    available_width_ = std::max(
        LayoutUnit(), base_available_width_ - cloned_box_decorations_end_size_);
  } else {
    available_width_ = base_available_width_;
  }
}

LineBreaker::LineBreaker(InlineNode node,
                         LineBreakerMode mode,
                         const ConstraintSpace& space,
                         const LineLayoutOpportunity& line_opportunity,
                         const LeadingFloats& leading_floats,
                         const InlineBreakToken* break_token,
                         const ColumnSpannerPath* column_spanner_path,
                         ExclusionSpace* exclusion_space)
    : line_opportunity_(line_opportunity),
      node_(node),
      mode_(mode),
      is_initial_letter_box_(node.IsInitialLetterBox()),
      is_svg_text_(node.IsSvgText()),
      is_text_combine_(node.IsTextCombine()),
      is_first_formatted_line_(
          (!break_token || !break_token->IsPastFirstFormattedLine()) &&
          node.CanContainFirstFormattedLine()),
      use_first_line_style_(is_first_formatted_line_ &&
                            node.UseFirstLineStyleItemsData()),
      sticky_images_quirk_(mode != LineBreakerMode::kContent &&
                           node.IsStickyImagesQuirkForContentSize()),
      items_data_(&node.ItemsData(use_first_line_style_)),
      end_item_index_(items_data_->items.size()),
      text_content_(
          !sticky_images_quirk_
              ? items_data_->text_content
              : InlineNode::TextContentForStickyImagesQuirk(*items_data_)),
      constraint_space_(space),
      exclusion_space_(exclusion_space),
      break_token_(break_token),
      column_spanner_path_(column_spanner_path),
      break_iterator_(text_content_),
      shaper_(text_content_),
      spacing_(text_content_, is_svg_text_),
      leading_floats_(leading_floats),
      base_direction_(node_.BaseDirection()) {
  UpdateAvailableWidth();
  if (is_svg_text_) {
    const auto& char_data_list = node_.SvgCharacterDataList();
    if (node_.SvgTextPathRangeList().empty() &&
        node_.SvgTextLengthRangeList().empty() &&
        (char_data_list.empty() ||
         (char_data_list.size() == 1 && char_data_list[0].first == 0))) {
      needs_svg_segmentation_ = false;
    } else {
      needs_svg_segmentation_ = true;
      svg_resolved_iterator_ =
          std::make_unique<ResolvedTextLayoutAttributesIterator>(
              char_data_list);
    }
  }
  // TODO(crbug.com/40362375): SVG <text> should not be auto_wrap_ for now.
  //
  // Combine text should not cause line break.
  //
  // TODO(crbug.com/40207613): Once we implement multiple line initial letter,
  // we should allow auto wrap. Below example causes multiple lines text in
  // initial letter box.
  //   <style>
  //    p::.first-letter { line-break: anywhere; }
  //    p { width: 0px; }
  //  </style>
  //  <p>(A) punctuation characters can be part of ::first-letter.</p>
  disallow_auto_wrap_ =
      is_svg_text_ || is_text_combine_ || is_initial_letter_box_;

  if (!break_token)
    return;

  const ComputedStyle* line_initial_style = break_token->Style();
  if (!line_initial_style) [[unlikely]] {
    // Usually an inline break token has the line initial style, but class C
    // breaks and last-resort breaks require a break token to start from the
    // beginning of the block. In that case, the line is still the first
    // formatted line, and the line initial style should be computed from the
    // containing block.
    DCHECK_EQ(break_token->StartItemIndex(), 0u);
    DCHECK_EQ(break_token->StartTextOffset(), 0u);
    DCHECK(!break_token->IsForcedBreak());
    DCHECK_EQ(current_, break_token->Start());
    DCHECK_EQ(is_forced_break_, break_token->IsForcedBreak());
    return;
  }

  current_ = break_token->Start();
  ruby_break_token_ = break_token->RubyData();
  break_iterator_.SetStartOffset(current_.text_offset);
  is_forced_break_ = break_token->IsForcedBreak();
  items_data_->AssertOffset(current_);
  SetCurrentStyle(*line_initial_style);
}

LineBreaker::~LineBreaker() = default;

void LineBreaker::SetLineOpportunity(
    const LineLayoutOpportunity& line_opportunity) {
  line_opportunity_ = line_opportunity;
  UpdateAvailableWidth();
}

void LineBreaker::OverrideAvailableWidth(LayoutUnit available_width) {
  DCHECK_GE(available_width, LayoutUnit());
  override_available_width_ = available_width;
  UpdateAvailableWidth();
}

void LineBreaker::SetBreakAt(const LineBreakPoint& offset) {
  break_at_ = offset;
  OverrideAvailableWidth(LayoutUnit::NearlyMax());
}

inline InlineItemResult* LineBreaker::AddItem(const InlineItem& item,
                                              unsigned end_offset,
                                              LineInfo* line_info) {
  if (item.Type() != InlineItem::kOpenRubyColumn) {
    DCHECK_EQ(&item, items_data_->items[current_.item_index]);
    DCHECK_GE(current_.text_offset, item.StartOffset());
    DCHECK_GE(end_offset, current_.text_offset);
    DCHECK_LE(end_offset, item.EndOffset());
  }
  if (item.IsTextCombine()) [[unlikely]] {
    line_info->SetHaveTextCombineOrRubyItem();
  }
  InlineItemResults* item_results = line_info->MutableResults();
  return &item_results->emplace_back(
      item, current_.item_index,
      TextOffsetRange(current_.text_offset, end_offset),
      break_anywhere_if_overflow_, ShouldCreateLineBox(*item_results),
      HasUnpositionedFloats(*item_results));
}

inline InlineItemResult* LineBreaker::AddItem(const InlineItem& item,
                                              LineInfo* line_info) {
  return AddItem(item, item.EndOffset(), line_info);
}

InlineItemResult* LineBreaker::AddEmptyItem(const InlineItem& item,
                                            LineInfo* line_info) {
  InlineItemResult* item_result =
      AddItem(item, current_.text_offset, line_info);

  // Prevent breaking before an empty item, but allow to break after if the
  // previous item had `can_break_after`.
  DCHECK(!item_result->can_break_after);
  if (line_info->Results().size() >= 2) {
    InlineItemResult* last_item_result = std::prev(item_result);
    if (last_item_result->can_break_after) {
      last_item_result->can_break_after = false;
      item_result->can_break_after = true;
    }
  }
  return item_result;
}

// Call |HandleOverflow()| if the position is beyond the available space.
inline bool LineBreaker::HandleOverflowIfNeeded(LineInfo* line_info) {
  if (state_ == LineBreakState::kContinue && !CanFitOnLine()) {
    HandleOverflow(line_info);
    return true;
  }
  return false;
}

void LineBreaker::SetIntrinsicSizeOutputs(
    MaxSizeCache* max_size_cache,
    bool* depends_on_block_constraints_out) {
  DCHECK_NE(mode_, LineBreakerMode::kContent);
  DCHECK(max_size_cache);
  max_size_cache_ = max_size_cache;
  depends_on_block_constraints_out_ = depends_on_block_constraints_out;
}

// Compute the base direction for bidi algorithm for this line.
void LineBreaker::ComputeBaseDirection() {
  // If 'unicode-bidi' is not 'plaintext', use the base direction of the block.
  if (node_.Style().GetUnicodeBidi() != UnicodeBidi::kPlaintext)
    return;

  const String& text = Text();
  if (text.Is8Bit())
    return;

  // If 'unicode-bidi: plaintext', compute the base direction for each
  // "paragraph" (separated by forced break.)
  wtf_size_t start_offset;
  if (previous_line_had_forced_break_) {
    start_offset = current_.text_offset;
  } else {
    // If this "paragraph" is at the beginning of the block, use
    // |node_.BaseDirection()|.
    if (!current_.text_offset) {
      return;
    }
    start_offset = text.rfind(uchar::kLineFeed, current_.text_offset - 1);
    if (start_offset == kNotFound)
      return;
    ++start_offset;
  }

  // LTR when no strong characters because `plaintext` uses P2 and P3 of UAX#9:
  // https://w3c.github.io/csswg-drafts/css-writing-modes-3/#valdef-unicode-bidi-plaintext
  // which sets to LTR if no strong characters.
  // https://unicode.org/reports/tr9/#P3
  base_direction_ = BidiParagraph::BaseDirectionForStringOrLtr(
      StringView(text, start_offset),
      // For CSS processing, line feed (U+000A) is treated as a segment break.
      // https://w3c.github.io/csswg-drafts/css-text-3/#segment-break
      Character::IsLineFeed);
}

inline LayoutUnit LineBreaker::ComputeFloatOffset() const {
  if (constraint_space_.AvailableSize().inline_size == kIndefiniteSize) {
    return LayoutUnit();
  }
  const LayoutUnit left = line_opportunity_.line_left_offset;
  const LayoutUnit bfc_left = constraint_space_.GetBfcOffset().line_offset;
  const LayoutUnit right = line_opportunity_.line_right_offset;
  const LayoutUnit bfc_right = constraint_space_.GetBfcOffset().line_offset +
                               constraint_space_.AvailableSize().inline_size;
  if (IsLtr(base_direction_)) {
    if (left <= bfc_left) {
      return LayoutUnit();
    }
    return left - bfc_left;
  } else {
    if (right >= bfc_right) {
      return LayoutUnit();
    }
    return bfc_right - right;
  }
}

void LineBreaker::RecalcClonedBoxDecorations() {
  cloned_box_decorations_count_ = 0u;
  cloned_box_decorations_initial_size_ = LayoutUnit();
  cloned_box_decorations_end_size_ = LayoutUnit();
  has_cloned_box_decorations_ = false;

  // Compute which tags are not closed at |current_.item_index|.
  InlineItemsData::OpenTagItems open_items;
  items_data_->GetOpenTagItems(0u, current_.item_index, &open_items);

  for (const InlineItem* item : open_items) {
    if (item->Style()->BoxDecorationBreak() == EBoxDecorationBreak::kClone) {
      has_cloned_box_decorations_ = true;
      disable_score_line_break_ = true;
      ++cloned_box_decorations_count_;
      InlineItemResult item_result;
      ComputeOpenTagResult(*item, constraint_space_, is_svg_text_,
                           &item_result);
      cloned_box_decorations_initial_size_ += item_result.inline_size;
      cloned_box_decorations_end_size_ += item_result.margins.inline_end +
                                          item_result.borders.inline_end +
                                          item_result.padding.inline_end;
    }
  }
  // Advance |position_| by the initial size so that the tab position can
  // accommodate cloned box decorations.
  position_ += cloned_box_decorations_initial_size_;
  // |cloned_box_decorations_initial_size_| may affect available width.
  UpdateAvailableWidth();
  DCHECK_GE(base_available_width_, cloned_box_decorations_initial_size_);
}

// Add a hyphen string to the |InlineItemResult|.
//
// This function changes |InlineItemResult::inline_size|, but does not change
// |position_|
LayoutUnit LineBreaker::AddHyphen(InlineItemResults* item_results,
                                  wtf_size_t index,
                                  InlineItemResult* item_result) {
  DCHECK(!HasHyphen());
  DCHECK_EQ(index, static_cast<wtf_size_t>(item_result - item_results->data()));
  DCHECK_LT(index, item_results->size());
  hyphen_index_ = index;

  if (!item_result->hyphen) {
    item_result->ShapeHyphen();
    has_any_hyphens_ = true;
  }
  DCHECK(item_result->hyphen);
  DCHECK(has_any_hyphens_);

  const LayoutUnit hyphen_inline_size = item_result->hyphen.InlineSize();
  item_result->inline_size += hyphen_inline_size;
  return hyphen_inline_size;
}

LayoutUnit LineBreaker::AddHyphen(InlineItemResults* item_results,
                                  wtf_size_t index) {
  InlineItemResult* item_result = &(*item_results)[index];
  DCHECK(item_result->item);
  return AddHyphen(item_results, index, item_result);
}

LayoutUnit LineBreaker::AddHyphen(InlineItemResults* item_results,
                                  InlineItemResult* item_result) {
  return AddHyphen(
      item_results,
      base::checked_cast<wtf_size_t>(item_result - item_results->data()),
      item_result);
}

// Remove the hyphen string from the |InlineItemResult|.
//
// This function changes |InlineItemResult::inline_size|, but does not change
// |position_|
LayoutUnit LineBreaker::RemoveHyphen(InlineItemResults* item_results) {
  DCHECK(HasHyphen());
  InlineItemResult* item_result = &(*item_results)[*hyphen_index_];
  DCHECK(item_result->hyphen);
  const LayoutUnit hyphen_inline_size = item_result->hyphen.InlineSize();
  item_result->inline_size -= hyphen_inline_size;
  // |hyphen_string| and |hyphen_shape_result| may be reused when rewinded.
  hyphen_index_.reset();
  return hyphen_inline_size;
}

// Add a hyphen string to the last inflow item in |item_results| if it is
// hyphenated. This can restore the hyphenation state after rewind.
void LineBreaker::RestoreLastHyphen(InlineItemResults* item_results) {
  DCHECK(!hyphen_index_);
  DCHECK(has_any_hyphens_);
  for (InlineItemResult& item_result : std::views::reverse(*item_results)) {
    DCHECK(item_result.item);
    if (item_result.hyphen) {
      AddHyphen(item_results, &item_result);
      return;
    }
    const InlineItem& item = *item_result.item;
    if (item.Type() == InlineItem::kText ||
        item.Type() == InlineItem::kAtomicInline) {
      return;
    }
  }
}

// Set the final hyphenation results to |item_results|.
void LineBreaker::FinalizeHyphen(InlineItemResults* item_results) {
  DCHECK(HasHyphen());
  InlineItemResult* item_result = &(*item_results)[*hyphen_index_];
  DCHECK(item_result->hyphen);
  item_result->is_hyphenated = true;
}

// Initialize internal states for the next line.
void LineBreaker::PrepareNextLine(LineInfo* line_info) {
  line_info->Reset();

  const InlineItemResults& item_results = line_info->Results();
  DCHECK(item_results.empty());

  if (parent_breaker_) {
    previous_line_had_forced_break_ =
        parent_breaker_->previous_line_had_forced_break_;
    is_forced_break_ = parent_breaker_->is_forced_break_;
    is_first_formatted_line_ = parent_breaker_->is_first_formatted_line_;
    use_first_line_style_ = parent_breaker_->use_first_line_style_;
    items_data_ = parent_breaker_->items_data_;
  } else if (!current_.IsZero()) {
    // We're past the first line
    previous_line_had_forced_break_ = is_forced_break_;
    is_forced_break_ = false;
    // If we resumed at a break token, and we're past the resume point stored
    // there, we're also past the first formatted line (otherwise, there may be
    // lines solely consisting of leading floats, and those don't count as
    // "formatted lines", since they aren't actually lines, as far as the spec
    // is concerned).
    if (!break_token_ || current_ != break_token_->Start()) {
      is_first_formatted_line_ = false;
      use_first_line_style_ = false;
    }
  }

  line_info->SetStart(current_);
  line_info->SetIsFirstFormattedLine(is_first_formatted_line_);
  line_info->SetIsStartOfParagraph(current_.IsZero() ||
                                   previous_line_had_forced_break_);
  line_info->SetLineStyle(node_, *items_data_, use_first_line_style_);

  DCHECK(!line_info->TextIndent());
  const ComputedStyle& style = line_info->LineStyle();
  if (ShouldApplyTextIndent(style, is_first_formatted_line_,
                            previous_line_had_forced_break_) &&
      !IsSubLineBreaker()) [[unlikely]] {
    const Length& length = style.TextIndent();
    LayoutUnit maximum_value;
    // Ignore percentages (resolve to 0) when calculating min/max intrinsic
    // sizes.
    if (length.HasPercent() && mode_ == LineBreakerMode::kContent) {
      maximum_value = constraint_space_.AvailableSize().inline_size;
    }
    line_info->SetTextIndent(MinimumValueForLength(length, maximum_value));
  }

  // Set the initial style of this line from the line style, if the style from
  // the end of previous line is not available. Example:
  //   <p>...<span>....</span></p>
  // When the line wraps in <span>, the 2nd line needs to start with the style
  // of the <span>.
  override_break_anywhere_ = false;
  disable_phrase_ = false;
  disable_score_line_break_ = false;
  disable_bisect_line_break_ = false;
  if (!current_style_)
    SetCurrentStyle(line_info->LineStyle());
  ComputeBaseDirection();
  line_info->SetBaseDirection(base_direction_);
  hyphen_index_.reset();
  has_any_hyphens_ = false;
  resume_block_in_inline_in_same_flow_ = false;

  // Use 'text-indent' as the initial position. This lets tab positions to align
  // regardless of 'text-indent'.
  applied_text_indent_ = line_info->TextIndent();
  position_ = applied_text_indent_;

  has_cloned_box_decorations_ = false;
  if ((break_token_ && break_token_->HasClonedBoxDecorations()) ||
      cloned_box_decorations_count_) [[unlikely]] {
    RecalcClonedBoxDecorations();
  }

  ResetRewindLoopDetector();
#if DCHECK_IS_ON()
  has_considered_creating_break_token_ = false;
#endif
}

void LineBreaker::NextLine(LineInfo* line_info) {
  PrepareNextLine(line_info);

  if (break_token_ && break_token_->IsInParallelBlockFlow()) {
    const auto* block_break_token = break_token_->GetBlockBreakToken();
    DCHECK(block_break_token);
    const InlineItem& item = *Items()[break_token_->StartItemIndex()];
    DCHECK_EQ(item.GetLayoutObject(),
              block_break_token->InputNode().GetLayoutBox());
    if (block_break_token->InputNode().IsFloating()) {
      HandleFloat(item, block_break_token, line_info);
    } else {
      // Overflowed block-in-inline, i.e. in a parallel flow.
      DCHECK_EQ(item.Type(), InlineItem::kBlockInInline);
      HandleBlockInInline(item, block_break_token, line_info);
    }

    state_ = LineBreakState::kDone;
    line_info->SetIsEmptyLine();
    return;
  }

  BreakLine(line_info);

  if (HasHyphen()) [[unlikely]] {
    FinalizeHyphen(line_info->MutableResults());
  }
  if (!disable_trailing_whitespace_collapsing_) {
    // TODO(abotella): Currently the behavior of the line-clamp ellipsis is
    // broken if the line ends with hanging spaces. Depending on the CSSWG
    // resolution, we might have to remove trailing hanging spaces if we're
    // line-clamping. See https://github.com/w3c/csswg-drafts/issues/12857
    RemoveTrailingCollapsibleSpace(line_info);
    SplitTrailingBidiPreservedSpace(line_info);
  }

  const InlineItemResults& item_results = line_info->Results();
#if DCHECK_IS_ON()
  for (const auto& result : item_results)
    result.CheckConsistency(mode_ == LineBreakerMode::kMinContent);
#endif

  // We should create a line-box when:
  //  - We have an item which needs a line box (text, etc).
  //  - A list-marker is present, and it would be the last line or last line
  //    before a forced new-line.
  //  - During min/max content sizing (to correctly determine the line width).
  //
  // With line-clamp, the entire contents of the line can be replaced by an
  // ellipsis. This can only happen when the non-displaced line would create a
  // line box, and the line box is still created after the replacement.
  //
  // TODO(kojii): There are cases where we need to PlaceItems() without creating
  // line boxes. These cases need to be reviewed.
  const bool should_create_line_box =
      ShouldCreateLineBox(item_results) ||
      (force_non_empty_if_last_line_ && line_info->IsLastLine()) ||
      mode_ != LineBreakerMode::kContent;

  // Don't add a line-clamp ellipsis if we're in an empty line or a
  // block-in-inline.
  if (line_clamp_ellipsis_width_ &&
      (!should_create_line_box || line_info->IsBlockInInline())) {
    line_clamp_ellipsis_width_ = LayoutUnit();
  }

  // If the line-clamp ellipsis would overflow, the entire line is replaced.
  if (line_clamp_ellipsis_width_ && !CanFitOnLine()) {
    Rewind(0, line_info);
    line_info->SetIsLastLine(false);

    // We only mark the line as overflowing (which matters for determining if
    // it gets pushed down by floats) if the ellipsis itself wouldn't fit.
    // We can't rely on AvailableWidth() here, since it gets clamped to be
    // non-negative after the ellipsis width gets subtracted, so we need to
    // check the line opportunity as well.
    bool has_overflow = !AvailableWidth() &&
                        line_opportunity_.AvailableInlineSize().AddEpsilon() <
                            line_clamp_ellipsis_width_;
    line_info->SetHasOverflow(has_overflow);

    // Overflow disables score and bisect line breaking, but if we fixed the
    // overflow by replacing the whole line, then we can reenable bisect.
    disable_bisect_line_break_ = has_overflow;
  }

  if (!should_create_line_box) {
    if (To<LayoutBlockFlow>(node_.GetLayoutBox())->HasLineIfEmpty())
      line_info->SetHasLineEvenIfEmpty();
    else
      line_info->SetIsEmptyLine();
  }

  line_info->SetEndItemIndex(current_.item_index);
  if (!disable_trailing_whitespace_collapsing_) {
    DCHECK_NE(trailing_whitespace_, WhitespaceState::kUnknown);
    if (trailing_whitespace_ == WhitespaceState::kPreserved) {
      line_info->SetHasTrailingSpaces();
    }
  }

  if (override_available_width_) [[unlikely]] {
    // Clear the overridden available width so that `line_info` has the original
    // available width for aligning.
    override_available_width_ = LayoutUnit();
    UpdateAvailableWidth();
  }
  ComputeLineLocation(line_info);
  DCHECK(!ruby_break_token_);
  const InlineItemResults& results = line_info->Results();
  if (!results.empty() && results.back().IsRubyColumn()) {
    ruby_break_token_ = results.back().ruby_column->end_ruby_break_token;
  }
  if (mode_ == LineBreakerMode::kContent) {
    line_info->SetBreakToken(CreateBreakToken(*line_info));
  }

#if EXPENSIVE_DCHECKS_ARE_ON()
  if (break_at_) [[unlikely]] {
    // If `break_at_` is set, the line should break `break_at_.offset`, but due
    // to minor differences in trailing spaces, it may not match exactly. It
    // should at least be beyond `break_at_.end`.
    DCHECK_GE(line_info->End(), break_at_.end);
  }
#endif  // EXPENSIVE_DCHECKS_ARE_ON()
}

void LineBreaker::BreakLine(LineInfo* line_info) {
  DCHECK(!line_info->IsLastLine());
  const InlineItems& items = Items();
  // If `kMinContent`, the line will overflow. Avoid calling `HandleOverflow()`
  // for the performance.
  if (mode_ == LineBreakerMode::kMinContent) [[unlikely]] {
    state_ = LineBreakState::kOverflow;
  } else {
    state_ = LineBreakState::kContinue;
  }
  trailing_whitespace_ = initial_whitespace_;

  while (state_ != LineBreakState::kDone) {
    if (ruby_break_token_) {
      HandleRuby(line_info);
      HandleOverflowIfNeeded(line_info);
      continue;
    }

    // If we reach at the end of the block, this is the last line.
    DCHECK_LE(current_.item_index, items.size());
    if (IsAtEnd()) {
      // Still check overflow because the last item may have overflowed.
      if (HandleOverflowIfNeeded(line_info) && !IsAtEnd()) {
        continue;
      }
      if (HasHyphen()) [[unlikely]] {
        position_ -= RemoveHyphen(line_info->MutableResults());
      }
      line_info->SetIsLastLine(true);
      return;
    }
    if (break_at_ && current_ >= break_at_.offset) [[unlikely]] {
      return;
    }

    // If |state_| is overflow, break at the earliest break opportunity.
    const InlineItemResults& item_results = line_info->Results();
    if (state_ == LineBreakState::kOverflow && CanBreakAfterLast(item_results))
        [[unlikely]] {
      state_ = LineBreakState::kTrailing;
    }

    // Handle trailable items first. These items may not be break before.
    // They (or part of them) may also overhang the available width.
    const InlineItem& item = *items[current_.item_index];
    if (item.Type() == InlineItem::kText) {
      if (item.Length())
        HandleText(item, *item.TextShapeResult(), line_info);
      else
        HandleEmptyText(item, line_info);
#if DCHECK_IS_ON()
      if (!item_results.empty())
        item_results.back().CheckConsistency(true);
#endif
      continue;
    }
    if (item.Type() == InlineItem::kOpenTag) {
      HandleOpenTag(item, line_info);
      continue;
    }
    if (item.Type() == InlineItem::kCloseTag) {
      HandleCloseTag(item, line_info);
      continue;
    }
    if (item.Type() == InlineItem::kControl) {
      HandleControlItem(item, line_info);
      continue;
    }
    if (item.Type() == InlineItem::kFloating) {
      HandleFloat(item, /* float_break_token */ nullptr, line_info);
      continue;
    }
    if (item.Type() == InlineItem::kBidiControl) {
      HandleBidiControlItem(item, line_info);
      continue;
    }
    if (item.Type() == InlineItem::kBlockInInline) {
      const BlockBreakToken* block_break_token =
          break_token_ ? break_token_->GetBlockBreakToken() : nullptr;
      HandleBlockInInline(item, block_break_token, line_info);
      continue;
    }
    if (item.Type() == InlineItem::kCloseRubyColumn ||
        item.Type() == InlineItem::kRubyLinePlaceholder) {
      AddItem(item, line_info);
      MoveToNextOf(item);
      continue;
    }

    // Items after this point are not trailable. If we're trailing, break before
    // any non-trailable items
    DCHECK(!IsTrailableItemType(item.Type()));
    if (state_ == LineBreakState::kTrailing) {
      DCHECK(!line_info->IsLastLine());
      return;
    }

    if (item.Type() == InlineItem::kAtomicInline) {
      HandleAtomicInline(item, line_info);
      continue;
    }
    if (item.Type() == InlineItem::kInitialLetterBox) [[unlikely]] {
      HandleInitialLetter(item, line_info);
      continue;
    }
    if (item.Type() == InlineItem::kOpenRubyColumn) {
      // Skip to call HandleRuby() for a placeholder-only ruby column.
      const wtf_size_t i = current_.item_index;
      if (items[i + 1]->Type() == InlineItem::kRubyLinePlaceholder &&
          (items[i + 2]->Type() == InlineItem::kCloseRubyColumn ||
           (items[i + 2]->Type() == InlineItem::kRubyLinePlaceholder &&
            items[i + 3]->Type() == InlineItem::kCloseRubyColumn))) {
        AddItem(item, line_info);
        MoveToNextOf(item);
        continue;
      }
      if (HandleRuby(line_info)) {
        HandleOverflowIfNeeded(line_info);
      } else {
        AddItem(item, line_info);
        MoveToNextOf(item);
      }
      continue;
    }
    if (item.Type() == InlineItem::kOutOfFlowPositioned) {
      HandleOutOfFlowPositioned(item, line_info);
    } else if (item.Length()) {
      NOTREACHED();
    } else if (item.Type() == InlineItem::kListMarker) {
      InlineItemResult* item_result = AddItem(item, line_info);
      force_non_empty_if_last_line_ = true;
      DCHECK(!item_result->can_break_after);
      MoveToNextOf(item);
    } else {
      NOTREACHED();
    }
  }
}

void LineBreaker::ComputeLineLocation(LineInfo* line_info) const {
  // Negative margins can make the position negative, but the inline size is
  // always positive or 0.
  LayoutUnit available_width = base_available_width_;
  line_info->SetWidth(available_width + line_clamp_ellipsis_width_,
                      position_ + cloned_box_decorations_end_size_ +
                          line_clamp_ellipsis_width_);
  line_info->SetBfcOffset(
      {line_opportunity_.line_left_offset, line_opportunity_.bfc_block_offset});
  if (mode_ == LineBreakerMode::kContent) {
    line_info->UpdateTextAlign();
  }
}

// Atomic inlines have break opportunities before and after, even when the
// adjacent character is U+00A0 NO-BREAK SPACE character, except when sticky
// images quirk is applied.
// Note: We treat text combine as text content instead of atomic inline box[1].
// [1] https://drafts.csswg.org/css-writing-modes-3/#text-combine-layout
bool LineBreaker::CanBreakAfterAtomicInline(const InlineItem& item) const {
  DCHECK(item.Type() == InlineItem::kAtomicInline ||
         item.Type() == InlineItem::kInitialLetterBox);
  if (!auto_wrap_) {
    return false;
  }
  if (item.EndOffset() == Text().length()) {
    return true;
  }
  // We can not break before sticky images quirk was applied.
  if (item.IsImage())
    return !sticky_images_quirk_;

  // Handles text combine
  // See "fast/writing-mode/text-combine-line-break.html".
  auto* const text_combine = MayBeTextCombine(&item);
  if (!text_combine) [[likely]] {
    return true;
  }

  // Populate |text_content| with |item| and text content after |item|.
  StringBuilder text_content;
  InlineNode(text_combine).PrepareLayoutIfNeeded();
  text_content.Append(text_combine->GetTextContent());
  const auto text_combine_end_offset = text_content.length();
  auto* const atomic_inline_item = TryGetAtomicInlineItemAfter(item);
  if (auto* next_text_combine = MayBeTextCombine(atomic_inline_item)) {
    // Note: In |LineBreakerMode::k{Min,Max}Content|, we've not laid
    // out atomic line box yet.
    InlineNode(next_text_combine).PrepareLayoutIfNeeded();
    text_content.Append(next_text_combine->GetTextContent());
  } else {
    text_content.Append(StringView(Text(), item.EndOffset(),
                                   Text().length() - item.EndOffset()));
  }

  DCHECK_EQ(Text(), break_iterator_.GetString());
  LazyLineBreakIterator break_iterator(break_iterator_,
                                       text_content.ReleaseString());
  return break_iterator.IsBreakable(text_combine_end_offset);
}

bool LineBreaker::CanBreakAfter(const InlineItem& item) const {
  DCHECK_NE(item.Type(), InlineItem::kAtomicInline);
  DCHECK(auto_wrap_);
  const bool can_break_after = break_iterator_.IsBreakable(item.EndOffset());
  if (item.Type() != InlineItem::kText) {
    DCHECK_EQ(item.Type(), InlineItem::kControl) << "We get the test case!";
    // Example: <div>12345\t\t678</div>
    //  InlineItem[0] kText "12345"
    //  InlineItem[1] kControl "\t\t"
    //  InlineItem[2] kText "678"
    // See LineBreakerTest.OverflowTab
    return can_break_after;
  }
  // Bidi controls produced by kOpenRubyColumn/kCloseRubyColumn are ignorable.
  unsigned ignorable_bidi_length = IgnorableBidiControlLength(item);
  if (ignorable_bidi_length > 0u) {
    return break_iterator_.IsBreakable(item.EndOffset() +
                                       ignorable_bidi_length);
  }
  auto* const atomic_inline_item = TryGetAtomicInlineItemAfter(item);
  if (!atomic_inline_item)
    return can_break_after;

  // We can not break before sticky images quirk was applied.
  if (Text()[atomic_inline_item->StartOffset()] == uchar::kNoBreakSpace)
      [[unlikely]] {
    // "One " <img> => We can break after "One ".
    // "One" <img> => We can not break after "One".
    // See "tables/mozilla/bugs/bug101674.html"
    DCHECK(atomic_inline_item->IsImage() && sticky_images_quirk_);
    return can_break_after;
  }

  // Handles text combine as its text contents followed by |item|.
  // See "fast/writing-mode/text-combine-line-break.html".
  auto* const text_combine = MayBeTextCombine(atomic_inline_item);
  if (!text_combine) [[likely]] {
    return true;
  }

  // Populate |text_content| with |item| and |text_combine|.
  // Following test reach here:
  //  * fast/writing-mode/text-combine-compress.html
  //  * virtual/text-antialias/international/text-combine-image-test.html
  //  * virtual/text-antialias/international/text-combine-text-transform.html
  StringBuilder text_content;
  text_content.Append(StringView(Text(), item.StartOffset(), item.Length()));
  const auto item_end_offset = text_content.length();
  // Note: In |LineBreakerMode::k{Min,Max}Content|, we've not laid out
  // atomic line box yet.
  InlineNode(text_combine).PrepareLayoutIfNeeded();
  text_content.Append(text_combine->GetTextContent());

  DCHECK_EQ(Text(), break_iterator_.GetString());
  LazyLineBreakIterator break_iterator(break_iterator_,
                                       text_content.ReleaseString());
  return break_iterator.IsBreakable(item_end_offset);
}

bool LineBreaker::MayBeAtomicInline(wtf_size_t offset) const {
  DCHECK_LT(offset, Text().length());
  const auto char_code = Text()[offset];
  if (char_code == uchar::kObjectReplacementCharacter) {
    return true;
  }
  return sticky_images_quirk_ && char_code == uchar::kNoBreakSpace;
}

const InlineItem* LineBreaker::TryGetAtomicInlineItemAfter(
    const InlineItem& item) const {
  DCHECK(auto_wrap_);
  const String& text = Text();
  if (item.EndOffset() == text.length())
    return nullptr;
  if (!MayBeAtomicInline(item.EndOffset()))
    return nullptr;

  // This kObjectReplacementCharacter can be any objects, such as a floating or
  // an OOF object. Check if it's really an atomic inline.
  for (const Member<InlineItem>& item_ptr :
       base::span(Items()).subspan(item.Index() + 1)) {
    const InlineItem& next_item = *item_ptr;
    DCHECK_EQ(next_item.StartOffset(), item.EndOffset());
    if (next_item.Type() == InlineItem::kAtomicInline) {
      return &next_item;
    }
    if (next_item.EndOffset() > item.EndOffset()) {
      return nullptr;
    }
  }
  return nullptr;
}

unsigned LineBreaker::IgnorableBidiControlLength(const InlineItem& item) const {
  size_t start_item_index = item.Index() + 1;
  for (const auto& item_i_ptr : base::span(Items()).subspan(
           start_item_index, end_item_index_ - start_item_index)) {
    const InlineItem& item_i = *item_i_ptr;
    if (item_i.Length() == 0u) {
      continue;
    }
    if (item_i.Type() != InlineItem::kOpenRubyColumn &&
        item_i.Type() != InlineItem::kCloseRubyColumn) {
      return item_i.StartOffset() - item.EndOffset();
    }
  }
  return (end_item_index_ >= Items().size()
              ? Text().length()
              : Items()[end_item_index_]->StartOffset()) -
         item.EndOffset();
}

void LineBreaker::HandleText(const InlineItem& item,
                             const ShapeResult& shape_result,
                             LineInfo* line_info) {
  DCHECK(item.Type() == InlineItem::kText ||
         (item.Type() == InlineItem::kControl &&
          Text()[item.StartOffset()] == uchar::kTab));
  DCHECK(&shape_result);
  DCHECK_EQ(auto_wrap_, ShouldAutoWrap(*item.Style()));

  // If we're trailing, only trailing spaces can be included in this line.
  if (state_ == LineBreakState::kTrailing) [[unlikely]] {
    HandleTrailingSpaces(item, &shape_result, line_info);
    return;
  }

  // Skip leading collapsible spaces.
  // Most cases such spaces are handled as trailing spaces of the previous line,
  // but there are some cases doing so is too complex.
  if (trailing_whitespace_ == WhitespaceState::kLeading) {
    if (item.Style()->ShouldCollapseWhiteSpaces() &&
        Text()[current_.text_offset] == uchar::kSpace) {
      // Skipping one whitespace removes all collapsible spaces because
      // collapsible spaces are collapsed to single space in
      // InlineItemBuilder.
      ++current_.text_offset;
      if (current_.text_offset == item.EndOffset()) {
        HandleEmptyText(item, line_info);
        return;
      }
    }
    // |trailing_whitespace_| will be updated as we read the text.
  }

  // Go to |HandleOverflow()| if the last item overflowed, and we're adding
  // text.
  if (state_ == LineBreakState::kContinue && !CanFitOnLine()) {
    // |HandleOverflow()| expects all trailable items are added. If this text
    // starts with trailable spaces, add them. TODO(kojii): This can be
    // optimzied further. This is necesasry only if |HandleOverflow()| does not
    // rewind, but in most cases it will rewind.
    const String& text = Text();
    if (auto_wrap_ && IsBreakableSpace(text[current_.text_offset])) {
      HandleTrailingSpaces(item, &shape_result, line_info);
      if (state_ != LineBreakState::kDone) {
        state_ = LineBreakState::kContinue;
        return;
      }
    }
    HandleOverflow(line_info);
    return;
  }

  if (HasHyphen()) [[unlikely]] {
    position_ -= RemoveHyphen(line_info->MutableResults());
  }

  // Try to commit |pending_end_overhang_| of a prior InlineItemResult.
  // |pending_end_overhang_| doesn't work well with bidi reordering. It's
  // difficult to compute overhang after bidi reordering because it affect
  // line breaking.
  if (maybe_have_end_overhang_) {
    position_ -= CommitPendingEndOverhang(item, shape_result, line_info);
  }

  InlineItemResult* item_result = nullptr;
  if (!is_svg_text_) {
    item_result = AddItem(item, line_info);
    item_result->should_create_line_box = true;
  }

  if (auto_wrap_) {
    // Check `parent_breaker_` because sub-LineInfo instances for <ruby>
    // require non-null InlineItemResult::shape_result.
    if (mode_ == LineBreakerMode::kMinContent && !parent_breaker_ &&
        HandleTextForFastMinContent(item_result, item, shape_result,
                                    line_info)) {
      return;
    }

    // Try to break inside of this text item.
    const LayoutUnit available_width = RemainingAvailableWidth();
    BreakResult break_result =
        BreakText(item_result, item, shape_result, available_width,
                  available_width, line_info);
    DCHECK(item_result->shape_result || !item_result->TextOffset().Length() ||
           (break_result == kOverflow && break_anywhere_if_overflow_ &&
            !override_break_anywhere_));
    position_ += item_result->inline_size;
    MoveToNextOf(*item_result);

    if (break_result == kSuccess) {
      DCHECK(item_result->shape_result || !item_result->TextOffset().Length());

      // If the break is at the middle of a text item, we know no trailable
      // items follow, only trailable spaces if any. This is very common that
      // shortcut to handling trailing spaces.
      if (item_result->EndOffset() < item.EndOffset())
        return HandleTrailingSpaces(item, &shape_result, line_info);

      // The break point found at the end of this text item. Continue looking
      // next items, because the next item maybe trailable, or can prohibit
      // breaking before.
      return;
    }
    if (break_result == kBreakAt) [[unlikely]] {
      // If this break is caused by `break_at_`, only trailing spaces or
      // trailing items can follow.
      if (item_result->EndOffset() < item.EndOffset()) {
        HandleTrailingSpaces(item, &shape_result, line_info);
        return;
      }
      state_ = LineBreakState::kTrailing;
      return;
    }
    DCHECK_EQ(break_result, kOverflow);

    // Handle `overflow-wrap` if it is enabled and if this text item overflows.
    if (!item_result->shape_result) [[unlikely]] {
      DCHECK(break_anywhere_if_overflow_ && !override_break_anywhere_);
      HandleOverflow(line_info);
      return;
    }

    // Hanging trailing spaces may resolve the overflow.
    if (item_result->has_only_pre_wrap_trailing_spaces) {
      state_ = LineBreakState::kTrailing;
      if (item_result->item->Style()->ShouldPreserveWhiteSpaces() &&
          IsBreakableSpace(Text()[item_result->EndOffset() - 1])) {
        unsigned end_index = base::checked_cast<unsigned>(
            item_result - line_info->Results().data());
        if (!parent_breaker_ || end_index > 0u) {
          Rewind(end_index, line_info);
        }
      }
      return;
    }

    // If we're seeking for the first break opportunity, update the state.
    if (state_ == LineBreakState::kOverflow) [[unlikely]] {
      if (item_result->can_break_after)
        state_ = LineBreakState::kTrailing;
      return;
    }

    // If this is all trailable spaces, this item is trailable, and next item
    // maybe too. Don't go to |HandleOverflow()| yet.
    if (IsAllBreakableSpaces(Text(), item_result->StartOffset(),
                             item_result->EndOffset()))
      return;

    HandleOverflow(line_info);
    return;
  }

  if (is_svg_text_) {
    SplitTextIntoSegments(item, line_info);
    return;
  }

  // Add until the end of the item if !auto_wrap. In most cases, it's the whole
  // item.
  DCHECK_EQ(item_result->EndOffset(), item.EndOffset());
  if (item_result->StartOffset() == item.StartOffset()) {
    item_result->inline_size =
        shape_result.SnappedWidth().ClampNegativeToZero();
    item_result->shape_result = ShapeResultView::Create(&shape_result);
  } else {
    // <wbr> can wrap even if !auto_wrap. Spaces after that will be leading
    // spaces and thus be collapsed.
    DCHECK(trailing_whitespace_ == WhitespaceState::kLeading &&
           item_result->StartOffset() >= item.StartOffset());
    item_result->shape_result = ShapeResultView::Create(
        &shape_result, item_result->StartOffset(), item_result->EndOffset());
    item_result->inline_size =
        item_result->shape_result->SnappedWidth().ClampNegativeToZero();
  }

  DCHECK(!item_result->may_break_inside);
  DCHECK(!item_result->can_break_after);
  trailing_whitespace_ = WhitespaceState::kUnknown;
  position_ += item_result->inline_size;
  MoveToNextOf(item);
}

// In SVG <text>, we produce InlineItemResult split into segments partitioned
// by x/y/dx/dy/rotate attributes.
//
// Split in PrepareLayout() or after producing FragmentItem would need
// additional memory overhead. So we split in LineBreaker while it converts
// InlineItems to InlineItemResults.
void LineBreaker::SplitTextIntoSegments(const InlineItem& item,
                                        LineInfo* line_info) {
  DCHECK(is_svg_text_);
  DCHECK_EQ(current_.text_offset, item.StartOffset());

  const ShapeResult& shape = *item.TextShapeResult();
  const unsigned num_glyphs = shape.NumGlyphs();
  if (num_glyphs == 0 || !needs_svg_segmentation_) {
    InlineItemResult* result = AddItem(item, line_info);
    result->should_create_line_box = true;
    result->shape_result = ShapeResultView::Create(&shape);
    result->inline_size = shape.SnappedWidth();
    current_.text_offset = item.EndOffset();
    position_ += result->inline_size;
    trailing_whitespace_ = WhitespaceState::kUnknown;
    MoveToNextOf(item);
    return;
  }

  Vector<unsigned> index_list;
  index_list.reserve(num_glyphs);
  shape.ForEachGlyph(0, CollectCharIndex, &index_list);
  if (shape.IsRtl())
    index_list.Reverse();
  wtf_size_t size = index_list.size();
  unsigned glyph_start = current_.text_offset;
  for (wtf_size_t i = 0; i < size; ++i) {
#if DCHECK_IS_ON()
    // The first glyph index can be greater than StartIndex() if the leading
    // part of the string was not mapped to any glyphs.
    if (i == 0)
      DCHECK_LE(glyph_start, index_list[0]);
    else
      DCHECK_EQ(glyph_start, index_list[i]);
#endif
    unsigned glyph_end = i + 1 < size ? index_list[i + 1] : shape.EndIndex();
    StringView text_view(Text());
    bool should_split = i == size - 1;
    for (; glyph_start < glyph_end;
         glyph_start = text_view.NextCodePointOffset(glyph_start)) {
      ++svg_addressable_offset_;
      should_split = should_split || ShouldCreateNewSvgSegment();
    }
    if (!should_split)
      continue;
    InlineItemResult* result = AddItem(item, glyph_end, line_info);
    result->should_create_line_box = true;
    auto* shape_result_view =
        ShapeResultView::Create(&shape, current_.text_offset, glyph_end);
    // For general CSS text, we apply SnappedWidth().ClampNegativeToZero().
    // However we need to remove ClampNegativeToZero() for SVG <text> in order
    // to get similar character positioning.
    //
    // For general CSS text, a negative word-spacing value decreases
    // inline_size of an InlineItemResult consisting of multiple characters,
    // and the inline_size rarely becomes negative.  However, for SVG <text>,
    // it decreases inline_size of an InlineItemResult consisting of only a
    // space character, and the inline_size becomes negative easily.
    //
    // See svg/W3C-SVG-1.1/text-spacing-01-b.svg.
    result->inline_size = shape_result_view->SnappedWidth();
    result->shape_result = std::move(shape_result_view);
    current_.text_offset = glyph_end;
    position_ += result->inline_size;
  }
  trailing_whitespace_ = WhitespaceState::kUnknown;
  MoveToNextOf(item);
}

bool LineBreaker::ShouldCreateNewSvgSegment() const {
  DCHECK(is_svg_text_);
  for (const auto& range : node_.SvgTextPathRangeList()) {
    if (range.start_index <= svg_addressable_offset_ &&
        svg_addressable_offset_ <= range.end_index)
      return true;
  }
  for (const auto& range : node_.SvgTextLengthRangeList()) {
    if (To<SVGTextContentElement>(range.layout_object->GetNode())
            ->lengthAdjust()
            ->CurrentEnumValue() == kSVGLengthAdjustSpacingAndGlyphs)
      continue;
    if (range.start_index <= svg_addressable_offset_ &&
        svg_addressable_offset_ <= range.end_index)
      return true;
  }
  const SvgCharacterData& char_data =
      svg_resolved_iterator_->AdvanceTo(svg_addressable_offset_);
  return char_data.HasRotate() || char_data.HasX() || char_data.HasY() ||
         char_data.HasDx() || char_data.HasDy();
}

LineBreaker::BreakResult LineBreaker::BreakText(
    InlineItemResult* item_result,
    const InlineItem& item,
    const ShapeResult& item_shape_result,
    LayoutUnit available_width,
    LayoutUnit available_width_with_hyphens,
    LineInfo* line_info) {
  DCHECK(item.Type() == InlineItem::kText ||
         (item.Type() == InlineItem::kControl &&
          Text()[item.StartOffset()] == uchar::kTab));
  DCHECK(&item_shape_result);
  item.AssertOffset(item_result->StartOffset());

  // The hyphenation state should be cleared before the entry. This function
  // may reset it, but this function cannot determine whether it should update
  // |position_| or not.
  DCHECK(!HasHyphen());

  DCHECK_EQ(item_shape_result.StartIndex(), item.StartOffset());
  DCHECK_EQ(item_shape_result.EndIndex(), item.EndOffset());
  class ShapingLineBreakerImpl : public ShapingLineBreaker {
    STACK_ALLOCATED();

   public:
    ShapingLineBreakerImpl(LineBreaker* line_breaker,
                           const InlineItem* item,
                           const ShapeResult* result)
        : ShapingLineBreaker(result,
                             &line_breaker->break_iterator_,
                             line_breaker->hyphenation_,
                             item->Style()->GetFont()),
          line_breaker_(line_breaker),
          item_(item) {}

   protected:
    const ShapeResult* Shape(wtf_size_t start,
                             wtf_size_t end,
                             ShapeOptions options) final {
      return line_breaker_->ShapeText(*item_, start, end, options);
    }

   private:
    LineBreaker* line_breaker_;
    const InlineItem* item_;

  } breaker(this, &item, &item_shape_result);

  const ComputedStyle& style = *item.Style();
  breaker.SetTextSpacingTrim(style.GetFontDescription().GetTextSpacingTrim());
  breaker.SetLineStart(line_info->StartOffset());
  breaker.SetIsAfterForcedBreak(previous_line_had_forced_break_);

  // Reshaping between the last character and trailing spaces is needed only
  // when we need accurate end position, because kerning between trailing spaces
  // is not visible.
  if (!NeedsAccurateEndPosition(*line_info, item))
    breaker.SetDontReshapeEndIfAtSpace();

  if (break_at_) [[unlikely]] {
    if (BreakTextAt(item_result, item, breaker, line_info)) {
      return kBreakAt;
    }
    return kSuccess;
  }

  // Use kNoResultIfOverflow if 'break-word' and we're trying to break normally
  // because if this item overflows, we will rewind and break line again. The
  // overflowing ShapeResult is not needed.
  if (break_anywhere_if_overflow_ && !override_break_anywhere_)
    breaker.SetNoResultIfOverflow();

#if DCHECK_IS_ON()
  unsigned try_count = 0;
#endif
  LayoutUnit inline_size;
  ShapingLineBreaker::Result result;
  while (true) {
#if DCHECK_IS_ON()
    ++try_count;
    DCHECK_LE(try_count, 2u);
#endif
    const ShapeResultView* shape_result =
        breaker.ShapeLine(item_result->StartOffset(),
                          available_width.ClampNegativeToZero(), &result);

    // If this item overflows and 'break-word' is set, this line will be
    // rewinded. Making this item long enough to overflow is enough.
    if (!shape_result) {
      DCHECK(breaker.NoResultIfOverflow());
      item_result->inline_size = available_width_with_hyphens + 1;
      item_result->text_offset.end = item.EndOffset();
      item_result->text_offset.AssertNotEmpty();
      return kOverflow;
    }
    DCHECK_EQ(shape_result->NumCharacters(),
              result.break_offset - item_result->StartOffset());
    // It is critical to move the offset forward, or LineBreaker may keep
    // adding InlineItemResult until all the memory is consumed.
    CHECK_GT(result.break_offset, item_result->StartOffset());

    inline_size = shape_result->SnappedWidth().ClampNegativeToZero();
    item_result->inline_size = inline_size;
    if (result.is_hyphenated) [[unlikely]] {
      InlineItemResults* item_results = line_info->MutableResults();
      const LayoutUnit hyphen_inline_size =
          AddHyphen(item_results, item_result);
      // If the hyphen overflows, retry with the reduced available width.
      if (!result.is_overflow && inline_size <= available_width) {
        const LayoutUnit space_for_hyphen =
            available_width_with_hyphens - inline_size;
        if (space_for_hyphen >= 0 && hyphen_inline_size > space_for_hyphen) {
          available_width -= hyphen_inline_size;
          RemoveHyphen(item_results);
          continue;
        }
      }
      inline_size = item_result->inline_size;
    }
    item_result->text_offset.end = result.break_offset;
    item_result->text_offset.AssertNotEmpty();
    item_result->has_only_pre_wrap_trailing_spaces = result.has_trailing_spaces;
    item_result->has_only_bidi_trailing_spaces = result.has_trailing_spaces;
    item_result->shape_result = shape_result;
    break;
  }

  // * If width <= available_width:
  //   * If offset < item.EndOffset(): the break opportunity to fit is found.
  //   * If offset == item.EndOffset(): the break opportunity at the end fits,
  //     or the first break opportunity is beyond the end.
  //     There may be room for more characters.
  // * If width > available_width: The first break opportunity does not fit.
  //   offset is the first break opportunity, either inside, at the end, or
  //   beyond the end.
  if (item_result->EndOffset() < item.EndOffset()) {
    item_result->can_break_after = true;

    if (break_iterator_.BreakType() == LineBreakType::kBreakCharacter)
        [[unlikely]] {
      trailing_whitespace_ = WhitespaceState::kUnknown;
    } else {
      trailing_whitespace_ = WhitespaceState::kNone;
    }
  } else {
    DCHECK_EQ(item_result->EndOffset(), item.EndOffset());
    item_result->can_break_after = CanBreakAfter(item);
    trailing_whitespace_ = WhitespaceState::kUnknown;
  }

  // This result is not breakable any further if overflow. This information is
  // useful to optimize |HandleOverflow()|.
  item_result->may_break_inside = !result.is_overflow;

  // TODO(crbug.com/1003742): We should use |result.is_overflow| here. For now,
  // use |inline_size| because some tests rely on this behavior.
  return inline_size <= available_width_with_hyphens ? kSuccess : kOverflow;
}

bool LineBreaker::BreakTextAt(InlineItemResult* item_result,
                              const InlineItem& item,
                              ShapingLineBreaker& breaker,
                              LineInfo* line_info) {
  DCHECK(break_at_);
  DCHECK_LE(current_.text_offset, break_at_.end.text_offset);
  DCHECK_LE(current_.item_index, break_at_.offset.item_index);
  const bool should_break = current_.item_index >= break_at_.end.item_index;
  if (should_break) {
    DCHECK_LE(break_at_.end.text_offset, item_result->text_offset.end);
    item_result->text_offset.end = break_at_.end.text_offset;
    item_result->text_offset.AssertValid();
  } else {
    DCHECK_GE(break_at_.end.text_offset, item_result->text_offset.end);
  }
  if (item_result->Length()) {
    const ShapeResultView* shape_result = breaker.ShapeLineAt(
        item_result->StartOffset(), item_result->EndOffset());
    item_result->inline_size =
        shape_result->SnappedWidth().ClampNegativeToZero();
    item_result->shape_result = shape_result;
    if (break_at_.is_hyphenated) {
      AddHyphen(line_info->MutableResults(), item_result);
    }
  } else {
    DCHECK_EQ(item_result->inline_size, LayoutUnit());
    DCHECK(!break_at_.is_hyphenated);
  }
  item_result->can_break_after = true;
  trailing_whitespace_ = WhitespaceState::kNone;
  return should_break;
}

// Breaks the text item at the previous break opportunity from
// |item_result->text_offset.end|. Returns false if there were no previous break
// opportunities.
bool LineBreaker::BreakTextAtPreviousBreakOpportunity(
    InlineItemResults& results,
    wtf_size_t item_result_index) {
  InlineItemResult* item_result = &results[item_result_index];
  DCHECK(item_result->item);
  DCHECK(item_result->may_break_inside);
  const InlineItem& item = *item_result->item;
  DCHECK_EQ(item.Type(), InlineItem::kText);
  DCHECK(item.Style() && item.Style()->ShouldWrapLine());
  DCHECK(!is_text_combine_);

  // TODO(jfernandez): Should we use the non-hangable-run-end instead ?
  unsigned break_opportunity = break_iterator_.PreviousBreakOpportunity(
      item_result->EndOffset() - 1, item_result->StartOffset());
  if (break_opportunity <= item_result->StartOffset())
    return false;
  item_result->text_offset.end = break_opportunity;
  item_result->text_offset.AssertNotEmpty();
  item_result->shape_result = ShapeResultView::Create(
      item.TextShapeResult(), item_result->StartOffset(),
      item_result->EndOffset());
  item_result->inline_size =
      item_result->shape_result->SnappedWidth().ClampNegativeToZero();
  item_result->can_break_after = true;

  if (trailing_collapsible_space_.has_value() &&
      trailing_collapsible_space_->item_results == &results &&
      trailing_collapsible_space_->item_result_index == item_result_index) {
    trailing_collapsible_space_.reset();
  }

  return true;
}

// This function handles text item for min-content. The specialized logic is
// because min-content is very expensive by breaking at every break opportunity
// and producing as many lines as the number of break opportunities.
//
// This function breaks the text in InlineItem at every break opportunity,
// computes the maximum width of all words, and creates one InlineItemResult
// that has the maximum width. For example, for a text item of "1 2 34 5 6",
// only the width of "34" matters for min-content.
//
// The first word and the last word, "1" and "6" in the example above, are
// handled in normal |HandleText()| because they may form a word with the
// previous/next item.
bool LineBreaker::HandleTextForFastMinContent(InlineItemResult* item_result,
                                              const InlineItem& item,
                                              const ShapeResult& shape_result,
                                              LineInfo* line_info) {
  DCHECK_EQ(mode_, LineBreakerMode::kMinContent);
  DCHECK(auto_wrap_);
  DCHECK(item.Type() == InlineItem::kText ||
         (item.Type() == InlineItem::kControl &&
          Text()[item.StartOffset()] == uchar::kTab));
  DCHECK(&shape_result);

  // Break the text at every break opportunity and measure each word.
  unsigned start_offset = item_result->StartOffset();
  DCHECK_LT(start_offset, item.EndOffset());
  DCHECK_EQ(shape_result.StartIndex(), item.StartOffset());
  DCHECK_GE(start_offset, shape_result.StartIndex());
  const unsigned item_end_offset = item.EndOffset();
  unsigned end_offset = item_end_offset;

  bool should_break_at_first_opportunity = false;
  const LayoutUnit indent = line_info->TextIndent();
  if (indent) [[unlikely]] {
    if (indent < 0) [[unlikely]] {
      // A negative `text-indent` can make this line not wrap at the first
      // break opportunity if it's in the indent. Use `HandleText()`.
      return false;
    }
    should_break_at_first_opportunity = true;
    end_offset = start_offset + 1;
  } else if (position_ < indent) [[unlikely]] {
    // A negative margin can move the position before the initial position.
    // This line may not wrap at the first break opportunity if it appears
    // before the initial position. Fall back to `HandleText()`.
    return false;
  } else {
    if (position_ != indent) [[unlikely]] {
      // Break at the first opportunity if there were previous items.
      should_break_at_first_opportunity = true;
      end_offset = start_offset + 1;
    }
#if EXPENSIVE_DCHECKS_ARE_ON()
    // Whether the start offset is at middle of a word or not can also be
    // determined by `line_info->Results()`. Check if they match.
    auto results = base::span(line_info->Results());
    DCHECK_EQ(item_result, &results.back());
    results = results.first(results.size() - 1);
    bool is_at_mid_word = false;
    for (const InlineItemResult& result : std::views::reverse(results)) {
      DCHECK(!result.can_break_after);
      if (result.inline_size) {
        is_at_mid_word = true;
        break;
      }
    }
    // Negative margins on separate items can cancel out positive inline sizes,
    // leaving `position_` at `indent` despite previous items having non-zero
    // `inline_size`. In that case `is_at_mid_word` is true but
    // `should_break_at_first_opportunity` is false. This is correct: with zero
    // accumulated width, the first word needs no special handling.
    DCHECK(should_break_at_first_opportunity
               ? (is_at_mid_word ||
                  (has_cloned_box_decorations_ &&
                   cloned_box_decorations_initial_size_ > LayoutUnit()))
               : (!is_at_mid_word || position_ == indent));
#endif  // EXPENSIVE_DCHECKS_ARE_ON()
  }

  shape_result.EnsurePositionData();
  const unsigned saved_start_offset = break_iterator_.StartOffset();
  FastMinTextContext context;
  const String& text = Text();
  const ComputedStyle& item_style = *item.Style();
  const bool should_break_spaces = item_style.ShouldBreakSpaces();
  unsigned next_break = 0;
  unsigned non_hangable_run_end = 0;
  bool can_break_after = false;
  while (start_offset < end_offset) {
    // TODO(crbug.com/332328872): `following()` scans back to the start of the
    // string. Resetting the ICU `BreakIterator` is faster than the scanning.
    break_iterator_.SetStartOffset(start_offset);

    next_break = break_iterator_.NextBreakOpportunity(
        start_offset + 1, std::min(item_end_offset + 1, text.length()));

    if (next_break > item_end_offset) [[unlikely]] {
      // The `item.EndOffset()` is not breakable; e.g., middle of a word.
      DCHECK_EQ(next_break, item_end_offset + 1);
      if (start_offset == item_result->StartOffset()) {
        // If this is the first word of this line, create an `InlineItemResult`
        // of this word with `!can_break_after`, so that it can create a line
        // with following items.
        next_break = item_end_offset;
        can_break_after = false;
      } else {
        const UChar next_ch = text[next_break - 1];
        if (next_ch == uchar::kLineFeed) {
          // Optimize to avoid splitting `InlineItemResult`. If the next is a
          // forced break, this line ends without additional widths.
          next_break = item_end_offset;
          can_break_after = false;
        } else {
          // If the end of `item` is middle of a word, spilt before the last
          // word. The last word should create a line with following items.
          next_break = start_offset;
          DCHECK(can_break_after);
          break;
        }
      }
    } else {
      can_break_after = true;
    }
    DCHECK_LE(next_break, item_end_offset);

    // Remove trailing spaces.
    non_hangable_run_end = next_break;
    if (!should_break_spaces) {
      while (non_hangable_run_end > start_offset &&
             IsBreakableSpace(text[non_hangable_run_end - 1])) {
        --non_hangable_run_end;
      }
    }

    // `word_len` may be zero if `start_offset` is at a breakable space.
    DCHECK_GE(non_hangable_run_end, start_offset);
    if (const wtf_size_t word_len = non_hangable_run_end - start_offset) {
      bool has_hyphen = can_break_after &&
                        text[non_hangable_run_end - 1] == uchar::kSoftHyphen;
      if (hyphenation_) [[unlikely]] {
        const StringView word(text, start_offset, word_len);
        if (should_break_at_first_opportunity) [[unlikely]] {
          if (const wtf_size_t location =
                  hyphenation_->FirstHyphenLocation(word, 0)) {
            next_break = non_hangable_run_end = start_offset + location;
            has_hyphen = can_break_after = true;
          }
          context.Add(shape_result, start_offset, non_hangable_run_end,
                      has_hyphen, *item_result);
        } else {
          context.AddHyphenated(shape_result, start_offset,
                                non_hangable_run_end, has_hyphen, *item_result,
                                *hyphenation_, word);
        }
      } else {
        context.Add(shape_result, start_offset, non_hangable_run_end,
                    has_hyphen, *item_result);
      }
    }

    DCHECK_GT(next_break, start_offset);
    start_offset = next_break;
  }

  break_iterator_.SetStartOffset(saved_start_offset);

  // Create an `InlineItemResult` that has the max of widths of all words.
  DCHECK_GE(non_hangable_run_end, item_result->StartOffset());
  DCHECK_LE(non_hangable_run_end, item_end_offset);
  if (item_style.ShouldCollapseWhiteSpaces()) {
    item_result->text_offset.end = non_hangable_run_end;
    trailing_whitespace_ = non_hangable_run_end != next_break
                               ? WhitespaceState::kCollapsed
                               : WhitespaceState::kNone;
  } else {
    item_result->text_offset.end = next_break;
    trailing_whitespace_ = non_hangable_run_end != next_break
                               ? WhitespaceState::kPreserved
                               : WhitespaceState::kNone;
  }
  item_result->text_offset.AssertValid();
  item_result->inline_size = context.MinInlineSize();
  position_ += item_result->inline_size;
  item_result->can_break_after = can_break_after;
  if (can_break_after) {
    state_ = LineBreakState::kTrailing;
  } else {
    state_ = LineBreakState::kOverflow;
  }

  DCHECK_GE(next_break, non_hangable_run_end);
  DCHECK_LE(next_break, item_end_offset);
  if (next_break >= item_end_offset) {
    MoveToNextOf(item);
  } else {
    // It's critical to move forward to avoid an infinite loop.
    DCHECK_EQ(current_.text_offset, item_result->StartOffset());
    CHECK_GT(next_break, current_.text_offset);
    current_.text_offset = next_break;
  }
  return true;
}

void LineBreaker::HandleEmptyText(const InlineItem& item, LineInfo* line_info) {
  // Add an empty `InlineItemResult` for empty or fully collapsed text. They
  // aren't necessary for line breaking/layout purposes, but callsites may need
  // to see all `InlineItem` by iterating `InlineItemResult`. For example,
  // `CreateLine` needs to `ClearNeedsLayout` for all `LayoutObject` including
  // empty or fully collapsed text.
  AddEmptyItem(item, line_info);
  MoveToNextOf(item);
}

// Re-shape the specified range of |InlineItem|.
const ShapeResult* LineBreaker::ShapeText(const InlineItem& item,
                                          unsigned start,
                                          unsigned end,
                                          ShapeOptions options) {
  ShapeResult* shape_result = nullptr;
  if (!items_data_->segments) {
    RunSegmenter::RunSegmenterRange segment_range =
        InlineItemSegment::UnpackSegmentData(start, end, item.SegmentData());
    shape_result = shaper_.Shape(item.Style()->GetFont(), item.Direction(),
                                 start, end, segment_range, options);
  } else {
    shape_result = items_data_->segments->ShapeText(
        &shaper_, item.Style()->GetFont(), item.Direction(), start, end,
        item.Index(), options);
  }
  if (spacing_.HasSpacing()) [[unlikely]] {
    shape_result->ApplySpacing(spacing_);
  }
  return shape_result;
}

void LineBreaker::AppendCandidates(const InlineItemResult& item_result,
                                   const LineInfo& line_info,
                                   LineBreakCandidateContext& context) {
  DCHECK(item_result.item);
  const InlineItem& item = *item_result.item;
  const wtf_size_t item_index = item_result.item_index;
  DCHECK(context.GetState() == LineBreakCandidateContext::kBreak ||
         !context.Candidates().empty());

  DCHECK_EQ(item.Type(), InlineItem::kText);
  if (!item.Length()) {
    // Fully collapsed spaces don't have break opportunities.
    context.AppendTrailingSpaces(
        item_result.can_break_after ? LineBreakCandidateContext::kBreak
                                    : context.GetState(),
        {item_result.item_index, item.EndOffset()}, context.Position());
    context.SetLast(&item, item.EndOffset());
    return;
  }

  DCHECK(item.TextShapeResult());
  struct ShapeResultWrapper {
    STACK_ALLOCATED();

   public:
    explicit ShapeResultWrapper(const ShapeResult* shape_result)
        : shape_result(shape_result),
          shape_result_start_index(shape_result->StartIndex()),
          is_ltr(shape_result->IsLtr()) {
      shape_result->EnsurePositionData();
    }

    bool IsLtr() const { return is_ltr; }

    // The returned position is in the external coordinate system set by
    // `SetBasePosition`, not the internal one of the `ShapeResult`.
    float PositionForOffset(unsigned offset) const {
      DCHECK_GE(offset, shape_result_start_index);
      const float position = shape_result->CachedPositionForOffset(
          offset - shape_result_start_index);
      return IsLtr() ? base_position + position : base_position - position;
    }

    // Adjusts the internal coordinate system of the `ShapeResult` to the
    // specified one.
    void SetBasePosition(wtf_size_t offset, float adjusted) {
      DCHECK_GE(offset, shape_result_start_index);
      const float position = shape_result->CachedPositionForOffset(
          offset - shape_result_start_index);
      base_position = IsLtr() ? adjusted - position : adjusted + position;
      DCHECK_EQ(adjusted, PositionForOffset(offset));
    }

    unsigned PreviousSafeToBreakOffset(unsigned offset) const {
      // Unlike `PositionForOffset`, `PreviousSafeToBreakOffset` takes the
      // "external" offset that takes care of `StartIndex()`.
      return shape_result->CachedPreviousSafeToBreakOffset(offset);
    }

    const ShapeResult* const shape_result;
    const wtf_size_t shape_result_start_index;
    float base_position = .0f;
    const bool is_ltr;
  } shape_result(item.TextShapeResult());
  const String& text_content = Text();

  // Extend the end offset to the end of the item or the end of this line,
  // whichever earlier. This is not only for performance but also to include
  // trailing spaces that may be removed by the line breaker.
  TextOffsetRange offset = item_result.TextOffset();
  offset.end = std::max(offset.end,
                        std::min(item.EndOffset(), line_info.EndTextOffset()));

  // Extend the start offset to `context.last_end_offset`. Trailing spaces may
  // be skipped, or leading spaces may be already handled.
  if (context.LastItem()) {
    DCHECK_GE(context.LastEndOffset(), item.StartOffset());
    if (context.LastEndOffset() >= offset.end) {
      return;  // Return if all characters were already handled.
    }
    offset.start = context.LastEndOffset();
    offset.AssertNotEmpty();
    shape_result.SetBasePosition(offset.start, context.Position());

    // Handle leading/trailing spaces if they were skipped.
    if (IsBreakableSpace(text_content[offset.start])) {
      DCHECK_GE(offset.start, item.StartOffset());
      do {
        ++offset.start;
      } while (offset.start < offset.end &&
               IsBreakableSpace(text_content[offset.start]));
      const float end_position = shape_result.PositionForOffset(offset.start);
      if (!offset.Length()) {
        context.AppendTrailingSpaces(item_result.can_break_after
                                         ? LineBreakCandidateContext::kBreak
                                         : LineBreakCandidateContext::kMidWord,
                                     {item_index, offset.start}, end_position);
        context.SetLast(&item, offset.end);
        return;
      }
      context.AppendTrailingSpaces(auto_wrap_
                                       ? LineBreakCandidateContext::kBreak
                                       : LineBreakCandidateContext::kMidWord,
                                   {item_index, offset.start}, end_position);
    }
  } else {
    shape_result.SetBasePosition(offset.start, context.Position());
  }
  offset.AssertNotEmpty();
  DCHECK_GE(offset.start, item.StartOffset());
  DCHECK_GE(offset.start, context.LastEndOffset());
  DCHECK_LE(offset.end, item.EndOffset());
  context.SetLast(&item, offset.end);

  // Setup the style and its derived fields for this `item`.
  if (offset.start < break_iterator_.StartOffset()) {
    break_iterator_.SetStartOffset(offset.start);
  }
  DCHECK(item.Style());
  SetCurrentStyle(*item.Style());

  // Find all break opportunities in `item_result`.
  std::optional<LayoutUnit> hyphen_advance_cache;
  for (;;) {
    // Compute the offset of the next break opportunity.
    wtf_size_t next_offset;
    if (auto_wrap_) {
      const wtf_size_t len = std::min(offset.end + 1, text_content.length());
      next_offset = break_iterator_.NextBreakOpportunity(offset.start + 1, len);
    } else {
      next_offset = offset.end + 1;
    }
    if (next_offset > offset.end && item_result.can_break_after) {
      // If `can_break_after`, honor it over `next_offset`. CSS can allow the
      // break at the end. E.g., fast/inline/line-break-atomic-inline.html
      next_offset = offset.end;
    }

    // Compute the position of the break opportunity and the end of the word.
    wtf_size_t end_offset;
    float next_position;
    float end_position;
    LineBreakCandidateContext::State next_state =
        LineBreakCandidateContext::kBreak;
    float penalty = 0;
    bool is_hyphenated = false;
    if (next_offset > offset.end) {
      // If the next break opportunity is beyond this item, stop at the end of
      // this item and set `is_middle_word`.
      end_offset = next_offset = offset.end;
      end_position = next_position =
          shape_result.PositionForOffset(next_offset);
      next_state = LineBreakCandidateContext::kMidWord;
    } else {
      if (next_offset == offset.end && !item_result.can_break_after) {
        // Can't break at `next_offset` by higher level protocols.
        // E.g., `<span>1 </span>2`.
        next_state = LineBreakCandidateContext::kMidWord;
      }
      next_position = shape_result.PositionForOffset(next_offset);

      // Exclude trailing spaces if any.
      end_offset = next_offset;
      DCHECK_GT(end_offset, offset.start);
      UChar last_ch = text_content[end_offset - 1];
      while (IsBreakableSpace(last_ch)) {
        --end_offset;
        if (end_offset == offset.start) {
          last_ch = 0;
          break;
        }
        last_ch = text_content[end_offset - 1];
      }
      DCHECK_LE(end_offset, offset.end);

      if (hyphenation_) [[unlikely]] {
        const LayoutUnit hyphen_advance =
            HyphenAdvance(*current_style_, shape_result.IsLtr(),
                          item_result.hyphen, hyphen_advance_cache);
        DCHECK_GT(end_offset, offset.start);
        const wtf_size_t word_len = end_offset - offset.start;
        const StringView word(text_content, offset.start, word_len);
        Vector<wtf_size_t, 8> locations = hyphenation_->HyphenLocations(word);
        // |locations| is a list of hyphenation points in the descending order.
#if EXPENSIVE_DCHECKS_ARE_ON()
        DCHECK(!locations.Contains(0u));
        DCHECK(!locations.Contains(word_len));
        DCHECK(std::is_sorted(locations.rbegin(), locations.rend()));
#endif  // EXPENSIVE_DCHECKS_ARE_ON()
        const float hyphen_penalty = context.HyphenPenalty();
        InlineItemTextIndex hyphen_offset = {item_index, 0};
        for (const wtf_size_t location : std::views::reverse(locations)) {
          hyphen_offset.text_offset = offset.start + location;
          const float position =
              shape_result.PositionForOffset(hyphen_offset.text_offset);
          context.Append(LineBreakCandidateContext::kBreak, hyphen_offset,
                         hyphen_offset, position, position + hyphen_advance,
                         hyphen_penalty,
                         /*is_hyphenated*/ true);
        }
      }

      // Compute the end position of this word, excluding trailing spaces.
      wtf_size_t end_safe_offset;
      switch (next_state) {
        case LineBreakCandidateContext::kBreak:
          end_safe_offset = shape_result.PreviousSafeToBreakOffset(end_offset);
          if (end_safe_offset < offset.start) {
            DCHECK_EQ(context.Candidates().back().offset.text_offset,
                      offset.start);
            end_safe_offset = offset.start;
          }
          break;
        case LineBreakCandidateContext::kMidWord:
          end_safe_offset = end_offset;
          break;
      }
      if (end_safe_offset == end_offset) {
        if (end_offset == next_offset) {
          end_position = next_position;
        } else {
          end_position = shape_result.PositionForOffset(end_offset);
        }
      } else {
        DCHECK_LT(end_safe_offset, end_offset);
        end_position = shape_result.PositionForOffset(end_safe_offset);
        const ShapeResult* end_shape_result =
            ShapeText(item, end_safe_offset, end_offset);
        end_position += end_shape_result->Width();
      }

      DCHECK(!is_hyphenated);
      if (end_offset == item_result.EndOffset()) {
        is_hyphenated = item_result.is_hyphenated;
      } else if (last_ch == uchar::kSoftHyphen &&
                 next_state == LineBreakCandidateContext::kBreak) [[unlikely]] {
        is_hyphenated = true;
      }
      if (is_hyphenated) {
        end_position += HyphenAdvance(*current_style_, shape_result.IsLtr(),
                                      item_result.hyphen, hyphen_advance_cache);
        penalty = context.HyphenPenalty();
      }
    }

    context.Append(next_state, {item_index, next_offset},
                   {item_index, end_offset}, next_position, end_position,
                   penalty, is_hyphenated);
    if (next_offset >= offset.end) {
      break;
    }
    offset.start = next_offset;
  }
}

bool LineBreaker::CanBreakInside(const LineInfo& line_info) {
  const InlineItemResults& item_results = line_info.Results();
  for (wtf_size_t i = 0; i < item_results.size() - 1; ++i) {
    if (item_results[i].can_break_after) {
      for (++i; i < item_results.size(); ++i) {
        if (!item_results[i].item->IsFloatingOrOutOfFlowPositioned()) {
          return true;
        }
      }
    }
  }
  for (const InlineItemResult& item_result : item_results) {
    DCHECK(item_result.item);
    const InlineItem& item = *item_result.item;
    if (item.Type() == InlineItem::kText) {
      if (item_result.may_break_inside && CanBreakInside(item_result)) {
        return true;
      }
    }
  }
  return false;
}

bool LineBreaker::CanBreakInside(const InlineItemResult& item_result) {
  DCHECK(item_result.may_break_inside);
  DCHECK(item_result.item);
  const InlineItem& item = *item_result.item;
  DCHECK_EQ(item.Type(), InlineItem::kText);
  DCHECK(item.Style());
  SetCurrentStyle(*item.Style());
  if (!auto_wrap_) {
    return false;
  }
  const TextOffsetRange& offset = item_result.TextOffset();
  if (offset.start < break_iterator_.StartOffset()) {
    break_iterator_.SetStartOffset(offset.start);
  }
  const wtf_size_t next_offset =
      break_iterator_.NextBreakOpportunity(offset.start + 1);
  return next_offset < offset.end;
}

// Compute a new ShapeResult for the specified end offset.
// The end is re-shaped if it is not safe-to-break.
const ShapeResultView* LineBreaker::TruncateLineEndResult(
    const LineInfo& line_info,
    const InlineItemResult& item_result,
    unsigned end_offset) {
  DCHECK(item_result.item);
  const InlineItem& item = *item_result.item;

  // Check given offsets require to truncate |item_result.shape_result|.
  const unsigned start_offset = item_result.StartOffset();
  const ShapeResultView* source_result = item_result.shape_result.Get();
  DCHECK(source_result);
  DCHECK_GE(start_offset, source_result->StartIndex());
  DCHECK_LE(end_offset, source_result->EndIndex());
  DCHECK(start_offset > source_result->StartIndex() ||
         end_offset < source_result->EndIndex());

  if (!NeedsAccurateEndPosition(line_info, item)) {
    return ShapeResultView::Create(source_result, start_offset, end_offset);
  }

  unsigned last_safe = source_result->PreviousSafeToBreakOffset(end_offset);
  DCHECK_LE(last_safe, end_offset);
  // TODO(abotella): Shouldn't last_safe <= start_offset trigger a reshaping?
  if (last_safe == end_offset || last_safe <= start_offset) {
    return ShapeResultView::Create(source_result, start_offset, end_offset);
  }

  const ShapeResult* end_result =
      ShapeText(item, std::max(last_safe, start_offset), end_offset);
  DCHECK_EQ(end_result->Direction(), source_result->Direction());
  ShapeResultView::Segment segments[2];
  segments[0] = {source_result, start_offset, last_safe};
  segments[1] = {end_result, 0, end_offset};
  return ShapeResultView::Create(segments);
}

// Update |ShapeResult| in |item_result| to match to its |start_offset| and
// |end_offset|. The end is re-shaped if it is not safe-to-break.
void LineBreaker::UpdateShapeResult(const LineInfo& line_info,
                                    InlineItemResult* item_result) {
  DCHECK(item_result);
  item_result->shape_result =
      TruncateLineEndResult(line_info, *item_result, item_result->EndOffset());
  DCHECK(item_result->shape_result);
  item_result->inline_size = item_result->shape_result->SnappedWidth();
}

inline void LineBreaker::HandleTrailingSpaces(const InlineItem& item,
                                              LineInfo* line_info) {
  const ShapeResult* shape_result = item.TextShapeResult();
  // Call |HandleTrailingSpaces| even if |item| does not have |ShapeResult|, so
  // that we skip spaces.
  HandleTrailingSpaces(item, shape_result, line_info);
}

void LineBreaker::HandleTrailingSpaces(const InlineItem& item,
                                       const ShapeResult* shape_result,
                                       LineInfo* line_info) {
  DCHECK(item.Type() == InlineItem::kText ||
         (item.Type() == InlineItem::kControl &&
          Text()[item.StartOffset()] == uchar::kTab));
  bool is_control_tab = item.Type() == InlineItem::kControl &&
                        Text()[item.StartOffset()] == uchar::kTab;
  DCHECK(item.Type() == InlineItem::kText || is_control_tab);
  DCHECK_GE(current_.text_offset, item.StartOffset());
  DCHECK_LT(current_.text_offset, item.EndOffset());
  const String& text = Text();
  DCHECK(item.Style());
  const ComputedStyle& style = *item.Style();

  if (!auto_wrap_) {
    state_ = LineBreakState::kDone;
    return;
  }
  DCHECK(!is_text_combine_);

  if (style.ShouldCollapseWhiteSpaces() &&
      !Character::IsOtherSpaceSeparator(text[current_.text_offset])) {
    if (text[current_.text_offset] != uchar::kSpace) {
      if (current_.text_offset > 0 &&
          IsBreakableSpace(text[current_.text_offset - 1])) {
        trailing_whitespace_ = WhitespaceState::kCollapsible;
      }
      state_ = LineBreakState::kDone;
      return;
    }

    // Skipping one whitespace removes all collapsible spaces because
    // collapsible spaces are collapsed to single space in InlineItemBuilder.
    // If these collapsible spaces follow preserved whitespace, we keep the
    // trailing whitespace status as preserved.
    current_.text_offset++;
    if (trailing_whitespace_ != WhitespaceState::kPreserved) {
      trailing_whitespace_ = WhitespaceState::kCollapsed;
    }

    // Make the last item breakable after, even if it was nowrap.
    InlineItemResults* item_results = line_info->MutableResults();
    DCHECK(!item_results->empty());
    item_results->back().can_break_after = true;
  } else if (!style.ShouldBreakSpaces()) {
    // Find the end of the run of space characters in this item.
    // Other white space characters (e.g., tab) are not included in this item.
    DCHECK(style.ShouldBreakOnlyAfterWhiteSpace() ||
           Character::IsOtherSpaceSeparator(text[current_.text_offset]));
    unsigned end = current_.text_offset;
    while (end < item.EndOffset() &&
           IsBreakableSpaceOrOtherSeparator(text[end]))
      end++;
    if (end == current_.text_offset) {
      if (IsBreakableSpaceOrOtherSeparator(text[end - 1]))
        trailing_whitespace_ = WhitespaceState::kPreserved;
      state_ = LineBreakState::kDone;
      return;
    }

    // TODO (jfernandez): Could we just modify the last ItemResult
    // instead of creating a new one ?
    // Probably we can (koji). We would need to review usage of these
    // item results, and change them to use "non_hangable_run_end"
    // instead.
    DCHECK(shape_result);
    InlineItemResult* item_result = AddItem(item, end, line_info);
    item_result->should_create_line_box = true;
    item_result->has_only_pre_wrap_trailing_spaces = true;
    item_result->has_only_bidi_trailing_spaces = true;
    item_result->shape_result = ShapeResultView::Create(shape_result);
    if (item_result->StartOffset() == item.StartOffset() &&
        item_result->EndOffset() == item.EndOffset()) {
      item_result->inline_size =
          item_result->shape_result && mode_ != LineBreakerMode::kMinContent &&
                  !line_clamp_ellipsis_width_
              ? item_result->shape_result->SnappedWidth()
              : LayoutUnit();
    } else {
      UpdateShapeResult(*line_info, item_result);
      if (mode_ == LineBreakerMode::kMinContent || line_clamp_ellipsis_width_) {
        item_result->inline_size = LayoutUnit();
      }
    }
    position_ += item_result->inline_size;
    item_result->can_break_after =
        end < text.length() && !IsBreakableSpaceOrOtherSeparator(text[end]);
    current_.text_offset = end;
    trailing_whitespace_ = WhitespaceState::kPreserved;
  }

  // If non-space characters follow, the line is done.
  // Otherwise keep checking next items for the break point.
  DCHECK_LE(current_.text_offset, item.EndOffset());
  if (current_.text_offset < item.EndOffset()) {
    state_ = LineBreakState::kDone;
    return;
  }
  DCHECK_EQ(current_.text_offset, item.EndOffset());
  const InlineItemResults& item_results = line_info->Results();
  if (item_results.empty() || item_results.back().item.Get() != &item) {
    // If at the end of `item` but the item hasn't been added to `line_info`,
    // add an empty text item. See `HandleEmptyText`.
    AddEmptyItem(item, line_info);
  }
  current_.item_index++;
  state_ = LineBreakState::kTrailing;
}

void LineBreaker::RewindTrailingOpenTags(LineInfo* line_info) {
  // Remove trailing open tags. Open tags are included as trailable items
  // because they are ambiguous. When the line ends, and if the end of line has
  // open tags, they are not trailable.
  // TODO(crbug.com/1009936): Open tags and trailing space items can interleave,
  // but the current code supports only one trailing space item. Multiple
  // trailing space items and interleaved open/close tags should be supported.
  const InlineItemResults& item_results = line_info->Results();
  for (const InlineItemResult& item_result :
       std::views::reverse(item_results)) {
    DCHECK(item_result.item);
    if (item_result.item->Type() != InlineItem::kOpenTag) {
      unsigned end_index =
          base::checked_cast<unsigned>(&item_result - item_results.data() + 1);
      if (end_index < item_results.size()) {
        const InlineItemResult& end_item_result = item_results[end_index];
        const InlineItemTextIndex end = end_item_result.Start();
        ResetRewindLoopDetector();
        Rewind(end_index, line_info);
        current_ = end;
        items_data_->AssertOffset(current_.item_index, current_.text_offset);
      }
      break;
    }
  }
}

// Remove trailing collapsible spaces in |line_info|.
// https://drafts.csswg.org/css-text-3/#white-space-phase-2
void LineBreaker::RemoveTrailingCollapsibleSpace(LineInfo* line_info) {
  // Rewind trailing open-tags to wrap before them, except when this line ends
  // with a forced break, including the one implied by block-in-inline.
  if (!is_forced_break_) {
    RewindTrailingOpenTags(line_info);
  }

  ComputeTrailingCollapsibleSpace(line_info);
  if (!trailing_collapsible_space_.has_value()) {
    return;
  }

  // We have a trailing collapsible space. Remove it.
  InlineItemResult* item_result = &trailing_collapsible_space_->ItemResult();
  bool position_was_saturated = position_ == LayoutUnit::Max();
  position_ -= item_result->inline_size;
  if (const ShapeResultView* collapsed_shape_result =
          trailing_collapsible_space_->collapsed_shape_result) {
    --item_result->text_offset.end;
    item_result->text_offset.AssertNotEmpty();
    item_result->shape_result = collapsed_shape_result;
    item_result->inline_size = item_result->shape_result->SnappedWidth();
    position_ += item_result->inline_size;
  } else {
    // Make it empty, but don't remove. See `HandleEmptyText`.
    item_result->text_offset.end = item_result->text_offset.start;
    item_result->shape_result = nullptr;
    item_result->inline_size = LayoutUnit();
  }
  for (const auto& [results, index] :
       trailing_collapsible_space_->ancestor_ruby_columns) {
    InlineItemResult& ruby_column = (*results)[index];
    CHECK(ruby_column.IsRubyColumn());
    LineInfo& base_line = ruby_column.ruby_column->base_line;
    LayoutUnit new_width = base_line.ComputeWidth();
    base_line.SetWidth(base_line.AvailableWidth(), new_width);
    // Update LineInfo::end_offset_for_justify_.
    base_line.UpdateTextAlign();
    for (auto& line : ruby_column.ruby_column->annotation_line_list) {
      new_width = std::max(new_width, line.Width());
    }
    ruby_column.inline_size = new_width;
  }
  if (position_was_saturated ||
      !trailing_collapsible_space_->ancestor_ruby_columns.empty()) {
    position_ = line_info->ComputeWidth();
  }
  trailing_collapsible_space_.reset();
  trailing_whitespace_ = WhitespaceState::kCollapsed;
}

// Compute the width of trailing spaces without removing it.
LayoutUnit LineBreaker::TrailingCollapsibleSpaceWidth(LineInfo* line_info) {
  ComputeTrailingCollapsibleSpace(line_info);
  if (!trailing_collapsible_space_.has_value())
    return LayoutUnit();

  // Normally, the width of new_reuslt is smaller, but technically it can be
  // larger. In such case, it means the trailing spaces has negative width.
  InlineItemResult& item_result = trailing_collapsible_space_->ItemResult();
  LayoutUnit width_diff = item_result.inline_size;
  if (const ShapeResultView* collapsed_shape_result =
          trailing_collapsible_space_->collapsed_shape_result) {
    width_diff -= collapsed_shape_result->SnappedWidth();
  }
  if (trailing_collapsible_space_->ancestor_ruby_columns.empty()) {
    return width_diff;
  }
  for (const auto& [results, index] :
       trailing_collapsible_space_->ancestor_ruby_columns) {
    InlineItemResult& ruby_column = (*results)[index];
    CHECK(ruby_column.IsRubyColumn());
    LayoutUnit new_width =
        ruby_column.ruby_column->base_line.Width() - width_diff;
    for (auto& line : ruby_column.ruby_column->annotation_line_list) {
      new_width = std::max(new_width, line.Width());
    }
    width_diff = ruby_column.inline_size - new_width;
    if (width_diff == LayoutUnit()) {
      break;
    }
  }
  return width_diff;
}

// Find trailing collapsible space in `line_info` and its descendants if exists.
// The result is cached to |trailing_collapsible_space_|.
void LineBreaker::ComputeTrailingCollapsibleSpace(LineInfo* line_info) {
  if (trailing_whitespace_ == WhitespaceState::kLeading ||
      trailing_whitespace_ == WhitespaceState::kNone ||
      trailing_whitespace_ == WhitespaceState::kCollapsed ||
      trailing_whitespace_ == WhitespaceState::kPreserved) {
    trailing_collapsible_space_.reset();
    return;
  }
  DCHECK(trailing_whitespace_ == WhitespaceState::kUnknown ||
         trailing_whitespace_ == WhitespaceState::kCollapsible);

  trailing_whitespace_ = WhitespaceState::kNone;
  if (!ComputeTrailingCollapsibleSpaceHelper(*line_info)) {
    trailing_collapsible_space_.reset();
  }
}

// Returns true if trailing_whitespace_ is determined.
bool LineBreaker::ComputeTrailingCollapsibleSpaceHelper(LineInfo& line_info) {
  const String& text = Text();
  for (auto& item_result : std::views::reverse(*line_info.MutableResults())) {
    DCHECK(item_result.item);
    const InlineItem& item = *item_result.item;
    if (item_result.IsRubyColumn()) {
      if (ComputeTrailingCollapsibleSpaceHelper(
              item_result.ruby_column->base_line)) {
        if (trailing_collapsible_space_ &&
            trailing_collapsible_space_->item_result_index != kNotFound) {
          trailing_collapsible_space_->ancestor_ruby_columns.push_back(
              std::make_pair(line_info.MutableResults(),
                             std::distance(line_info.MutableResults()->data(),
                                           &item_result)));
        }
        return true;
      }
      continue;
    } else if (item.EndCollapseType() == InlineItem::kOpaqueToCollapsing) {
      continue;
    }
    if (item.Type() == InlineItem::kText) {
      if (item_result.Length() == 0) {
        continue;
      }
      DCHECK_GT(item_result.EndOffset(), 0u);
      DCHECK(item.Style());
      if (Character::IsOtherSpaceSeparator(text[item_result.EndOffset() - 1])) {
        trailing_whitespace_ = WhitespaceState::kPreserved;
        trailing_collapsible_space_.reset();
        return true;
      }
      if (!IsBreakableSpace(text[item_result.EndOffset() - 1])) {
        trailing_collapsible_space_.reset();
        return true;
      }
      if (item.Style()->ShouldPreserveWhiteSpaces()) {
        trailing_whitespace_ = WhitespaceState::kPreserved;
        trailing_collapsible_space_.reset();
        return true;
      }
      // |shape_result| is nullptr if this is an overflow because BreakText()
      // uses kNoResultIfOverflow option.
      if (!item_result.shape_result) {
        trailing_collapsible_space_.reset();
        return true;
      }

      InlineItemResults* results = line_info.MutableResults();
      wtf_size_t index = CheckedDistance(results->data(), &item_result);
      if (!trailing_collapsible_space_.has_value() ||
          trailing_collapsible_space_->item_results != results ||
          trailing_collapsible_space_->item_result_index != index) {
        trailing_collapsible_space_.emplace();
        trailing_collapsible_space_->item_results = results;
        trailing_collapsible_space_->item_result_index = index;
        if (item_result.EndOffset() - 1 > item_result.StartOffset()) {
          trailing_collapsible_space_->collapsed_shape_result =
              TruncateLineEndResult(line_info, item_result,
                                    item_result.EndOffset() - 1);
        }
      }
      trailing_whitespace_ = WhitespaceState::kCollapsible;
      return true;
    }
    if (item.Type() == InlineItem::kControl) {
      if (item.TextType() == TextItemType::kForcedLineBreak) {
        DCHECK_EQ(text[item.StartOffset()], uchar::kLineFeed);
        continue;
      }
      trailing_whitespace_ = WhitespaceState::kPreserved;
      trailing_collapsible_space_.reset();
      return true;
    }
    trailing_collapsible_space_.reset();
    return true;
  }
  return false;
}

// Per UAX#9 L1, any spaces logically at the end of a line must be reset to the
// paragraph's bidi level. If there are any such trailing spaces in an item
// result together with other non-space characters, this method splits them into
// their own item result.
//
// Furthermore, item results can't override their item's bidi level, so this
// method instead marks all such item results with `has_only_trailing_spaces`,
// which will cause them to be treated as having the base bidi level in
// InlineLayoutAlgorithm::BidiReorder.
void LineBreaker::SplitTrailingBidiPreservedSpace(LineInfo* line_info) {
  DCHECK(trailing_whitespace_ == WhitespaceState::kLeading ||
         trailing_whitespace_ == WhitespaceState::kNone ||
         trailing_whitespace_ == WhitespaceState::kCollapsed ||
         trailing_whitespace_ == WhitespaceState::kPreserved);

  if (trailing_whitespace_ == WhitespaceState::kLeading ||
      trailing_whitespace_ == WhitespaceState::kNone) {
    return;
  }

  if (!node_.IsBidiEnabled()) {
    return;
  }

  // TODO(abotella): This early return fixes a crash (crbug.com/324684931)
  // caused by |HandleTextForFastMinContent| creating item results with null
  // |shape_result|. This might affect hanging other space separators, but their
  // behavior with min-content is known to have bugs even in purely LTR text.
  if (mode_ == LineBreakerMode::kMinContent) {
    return;
  }

  // At this point, all trailing collapsible spaces have been collapsed, and all
  // remaining trailing spaces must be preserved.

  const String& text = Text();
  wtf_size_t result_index = line_info->Results().size();
  for (auto& item_result : std::views::reverse(*line_info->MutableResults())) {
    result_index--;
    DCHECK(item_result.item);
    const InlineItem& item = *item_result.item;

    if (item_result.has_only_bidi_trailing_spaces ||
        item.EndCollapseType() == InlineItem::kOpaqueToCollapsing ||
        item.TextType() == TextItemType::kForcedLineBreak) {
      continue;
    }

    if (item.Type() != InlineItem::kText &&
        item.Type() != InlineItem::kControl) {
      return;
    }

    if (!item_result.Length()) {
      item_result.has_only_bidi_trailing_spaces = true;
      continue;
    }

    DCHECK_GT(item_result.EndOffset(), 0u);

    wtf_size_t i = item_result.EndOffset();
    for (; i > item_result.StartOffset() &&
           (IsBreakableSpace(text[i - 1]) || IsBidiTrailingSpace(text[i - 1]));
         i--) {
    }

    if (i == item_result.StartOffset()) {
      item_result.has_only_bidi_trailing_spaces = true;
    } else if (i == item_result.EndOffset()) {
      break;
    } else {
      // Only split the item if its bidi level doesn't match the paragraph's.
      // We check the item's bidi level, rather than its direction, because
      // higher bidi levels with the same direction (i.e. level 2 on an LTR
      // paragraph) must also be reset.
      if (item.BidiLevel() != (UBiDiLevel)base_direction_) {
        const ShapeResultView* source_shape_result =
            item_result.shape_result.Get();
        LayoutUnit prev_inline_size = item_result.inline_size;
        wtf_size_t start = item_result.StartOffset();
        wtf_size_t end = item_result.EndOffset();

        item_result.text_offset.end = i;
        item_result.shape_result =
            ShapeResultView::Create(source_shape_result, start, i);
        item_result.inline_size = item_result.shape_result->SnappedWidth();
        DCHECK_LE(item_result.inline_size, prev_inline_size);

        InlineItemResult spaces_result(item, item_result.item_index,
                                       TextOffsetRange(i, end),
                                       item_result.break_anywhere_if_overflow,
                                       item_result.should_create_line_box,
                                       item_result.has_unpositioned_floats);
        spaces_result.has_only_bidi_trailing_spaces = true;
        spaces_result.shape_result =
            ShapeResultView::Create(source_shape_result, i, end);
        spaces_result.inline_size = prev_inline_size - item_result.inline_size;

        line_info->MutableResults()->insert(result_index + 1,
                                            std::move(spaces_result));
      }
      break;
    }
  }
}

// |item| is |nullptr| if this is an implicit forced break.
void LineBreaker::HandleForcedLineBreak(const InlineItem* item,
                                        LineInfo* line_info) {
  // Check overflow, because the last item may have overflowed.
  if (HandleOverflowIfNeeded(line_info))
    return;

  if (item) {
    DCHECK_EQ(item->TextType(), TextItemType::kForcedLineBreak);
    DCHECK_EQ(Text()[item->StartOffset()], uchar::kLineFeed);

    // Special-code for BR clear elements. If we have floats that extend into
    // subsequent fragmentainers, we cannot get past the floats in the current
    // fragmentainer. If this is the case, and if there's anything on the line
    // before the BR element, add a line break before it, so that we at least
    // attempt to place that part of the line right away. The remaining BR clear
    // element will be placed on a separate line, which we'll push past as many
    // fragmentainers as we need to. Example:
    //
    // <div style="columns:4; column-fill:auto; height:100px;">
    //   <div style="float:left; width:10px; height:350px;"></div>
    //   first column<br clear="all">
    //   fourth column
    // </div>
    //
    // Here we'll create one line box for the first float fragment and the text
    // "first column". We'll later on attempt to create another line box for the
    // BR element, but it will fail in the inline layout algorithm, because it's
    // impossible to clear past the float. We'll retry in the second and third
    // columns, but the float is still in the way. Finally, in the fourth
    // column, we'll add the BR, add clearance, and then create a line for the
    // text "fourth column" past the float.
    //
    // This solution isn't perfect, because of this additional line box for the
    // BR element. We'll push the line box containing the BR to a fragmentainer
    // where it doesn't really belong, and it will take up block space there
    // (this can be observed if the float clearance is less than the height of
    // the line, so that there will be a gap between the bottom of the float and
    // the content that follows). No browser engines currently get BR clearance
    // across fragmentainers right.
    if (constraint_space_.HasBlockFragmentation() && item->GetLayoutObject() &&
        item->GetLayoutObject()->IsBR() &&
        exclusion_space_->NeedsClearancePastFragmentainer(
            item->Style()->Clear(*current_style_))) {
      if (!line_info->Results().empty()) {
        state_ = LineBreakState::kDone;
        return;
      }
    }

    InlineItemResult* item_result = AddItem(*item, line_info);
    item_result->should_create_line_box = true;
    item_result->has_only_pre_wrap_trailing_spaces = true;
    item_result->has_only_bidi_trailing_spaces = true;
    item_result->can_break_after = true;
    MoveToNextOf(*item);

    // Include following close tags. The difference is visible when they have
    // margin/border/padding.
    //
    // This is not a defined behavior, but legacy/WebKit do this for preserved
    // newlines and <br>s. Gecko does this only for preserved newlines (but
    // not for <br>s).
    const InlineItems& items = Items();
    while (!IsAtEnd()) {
      const InlineItem& next_item = *items[current_.item_index];
      if (next_item.Type() == InlineItem::kCloseTag) {
        HandleCloseTag(next_item, line_info);
        continue;
      }
      if (next_item.Type() == InlineItem::kText && !next_item.Length()) {
        HandleEmptyText(next_item, line_info);
        continue;
      }
      break;
    }
  }

  if (HasHyphen()) [[unlikely]] {
    position_ -= RemoveHyphen(line_info->MutableResults());
  }
  is_forced_break_ = true;
  line_info->SetHasForcedBreak();
  line_info->SetIsLastLine(true);
  state_ = LineBreakState::kDone;
}

// Measure control items; new lines and tab, that are similar to text, affect
// layout, but do not need shaping/painting.
void LineBreaker::HandleControlItem(const InlineItem& item,
                                    LineInfo* line_info) {
  DCHECK_GE(item.Length(), 1u);
  if (item.TextType() == TextItemType::kForcedLineBreak) {
    HandleForcedLineBreak(&item, line_info);
    return;
  }

  DCHECK_EQ(item.TextType(), TextItemType::kFlowControl);
  UChar character = Text()[item.StartOffset()];
  switch (character) {
    case uchar::kTab: {
      DCHECK(item.Style());
      const ComputedStyle& style = *item.Style();
      if (!style.GetFont()->PrimaryFont()) {
        // TODO(crbug.com/561873): PrimaryFont should not be nullptr.
        HandleEmptyText(item, line_info);
        return;
      }
      const ShapeResult* shape_result =
          ShapeResult::CreateForTabulationCharacters(
              &node_.FontForTab(), item.Direction(), style.GetTabSize(),
              (RuntimeEnabledFeatures::TabAlignmentWithFloatsEnabled()
                   ? position_ + ComputeFloatOffset()
                   : position_) +
                  tab_stop_offset_,
              item.StartOffset(), item.Length());
      HandleText(item, *shape_result, line_info);
      return;
    }
    case uchar::kZeroWidthSpace: {
      // <wbr> tag creates break opportunities regardless of auto_wrap.
      InlineItemResult* item_result = AddItem(item, line_info);
      // A generated break opportunity doesn't generate fragments, but we still
      // need to add this for rewind to find this opportunity. This will be
      // discarded in |InlineLayoutAlgorithm| when it generates fragments.
      if (!item.IsGeneratedForLineBreak())
        item_result->should_create_line_box = true;
      item_result->can_break_after = true;
      break;
    }
    case uchar::kCarriageReturn:
    case uchar::kFormFeed:
      // Ignore carriage return and form feed.
      // https://drafts.csswg.org/css-text-3/#white-space-processing
      // https://github.com/w3c/csswg-drafts/issues/855
      HandleEmptyText(item, line_info);
      return;
    default:
      NOTREACHED();
  }
  MoveToNextOf(item);
}

void LineBreaker::HandleBidiControlItem(const InlineItem& item,
                                        LineInfo* line_info) {
  DCHECK_EQ(item.Length(), 1u);

  // Bidi control characters have enter/exit semantics. Handle "enter"
  // characters simialr to open-tag, while "exit" (pop) characters similar to
  // close-tag.
  UChar character = Text()[item.StartOffset()];
  bool is_pop = character == uchar::kPopDirectionalIsolate ||
                character == uchar::kPopDirectionalFormatting;
  InlineItemResults* item_results = line_info->MutableResults();
  if (is_pop) {
    if (!item_results->empty()) {
      InlineItemResult* item_result = AddItem(item, line_info);
      InlineItemResult* last = &(*item_results)[item_results->size() - 2];
      // Honor the last |can_break_after| if it's true, in case it was
      // artificially set to true for break-after-space.
      if (last->can_break_after) {
        item_result->can_break_after = last->can_break_after;
        last->can_break_after = false;
      } else {
        // Otherwise compute from the text. |LazyLineBreakIterator| knows how to
        // break around bidi control characters.
        ComputeCanBreakAfter(item_result, auto_wrap_, break_iterator_);
      }
    } else {
      AddItem(item, line_info);
    }
  } else {
    if (state_ == LineBreakState::kTrailing &&
        CanBreakAfterLast(*item_results)) {
      DCHECK(!line_info->IsLastLine());
      MoveToNextOf(item);
      state_ = LineBreakState::kDone;
      return;
    }
    if (!item_results->empty() &&
        RuntimeEnabledFeatures::LineBreakBidiControlEnterEnabled()) {
      InlineItemResult* item_result = AddItem(item, line_info);
      ComputeCanBreakAfter(item_result, auto_wrap_, break_iterator_);
    } else {
      AddItem(item, line_info);
    }
  }
  MoveToNextOf(item);
}

void LineBreaker::HandleAtomicInline(const InlineItem& item,
                                     LineInfo* line_info) {
  DCHECK(item.Type() == InlineItem::kAtomicInline ||
         item.Type() == InlineItem::kInitialLetterBox);
  DCHECK(item.Style());
  const ComputedStyle& style = *item.Style();

  const LayoutUnit remaining_width = RemainingAvailableWidth();
  bool ignore_overflow_if_negative_margin = false;
  if (state_ == LineBreakState::kContinue && remaining_width < 0 &&
      (!parent_breaker_ || auto_wrap_)) {
    const unsigned item_index = current_.item_index;
    DCHECK_EQ(item_index, item.Index());
    HandleOverflow(line_info);
    if (!line_info->HasOverflow() || item_index != current_.item_index) {
      return;
    }
    // Compute margins if this line overflows. Negative margins can put the
    // position back.
    DCHECK_NE(state_, LineBreakState::kContinue);
    ignore_overflow_if_negative_margin = true;
  }

  // Compute margins before computing overflow, because even when the current
  // position is beyond the end, negative margins can bring this item back to on
  // the current line.
  InlineItemResult* item_result = AddItem(item, line_info);
  item_result->margins =
      ComputeLineMarginsForVisualContainer(constraint_space_, style);
  LayoutUnit inline_margins = item_result->margins.InlineSum();
  if (ignore_overflow_if_negative_margin) [[unlikely]] {
    DCHECK_LT(remaining_width, 0);
    // The margin isn't negative, or the negative margin isn't large enough to
    // put the position back. Break this line before this item.
    if (inline_margins >= remaining_width) {
      RemoveLastItem(line_info);
      return;
    }
    // This line once overflowed, but the negative margin puts the position
    // back.
    state_ = LineBreakState::kContinue;
    line_info->SetHasOverflow(false);
  }

  // Last item may have ended with a hyphen, because at that point the line may
  // have ended there. Remove it because there are more items.
  if (HasHyphen()) [[unlikely]] {
    position_ -= RemoveHyphen(line_info->MutableResults());
  }

  const LineBreaker* root_breaker = this;
  while (root_breaker->parent_breaker_) {
    root_breaker = root_breaker->parent_breaker_;
  }
  const LineBreakerMode mode = root_breaker->mode_;
  const bool is_initial_letter_box =
      item.Type() == InlineItem::kInitialLetterBox;
  // When we're just computing min/max content sizes, we can skip the full
  // layout and just compute those sizes. On the other hand, for regular
  // layout we need to do the full layout and get the layout result.
  // Doing a full layout for min/max content can also have undesirable
  // side effects when that falls back to legacy layout.
  if ((!IsComputingContentSize() && mode == LineBreakerMode::kContent) || [&] {
        if (is_initial_letter_box) [[unlikely]] {
          return true;
        }
        return false;
      }()) {
    // If our baseline-source is non-auto use the easier to reason about
    // "default" algorithm type.
    BaselineAlgorithmType baseline_algorithm_type =
        style.BaselineSource() == EBaselineSource::kAuto
            ? BaselineAlgorithmType::kInlineBlock
            : BaselineAlgorithmType::kDefault;

    // https://drafts.csswg.org/css-pseudo-4/#first-text-line
    // > The first line of a table-cell or inline-block cannot be the first
    // > formatted line of an ancestor element.
    item_result->layout_result =
        BlockNode(To<LayoutBox>(item.GetLayoutObject()))
            .LayoutAtomicInline(constraint_space_, node_.Style(),
                                /* use_first_line_style */ false,
                                baseline_algorithm_type);
    // Ensure `NeedsCollectInlines` isn't set, or it may cause security risks.
    CHECK(!node_.GetLayoutBox()->NeedsCollectInlines());

    const auto& physical_box_fragment = To<PhysicalBoxFragment>(
        item_result->layout_result->GetPhysicalFragment());
    item_result->inline_size =
        LogicalFragment(constraint_space_.GetWritingDirection(),
                        physical_box_fragment)
            .InlineSize();

    if (is_initial_letter_box &&
        ShouldApplyInlineKerning(physical_box_fragment)) [[unlikely]] {
      // Apply "Inline Kerning" to the initial letter box[1].
      // [1] https://drafts.csswg.org/css-inline/#initial-letter-inline-position
      const LineBoxStrut side_bearing =
          ComputeNegativeSideBearings(physical_box_fragment);
      if (IsLtr(base_direction_)) {
        item_result->margins.inline_start += side_bearing.inline_start;
        inline_margins += side_bearing.inline_start;
      } else {
        item_result->margins.inline_end += side_bearing.inline_end;
        inline_margins += side_bearing.inline_end;
      }
    }

    item_result->inline_size += inline_margins;
  } else {
    DCHECK(IsComputingContentSize() || mode == LineBreakerMode::kMaxContent ||
           mode == LineBreakerMode::kMinContent);
    ComputeMinMaxContentSizeForBlockChild(item, item_result, root_breaker);
  }

  item_result->should_create_line_box = true;
  item_result->can_break_after = CanBreakAfterAtomicInline(item);

  position_ += item_result->inline_size;

  trailing_whitespace_ = WhitespaceState::kNone;
  MoveToNextOf(item);
}

void LineBreaker::ComputeMinMaxContentSizeForBlockChild(
    const InlineItem& item,
    InlineItemResult* item_result,
    const LineBreaker* root_breaker) {
  const LineBreakerMode mode = root_breaker->mode_;
  MaxSizeCache* size_cache = root_breaker->max_size_cache_;

  DCHECK(IsComputingContentSize() || mode == LineBreakerMode::kMaxContent ||
         mode == LineBreakerMode::kMinContent);
  if (mode == LineBreakerMode::kMaxContent && size_cache) {
    const unsigned item_index = item.Index();
    item_result->inline_size = (*size_cache)[item_index];
    return;
  }

  DCHECK(IsComputingContentSize() || mode == LineBreakerMode::kMinContent ||
         !size_cache);
  BlockNode child(To<LayoutBox>(item.GetLayoutObject()));

  MinMaxConstraintSpaceBuilder builder(constraint_space_, node_.Style(), child,
                                       /* is_new_fc */ true);
  builder.SetAvailableBlockSize(constraint_space_.AvailableSize().block_size);
  builder.SetPercentageResolutionBlockSize(
      child.IsReplaced()
          ? constraint_space_.ReplacedChildPercentageResolutionBlockSize()
          : constraint_space_.PercentageResolutionBlockSize());
  const auto space = builder.ToConstraintSpace();

  const MinMaxSizesResult result = ComputeMinAndMaxContentContribution(
      node_.Style(), child, space,
      MinMaxSizesInput::Constrained(
          root_breaker->line_opportunity_.AvailableInlineSize()));
  // Ensure `NeedsCollectInlines` isn't set, or it may cause security risks.
  CHECK(!node_.GetLayoutBox()->NeedsCollectInlines());
  const LayoutUnit inline_margins = item_result->margins.InlineSum();
  const LineBreaker* main_breaker = root_breaker;
  if (main_breaker->mode_ == LineBreakerMode::kMinContent) {
    item_result->inline_size = result.sizes.min_size + inline_margins;
    if (root_breaker->depends_on_block_constraints_out_) {
      *root_breaker->depends_on_block_constraints_out_ |=
          result.depends_on_block_constraints;
    }
    if ((size_cache = main_breaker->max_size_cache_)) {
      if (size_cache->empty()) {
        size_cache->resize(Items().size());
      }
      const unsigned item_index = item.Index();
      (*size_cache)[item_index] = result.sizes.max_size + inline_margins;
    }
    return;
  }

  DCHECK(mode == LineBreakerMode::kContent ||
         (mode == LineBreakerMode::kMaxContent && !size_cache));
  item_result->inline_size = result.sizes.max_size + inline_margins;
}

void LineBreaker::HandleBlockInInline(const InlineItem& item,
                                      const BlockBreakToken* block_break_token,
                                      LineInfo* line_info) {
  DCHECK_EQ(item.Type(), InlineItem::kBlockInInline);
  DCHECK(!block_break_token || block_break_token->InputNode().GetLayoutBox() ==
                                   item.GetLayoutObject());

  if (!line_info->Results().empty()) {
    // If there were any items, force a line break before this item.
    force_non_empty_if_last_line_ = false;
    HandleForcedLineBreak(nullptr, line_info);
    return;
  }

  InlineItemResult* item_result = AddItem(item, line_info);
  bool move_past_block = true;
  if (mode_ == LineBreakerMode::kContent) {
    // The exclusion spaces *must* match. If they don't we'll have an incorrect
    // layout (as it will potentially won't consider some preceeding floats).
    // Move the derived geometry for performance.
    DCHECK(*exclusion_space_ == constraint_space_.GetExclusionSpace());
    constraint_space_.GetExclusionSpace().MoveAndUpdateDerivedGeometry(
        *exclusion_space_);

    BlockNode block_node(To<LayoutBox>(item.GetLayoutObject()));
    std::optional<ConstraintSpace> modified_space;
    const ConstraintSpace& child_space =
        constraint_space_.CloneForBlockInInlineIfNeeded(modified_space);
    const ColumnSpannerPath* spanner_path_for_child =
        FollowColumnSpannerPath(column_spanner_path_, block_node);
    const LayoutResult* layout_result =
        block_node.Layout(child_space, block_break_token,
                          /* early_break */ nullptr, spanner_path_for_child);
    // Ensure `NeedsCollectInlines` isn't set, or it may cause security risks.
    CHECK(!node_.GetLayoutBox()->NeedsCollectInlines());
    line_info->SetBlockInInlineLayoutResult(layout_result);

    // Early exit if the layout didn't succeed.
    if (layout_result->Status() != LayoutResult::kSuccess) {
      state_ = LineBreakState::kDone;
      return;
    }

    const auto& fragment = layout_result->GetPhysicalFragment();
    item_result->inline_size =
        LogicalFragment(constraint_space_.GetWritingDirection(), fragment)
            .InlineSize();

    item_result->should_create_line_box = !layout_result->IsSelfCollapsing();
    item_result->layout_result = layout_result;

    if (const auto* outgoing_block_break_token = To<BlockBreakToken>(
            layout_result->GetPhysicalFragment().GetBreakToken())) {
      // The block broke inside. If the block itself fits, but some content
      // inside overflowed, we now need to enter a parallel flow, i.e. resume
      // the block-in-inline in the next fragmentainer, but continue layout of
      // any actual inline content after the block-in-inline in the current
      // fragmentainer. Also do this if the incoming break token was in a
      // parallel flow. Then, by right, the outgoing break token should also be
      // in a parallel flow, but this inconsistency may occur if a column
      // spanner is discovered (at which point we avoid parallel flows before
      // it) in a subsequent outer fragmentainer after having overflowed a block
      // in a previous outer fragmentainer. See crbug.com/430249827
      if (outgoing_block_break_token->IsAtBlockEnd() ||
          (break_token_ && break_token_->IsInParallelFlow())) {
        const auto* parallel_token =
            InlineBreakToken::CreateForParallelBlockFlow(
                node_, current_, *outgoing_block_break_token);
        line_info->PropagateParallelFlowBreakToken(parallel_token);
      } else {
        // The block-in-inline broke inside, and it's still in the same flow.
        resume_block_in_inline_in_same_flow_ = true;
        move_past_block = false;
      }
    }
  } else {
    DCHECK(mode_ == LineBreakerMode::kMaxContent ||
           mode_ == LineBreakerMode::kMinContent);
    ComputeMinMaxContentSizeForBlockChild(item, item_result, this);
  }

  position_ += item_result->inline_size;
  line_info->SetIsBlockInInline();
  line_info->SetHasForcedBreak();
  is_forced_break_ = true;
  trailing_whitespace_ = WhitespaceState::kNone;

  // If there's no break inside the block, or if the break inside the block is
  // for a parallel flow, proceed to the next item for the next line.
  if (move_past_block) {
    MoveToNextOf(item);
  }
  state_ = LineBreakState::kDone;
}

bool LineBreaker::HandleRuby(LineInfo* line_info, LayoutUnit retry_size) {
  const RubyBreakTokenData* ruby_token = ruby_break_token_;
  // Clear ruby_break_token_ first because HandleRuby() might set it again due
  // to rewinding.
  ruby_break_token_ = nullptr;
  InlineItemTextIndex base_start = current_;
  wtf_size_t base_end_index;
  Vector<AnnotationBreakTokenData, 1> annotation_data;
  wtf_size_t open_column_item_index;
  if (!ruby_token) {
    open_column_item_index = current_.item_index;
    RubyItemIndexes ruby_indexes =
        ParseRubyInInlineItems(Items(), current_.item_index);
    base_end_index = ruby_indexes.base_end;
    if (Items()[base_end_index]->Type() == InlineItem::kCloseRubyColumn) {
      // No ruby-text. We don't need a kOpenRubyColumn result.
      return false;
    }
    UseCounter::Count(GetDocument(), WebFeature::kRenderRuby);
    DCHECK_EQ(Items()[base_end_index]->Type(), InlineItem::kOpenTag);
    DCHECK(Items()[base_end_index]->GetLayoutObject()->IsInlineRubyText());
    base_start = {current_.item_index + 1,
                  Items()[current_.item_index]->EndOffset()};

    wtf_size_t start = ruby_indexes.annotation_start;
    annotation_data.push_back(
        AnnotationBreakTokenData{{start, Items()[start]->StartOffset()},
                                 start,
                                 ruby_indexes.column_end});
  } else {
    open_column_item_index = ruby_token->open_column_item_index;
    base_end_index = ruby_token->ruby_base_end_item_index;
    annotation_data = ruby_token->annotation_data;
  }
  const InlineItem& item = *Items()[open_column_item_index];

  LineInfo base_line_info = CreateSubLineInfo(
      base_start, base_end_index, LineBreakerMode::kMaxContent, kIndefiniteSize,
      trailing_whitespace_, /* disable_trailing_whitespace_collapsing */ true);
  base_line_info.OverrideLineStyle(*current_style_);
  base_line_info.SetIsRubyBase();
  base_line_info.UpdateTextAlign();

  const wtf_size_t number_of_annotations = annotation_data.size();
  HeapVector<LineInfo, 1> annotation_line_list;
  annotation_line_list.reserve(number_of_annotations);
  for (const auto& data : annotation_data) {
    annotation_line_list.push_back(CreateSubLineInfo(
        data.start, data.end_item_index, LineBreakerMode::kMaxContent,
        kIndefiniteSize, WhitespaceState::kLeading));
    annotation_line_list.back().OverrideLineStyle(
        Items()[data.start_item_index]->GetLayoutObject()->StyleRef());
  }

  LayoutUnit ruby_size = MaxLineWidth(base_line_info, annotation_line_list);
  LayoutUnit available = RemainingAvailableWidth().ClampNegativeToZero();
  wtf_size_t ruby_index = line_info->Results().size();
  AnnotationOverhang overhang = GetOverhang(
      ruby_size, base_line_info, annotation_line_list, *line_info, ruby_index);
  if (!CanApplyStartOverhang(*line_info, ruby_index, *current_style_,
                             overhang.start)) {
    overhang.start = LayoutUnit();
  }
  bool is_monolithic = IsMonolithicRuby(base_line_info, annotation_line_list);
  if ((retry_size == kIndefiniteSize &&
       ruby_size <= available + overhang.start) ||
      is_monolithic) {
    if (mode_ == LineBreakerMode::kContent) {
      // Recreate lines because lines created with LineBreakerMode::kMaxContent
      // are not usable in InlineLayoutAlgorithm.
      base_line_info = CreateSubLineInfo(
          base_start, base_end_index, LineBreakerMode::kContent,
          kIndefiniteSize, trailing_whitespace_,
          /* disable_trailing_whitespace_collapsing */ true);
      for (wtf_size_t i = 0; i < annotation_data.size(); ++i) {
        annotation_line_list[i] = CreateSubLineInfo(
            annotation_data[i].start, annotation_data[i].end_item_index,
            LineBreakerMode::kContent, kIndefiniteSize,
            WhitespaceState::kLeading);
      }
    }

    InlineItemResult* result =
        AddRubyColumnResult(item, base_line_info, annotation_line_list,
                            annotation_data, ruby_size, ruby_token, *line_info);
    result->ruby_column->start_ruby_break_token = ruby_token;
    result->may_break_inside = !is_monolithic;
    position_ += ruby_size;
    // Move to a kCloseRubyColumn item.
    current_ = annotation_line_list[0].End();
    return true;
  }

  // Try to break the ruby column.

  LayoutUnit base_intrinsic_size = base_line_info.Width();
  LayoutUnit base_target = retry_size == kIndefiniteSize
                               ? (available * base_intrinsic_size / ruby_size)
                               : retry_size - 1;
  base_line_info = CreateSubLineInfo(base_start, base_end_index, mode_,
                                     base_target, trailing_whitespace_);
  // We assume a base LineInfo contains at least one InlineItemResult.
  // If it's zero, we can't adjust LogicalRubyColumns on bidi reorder.
  CHECK_GT(base_line_info.Results().size(), 0u);

  bool annotation_is_broken = false;
  for (wtf_size_t i = 0; i < number_of_annotations; ++i) {
    LineInfo& line = annotation_line_list[i];
    // If all items in the base line is consumed, we should consume all items
    // in annotation lines too.  The point just after the base line might be
    // non-breakable and we need to continue handling the following InlineItems
    // in such case. However it's very difficult if annotation items remain.
    LayoutUnit limit = kIndefiniteSize;
    LineBreakerMode mode = mode_;
    if (base_line_info.GetBreakToken()) {
      if (retry_size != kIndefiniteSize) {
        limit = line.Width() * base_line_info.Width() / base_intrinsic_size;
      } else {
        limit = available * line.Width() / ruby_size;
      }
    } else {
      // If the base is consumed entirely, the corresponding annotations should
      // be consumed entirely too.
      if (mode == LineBreakerMode::kMinContent) {
        mode = LineBreakerMode::kMaxContent;
      }
    }
    line = CreateSubLineInfo(annotation_data[i].start,
                             annotation_data[i].end_item_index, mode, limit,
                             WhitespaceState::kLeading);
    annotation_is_broken = annotation_is_broken || line.GetBreakToken();
  }

  ruby_size = MaxLineWidth(base_line_info, annotation_line_list);
  InlineItemResult* result =
      AddRubyColumnResult(item, base_line_info, annotation_line_list,
                          annotation_data, ruby_size, ruby_token, *line_info);
  result->ruby_column->start_ruby_break_token = ruby_token;
  result->may_break_inside = true;
  position_ += ruby_size;

  // If the base line and annotation lines have no BreakToken, we should add
  // them even though they are wider than the available width.  The
  // InlineItemResult for the ruby column may be rewound.
  if (!base_line_info.GetBreakToken() && !annotation_is_broken) {
    current_ = annotation_line_list[0].End();
    return true;
  }
  DCHECK(base_line_info.GetBreakToken());
  current_ = base_line_info.End();

  // We have a broken line, and need to provide a RubyBreakTokenData.
  Vector<AnnotationBreakTokenData, 1> breaks;
  breaks.reserve(number_of_annotations);
  for (wtf_size_t i = 0; i < number_of_annotations; ++i) {
    breaks.push_back(AnnotationBreakTokenData{
        annotation_line_list[i].End(), annotation_data[i].start_item_index,
        annotation_data[i].end_item_index});
  }
  result->ruby_column->end_ruby_break_token =
      MakeGarbageCollected<RubyBreakTokenData>(open_column_item_index,
                                               base_end_index, breaks);

  if (retry_size == kIndefiniteSize) {
    // We can't continue to handle following InlineItems if we break inside a
    // ruby column. So we try to rewind if necessary, then finish this line.
    HandleOverflowIfNeeded(line_info);
    if (!line_info->Results().empty()) {
      state_ = LineBreakState::kDone;
    }
  }
  return true;
}

bool LineBreaker::IsMonolithicRuby(
    const LineInfo& base_line,
    const HeapVector<LineInfo, 1>& annotation_line_list) const {
  // Not breakable if it's an inner ruby column of nested rubies.
  if (end_item_index_ != Items().size()) {
    return true;
  }

  if (!auto_wrap_) {
    return true;
  }

  // The base line is not breakable.
  if (base_line.Width() <= LayoutUnit()) {
    return true;
  }

  // We don't break rubies in text-wrap:balance and text-wrap:pretty
  // because the sum of broken ruby inline-size can be different from the
  // inline-size of a non-broken ruby.
  if (!node_.Style().ShouldWrapLineGreedy()) {
    return true;
  }

  // Not breakable if the number of the base letters is <= 4 and the number of
  // the annotation letters is <= 8.
  //
  // TODO(layout-dev): Should we take into account of East Asian Width?
  constexpr wtf_size_t kBaseLetterLimit = 4;
  constexpr wtf_size_t kAnnotationLetterLimit = 8;
  if (!base_line.GlyphCountIsGreaterThan(kBaseLetterLimit)) {
    auto iter = std::find_if(
        annotation_line_list.begin(), annotation_line_list.end(),
        [](const LineInfo& line) {
          return line.GlyphCountIsGreaterThan(kAnnotationLetterLimit);
        });
    if (iter == annotation_line_list.end()) {
      return true;
    }
  }

  return false;
}

LineInfo LineBreaker::CreateSubLineInfo(
    InlineItemTextIndex start,
    wtf_size_t end_item_index,
    LineBreakerMode mode,
    LayoutUnit limit,
    WhitespaceState initial_whitespace_state,
    bool disable_trailing_whitespace_collapsing) {
  bool disallow_auto_wrap = false;
  if (limit == kIndefiniteSize) {
    limit = LayoutUnit::Max();
    disallow_auto_wrap = true;
  }
  ExclusionSpace empty_exclusion_space;
  LeadingFloats empty_leading_floats;
  LineInfo sub_line_info;
  LineBreaker sub_line_breaker(
      node_, mode, constraint_space_, LineLayoutOpportunity(limit),
      empty_leading_floats,
      /* break_token */ nullptr,
      /* column_spanner_path */ nullptr, &empty_exclusion_space);
  sub_line_breaker.disallow_auto_wrap_ = disallow_auto_wrap;
  sub_line_breaker.SetInputRange(start, end_item_index,
                                 initial_whitespace_state, this);
  if (RuntimeEnabledFeatures::TabSizeInRubyBaseEnabled()) {
    // Tab stops occur at points that are multiples of the tab size from the
    // starting content edge of the preserved tab’s nearest block container
    // ancestor.
    // https://www.w3.org/TR/css-text-3/#white-space-phase-2
    sub_line_breaker.tab_stop_offset_ = position_ + tab_stop_offset_;
  }
  sub_line_breaker.disable_trailing_whitespace_collapsing_ =
      disable_trailing_whitespace_collapsing;
  // OverrideAvailableWidth() prevents HandleFloat() from updating
  // available_width_.
  sub_line_breaker.OverrideAvailableWidth(limit);
  sub_line_breaker.NextLine(&sub_line_info);
  if (disallow_auto_wrap) {
    // If this check fails, a forced break or a block-in-inline might
    // appear unexpectedly.
    CHECK(sub_line_breaker.IsAtEnd());
  }
  return sub_line_info;
}

InlineItemResult* LineBreaker::AddRubyColumnResult(
    const InlineItem& item,
    const LineInfo& base_line_info,
    const HeapVector<LineInfo, 1>& annotation_line_list,
    const Vector<AnnotationBreakTokenData, 1>& annotation_data_list,
    LayoutUnit ruby_size,
    bool is_continuation,
    LineInfo& line_info) {
  CHECK_EQ(item.Type(), InlineItem::kOpenRubyColumn);
  InlineItemResult* column_result = AddEmptyItem(item, &line_info);
  column_result->inline_size = ruby_size;
  auto* data = MakeGarbageCollected<InlineItemResultRubyColumn>();
  column_result->ruby_column = data;
  data->base_line = base_line_info;
  data->base_line.OverrideLineStyle(*current_style_);
  data->base_line.SetIsRubyBase();
  data->base_line.UpdateTextAlign();
  if (data->base_line.MayHaveRubyOverhang()) {
    line_info.SetMayHaveRubyOverhang();
  }
  line_info.SetHaveTextCombineOrRubyItem();
  data->is_continuation = is_continuation;

  data->annotation_line_list = annotation_line_list;
  for (wtf_size_t i = 0; i < annotation_line_list.size(); ++i) {
    LayoutObject& annotation_object =
        *Items()[annotation_data_list[i].start_item_index]->GetLayoutObject();
    data->annotation_line_list[i].OverrideLineStyle(
        annotation_object.StyleRef());
    data->annotation_line_list[i].SetIsRubyText();
    data->annotation_line_list[i].UpdateTextAlign();
    const LayoutObject* parent = annotation_object.Parent();
    data->position_list.push_back(
        parent->IsInlineRuby()
            ? parent->StyleRef(use_first_line_style_).GetRubyPosition()
            : RubyPosition::kOver);
  }
  DCHECK_EQ(data->annotation_line_list.size(), data->position_list.size());

  column_result->text_offset.end = annotation_line_list[0].EndTextOffset();
  column_result->should_create_line_box = true;
  column_result->can_break_after = CanBreakAfterRubyColumn(
      *column_result, annotation_data_list[0].end_item_index);

  if (base_line_info.Width() < ruby_size) {
    line_info.SetMayHaveRubyOverhang();
    wtf_size_t ruby_index = line_info.Results().size() - 1;
    AnnotationOverhang overhang =
        GetOverhang(*column_result, line_info, ruby_index);
    if (overhang.end > LayoutUnit()) {
      column_result->pending_end_overhang = overhang.end;
      maybe_have_end_overhang_ = true;
    }

    if (CanApplyStartOverhang(line_info, ruby_index,
                              column_result->item->GetLayoutObject()
                                  ? *column_result->item->Style()
                                  : *current_style_,
                              overhang.start)) {
      DCHECK_EQ(column_result->margins.inline_start, LayoutUnit());
      DCHECK_EQ((*column_result->ruby_column->base_line.MutableResults())[0]
                    .item->Type(),
                InlineItem::kRubyLinePlaceholder);
      (*column_result->ruby_column->base_line.MutableResults())[0]
          .margins.inline_start = -overhang.start;
      position_ -= overhang.start;
    }
  }
  trailing_whitespace_ = WhitespaceState::kUnknown;
  return column_result;
}

bool LineBreaker::CanBreakAfterRubyColumn(
    const InlineItemResult& column_result,
    wtf_size_t column_end_item_index) const {
  DCHECK_EQ(column_result.item->Type(), InlineItem::kOpenRubyColumn);
  DCHECK(column_result.ruby_column);
  if (!auto_wrap_) {
    return false;
  }
  const LineInfo& base_line = column_result.ruby_column->base_line;
  if (base_line.GetBreakToken()) {
    return true;
  }
  // Populate `text_content` with column_result's base text and text content
  // after `column_result`.
  StringBuilder text_content;
  unsigned base_text_length =
      base_line.EndTextOffset() - base_line.StartOffset();
  text_content.Append(
      StringView(Text(), base_line.StartOffset(), base_text_length));
  const InlineItem& next_item = *Items()[column_end_item_index];
  DCHECK_EQ(next_item.Type(), InlineItem::kCloseRubyColumn);
  unsigned ignorable_bidi_length = 1 + IgnorableBidiControlLength(next_item);
  text_content.Append(
      StringView(Text(), next_item.StartOffset() + ignorable_bidi_length));
  LazyLineBreakIterator break_iterator(break_iterator_,
                                       text_content.ReleaseString());
  return break_iterator.IsBreakable(base_text_length);
}

// Figure out if the float should be pushed after the current line. This
// should only be considered if we're not resuming the float, after having
// broken inside or before it in the previous fragmentainer. Otherwise we must
// attempt to place it.
bool LineBreaker::ShouldPushFloatAfterLine(
    UnpositionedFloat* unpositioned_float,
    LineInfo* line_info) {
  if (unpositioned_float->token) {
    return false;
  }

  LayoutUnit inline_margin_size =
      ComputeMarginBoxInlineSizeForUnpositionedFloat(unpositioned_float);

  LayoutUnit used_size = position_ + inline_margin_size +
                         ComputeFloatAncestorInlineEndSize(
                             constraint_space_, Items(), current_.item_index);
  bool can_fit_float =
      used_size <= line_opportunity_.AvailableFloatInlineSize().AddEpsilon();
  if (!can_fit_float) {
    // Floats need to know the current line width to determine whether to put it
    // into the current line or to the next line. Trailing spaces will be
    // removed if this line breaks here because they should be collapsed across
    // floats, but they are still included in the current line position at this
    // point. Exclude it when computing whether this float can fit or not.
    can_fit_float = used_size - TrailingCollapsibleSpaceWidth(line_info) <=
                    line_opportunity_.AvailableFloatInlineSize().AddEpsilon();
  }

  LayoutUnit bfc_block_offset =
      unpositioned_float->origin_bfc_offset.block_offset;

  // The float should be positioned after the current line if:
  //  - It can't fit within the non-shape area. (Assuming the current position
  //    also is strictly within the non-shape area).
  //  - It will be moved down due to block-start edge alignment.
  //  - It will be moved down due to clearance.
  //  - An earlier float has been pushed to the next fragmentainer.
  return !can_fit_float ||
         exclusion_space_->LastFloatBlockStart() > bfc_block_offset ||
         exclusion_space_->ClearanceOffset(unpositioned_float->ClearType(
             constraint_space_.Direction())) > bfc_block_offset;
}

// Performs layout and positions a float.
//
// If there is a known available_width (e.g. something has resolved the
// container BFC block offset) it will attempt to position the float on the
// current line.
// Additionally updates the available_width for the line as the float has
// (probably) consumed space.
//
// If the float is too wide *or* we already have UnpositionedFloats we add it
// as an UnpositionedFloat. This should be positioned *immediately* after we
// are done with the current line.
// We have this check if there are already UnpositionedFloats as we aren't
// allowed to position a float "above" another float which has come before us
// in the document.
void LineBreaker::HandleFloat(const InlineItem& item,
                              const BlockBreakToken* float_break_token,
                              LineInfo* line_info) {
  // When rewind occurs, an item may be handled multiple times.
  // Since floats are put into a separate list, avoid handling same floats
  // twice.
  // Ideally rewind can take floats out of floats list, but the difference is
  // sutble compared to the complexity.
  //
  // Additionally, we need to skip floats if we're retrying a line after a
  // fragmentainer break. In that case the floats associated with this line will
  // already have been processed.
  InlineItemResult* item_result = AddItem(item, line_info);
  auto index_before_float = current_;

  // Out-of-flow elements must be ignored for the text processing.
  // https://drafts.csswg.org/css-text-3/#text-encoding
  DCHECK(!item_result->can_break_after);
  if (!item_result->should_create_line_box) {
    // Allow to break after leading floats. This is not defined, but it's a
    // historical behavior some tests require, and is compatible with WebKit.
    item_result->can_break_after = auto_wrap_;
  }
  MoveToNextOf(item);

  // If we are currently computing our min/max-content size, simply append the
  // unpositioned floats to |LineInfo| and abort.
  if (mode_ != LineBreakerMode::kContent) {
    return;
  }

  // Make sure we populate the positioned_float inside the |item_result|.
  if (current_.item_index <= leading_floats_.HandledIndex() &&
      !leading_floats_.Empty()) {
    DCHECK_LT(leading_floats_index_, leading_floats_.Count());

    const LeadingFloat& leading_float =
        leading_floats_.At(leading_floats_index_++);
    item_result->positioned_float = leading_float.positioned_float;
    if (leading_float.parallel_flow_break_token) {
      line_info->PropagateParallelFlowBreakToken(
          leading_float.parallel_flow_break_token);
    }

    // Save a backup copy of `exclusion_space_` even if leading floats don't
    // modify it. See `RewindFloats`.
    DCHECK(exclusion_space_);
    item_result->exclusion_space_before_position_float.CopyFrom(
        *exclusion_space_);
    return;
  }

  const LayoutUnit bfc_block_offset = line_opportunity_.bfc_block_offset;
  LineClampFloatState line_clamp_state =
      constraint_space_.GetLineClampData().FloatState();

  const BlockNode float_node(To<LayoutBox>(item.GetLayoutObject()));
  UnpositionedFloat unpositioned_float(
      float_node, float_break_token, constraint_space_.AvailableSize(),
      float_node.IsReplaced()
          ? constraint_space_.ReplacedChildPercentageResolutionSize()
          : constraint_space_.PercentageResolutionSize(),
      {constraint_space_.GetBfcOffset().line_offset, bfc_block_offset},
      constraint_space_, node_.Style(),
      constraint_space_.FragmentainerBlockSize(),
      constraint_space_.FragmentainerOffset(), line_clamp_state);

  bool float_after_line =
      ShouldPushFloatAfterLine(&unpositioned_float, line_info);

  // Check if we already have a pending float. That's because a float cannot be
  // higher than any block or floated box generated before.
  if (HasUnpositionedFloats(line_info->Results()) || float_after_line) {
    item_result->has_unpositioned_floats = true;
    return;
  }

  // Save a backup copy of `exclusion_space_` for when rewinding. See
  // `RewindFloats`.
  DCHECK(exclusion_space_);
  item_result->exclusion_space_before_position_float.CopyFrom(
      *exclusion_space_);

  item_result->positioned_float =
      PositionFloat(&unpositioned_float, exclusion_space_);
  // Ensure `NeedsCollectInlines` isn't set, or it may cause security risks.
  CHECK(!node_.GetLayoutBox()->NeedsCollectInlines());

  if (constraint_space_.HasBlockFragmentation()) {
    if (const auto* break_token = item_result->positioned_float->BreakToken()) {
      const auto* parallel_token = InlineBreakToken::CreateForParallelBlockFlow(
          node_, index_before_float, *break_token);
      line_info->PropagateParallelFlowBreakToken(parallel_token);
      if (item_result->positioned_float->minimum_space_shortage) {
        line_info->PropagateMinimumSpaceShortage(
            item_result->positioned_float->minimum_space_shortage);
        DCHECK_EQ(item_result->positioned_float->tallest_unbreakable_block_size,
                  LayoutUnit());
      } else if (item_result->positioned_float
                     ->tallest_unbreakable_block_size) {
        line_info->PropagateTallestUnbreakableBlockSize(
            item_result->positioned_float->tallest_unbreakable_block_size);
      }
      if (break_token->IsBreakBefore()) {
        return;
      }
    }
  }

  UpdateLineOpportunity();
}

void LineBreaker::UpdateLineOpportunity() {
  const LayoutUnit bfc_block_offset = line_opportunity_.bfc_block_offset;
  LayoutOpportunity opportunity = exclusion_space_->FindLayoutOpportunity(
      {constraint_space_.GetBfcOffset().line_offset, bfc_block_offset},
      constraint_space_.AvailableSize().inline_size,
      constraint_space_.Direction());

  DCHECK_EQ(bfc_block_offset, opportunity.rect.BlockStartOffset());

  line_opportunity_ = opportunity.ComputeLineLayoutOpportunity(
      constraint_space_, line_opportunity_.line_block_size, LayoutUnit());
  UpdateAvailableWidth();

  DCHECK_GE(AvailableWidth(), LayoutUnit());
}

// Restore the states changed by `HandleFloat` to before
// `item_results[new_end]`.
void LineBreaker::RewindFloats(unsigned new_end,
                               LineInfo& line_info,
                               InlineItemResults& item_results) {
  for (const InlineItemResult& item_result :
       base::span(item_results).subspan(new_end)) {
    if (item_result.positioned_float) {
      const unsigned item_index = item_result.item_index;
      line_info.RemoveParallelFlowBreakToken(item_index);

      // Adjust `leading_floats_index_` if this is a leading float. See
      // `HandleFloat` and `PositionLeadingFloats`.
      if (item_index < leading_floats_.HandledIndex()) {
        for (unsigned i = 0; i < leading_floats_.Count(); ++i) {
          if (leading_floats_.At(i).positioned_float.layout_result ==
              item_result.positioned_float->layout_result) {
            leading_floats_index_ = i;
            // Need to restore `exclusion_space_` even if leading floats don't
            // modify `exclusion_space_`, because there may be following
            // non-leading floats that modified it.
            break;
          }
        }
      }

      *exclusion_space_ = item_result.exclusion_space_before_position_float;
      UpdateLineOpportunity();
      break;
    }
  }
}

void LineBreaker::HandleInitialLetter(const InlineItem& item,
                                      LineInfo* line_info) {
  // TODO(crbug.com/1276900): We should check behavior when line breaking
  // after initial letter box.
  HandleAtomicInline(item, line_info);
}

void LineBreaker::HandleOutOfFlowPositioned(const InlineItem& item,
                                            LineInfo* line_info) {
  DCHECK_EQ(item.Type(), InlineItem::kOutOfFlowPositioned);
  InlineItemResult* item_result = AddItem(item, line_info);

  // Out-of-flow elements must be ignored for the text processing.
  // https://drafts.csswg.org/css-text-3/#text-encoding
  // But when the point is breakable, whether to break before or after isn't
  // well-defined.
  DCHECK(!item_result->can_break_after);
  InlineItemResults& item_results = *line_info->MutableResults();
  if (item_results.size() >= 2) {
    InlineItemResult* last_item_result = std::prev(item_result);
    if (last_item_result->IsEmptyText() && !last_item_result->can_break_after)
        [[unlikely]] {
      // Text+SP OP OOF Text => !break_after.
      //     line-breaking-018, line-breaking-019
      // Text+SP OP Empty OOF Text => break_after.
      //     position-absolute-in-inline-004
      ComputeCanBreakAfter(item_result, auto_wrap_, break_iterator_);
    } else {
      item_result->can_break_after = last_item_result->can_break_after;
    }
  }

  MoveToNextOf(item);
}

bool LineBreaker::ComputeOpenTagResult(const InlineItem& item,
                                       const ConstraintSpace& constraint_space,
                                       bool is_in_svg_text,
                                       InlineItemResult* item_result) {
  DCHECK_EQ(item.Type(), InlineItem::kOpenTag);
  DCHECK(item.Style());
  const ComputedStyle& style = *item.Style();
  if (!is_in_svg_text && item.ShouldCreateBoxFragment() &&
      (style.HasBorder() || style.MayHavePadding() || style.MayHaveMargin())) {
    item_result->borders = ComputeLineBorders(style);
    item_result->padding = ComputeLinePadding(constraint_space, style);
    item_result->margins = ComputeLineMarginsForSelf(constraint_space, style);
    item_result->inline_size = item_result->margins.inline_start +
                               item_result->borders.inline_start +
                               item_result->padding.inline_start;
    return true;
  }
  return false;
}

void LineBreaker::HandleOpenTag(const InlineItem& item, LineInfo* line_info) {
  DCHECK_EQ(item.Type(), InlineItem::kOpenTag);

  InlineItemResult* item_result = AddItem(item, line_info);
  DCHECK(item.Style());
  const ComputedStyle& style = *item.Style();
  if (ComputeOpenTagResult(item, constraint_space_, is_svg_text_,
                           item_result)) {
    // Negative margins on open tags may bring the position back. Update
    // |state_| if that happens.
    if (item_result->inline_size < 0 && state_ == LineBreakState::kTrailing)
        [[unlikely]] {
      LayoutUnit available_width = AvailableWidthToFit();
      if (position_ > available_width &&
          position_ + item_result->inline_size <= available_width) {
        state_ = LineBreakState::kContinue;
      }
    }

    position_ += item_result->inline_size;

    // While the spec defines "non-zero margins, padding, or borders" prevents
    // line boxes to be zero-height, tests indicate that only inline direction
    // of them do so. See should_create_line_box_.
    // Force to create a box, because such inline boxes affect line heights.
    if (!item_result->should_create_line_box && !item.IsEmptyItem())
      item_result->should_create_line_box = true;
  }

  if (style.BoxDecorationBreak() == EBoxDecorationBreak::kClone) [[unlikely]] {
    // Compute even when no margins/borders/padding to ensure correct counting.
    has_cloned_box_decorations_ = true;
    disable_score_line_break_ = true;
    ++cloned_box_decorations_count_;
    cloned_box_decorations_end_size_ += item_result->margins.inline_end +
                                        item_result->borders.inline_end +
                                        item_result->padding.inline_end;
    UpdateAvailableWidthFromBaseAvailableWidth();
  }

  bool was_auto_wrap = auto_wrap_;
  SetCurrentStyle(style);
  MoveToNextOf(item);

  DCHECK(!item_result->can_break_after);
  const InlineItemResults& item_results = line_info->Results();
  if (!was_auto_wrap && auto_wrap_ && item_results.size() >= 2) [[unlikely]] {
    if (IsPreviousItemOfType(InlineItem::kText)) {
      ComputeCanBreakAfter(std::prev(item_result), auto_wrap_, break_iterator_);
    }
  }
}

void LineBreaker::HandleCloseTag(const InlineItem& item, LineInfo* line_info) {
  InlineItemResult* item_result = AddItem(item, line_info);

  if (!is_svg_text_) {
    DCHECK(item.Style());
    const ComputedStyle& style = *item.Style();
    item_result->inline_size = ComputeInlineEndSize(constraint_space_, &style);
    position_ += item_result->inline_size;

    if (!item_result->should_create_line_box && !item.IsEmptyItem())
      item_result->should_create_line_box = true;

    if (style.BoxDecorationBreak() == EBoxDecorationBreak::kClone)
        [[unlikely]] {
      DCHECK_GT(cloned_box_decorations_count_, 0u);
      --cloned_box_decorations_count_;
      DCHECK_GE(cloned_box_decorations_end_size_, item_result->inline_size);
      cloned_box_decorations_end_size_ -= item_result->inline_size;
      UpdateAvailableWidthFromBaseAvailableWidth();
    }
  }
  DCHECK(item.GetLayoutObject() && item.GetLayoutObject()->Parent());
  bool was_auto_wrap = auto_wrap_;
  SetCurrentStyle(item.GetLayoutObject()->Parent()->StyleRef());
  MoveToNextOf(item);

  // If the line can break after the previous item, prohibit it and allow break
  // after this close tag instead. Even when the close tag has "nowrap", break
  // after it is allowed if the line is breakable after the previous item.
  const InlineItemResults& item_results = line_info->Results();
  if (item_results.size() >= 2) {
    InlineItemResult* last = std::prev(item_result);
    if (IsA<LayoutTextCombine>(last->item->GetLayoutObject())) [[unlikely]] {
      // |can_break_after| for close tag should be as same as text-combine box.
      // See "text-combine-upright-break-inside-001a.html"
      // e.g. A<tcy style="white-space: pre">x y</tcy>B
      item_result->can_break_after = last->can_break_after;
      return;
    }
    if (last->can_break_after) {
      // A break opportunity before a close tag always propagates to after the
      // close tag.
      item_result->can_break_after = true;
      last->can_break_after = false;
      return;
    }
    if (was_auto_wrap) {
      // We can break before a breakable space if we either:
      //   a) allow breaking before a white space, or
      //   b) the break point is preceded by another breakable space.
      bool preceded_by_breakable_space =
          item_result->EndOffset() > 0 &&
          IsBreakableSpace(Text()[item_result->EndOffset() - 1]);
      item_result->can_break_after =
          IsBreakableSpace(Text()[item_result->EndOffset()]) &&
          (!current_style_->ShouldBreakOnlyAfterWhiteSpace() ||
           preceded_by_breakable_space) &&
          (!RuntimeEnabledFeatures::LineBreakAfterSpaceBeforeOpenTagEnabled() ||
           !IsNextNonBidiControlItemOpenTag());
      return;
    }
    if (auto_wrap_ && !IsBreakableSpace(Text()[item_result->EndOffset() - 1]))
      ComputeCanBreakAfter(item_result, auto_wrap_, break_iterator_);
  }
}

// Handles when the last item overflows.
// At this point, item_results does not fit into the current line, and there
// are no break opportunities in item_results.back().
void LineBreaker::HandleOverflow(LineInfo* line_info) {
  const LayoutUnit available_width = AvailableWidthToFit();
  DCHECK_GT(position_, available_width);

  // Save the hyphenation states before we may make changes.
  InlineItemResults* item_results = line_info->MutableResults();
  std::optional<wtf_size_t> hyphen_index_before = hyphen_index_;
  if (HasHyphen()) [[unlikely]] {
    position_ -= RemoveHyphen(item_results);
  }

  // Compute the width needing to rewind. When |width_to_rewind| goes negative,
  // items can fit within the line.
  LayoutUnit width_to_rewind = position_ - available_width;

  // Keep track of the shortest break opportunity.
  unsigned break_before = 0;

  // True if there is at least one item that has `break-word`.
  bool has_break_anywhere_if_overflow = break_anywhere_if_overflow_;

  // Search for a break opportunity that can fit.
  for (unsigned i = item_results->size(); i;) {
    InlineItemResult* item_result = &(*item_results)[--i];
    has_break_anywhere_if_overflow |= item_result->break_anywhere_if_overflow;

    // Try to break after this item.
    if (i < item_results->size() - 1 && item_result->can_break_after) {
      if (width_to_rewind <= 0) {
        position_ = available_width + width_to_rewind;
        RewindOverflow(i + 1, line_info);
        return;
      }
      break_before = i + 1;
    }

    // Compute the position after this item was removed entirely.
    width_to_rewind -= item_result->inline_size;

    // Try next if still does not fit.
    if (width_to_rewind > 0)
      continue;

    DCHECK(item_result->item);
    const InlineItem& item = *item_result->item;
    if (item.Type() == InlineItem::kText) {
      if (!item_result->Length()) {
        // Empty text items are trailable, see `HandleEmptyText`.
        continue;
      }
      DCHECK(item_result->shape_result ||
             (item_result->break_anywhere_if_overflow &&
              !override_break_anywhere_) ||
             // |HandleTextForFastMinContent| can produce an item without
             // |ShapeResult|. In this case, it is not breakable.
             (mode_ == LineBreakerMode::kMinContent &&
              !item_result->may_break_inside));
      // If space is available, and if this text is breakable, part of the text
      // may fit. Try to break this item.
      if (width_to_rewind < 0 && item_result->may_break_inside) {
        const LayoutUnit item_available_width = -width_to_rewind;
        // Make sure the available width is smaller than the current width. The
        // break point must not be at the end when e.g., the text fits but its
        // right margin does not or following items do not.
        const LayoutUnit min_available_width = item_result->inline_size - 1;
        // If |inline_size| is zero (e.g., `font-size: 0`), |BreakText| cannot
        // make it shorter. Take the previous break opportunity.
        if (min_available_width <= 0) [[unlikely]] {
          if (BreakTextAtPreviousBreakOpportunity(*item_results, i)) {
            RewindOverflow(i + 1, line_info);
            return;
          }
          continue;
        }
        const ComputedStyle* was_current_style = current_style_;
        SetCurrentStyle(*item.Style());
        const InlineItemResult item_result_before = *item_result;
        BreakText(item_result, item, *item.TextShapeResult(),
                  std::min(item_available_width, min_available_width),
                  item_available_width, line_info);
#if DCHECK_IS_ON()
        item_result->CheckConsistency(true);
#endif

        // If BreakText() changed this item small enough to fit, break here.
        if (item_result->can_break_after &&
            item_result->inline_size <= item_available_width &&
            item_result->EndOffset() < item_result_before.EndOffset()) {
          DCHECK_LT(item_result->EndOffset(), item.EndOffset());

          // If this is the last item, adjust it to accommodate the change.
          const unsigned new_end = i + 1;
          CHECK_LE(new_end, item_results->size());
          if (new_end == item_results->size()) {
            position_ =
                available_width + width_to_rewind + item_result->inline_size;
            DCHECK_EQ(position_, line_info->ComputeWidth());
            current_ = item_result->End();
            items_data_->AssertOffset(current_);
            HandleTrailingSpaces(item, line_info);
            return;
          }

          state_ = LineBreakState::kTrailing;
          Rewind(new_end, line_info);
          return;
        }

        // Failed to break to fit. Restore to the original state.
        //
        // Generally, breaking at a smaller width should result in a shorter or
        // equal end offset; i.e., `item_result->EndOffset()` should be
        // `<= item_result_before.EndOffset()`.
        // However, due to reshaping (especially at huge font sizes) or
        // float-to-LayoutUnit rounding mismatches, the `EndOffset()` becoming
        // larger is possible.
        if (HasHyphen()) [[unlikely]] {
          RemoveHyphen(item_results);
        }
        *item_result = std::move(item_result_before);
        SetCurrentStyle(*was_current_style);
      }
    } else if (item_result->IsRubyColumn()) {
      // If space is available, and if this ruby column is breakable, part of
      // the ruby column may fit. Try to break this item.
      if (width_to_rewind < 0 && item_result->may_break_inside) {
        const auto& rewound_ruby_column = *item_result->ruby_column;
        const LineInfo& base_line = rewound_ruby_column.base_line;
        Rewind(i, line_info);
        HandleRuby(line_info, base_line.Width());
        const LineInfo& new_base_line =
            line_info->Results().back().ruby_column->base_line;
        LayoutUnit new_width = new_base_line.Width();
        if (new_width > LayoutUnit() && new_width != base_line.Width()) {
          // We succeeded to shorten the ruby column.
          state_ = LineBreakState::kDone;
          return;
        } else if (i == 0 && new_base_line.GetBreakToken()) {
          // We couldn't shorten the ruby column and can't rewind more.
          // We accept this result.
          state_ = LineBreakState::kDone;
          return;
        }
      }
    }
  }

  if (applied_text_indent_ && width_to_rewind > LayoutUnit() &&
      is_first_formatted_line_ && !leading_floats_.Empty()) {
    // If there is no inflow content and there are only leading floats, also
    // rewind text indentation. The idea here is that text-indent alone
    // shouldn't contribute to overflow (and it doesn't even belong on this
    // line, since we've rewound past everything else), so that we won't attempt
    // to place inline content in the layout opportunity below the floats, but
    // rather create a "line" with just the leading floats, and then another
    // line for any real inline content.
    //
    // This matters for block fragmentation. If there's one line box with both
    // leading floats and a real inline content below them, the fragmentation
    // engine has no means of inserting a fragmentation break between the float
    // and the inline content, since lines are monolithic.
    position_ -= applied_text_indent_;
    width_to_rewind -= applied_text_indent_;
    applied_text_indent_ = LayoutUnit();
    if (width_to_rewind <= LayoutUnit()) {
      state_ = LineBreakState::kDone;
      return;
    }
  }

  // Reaching here means that the rewind point was not found.

  if (break_iterator_.BreakType() == LineBreakType::kPhrase &&
      !disable_phrase_ && mode_ == LineBreakerMode::kContent) {
    // If the phrase line break overflowed, retry with the normal line break.
    disable_phrase_ = true;
    RetryAfterOverflow(line_info, item_results);
    return;
  }

  if (!override_break_anywhere_ && has_break_anywhere_if_overflow) {
    // Overflow occurred but `overflow-wrap` is set. Change the break type and
    // retry the line breaking.
    override_break_anywhere_ = true;
    RetryAfterOverflow(line_info, item_results);
    return;
  }

  // Let this line overflow.
  line_info->SetHasOverflow();

  // TODO(kojii): `ScoreLineBreaker::ComputeScores` gets confused if there're
  // overflowing lines. Disable the score line break for now. E.g.:
  //   css2.1/t1601-c547-indent-01-d.html
  //   virtual/text-antialias/international/bdi-neutral-wrapped.html
  disable_score_line_break_ = true;
  // The bisect line breaker doesn't support overflowing content.
  disable_bisect_line_break_ = true;

  // Restore the hyphenation states to before the loop if needed.
  // The loop above may have shrunk `item_results` via `Rewind()` (e.g. when
  // handling a ruby column), so the saved index can be stale. Only restore
  // the hyphen if the item it referenced still exists. See crbug.com/435058045.
  DCHECK(!HasHyphen());
  if (hyphen_index_before && *hyphen_index_before < item_results->size())
      [[unlikely]] {
    position_ += AddHyphen(item_results, *hyphen_index_before);
  }

  // If there was a break opportunity, the overflow should stop there.
  if (break_before) {
    RewindOverflow(break_before, line_info);
    return;
  }

  if (CanBreakAfterLast(*item_results)) {
    state_ = LineBreakState::kTrailing;
    return;
  }

  // No break opportunities. Break at the earliest break opportunity.
  DCHECK(std::ranges::all_of(*item_results,
                             [](const InlineItemResult& item_result) {
                               return !item_result.can_break_after;
                             }));
  state_ = LineBreakState::kOverflow;
}

void LineBreaker::RetryAfterOverflow(LineInfo* line_info,
                                     InlineItemResults* item_results) {
  // `ScoreLineBreaker` doesn't support multi-pass line breaking.
  disable_score_line_break_ = true;
  // The bisect line breaker doesn't support multi-pass line breaking.
  disable_bisect_line_break_ = true;

  state_ = LineBreakState::kContinue;

  // Rewind all items.
  //
  // Also `SetCurrentStyle` forcibly, because the retry uses different
  // conditions such as `override_break_anywhere_`.
  //
  // TODO(kojii): Not all items need to rewind, but such case is rare and
  // rewinding all items simplifes the code.
  if (!item_results->empty()) {
    SetCurrentStyleForce(ComputeCurrentStyle(0, line_info));
    Rewind(0, line_info);
  } else {
    SetCurrentStyleForce(*current_style_);
  }
  ResetRewindLoopDetector();
}

// Rewind to |new_end| on overflow. If trailable items follow at |new_end|, they
// are included (not rewound).
void LineBreaker::RewindOverflow(unsigned new_end, LineInfo* line_info) {
  const InlineItemResults& item_results = line_info->Results();
  DCHECK_LT(new_end, item_results.size());

  unsigned open_tag_count = 0;
  const String& text = Text();
  for (unsigned index = new_end; index < item_results.size(); index++) {
    const InlineItemResult& item_result = item_results[index];
    DCHECK(item_result.item);
    const InlineItem& item = *item_result.item;
    if (item.Type() == InlineItem::kText) {
      // Text items are trailable if they start with trailable spaces.
      if (!item_result.Length()) {
        // Empty text items are trailable, see `HandleEmptyText`.
        continue;
      }
      if (item_result.shape_result ||  // kNoResultIfOverflow if 'break-word'
          (break_anywhere_if_overflow_ && !override_break_anywhere_)) {
        DCHECK(item.Style());
        const ComputedStyle& style = *item.Style();
        if (style.ShouldWrapLine() && !style.ShouldBreakSpaces() &&
            IsBreakableSpace(text[item_result.StartOffset()])) {
          // If all characters are trailable spaces, check the next item.
          if (item_result.shape_result &&
              IsAllBreakableSpaces(text, item_result.StartOffset() + 1,
                                   item_result.EndOffset())) {
            continue;
          }
          // If this item starts with spaces followed by non-space characters,
          // the line should break after the spaces. Rewind to before this item.
          //
          // After the rewind, we want to |HandleTrailingSpaces| in this |item|,
          // but |Rewind| may have failed when we have floats. Set the |state_|
          // to |kTrailing| and let the next |HandleText| to handle this.
          state_ = LineBreakState::kTrailing;
          Rewind(index, line_info);
          return;
        }
      }
    } else if (item.Type() == InlineItem::kControl) {
      // All control characters except newline are trailable if auto_wrap. We
      // should not have rewound if there was a newline, so safe to assume all
      // controls are trailable.
      DCHECK_NE(text[item_result.StartOffset()], uchar::kLineFeed);
      DCHECK(item.Style());
      const ComputedStyle& style = *item.Style();
      if (style.ShouldWrapLine() && !style.ShouldBreakSpaces()) {
        continue;
      }
    } else if (item.Type() == InlineItem::kOpenTag) {
      // Open tags are ambiguous. This open tag is not trailable:
      //   <span>text
      // but these are trailable:
      //   <span> text
      //   <span></span>text
      //   <span> </span>text
      // Count the nest-level and mark where the nest-level was 0.
      if (!open_tag_count)
        new_end = index;
      open_tag_count++;
      continue;
    } else if (item.Type() == InlineItem::kCloseTag) {
      if (open_tag_count > 0)
        open_tag_count--;
      continue;
    } else if (IsTrailableItemType(item.Type())) {
      continue;
    }

    // Found a non-trailable item. Rewind to before the item, or to before the
    // open tag if the nest-level is not zero.
    if (open_tag_count)
      index = new_end;
    state_ = LineBreakState::kDone;
    DCHECK(!line_info->IsLastLine());
    Rewind(index, line_info);
    return;
  }

  // The open tag turned out to be non-trailable if the nest-level is not zero.
  // Rewind to before the open tag.
  if (open_tag_count) {
    state_ = LineBreakState::kDone;
    DCHECK(!line_info->IsLastLine());
    Rewind(new_end, line_info);
    return;
  }

  // All items are trailable. Done without rewinding.
  trailing_whitespace_ = WhitespaceState::kUnknown;
  position_ = line_info->ComputeWidth();
  state_ = LineBreakState::kDone;
  DCHECK(!line_info->IsLastLine());
  if (IsAtEnd()) {
    line_info->SetIsLastLine(true);
  }
}

void LineBreaker::Rewind(unsigned new_end, LineInfo* line_info) {
  InlineItemResults& item_results = *line_info->MutableResults();
  DCHECK_LT(new_end, item_results.size());
  if (last_rewind_) {
    // Detect rewind-loop. If we're trying to rewind to the same index twice,
    // we're in the infinite loop.
    if (current_.item_index == last_rewind_->from_item_index &&
        new_end == last_rewind_->to_index) {
      NOTREACHED();
    }
    last_rewind_.emplace(RewindIndex{current_.item_index, new_end});
  }

  // Check if floats are being rewound.
  RewindFloats(new_end, *line_info, item_results);

  if (new_end) {
    // Use |results[new_end - 1].end_offset| because it may have been truncated
    // and may not be equal to |results[new_end].start_offset|.
    MoveToNextOf(item_results[new_end - 1]);
    trailing_whitespace_ = WhitespaceState::kUnknown;
    // When space item is followed by empty text, we will break line at empty
    // text. See http://crbug.com/1104534
    // Example:
    //   [0] kOpeNTag 0-0 <i>
    //   [1] kText 0-10 "012345679"
    //   [2] kOpenTag 10-10 <b> <= |item_results[new_end - 1]|
    //   [3] kText 10-10 ""     <= |current_.item_index|
    //   [4] kText 10-11 " "
    //   [5] kCloseTag 11-11 <b>
    //   [6] kText 11-13 "ab"
    //   [7] kCloseTag 13-13 <i>
    // Note: We can have multiple empty |LayoutText| by ::first-letter, nested
    // <q>, Text.splitText(), etc.
    const InlineItems& items = Items();
    while (!IsAtEnd() &&
           items[current_.item_index]->Type() == InlineItem::kText &&
           !items[current_.item_index]->Length()) {
      HandleEmptyText(*items[current_.item_index], line_info);
    }
  } else {
    // Rewinding all items.
    current_ = line_info->Start();
    if (!item_results.empty() && item_results.front().IsRubyColumn()) {
      ruby_break_token_ =
          item_results.front().ruby_column->start_ruby_break_token;
    }
    trailing_whitespace_ = WhitespaceState::kLeading;
    maybe_have_end_overhang_ = false;
  }
  SetCurrentStyle(ComputeCurrentStyle(new_end, line_info));

  item_results.Shrink(new_end);

  trailing_collapsible_space_.reset();
  if (hyphen_index_ && *hyphen_index_ >= new_end) [[unlikely]] {
    hyphen_index_.reset();
  }
  if (!hyphen_index_ && has_any_hyphens_) [[unlikely]] {
    RestoreLastHyphen(&item_results);
  }
  position_ = line_info->ComputeWidth();
  if (has_cloned_box_decorations_) [[unlikely]] {
    RecalcClonedBoxDecorations();
  }
}

// Returns the style to use for |item_result_index|. Normally when handling
// items sequentially, the current style is updated on open/close tag. When
// rewinding, this function computes the style for the specified item.
const ComputedStyle& LineBreaker::ComputeCurrentStyle(
    unsigned item_result_index,
    LineInfo* line_info) const {
  const InlineItemResults& item_results = line_info->Results();

  // Use the current item if it can compute the current style.
  const InlineItem* item = item_results[item_result_index].item.Get();
  DCHECK(item);
  if (item->Type() == InlineItem::kText ||
      item->Type() == InlineItem::kCloseTag) {
    DCHECK(item->Style());
    return *item->Style();
  }

  // Otherwise look back an item that can compute the current style.
  while (item_result_index) {
    item = item_results[--item_result_index].item.Get();
    if (item->Type() == InlineItem::kText ||
        item->Type() == InlineItem::kOpenTag) {
      DCHECK(item->Style());
      return *item->Style();
    }
    if (item->Type() == InlineItem::kCloseTag) {
      return item->GetLayoutObject()->Parent()->StyleRef();
    }
  }

  // Use the style at the beginning of the line if no items are available.
  if (break_token_ && break_token_->Style())
    return *break_token_->Style();
  return line_info->LineStyle();
}

void LineBreaker::SetCurrentStyle(const ComputedStyle& style) {
  if (&style == current_style_) {
#if EXPENSIVE_DCHECKS_ARE_ON()
    // Check that cache fields are already setup correctly.
    DCHECK_EQ(auto_wrap_, ShouldAutoWrap(style));
    if (auto_wrap_) {
      DCHECK_EQ(break_iterator_.IsSoftHyphenEnabled(),
                style.GetHyphens() != Hyphens::kNone &&
                    (disable_phrase_ ||
                     style.WordBreak() != EWordBreak::kAutoPhrase));
      DCHECK_EQ(break_iterator_.Locale(), style.GetFontDescription().Locale());
    }
    ShapeResultSpacing spacing(spacing_.Text(), is_svg_text_);
    spacing.SetSpacing(style.GetFont()->GetFontDescription());
    DCHECK_EQ(spacing.LetterSpacing(), spacing_.LetterSpacing());
    DCHECK_EQ(spacing.WordSpacing(), spacing_.WordSpacing());
#endif  //  EXPENSIVE_DCHECKS_ARE_ON()
    return;
  }
  SetCurrentStyleForce(style);
}

void LineBreaker::SetCurrentStyleForce(const ComputedStyle& style) {
  current_style_ = &style;

  const FontDescription& font_description = style.GetFontDescription();
  spacing_.SetSpacing(font_description);

  auto_wrap_ = ShouldAutoWrap(style);
  if (auto_wrap_) {
    DCHECK(!is_text_combine_);
    break_iterator_.SetLocale(font_description.Locale());
    Hyphens hyphens = style.GetHyphens();
    const LineBreak line_break = style.GetLineBreak();
    if (line_break == LineBreak::kAnywhere) [[unlikely]] {
      break_iterator_.SetStrictness(LineBreakStrictness::kDefault);
      break_iterator_.SetBreakType(LineBreakType::kBreakCharacter);
      break_anywhere_if_overflow_ = false;
    } else {
      break_iterator_.SetStrictness(StrictnessFromLineBreak(line_break));
      LineBreakType line_break_type;
      switch (style.WordBreak()) {
        case EWordBreak::kNormal:
          line_break_type = LineBreakType::kNormal;
          break_anywhere_if_overflow_ = false;
          break;
        case EWordBreak::kBreakAll:
          line_break_type = LineBreakType::kBreakAll;
          break_anywhere_if_overflow_ = false;
          break;
        case EWordBreak::kBreakWord:
          line_break_type = LineBreakType::kNormal;
          // overflow-wrap doesn't take effect if the line has a line-clamp
          // ellipsis, and neither does `word-break: break-word`.
          break_anywhere_if_overflow_ = !line_clamp_ellipsis_width_;
          break;
        case EWordBreak::kKeepAll:
          line_break_type = LineBreakType::kKeepAll;
          break_anywhere_if_overflow_ = false;
          break;
        case EWordBreak::kAutoPhrase:
          if (disable_phrase_) [[unlikely]] {
            line_break_type = LineBreakType::kNormal;
          } else {
            line_break_type = LineBreakType::kPhrase;
            hyphens = Hyphens::kNone;
            UseCounter::Count(GetDocument(), WebFeature::kLineBreakPhrase);
          }
          break_anywhere_if_overflow_ = false;
          break;
      }
      if (!break_anywhere_if_overflow_ && !line_clamp_ellipsis_width_) {
        // `overflow-wrap: anywhere` affects both layout and min-content, while
        // `break-word` affects layout but not min-content.
        // This only takes effect if this line doesn't have a line-clamp
        // ellipsis.
        const EOverflowWrap overflow_wrap = style.OverflowWrap();
        break_anywhere_if_overflow_ =
            overflow_wrap == EOverflowWrap::kAnywhere ||
            (overflow_wrap == EOverflowWrap::kBreakWord &&
             mode_ == LineBreakerMode::kContent);
      }
      if (break_anywhere_if_overflow_) [[unlikely]] {
        if (override_break_anywhere_) [[unlikely]] {
          line_break_type = LineBreakType::kBreakCharacter;
        } else if (mode_ == LineBreakerMode::kMinContent) [[unlikely]] {
          override_break_anywhere_ = true;
          line_break_type = LineBreakType::kBreakCharacter;
        }
      }
      break_iterator_.SetBreakType(line_break_type);
    }

    if (hyphens == Hyphens::kNone) [[unlikely]] {
      break_iterator_.EnableSoftHyphen(false);
      hyphenation_ = nullptr;
    } else {
      break_iterator_.EnableSoftHyphen(true);
      hyphenation_ = style.GetHyphenationWithLimits();
    }

    if (style.ShouldBreakSpaces()) {
      break_iterator_.SetBreakSpace(BreakSpaceType::kAfterEverySpace);
      disable_score_line_break_ = true;
    } else {
      break_iterator_.SetBreakSpace(BreakSpaceType::kAfterSpaceRun);
    }
  }
}

bool LineBreaker::IsPreviousItemOfType(InlineItem::InlineItemType type) {
  return current_.item_index > 0 &&
         Items().at(current_.item_index - 1)->Type() == type;
}

bool LineBreaker::IsNextNonBidiControlItemOpenTag() const {
  const InlineItems& items = Items();
  for (wtf_size_t i = current_.item_index; i < items.size(); ++i) {
    const InlineItem::InlineItemType type = items[i]->Type();
    if (type == InlineItem::kOpenTag) {
      return true;
    }
    if (type == InlineItem::kBidiControl) {
      continue;
    }
    return false;
  }
  return false;
}

void LineBreaker::MoveToNextOf(const InlineItem& item) {
  current_.text_offset = item.EndOffset();
  current_.item_index++;
#if DCHECK_IS_ON()
  const InlineItems& items = Items();
  if (current_.item_index < items.size()) {
    items[current_.item_index]->AssertOffset(current_.text_offset);
  } else {
    DCHECK_EQ(current_.text_offset, Text().length());
  }
#endif
}

void LineBreaker::MoveToNextOf(const InlineItemResult& item_result) {
  current_ = item_result.End();
  DCHECK(item_result.item);
  if (current_.text_offset == item_result.item->EndOffset()) {
    current_.item_index++;
  }
}

void LineBreaker::SetInputRange(InlineItemTextIndex start,
                                wtf_size_t end_item_index,
                                WhitespaceState initial_whitespace_state,
                                const LineBreaker* parent) {
  current_ = start;
  end_item_index_ = end_item_index;
  initial_whitespace_ = initial_whitespace_state;
  parent_breaker_ = parent;
}

const InlineBreakToken* LineBreaker::CreateBreakToken(
    const LineInfo& line_info) {
#if DCHECK_IS_ON()
  DCHECK(!has_considered_creating_break_token_);
  has_considered_creating_break_token_ = true;
#endif

  DCHECK(current_style_);
  const InlineItems& items = Items();
  DCHECK_LE(current_.item_index, items.size());
  // If we have reached the end, create no break token.
  if (IsAtEnd()) {
    return nullptr;
  }

  const BlockBreakToken* sub_break_token = nullptr;
  if (resume_block_in_inline_in_same_flow_) {
    const auto* block_in_inline = line_info.BlockInInlineLayoutResult();
    DCHECK(block_in_inline);
    if (block_in_inline->Status() != LayoutResult::kSuccess) [[unlikely]] {
      return nullptr;
    }
    // Look for a break token inside the block-in-inline, so that we can add it
    // to the inline break token that we're about to create.
    const auto& block_in_inline_fragment =
        To<PhysicalBoxFragment>(block_in_inline->GetPhysicalFragment());
    sub_break_token = block_in_inline_fragment.GetBreakToken();
  }

  const bool is_past_first_formatted_line =
      !is_first_formatted_line_ || !line_info.IsEmptyLine();

  const bool is_line_clamp_displaced_line =
      line_clamp_ellipsis_width_ && line_info.Results().empty();

  DCHECK_EQ(line_info.HasForcedBreak(), is_forced_break_);
  unsigned flags =
      (is_forced_break_ ? InlineBreakToken::kIsForcedBreak : 0) |
      (line_info.UseFirstLineStyle() ? InlineBreakToken::kUseFirstLineStyle
                                     : 0) |
      (cloned_box_decorations_count_
           ? InlineBreakToken::kHasClonedBoxDecorations
           : 0) |
      (is_past_first_formatted_line
           ? InlineBreakToken::kIsPastFirstFormattedLine
           : 0) |
      (is_line_clamp_displaced_line
           ? InlineBreakToken::kIsLineClampDisplacedLine
           : 0);

  InlineItemTextIndex next_start = current_;
  if (line_info.UseFirstLineStyle()) [[unlikely]] {
    if (const auto& offset_map = node_.FirstLineOffsetMap()) [[unlikely]] {
      // The `::first-line` style has changed the text length.
      // Adjust `next_start` to the offset for the text without `::first-line`.
      next_start.text_offset =
          offset_map->InverseMapOffset(next_start.text_offset);
      const auto& base_items = node_.ItemsData(false).items;
      while (next_start.item_index < base_items.size()) {
        const auto& item = base_items[next_start.item_index];
        if (item->Length() > 0 && next_start.text_offset >= item->EndOffset()) {
          ++next_start.item_index;
        } else if (item->Length() == 0 &&
                   next_start.text_offset > item->EndOffset()) {
          ++next_start.item_index;
        } else {
          break;
        }
      }
      if (next_start.item_index >= base_items.size()) {
        return nullptr;
      }
    }
  }

  return InlineBreakToken::Create(node_, current_style_, next_start, flags,
                                  sub_break_token, ruby_break_token_);
}

}  // namespace blink
```

## line_breaker.h

Original: [complete immutable source](https://chromium.googlesource.com/chromium/src/+/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/third_party/blink/renderer/core/layout/inline/line_breaker.h). Path: `third_party/blink/renderer/core/layout/inline/line_breaker.h`. Source bytes: 21356; source lines: 532; SHA-256: `eaa5db198a204ca2bda723760fdae9622d36b900f8c9dc9ad8d8dce8674538ec`.

```cpp
// Copyright 2017 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#ifndef THIRD_PARTY_BLINK_RENDERER_CORE_LAYOUT_INLINE_LINE_BREAKER_H_
#define THIRD_PARTY_BLINK_RENDERER_CORE_LAYOUT_INLINE_LINE_BREAKER_H_

#include <optional>

#include "base/check_op.h"
#include "third_party/blink/renderer/core/core_export.h"
#include "third_party/blink/renderer/core/layout/exclusions/line_layout_opportunity.h"
#include "third_party/blink/renderer/core/layout/inline/inline_item_result.h"
#include "third_party/blink/renderer/core/layout/inline/inline_item_text_index.h"
#include "third_party/blink/renderer/core/layout/inline/inline_node.h"
#include "third_party/blink/renderer/core/layout/inline/leading_floats.h"
#include "third_party/blink/renderer/core/layout/inline/line_break_point.h"
#include "third_party/blink/renderer/platform/fonts/shaping/harfbuzz_shaper.h"
#include "third_party/blink/renderer/platform/fonts/shaping/shape_result_spacing.h"
#include "third_party/blink/renderer/platform/text/text_break_iterator.h"
#include "third_party/blink/renderer/platform/wtf/allocator/allocator.h"

namespace blink {

class ColumnSpannerPath;
class Hyphenation;
class InlineBreakToken;
class InlineItem;
class LineBreakCandidateContext;
class LineInfo;
class ResolvedTextLayoutAttributesIterator;
class ShapingLineBreaker;
struct AnnotationBreakTokenData;
struct RubyBreakTokenData;

// The line breaker needs to know which mode its in to properly handle floats.
enum class LineBreakerMode { kContent, kMinContent, kMaxContent };

// Represents a line breaker.
//
// This class measures each InlineItem and determines items to form a line,
// so that InlineLayoutAlgorithm can build a line box from the output.
class CORE_EXPORT LineBreaker {
  STACK_ALLOCATED();

 public:
  LineBreaker(InlineNode,
              LineBreakerMode,
              const ConstraintSpace&,
              const LineLayoutOpportunity&,
              const LeadingFloats& leading_floats,
              const InlineBreakToken*,
              const ColumnSpannerPath*,
              ExclusionSpace*);
  ~LineBreaker();

  const InlineItemsData& ItemsData() const { return *items_data_; }

  // True if the last line has `box-decoration-break: clone`, which affected the
  // size.
  bool HasClonedBoxDecorations() const { return has_cloned_box_decorations_; }

  // Compute the next line break point and produces InlineItemResults for
  // the line.
  void NextLine(LineInfo*);

  bool IsFinished() const { return current_.item_index >= Items().size(); }

  // True if there are items that `ScoreLineBreaker` doesn't support.
  // Conditions that can be determined by `CollectInlines` are done by
  // `InlineNode::IsScoreLineBreakDisabled()`, but some conditions can change
  // withoiut `CollectInlines`. They are determined by this.
  bool ShouldDisableScoreLineBreak() const { return disable_score_line_break_; }
  // True if there are items that `ParagraphLineBreaker` doesn't support.
  bool ShouldDisableBisectLineBreak() const {
    return disable_bisect_line_break_;
  }

  void SetLineOpportunity(const LineLayoutOpportunity& line_opportunity);
  // Override the available width to compute line breaks. This is reset after
  // each `NextLine`.
  void OverrideAvailableWidth(LayoutUnit available_width);
  // Specify to break at the `offset` rather than the available width.
  void SetBreakAt(const LineBreakPoint& offset);

  void SetLineClampEllipsisWidth(LayoutUnit width) {
    DCHECK(RuntimeEnabledFeatures::CSSLineClampLineBreakingEllipsisEnabled());
    line_clamp_ellipsis_width_ = width;
    UpdateAvailableWidth();
  }

  // Computing |LineBreakerMode::kMinContent| with |MaxSizeCache| caches
  // information that can help computing |kMaxContent|. It is recommended to set
  // this when computing both |kMinContent| and |kMaxContent|.
  using MaxSizeCache = Vector<LayoutUnit, 64>;
  void SetIntrinsicSizeOutputs(MaxSizeCache* max_size_cache,
                               bool* depends_on_block_constraints_out);

  // Compute InlineItemResult for an open tag item.
  // Returns true if this item has edge and may have non-zero inline size.
  static bool ComputeOpenTagResult(const InlineItem&,
                                   const ConstraintSpace&,
                                   bool is_in_svg_text,
                                   InlineItemResult*);

  // This enum is private, except for |WhitespaceStateForTesting()|. See
  // |whitespace_| member.
  enum class WhitespaceState {
    kLeading,
    kNone,
    kUnknown,
    kCollapsible,
    kCollapsed,
    kPreserved,
  };
  WhitespaceState TrailingWhitespaceForTesting() const {
    return trailing_whitespace_;
  }

  // Find break candidates in the `item_result` and append to `context`. See
  // `LineBreakCandidate` and `LineBreakCandidateContext` for more details.
  void AppendCandidates(const InlineItemResult& item_result,
                        const LineInfo& line_info,
                        LineBreakCandidateContext& context);

  // True if the argument can break; i.e. has at least one break opportunity.
  bool CanBreakInside(const LineInfo& line_info);
  bool CanBreakInside(const InlineItemResult& item_result);

  // This LineBreaker handles only [start, end_item_index) of `Items()`.
  void SetInputRange(InlineItemTextIndex start,
                     wtf_size_t end_item_index,
                     WhitespaceState initial_whitespace_state,
                     const LineBreaker* parent);

 private:
  Document& GetDocument() const { return node_.GetDocument(); }

  // Returns true if this LineBreaker is computing min/max content size.
  bool IsComputingContentSize() const;

  // True if `this` is for a part of an IFC. Used by Ruby.
  bool IsSubLineBreaker() const { return end_item_index_ != Items().size(); }

  const String& Text() const { return text_content_; }
  const InlineItems& Items() const { return items_data_->items; }

  InlineItemResult* AddItem(const InlineItem&, unsigned end_offset, LineInfo*);
  InlineItemResult* AddItem(const InlineItem&, LineInfo*);
  InlineItemResult* AddEmptyItem(const InlineItem&, LineInfo*);

  void BreakLine(LineInfo*);
  void PrepareNextLine(LineInfo*);

  void ComputeLineLocation(LineInfo*) const;

  // Returns true if CSS property "white-space" specified in |style| allows
  // wrap. Note: For "text-combine-upright:all", this function returns false
  // event if "white-space" means wrap, because combined text should be laid
  // out in one line.
  bool ShouldAutoWrap(const ComputedStyle& style) const;

  enum class LineBreakState {
    // The line breaking is complete.
    kDone,

    // Overflow is detected without any earlier break opportunities. This line
    // should break at the earliest break opportunity.
    kOverflow,

    // Should complete the line at the earliest possible point.
    // Trailing spaces, <br>, or close tags should be included to the line even
    // when it is overflowing.
    kTrailing,

    // Looking for more items to fit into the current line.
    kContinue,
  };

  void HandleText(const InlineItem& item, const ShapeResult&, LineInfo*);
  // Split |item| into segments, and add them to |line_info|.
  // This is for SVG <text>.
  void SplitTextIntoSegments(const InlineItem& item, LineInfo* line_info);
  // Returns true if we should split InlineItem before
  // svg_addressable_offset_.
  bool ShouldCreateNewSvgSegment() const;
  enum BreakResult { kSuccess, kOverflow, kBreakAt };
  BreakResult BreakText(InlineItemResult*,
                        const InlineItem&,
                        const ShapeResult&,
                        LayoutUnit available_width,
                        LayoutUnit available_width_with_hyphens,
                        LineInfo*);
  bool BreakTextAt(InlineItemResult*,
                   const InlineItem&,
                   ShapingLineBreaker& breaker,
                   LineInfo*);
  bool BreakTextAtPreviousBreakOpportunity(InlineItemResults& results,
                                           wtf_size_t item_result_index);
  bool HandleTextForFastMinContent(InlineItemResult*,
                                   const InlineItem&,
                                   const ShapeResult&,
                                   LineInfo*);
  void HandleEmptyText(const InlineItem& item, LineInfo*);

  const ShapeResultView* TruncateLineEndResult(const LineInfo&,
                                               const InlineItemResult&,
                                               unsigned end_offset);
  void UpdateShapeResult(const LineInfo&, InlineItemResult*);
  const ShapeResult* ShapeText(const InlineItem&,
                               unsigned start,
                               unsigned end,
                               ShapeOptions = ShapeOptions());

  void HandleTrailingSpaces(const InlineItem&, LineInfo*);
  void HandleTrailingSpaces(const InlineItem&, const ShapeResult*, LineInfo*);
  void RemoveTrailingCollapsibleSpace(LineInfo*);
  void SplitTrailingBidiPreservedSpace(LineInfo*);
  LayoutUnit TrailingCollapsibleSpaceWidth(LineInfo*);
  void ComputeTrailingCollapsibleSpace(LineInfo*);
  bool ComputeTrailingCollapsibleSpaceHelper(LineInfo&);
  void RewindTrailingOpenTags(LineInfo*);

  void HandleControlItem(const InlineItem&, LineInfo*);
  void HandleForcedLineBreak(const InlineItem*, LineInfo*);
  void HandleBidiControlItem(const InlineItem&, LineInfo*);
  void HandleAtomicInline(const InlineItem&, LineInfo*);
  void HandleBlockInInline(const InlineItem&,
                           const BlockBreakToken*,
                           LineInfo*);
  void ComputeMinMaxContentSizeForBlockChild(const InlineItem&,
                                             InlineItemResult*,
                                             const LineBreaker* root_breaker);
  // Returns false if we can't handle the current InlineItem as a ruby.
  // NOINLINE prevents a compiler for Android 64bit from inlining
  // HandleRuby() twice.
  //
  // `retry_size` - If this is not kIndefiniteSize, the function tries to break
  //   the ruby column so that its inline-size is less than `retry_size`.
  NOINLINE bool HandleRuby(LineInfo* line_info,
                           LayoutUnit retry_size = kIndefiniteSize);
  bool IsMonolithicRuby(
      const LineInfo& base_line,
      const HeapVector<LineInfo, 1>& annotation_line_list) const;
  // `mode`: Must be kMaxContent or kContent.
  // `limit`: Must be non-negative or kIndefiniteSize, which means no auto-wrap.
  LineInfo CreateSubLineInfo(
      InlineItemTextIndex start,
      wtf_size_t end_item_index,
      LineBreakerMode mode,
      LayoutUnit limit,
      WhitespaceState initial_whitespace_state,
      bool disable_trailing_whitespace_collapsing = false);
  InlineItemResult* AddRubyColumnResult(
      const InlineItem& item,
      const LineInfo& base_line_info,
      const HeapVector<LineInfo, 1>& annotation_line_list,
      const Vector<AnnotationBreakTokenData, 1>& annotation_data_list,
      LayoutUnit ruby_size,
      bool is_continuation,
      LineInfo& line_info);
  bool CanBreakAfterRubyColumn(const InlineItemResult& column_result,
                               wtf_size_t column_end_item_index) const;

  bool CanBreakAfterAtomicInline(const InlineItem& item) const;
  bool CanBreakAfter(const InlineItem& item) const;
  // Returns true when text content at |offset| is
  //    kObjectReplacementCharacter (U+FFFC), or
  //    kNoBreakSpace (U+00A0) if |sticky_images_quirk_|.
  bool MayBeAtomicInline(wtf_size_t offset) const;
  const InlineItem* TryGetAtomicInlineItemAfter(const InlineItem& item) const;
  unsigned IgnorableBidiControlLength(const InlineItem& item) const;

  bool ShouldPushFloatAfterLine(UnpositionedFloat*, LineInfo*);
  void HandleFloat(const InlineItem&,
                   const BlockBreakToken* float_break_token,
                   LineInfo*);
  void UpdateLineOpportunity();
  void RewindFloats(unsigned new_end, LineInfo&, InlineItemResults&);

  void HandleInitialLetter(const InlineItem&, LineInfo*);
  void HandleOutOfFlowPositioned(const InlineItem&, LineInfo*);

  void HandleOpenTag(const InlineItem&, LineInfo*);
  void HandleCloseTag(const InlineItem&, LineInfo*);

  bool HandleOverflowIfNeeded(LineInfo*);
  // NOINLINE prevents a compiler for Android 64bit from code size bloat.
  NOINLINE void HandleOverflow(LineInfo*);
  void RetryAfterOverflow(LineInfo*, InlineItemResults*);
  void RewindOverflow(unsigned new_end, LineInfo*);
  void Rewind(unsigned new_end, LineInfo*);
  void ResetRewindLoopDetector() { last_rewind_.reset(); }

  const ComputedStyle& ComputeCurrentStyle(unsigned item_result_index,
                                           LineInfo*) const;
  void SetCurrentStyle(const ComputedStyle&);
  void SetCurrentStyleForce(const ComputedStyle&);

  bool IsPreviousItemOfType(InlineItem::InlineItemType);
  bool IsNextNonBidiControlItemOpenTag() const;
  void MoveToNextOf(const InlineItem&);
  void MoveToNextOf(const InlineItemResult&);
  bool IsAtEnd() const { return current_.item_index >= end_item_index_; }

  void ComputeBaseDirection();
  LayoutUnit ComputeFloatOffset() const;
  void RecalcClonedBoxDecorations();

  LayoutUnit AvailableWidth() const { return available_width_; }
  LayoutUnit AvailableWidthToFit() const {
    return AvailableWidth().AddEpsilon();
  }
  LayoutUnit RemainingAvailableWidth() const {
    return AvailableWidthToFit() - position_;
  }
  bool CanFitOnLine() const {
    return position_ <= AvailableWidthToFit() ||
           (parent_breaker_ && !auto_wrap_);
  }
  void UpdateAvailableWidth();
  void UpdateAvailableWidthFromBaseAvailableWidth();

  // True if the current line is hyphenated.
  bool HasHyphen() const { return hyphen_index_.has_value(); }
  LayoutUnit AddHyphen(InlineItemResults* item_results,
                       wtf_size_t index,
                       InlineItemResult* item_result);
  LayoutUnit AddHyphen(InlineItemResults* item_results, wtf_size_t index);
  LayoutUnit AddHyphen(InlineItemResults* item_results,
                       InlineItemResult* item_result);
  LayoutUnit RemoveHyphen(InlineItemResults* item_results);
  void RestoreLastHyphen(InlineItemResults* item_results);
  void FinalizeHyphen(InlineItemResults* item_results);

  // Create an InlineBreakToken for the last line returned by NextLine().
  // Only call once per instance.
  const InlineBreakToken* CreateBreakToken(const LineInfo&);

  // Represents the current offset of the input.
  LineBreakState state_;
  InlineItemTextIndex current_;
  unsigned svg_addressable_offset_ = 0;
  LineBreakPoint break_at_;

  // |WhitespaceState| of the current end. When a line is broken, this indicates
  // the state of trailing whitespaces.
  // This field is not used for sub-LineBreakers.
  WhitespaceState trailing_whitespace_ = WhitespaceState::kUnknown;
  // The state just after starting BreakLine(). This can be overridden by
  // SetInputRange().
  WhitespaceState initial_whitespace_ = WhitespaceState::kLeading;

  // The current position from inline_start. Unlike InlineLayoutAlgorithm
  // that computes position in visual order, this position in logical order.
  LayoutUnit position_;

  // Offset of this (sub-)line's start from the tab-stop origin, i.e. the start
  // content edge of the nearest block container ancestor. Non-zero only for
  // ruby sub-LineBreakers.
  LayoutUnit tab_stop_offset_;

  LayoutUnit applied_text_indent_;
  LayoutUnit available_width_;
  // Available width without `box-decoration-break`.
  LayoutUnit base_available_width_;
  LineLayoutOpportunity line_opportunity_;

  InlineNode node_;

  LineBreakerMode mode_;

  // True if node_ is an initial letter box.
  const bool is_initial_letter_box_;

  // True if node_ is an SVG <text>.
  const bool is_svg_text_;

  // True if node_ is LayoutNGTextCombine.
  const bool is_text_combine_;

  // True if this line is the "first formatted line".
  // https://www.w3.org/TR/CSS22/selector.html#first-formatted-line
  bool is_first_formatted_line_ = false;

  bool use_first_line_style_ = false;

  // True when current box allows line wrapping.
  bool auto_wrap_ = false;

  // Disallow line wrapping even if the ComputedStyle allows it.
  bool disallow_auto_wrap_ = false;

  // True when current box should fallback to break anywhere if it overflows.
  bool break_anywhere_if_overflow_ = false;

  // Force LineBreakType::kBreakCharacter by ignoring the current style if
  // |break_anywhere_if_overflow_| is set. Set to find grapheme cluster
  // boundaries for 'break-word' after overflow.
  bool override_break_anywhere_ = false;

  // Disable `LineBreakType::kPhrase` even if specified by the CSS.
  bool disable_phrase_ = false;

  bool disable_score_line_break_ = false;
  bool disable_bisect_line_break_ = false;

  bool disable_trailing_whitespace_collapsing_ = false;

  // True when the line should be non-empty if |IsLastLine|..
  bool force_non_empty_if_last_line_ = false;

  // Set when the line ended with a forced break, one for the current line and
  // another for the previous line.
  bool is_forced_break_ = false;
  bool previous_line_had_forced_break_ = false;

  // Set in quirks mode when we're not supposed to break inside table cells
  // between images, and between text and images.
  bool sticky_images_quirk_ = false;

  // True if the resultant line contains a RubyColumn with inline-end overhang.
  bool maybe_have_end_overhang_ = false;

  // True if ShouldCreateNewSvgSegment() should be called.
  bool needs_svg_segmentation_ = false;

  // True if the block-in-inline broke inside, and it is to be resumed in the
  // same flow.
  bool resume_block_in_inline_in_same_flow_ = false;

#if DCHECK_IS_ON()
  bool has_considered_creating_break_token_ = false;
#endif

  const InlineItemsData* items_data_;

  // `end_item_index_` is usually `Items().size()`.
  // SetInputRange() updates it.
  wtf_size_t end_item_index_;

  // The text content of this node. This is same as |items_data_.text_content|
  // except when sticky images quirk is needed. See
  // |InlineNode::TextContentForContentSize|.
  String text_content_;

  const ConstraintSpace& constraint_space_;
  ExclusionSpace* exclusion_space_;
  const InlineBreakToken* break_token_;
  // This is set by the constructor, or set after filling a LineInfo.
  // BreakLine consumes it.
  const RubyBreakTokenData* ruby_break_token_ = nullptr;
  const ColumnSpannerPath* column_spanner_path_;
  const ComputedStyle* current_style_ = nullptr;

  LazyLineBreakIterator break_iterator_;
  HarfBuzzShaper shaper_;
  ShapeResultSpacing spacing_;
  const Hyphenation* hyphenation_ = nullptr;

  std::optional<wtf_size_t> hyphen_index_;
  bool has_any_hyphens_ = false;

  // Cache the result of |ComputeTrailingCollapsibleSpace| to avoid shaping
  // multiple times.
  struct TrailingCollapsibleSpace {
    STACK_ALLOCATED();

   public:
    InlineItemResults* item_results = nullptr;
    wtf_size_t item_result_index = kNotFound;
    const ShapeResultView* collapsed_shape_result = nullptr;
    // Ancestors of `item_result`. ancestor_ruby_columns[0] is the parent of
    // `item_result`, and ancestor_ruby_columns[n+1] is the parent of
    // ancestor_ruby_columns[n]. This list is empty if `item_result` is not
    // in a ruby column.
    //
    // It's difficult to trace InlineItemResults below because it's a part of
    // LineInfo, and LineInfo is stack-allocated or a part of
    // InlineItemResultRubyColumn. Storing raw pointers should be safe because
    // the InlineItemResults are owned by a LineInfo tree and they are not
    // movable.
    GC_PLUGIN_IGNORE("See the above comment")
    Vector<std::pair<InlineItemResults*, wtf_size_t>> ancestor_ruby_columns;

    InlineItemResult& ItemResult() const {
      return (*item_results)[item_result_index];
    }
  };
  std::optional<TrailingCollapsibleSpace> trailing_collapsible_space_;

  LayoutUnit override_available_width_;

  // Keep track of handled float items. See HandleFloat().
  const LeadingFloats& leading_floats_;
  unsigned leading_floats_index_ = 0u;

  // Cache for computing |MinMaxSize|. See |MaxSizeCache|.
  MaxSizeCache* max_size_cache_ = nullptr;

  bool* depends_on_block_constraints_out_ = nullptr;

  // The current base direction for the bidi algorithm.
  // This is copied from InlineNode, then updated after each forced line break
  // if 'unicode-bidi: plaintext'.
  TextDirection base_direction_;

  // Fields for `box-decoration-break: clone`.
  unsigned cloned_box_decorations_count_ = 0;
  LayoutUnit cloned_box_decorations_initial_size_;
  LayoutUnit cloned_box_decorations_end_size_;
  bool has_cloned_box_decorations_ = false;

  // These fields are to detect rewind-loop.
  struct RewindIndex {
    wtf_size_t from_item_index;
    wtf_size_t to_index;
  };
  std::optional<RewindIndex> last_rewind_;

  // This has a valid object if is_svg_text_.
  std::unique_ptr<ResolvedTextLayoutAttributesIterator> svg_resolved_iterator_;

  LayoutUnit line_clamp_ellipsis_width_;

  // This member is available after calling SetInputRange().
  const LineBreaker* parent_breaker_ = nullptr;
};

}  // namespace blink

#endif  // THIRD_PARTY_BLINK_RENDERER_CORE_LAYOUT_INLINE_LINE_BREAKER_H_
```
