# Bezy Crates.io Release Checklist

This checklist covers all tasks needed to prepare Bezy for publication on crates.io.

## Pre-Release Code Quality Pass

### Dead Code & Legacy Cleanup
- [ ] Search and remove all TODO comments or convert to GitHub issues
- [ ] Remove commented-out code blocks
- [ ] Delete unused imports and dependencies
- [ ] Remove experimental/WIP features not ready for release
- [ ] Clean up debug print statements (`println!`, `dbg!`, `eprintln!`)
- [ ] Remove test/development assets not needed for release
- [ ] Review and remove duplicate/redundant code

### Code Organization & Refactoring
- [ ] Ensure module structure follows documented architecture (see CLAUDE.md)
- [ ] Verify separation of concerns (editing vs rendering vs tools)
- [ ] Consolidate related functionality into appropriate modules
- [ ] Review public API surface - mark items as `pub(crate)` where appropriate
- [ ] Ensure consistent naming conventions across codebase
- [ ] Remove or properly gate experimental features

### Bug Fixes & Stability
- [ ] Fix known Glyphs.app UFO compatibility issues
- [ ] Resolve Transform vs FontIR synchronization bugs
- [ ] Fix any panics in core workflows
- [ ] Address clippy warnings (run `cargo clippy`)
- [ ] Fix compiler warnings (run `cargo build --release`)
- [ ] Ensure proper error handling (no unwrap() in production paths)
- [ ] Test save/load cycle integrity

### Performance & Optimization
- [ ] Profile and optimize hot paths
- [ ] Review and optimize mesh generation in renderer
- [ ] Ensure efficient change detection patterns
- [ ] Minimize unnecessary allocations
- [ ] Optimize startup time

## Cargo.toml Preparation

### Package Metadata
- [ ] Set appropriate version number (following semver)
- [ ] Add comprehensive package description
- [ ] Set license field (MIT/Apache-2.0 dual license typical for Rust)
- [ ] Add homepage URL
- [ ] Add repository URL
- [ ] Add documentation URL (docs.rs will auto-generate)
- [ ] Add keywords (max 5, e.g., "font", "editor", "ufo", "typography", "bevy")
- [ ] Add categories (e.g., "graphics", "text-editors")
- [ ] Set readme = "README.md"
- [ ] Add authors list with email addresses

### Dependencies Audit
- [ ] Review all dependencies for necessity
- [ ] Update to latest stable versions
- [ ] Check for security advisories (`cargo audit`)
- [ ] Minimize feature flags to reduce compile time
- [ ] Consider moving dev-only deps to dev-dependencies
- [ ] Pin versions appropriately (^ for compatible updates)

### Binary Configuration
- [ ] Set appropriate binary name in [[bin]] section
- [ ] Add binary metadata if needed
- [ ] Consider using `default-run` if multiple binaries

## Documentation

### User Documentation
- [ ] Write comprehensive README.md with:
  - [ ] Clear project description
  - [ ] Installation instructions via cargo
  - [ ] Quick start guide
  - [ ] Feature overview
  - [ ] Screenshots/demo GIFs
  - [ ] System requirements
  - [ ] License information
  - [ ] Contributing guidelines
- [ ] Create CHANGELOG.md with initial release notes
- [ ] Document keyboard shortcuts and tool usage
- [ ] Add examples of common workflows

### Developer Documentation
- [ ] Add rustdoc comments for public API
- [ ] Document module-level purpose and usage
- [ ] Add examples in doc comments where helpful
- [ ] Ensure CLAUDE.md is up-to-date
- [ ] Document build requirements and setup

### Legal & Licensing
- [ ] Add LICENSE file (or LICENSE-MIT and LICENSE-APACHE)
- [ ] Add copyright headers where required
- [ ] Verify all embedded assets have appropriate licenses
- [ ] Document third-party asset attributions

## Testing & Validation

### Automated Testing
- [ ] Add unit tests for core functionality
- [ ] Add integration tests for file operations
- [ ] Test UFO load/save round-trip
- [ ] Test undo/redo system
- [ ] Run full test suite (`cargo test`)
- [ ] Set up CI/CD for automated testing

### Manual Testing
- [ ] Test on fresh system (no dev environment)
- [ ] Test with various UFO files
- [ ] Test all tools and editing operations
- [ ] Verify keyboard shortcuts work correctly
- [ ] Test theme switching
- [ ] Test error cases and recovery
- [ ] Performance test with large fonts

### Platform Testing
- [ ] Test on Linux (primary platform)
- [ ] Test on macOS
- [ ] Test on Windows
- [ ] Document any platform-specific issues
- [ ] Verify asset embedding works on all platforms

## Release Process

### Pre-Publication Checks
- [ ] Run `cargo fmt --check`
- [ ] Run `cargo clippy -- -D warnings`
- [ ] Run `cargo test --release`
- [ ] Run `cargo doc --no-deps` to verify docs build
- [ ] Build release binary (`cargo build --release`)
- [ ] Test release binary manually
- [ ] Check package size (`cargo package --list`)

### Crates.io Publication
- [ ] Create crates.io account and get API token
- [ ] Run `cargo login` with API token
- [ ] Do a dry run: `cargo publish --dry-run`
- [ ] Review package contents: `cargo package --list`
- [ ] Verify version number is correct
- [ ] Tag release in git: `git tag v0.1.0`
- [ ] Publish: `cargo publish`
- [ ] Push git tag: `git push origin v0.1.0`

### Post-Release
- [ ] Create GitHub release with changelog
- [ ] Announce on relevant forums/communities
- [ ] Update project website/documentation
- [ ] Monitor for initial user feedback
- [ ] Set up issue templates on GitHub
- [ ] Plan next version roadmap

## Known Issues to Address

### Critical Issues
- [ ] Glyphs.app UFO anchor format incompatibility
- [ ] Transform vs FontIR synchronization bugs
- [ ] Missing error handling in file operations

### Nice-to-Have Improvements
- [ ] Add more comprehensive theme options
- [ ] Improve startup performance
- [ ] Add font validation features
- [ ] Implement missing tools from TODO list
- [ ] Add preferences/settings persistence
- [ ] Implement proper font compilation via fontc

## Version Strategy

Recommended initial version: **0.1.0**
- 0.x.x indicates pre-1.0 (breaking changes allowed in minor versions)
- Start conservative, iterate based on user feedback
- Plan for 1.0.0 after stabilization and community testing

## Notes

- Consider releasing as alpha/beta initially (0.1.0-alpha.1)
- Monitor crates.io download stats and user issues
- Be responsive to initial user feedback
- Have a clear roadmap for future versions
- Consider setting up a Discord/Matrix channel for community support