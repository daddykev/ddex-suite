"""Results the Python builder used to make up (round-trip success and score, fidelity
statistics and verification, presets, validate) are now computed, or None where nothing is
measured. Round trips use the official sample examples/Samples43/1 Audio.xml.

Run (with both extensions built, e.g. `maturin develop` here and in
packages/ddex-parser/bindings/python):
    python -m pytest test_honest_results.py
"""

import os

import pytest

from ddex_builder import DdexBuilder, FidelityOptions
from test_smoke_ern43 import REQUEST

# CI runs these tests from a copy outside the repository.
REPO = os.environ.get(
    "GITHUB_WORKSPACE", os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "..", "..")
)
SAMPLE = os.path.join(REPO, "examples", "Samples43", "1 Audio.xml")


def sample():
    with open(SAMPLE, encoding="utf-8") as f:
        return f.read()


def test_round_trip_of_an_official_sample_passes():
    result = DdexBuilder().test_round_trip_fidelity(sample())
    assert result.round_trip_success, result.issues
    assert result.issues == []
    assert result.fidelity_score is None
    assert isinstance(result.determinism_verified, bool)
    assert isinstance(result.checksums_match, bool)


def test_round_trip_reports_a_changed_field_as_a_failure():
    # An empty ISRC in the source: the rebuilt request fails the builder's preflight
    # validation, so the round trip fails and says why (before 0.5.1 it reported success).
    xml = sample().replace("<ISRC>JPTO09404900</ISRC>", "<ISRC></ISRC>", 1)
    assert xml != sample()
    result = DdexBuilder().test_round_trip_fidelity(xml)
    assert not result.round_trip_success
    assert result.issues and result.issues[0].startswith("round trip failed:"), result.issues
    assert result.fidelity_score is None


def test_round_trip_of_non_ddex_input_fails():
    result = DdexBuilder().test_round_trip_fidelity("<html/>")
    assert not result.round_trip_success
    assert result.checksums_match is None


def test_build_with_fidelity_statistics_and_verification_are_computed():
    options = FidelityOptions(collect_statistics=True, enable_verification=True)
    result = DdexBuilder().build_with_fidelity(options, REQUEST)
    stats = result.statistics
    assert stats.element_count == result.xml.count("<") - result.xml.count("</") - result.xml.count("<?")
    assert stats.attribute_count > 0 and stats.namespace_count > 0
    assert stats.memory_used_bytes is None
    assert stats.canonicalization_time_ms is None
    assert stats.extension_count is None
    assert stats.verification_time_ms is not None
    verification = result.verification
    assert verification.round_trip_success, verification.issues
    assert verification.fidelity_score is None

    with pytest.raises(NotImplementedError, match="not implemented in 0.5.x"):
        DdexBuilder().build_with_fidelity(FidelityOptions(canonicalization="c14n"), REQUEST)


def test_presets_are_the_builders():
    builder = DdexBuilder()
    names = builder.get_available_presets()
    assert "audio_album" in names
    assert not any(name.startswith(("spotify", "apple")) for name in names), names
    info = builder.get_preset_info("audio_album")
    assert info.name == "audio_album" and info.required_fields
    builder.apply_preset("audio_album")
    with pytest.raises(ValueError):
        builder.apply_preset("spotify_album")
    assert isinstance(builder.get_preset_validation_rules("audio_album"), list)


def test_validate_runs_preflight_validation():
    builder = DdexBuilder()
    result = builder.validate()
    assert not result.is_valid and result.errors
    assert builder.get_stats().validation_errors == len(result.errors)
