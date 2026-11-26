"""The native DDEXParser.from_dataframe raises NotImplementedError (before 0.5.1 it returned
a mock document with a made-up header). Building from a DataFrame is ddex_builder's job.

Run (with the extension built, e.g. `maturin develop`, plus pandas):
    python -m pytest tests/test_from_dataframe.py
"""

import pandas as pd
import pytest

from ddex_parser._internal import DDEXParser


def test_from_dataframe_is_not_implemented():
    df = pd.DataFrame([{"release_id": "R1", "title": "T"}])
    with pytest.raises(NotImplementedError, match="not implemented in 0.5.x"):
        DDEXParser().from_dataframe(df)
