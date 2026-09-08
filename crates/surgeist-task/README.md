# surgeist-task

Task contracts and Tokio-backed execution for Rust applications integrating
Surgeist. The crate gives task authors typed identity, scope, lifecycle,
cancellation, progress, and output contracts, and gives integrators bounded event
queueing and an executor interface.

Version `0.1.0` implements async and blocking execution. Cancellation is
cooperative; priority, retry, and deduplication settings describe policy without
implementing a scheduler for those settings. App reducers, UI integration, and
mapping task events into app inputs belong to root `surgeist`.

## Start

From a prepared checkout, run the small task example:

```sh
cargo run --offline -p surgeist-task --example basic_task
```

It prints `emitted 2 task events` after a blocking job emits progress and output.
The example calls the job directly. See [Getting started](docs/getting-started.md)
for prerequisites and what it demonstrates.

## Documentation

- [Getting started](docs/getting-started.md): run and understand the example.
- [How-to](docs/how-to.md): execute work, handle cancellation, and drain events.
- [Reference](docs/reference.md): public interfaces, lifecycle, policy, and checks.
- [Explanation](docs/explanation.md): ownership, runtime design, and limitations.

## License and attribution

The project is distributed under the [MIT License](LICENSE). Third-party
attribution and accompanying license material are listed in [NOTICE.md](NOTICE.md).
