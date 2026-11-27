# Determinism

The builder reads no clock or random source and uses no unordered iteration on the build
path (enforced by clippy lints). The same request and options produce byte-identical
output with the same crate version; repeated builds are tested to be byte-identical.
