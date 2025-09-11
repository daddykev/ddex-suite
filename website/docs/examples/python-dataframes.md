# Using DDEX with Pandas DataFrames

Learn how to leverage pandas DataFrames for powerful DDEX data analysis, transformation, and visualization.

## Overview

The DDEX Suite provides seamless integration with pandas DataFrames, enabling:
- **Data Analysis**: Statistical analysis of catalog metadata
- **Data Cleaning**: Identifying and fixing data quality issues
- **Transformation**: Converting between formats and structures
- **Visualization**: Creating charts and reports from DDEX data
- **Machine Learning**: Feature extraction for recommendation systems

## Basic DataFrame Operations

### Converting DDEX to DataFrame

```python
import pandas as pd
from ddex_parser import DDEXParser
from ddex_builder import DDEXBuilder

# Initialize parser
parser = DDEXParser()

# Parse DDEX XML to DataFrame
with open('release.xml', 'r') as f:
    xml_content = f.read()

# Direct conversion to DataFrame
df = parser.to_dataframe(xml_content)
print(f"DataFrame shape: {df.shape}")
print(f"Columns: {list(df.columns)}")

# Preview the data
print(df.head())
```

### DataFrame Structure

The DDEX DataFrame has a flattened structure with these key columns:

```python
# Examine DataFrame structure
print("Core columns:")
core_columns = [
    'message_id', 'release_id', 'sound_recording_id', 'isrc',
    'title', 'display_artist', 'label_name', 'release_date',
    'genre', 'territory', 'deal_type', 'distribution_channel'
]

for col in core_columns:
    if col in df.columns:
        print(f"  {col}: {df[col].dtype}")

# Check for missing values
print("\nMissing values:")
print(df.isnull().sum())
```

## Data Analysis Examples

### Catalog Statistics

```python
import matplotlib.pyplot as plt
import seaborn as sns

class DDEXAnalyzer:
    def __init__(self, df: pd.DataFrame):
        self.df = df
    
    def catalog_overview(self) -> dict:
        """Generate comprehensive catalog statistics"""
        stats = {
            'total_releases': self.df['release_id'].nunique(),
            'total_tracks': self.df['sound_recording_id'].nunique(),
            'unique_artists': self.df['display_artist'].nunique(),
            'unique_labels': self.df['label_name'].nunique(),
            'date_range': {
                'earliest': self.df['release_date'].min(),
                'latest': self.df['release_date'].max()
            },
            'territories': self.df['territory'].nunique(),
            'genres': self.df['genre'].nunique()
        }
        
        return stats
    
    def top_entities(self, n: int = 10) -> dict:
        """Find top artists, labels, and genres"""
        return {
            'artists': self.df['display_artist'].value_counts().head(n),
            'labels': self.df['label_name'].value_counts().head(n),
            'genres': self.df['genre'].value_counts().head(n)
        }
    
    def release_trends(self) -> pd.DataFrame:
        """Analyze release trends over time"""
        # Extract year from release_date
        self.df['release_year'] = pd.to_datetime(self.df['release_date']).dt.year
        
        trends = self.df.groupby('release_year').agg({
            'release_id': 'nunique',
            'sound_recording_id': 'nunique',
            'display_artist': 'nunique'
        }).rename(columns={
            'release_id': 'releases',
            'sound_recording_id': 'tracks',
            'display_artist': 'artists'
        })
        
        return trends
    
    def territory_analysis(self) -> pd.DataFrame:
        """Analyze content distribution by territory"""
        territory_stats = self.df.groupby('territory').agg({
            'release_id': 'nunique',
            'display_artist': 'nunique',
            'label_name': 'nunique',
            'genre': lambda x: x.mode().iloc[0] if not x.empty else None
        }).rename(columns={
            'release_id': 'releases',
            'display_artist': 'artists',
            'label_name': 'labels',
            'genre': 'top_genre'
        })
        
        return territory_stats.sort_values('releases', ascending=False)

# Usage
analyzer = DDEXAnalyzer(df)
stats = analyzer.catalog_overview()
print(f"Catalog contains {stats['total_releases']} releases")
print(f"Date range: {stats['date_range']['earliest']} to {stats['date_range']['latest']}")

top_entities = analyzer.top_entities()
print(f"Top artist: {top_entities['artists'].index[0]} ({top_entities['artists'].iloc[0]} releases)")
```

### Data Quality Assessment

```python
class DataQualityChecker:
    def __init__(self, df: pd.DataFrame):
        self.df = df
        self.issues = []
    
    def check_missing_fields(self) -> pd.DataFrame:
        """Identify missing critical fields"""
        critical_fields = ['title', 'display_artist', 'isrc', 'release_date']
        
        missing_report = pd.DataFrame({
            'field': critical_fields,
            'missing_count': [self.df[field].isnull().sum() for field in critical_fields],
            'missing_percentage': [self.df[field].isnull().mean() * 100 for field in critical_fields]
        })
        
        return missing_report.sort_values('missing_percentage', ascending=False)
    
    def check_duplicate_isrcs(self) -> pd.DataFrame:
        """Find duplicate ISRCs (potential data quality issue)"""
        isrc_counts = self.df['isrc'].value_counts()
        duplicates = isrc_counts[isrc_counts > 1]
        
        if len(duplicates) > 0:
            duplicate_details = self.df[self.df['isrc'].isin(duplicates.index)][
                ['isrc', 'title', 'display_artist', 'release_date']
            ].sort_values('isrc')
            return duplicate_details
        
        return pd.DataFrame()
    
    def check_date_consistency(self) -> pd.DataFrame:
        """Check for date inconsistencies"""
        # Convert to datetime
        self.df['release_date_parsed'] = pd.to_datetime(self.df['release_date'], errors='coerce')
        
        # Find invalid dates
        invalid_dates = self.df[self.df['release_date_parsed'].isnull() & self.df['release_date'].notna()]
        
        # Find future dates
        future_dates = self.df[self.df['release_date_parsed'] > pd.Timestamp.now()]
        
        issues = pd.DataFrame({
            'issue_type': ['Invalid Date Format', 'Future Release Date'],
            'count': [len(invalid_dates), len(future_dates)]
        })
        
        return issues
    
    def generate_quality_report(self) -> dict:
        """Generate comprehensive data quality report"""
        report = {
            'missing_fields': self.check_missing_fields(),
            'duplicate_isrcs': self.check_duplicate_isrcs(),
            'date_issues': self.check_date_consistency(),
            'completeness_score': self.calculate_completeness_score()
        }
        
        return report
    
    def calculate_completeness_score(self) -> float:
        """Calculate overall data completeness score (0-100)"""
        critical_fields = ['title', 'display_artist', 'isrc', 'release_date', 'genre']
        
        total_cells = len(self.df) * len(critical_fields)
        filled_cells = total_cells - self.df[critical_fields].isnull().sum().sum()
        
        return (filled_cells / total_cells) * 100

# Usage
quality_checker = DataQualityChecker(df)
quality_report = quality_checker.generate_quality_report()

print(f"Data completeness score: {quality_report['completeness_score']:.1f}%")
print("\nMissing field analysis:")
print(quality_report['missing_fields'])

if not quality_report['duplicate_isrcs'].empty:
    print(f"\nFound {len(quality_report['duplicate_isrcs'])} duplicate ISRCs")
```

## Data Transformation

### Cleaning and Standardization

```python
class DDEXDataCleaner:
    def __init__(self, df: pd.DataFrame):
        self.df = df.copy()
    
    def standardize_artist_names(self) -> pd.DataFrame:
        """Standardize artist name formats"""
        # Remove extra whitespace
        self.df['display_artist'] = self.df['display_artist'].str.strip()
        
        # Standardize featuring formats
        self.df['display_artist'] = self.df['display_artist'].str.replace(
            r'\s+(ft\.?|feat\.?|featuring)\s+', ' feat. ', regex=True, case=False
        )
        
        # Standardize "and" vs "&"
        self.df['display_artist'] = self.df['display_artist'].str.replace(' & ', ' and ')
        
        return self.df
    
    def normalize_genres(self) -> pd.DataFrame:
        """Normalize genre categories"""
        genre_mapping = {
            'Hip-Hop': ['Hip Hop', 'Rap', 'Hip-hop'],
            'Electronic': ['EDM', 'House', 'Techno', 'Electronic Dance Music'],
            'R&B': ['RnB', 'R and B', 'Rhythm and Blues'],
            'Alternative': ['Alt Rock', 'Alternative Rock', 'Indie Rock']
        }
        
        for standard_genre, variants in genre_mapping.items():
            mask = self.df['genre'].isin(variants)
            self.df.loc[mask, 'genre'] = standard_genre
        
        return self.df
    
    def fix_date_formats(self) -> pd.DataFrame:
        """Standardize date formats"""
        # Convert to datetime and back to ISO format
        self.df['release_date'] = pd.to_datetime(
            self.df['release_date'], 
            errors='coerce'
        ).dt.strftime('%Y-%m-%d')
        
        return self.df
    
    def deduplicate_releases(self) -> pd.DataFrame:
        """Remove duplicate releases based on ISRC"""
        # Keep first occurrence of each ISRC
        self.df = self.df.drop_duplicates(subset=['isrc'], keep='first')
        return self.df
    
    def clean_all(self) -> pd.DataFrame:
        """Apply all cleaning operations"""
        self.standardize_artist_names()
        self.normalize_genres()
        self.fix_date_formats()
        self.deduplicate_releases()
        return self.df

# Usage
cleaner = DDEXDataCleaner(df)
cleaned_df = cleaner.clean_all()
print(f"Original: {len(df)} rows, Cleaned: {len(cleaned_df)} rows")
```

### Feature Engineering

```python
class DDEXFeatureEngineer:
    def __init__(self, df: pd.DataFrame):
        self.df = df.copy()
    
    def extract_temporal_features(self) -> pd.DataFrame:
        """Extract temporal features from release dates"""
        self.df['release_date'] = pd.to_datetime(self.df['release_date'])
        
        self.df['release_year'] = self.df['release_date'].dt.year
        self.df['release_month'] = self.df['release_date'].dt.month
        self.df['release_quarter'] = self.df['release_date'].dt.quarter
        self.df['release_weekday'] = self.df['release_date'].dt.day_name()
        
        # Calculate time since release
        self.df['days_since_release'] = (pd.Timestamp.now() - self.df['release_date']).dt.days
        
        return self.df
    
    def create_artist_features(self) -> pd.DataFrame:
        """Create artist-level features"""
        # Artist productivity
        artist_stats = self.df.groupby('display_artist').agg({
            'release_id': 'nunique',
            'sound_recording_id': 'nunique',
            'genre': lambda x: x.mode().iloc[0] if not x.empty else None,
            'label_name': 'nunique'
        }).rename(columns={
            'release_id': 'artist_release_count',
            'sound_recording_id': 'artist_track_count',
            'genre': 'artist_primary_genre',
            'label_name': 'artist_label_count'
        })
        
        # Merge back to main DataFrame
        self.df = self.df.merge(artist_stats, left_on='display_artist', right_index=True)
        
        # Artist collaboration indicator
        self.df['is_collaboration'] = self.df['display_artist'].str.contains(
            r'(feat\.?|ft\.?|and|&|,)', regex=True, case=False
        )
        
        return self.df
    
    def create_label_features(self) -> pd.DataFrame:
        """Create label-level features"""
        label_stats = self.df.groupby('label_name').agg({
            'release_id': 'nunique',
            'display_artist': 'nunique',
            'genre': lambda x: x.mode().iloc[0] if not x.empty else None
        }).rename(columns={
            'release_id': 'label_release_count',
            'display_artist': 'label_artist_count',
            'genre': 'label_primary_genre'
        })
        
        self.df = self.df.merge(label_stats, left_on='label_name', right_index=True)
        return self.df
    
    def create_genre_features(self) -> pd.DataFrame:
        """Create genre-based features"""
        # One-hot encode genres
        genre_dummies = pd.get_dummies(self.df['genre'], prefix='genre')
        self.df = pd.concat([self.df, genre_dummies], axis=1)
        
        # Genre popularity score
        genre_popularity = self.df['genre'].value_counts(normalize=True)
        self.df['genre_popularity_score'] = self.df['genre'].map(genre_popularity)
        
        return self.df

# Usage
feature_engineer = DDEXFeatureEngineer(cleaned_df)
feature_engineer.extract_temporal_features()
feature_engineer.create_artist_features()
feature_engineer.create_label_features()
feature_engineer.create_genre_features()

enriched_df = feature_engineer.df
print(f"Added features. New shape: {enriched_df.shape}")
```

## Converting DataFrame Back to DDEX

### Building DDEX from Modified DataFrame

```python
from ddex_builder import DDEXBuilder

class DataFrameToDDEX:
    def __init__(self):
        self.builder = DDEXBuilder()
    
    def df_to_ddex(self, df: pd.DataFrame, preset: str = 'spotify') -> str:
        """Convert DataFrame back to DDEX XML"""
        
        # Group by release to reconstruct release structure
        releases = []
        
        for release_id, release_group in df.groupby('release_id'):
            # Get release-level info (first row)
            release_info = release_group.iloc[0]
            
            # Collect sound recordings for this release
            sound_recordings = []
            for _, track in release_group.iterrows():
                if pd.notna(track['sound_recording_id']):
                    sound_recordings.append({
                        'soundRecordingId': track['sound_recording_id'],
                        'isrc': track['isrc'],
                        'title': track['title'],
                        'displayArtist': track['display_artist'],
                        'duration': track.get('duration', 'PT3M30S')
                    })
            
            # Create release object
            release = {
                'releaseId': release_id,
                'releaseType': release_info.get('release_type', 'Single'),
                'releaseDetailsByTerritory': [{
                    'territory': release_info.get('territory', 'Worldwide'),
                    'displayArtist': release_info['display_artist'],
                    'labelName': release_info['label_name'],
                    'title': release_info['title'],
                    'releaseDate': release_info['release_date'],
                    'genre': release_info['genre']
                }],
                'soundRecordings': sound_recordings
            }
            releases.append(release)
        
        # Create build request
        build_request = {
            'version': '4.3',
            'messageHeader': {
                'messageId': f"MSG_{pd.Timestamp.now().strftime('%Y%m%d_%H%M%S')}",
                'sentOnBehalfOf': 'DataProcessor',
                'messageRecipient': 'Platform'
            },
            'updateIndicator': 'OriginalMessage',
            'messageControlType': 'LiveMessage',
            'releaseList': releases
        }
        
        # Build XML
        return self.builder.build(build_request, preset=preset)

# Usage - modify DataFrame and convert back to DDEX
modified_df = enriched_df.copy()

# Example modification: update all pop genre to "Pop/Rock"
modified_df.loc[modified_df['genre'] == 'Pop', 'genre'] = 'Pop/Rock'

# Convert back to DDEX
converter = DataFrameToDDEX()
updated_xml = converter.df_to_ddex(modified_df)

# Save updated DDEX
with open('updated_catalog.xml', 'w') as f:
    f.write(updated_xml)
```

## Advanced Analytics

### Statistical Analysis

```python
import scipy.stats as stats
from sklearn.preprocessing import LabelEncoder
from sklearn.cluster import KMeans

class DDEXStatistics:
    def __init__(self, df: pd.DataFrame):
        self.df = df
    
    def release_frequency_analysis(self) -> dict:
        """Analyze release frequency patterns"""
        # Releases per artist distribution
        releases_per_artist = self.df.groupby('display_artist')['release_id'].nunique()
        
        return {
            'mean_releases_per_artist': releases_per_artist.mean(),
            'median_releases_per_artist': releases_per_artist.median(),
            'distribution_stats': releases_per_artist.describe(),
            'prolific_artists': releases_per_artist.nlargest(10)
        }
    
    def genre_correlation_analysis(self) -> pd.DataFrame:
        """Analyze correlations between genres and other factors"""
        # Encode categorical variables
        le_genre = LabelEncoder()
        le_label = LabelEncoder()
        
        analysis_df = self.df.copy()
        analysis_df['genre_encoded'] = le_genre.fit_transform(analysis_df['genre'])
        analysis_df['label_encoded'] = le_label.fit_transform(analysis_df['label_name'])
        
        # Calculate correlations
        correlation_matrix = analysis_df[[
            'genre_encoded', 'label_encoded', 'release_year', 
            'artist_release_count', 'label_release_count'
        ]].corr()
        
        return correlation_matrix
    
    def seasonal_analysis(self) -> pd.DataFrame:
        """Analyze seasonal release patterns"""
        monthly_releases = self.df.groupby('release_month').agg({
            'release_id': 'nunique',
            'genre': lambda x: x.mode().iloc[0] if not x.empty else None
        }).rename(columns={
            'release_id': 'release_count',
            'genre': 'dominant_genre'
        })
        
        # Add month names
        month_names = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun',
                      'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec']
        monthly_releases['month_name'] = [month_names[i-1] for i in monthly_releases.index]
        
        return monthly_releases

# Usage
stats_analyzer = DDEXStatistics(enriched_df)
frequency_analysis = stats_analyzer.release_frequency_analysis()
print(f"Average releases per artist: {frequency_analysis['mean_releases_per_artist']:.2f}")

seasonal_data = stats_analyzer.seasonal_analysis()
print("\nSeasonal release patterns:")
print(seasonal_data[['month_name', 'release_count', 'dominant_genre']])
```

### Clustering and Segmentation

```python
class DDEXClustering:
    def __init__(self, df: pd.DataFrame):
        self.df = df
    
    def artist_segmentation(self, n_clusters: int = 5) -> pd.DataFrame:
        """Segment artists based on their characteristics"""
        # Prepare features for clustering
        artist_features = self.df.groupby('display_artist').agg({
            'release_id': 'nunique',
            'sound_recording_id': 'nunique',
            'genre': lambda x: x.mode().iloc[0] if not x.empty else 'Unknown',
            'label_name': 'nunique',
            'days_since_release': 'mean'
        }).rename(columns={
            'release_id': 'total_releases',
            'sound_recording_id': 'total_tracks',
            'genre': 'primary_genre',
            'label_name': 'label_count',
            'days_since_release': 'avg_days_since_release'
        })
        
        # Encode categorical variables
        le_genre = LabelEncoder()
        artist_features['genre_encoded'] = le_genre.fit_transform(artist_features['primary_genre'])
        
        # Select numeric features for clustering
        cluster_features = artist_features[['total_releases', 'total_tracks', 
                                         'label_count', 'genre_encoded']]
        
        # Perform clustering
        kmeans = KMeans(n_clusters=n_clusters, random_state=42)
        artist_features['cluster'] = kmeans.fit_predict(cluster_features)
        
        # Analyze clusters
        cluster_analysis = artist_features.groupby('cluster').agg({
            'total_releases': ['mean', 'std'],
            'total_tracks': ['mean', 'std'],
            'primary_genre': lambda x: x.mode().iloc[0] if not x.empty else 'Mixed'
        })
        
        return artist_features, cluster_analysis

# Usage
clusterer = DDEXClustering(enriched_df)
artist_segments, cluster_summary = clusterer.artist_segmentation()

print("Artist segments:")
for cluster in cluster_summary.index:
    artists_in_cluster = artist_segments[artist_segments['cluster'] == cluster]
    print(f"\nCluster {cluster}: {len(artists_in_cluster)} artists")
    print(f"  Avg releases: {cluster_summary.loc[cluster, ('total_releases', 'mean')]:.1f}")
    print(f"  Dominant genre: {cluster_summary.loc[cluster, ('primary_genre', '<lambda>')]}")
```

## Visualization

### Creating Charts and Reports

```python
import matplotlib.pyplot as plt
import seaborn as sns
import plotly.express as px
import plotly.graph_objects as go

class DDEXVisualizer:
    def __init__(self, df: pd.DataFrame):
        self.df = df
        plt.style.use('seaborn-v0_8')
    
    def create_release_timeline(self) -> None:
        """Create timeline of releases"""
        timeline_data = self.df.groupby('release_year')['release_id'].nunique()
        
        plt.figure(figsize=(12, 6))
        plt.plot(timeline_data.index, timeline_data.values, marker='o', linewidth=2)
        plt.title('Release Count Over Time')
        plt.xlabel('Year')
        plt.ylabel('Number of Releases')
        plt.grid(True, alpha=0.3)
        plt.tight_layout()
        plt.show()
    
    def create_genre_distribution(self) -> None:
        """Create genre distribution chart"""
        genre_counts = self.df['genre'].value_counts().head(10)
        
        plt.figure(figsize=(10, 6))
        sns.barplot(x=genre_counts.values, y=genre_counts.index)
        plt.title('Top 10 Genres by Release Count')
        plt.xlabel('Number of Releases')
        plt.tight_layout()
        plt.show()
    
    def create_interactive_dashboard(self) -> None:
        """Create interactive Plotly dashboard"""
        # Prepare data
        artist_stats = self.df.groupby('display_artist').agg({
            'release_id': 'nunique',
            'sound_recording_id': 'nunique',
            'genre': lambda x: x.mode().iloc[0] if not x.empty else 'Unknown'
        }).reset_index()
        
        # Create scatter plot
        fig = px.scatter(
            artist_stats,
            x='release_id',
            y='sound_recording_id',
            color='genre',
            hover_data=['display_artist'],
            title='Artist Productivity: Releases vs Tracks',
            labels={'release_id': 'Number of Releases', 
                   'sound_recording_id': 'Number of Tracks'}
        )
        
        fig.show()
    
    def create_territory_heatmap(self) -> None:
        """Create territory analysis heatmap"""
        territory_genre = pd.crosstab(self.df['territory'], self.df['genre'])
        
        plt.figure(figsize=(12, 8))
        sns.heatmap(territory_genre, annot=True, fmt='d', cmap='YlOrRd')
        plt.title('Release Distribution: Territory vs Genre')
        plt.tight_layout()
        plt.show()

# Usage
visualizer = DDEXVisualizer(enriched_df)
visualizer.create_release_timeline()
visualizer.create_genre_distribution()
visualizer.create_interactive_dashboard()
```

## Best Practices

### Performance Optimization

```python
# Use efficient data types
def optimize_dataframe(df: pd.DataFrame) -> pd.DataFrame:
    """Optimize DataFrame memory usage"""
    # Convert object columns to category if appropriate
    for col in df.select_dtypes(include=['object']):
        if df[col].nunique() / len(df) < 0.5:  # Less than 50% unique values
            df[col] = df[col].astype('category')
    
    # Optimize integer columns
    for col in df.select_dtypes(include=['int64']):
        if df[col].min() >= 0:
            if df[col].max() < 255:
                df[col] = df[col].astype('uint8')
            elif df[col].max() < 65535:
                df[col] = df[col].astype('uint16')
    
    return df

# Use chunking for large datasets
def process_large_catalog(file_paths: list, chunk_size: int = 1000):
    """Process large catalogs in chunks"""
    all_chunks = []
    
    for i in range(0, len(file_paths), chunk_size):
        chunk_files = file_paths[i:i + chunk_size]
        chunk_df = process_file_batch(chunk_files)
        all_chunks.append(chunk_df)
    
    return pd.concat(all_chunks, ignore_index=True)
```

### Error Handling

```python
def safe_dataframe_operations(df: pd.DataFrame) -> pd.DataFrame:
    """Perform DataFrame operations with error handling"""
    try:
        # Validate required columns
        required_cols = ['release_id', 'title', 'display_artist']
        missing_cols = [col for col in required_cols if col not in df.columns]
        
        if missing_cols:
            raise ValueError(f"Missing required columns: {missing_cols}")
        
        # Check for empty DataFrame
        if df.empty:
            raise ValueError("DataFrame is empty")
        
        # Perform operations
        result_df = df.copy()
        
        # Safe date parsing
        if 'release_date' in result_df.columns:
            result_df['release_date'] = pd.to_datetime(
                result_df['release_date'], 
                errors='coerce'
            )
        
        return result_df
        
    except Exception as e:
        print(f"Error processing DataFrame: {e}")
        return pd.DataFrame()  # Return empty DataFrame on error
```

## Integration Patterns

### Combining with Other Tools

```python
# Export to different formats
def export_analysis_results(df: pd.DataFrame, base_path: str):
    """Export analysis results to multiple formats"""
    
    # Excel with multiple sheets
    with pd.ExcelWriter(f'{base_path}_analysis.xlsx') as writer:
        df.to_excel(writer, sheet_name='Raw Data', index=False)
        
        # Summary statistics
        summary = df.describe()
        summary.to_excel(writer, sheet_name='Summary Stats')
        
        # Top entities
        top_artists = df['display_artist'].value_counts().head(20)
        top_artists.to_excel(writer, sheet_name='Top Artists')
    
    # CSV for data science workflows
    df.to_csv(f'{base_path}_data.csv', index=False)
    
    # JSON for web applications
    df.to_json(f'{base_path}_data.json', orient='records', indent=2)
    
    # Parquet for big data workflows
    df.to_parquet(f'{base_path}_data.parquet', index=False)

# Database integration
def save_to_database(df: pd.DataFrame, connection_string: str):
    """Save DataFrame to database"""
    from sqlalchemy import create_engine
    
    engine = create_engine(connection_string)
    df.to_sql('ddex_releases', engine, if_exists='replace', index=False)
```

The pandas integration provides powerful capabilities for DDEX data analysis, from basic statistics to advanced machine learning workflows.