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

from importlib.metadata import version as _version

__version__ = _version("ddex-parser")

# The native extension (the Rust parser) is required: there is no pure-Python fallback,
# so a failed load is an ImportError that names the real cause.
try:
    from ._internal import DDEXParser as _DDEXParser, StreamIterator
except ImportError as e:
    raise ImportError(
        f"ddex_parser: the native extension (ddex_parser._internal) failed to load: {e}. "
        "Reinstall a wheel built for this platform and Python version "
        "(pip install --force-reinstall ddex-parser), or, in a source checkout, build it with "
        "`maturin develop` in packages/ddex-parser/bindings/python."
    ) from e

__all__ = ["DDEXParser", "ParseOptions", "ParseResult", "parse", "__version__"]


class ParseOptions:
    """Options for parsing DDEX XML.

    Accepted for forward compatibility; none of them currently changes the result
    (see KNOWN_ISSUES.md):

    - ``include_raw_extensions``, ``include_comments``: passed to the Rust parser, which
      does not read them.
    - ``validate_references``: when True (the default) the Rust parser logs unresolved
      references through ``tracing``, which the Python binding does not surface; the
      parsed message is the same either way.
    - ``streaming``: not read by the binding; ``parse`` always uses the complete parser.
    - ``timeout``: not passed to the Rust parser (``to_dict`` leaves it out).
    """

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
    """Unused: ``DDEXParser.parse`` returns the native ``ParsedERNMessage``, whose
    ``message_id()``, ``version()`` and ``release_count()`` are methods. Kept so existing
    imports keep working."""
    
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
        self._parser = _DDEXParser()
    
    def __repr__(self) -> str:
        return f"DDEXParser(version='{__version__}')"
    
    def parse(self, xml: Union[str, bytes], options: Optional[ParseOptions] = None) -> ParseResult:
        """Parse DDEX XML synchronously."""
        opts = options.to_dict() if options else None
        result = self._parser.parse(xml, opts)
        return result  # Return PyParsedERNMessage directly
    
    async def parse_async(self, xml: Union[str, bytes], options: Optional[ParseOptions] = None) -> ParseResult:
        """Parse DDEX XML asynchronously."""
        opts = options.to_dict() if options else None
        result = await self._parser.parse_async(xml, opts)
        return result  # Return PyParsedERNMessage directly
    
    def stream(self, xml: Union[str, bytes], options: Optional[ParseOptions] = None) -> Iterator[Dict[str, Any]]:
        """Stream parse large DDEX files."""
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
        return self._parser.to_dataframe(xml, schema)
    
    def detect_version(self, xml: Union[str, bytes]) -> str:
        """Detect DDEX version from XML."""
        return self._parser.detect_version(xml)
    
    def sanity_check(self, xml: Union[str, bytes]) -> Dict[str, Any]:
        """Quick check of a document: detects the ERN version and checks that the XML is
        well-formed (syntax, matching end tags, attributes, entity references, one root
        element, no DTD). Not XSD or AVS validation, and ``parse`` can still fail on a
        document that passes.

        Returns ``{"is_valid": bool, "version": str | None, "errors": [str], "warnings":
        [str]}``. ``version`` is the detected version (``"V4_3"``, the spelling
        ``detect_version`` uses) or ``None`` when no ERN namespace was found; ``is_valid``
        is True only when a version was detected and no problem was found."""
        return self._parser.sanity_check(xml)


# Convenience function
def parse(xml: Union[str, bytes], **kwargs) -> ParseResult:
    """Convenience function to parse DDEX XML."""
    parser = DDEXParser()
    options = ParseOptions(**kwargs) if kwargs else None
    return parser.parse(xml, options)


# Import CLI main function
from .cli import main  # noqa: E402
