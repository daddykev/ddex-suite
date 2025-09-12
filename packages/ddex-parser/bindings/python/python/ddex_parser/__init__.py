# packages/ddex-parser/bindings/python/python/ddex_parser/__init__.py
"""
DDEX Parser - High-performance DDEX XML parser for Python
"""

from __future__ import annotations
from typing import Optional, Union, Dict, Any, Iterator, IO, TYPE_CHECKING
import asyncio
from pathlib import Path

if TYPE_CHECKING:
    import pandas as pd

# Import the Rust extension
try:
    from ._internal import DDEXParser as _DDEXParser, StreamIterator, __version__
except ImportError:
    # Fallback for development
    print("Warning: Rust extension not built yet")
    _DDEXParser = None
    StreamIterator = None
    __version__ = "0.1.0"

__all__ = ["DDEXParser", "ParseOptions", "ParseResult", "parse", "__version__"]


class ParseOptions:
    """Options for parsing DDEX XML."""
    
    def __init__(
        self,
        include_raw_extensions: bool = False,
        include_comments: bool = False,
        validate_references: bool = True,
        streaming: bool = False,
        timeout: float = 30.0,
    ):
        self.include_raw_extensions = include_raw_extensions
        self.include_comments = include_comments
        self.validate_references = validate_references
        self.streaming = streaming
        self.timeout = timeout
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary for Rust."""
        return {
            "include_raw_extensions": self.include_raw_extensions,
            "include_comments": self.include_comments,
            "validate_references": self.validate_references,
            "streaming": self.streaming,
        }


class ParseResult:
    """Result of parsing a DDEX message."""
    
    def __init__(self, data: Dict[str, Any]):
        self._data = data
        self.message_id = data.get("message_id", "")
        self.version = data.get("version", "")
        self.release_count = data.get("release_count", 0)
        self.releases = data.get("releases", [])
    
    def __repr__(self) -> str:
        return f"ParseResult(message_id='{self.message_id}', version='{self.version}', releases={self.release_count})"


class DDEXParser:
    """High-performance DDEX XML parser."""
    
    def __init__(self):
        """Initialize the DDEX parser."""
        if _DDEXParser:
            self._parser = _DDEXParser()
        else:
            self._parser = None
    
    def __repr__(self) -> str:
        return f"DDEXParser(version='{__version__}')"
    
    def parse(self, xml: Union[str, bytes], options: Optional[ParseOptions] = None) -> ParseResult:
        """Parse DDEX XML synchronously."""
        if not self._parser:
            # Mock for testing
            return ParseResult({"message_id": "TEST", "version": "4.3", "release_count": 0, "releases": []})
        
        opts = options.to_dict() if options else None
        result = self._parser.parse(xml, opts)
        return result  # Return PyParsedERNMessage directly
    
    async def parse_async(self, xml: Union[str, bytes], options: Optional[ParseOptions] = None) -> ParseResult:
        """Parse DDEX XML asynchronously."""
        if not self._parser:
            # Mock for testing
            await asyncio.sleep(0.01)  # Simulate async work
            return ParseResult({"message_id": "TEST", "version": "4.3", "release_count": 0, "releases": []})
        
        opts = options.to_dict() if options else None
        result = await self._parser.parse_async(xml, opts)
        return result  # Return PyParsedERNMessage directly
    
    def stream(self, xml: Union[str, bytes], options: Optional[ParseOptions] = None) -> Iterator[Dict[str, Any]]:
        """Stream parse large DDEX files."""
        if not self._parser:
            # Mock iterator
            for i in range(3):
                yield {"release_id": f"R{i+1:03}", "title": f"Release {i+1}", "artist": "Test Artist"}
            return
        
        opts = options.to_dict() if options else None
        stream_iter = self._parser.stream(xml, opts)
        
        # Yield from the Rust iterator
        try:
            while True:
                item = next(stream_iter)
                if item is None:
                    break
                yield item
        except StopIteration:
            pass
    
    def to_dataframe(self, xml: Union[str, bytes], schema: str = 'flat') -> 'pd.DataFrame':
        """Convert DDEX XML to pandas DataFrame."""
        if not self._parser:
            # Mock for testing
            try:
                import pandas as pd
                data = [
                    {"release_id": "REL001", "title": "Test Album", "artist": "Test Artist", "track_count": 12}
                ]
                return pd.DataFrame(data)
            except ImportError:
                raise ImportError("pandas is required for to_dataframe(). Install with: pip install pandas")
        
        return self._parser.to_dataframe(xml, schema)
    
    def detect_version(self, xml: Union[str, bytes]) -> str:
        """Detect DDEX version from XML."""
        if not self._parser:
            # Mock detection
            xml_str = xml if isinstance(xml, str) else xml.decode('utf-8')
            if 'ern/43' in xml_str or 'xml/ern/43' in xml_str:
                return '4.3'
            elif 'ern/42' in xml_str or 'xml/ern/42' in xml_str:
                return '4.2'
            elif 'ern/382' in xml_str or 'xml/ern/382' in xml_str:
                return '3.8.2'
            return 'Unknown'
        return self._parser.detect_version(xml)
    
    def sanity_check(self, xml: Union[str, bytes]) -> Dict[str, Any]:
        """Perform sanity check on DDEX XML."""
        if not self._parser:
            return {"is_valid": True, "version": "4.3", "errors": [], "warnings": []}
        return self._parser.sanity_check(xml)


# Convenience function
def parse(xml: Union[str, bytes], **kwargs) -> ParseResult:
    """Convenience function to parse DDEX XML."""
    parser = DDEXParser()
    options = ParseOptions(**kwargs) if kwargs else None
    return parser.parse(xml, options)
# Import CLI main function
from .cli import main

# Import CLI main function
from .cli import main
