# surgeist-typed-winsafe

This crate depends on WinSafe 0.0.29 on Windows with its `user` feature and
the required `kernel` feature. WinSafe has no further Cargo dependencies.
Downstream feature unification and resolution can change the dependency set.

- Copyright: Rodrigo Cesar de Freitas Dias, 2019–present.
- License: [MIT](licenses/winsafe/LICENSE.md).
- Upstream: [WinSafe](https://github.com/rodrigocfd/winsafe).

The crate's own license is [MIT](LICENSE). Native ABI definitions follow
Microsoft's documented `LOGFONTW`, `NONCLIENTMETRICSW` and
`SystemParametersInfoForDpi` contracts; no Windows SDK implementation is copied.
