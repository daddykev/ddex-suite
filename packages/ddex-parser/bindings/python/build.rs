fn main() {
    // Without this, linking the cdylib from the workspace root fails on macOS:
    // the per-package .cargo/config.toml with `-undefined dynamic_lookup` is
    // only read when cargo runs inside this directory.
    pyo3_build_config::add_extension_module_link_args();
}
