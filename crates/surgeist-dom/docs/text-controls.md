# Output and Textarea values

Read this contract for current/default value distinctions, exact text, setters, reset, clone and adoption. Both families use the existing [Form/control owner](forms-and-select.md) and [semantic capture](semantic-captures.md).

## Output

Native HTML Output has a checked `DomNode::as_output()` view over its actual
nullable default-value override and canonical Form owner/custom-error state.
`None`, `Some(empty)` and `Some(text)` remain distinct. Read current and default
values through the existing bounded HTML semantic `OutputValue` and
`OutputDefaultValue` queries on `html_capture`; no second text getter or cached
current value is retained. Output text is exact ordinary descendant UTF-16,
including nested control default Text, without Textarea newline normalization
or traversal into owned Template contents or ShadowRoot associations.

`set_output_value` stores the current default as an override before replacing
children. `set_output_default_value` replaces children only while the override
is null; otherwise it changes only that override. `reset_output` reads the
default, replaces children, then clears the override. The shared HTML cursor
owns this order. Output and Textarea use one real DOM string-replace-all helper:
new Text admission precedes old-child removal, detached nodes stay managed, and
every actual remove/insert children hook still runs. Equal text does not skip
identity-changing replacement. No extra final children hook is invented.

Mutation-time descendant reading walks ordinary parent/sibling links without a
traversal buffer. Two cancellable passes count and materialize one exact UTF-16
buffer; its actual capacity is bounded by the canonical string-unit ceiling.
This finite tree/string work does not consume the numeric/CSS ledger. Override,
custom message, new Text and detached old Text are independently retained and
accounted. Clone/import creates a fresh null override and custom error before
copying descendants; adoption preserves established state. Both initial and
Slot-supplemented captures carry actual Output defaults, with aggregate override
units bounded by the initial capture's UTF-16 ceiling. Native installation uses
the same family and tree owner. Per-control reset does not dispatch a whole-Form
cancelable reset event.

## Textarea

Native HTML Textarea has a checked `DomNode::as_textarea()` view over one exact
raw UTF-16 value, shared dirty/user-validity flags, actual selection and native
association/custom-error state. `state().raw_value()` is borrowed raw storage;
`value(DomTextareaReadLimits, cancel)` materializes the API value through HTML's
newline normalizer, and `text_length` counts that normalized value. `default_value`
reads actual immediate child Text, excluding nested text. These finite reads
have explicit output-unit and cumulative-work ceilings. Read work charges each
raw normalization read, output write, child visit and output reservation; counting
and materialization are separate passes. Actual output capacity is admitted and
failure returns a typed error. No hidden getter cache or mutable API-value mirror
is retained, and this API does not claim submitted hard wrapping.

`set_textarea_value` retains exact raw input, marks dirty, clamps selection to
normalized API length and consumes HTML's additional setter-finish decision.
Changed API values unselect and move to end even when a range survives clamping;
equivalent CRLF/LF raw spellings preserve selection. The actual platform's two
independent support facts apply to source-required writes, while adoption retains
established state. Temporary normalized comparison buffers are each bounded by
their admitted raw operand, at most twice the existing canonical unit ceiling;
all scans/copies poll cancellation. No string work spends numeric/CSS allowance.

`set_textarea_default_value` string-replaces actual children through normal
creation/removal/insertion hooks; detached old children remain addressable.
Clean children-changed steps copy immediate Text in order, including reached
intermediates during same-parent moves, Fragment transfer and CharacterData edits.
Dirty raw stays independent. `reset_textarea` clears dirty/user-validity and reads
the actual default children while preserving custom error and Form owner. The
shared children dispatcher also preserves Script's connected preparation order.
Clone/import copies only raw and dirty into fresh native state before descendants;
ordinary descendant hooks still run. Adoption preserves current state, and
retirement uses the existing canonical owner cleanup. Native fragment installation
uses these same actual family/tree hooks. Parser-pop reset, user-edit events,
selection events, whole-Form reset dispatch and submitted wrapping are separate
boundaries; none is fabricated by native creation.

[Back to surgeist-dom](../README.md).
