# Captured browser expectations

This directory owns the fixed parent for versioned, source-qualified capture
editions. Historical [HTML](../html/) and [raw XML](../xml/) remain unchanged.

The root-context-v4 edition is created only by the maintained
[qualify-root-context host](../../../../examples/qualify-root-context/README.md#durable-root-context-edition).
The [Grid static-position edition](../../../../examples/qualify-root-context/README.md#grid-static-position-edition)
and [relative inset edition](../../../../examples/qualify-root-context/README.md#relative-inset-edition)
use the same host and retain exact browser observations alongside the Layout
packet and difference specifications.
Inputs inside an edition are immutable capture witnesses of their handwritten
owners; generated XML/report/ledger bytes belong to Generator. Never copy XML
alone, hand-edit observations, relabel historical captures or reuse an existing
edition directory for a retry.

A partial or failed candidate remains unregistered diagnostic material. All
source/provenance/history/certificate/strict-geometry and browser-free checks must
pass before a separate reviewed caller registration and commit. Machine-local
.surgeist-generator/ state is ignored and is not fixture data.
