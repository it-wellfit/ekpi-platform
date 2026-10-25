# ADR-0001: GitHub Issues and reproducible Rust delivery checks

- Status: Accepted
- Date: 2026-10-25

## Context

The repository contains two separate Rust crates, eval and platform; the latter depends on the former. There is no root Cargo workspace. The project targets Rust 1.99.1, has a CSS-first zero-JavaScript web presentation layer, and needs traceable specs, repeatable verification, and a reviewable release path. This checkout was not connected to GitHub when the workflow was configured.

## Decision

Use GitHub Issues as the sole ticket/spec tracker and the Matt Pocock Skills workflow for feature, bug, research, design, and retrospective routing. Keep both Cargo crates independent and pin the tested Rust toolchain in rust-toolchain.toml. Run formatting, Clippy, tests, and existing self-tests on Linux and Windows for every PR; audit both lockfiles and scan newly committed change ranges for secrets. Do not automate production deployment until its target and operating controls are approved.

## Consequences

The GitHub issue/PR workflow is configured in repo docs, but issue publishing, labels, CI execution, and branch protection remain unavailable until the existing GitHub URL is connected and gh is authenticated. CI commands must be run separately in each crate directory so their relative data and schema paths resolve correctly. Updating the Rust toolchain or scanner pins requires review and rerunning the complete baseline.

