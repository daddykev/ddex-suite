"""DDEX Parser CLI."""
import sys
import json

def main():
    """CLI entry point."""
    from ddex_parser import DDEXParser, __version__
    
    if len(sys.argv) < 2 or sys.argv[1] in ["--help", "-h"]:
        print(f"DDEX Parser v{__version__}")
        print("\nUsage: ddex-parser <command> <file>")
        print("\nCommands:")
        print("  parse <file>    Parse DDEX XML file")
        print("  version <file>  Detect DDEX version")
        print("  check <file>    Check file validity")
        print("\nOptions:")
        print("  --version       Show version")
        print("  --help, -h      Show this help")
        return
    
    if sys.argv[1] == "--version":
        print(f"ddex-parser {__version__}")
        return
    
    command = sys.argv[1]
    
    if len(sys.argv) < 3:
        print(f"Error: Command '{command}' requires a file argument")
        sys.exit(1)
    
    filepath = sys.argv[2]
    parser = DDEXParser()
    
    try:
        with open(filepath, 'r') as f:
            xml = f.read()
    except FileNotFoundError:
        print(f"Error: File not found: {filepath}")
        sys.exit(1)
    except Exception as e:
        print(f"Error reading file: {e}")
        sys.exit(1)
    
    try:
        if command == "parse":
            result = parser.parse(xml)
            data = {
                'message_id': result.message_id,
                'version': result.version,
                'release_count': result.release_count,
                'releases': result.releases[:3] if result.releases else []  # First 3 releases
            }
            print(json.dumps(data, indent=2))
        elif command == "version":
            v = parser.detect_version(xml)
            print(f"DDEX Version: {v}")
        elif command == "check":
            result = parser.sanity_check(xml)
            if result['is_valid']:
                print(f"✅ Valid DDEX {result['version']}")
            else:
                print(f"❌ Invalid DDEX")
                if result.get('errors'):
                    for error in result['errors']:
                        print(f"  - {error}")
                sys.exit(1)
        else:
            print(f"Error: Unknown command '{command}'")
            print("Run 'ddex-parser --help' for usage")
            sys.exit(1)
    except Exception as e:
        print(f"Error processing file: {e}")
        sys.exit(1)

if __name__ == '__main__':
    main()
