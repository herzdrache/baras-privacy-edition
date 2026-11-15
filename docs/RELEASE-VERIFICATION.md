# Release verification

The publication source is derived from upstream BARAS v2026.11.13 and the local Privacy Edition changes. The Rust dependency refresh resolved all 27 vulnerabilities reported by the initial RustSec scan. The rescan reported zero vulnerabilities and nine warnings (seven unmaintained and two unsound). These counts are specific to the advisory snapshot used.

The roster is runtime data in characters.json under the user's BARAS Privacy Edition data directory. The release sources and configured bundle resources contain no roster snapshot, combat-log copies, or personal configuration. A fresh Privacy Edition profile starts with an empty roster and can populate it by reading that user's logs. An existing installation continues to read its existing profile.

For isolated release validation, BARAS_PRIVACY_TEST_PROFILE accepts an absolute path to a dedicated profile directory. Synthetic raid entries remain debug-only.
