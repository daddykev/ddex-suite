"""Import/API-surface checks for the ddex_builder Python bindings (run with pytest)."""

import ddex_builder


def test_class_creation():
    """The main classes can be constructed with their required arguments."""
    assert ddex_builder.DdexBuilder() is not None
    release = ddex_builder.Release(
        release_id="TEST001",
        release_type="Album",
        title="Test Album",
        artist="Test Artist",
    )
    assert release is not None
    resource = ddex_builder.Resource(
        resource_id="RES001",
        resource_type="SoundRecording",
        title="Test Track",
        artist="Test Artist",
    )
    assert resource is not None


def test_available_methods():
    """DdexBuilder exposes the documented methods."""
    builder = ddex_builder.DdexBuilder()
    expected = {"add_release", "add_resource", "apply_preset", "build", "get_available_presets"}
    available = {name for name in dir(builder) if not name.startswith("_")}
    assert expected <= available, f"missing methods: {sorted(expected - available)}"


def test_module_attributes():
    """The module exports the documented classes."""
    expected = {"DdexBuilder", "Release", "Resource", "ValidationResult"}
    available = set(dir(ddex_builder))
    assert expected <= available, f"missing attributes: {sorted(expected - available)}"
