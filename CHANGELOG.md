TBD
===
Unreleased changes. Release notes have not yet been written.

0.3.0
=====
Bug fixes:

* Fix potential panic in clients when the server terminates the connection in
  the middle of a paste or move operation.
* Fix server becoming unresponsive during long-running move operations.

Feature enhancements:

* Exclude --help and --version from tab completion if a subcommand has been
  entered.
* Provide a more informative error message when the server terminates the
  connection.
* Provide visual feedback when typing passwords.
* Support SIGINFO on illumos.

Miscellaneous:

* Return an error when the `connect` or `listen` fields are invalid in the
  config file instead of silently falling back to the default.
* Note in the man page that failure to parse CLI arguments results in exit
  code 2.

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
