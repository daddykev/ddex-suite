"""End-to-end tests of the installed `ddex-parser` console script on official DDEX samples.

The CLI is run as a subprocess through the entry point that pip installed
([project.scripts] in pyproject.toml), so this covers the packaging as well as cli.py.
The expected MessageId and release count are read from each sample with ElementTree,
independently of the parser.

Run (with the wheel installed, e.g. `maturin build` + `pip install`, or `maturin develop`):
    python -m pytest tests/test_cli.py
"""

import json
import os
import shutil
import subprocess
import sys
import sysconfig
import xml.etree.ElementTree as ET

import pytest

REPO = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", "..", "..", ".."))
SAMPLES = [
    os.path.join(REPO, "examples", "Samples43", "1 Audio.xml"),
    os.path.join(REPO, "examples", "Samples43", "4 SimpleAudioSingle.xml"),
    os.path.join(REPO, "examples", "Samples42", "1 Audio.xml"),
]


def cli_path():
    """The console script pip installed next to this interpreter."""
    name = "ddex-parser.exe" if sys.platform == "win32" else "ddex-parser"
    path = os.path.join(sysconfig.get_path("scripts"), name)
    if os.path.exists(path):
        return path
    found = shutil.which("ddex-parser")
    assert found, "the ddex-parser console script is not installed"
    return found


def run_cli(*args):
    return subprocess.run(
        [cli_path(), *args], capture_output=True, text=True, encoding="utf-8", timeout=120
    )


def local(tag):
    return tag.rsplit("}", 1)[-1]


def expected(sample):
    """MessageId and the number of ReleaseList/Release elements, straight from the XML."""
    root = ET.parse(sample).getroot()
    header = next(el for el in root if local(el.tag) == "MessageHeader")
    message_id = next(el for el in header if local(el.tag) == "MessageId").text or ""
    release_list = next(el for el in root if local(el.tag) == "ReleaseList")
    releases = [el for el in release_list if local(el.tag) == "Release"]
    return message_id, len(releases)


@pytest.mark.parametrize("sample", SAMPLES, ids=os.path.basename)
def test_parse_prints_message_id_and_release_count(sample):
    message_id, release_count = expected(sample)
    proc = run_cli("parse", sample)
    assert proc.returncode == 0, proc.stderr
    data = json.loads(proc.stdout)
    assert data["message_id"] == message_id
    assert data["release_count"] == release_count
    assert len(data["releases"]) == min(release_count, 3)
    assert set(data["releases"][0]) == {"release_id", "title", "artist", "track_count"}
    assert data["version"] in ("V4_2", "V4_3")


@pytest.mark.parametrize("sample", SAMPLES, ids=os.path.basename)
def test_version_and_check(sample):
    with open(sample, encoding="utf-8") as f:
        want = "V4_3" if 'xmlns:ern="http://ddex.net/xml/ern/43"' in f.read() else "V4_2"

    proc = run_cli("version", sample)
    assert proc.returncode == 0, proc.stderr
    assert proc.stdout.strip() == f"DDEX Version: {want}"

    _, release_count = expected(sample)
    proc = run_cli("check", sample)
    assert proc.returncode == 0, proc.stderr
    assert proc.stdout.startswith(f"OK: parsed as ERN {want}, {release_count} release(s).")


def test_check_rejects_a_document_that_does_not_parse(tmp_path):
    bad = tmp_path / "bad.xml"
    bad.write_text(
        "<ern:NewReleaseMessage xmlns:ern='http://ddex.net/xml/ern/43'>"
        "<MessageHeader></Mismatched></ern:NewReleaseMessage>"
    )
    proc = run_cli("check", str(bad))
    assert proc.returncode == 1
    assert proc.stdout == ""
    assert "check failed" in proc.stderr


def test_usage_errors():
    proc = run_cli("--version")
    assert proc.returncode == 0 and proc.stdout.startswith("ddex-parser ")
    proc = run_cli("--help")
    assert proc.returncode == 0 and "Usage: ddex-parser" in proc.stdout
    for args in (["parse"], ["frobnicate", SAMPLES[0]], ["parse", "/nonexistent/file.xml"]):
        proc = run_cli(*args)
        assert proc.returncode == 1, args
        assert proc.stderr.startswith("Error: "), args
