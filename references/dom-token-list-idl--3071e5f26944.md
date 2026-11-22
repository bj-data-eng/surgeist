# DOMTokenList Web IDL source excerpt

Copyright WHATWG (Apple, Google, Mozilla, Microsoft). Specification prose is CC BY 4.0. Source incorporation follows the retained [WHATWG license](../licenses/whatwg/LICENSE).

Exact DOM revision `3071e5f269448350fdea79bb284eedaf17bc431e`. The full interface block is copied from this revision.

## <a id="interface-domtokenlist"></a>DOMTokenList

[DOMTokenList](https://dom.spec.whatwg.org/commit-snapshots/3071e5f269448350fdea79bb284eedaf17bc431e/#interface-domtokenlist)

```webidl
[Exposed=Window]
interface DOMTokenList {
  readonly attribute unsigned long length;
  getter DOMString? item(unsigned long index);
  boolean contains(DOMString token);
  [CEReactions] undefined add(DOMString... tokens);
  [CEReactions] undefined remove(DOMString... tokens);
  [CEReactions] boolean toggle(DOMString token, optional boolean force);
  [CEReactions] boolean replace(DOMString token, DOMString newToken);
  boolean supports(DOMString token);
  [CEReactions] stringifier attribute DOMString value;
  iterable<DOMString>;
};
```
