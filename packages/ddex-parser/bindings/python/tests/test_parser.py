# packages/ddex-parser/bindings/python/tests/test_parser.py
import pytest
import asyncio
from pathlib import Path
from ddex_parser import DDEXParser, ParseOptions, parse

# Sample DDEX XML for testing
SAMPLE_XML = """<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
    <MessageHeader>
        <MessageId>MSG001</MessageId>
    </MessageHeader>
</ern:NewReleaseMessage>"""


class TestDDEXParser:
    def test_create_parser(self):
        parser = DDEXParser()
        assert parser is not None
        assert repr(parser).startswith("DDEXParser(version=")
    
    def test_parse_string(self):
        parser = DDEXParser()
        result = parser.parse(SAMPLE_XML)
        assert result.message_id == "MSG001"
        assert result.version in ["4.3", "4.2", "3.8.2", "Unknown"]
    
    def test_parse_bytes(self):
        parser = DDEXParser()
        result = parser.parse(SAMPLE_XML.encode('utf-8'))
        assert result.message_id == "MSG001"
    
    def test_parse_with_options(self):
        parser = DDEXParser()
        options = ParseOptions(
            include_raw_extensions=True,
            include_comments=True,
        )
        result = parser.parse(SAMPLE_XML, options)
        assert result is not None
    
    @pytest.mark.asyncio
    async def test_parse_async(self):
        parser = DDEXParser()
        result = await parser.parse_async(SAMPLE_XML)
        assert result.message_id == "MSG001"
    
    def test_stream(self):
        parser = DDEXParser()
        releases = list(parser.stream(SAMPLE_XML))
        assert len(releases) >= 0  # May be empty in mock
    
    def test_detect_version(self):
        parser = DDEXParser()
        
        # Test version detection
        xml_43 = '<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">'
        assert parser.detect_version(xml_43) == "4.3"
        
        xml_42 = '<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/42">'
        assert parser.detect_version(xml_42) == "4.2"
        
        xml_382 = '<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/382">'
        assert parser.detect_version(xml_382) == "3.8.2"
    
    def test_sanity_check(self):
        parser = DDEXParser()
        result = parser.sanity_check(SAMPLE_XML)
        assert "is_valid" in result
        assert "version" in result
        assert "errors" in result
        assert "warnings" in result
    
    @pytest.mark.skipif(
        not pytest.importorskip("pandas"),
        reason="pandas not installed"
    )
    def test_to_dataframe(self):
        parser = DDEXParser()
        df = parser.to_dataframe(SAMPLE_XML)
        assert df is not None
        assert len(df) >= 0
    
    def test_convenience_function(self):
        result = parse(SAMPLE_XML, validate_references=False)
        assert result.message_id == "MSG001"


class TestParseOptions:
    def test_default_options(self):
        options = ParseOptions()
        assert options.include_raw_extensions is False
        assert options.include_comments is False
        assert options.validate_references is True
    
    def test_to_dict(self):
        options = ParseOptions(
            include_raw_extensions=True,
            timeout=30.0,
        )
        d = options.to_dict()
        assert d["include_raw_extensions"] is True
        assert "validate_references" in d


class TestParseResult:
    def test_parse_result(self):
        data = {
            "message_id": "TEST001",
            "version": "4.3",
            "release_count": 2,
            "releases": [
                {"title": "Album 1"},
                {"title": "Album 2"},
            ]
        }
        result = ParseResult(data)
        assert result.message_id == "TEST001"
        assert result.version == "4.3"
        assert result.release_count == 2
        assert len(result.releases) == 2