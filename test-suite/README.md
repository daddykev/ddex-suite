# DDEX Parser Test Suite

This directory contains synthetic test files for the DDEX Parser.

## Structure

- `valid/` - Valid DDEX files for each supported version
- `edge-cases/` - Edge cases and real-world quirks
- `nasty/` - Security test cases (XXE, billion laughs, etc.)
- `golden/` - Expected output for regression testing

## License

These synthetic test files are part of the DDEX Parser project and are
licensed under the MIT License. They are NOT real DDEX data and should
not be used for any purpose other than testing this parser.

## Generating Test Files

Run the Python script to regenerate test files:

```bash
python test-suite/generate_test_corpus.py
```
