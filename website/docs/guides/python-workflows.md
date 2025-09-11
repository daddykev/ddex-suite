# Python Workflows Guide

Comprehensive guide to data analysis workflows using the DDEX Suite's Python bindings with pandas, polars, and modern data science tools.

## Problem Statement

Music industry data analysis often involves:

- **Large DDEX catalogs** with thousands of releases and complex metadata
- **Cross-platform analytics** requiring data transformation and aggregation
- **Business intelligence** needs like royalty calculations and market analysis
- **Data quality assessment** and anomaly detection across metadata
- **Performance optimization** for real-time analytics and reporting
- **Integration with existing** Python data science ecosystems

Traditional XML processing tools are inadequate for these analytical workflows, requiring custom parsing, complex data transformations, and manual schema handling.

## Solution Approach

The DDEX Suite's Python bindings provide native integration with pandas and polars, enabling:

1. **Direct DataFrame conversion** from DDEX XML
2. **High-performance analytics** with vectorized operations
3. **Seamless integration** with existing data science workflows
4. **Memory-efficient processing** for large catalogs
5. **Advanced querying** and aggregation capabilities

## Basic Python Integration

### Installation and Setup

```bash
# Install from PyPI
pip install ddex-parser ddex-builder

# With optional analytics dependencies
pip install ddex-parser[analytics]  # Includes pandas, polars, matplotlib

# Development installation
pip install ddex-parser[dev]  # Includes testing and development tools
```

### Basic DataFrame Conversion

```python
import pandas as pd
from ddex_parser import DDEXParser
import asyncio

async def basic_dataframe_example():
    """Convert DDEX XML to pandas DataFrame"""
    
    # Initialize parser
    parser = DDEXParser()
    
    # Load DDEX XML file
    with open('catalog.xml', 'r') as f:
        xml_content = f.read()
    
    # Convert to DataFrame
    df = await parser.to_dataframe(xml_content)
    
    # Basic DataFrame operations
    print(f"Total releases: {len(df)}")
    print(f"Columns: {list(df.columns)}")
    print(df.head())
    
    return df

# Run async function
df = asyncio.run(basic_dataframe_example())
```

### Advanced Parser Configuration

```python
from ddex_parser import DDEXParser, ParserConfig
from typing import Dict, Any, List

class AnalyticsParser:
    def __init__(self, config: Dict[str, Any] = None):
        self.config = config or {}
        self.parser = DDEXParser()
    
    async def parse_for_analytics(self, xml_content: str) -> pd.DataFrame:
        """Parse DDEX with analytics-optimized configuration"""
        
        # Configure for analytics
        result = await self.parser.parse(xml_content, {
            'flatten_nested_objects': True,    # Easier DataFrame operations
            'include_metadata_stats': True,    # Add computed columns
            'normalize_identifiers': True,     # Standardize ID formats
            'extract_dates': True,             # Parse all date fields
            'compute_derived_fields': True     # Add calculated columns
        })
        
        # Convert to DataFrame with analytics columns
        df = await self.parser.to_dataframe(xml_content, {
            'include_analytics_columns': True,
            'add_derived_metrics': True,
            'normalize_currencies': True,
            'standardize_territories': True
        })
        
        return self._add_computed_columns(df)
    
    def _add_computed_columns(self, df: pd.DataFrame) -> pd.DataFrame:
        """Add computed columns for analytics"""
        
        # Duration in seconds
        if 'duration' in df.columns:
            df['duration_seconds'] = pd.to_timedelta(df['duration']).dt.total_seconds()
        
        # Release age
        if 'release_date' in df.columns:
            df['release_date'] = pd.to_datetime(df['release_date'])
            df['days_since_release'] = (pd.Timestamp.now() - df['release_date']).dt.days
        
        # Genre hierarchies
        if 'genre' in df.columns:
            df['primary_genre'] = df['genre'].str.split('/').str[0]
            df['sub_genre'] = df['genre'].str.split('/').str[1]
        
        # Territory groupings
        if 'territory' in df.columns:
            df['region'] = self._map_territories_to_regions(df['territory'])
        
        return df
    
    def _map_territories_to_regions(self, territories: pd.Series) -> pd.Series:
        """Map territory codes to regions"""
        region_mapping = {
            'US': 'North America',
            'CA': 'North America',
            'GB': 'Europe',
            'DE': 'Europe',
            'FR': 'Europe',
            'JP': 'Asia',
            'AU': 'Oceania',
            # Add more mappings
        }
        
        return territories.map(region_mapping).fillna('Other')
```

## Advanced Analytics Workflows

### Catalog Analysis and Reporting

```python
import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns
from typing import Tuple, List, Dict

class CatalogAnalyzer:
    def __init__(self, df: pd.DataFrame):
        self.df = df
        self.setup_analytics()
    
    def setup_analytics(self):
        """Prepare DataFrame for analytics"""
        # Ensure numeric columns
        numeric_columns = ['duration_seconds', 'track_count', 'days_since_release']
        for col in numeric_columns:
            if col in self.df.columns:
                self.df[col] = pd.to_numeric(self.df[col], errors='coerce')
        
        # Setup categorical data
        categorical_columns = ['genre', 'label', 'territory', 'format']
        for col in categorical_columns:
            if col in self.df.columns:
                self.df[col] = self.df[col].astype('category')
    
    def generate_catalog_summary(self) -> Dict[str, Any]:
        """Generate comprehensive catalog summary"""
        
        summary = {
            'overview': {
                'total_releases': len(self.df),
                'unique_artists': self.df['primary_artist'].nunique(),
                'unique_labels': self.df['label'].nunique(),
                'date_range': {
                    'earliest': self.df['release_date'].min(),
                    'latest': self.df['release_date'].max()
                },
                'total_duration_hours': self.df['duration_seconds'].sum() / 3600
            },
            'distributions': {
                'releases_by_year': self._releases_by_year(),
                'genre_distribution': self._genre_distribution(),
                'duration_statistics': self._duration_statistics(),
                'territory_coverage': self._territory_coverage()
            },
            'quality_metrics': {
                'completeness_score': self._calculate_completeness(),
                'duplicate_detection': self._detect_duplicates(),
                'anomaly_count': self._detect_anomalies()
            }
        }
        
        return summary
    
    def _releases_by_year(self) -> pd.Series:
        """Analyze release distribution by year"""
        return self.df.groupby(self.df['release_date'].dt.year).size()
    
    def _genre_distribution(self) -> pd.Series:
        """Analyze genre distribution"""
        return self.df['primary_genre'].value_counts()
    
    def _duration_statistics(self) -> Dict[str, float]:
        """Calculate duration statistics"""
        durations = self.df['duration_seconds'].dropna()
        return {
            'mean': durations.mean(),
            'median': durations.median(),
            'std': durations.std(),
            'min': durations.min(),
            'max': durations.max()
        }
    
    def _territory_coverage(self) -> pd.Series:
        """Analyze territory coverage"""
        return self.df['region'].value_counts()
    
    def _calculate_completeness(self) -> float:
        """Calculate metadata completeness score"""
        required_fields = ['title', 'primary_artist', 'release_date', 'genre']
        optional_fields = ['label', 'catalog_number', 'upc', 'isrc']
        
        # Required field completeness (weighted higher)
        required_complete = self.df[required_fields].notna().all(axis=1).mean() * 0.7
        
        # Optional field completeness
        optional_complete = self.df[optional_fields].notna().mean().mean() * 0.3
        
        return required_complete + optional_complete
    
    def _detect_duplicates(self) -> Dict[str, int]:
        """Detect potential duplicates"""
        return {
            'exact_title_matches': self.df.duplicated(subset=['title', 'primary_artist']).sum(),
            'similar_titles': self._find_similar_titles(),
            'duplicate_isrc': self.df.duplicated(subset=['isrc']).sum(),
            'duplicate_upc': self.df.duplicated(subset=['upc']).sum()
        }
    
    def _find_similar_titles(self) -> int:
        """Find titles that might be duplicates"""
        from difflib import SequenceMatcher
        
        titles = self.df['title'].dropna().unique()
        similar_count = 0
        
        for i, title1 in enumerate(titles):
            for title2 in titles[i+1:]:
                similarity = SequenceMatcher(None, title1.lower(), title2.lower()).ratio()
                if 0.8 <= similarity < 1.0:  # Very similar but not exact
                    similar_count += 1
        
        return similar_count
    
    def _detect_anomalies(self) -> Dict[str, int]:
        """Detect data anomalies"""
        anomalies = {}
        
        # Duration anomalies
        if 'duration_seconds' in self.df.columns:
            duration_q1 = self.df['duration_seconds'].quantile(0.25)
            duration_q3 = self.df['duration_seconds'].quantile(0.75)
            duration_iqr = duration_q3 - duration_q1
            duration_outliers = (
                (self.df['duration_seconds'] < duration_q1 - 1.5 * duration_iqr) |
                (self.df['duration_seconds'] > duration_q3 + 1.5 * duration_iqr)
            ).sum()
            anomalies['duration_outliers'] = duration_outliers
        
        # Future release dates
        future_releases = (self.df['release_date'] > pd.Timestamp.now()).sum()
        anomalies['future_releases'] = future_releases
        
        # Missing critical identifiers
        missing_isrc = self.df['isrc'].isna().sum()
        anomalies['missing_isrc'] = missing_isrc
        
        return anomalies
    
    def create_visualizations(self, output_dir: str = './analytics'):
        """Create comprehensive visualizations"""
        import os
        os.makedirs(output_dir, exist_ok=True)
        
        # Set style
        plt.style.use('default')
        sns.set_palette("husl")
        
        # 1. Release timeline
        fig, ax = plt.subplots(figsize=(12, 6))
        releases_by_month = self.df.groupby(self.df['release_date'].dt.to_period('M')).size()
        releases_by_month.plot(kind='line', ax=ax)
        ax.set_title('Release Timeline')
        ax.set_xlabel('Date')
        ax.set_ylabel('Number of Releases')
        plt.xticks(rotation=45)
        plt.tight_layout()
        plt.savefig(f'{output_dir}/release_timeline.png', dpi=300, bbox_inches='tight')
        plt.close()
        
        # 2. Genre distribution
        fig, ax = plt.subplots(figsize=(10, 8))
        genre_counts = self.df['primary_genre'].value_counts().head(15)
        genre_counts.plot(kind='barh', ax=ax)
        ax.set_title('Top 15 Genres by Release Count')
        ax.set_xlabel('Number of Releases')
        plt.tight_layout()
        plt.savefig(f'{output_dir}/genre_distribution.png', dpi=300, bbox_inches='tight')
        plt.close()
        
        # 3. Duration distribution
        fig, ax = plt.subplots(figsize=(10, 6))
        self.df['duration_seconds'].hist(bins=50, ax=ax, alpha=0.7)
        ax.axvline(self.df['duration_seconds'].median(), color='red', linestyle='--', label='Median')
        ax.set_title('Track Duration Distribution')
        ax.set_xlabel('Duration (seconds)')
        ax.set_ylabel('Frequency')
        ax.legend()
        plt.tight_layout()
        plt.savefig(f'{output_dir}/duration_distribution.png', dpi=300, bbox_inches='tight')
        plt.close()
        
        # 4. Territory heatmap
        if 'territory' in self.df.columns and 'release_date' in self.df.columns:
            fig, ax = plt.subplots(figsize=(12, 8))
            territory_by_year = pd.crosstab(
                self.df['release_date'].dt.year,
                self.df['territory']
            )
            sns.heatmap(territory_by_year, annot=False, cmap='YlOrRd', ax=ax)
            ax.set_title('Release Activity by Territory and Year')
            plt.tight_layout()
            plt.savefig(f'{output_dir}/territory_heatmap.png', dpi=300, bbox_inches='tight')
            plt.close()
        
        print(f"Visualizations saved to {output_dir}/")
```

### High-Performance Analytics with Polars

```python
import polars as pl
from ddex_parser import DDEXParser
from typing import List, Dict, Any

class PolarsAnalyzer:
    """High-performance analytics using Polars for large datasets"""
    
    def __init__(self):
        self.parser = DDEXParser()
    
    async def parse_to_polars(self, xml_content: str) -> pl.DataFrame:
        """Convert DDEX to Polars DataFrame for high-performance analytics"""
        
        # Parse with Polars optimization
        pandas_df = await self.parser.to_dataframe(xml_content, {
            'optimize_for_polars': True,
            'memory_efficient': True
        })
        
        # Convert to Polars with optimal dtypes
        polars_df = pl.from_pandas(pandas_df)
        
        return self._optimize_schema(polars_df)
    
    def _optimize_schema(self, df: pl.DataFrame) -> pl.DataFrame:
        """Optimize Polars DataFrame schema for performance"""
        
        optimizations = []
        
        # String columns to categorical
        string_cols = [col for col, dtype in zip(df.columns, df.dtypes) if dtype == pl.Utf8]
        for col in string_cols:
            unique_ratio = df[col].n_unique() / df.height
            if unique_ratio < 0.5:  # Less than 50% unique values
                optimizations.append(pl.col(col).cast(pl.Categorical))
        
        # Optimize numeric columns
        for col, dtype in zip(df.columns, df.dtypes):
            if dtype == pl.Float64:
                # Check if can be float32
                if df[col].max() <= 3.4e38 and df[col].min() >= -3.4e38:
                    optimizations.append(pl.col(col).cast(pl.Float32))
            elif dtype == pl.Int64:
                # Check if can be smaller int type
                col_max = df[col].max()
                col_min = df[col].min()
                if col_max <= 32767 and col_min >= -32768:
                    optimizations.append(pl.col(col).cast(pl.Int16))
                elif col_max <= 2147483647 and col_min >= -2147483648:
                    optimizations.append(pl.col(col).cast(pl.Int32))
        
        if optimizations:
            df = df.with_columns(optimizations)
        
        return df
    
    def analyze_large_catalog(self, df: pl.DataFrame) -> Dict[str, Any]:
        """Perform high-performance analytics on large catalogs"""
        
        # Lazy evaluation for memory efficiency
        lazy_df = df.lazy()
        
        # Complex aggregations
        analytics = {
            'summary_stats': self._compute_summary_stats(lazy_df),
            'time_series_analysis': self._time_series_analysis(lazy_df),
            'genre_analytics': self._genre_analytics(lazy_df),
            'performance_metrics': self._performance_metrics(lazy_df)
        }
        
        return analytics
    
    def _compute_summary_stats(self, lazy_df: pl.LazyFrame) -> Dict[str, Any]:
        """Compute summary statistics efficiently"""
        
        stats = lazy_df.select([
            pl.count().alias('total_releases'),
            pl.col('primary_artist').n_unique().alias('unique_artists'),
            pl.col('label').n_unique().alias('unique_labels'),
            pl.col('duration_seconds').mean().alias('avg_duration'),
            pl.col('duration_seconds').median().alias('median_duration'),
            pl.col('release_date').min().alias('earliest_release'),
            pl.col('release_date').max().alias('latest_release'),
            pl.col('duration_seconds').sum().alias('total_duration_seconds')
        ]).collect()
        
        return stats.to_dicts()[0]
    
    def _time_series_analysis(self, lazy_df: pl.LazyFrame) -> pl.DataFrame:
        """Perform time series analysis"""
        
        return lazy_df.group_by_dynamic(
            'release_date',
            every='1mo',
            period='1mo'
        ).agg([
            pl.count().alias('release_count'),
            pl.col('duration_seconds').mean().alias('avg_duration'),
            pl.col('primary_genre').mode().first().alias('dominant_genre')
        ]).sort('release_date').collect()
    
    def _genre_analytics(self, lazy_df: pl.LazyFrame) -> pl.DataFrame:
        """Advanced genre analytics"""
        
        return lazy_df.group_by('primary_genre').agg([
            pl.count().alias('release_count'),
            pl.col('duration_seconds').mean().alias('avg_duration'),
            pl.col('duration_seconds').std().alias('duration_std'),
            pl.col('primary_artist').n_unique().alias('unique_artists'),
            pl.col('release_date').min().alias('first_release'),
            pl.col('release_date').max().alias('latest_release')
        ]).sort('release_count', descending=True).collect()
    
    def _performance_metrics(self, lazy_df: pl.LazyFrame) -> Dict[str, Any]:
        """Calculate performance and quality metrics"""
        
        # Data quality metrics
        completeness = lazy_df.select([
            (pl.col('title').is_not_null().mean() * 100).alias('title_completeness'),
            (pl.col('primary_artist').is_not_null().mean() * 100).alias('artist_completeness'),
            (pl.col('isrc').is_not_null().mean() * 100).alias('isrc_completeness'),
            (pl.col('upc').is_not_null().mean() * 100).alias('upc_completeness')
        ]).collect()
        
        # Duplicate detection
        duplicates = lazy_df.select([
            pl.col('title').is_duplicated().sum().alias('duplicate_titles'),
            pl.col('isrc').is_duplicated().sum().alias('duplicate_isrcs'),
            pl.col('upc').is_duplicated().sum().alias('duplicate_upcs')
        ]).collect()
        
        return {
            'completeness': completeness.to_dicts()[0],
            'duplicates': duplicates.to_dicts()[0]
        }

# Batch processing for multiple files
async def process_catalog_batch(file_paths: List[str]) -> pl.DataFrame:
    """Process multiple DDEX files efficiently"""
    
    analyzer = PolarsAnalyzer()
    dataframes = []
    
    for file_path in file_paths:
        with open(file_path, 'r') as f:
            xml_content = f.read()
        
        df = await analyzer.parse_to_polars(xml_content)
        df = df.with_columns(pl.lit(file_path).alias('source_file'))
        dataframes.append(df)
    
    # Concatenate all DataFrames
    combined_df = pl.concat(dataframes, how='vertical')
    
    return combined_df
```

### Advanced Querying and Filtering

```python
from datetime import datetime, timedelta
import re

class DDEXQueryEngine:
    """Advanced querying capabilities for DDEX data"""
    
    def __init__(self, df: pl.DataFrame):
        self.df = df
    
    def query_releases_by_criteria(
        self,
        genres: List[str] = None,
        artists: List[str] = None,
        date_range: Tuple[str, str] = None,
        territories: List[str] = None,
        min_duration: int = None,
        max_duration: int = None,
        labels: List[str] = None
    ) -> pl.DataFrame:
        """Complex multi-criteria querying"""
        
        lazy_df = self.df.lazy()
        
        # Apply filters
        if genres:
            lazy_df = lazy_df.filter(pl.col('primary_genre').is_in(genres))
        
        if artists:
            # Support partial matching for artists
            artist_pattern = '|'.join(re.escape(artist) for artist in artists)
            lazy_df = lazy_df.filter(
                pl.col('primary_artist').str.contains(artist_pattern, literal=False)
            )
        
        if date_range:
            start_date, end_date = date_range
            lazy_df = lazy_df.filter(
                pl.col('release_date').is_between(
                    pl.lit(start_date).str.strptime(pl.Date, '%Y-%m-%d'),
                    pl.lit(end_date).str.strptime(pl.Date, '%Y-%m-%d')
                )
            )
        
        if territories:
            lazy_df = lazy_df.filter(pl.col('territory').is_in(territories))
        
        if min_duration:
            lazy_df = lazy_df.filter(pl.col('duration_seconds') >= min_duration)
        
        if max_duration:
            lazy_df = lazy_df.filter(pl.col('duration_seconds') <= max_duration)
        
        if labels:
            label_pattern = '|'.join(re.escape(label) for label in labels)
            lazy_df = lazy_df.filter(
                pl.col('label').str.contains(label_pattern, literal=False)
            )
        
        return lazy_df.collect()
    
    def find_similar_releases(
        self,
        reference_release_id: str,
        similarity_threshold: float = 0.8
    ) -> pl.DataFrame:
        """Find releases similar to a reference release"""
        
        # Get reference release
        reference = self.df.filter(pl.col('release_id') == reference_release_id)
        if reference.height == 0:
            raise ValueError(f"Release {reference_release_id} not found")
        
        ref_data = reference.to_dicts()[0]
        
        # Calculate similarity scores
        similar_df = self.df.with_columns([
            # Genre similarity (exact match)
            (pl.col('primary_genre') == ref_data['primary_genre']).cast(pl.Float32).alias('genre_match'),
            
            # Duration similarity (normalized difference)
            (1 - (pl.col('duration_seconds') - ref_data['duration_seconds']).abs() 
             / pl.max_horizontal([pl.col('duration_seconds'), pl.lit(ref_data['duration_seconds'])])).alias('duration_similarity'),
            
            # Artist similarity (fuzzy matching would require custom function)
            (pl.col('primary_artist') == ref_data['primary_artist']).cast(pl.Float32).alias('artist_match'),
            
            # Label similarity
            (pl.col('label') == ref_data['label']).cast(pl.Float32).alias('label_match')
        ]).with_columns([
            # Composite similarity score
            (pl.col('genre_match') * 0.3 + 
             pl.col('duration_similarity') * 0.2 + 
             pl.col('artist_match') * 0.3 + 
             pl.col('label_match') * 0.2).alias('similarity_score')
        ]).filter(
            pl.col('similarity_score') >= similarity_threshold
        ).filter(
            pl.col('release_id') != reference_release_id  # Exclude reference itself
        ).sort('similarity_score', descending=True)
        
        return similar_df
    
    def analyze_trends(self, groupby_column: str, time_window: str = '1mo') -> pl.DataFrame:
        """Analyze trends over time for any categorical column"""
        
        return self.df.group_by_dynamic(
            'release_date',
            every=time_window,
            period=time_window
        ).agg([
            pl.count().alias('total_releases'),
            pl.col(groupby_column).mode().first().alias(f'top_{groupby_column}'),
            pl.col(groupby_column).n_unique().alias(f'unique_{groupby_column}')
        ]).sort('release_date')
    
    def calculate_market_share(self, group_by: str = 'label') -> pl.DataFrame:
        """Calculate market share by label, artist, or genre"""
        
        total_releases = self.df.height
        
        return self.df.group_by(group_by).agg([
            pl.count().alias('release_count'),
            pl.col('duration_seconds').sum().alias('total_duration'),
            pl.col('territory').n_unique().alias('territory_coverage')
        ]).with_columns([
            (pl.col('release_count') / total_releases * 100).alias('market_share_percent'),
            (pl.col('total_duration') / 3600).alias('total_hours')
        ]).sort('market_share_percent', descending=True)
```

## Integration with Data Science Ecosystem

### Jupyter Notebook Integration

```python
# jupyter_ddex_utils.py
import pandas as pd
import plotly.express as px
import plotly.graph_objects as go
from plotly.subplots import make_subplots
from IPython.display import display, HTML
import ipywidgets as widgets
from ddex_parser import DDEXParser

class JupyterDDEXAnalyzer:
    """Interactive DDEX analysis for Jupyter notebooks"""
    
    def __init__(self, df: pd.DataFrame):
        self.df = df
        self.setup_widgets()
    
    def setup_widgets(self):
        """Create interactive widgets for analysis"""
        
        # Filter widgets
        self.genre_filter = widgets.SelectMultiple(
            options=sorted(self.df['primary_genre'].unique()),
            description='Genres:',
            disabled=False
        )
        
        self.date_range = widgets.SelectionRangeSlider(
            options=sorted(self.df['release_date'].dt.year.unique()),
            index=(0, len(self.df['release_date'].dt.year.unique()) - 1),
            description='Year Range',
            disabled=False
        )
        
        self.artist_search = widgets.Text(
            placeholder='Search artists...',
            description='Artist:',
            disabled=False
        )
        
        # Create interactive dashboard
        self.create_dashboard()
    
    def create_dashboard(self):
        """Create interactive dashboard"""
        
        # Combine widgets
        filter_box = widgets.VBox([
            self.genre_filter,
            self.date_range,
            self.artist_search
        ])
        
        # Create output widget for plots
        self.output = widgets.Output()
        
        # Update function
        def update_plots(*args):
            with self.output:
                self.output.clear_output(wait=True)
                filtered_df = self.apply_filters()
                self.create_interactive_plots(filtered_df)
        
        # Connect widgets to update function
        self.genre_filter.observe(update_plots, names='value')
        self.date_range.observe(update_plots, names='value')
        self.artist_search.observe(update_plots, names='value')
        
        # Display dashboard
        dashboard = widgets.HBox([filter_box, self.output])
        display(dashboard)
        
        # Initial plot
        update_plots()
    
    def apply_filters(self) -> pd.DataFrame:
        """Apply current filter settings"""
        
        filtered_df = self.df.copy()
        
        # Genre filter
        if self.genre_filter.value:
            filtered_df = filtered_df[
                filtered_df['primary_genre'].isin(self.genre_filter.value)
            ]
        
        # Date range filter
        if self.date_range.value:
            start_year, end_year = self.date_range.value
            filtered_df = filtered_df[
                (filtered_df['release_date'].dt.year >= start_year) &
                (filtered_df['release_date'].dt.year <= end_year)
            ]
        
        # Artist search filter
        if self.artist_search.value:
            search_term = self.artist_search.value.lower()
            filtered_df = filtered_df[
                filtered_df['primary_artist'].str.lower().str.contains(search_term, na=False)
            ]
        
        return filtered_df
    
    def create_interactive_plots(self, df: pd.DataFrame):
        """Create interactive plots with Plotly"""
        
        # Create subplots
        fig = make_subplots(
            rows=2, cols=2,
            subplot_titles=(
                'Release Timeline',
                'Genre Distribution',
                'Duration Distribution',
                'Territory Coverage'
            ),
            specs=[[{'secondary_y': False}, {'type': 'xy'}],
                   [{'type': 'xy'}, {'type': 'xy'}]]
        )
        
        # 1. Release timeline
        timeline_data = df.groupby(df['release_date'].dt.to_period('M')).size()
        fig.add_trace(
            go.Scatter(
                x=timeline_data.index.astype(str),
                y=timeline_data.values,
                mode='lines+markers',
                name='Releases'
            ),
            row=1, col=1
        )
        
        # 2. Genre distribution
        genre_counts = df['primary_genre'].value_counts().head(10)
        fig.add_trace(
            go.Bar(
                x=genre_counts.values,
                y=genre_counts.index,
                orientation='h',
                name='Genres'
            ),
            row=1, col=2
        )
        
        # 3. Duration distribution
        fig.add_trace(
            go.Histogram(
                x=df['duration_seconds'],
                nbinsx=30,
                name='Duration'
            ),
            row=2, col=1
        )
        
        # 4. Territory coverage
        territory_counts = df['territory'].value_counts().head(10)
        fig.add_trace(
            go.Bar(
                x=territory_counts.index,
                y=territory_counts.values,
                name='Territories'
            ),
            row=2, col=2
        )
        
        # Update layout
        fig.update_layout(
            height=800,
            showlegend=False,
            title_text=f"DDEX Catalog Analysis ({len(df)} releases)"
        )
        
        fig.show()
        
        # Display summary statistics
        summary_html = f"""
        <div style="background-color: #f0f0f0; padding: 15px; margin: 10px 0; border-radius: 5px;">
            <h3>Summary Statistics</h3>
            <ul>
                <li><strong>Total Releases:</strong> {len(df)}</li>
                <li><strong>Unique Artists:</strong> {df['primary_artist'].nunique()}</li>
                <li><strong>Unique Labels:</strong> {df['label'].nunique()}</li>
                <li><strong>Average Duration:</strong> {df['duration_seconds'].mean() / 60:.1f} minutes</li>
                <li><strong>Date Range:</strong> {df['release_date'].min().strftime('%Y-%m-%d')} to {df['release_date'].max().strftime('%Y-%m-%d')}</li>
            </ul>
        </div>
        """
        display(HTML(summary_html))

# Usage in Jupyter notebook
async def load_and_analyze_in_jupyter(xml_file_path: str):
    """Load DDEX file and create interactive analysis"""
    
    # Load and parse DDEX
    parser = DDEXParser()
    with open(xml_file_path, 'r') as f:
        xml_content = f.read()
    
    df = await parser.to_dataframe(xml_content)
    
    # Create interactive analyzer
    analyzer = JupyterDDEXAnalyzer(df)
    
    return analyzer
```

## Performance Optimization

### Memory-Efficient Processing

```python
import gc
from typing import Iterator, Callable
import asyncio

class MemoryEfficientProcessor:
    """Memory-efficient processing for large DDEX catalogs"""
    
    def __init__(self, chunk_size: int = 1000):
        self.chunk_size = chunk_size
        self.parser = DDEXParser()
    
    async def process_large_catalog_streaming(
        self,
        xml_files: List[str],
        processor_func: Callable[[pd.DataFrame], pd.DataFrame]
    ) -> Iterator[pd.DataFrame]:
        """Process large catalogs in streaming fashion"""
        
        for xml_file in xml_files:
            try:
                # Process file in chunks if possible
                async for chunk_df in self._parse_file_chunked(xml_file):
                    # Apply processing function
                    processed_chunk = processor_func(chunk_df)
                    
                    yield processed_chunk
                    
                    # Force garbage collection
                    gc.collect()
                    
            except Exception as e:
                print(f"Error processing {xml_file}: {e}")
                continue
    
    async def _parse_file_chunked(self, xml_file: str) -> Iterator[pd.DataFrame]:
        """Parse large XML files in chunks"""
        
        with open(xml_file, 'r') as f:
            xml_content = f.read()
        
        # For very large files, implement streaming XML parsing
        if len(xml_content) > 50 * 1024 * 1024:  # 50MB threshold
            async for chunk in self._stream_parse_large_file(xml_content):
                yield chunk
        else:
            # Regular parsing for smaller files
            df = await self.parser.to_dataframe(xml_content)
            
            # Yield in chunks
            for i in range(0, len(df), self.chunk_size):
                chunk = df.iloc[i:i + self.chunk_size].copy()
                yield chunk
    
    async def _stream_parse_large_file(self, xml_content: str) -> Iterator[pd.DataFrame]:
        """Stream parse very large XML files"""
        
        # Use streaming parser for large files
        stream_parser = self.parser.create_streaming_parser()
        
        async for release_batch in stream_parser.parse_releases(xml_content, batch_size=self.chunk_size):
            # Convert batch to DataFrame
            df_chunk = pd.DataFrame(release_batch)
            yield df_chunk
    
    async def aggregate_results_efficiently(
        self,
        result_iterator: Iterator[pd.DataFrame],
        aggregation_funcs: Dict[str, Callable]
    ) -> Dict[str, Any]:
        """Efficiently aggregate results from streaming processing"""
        
        # Initialize accumulators
        accumulators = {}
        total_processed = 0
        
        async for chunk_df in result_iterator:
            total_processed += len(chunk_df)
            
            # Apply aggregation functions
            for name, func in aggregation_funcs.items():
                if name not in accumulators:
                    accumulators[name] = func(chunk_df)
                else:
                    # Combine with existing accumulator
                    accumulators[name] = self._combine_aggregations(
                        accumulators[name],
                        func(chunk_df),
                        name
                    )
            
            # Progress reporting
            if total_processed % 10000 == 0:
                print(f"Processed {total_processed} releases...")
        
        return {
            'results': accumulators,
            'total_processed': total_processed
        }
    
    def _combine_aggregations(self, acc1: Any, acc2: Any, agg_type: str) -> Any:
        """Combine two aggregation results"""
        
        if agg_type in ['sum', 'count']:
            return acc1 + acc2
        elif agg_type == 'mean':
            # Weighted average combination
            if isinstance(acc1, dict) and 'sum' in acc1 and 'count' in acc1:
                total_sum = acc1['sum'] + acc2['sum']
                total_count = acc1['count'] + acc2['count']
                return {
                    'sum': total_sum,
                    'count': total_count,
                    'mean': total_sum / total_count if total_count > 0 else 0
                }
        elif agg_type in ['min', 'max']:
            return min(acc1, acc2) if agg_type == 'min' else max(acc1, acc2)
        
        return acc2  # Default to latest value
```

## Common Pitfalls and Solutions

### 1. Memory Issues with Large DataFrames

**Pitfall**: Loading entire large catalogs into memory

```python
# DON'T - Memory intensive
df = await parser.to_dataframe(large_xml_content)  # Loads everything
results = df.groupby('genre').sum()  # High memory usage

# DO - Use streaming processing
async for chunk in process_large_catalog_streaming(xml_files, analyze_chunk):
    partial_results = chunk.groupby('genre').sum()
    aggregate_partial_results(partial_results)
```

### 2. Inefficient Data Types

**Pitfall**: Using default pandas dtypes without optimization

```python
# DON'T - Inefficient dtypes
df['genre'] = df['genre'].astype(str)  # Memory intensive for repetitive data

# DO - Use categorical for repetitive data
df['genre'] = df['genre'].astype('category')  # Much more memory efficient
df['territory'] = df['territory'].astype('category')
```

### 3. Missing Data Handling

**Pitfall**: Not properly handling missing DDEX data

```python
# DON'T - Ignore missing data
average_duration = df['duration_seconds'].mean()  # Skews results

# DO - Explicit missing data handling
# Check data completeness first
completeness = df['duration_seconds'].notna().mean()
print(f"Duration data completeness: {completeness:.1%}")

# Handle missing data appropriately
if completeness > 0.8:
    average_duration = df['duration_seconds'].mean()
else:
    # Use alternative calculation or imputation
    average_duration = df['duration_seconds'].median()
```

## Links to API Documentation

- [Python Parser API](/api/parser/python)
- [Python Builder API](/api/builder/python)
- [DataFrame Integration](/api/parser/python#dataframe-integration)
- [Polars Support](/api/parser/python#polars-support)
- [Analytics Extensions](/api/analytics/python)

This comprehensive guide provides everything needed for advanced DDEX data analysis workflows using Python, from basic DataFrame operations to high-performance analytics with modern data science tools.