# HTML popover target reflection

This IDL block comes from the pinned HTML revision
[`877d6146487f15b15bc8dd0e3c625063a37b456a`](https://github.com/whatwg/html/blob/877d6146487f15b15bc8dd0e3c625063a37b456a/source#L93651-L93662).
Copyright © WHATWG (Apple, Google, Mozilla, Microsoft).
See the [WHATWG license](../licenses/whatwg/LICENSE).

``` idl
interface mixin PopoverTargetAttributes {
  [CEReactions, Reflect="popovertarget"] attribute Element? popoverTargetElement;
  [CEReactions] attribute DOMString popoverTargetAction;
};
```

The Element accessor uses the [reflected Element algorithm](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-associated-element).
The action accessor reflects the [enumerated content attribute](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-popovertargetaction)
limited to known values. Button and Input include this mixin.
