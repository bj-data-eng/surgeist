# Explanation

## Animation ownership

This crate owns the standards-oriented animation engine layer for Surgeist:
easing evaluation, transition timing, keyframe timing, iteration, direction,
fill-mode, play-state, interpolation contracts, sampled animation values, and
animation-specific diagnostics.

- `surgeist-style` owns authored CSS animation and transition declarations.
- `surgeist-animation` owns executable animation timing and sampling contracts.
- `surgeist-runtime` owns clocks, scheduling, lifecycle, and invalidation.
- Root `surgeist` owns cross-crate lowering and integration.
- `surgeist-render`, `surgeist-layout`, and `surgeist-text` consume sampled
  values through root-owned integration.

The crate and facade now share one source repository and Cargo workspace.
Cross-crate composition and generated API audits remain root-owned; their current
behavior and coverage come from root source and verification evidence.

## Why inputs are normalized

Separating authored style from executable timing lets callers resolve property
support, computed endpoints, keyframe lookup, and list pairing before sampling.
Runtime resolves clocks and track start times into effective elapsed time. The
animation crate can then sample explicit inputs without owning a scheduler or
CSS parser. Root owns the adapters between these boundaries.

## Values and failure semantics

Unrestricted finite percentages and eased progress preserve values outside the
unit interval. Explicit premultiplied color spaces avoid ambiguity about the
representation being interpolated. Consumers retain responsibility for conversion,
gamut mapping, and final property clamping.

Unsupported interpolation is a permanent capability outcome; arithmetic failure
is local to a sample and can recover at a later endpoint. Next-frame hints also
account for play state, so paused samples remain stable until external input.
See the [reference](reference.md) for the exact contracts and limitations.

## App-native motion

This remains distinct from app-native motion work. A future `surgeist-motion`
crate could own higher-level springs, gestures, layout transitions, presence,
and scroll-linked motion if those boundaries become concrete. That possibility
does not add those responsibilities to this crate.
