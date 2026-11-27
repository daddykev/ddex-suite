"""DDEX Builder - High-performance DDEX XML builder with deterministic output."""

from importlib.metadata import version as _version

__version__ = _version("ddex-builder")

# The native extension (the Rust core builder) is required: there is no pure-Python
# fallback, so a failed load is an ImportError that names the real cause.
try:
    from ddex_builder._internal import (
        DdexBuilder,
        Release,
        Resource,
        ValidationResult,
        BuilderStats,
        BuildResult,
        FidelityOptions,
        VerificationResult,
        batch_build,
        validate_structure,
    )
except ImportError as e:
    raise ImportError(
        f"ddex_builder: the native extension (ddex_builder._internal) failed to load: {e}. "
        "Reinstall a wheel built for this platform and Python version "
        "(pip install --force-reinstall ddex-builder), or, in a source checkout, build it with "
        "`maturin develop` in packages/ddex-builder/bindings/python."
    ) from e

from ddex_builder.build_request import BuildRequest  # noqa: E402  (request types)

# Public API
__all__ = [
    "BuildRequest",
    "__version__",
    "DdexBuilder",
    "Release",
    "Resource",
    "ValidationResult",
    "BuilderStats",
    "BuildResult",
    "FidelityOptions",
    "VerificationResult",
    "batch_build",
    "validate_structure",
]


def is_rust_available():
    """Always True: the package no longer imports without its native extension."""
    return True
