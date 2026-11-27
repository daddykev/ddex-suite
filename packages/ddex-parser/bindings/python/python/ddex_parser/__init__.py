# packages/ddex-parser/bindings/python/python/ddex_parser/__init__.py
"""
DDEX Parser - High-performance DDEX XML parser for Python
"""

from __future__ import annotations
from typing import Union, Dict, Any, Iterator, IO, TYPE_CHECKING
import asyncio
from pathlib import Path

if TYPE_CHECKING:
    import pandas as pd

from importlib.metadata import version as _version

__version__ = _version("ddex-parser")

# The native extension (the Rust parser) is required: there is no pure-Python fallback,
# so a failed load is an ImportError that names the real cause.
try:
    from ._internal import DDEXParser as _DDEXParser, ParsedERNMessage, StreamIterator
except ImportError as e:
    raise ImportError(
        f"ddex_parser: the native extension (ddex_parser._internal) failed to load: {e}. "
        "Reinstall a wheel built for this platform and Python version "
        "(pip install --force-reinstall ddex-parser), or, in a source checkout, build it with "
        "`maturin develop` in packages/ddex-parser/bindings/python."
    ) from e

__all__ = [
    "DDEXParser",
    "ParsedERNMessage",
    "parse",
    "__version__",
]


class DDEXParser:
    """High-performance DDEX XML parser."""
    
    def __init__(self):
        """Initialize the DDEX parser."""
        self._parser = _DDEXParser()
    
    def __repr__(self) -> str:
        return f"DDEXParser(version='{__version__}')"
    
    def parse(self, xml: Union[str, bytes]) -> ParsedERNMessage:
        """Parse DDEX XML synchronously."""
        result = self._parser.parse(xml)
        return result  # Return PyParsedERNMessage directly
    
    async def parse_async(self, xml: Union[str, bytes]) -> ParsedERNMessage:
        """Parse DDEX XML asynchronously."""
        result = await self._parser.parse_async(xml)
        return result  # Return PyParsedERNMessage directly
    
    def stream(self, xml: Union[str, bytes]) -> Iterator[Dict[str, Any]]:
        """Stream parse large DDEX files."""
        stream_iter = self._parser.stream(xml)
        
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
        return self._parser.to_dataframe(xml, schema)
    
    def detect_version(self, xml: Union[str, bytes]) -> str:
        """Detect DDEX version from XML."""
        return self._parser.detect_version(xml)
    
    def sanity_check(self, xml: Union[str, bytes]) -> Dict[str, Any]:
        """Quick check of a document: detects the ERN version and checks that the XML is
        well-formed (syntax, matching end tags, attributes, entity references, one root
        element, no DTD). Not XSD or AVS validation, and ``parse`` can still fail on a
        document that passes.

        Returns ``{"is_valid": bool, "version": str | None, "errors": [str]}``. ``version`` is
        the detected version (``"V4_3"``, the spelling ``detect_version`` uses) or ``None``
        when no ERN version is recognised; ``is_valid`` is True only when a version was
        detected and no problem was found."""
        return self._parser.sanity_check(xml)


# Convenience function
def parse(xml: Union[str, bytes]) -> ParsedERNMessage:
    """Convenience function to parse DDEX XML."""
    return DDEXParser().parse(xml)


# Import CLI main function
from .cli import main  # noqa: E402
