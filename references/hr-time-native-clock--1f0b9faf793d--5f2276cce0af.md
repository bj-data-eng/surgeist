# High Resolution Time: native hosted creation clock clauses

Bounded source retention for the native hosted lifecycle contribution. The
original W3C source remains authoritative; the separate implementation choice
for native clock sampling/coarsening does not modify these algorithms or claim
Performance, Unix-aligned time-origin exposure or JavaScript implementation.

## Source basis and attribution

Source: W3C Web Performance Working Group, *High Resolution Time*, Editor's
Draft, editor Yoav Weiss (source metadata). Immutable
[index.bs](https://github.com/w3c/hr-time/blob/1f0b9faf793db3b06c8e85462448e8a812d00fb9/index.bs),
commit `1f0b9faf793db3b06c8e85462448e8a812d00fb9` (1 September 2026).
This selected revision precedes HTML877d (9 October 2026). The retained full raw
input is 36,368 bytes, SHA-256
`e928551146aa839cb2d6da490b4c2b07921b596d92514c8a37c6062eac6bddde`,
Git blob `81721d312b772ca28e36f524d0bc388c99b94987`.

This document includes material copied from or derived from
[High Resolution Time](https://github.com/w3c/hr-time/blob/1f0b9faf793db3b06c8e85462448e8a812d00fb9/index.bs).
Copyright © 2026 World Wide Web Consortium. All Rights Reserved. This work is
distributed under the [W3C Software and Document License](../licenses/w3c/software-license-2023.txt)
in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the
implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.

The exact upstream [repository license declaration at the pin](https://github.com/w3c/hr-time/blob/1f0b9faf793db3b06c8e85462448e8a812d00fb9/LICENSE.md)
licenses all repository documents by contributors under the W3C Software and
Document License. Its named official license URL resolves to the
[2023 version](https://www.w3.org/copyright/software-license-2023/), whose full
notice and disclaimers are retained in the linked local file, SHA-256
`bd0e4ada87e6f70c3daceada3401c2f018e3333600b5a50fd57d3da6cea82da4`.
The exact 166-byte upstream declaration is retained with the source investigation,
SHA-256 `6eabf929228fcdce39d1aff9a837175928c4a19eafc4517b72d5781db9ece661`.
No attribution date range or alternate license is inferred.

Changes by Surgeist on 16 October 2026: offline bounded extraction, explanatory
headings/provenance and editorial execution-scope notes. The selected original
Bikeshed source fragments are preserved **byte for byte** in fenced text below;
source links in the wrapper point to their immutable original line ranges.
Original HTML tags, Bikeshed references/variables, spelling, units, conditions,
step order, normative language, notes and issue markers remain unchanged. This
literal representation does not render or execute Bikeshed, HTML or scripts;
it is implementation-support reference material, not a new specification.

The four original fragments, in table order and joined by one extra LF, have
SHA-256 `5f2276cce0afddcce60b76ebf97d5afb985bd1ff9e3d7d291f18d4183e6adb3a`.
The filename suffix identifies these original fragments, not the final file.

| Original index.bs lines | Complete retained unit | Original fragment SHA-256 |
| --- | --- | --- |
| [230–306](https://github.com/w3c/hr-time/blob/1f0b9faf793db3b06c8e85462448e8a812d00fb9/index.bs#L230-L306) | Clocks, moments and durations: complete Time Concepts section | `46b6b335ef01e8d7d02d1f08d3d87c74eb2e9505aa20e4aaa4f2d6827378a1d7` |
| [472–544](https://github.com/w3c/hr-time/blob/1f0b9faf793db3b06c8e85462448e8a812d00fb9/index.bs#L472-L544) | Complete Time Origin section, including external native-scope exclusions | `4a059e1c81460ce2394d9c10397729744bbb21fb58faac2680142f767a0ad965` |
| [545–565](https://github.com/w3c/hr-time/blob/1f0b9faf793db3b06c8e85462448e8a812d00fb9/index.bs#L545-L565) | Complete coarsen-time algorithm, both isolation branches | `28476156c0ca7de0bc0268515d6df15726fe3cb4ca1e94191ca9779d96dc2372` |
| [566–600](https://github.com/w3c/hr-time/blob/1f0b9faf793db3b06c8e85462448e8a812d00fb9/index.bs#L566-L600) | Complete relative, coarse-relative, current and shared-time algorithms | `3539097a32ed90eef218e40a4a35a7034a96f87f171abd0912d50c7416c5ab37` |

## Editorial native execution boundary

The selected native contract belongs to the owning #1124/#136 Architecture
determination. It uses one communicating owner's monotonic clock domain/epoch,
actual production `std::time::Instant` sampling and checked, domain-qualified
integer durations. That epoch is shared across its Globals; the actual initial
sample remains retained across Pending. Raw sampling capability is distinct from
coarsening: the current false-isolation projection selects a 100-microsecond
floor with no jitter. Missing capability, wrong domain or overflow refuses
without a synthetic value. These choices are not normative changes to the source
below, which still permits larger time-resolution values and retains the true-isolated
5-microsecond branch and potential jitter.

This contract exposes no Unix alignment, Performance API or timeOrigin value.
The complete source Time Origin section is nevertheless retained, including its
estimated Unix-epoch initialization, get-time-origin-timestamp algorithm, worker
note and unresolved communicating-group issue. Retaining those algorithms does
not implement or silently waive them; they remain outside this native execution
and exposure boundary. Broader clock/Performance behavior would need its actual
source and owner before being claimed.

The [hosted initial constructor/global/settings reference](html-hosted-global-agent-settings--877d6146487f--03a314e0aa55.md)
retains the actual HTML calls to unsafe shared current time and coarsen time,
including the settings object's complete isolated-capability algorithm. The
[navigable lifecycle reference](html-hosted-navigable-lifecycle--877d6146487f--3a9658155866.md)
retains make-active's source-ordered execution-ready update. These references do
not establish a native active-Boolean substitute or certify transitive browser
source closure. Document destruction/history/task dependencies remain separate
bounded source-retention work.

## Clocks, moments and durations: complete Time Concepts section

<!-- selected-source-fragment:230-306 -->
```text
<h2 id="sec-concepts">
  Time Concepts
</h2>

<h3 id="sec-clocks">
  Clocks
</h3>
<p>
  A <dfn>clock</dfn> tracks the passage of time and can report the
  <dfn data-export>unsafe current time</dfn> that an algorithm step is executing.
  There are many kinds of clocks. All clocks on the web platform
  attempt to count 1 millisecond of clock time per 1 millisecond of
  real-world time, but they differ in how they handle cases where they
  can't be exactly correct.
</p>
<div>

: The <dfn data-export id=dfn-wall-clock>wall clock</dfn>'s <dfn data-dfn-for="wall clock" data-export id="wall-clock-unsafe-current-time">unsafe current time</dfn>
:: is always as close as possible to a user's notion of time. Since a computer sometimes runs slow or fast or loses track of time, its [=wall clock=] sometimes needs to be adjusted, which means the [=wall clock/unsafe current time=] can decrease, making it unreliable for performance measurement or recording the orders of events. The web platform shares a [=wall clock=] with [[ECMA-262]] <a data-link-type="dfn" href="https://tc39.es/ecma262/#sec-time-values-and-time-range">time</a>.

: The <dfn data-export id=dfn-monotonic-clock>monotonic clock</dfn>'s <dfn data-export data-dfn-for="monotonic clock" id="monotonic-clock-unsafe-current-time">unsafe current time</dfn>
:: never decreases, so it can't be changed by system clock adjustments. The [=monotonic clock=] only exists within a single execution of the [=user agent=], so it can't be used to compare events that might happen in different executions.

</div>

<p class="note"><span class="allow-2119">Since the [=monotonic clock=] can't be adjusted to match the user's notion of time, it should be used for measurement, rather than user-visible times. For any time communication with the user, use the wall clock.</span></p>

<p class="note"><span class="allow-2119">The user agent can pick a new [=estimated monotonic time of the Unix epoch=] when the browser restarts, when it starts an isolated browsing session—e.g. incognito or a similar browsing mode—or when it creates an [=environment settings object=] that can't communicate with any existing settings objects. As a result, developers should not use shared timestamps as absolute time that holds its monotonic properties across all past, present, and future contexts; in practice, the monotonic properties only apply for contexts that can reach each other by exchanging messages via one of the provided messaging mechanisms - e.g. {{Window/postMessage(message, options)}}, {{BroadcastChannel}}, etc.</span></p>

<p class="note"><span class="allow-2119">In certain scenarios (e.g. when a tab is backgrounded), the user agent may choose to throttle timers and periodic callbacks run in that context or even freeze them entirely. Any such throttling should not affect the resolution or accuracy of the time returned by the monotonic clock.</span></p>

<h3 id="sec-moments-and-durations">
  Moments and Durations
</h3>
<p>
  Each [=clock=]'s <a href="#unsafe-current-time">unsafe current time</a> returns an <dfn>unsafe
  moment</dfn>. [=Coarsen time=] converts these [=unsafe moments=] to
  <dfn data-export data-lt="moment|coarsened moment" id=dfn-moment>coarsened moments</dfn> or just
  [=moments=]. [=Unsafe moments=] and [=moments=] from different clocks
  are not comparable.
</p>
<p class="note">
  [=Moments=] and [=unsafe moments=] represent points in time, which
  means they can't be directly stored as numbers. Implementations will
  usually represent a [=moment=] as a [=duration=] from some other
  fixed point in time, but specifications ought to deal in the
  [=moments=] themselves.
</p>
<p>
  A <dfn data-export id=dfn-duration>duration</dfn> is the distance from one
  [=moment=] to another from the same [=clock=]. Neither endpoint can
  be an [=unsafe moment=] so that both [=durations=] and differences of
  [=durations=] mitigate the concerns in [[[#clock-resolution]]].
  [=Durations=] are measured in milliseconds, seconds, etc. Since all
  [=clocks=] attempt to count at the same rate, [=durations=] don't
  have an associated [=clock=], and a [=duration=] calculated from two
  [=moments=] on one clock can be added to a [=moment=] from a second
  [=clock=], to produce another [=moment=] on that second [=clock=].
</p>
<p>
  The <dfn data-export id=dfn-duration-from>duration from</dfn> |a| to |b|
  is the result of the following algorithm:
</p>
<ol class="algorithm">
  <li>Assert: |a| was created by the same [=clock=] as |b|.
  </li>
  <li>Assert: Both |a| and |b| are [=coarsened moments=].
  </li>
  <li>Return the amount of time from |a| to |b| as a [=duration=]. If
  |b| came before |a|, this will be a negative [=duration=].
  </li>
</ol>
<p>
  [=Durations=] can be used implicitly as {{DOMHighResTimeStamp}}s. To
  <dfn>implicitly convert a duration to a timestamp</dfn>, given a
  [=duration=] <var ignore>d</var>, return the number of milliseconds in <var ignore>d</var>.
</p>
```

## Complete Time Origin section, including external native-scope exclusions

<!-- selected-source-fragment:472-544 -->
```text
<h2 id="sec-time-origin">
  Time Origin
</h2>
<p>
  The <dfn data-export id=dfn-unix-epoch>Unix epoch</dfn> is the [=moment=] on the
  [=wall clock=] corresponding to 1 January 1970 00:00:00 UTC.
</p>
<p>
  Each group of [=environment settings objects=] that could possibly
  communicate in any way has an <dfn>estimated monotonic time of the Unix
  epoch</dfn>, a [=moment=] on the [=monotonic clock=], whose value is
  initialized by the following steps:
</p>
<ol data-algorithm=
"initialize the estimated monotonic time of the Unix epoch">
  <li>Let <var ignore>wall time</var> be the [=wall
  clock=]'s [=wall clock/unsafe current time=].
  </li>
  <li>Let <var ignore>monotonic time</var> be the
  [=monotonic clock=]'s [=monotonic clock/unsafe current time=].
  </li>
  <li>Let <var ignore>epoch time</var> be
  <code><var ignore>monotonic time</var> - (<var ignore>wall time</var> - [=Unix epoch=])</code>
  </li>
  <li>Initialize the [=estimated monotonic time of the Unix epoch=] to
  the result of calling [=coarsen time=] with <var ignore>epoch time</var>.
  </li>
</ol>
<div class="issue">
  The above set of settings-objects-that-could-possibly-communicate needs
  to be specified better. It's similar to <a href=
  "https://html.spec.whatwg.org/multipage/browsers.html#familiar-with">familiar with</a> but includes
  {{Worker}}s.
</div>
<p>
  Performance measurements report a [=duration=] from a [=moment=] early
  in the initialization of a relevant [=environment settings object=].
  That [=moment=] is stored in that settings object's [=environment
  settings object/time origin=].
</p>
<p>
  To <dfn>get time origin timestamp</dfn>, given a [=/global object=]
  |global|, run the following steps, which return a
  [=duration=]:
</p>
<ol>
  <li>
    <p>
      Let <var ignore>timeOrigin</var> be |global|'s
      [=relevant settings object=]'s [=environment settings object/time
      origin=].
    </p>
    <p class="note">
      In {{Window}} contexts, this value represents the time when
      <a href="https://html.spec.whatwg.org/multipage/browsing-the-web.html#navigate">navigation has started</a>. In {{Worker}} and
      {{ServiceWorker}} contents, this value represent the time when the
      <a href="https://html.spec.whatwg.org/multipage/webappapis.html#run-a-worker">worker is run</a>. [[SERVICE-WORKERS]]
    </p>
  </li>
  <li>Return the [=duration from=] the [=estimated monotonic time of the
  Unix epoch=] to <var ignore>timeOrigin</var>.
  </li>
</ol>
<p class="note">
  <span class="allow-2119">
  The value returned by [=get time origin timestamp=] is approximately
  the time after the [=Unix epoch=] that |global|'s [=environment
  settings object/time origin=] happened. It may differ from the value
  returned by <a data-link-type="dfn" href="https://tc39.es/ecma262/#sec-date.now">Date.now()</a> executed at the time origin, because the
  former is recorded with respect to a [=monotonic clock=] that is not
  subject to system and user clock adjustments, clock skew, and so on.
  </span>
</p>
```

## Complete coarsen-time algorithm, both isolation branches

<!-- selected-source-fragment:545-565 -->
```text
<div data-algorithm="coarsen time">
  The <dfn data-export id=dfn-coarsen-time>coarsen time</dfn> algorithm, given an [=unsafe
  moment=] |timestamp| on some [=clock=] and an optional
  boolean <var ignore>crossOriginIsolatedCapability</var> (default false), runs
  the following steps:
  <ol>
    <li>Let |time resolution| be 100 microseconds, or a higher
    [=implementation-defined=] value.
    </li>
    <li>If <var ignore>crossOriginIsolatedCapability</var> is true, set |time resolution|
    to be 5 microseconds, or a higher [=implementation-defined=]
    value.
    </li>
    <li>In an [=implementation-defined=] manner, coarsen and
    potentially jitter |timestamp| such that its resolution will not
    exceed |time resolution|.
    </li>
    <li>Return |timestamp| as a [=moment=].
    </li>
  </ol>
</div>
```

## Complete relative, coarse-relative, current and shared-time algorithms

<!-- selected-source-fragment:566-600 -->
```text
<div data-algorithm="relative high resolution time">
  The <dfn data-export id=dfn-relative-high-resolution-time>relative high resolution time</dfn> given an
  [=unsafe moment=] from the [=monotonic clock=] <var ignore>time</var>
  and a [=/global object=] |global|, is the [=duration=] returned from the following steps:
  <ol>
    <li>Let <var ignore>coarse time</var> be the result of
    calling [=coarsen time=] with <var ignore>time</var> and |global|'s [=relevant
    settings object=]'s [=environment settings object/cross-origin
    isolated capability=].
    </li>
    <li>Return the [=relative high resolution coarse time=] for <var ignore>coarse
    time</var> and |global|.
    </li>
  </ol>The <dfn data-export id=dfn-relative-high-resolution-coarse-time>relative high resolution coarse time</dfn>
  given a [=moment=] from the [=monotonic clock=] <var ignore>coarseTime</var>
  and a [=/global object=] |global|, is the [=duration from=] |global|'s [=relevant settings
  object=]'s [=environment settings object/time origin=] to <var ignore>coarseTime</var>.
</div>
<p>
  The <dfn data-export id=dfn-current-high-resolution-time>current high resolution time</dfn> given a
  [=/global object=] <var ignore>current global</var> must return the
  result of [=relative high resolution time=] given [=unsafe shared
  current time=] and <var ignore>current global</var>.
</p>
<p>
  The <dfn data-export id=dfn-coarsened-shared-current-time>coarsened shared current time</dfn> given an
  optional boolean <var ignore>crossOriginIsolatedCapability</var> (default
  false), must return the result of calling [=coarsen time=] with the
  [=unsafe shared current time=] and <var ignore>crossOriginIsolatedCapability</var>.
</p>
<p>
  The <dfn data-export id=dfn-unsafe-shared-current-time>unsafe shared current time</dfn> must return
  the [=monotonic clock/unsafe current time=] of the [=monotonic
  clock=].
</p>
```

