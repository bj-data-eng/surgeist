# surgeist::dialog Requirements

`surgeist::dialog` is the native dialog boundary for Surgeist. It owns small,
typed requests for transient host dialogs such as file, folder, and save
pickers, plus a backend contract for tests and later DSL command integration.

## Scope

This module owns:

- File, folder, multi-file, multi-folder, and save-file picker requests.
- Typed dialog options: title, starting directory, suggested file name, and
  extension filters.
- Stable diagnostics for disabled backends and invalid options.
- A system backend over optional native dialog support.
- A fake backend for command, DSL, and app tests.

This module describes host dialog requests. It does not own app windows, modal
lifecycle, layout, rendering, input routing, retained UI state, application
commands, path normalization policy, or filesystem validation.

## Dependencies

Expected direct dependencies:

```text
surgeist::dialog
  -> rfd (optional, behind dialog-system)
```

`surgeist::dialog` must remain independent from render, window lifecycle,
retained state, text, shape, widgets, DSL, Python, and app behavior. Callers may
trigger dialogs from commands or UI events, but this module only performs the
host request and returns selected paths or cancellation.

## Public API

The public front door is intentionally compact:

```rust
pub struct FileDialog;
pub struct Options;
pub struct Filter;
pub trait Backend;
pub struct SystemBackend;
pub struct FakeBackend;
pub enum Call;
pub struct Error;
pub enum ErrorCode;
pub type Result<T>;
```

`FileDialog` is the fluent authoring surface:

```rust
FileDialog::new()
    .title("Open project")
    .directory("/projects")
    .filter("Surgeist", ["surgeist"])
    .open_file()
```

Every terminal operation has a backend-injected variant for tests:

```rust
let mut backend = FakeBackend::new();
let selected = FileDialog::new()
    .filter("Rust", ["rs"])
    .open_file_with(&mut backend)?;
```

## Validation

Options are validated before a backend is invoked.

- Filter names must not be empty after trimming.
- Filters must include at least one extension.
- Filter extensions must not be empty after trimming.
- Dialog cancellation is represented as `Ok(None)`, not an error.
- Disabled system support is represented as `ErrorCode::BackendUnavailable`.
- Invalid options are represented as `ErrorCode::InvalidOptions`.

Path existence, path normalization, permissions, and extension matching are app
or filesystem policy, not dialog policy.

## Tests

Contract tests should prove:

- Fluent options build the expected value.
- Fake backends record calls and return queued results.
- Fake backends default to cancellation when no result is queued.
- Fluent dialogs can execute against caller-supplied backends.
- Invalid filter names and extensions fail before the backend is called.
