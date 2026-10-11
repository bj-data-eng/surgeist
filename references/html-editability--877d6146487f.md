# HTML: editability, design mode and spellcheck

Source: [HTML commit snapshot](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/), revision `877d6146487f15b15bc8dd0e3c625063a37b456a`; complete HTML SHA-256 `877d6cc00dae6c18c16d952fdcbf944acc58919579621b4257fa0ed1888a871f`. Retrieved 2026-10-11. Full sections 6.8.1, 6.8.2, 6.8.4 and 6.8.5. Source authority remains the pinned English HTML standard.

Copyright © WHATWG (Apple, Google, Mozilla, Microsoft). Specification prose is CC BY 4.0; see [WHATWG license](../licenses/whatwg/LICENSE). Partial extraction and Markdown reformatting by Surgeist, 2026-10-11, Pandoc 3.1.11.1. Source scripts and MDN annotations omitted; links made absolute. No endorsement or semantic changes. This reference does not expand implementation scope to editing commands, focus execution, or spelling services.

<div>

#### <span class="secno">6.8.1</span> Making document regions editable: The [`contenteditable`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-contenteditable) content attribute<a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#contenteditable" class="self-link"></a>

``` idl
interface mixin ElementContentEditable {
  [CEReactions] attribute DOMString contentEditable;
  [CEReactions] attribute DOMString enterKeyHint;
  readonly attribute boolean isContentEditable;
  [CEReactions] attribute DOMString inputMode;
};
```

The <span id="attr-contenteditable" class="dfn" dfn-for="html-global" dfn-type="element-attr">`contenteditable`</span> content attribute is an <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#enumerated-attribute" id="contenteditable:enumerated-attribute">enumerated attribute</a> with the following keywords and states:

| Keyword                                                                                                                                       | State                                                                                 | Brief description                                                             |
|-----------------------------------------------------------------------------------------------------------------------------------------------|---------------------------------------------------------------------------------------|-------------------------------------------------------------------------------|
| <span id="attr-contenteditable-true" class="dfn" dfn-for="html-global/contenteditable" dfn-type="attr-value">`true`</span>                    | <span id="attr-contenteditable-true-state" class="dfn">True</span>                    | The element is editable.                                                      |
| <span id="attr-contenteditable-false" class="dfn" dfn-for="html-global/contenteditable" dfn-type="attr-value">`false`</span>                  | <span id="attr-contenteditable-false-state" class="dfn">False</span>                  | The element is not editable.                                                  |
| <span id="attr-contenteditable-plaintextonly" class="dfn" dfn-for="html-global/contenteditable" dfn-type="attr-value">`plaintext-only`</span> | <span id="attr-contenteditable-plaintextonly-state" class="dfn">Plaintext-Only</span> | Only the element's raw text content is editable; rich formatting is disabled. |

<div algorithm="">

The attribute's *[missing value default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#missing-value-default)* and *[invalid value default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#invalid-value-default)* are both the <span id="attr-contenteditable-inherit-state" class="dfn">Inherit</span> state. The inherit state indicates that the element is editable (or not) based on the parent element's state. The attribute's *[empty value default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#empty-value-default)* is the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-contenteditable-true-state" id="contenteditable:attr-contenteditable-true-state">True</a> state.

</div>

<div class="example">

For example, consider a page that has a [`form`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-form-element) and a [`textarea`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-textarea-element) to publish a new article, where the user is expected to write the article using HTML:

``` html
<form method=POST>
 <fieldset>
  <legend>New article</legend>
  <textarea name=article>&lt;p>Hello world.&lt;/p></textarea>
 </fieldset>
 <p><button>Publish</button></p>
</form>
```

When scripting is enabled, the [`textarea`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-textarea-element) element could be replaced with a rich text control instead, using the [`contenteditable`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-contenteditable) attribute:

``` html
<form method=POST>
 <fieldset>
  <legend>New article</legend>
  <textarea id=textarea name=article>&lt;p>Hello world.&lt;/p></textarea>
  <div id=div style="white-space: pre-wrap" hidden><p>Hello world.</p></div>
  <script>
   let textarea = document.getElementById("textarea");
   let div = document.getElementById("div");
   textarea.hidden = true;
   div.hidden = false;
   div.contentEditable = "true";
   div.oninput = (e) => {
     textarea.value = div.innerHTML;
   };
  </script>
 </fieldset>
 <p><button>Publish</button></p>
</form>
```

Features to enable, e.g., inserting links, can be implemented using the <a href="https://w3c.github.io/editing/docs/execCommand/#execcommand%28%29" data-x-internal="execCommand"><code id="contenteditable:execCommand">document.execCommand()</code></a> API, or using <a href="https://w3c.github.io/selection-api/#selection-interface" data-x-internal="selection"><code id="contenteditable:selection">Selection</code></a> APIs and other DOM APIs. [\[EXECCOMMAND\]](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsEXECCOMMAND) [\[SELECTION\]](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsSELECTION) [\[DOM\]](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsDOM)

</div>

<div class="example">

The [`contenteditable`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-contenteditable) attribute can also be used to great effect:

``` html
<!doctype html>
<html lang=en>
<title>Live CSS editing!</title>
<style style=white-space:pre contenteditable>
html { margin:.2em; font-size:2em; color:lime; background:purple }
head, title, style { display:block }
body { display:none }
</style>
```

</div>

<code><var>element</var>.<a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-contenteditable" id="dom-contenteditable-dev">contentEditable</a> [ = <var>value</var> ]</code>  
Returns "`true`", "`plaintext-only`", "`false`", or "`inherit`", based on the state of the [`contenteditable`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-contenteditable) attribute.

Can be set, to change that state.

Throws a <a href="https://webidl.spec.whatwg.org/#syntaxerror" id="contenteditable:syntaxerror" data-x-internal="syntaxerror">"<code>SyntaxError</code>"</a> <a href="https://webidl.spec.whatwg.org/#dfn-DOMException" data-x-internal="domexception"><code id="contenteditable:domexception">DOMException</code></a> if the new value isn't one of those strings.

<code><var>element</var>.<a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-iscontenteditable" id="dom-iscontenteditable-dev">isContentEditable</a></code>  
Returns true if the element is editable; otherwise, returns false.

<div algorithm="">

The <span id="dom-contenteditable" class="dfn" dfn-for="ElementContentEditable" dfn-type="attribute">`contentEditable`</span> IDL attribute, on getting, must return the string "`true`" if the content attribute is set to the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-contenteditable-true-state" id="contenteditable:attr-contenteditable-true-state-2">True</a> state, "`plaintext-only`" if the content attribute is set to the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-contenteditable-plaintextonly-state" id="contenteditable:attr-contenteditable-plaintextonly-state">Plaintext-Only</a> state, "`false`" if the content attribute is set to the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-contenteditable-false-state" id="contenteditable:attr-contenteditable-false-state">False</a> state, and "`inherit`" otherwise. On setting, if the new value is an <a href="https://infra.spec.whatwg.org/#ascii-case-insensitive" id="contenteditable:ascii-case-insensitive" data-x-internal="ascii-case-insensitive">ASCII case-insensitive</a> match for the string "`inherit`", then the content attribute must be removed, if the new value is an <a href="https://infra.spec.whatwg.org/#ascii-case-insensitive" id="contenteditable:ascii-case-insensitive-2" data-x-internal="ascii-case-insensitive">ASCII case-insensitive</a> match for the string "`true`", then the content attribute must be set to the string "`true`", if the new value is an <a href="https://infra.spec.whatwg.org/#ascii-case-insensitive" id="contenteditable:ascii-case-insensitive-3" data-x-internal="ascii-case-insensitive">ASCII case-insensitive</a> match for the string "`plaintext-only`", then the content attribute must be set to the string "`plaintext-only`", if the new value is an <a href="https://infra.spec.whatwg.org/#ascii-case-insensitive" id="contenteditable:ascii-case-insensitive-4" data-x-internal="ascii-case-insensitive">ASCII case-insensitive</a> match for the string "`false`", then the content attribute must be set to the string "`false`", and otherwise the attribute setter must throw a <a href="https://webidl.spec.whatwg.org/#syntaxerror" id="contenteditable:syntaxerror-2" data-x-internal="syntaxerror">"<code>SyntaxError</code>"</a> <a href="https://webidl.spec.whatwg.org/#dfn-DOMException" data-x-internal="domexception"><code id="contenteditable:domexception-2">DOMException</code></a>.

</div>

<div algorithm="">

The <span id="dom-iscontenteditable" class="dfn" dfn-for="ElementContentEditable" dfn-type="attribute">`isContentEditable`</span> IDL attribute, on getting, must return true if the element is either an <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#editing-host" id="contenteditable:editing-host">editing host</a> or <a href="https://w3c.github.io/editing/docs/execCommand/#editable" id="contenteditable:editable" data-x-internal="editable">editable</a>, and false otherwise.

</div>

#### <span class="secno">6.8.2</span> Making entire documents editable: the [`designMode`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#designMode) getter and setter<a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#making-entire-documents-editable:-the-designmode-idl-attribute" class="self-link"></a>

<code><var>document</var>.<a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#designMode" id="dom-document-designmode-dev">designMode</a> [ = <var>value</var> ]</code>  
Returns "`on`" if the document is editable, and "`off`" if it isn't.

Can be set, to change the document's current state. This focuses the document and resets the selection in that document.

[`Document`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document) objects have an associated <span id="design-mode-enabled" class="dfn">design mode enabled</span>, which is a boolean. It is initially false.

<div algorithm="">

The <span id="designMode" class="dfn" dfn-for="Document" dfn-type="attribute">`designMode`</span> getter steps are to return "`on`" if <a href="https://webidl.spec.whatwg.org/#this" id="making-entire-documents-editable:-the-designmode-idl-attribute:this" data-x-internal="this">this</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#design-mode-enabled" id="making-entire-documents-editable:-the-designmode-idl-attribute:design-mode-enabled">design mode enabled</a> is true; otherwise "`off`".

</div>

<div algorithm="">

The [`designMode`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#designMode) setter steps are:

1.  Let `value` be the given value, <a href="https://infra.spec.whatwg.org/#ascii-lowercase" id="making-entire-documents-editable:-the-designmode-idl-attribute:converted-to-ascii-lowercase" data-x-internal="converted-to-ascii-lowercase">converted to ASCII lowercase</a>.

2.  If `value` is "`on`" and <a href="https://webidl.spec.whatwg.org/#this" id="making-entire-documents-editable:-the-designmode-idl-attribute:this-2" data-x-internal="this">this</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#design-mode-enabled" id="making-entire-documents-editable:-the-designmode-idl-attribute:design-mode-enabled-2">design mode enabled</a> is false:

    1.  Set <a href="https://webidl.spec.whatwg.org/#this" id="making-entire-documents-editable:-the-designmode-idl-attribute:this-3" data-x-internal="this">this</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#design-mode-enabled" id="making-entire-documents-editable:-the-designmode-idl-attribute:design-mode-enabled-3">design mode enabled</a> to true.

    2.  Reset <a href="https://webidl.spec.whatwg.org/#this" id="making-entire-documents-editable:-the-designmode-idl-attribute:this-4" data-x-internal="this">this</a>'s <a href="https://w3c.github.io/editing/docs/execCommand/#active-range" id="making-entire-documents-editable:-the-designmode-idl-attribute:active-range" data-x-internal="active-range">active range</a>'s start and end boundary points to be at the start of <a href="https://webidl.spec.whatwg.org/#this" id="making-entire-documents-editable:-the-designmode-idl-attribute:this-5" data-x-internal="this">this</a>.

    3.  Run the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#focusing-steps" id="making-entire-documents-editable:-the-designmode-idl-attribute:focusing-steps">focusing steps</a> for <a href="https://webidl.spec.whatwg.org/#this" id="making-entire-documents-editable:-the-designmode-idl-attribute:this-6" data-x-internal="this">this</a>'s <a href="https://dom.spec.whatwg.org/#document-element" id="making-entire-documents-editable:-the-designmode-idl-attribute:document-element" data-x-internal="document-element">document element</a>, if non-null.

3.  If `value` is "`off`", then set <a href="https://webidl.spec.whatwg.org/#this" id="making-entire-documents-editable:-the-designmode-idl-attribute:this-7" data-x-internal="this">this</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#design-mode-enabled" id="making-entire-documents-editable:-the-designmode-idl-attribute:design-mode-enabled-4">design mode enabled</a> to false.

</div>

#### <span class="secno">6.8.4</span> Editing APIs<a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#editing-apis" class="self-link"></a>

<div algorithm="">

An <span id="editing-host" class="dfn" export="">editing host</span> is either an <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#html-elements" id="editing-apis:html-elements">HTML element</a> with its [`contenteditable`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-contenteditable) attribute in the *true* state or *plaintext-only* state, or a <a href="https://dom.spec.whatwg.org/#concept-tree-child" id="editing-apis:concept-tree-child" data-x-internal="concept-tree-child">child</a> <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#html-elements" id="editing-apis:html-elements-2">HTML element</a> of a [`Document`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document) whose <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#design-mode-enabled" id="editing-apis:design-mode-enabled">design mode enabled</a> is true.

</div>

The definition of the terms <span id="active-range" class="dfn">[active range](https://w3c.github.io/editing/docs/execCommand/#active-range)</span>, <span id="editing-host-of" class="dfn">[editing host of](https://w3c.github.io/editing/docs/execCommand/#editing-host-of)</span>, and <span id="editable" class="dfn">[editable](https://w3c.github.io/editing/docs/execCommand/#editable)</span>, the user interface requirements of elements that are <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#editing-host" id="editing-apis:editing-host">editing hosts</a> or <a href="https://w3c.github.io/editing/docs/execCommand/#editable" id="editing-apis:editable" data-x-internal="editable">editable</a>, the <span id="execCommand" class="dfn">[`execCommand()`](https://w3c.github.io/editing/docs/execCommand/#execcommand%28%29)</span>, <span id="dom-document-querycommandenabled" class="dfn">[`queryCommandEnabled()`](https://w3c.github.io/editing/docs/execCommand/#querycommandenabled%28%29)</span>, <span id="dom-document-querycommandindeterm" class="dfn">[`queryCommandIndeterm()`](https://w3c.github.io/editing/docs/execCommand/#querycommandindeterm%28%29)</span>, <span id="dom-document-querycommandstate" class="dfn">[`queryCommandState()`](https://w3c.github.io/editing/docs/execCommand/#querycommandstate%28%29)</span>, <span id="dom-document-querycommandsupported" class="dfn">[`queryCommandSupported()`](https://w3c.github.io/editing/docs/execCommand/#querycommandsupported%28%29)</span>, and <span id="dom-document-querycommandvalue" class="dfn">[`queryCommandValue()`](https://w3c.github.io/editing/docs/execCommand/#querycommandvalue%28%29)</span> methods, text selections, and the <span id="delete-the-selection" class="dfn">[delete the selection](https://w3c.github.io/editing/docs/execCommand/#delete-the-selection)</span> algorithm are defined in execCommand. [\[EXECCOMMAND\]](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsEXECCOMMAND)

#### <span class="secno">6.8.5</span> Spelling and grammar checking<a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#spelling-and-grammar-checking" class="self-link"></a>

User agents can support the checking of spelling and grammar of editable text, either in form controls (such as the value of [`textarea`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-textarea-element) elements), or in elements in an <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#editing-host" id="spelling-and-grammar-checking:editing-host">editing host</a> (e.g. using [`contenteditable`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-contenteditable)).

For each element, user agents must establish a <span id="concept-spellcheck-default" class="dfn">default behavior</span>, either through defaults or through preferences expressed by the user. There are three possible default behaviors for each element:

<span id="concept-spellcheck-default-true" class="dfn">true-by-default</span>  
The element will be checked for spelling and grammar if its contents are editable and spellchecking is not explicitly disabled through the [`spellcheck`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-spellcheck) attribute.

<span id="concept-spellcheck-default-false" class="dfn">false-by-default</span>  
The element will never be checked for spelling and grammar unless spellchecking is explicitly enabled through the [`spellcheck`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-spellcheck) attribute.

<span id="concept-spellcheck-default-inherit" class="dfn">inherit-by-default</span>  
The element's default behavior is the same as its parent element's. Elements that have no parent element cannot have this as their default behavior.

------------------------------------------------------------------------

The <span id="attr-spellcheck" class="dfn" dfn-for="html-global" dfn-type="element-attr">`spellcheck`</span> attribute is an <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#enumerated-attribute" id="spelling-and-grammar-checking:enumerated-attribute">enumerated attribute</a> with the following keywords and states:

| Keyword                                                                                                            | State                                                           | Brief description                         |
|--------------------------------------------------------------------------------------------------------------------|-----------------------------------------------------------------|-------------------------------------------|
| <span id="attr-spellcheck-true" class="dfn" dfn-for="html-global/spellcheck" dfn-type="attr-value">`true`</span>   | <span id="attr-spellcheck-true-state" class="dfn">True</span>   | Spelling and grammar will be checked.     |
| <span id="attr-spellcheck-false" class="dfn" dfn-for="html-global/spellcheck" dfn-type="attr-value">`false`</span> | <span id="attr-spellcheck-false-state" class="dfn">False</span> | Spelling and grammar will not be checked. |

<div algorithm="">

The attribute's *[missing value default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#missing-value-default)* and *[invalid value default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#invalid-value-default)* are both the <span id="attr-spellcheck-default-state" class="dfn">Default</span> state. The default state indicates that the element is to act according to a default behavior, possibly based on the parent element's own [`spellcheck`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-spellcheck) state, as defined below. The attribute's *[empty value default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#empty-value-default)* is the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-spellcheck-true-state" id="spelling-and-grammar-checking:attr-spellcheck-true-state">True</a> state.

</div>

------------------------------------------------------------------------

<code><var>element</var>.<a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-spellcheck" id="dom-spellcheck-dev">spellcheck</a> [ = <var>value</var> ]</code>  
Returns true if the element is to have its spelling and grammar checked; otherwise, returns false.

Can be set, to override the default and set the [`spellcheck`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-spellcheck) content attribute.

<div algorithm="">

The <span id="dom-spellcheck" class="dfn" dfn-for="HTMLElement" dfn-type="attribute">`spellcheck`</span> IDL attribute, on getting, must return true if the element's [`spellcheck`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-spellcheck) content attribute is in the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-spellcheck-true-state" id="spelling-and-grammar-checking:attr-spellcheck-true-state-2">True</a> state, or if the element's [`spellcheck`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-spellcheck) content attribute is in the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-spellcheck-default-state" id="spelling-and-grammar-checking:attr-spellcheck-default-state">Default</a> state and the element's <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-spellcheck-default" id="spelling-and-grammar-checking:concept-spellcheck-default">default behavior</a> is <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-spellcheck-default-true" id="spelling-and-grammar-checking:concept-spellcheck-default-true">true-by-default</a>, or if the element's [`spellcheck`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-spellcheck) content attribute is in the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-spellcheck-default-state" id="spelling-and-grammar-checking:attr-spellcheck-default-state-2">Default</a> state and the element's <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-spellcheck-default" id="spelling-and-grammar-checking:concept-spellcheck-default-2">default behavior</a> is <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-spellcheck-default-inherit" id="spelling-and-grammar-checking:concept-spellcheck-default-inherit">inherit-by-default</a> and the element's parent element's [`spellcheck`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-spellcheck) IDL attribute would return true; otherwise, if none of those conditions applies, then the attribute must instead return false.

</div>

The [`spellcheck`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-spellcheck) IDL attribute is not affected by user preferences that override the [`spellcheck`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-spellcheck) content attribute, and therefore might not reflect the actual spellchecking state.

<div algorithm="">

On setting, if the new value is true, then the element's [`spellcheck`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-spellcheck) content attribute must be set to "`true`", otherwise it must be set to "`false`".

</div>

------------------------------------------------------------------------

User agents should only consider the following pieces of text as checkable for the purposes of this feature:

- The <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-fe-value" id="spelling-and-grammar-checking:concept-fe-value">value</a> of [`input`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-input-element) elements whose [`type`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-input-type) attributes are in the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#text-(type=text)-state-and-search-state-(type=search)" id="spelling-and-grammar-checking:text-(type=text)-state-and-search-state-(type=search)">Text</a>, <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#text-(type=text)-state-and-search-state-(type=search)" id="spelling-and-grammar-checking:text-(type=text)-state-and-search-state-(type=search)-2">Search</a>, <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#url-state-(type=url)" id="spelling-and-grammar-checking:url-state-(type=url)">URL</a>, or <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#email-state-(type=email)" id="spelling-and-grammar-checking:email-state-(type=email)">Email</a> states and that are <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-fe-mutable" id="spelling-and-grammar-checking:concept-fe-mutable">mutable</a> (i.e. that do not have the [`readonly`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-input-readonly) attribute specified and that are not <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-fe-disabled" id="spelling-and-grammar-checking:concept-fe-disabled">disabled</a>).
- The <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-fe-value" id="spelling-and-grammar-checking:concept-fe-value-2">value</a> of [`textarea`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-textarea-element) elements that do not have a [`readonly`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-textarea-readonly) attribute and that are not <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-fe-disabled" id="spelling-and-grammar-checking:concept-fe-disabled-2">disabled</a>.
- Text in <a href="https://dom.spec.whatwg.org/#interface-text" data-x-internal="text"><code id="spelling-and-grammar-checking:text">Text</code></a> nodes that are children of <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#editing-host" id="spelling-and-grammar-checking:editing-host-2">editing hosts</a> or <a href="https://w3c.github.io/editing/docs/execCommand/#editable" id="spelling-and-grammar-checking:editable" data-x-internal="editable">editable</a> elements.
- Text in attributes of <a href="https://w3c.github.io/editing/docs/execCommand/#editable" id="spelling-and-grammar-checking:editable-2" data-x-internal="editable">editable</a> elements.

For text that is part of a <a href="https://dom.spec.whatwg.org/#interface-text" data-x-internal="text"><code id="spelling-and-grammar-checking:text-2">Text</code></a> node, the element with which the text is associated is the element that is the immediate parent of the first character of the word, sentence, or other piece of text. For text in attributes, it is the attribute's element. For the values of [`input`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-input-element) and [`textarea`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-textarea-element) elements, it is the element itself.

<div algorithm="">

To determine if a word, sentence, or other piece of text in an applicable element (as defined above) is to have spelling- and grammar-checking enabled, the UA must use the following algorithm:

1.  If the user has disabled the checking for this text, then the checking is disabled.
2.  Otherwise, if the user has forced the checking for this text to always be enabled, then the checking is enabled.
3.  Otherwise, if the element with which the text is associated has a [`spellcheck`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-spellcheck) content attribute, then: if that attribute is in the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-spellcheck-true-state" id="spelling-and-grammar-checking:attr-spellcheck-true-state-3">True</a> state, then checking is enabled; otherwise, if that attribute is in the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-spellcheck-false-state" id="spelling-and-grammar-checking:attr-spellcheck-false-state">False</a> state, then checking is disabled.
4.  Otherwise, if there is an ancestor element with a [`spellcheck`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-spellcheck) content attribute that is not in the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-spellcheck-default-state" id="spelling-and-grammar-checking:attr-spellcheck-default-state-3">Default</a> state, then: if the nearest such ancestor's [`spellcheck`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-spellcheck) content attribute is in the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-spellcheck-true-state" id="spelling-and-grammar-checking:attr-spellcheck-true-state-4">True</a> state, then checking is enabled; otherwise, checking is disabled.
5.  Otherwise, if the element's <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-spellcheck-default" id="spelling-and-grammar-checking:concept-spellcheck-default-3">default behavior</a> is <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-spellcheck-default-true" id="spelling-and-grammar-checking:concept-spellcheck-default-true-2">true-by-default</a>, then checking is enabled.
6.  Otherwise, if the element's <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-spellcheck-default" id="spelling-and-grammar-checking:concept-spellcheck-default-4">default behavior</a> is <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-spellcheck-default-false" id="spelling-and-grammar-checking:concept-spellcheck-default-false">false-by-default</a>, then checking is disabled.
7.  Otherwise, if the element's parent element has *its* checking enabled, then checking is enabled.
8.  Otherwise, checking is disabled.

</div>

If the checking is enabled for a word/sentence/text, the user agent should indicate spelling and grammar errors in that text. User agents should take into account the other semantics given in the document when suggesting spelling and grammar corrections. User agents may use the language of the element to determine what spelling and grammar rules to use, or may use the user's preferred language settings. UAs should use [`input`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-input-element) element attributes such as [`pattern`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-input-pattern) to ensure that the resulting value is valid, where possible.

If checking is disabled, the user agent should not indicate spelling or grammar errors for that text.

<div class="example">

The element with ID "a" in the following example would be the one used to determine if the word "Hello" is checked for spelling errors. In this example, it would not be.

``` html
<div contenteditable="true">
 <span spellcheck="false" id="a">Hell</span><em>o!</em>
</div>
```

The element with ID "b" in the following example would have checking enabled (the leading space character in the attribute's value on the [`input`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-input-element) element causes the attribute to be ignored, so the ancestor's value is used instead, regardless of the default).

``` bad
<p spellcheck="true">
 <label>Name: <input spellcheck=" false" id="b"></label>
</p>
```

</div>

This specification does not define the user interface for spelling and grammar checkers. A user agent could offer on-demand checking, could perform continuous checking while the checking is enabled, or could use other interfaces.

</div>
