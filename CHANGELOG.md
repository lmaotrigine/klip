TBD
===
Unreleased changes. Release notes have not yet been written.

Feature enhancements:

* Tab completion does not include --help and --version if a subcommand has been entered.

Miscellaneous:

* Man page mentions that failure to parse CLI arguments results in exit code 2.

0.2.0
=====
Platform support:

* Since the release process is now automated through GitHub Actions, many more
  platforms are now included in the release binaries for klip.

Bug fixes:

* Fix move operations deleting clipboard content despite I/O failure
  [#1](https://github.com/lmaotrigine/klip/pull/1)
* Fix arbitrary timestamps bypassing TTL enforcement
  [#2](https://github.com/lmaotrigine/klip/pull/2)
* Fix build failure when git commit hash cannot be determined.

Miscellaneous:

* Add workflows for CI and release automation.
* Remove enforcement of `lld` for linking.
* Use the RustCrypto ecosystem for cryptography implementations.
