# HTML: writing suggestions, autocapitalization and autocorrection

Source: [HTML commit snapshot](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/), revision `877d6146487f15b15bc8dd0e3c625063a37b456a`; complete HTML SHA-256 `877d6cc00dae6c18c16d952fdcbf944acc58919579621b4257fa0ed1888a871f`, verified against the bytes retrieved 2026-11-02. Full sections 6.8.6, 6.8.7 and 6.8.8. Source authority remains the pinned English HTML standard.

Copyright © WHATWG (Apple, Google, Mozilla, Microsoft). Specification prose is CC BY 4.0; see [WHATWG license](../licenses/whatwg/LICENSE). Partial extraction and Markdown reformatting by Surgeist, 2026-11-02, with a small Python `html.parser` converter because Pandoc was not available; the multiset of words in the three sections was compared with the original markup. Source scripts and MDN annotations omitted; links made absolute. No endorsement or semantic changes. This reference does not expand implementation scope to input methods, autocorrection or suggestion interfaces.

#### <span class="secno">6.8.6</span> Writing suggestions<a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#writing-suggestions" class="self-link"></a>

User agents offer writing suggestions as users type into editable regions, either in form controls (e.g., the [`textarea`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-textarea-element) element) or in elements in an <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#editing-host" id="writing-suggestions:editing-host"> editing host</a>.

The <span id="attr-writingsuggestions" class="dfn" dfn-for="html-global" dfn-type="element-attr">`writingsuggestions`</span> content attribute is an <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#enumerated-attribute" id="writing-suggestions:enumerated-attribute">enumerated attribute</a> with the following keywords and states:

| Keyword | State | Brief description |
|---|---|---|
| <span id="attr-writingsuggestions-true" class="dfn" dfn-for="html-global/writingsuggestions" dfn-type="attr-value">`true`</span> | <span id="attr-writingsuggestions-true-state" class="dfn">True</span> | Writing suggestions should be offered on this element. |
| <span id="attr-writingsuggestions-false" class="dfn" dfn-for="html-global/writingsuggestions" dfn-type="attr-value">`false`</span> | <span id="attr-writingsuggestions-false-state" class="dfn">False</span> | Writing suggestions should not be offered on this element. |

<div algorithm="">

The attribute's *[missing value default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#missing-value-default)* is the <span id="attr-writingsuggestions-default-state" class="dfn">Default</span> state. The default state indicates that the element is to act according to a default behavior, possibly based on the parent element's own [`writingsuggestions`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-writingsuggestions) state, as defined below.

</div>

<div algorithm="">

The attribute's *[invalid value default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#invalid-value-default)* and *[empty value default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#empty-value-default)* are both the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-writingsuggestions-true-state" id="writing-suggestions:attr-writingsuggestions-true-state">True</a> state.

</div>

<code><var>element</var>.<a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-writingsuggestions" id="dom-writingsuggestions-dev">writingSuggestions</a> [ = <var>value</var> ]</code>  
Returns "`true`" if the user agent is to offer writing suggestions under the scope of the element; otherwise, returns "`false`".

Can be set, to override the default and set the [`writingsuggestions`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-writingsuggestions) content attribute.


<div algorithm="">

The <span id="computed-writing-suggestions-value" class="dfn">computed writing suggestions value</span> of a given <var>element</var> is determined by running the following steps:

1.  If <var>element</var>'s [`writingsuggestions`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-writingsuggestions) content attribute is in the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-writingsuggestions-false-state" id="writing-suggestions:attr-writingsuggestions-false-state">False</a> state, return "`false`".
2.  If <var>element</var>'s [`writingsuggestions`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-writingsuggestions) content attribute is in the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-writingsuggestions-default-state" id="writing-suggestions:attr-writingsuggestions-default-state">Default</a> state, <var>element</var> has a parent element, and the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#computed-writing-suggestions-value" id="writing-suggestions:computed-writing-suggestions-value">computed writing suggestions value</a> of <var>element</var>'s parent element is "`false`", then return "`false`".
3.  Return "`true`".

</div>

<div algorithm="">

The <span id="dom-writingsuggestions" class="dfn" dfn-for="HTMLElement" dfn-type="attribute">`writingSuggestions`</span> getter steps are:

1.  Return <a href="https://webidl.spec.whatwg.org/#this" id="writing-suggestions:this" data-x-internal="this">this</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#computed-writing-suggestions-value" id="writing-suggestions:computed-writing-suggestions-value-2">computed writing suggestions value</a>.

</div>

The [`writingSuggestions`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-writingsuggestions) IDL attribute is not affected by user preferences that override the [`writingsuggestions`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-writingsuggestions) content attribute, and therefore might not reflect the actual writing suggestions state.

------------------------------------------------------------------------

<div algorithm="">

User agents should only offer suggestions within an element's scope if the result of running the following algorithm given <var>element</var> returns true:

1.  If the user has disabled writing suggestions, then return false.
2.  If none of the following conditions are true:
    - <var>element</var> is an [`input`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-input-element) element whose [`type`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-input-type) attribute is in either the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#text-(type=text)-state-and-search-state-(type=search)" id="writing-suggestions:text-(type=text)-state-and-search-state-(type=search)">Text</a>, <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#text-(type=text)-state-and-search-state-(type=search)" id="writing-suggestions:text-(type=text)-state-and-search-state-(type=search)-2">Search</a>, <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#telephone-state-(type=tel)" id="writing-suggestions:telephone-state-(type=tel)">Telephone</a>, <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#url-state-(type=url)" id="writing-suggestions:url-state-(type=url)">URL</a>, or <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#email-state-(type=email)" id="writing-suggestions:email-state-(type=email)">Email</a> state and <var>element</var> is <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-fe-mutable" id="writing-suggestions:concept-fe-mutable">mutable</a>;
    - <var>element</var> is a [`textarea`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-textarea-element) element that is <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-fe-mutable" id="writing-suggestions:concept-fe-mutable-2">mutable</a>; or
    - <var>element</var> is an <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#editing-host" id="writing-suggestions:editing-host-2">editing host</a> or is <a href="https://w3c.github.io/editing/docs/execCommand/#editable" id="writing-suggestions:editable" data-x-internal="editable">editable</a>,

    then return false.
3.  If <var>element</var> has an <a href="https://dom.spec.whatwg.org/#concept-tree-inclusive-ancestor" id="writing-suggestions:inclusive-ancestor" data-x-internal="inclusive-ancestor">inclusive ancestor</a> with a [`writingsuggestions`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-writingsuggestions) content attribute that's not in the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-writingsuggestions-default-state" id="writing-suggestions:attr-writingsuggestions-default-state-2">Default</a> and the nearest such ancestor's [`writingsuggestions`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-writingsuggestions) content attribute is in the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-writingsuggestions-false-state" id="writing-suggestions:attr-writingsuggestions-false-state-2">False</a> state, then return false.
4.  Otherwise, return true.

</div>

This specification does not define the user interface for writing suggestions. A user agent could offer on-demand suggestions, continuous suggestions as the user types, inline suggestions, autofill-like suggestions in a popup, or could use other interfaces.

#### <span class="secno">6.8.7</span> Autocapitalization<a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#autocapitalization" class="self-link"></a>

Some methods of entering text, for example virtual keyboards on mobile devices, and also voice input, often assist users by automatically capitalizing the first letter of sentences (when composing text in a language with this convention). A virtual keyboard that implements autocapitalization might automatically switch to showing uppercase letters (but allow the user to toggle it back to lowercase) when a letter that should be autocapitalized is about to be typed. Other types of input, for example voice input, may perform autocapitalization in a way that does not give users an option to intervene first. The [`autocapitalize`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-autocapitalize) attribute allows authors to control such behavior.

The [`autocapitalize`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-autocapitalize) attribute, as typically implemented, does not affect behavior when typing on a physical keyboard. (For this reason, as well as the ability for users to override the autocapitalization behavior in some cases or edit the text after initial input, the attribute must not be relied on for any sort of input validation.)

The [`autocapitalize`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-autocapitalize) attribute can be used on an <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#editing-host" id="autocapitalization:editing-host">editing host</a> to control autocapitalization behavior for the hosted editable region, on an [`input`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-input-element) or [`textarea`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-textarea-element) element to control the behavior for inputting text into that element, or on a [`form`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-form-element) element to control the default behavior for all <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#category-autocapitalize" id="autocapitalization:category-autocapitalize">autocapitalize-and-autocorrect inheriting elements</a> associated with the [`form`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-form-element) element.

The [`autocapitalize`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-autocapitalize) attribute never causes autocapitalization to be enabled for [`input`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-input-element) elements whose [`type`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-input-type) attribute is in one of the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#url-state-(type=url)" id="autocapitalization:url-state-(type=url)">URL</a>, <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#email-state-(type=email)" id="autocapitalization:email-state-(type=email)">Email</a>, or <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#password-state-(type=password)" id="autocapitalization:password-state-(type=password)">Password</a> states. (This behavior is included in the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#used-autocapitalization-hint" id="autocapitalization:used-autocapitalization-hint">used autocapitalization hint</a> algorithm below.)

The autocapitalization processing model is based on selecting among five <span id="autocapitalization-hint" class="dfn">autocapitalization hints</span>, defined as follows:

<span id="autocap-hint-default" class="dfn">Default</span>  
The user agent and input method should make their own determination of whether or not to enable autocapitalization.

<span id="autocap-hint-none" class="dfn">None</span>

No autocapitalization should be applied (all letters should default to lowercase).

<span id="autocap-hint-sentences" class="dfn">Sentences</span>

The first letter of each sentence should default to a capital letter; all other letters should default to lowercase.

<span id="autocap-hint-words" class="dfn">Words</span>

The first letter of each word should default to a capital letter; all other letters should default to lowercase.

<span id="autocap-hint-characters" class="dfn">Characters</span>

All letters should default to uppercase.


The <span id="attr-autocapitalize" class="dfn" dfn-for="html-global" dfn-type="element-attr">`autocapitalize`</span> attribute is an <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#enumerated-attribute" id="autocapitalization:enumerated-attribute">enumerated attribute</a> whose states are the possible <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#autocapitalization-hint" id="autocapitalization:autocapitalization-hint">autocapitalization hints</a>. The <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#autocapitalization-hint" id="autocapitalization:autocapitalization-hint-2">autocapitalization hint</a> specified by the attribute's state combines with other considerations to form the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#used-autocapitalization-hint" id="autocapitalization:used-autocapitalization-hint-2">used autocapitalization hint</a>, which informs the behavior of the user agent. The keywords for this attribute and their state mappings are as follows:

| Keyword | State |
|---|---|
| <span id="attr-autocapitalize-off" class="dfn" dfn-for="html-global/autocapitalize" dfn-type="attr-value">`off`</span> | <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#autocap-hint-none" id="autocapitalization:autocap-hint-none">None</a> |
| <span id="attr-autocapitalize-none" class="dfn" dfn-for="html-global/autocapitalize" dfn-type="attr-value">`none`</span> |  |
| <span id="attr-autocapitalize-on" class="dfn" dfn-for="html-global/autocapitalize" dfn-type="attr-value">`on`</span> | <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#autocap-hint-sentences" id="autocapitalization:autocap-hint-sentences">Sentences</a> |
| <span id="attr-autocapitalize-sentences" class="dfn" dfn-for="html-global/autocapitalize" dfn-type="attr-value">`sentences`</span> |  |
| <span id="attr-autocapitalize-words" class="dfn" dfn-for="html-global/autocapitalize" dfn-type="attr-value">`words`</span> | <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#autocap-hint-words" id="autocapitalization:autocap-hint-words">Words</a> |
| <span id="attr-autocapitalize-characters" class="dfn" dfn-for="html-global/autocapitalize" dfn-type="attr-value">`characters`</span> | <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#autocap-hint-characters" id="autocapitalization:autocap-hint-characters">Characters</a> |

<div algorithm="">

The attribute's *[missing value default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#missing-value-default)* is the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#autocap-hint-default" id="autocapitalization:autocap-hint-default">Default</a> state, and its *[invalid value default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#invalid-value-default)* is the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#autocap-hint-sentences" id="autocapitalization:autocap-hint-sentences-2">Sentences</a> state.

</div>

<code><var>element</var>.<a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-autocapitalize" id="dom-autocapitalize-dev">autocapitalize</a> [ = <var>value</var> ]</code>  
Returns the current autocapitalization state for the element, or an empty string if it hasn't been set. Note that for [`input`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-input-element) and [`textarea`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-textarea-element) elements that inherit their state from a [`form`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-form-element) element, this will return the autocapitalization state of the [`form`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-form-element) element, but for an element in an editable region, this will not return the autocapitalization state of the editing host (unless this element is, in fact, the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#editing-host" id="autocapitalization:editing-host-2">editing host</a>).

Can be set, to set the [`autocapitalize`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-autocapitalize) content attribute (and thereby change the autocapitalization behavior for the element).


<div algorithm="">

To compute the <span id="own-autocapitalization-hint" class="dfn">own autocapitalization hint</span> of an element <var>element</var>, run the following steps:

1.  If the [`autocapitalize`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-autocapitalize) content attribute is present on <var>element</var>, and its value is not the empty string, return the state of the attribute.
2.  If <var>element</var> is an <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#category-autocapitalize" id="autocapitalization:category-autocapitalize-2">autocapitalize-and-autocorrect inheriting element</a> and has a non-null <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#form-owner" id="autocapitalization:form-owner">form owner</a>, return the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#own-autocapitalization-hint" id="autocapitalization:own-autocapitalization-hint">own autocapitalization hint</a> of <var>element</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#form-owner" id="autocapitalization:form-owner-2">form owner</a>.
3.  Return <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#autocap-hint-default" id="autocapitalization:autocap-hint-default-2">Default</a>.

</div>

<div algorithm="">

The <span id="dom-autocapitalize" class="dfn" dfn-for="HTMLElement" dfn-type="attribute">`autocapitalize`</span> getter steps are to:

1.  Let <var>state</var> be the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#own-autocapitalization-hint" id="autocapitalization:own-autocapitalization-hint-2">own autocapitalization hint</a> of <a href="https://webidl.spec.whatwg.org/#this" id="autocapitalization:this" data-x-internal="this">this</a>.
2.  If <var>state</var> is <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#autocap-hint-default" id="autocapitalization:autocap-hint-default-3">Default</a>, then return the empty string.
3.  If <var>state</var> is <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#autocap-hint-none" id="autocapitalization:autocap-hint-none-2">None</a>, then return "[`none`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-autocapitalize-none)".
4.  If <var>state</var> is <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#autocap-hint-sentences" id="autocapitalization:autocap-hint-sentences-3">Sentences</a>, then return "[`sentences`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-autocapitalize-sentences)".
5.  Return the keyword value corresponding to <var>state</var>.

</div>

------------------------------------------------------------------------

User agents that support customizable autocapitalization behavior for a text input method and wish to allow web developers to control this functionality should, during text input into an element, compute the <span id="used-autocapitalization-hint" class="dfn">used autocapitalization hint</span> for the element. This will be an <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#autocapitalization-hint" id="autocapitalization:autocapitalization-hint-3">autocapitalization hint</a> that describes the recommended autocapitalization behavior for text input into the element.

User agents or input methods may choose to ignore or override the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#used-autocapitalization-hint" id="autocapitalization:used-autocapitalization-hint-3">used autocapitalization hint</a> in certain circumstances.

<div algorithm="">

The <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#used-autocapitalization-hint" id="autocapitalization:used-autocapitalization-hint-4">used autocapitalization hint</a> for an element <var>element</var> is computed using the following algorithm:

1.  If <var>element</var> is an [`input`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-input-element) element whose [`type`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-input-type) attribute is in one of the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#url-state-(type=url)" id="autocapitalization:url-state-(type=url)-2">URL</a>, <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#email-state-(type=email)" id="autocapitalization:email-state-(type=email)-2">Email</a>, or <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#password-state-(type=password)" id="autocapitalization:password-state-(type=password)-2">Password</a> states, then return <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#autocap-hint-default" id="autocapitalization:autocap-hint-default-4">Default</a>.
2.  If <var>element</var> is an [`input`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-input-element) element or a [`textarea`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-textarea-element) element, then return <var>element</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#own-autocapitalization-hint" id="autocapitalization:own-autocapitalization-hint-3">own autocapitalization hint</a>.
3.  If <var>element</var> is an <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#editing-host" id="autocapitalization:editing-host-3">editing host</a> or an <a href="https://w3c.github.io/editing/docs/execCommand/#editable" id="autocapitalization:editable" data-x-internal="editable">editable</a> element, then return the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#own-autocapitalization-hint" id="autocapitalization:own-autocapitalization-hint-4">own autocapitalization hint</a> of the <a href="https://w3c.github.io/editing/docs/execCommand/#editing-host-of" id="autocapitalization:editing-host-of" data-x-internal="editing-host-of">editing host of</a> <var>element</var>.
4.  <a href="https://infra.spec.whatwg.org/#assert" id="autocapitalization:assert" data-x-internal="assert">Assert</a>: this step is never reached, since text input only occurs in elements that meet one of the above criteria.

</div>

#### <span class="secno">6.8.8</span> Autocorrection<a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#autocorrection" class="self-link"></a>

Some methods of entering text assist users by automatically correcting misspelled words while typing, a process also known as autocorrection. User agents can support autocorrection of editable text, either in form controls (such as the value of [`textarea`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-textarea-element) elements), or in elements in an <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#editing-host" id="autocorrection:editing-host">editing host</a> (e.g., using [`contenteditable`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-contenteditable)). Autocorrection may be accompanied by user interfaces indicating that text is about to be autocorrected or has been autocorrected, and is commonly performed when inserting punctuation characters, spaces, or new paragraphs after misspelled words. The [`autocorrect`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-autocorrect) attribute allows authors to control such behavior.

The [`autocorrect`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-autocorrect) attribute can be used on an editing host to control autocorrection behavior for the hosted editable region, on an [`input`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-input-element) or [`textarea`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-textarea-element) element to control the behavior when inserting text into that element, or on a [`form`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-form-element) element to control the default behavior for all <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#category-autocapitalize" id="autocorrection:category-autocapitalize">autocapitalize-and-autocorrect inheriting elements</a> associated with the [`form`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-form-element) element.

The [`autocorrect`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-autocorrect) attribute never causes autocorrection to be enabled for [`input`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-input-element) elements whose [`type`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-input-type) attribute is in one of the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#url-state-(type=url)" id="autocorrection:url-state-(type=url)">URL</a>, <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#email-state-(type=email)" id="autocorrection:email-state-(type=email)">Email</a>, or <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#password-state-(type=password)" id="autocorrection:password-state-(type=password)">Password</a> states. (This behavior is included in the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#used-autocorrection-state" id="autocorrection:used-autocorrection-state">used autocorrection state</a> algorithm below.)

The <span id="attr-autocorrect" class="dfn" dfn-for="html-global" dfn-type="element-attr">`autocorrect`</span> attribute is an enumerated attribute with the following keywords and states:

| Keyword | State | Brief description |
|---|---|---|
| <span id="attr-autocorrect-on" class="dfn" dfn-for="html-global/autocorrect" dfn-type="attr-value">`on`</span> | <span id="concept-autocorrection-on" class="dfn">On</span> | The user agent is permitted to automatically correct spelling errors while the user types. Whether spelling is automatically corrected while typing left is for the user agent to decide, and may depend on the element as well as the user's preferences. |
| <span id="attr-autocorrect-off" class="dfn" dfn-for="html-global/autocorrect" dfn-type="attr-value">`off`</span> | <span id="concept-autocorrection-off" class="dfn">Off</span> | The user agent is not allowed to automatically correct spelling while the user types. |

<div algorithm="">

The attribute's *[invalid value default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#invalid-value-default)*, *[missing value default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#missing-value-default)*, and *[empty value default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#empty-value-default)* are all the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-autocorrection-on" id="autocorrection:concept-autocorrection-on">On</a> state.

</div>

<div algorithm="">

The <span id="dom-autocorrect" class="dfn" dfn-for="HTMLElement" dfn-type="attribute">`autocorrect`</span> getter steps are: return true if the element's <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#used-autocorrection-state" id="autocorrection:used-autocorrection-state-2">used autocorrection state</a> is <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-autocorrection-on" id="autocorrection:concept-autocorrection-on-2">On</a> and false if the element's <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#used-autocorrection-state" id="autocorrection:used-autocorrection-state-3">used autocorrection state</a> is <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-autocorrection-off" id="autocorrection:concept-autocorrection-off">Off</a>. The setter steps are: if the given value is true, then the element's [`autocorrect`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-autocorrect) attribute must be set to "`on`"; otherwise it must be set to "`off`".

</div>

<div algorithm="">

To compute the <span id="used-autocorrection-state" class="dfn">used autocorrection state</span> of an element <var>element</var>, run these steps:

1.  If <var>element</var> is an [`input`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-input-element) element whose [`type`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-input-type) attribute is in one of the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#url-state-(type=url)" id="autocorrection:url-state-(type=url)-2">URL</a>, <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#email-state-(type=email)" id="autocorrection:email-state-(type=email)-2">Email</a>, or <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#password-state-(type=password)" id="autocorrection:password-state-(type=password)-2">Password</a> states, then return <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-autocorrection-off" id="autocorrection:concept-autocorrection-off-2">Off</a>.
2.  If the [`autocorrect`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-autocorrect) content attribute is present on <var>element</var>, then return the state of the attribute.
3.  If <var>element</var> is an <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#category-autocapitalize" id="autocorrection:category-autocapitalize-2">autocapitalize-and-autocorrect inheriting element</a> and has a non-null <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#form-owner" id="autocorrection:form-owner">form owner</a>, then return the state of <var>element</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#form-owner" id="autocorrection:form-owner-2">form owner</a>'s [`autocorrect`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-autocorrect) attribute.
4.  Return <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-autocorrection-on" id="autocorrection:concept-autocorrection-on-3">On</a>.

</div>

<var>element</var> . [`autocorrect`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-autocorrect)  
Returns the autocorrection behavior of the element. Note that for <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#category-autocapitalize" id="autocorrection:category-autocapitalize-3">autocapitalize-and-autocorrect inheriting elements</a> that inherit their state from a [`form`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-form-element) element, this will return the autocorrection behavior of the [`form`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-form-element) element, but for an element in an editable region, this will not return the autocorrection behavior of the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#editing-host" id="autocorrection:editing-host-2">editing host</a> (unless this element is, in fact, the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#editing-host" id="autocorrection:editing-host-3">editing host</a>).

<var>element</var> . [`autocorrect`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-autocorrect) = <var>value</var>

Updates the [`autocorrect`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-autocorrect) content attribute (and thereby changes the autocorrection behavior of the element).


<div class="example">

The [`input`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-input-element) element in the following example would not allow autocorrection, since it does not have an [`autocorrect`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-autocorrect) content attribute and therefore inherits from the [`form`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-form-element) element, which has an attribute of "[`off`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-autocorrect-off)". However, the [`textarea`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-textarea-element) element would allow autocorrection, since it has an [`autocorrect`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-autocorrect) content attribute with a value of "[`on`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-autocorrect-on)".

``` html
<form autocorrect="off">
 <input type="search">
 <textarea autocorrect="on"></textarea>
</form>
```

</div>
