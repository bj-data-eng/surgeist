# HTML PageTransitionEvent

This IDL block comes from the pinned HTML revision
[`877d6146487f15b15bc8dd0e3c625063a37b456a`](https://github.com/whatwg/html/blob/877d6146487f15b15bc8dd0e3c625063a37b456a/source#L104699-L104708).
Copyright © WHATWG (Apple, Google, Mozilla, Microsoft).
See the [WHATWG license](../licenses/whatwg/LICENSE).

``` idl
[Exposed=Window]
interface PageTransitionEvent : Event {
  constructor(DOMString type, optional PageTransitionEventInit eventInitDict = {});

  readonly attribute boolean persisted;
};

dictionary PageTransitionEventInit : EventInit {
  boolean persisted = false;
};
```

The [page transition event firing algorithm](https://github.com/whatwg/html/blob/877d6146487f15b15bc8dd0e3c625063a37b456a/source#L104748-L104755)
initializes `bubbles` and `cancelable` to true and sets the legacy target
override. [Stop parsing](https://github.com/whatwg/html/blob/877d6146487f15b15bc8dd0e3c625063a37b456a/source#L149722-L149757)
uses this algorithm for `pageshow` with `persisted` false.
