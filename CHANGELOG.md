# Changelog

All notable changes to this project will be documented in this file.

This project adheres to [Semantic Versioning](https://semver.org).

Releases may yanked if there is a security bug, a soundness bug, or a regression.

<!--
Note: In this file, do not use the hard wrap in the middle of a sentence for compatibility with GitHub comment style markdown rendering.
-->

## [Unreleased]

## [0.3.2] - 2025-07-14

- Fixes to JSON output

## [0.3.1] - 2023-10-18

- Improve compile time.

## [0.3.0] - 2023-07-16

- Update to syn 2.0. ([#29](https://github.com/taiki-e/syn-serde/pull/29))

  Major changes:
  - Rename/Remove/Add various types/variants/fields, e.g., removal of `box` syntax.
  - Attribute changes related to removal of `NestedMeta`.
  - `"mut": true` -> `"mut": "mut"` in `{,Foreign}ItemStatic` for [restrictions](https://rust-lang.github.io/rfcs/3323-restrictions.html) support.

  See also [syn 2.0.0 release note](https://github.com/dtolnay/syn/releases/tag/2.0.0).

## [0.2.4] - 2023-06-29

- Increase the minimum supported Rust version from Rust 1.31 to Rust 1.56.

- Fix build error from dependency when built with `-Z minimal-versions`.

## [0.2.3] - 2021-04-06

- Apply `doc(cfg(...))` on feature gated APIs. ([#20](https://github.com/taiki-e/syn-serde/pull/20))

## [0.2.2] - 2021-01-05

- Exclude unneeded files from crates.io.

## [0.2.1] - 2020-12-29

- Documentation improvements.

## [0.2.0] - 2019-09-16

- [Removed error from `to_string` / `to_vec`.](https://github.com/taiki-e/syn-serde/commit/e9492636eb7d58565fc415e55fd824b06b37f3d3)

## [0.1.0] - 2019-09-16

Initial release

[Unreleased]: https://github.com/taiki-e/syn-serde/compare/v0.3.1...HEAD
[0.3.1]: https://github.com/taiki-e/syn-serde/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/taiki-e/syn-serde/compare/v0.2.4...v0.3.0
[0.2.4]: https://github.com/taiki-e/syn-serde/compare/v0.2.3...v0.2.4
[0.2.3]: https://github.com/taiki-e/syn-serde/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/taiki-e/syn-serde/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/taiki-e/syn-serde/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/taiki-e/syn-serde/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/taiki-e/syn-serde/releases/tag/v0.1.0
