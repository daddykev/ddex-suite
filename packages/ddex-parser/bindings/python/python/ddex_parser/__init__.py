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
    from ._internal import DDEXParser as _DDEXParser
except ImportError:
    # Fallback for development
    print("Warning: Rust extension not built yet")
    _DDEXParser = None

__version__ = "0.1.0"
__all__ = ["DDEXParser", "ParseOptions", "ParseResult", "__version__"]


class ParseOptions:
    """Options for parsing DDEX XML."""
    
    def __init__(
        self,
        include_raw_extensions: bool = False,
        include_comments: bool = False,
        validate_references: bool = True,
        streaming: bool = False,
    ):
        self.include_raw_extensions = include_raw_extensions
        self.include_comments = include_comments
        self.validate_references = validate_references
        self.streaming = streaming
    
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


class DDEXParser:
    """High-performance DDEX XML parser."""
    
    def __init__(self):
        """Initialize the DDEX parser."""
        if _DDEXParser:
            self._parser = _DDEXParser()
        else:
            self._parser = None
    
    def parse(self, xml: Union[str, bytes], options: Optional[ParseOptions] = None) -> ParseResult:
        """Parse DDEX XML synchronously."""
        if not self._parser:
            # Mock for testing
            return ParseResult({"message_id": "TEST", "version": "4.3", "release_count": 0, "releases": []})
        
        opts = options.to_dict() if options else None
        result = self._parser.parse(xml, opts)
        return ParseResult(result)
    
    def detect_version(self, xml: Union[str, bytes]) -> str:
        """Detect DDEX version from XML."""
        if not self._parser:
            return "4.3"
        return self._parser.detect_version(xml)
    
    def sanity_check(self, xml: Union[str, bytes]) -> Dict[str, Any]:
        """Perform sanity check on DDEX XML."""
        if not self._parser:
            return {"is_valid": True, "version": "4.3", "errors": [], "warnings": []}
        return self._parser.sanity_check(xml)