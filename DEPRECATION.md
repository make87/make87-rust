# Deprecation Notice

## What is deprecated

The `make87` Rust SDK is deprecated. This SDK provided tools and libraries to interact with the make87 platform using the legacy "application SDK integration" workflow.

## What to use instead

make87 has pivoted to a **CLI/agent-based device management** workflow. The legacy SDK integration approach is no longer supported.

## Maintenance stance

**This package is unmaintained and will receive no further updates, bug fixes, or security patches.**

- Status: Deprecated
- Maintenance: None
- Issues/PRs: Not accepted

## Migration notes

If you are currently using this SDK:

1. **Pin your dependency** to the last working version to avoid any breaking changes
2. **Plan migration** off this package as soon as feasible
3. **Do not rely** on this SDK for new projects or features

For information about make87's current device management approach, visit [make87.com](https://www.make87.com).

## Timeline

This is the final release of the `make87` Rust SDK. The GitHub repository will be archived following this release.
