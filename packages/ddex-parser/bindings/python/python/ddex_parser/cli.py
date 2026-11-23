"""DDEX Parser CLI (the `ddex-parser` console script)."""
import json
import sys

USAGE = """\
Usage: ddex-parser <command> <file>

Commands:
  parse <file>    Parse a DDEX XML file and print a JSON summary
  version <file>  Detect the ERN version
  check <file>    Check that the file parses (this is not XSD or AVS validation)

Options:
  --version       Show version
  --help, -h      Show this help"""

COMMANDS = ("parse", "version", "check")


def _fail(message):
    print(f"Error: {message}", file=sys.stderr)
    sys.exit(1)


def main():
    """CLI entry point."""
    from ddex_parser import DDEXParser, __version__

    if len(sys.argv) < 2 or sys.argv[1] in ["--help", "-h"]:
        print(f"DDEX Parser v{__version__}\n")
        print(USAGE)
        return

    if sys.argv[1] == "--version":
        print(f"ddex-parser {__version__}")
        return

    command = sys.argv[1]
    if command not in COMMANDS:
        _fail(f"Unknown command '{command}'. Run 'ddex-parser --help' for usage")
    if len(sys.argv) < 3:
        _fail(f"Command '{command}' requires a file argument")
    if len(sys.argv) > 3:
        _fail(f"Command '{command}' takes one file argument, got {len(sys.argv) - 2}")

    filepath = sys.argv[2]
    try:
        # Bytes: the native parser decodes UTF-8 itself, independent of the locale.
        with open(filepath, "rb") as f:
            xml = f.read()
    except FileNotFoundError:
        _fail(f"File not found: {filepath}")
    except OSError as e:
        _fail(f"Cannot read {filepath}: {e}")

    parser = DDEXParser()
    try:
        if command == "parse":
            # ParsedERNMessage exposes these as methods (see KNOWN_ISSUES.md).
            result = parser.parse(xml)
            # It has no release list; stream() yields one summary dict per release.
            releases = []
            for release in parser.stream(xml):
                if len(releases) == 3:  # first 3 releases
                    break
                releases.append(release)
            data = {
                "message_id": result.message_id(),
                "version": result.version(),
                "message_control_type": result.message_control_type(),
                "release_count": result.release_count(),
                "releases": releases,
            }
            print(json.dumps(data, indent=2, ensure_ascii=False))
        elif command == "version":
            print(f"DDEX Version: {parser.detect_version(xml)}")
        elif command == "check":
            # A full parse. (DDEXParser.sanity_check is a placeholder that reports any input
            # as valid ERN 4.3, so it is not used here; see KNOWN_ISSUES.md.)
            result = parser.parse(xml)
            print(
                f"OK: parsed as ERN {result.version()}, "
                f"{result.release_count()} release(s). "
                "Not validated against the XSD or the AVS rules."
            )
    except Exception as e:
        _fail(f"{command} failed for {filepath}: {e}")


if __name__ == "__main__":
    main()
