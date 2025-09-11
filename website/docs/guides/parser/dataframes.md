# DataFrame Integration

Advanced pandas integration for music metadata analysis and data science workflows.

## Quick Start

Export parsed DDEX data directly to pandas DataFrames:

```python
from ddex_parser import DDEXParser
import pandas as pd

parser = DDEXParser()
result = parser.parse(xml_content)

# Convert to DataFrames
releases_df = parser.to_dataframe(result, 'releases')
tracks_df = parser.to_dataframe(result, 'tracks') 
parties_df = parser.to_dataframe(result, 'parties')

print(f"Found {len(releases_df)} releases with {len(tracks_df)} tracks")
```

## Available DataFrames

DDEX Suite can generate DataFrames for different data types:

### Releases DataFrame

```python
releases_df = parser.to_dataframe(result, 'releases')
print(releases_df.columns)
# Output: ['release_id', 'title', 'artist', 'upc', 'release_type', 
#          'release_date', 'p_line', 'c_line', 'genres', 'label']

# Basic analysis
print(f"Total releases: {len(releases_df)}")
print(f"Unique artists: {releases_df['artist'].nunique()}")
print(f"Release types: {releases_df['release_type'].value_counts()}")
```

### Tracks DataFrame

```python
tracks_df = parser.to_dataframe(result, 'tracks')
print(tracks_df.columns)
# Output: ['track_id', 'title', 'artist', 'isrc', 'duration_ms', 
#          'track_number', 'genres', 'release_id']

# Duration analysis
tracks_df['duration_minutes'] = tracks_df['duration_ms'] / 60000
avg_duration = tracks_df['duration_minutes'].mean()
print(f"Average track duration: {avg_duration:.2f} minutes")
```

### Parties DataFrame

```python
parties_df = parser.to_dataframe(result, 'parties')
print(parties_df.columns)
# Output: ['party_id', 'name', 'role', 'party_type', 'namespace']

# Role analysis
role_counts = parties_df['role'].value_counts()
print("Party roles:", role_counts.to_dict())
```

## Advanced Analytics

### Genre Analysis

```python
def analyze_genres(releases_df, tracks_df):
    """Comprehensive genre analysis across releases and tracks"""
    
    # Explode genre lists into separate rows
    release_genres = releases_df.explode('genres')['genres'].value_counts()
    track_genres = tracks_df.explode('genres')['genres'].value_counts()
    
    # Create comparison DataFrame
    genre_comparison = pd.DataFrame({
        'release_count': release_genres,
        'track_count': track_genres
    }).fillna(0)
    
    genre_comparison['total_mentions'] = (
        genre_comparison['release_count'] + genre_comparison['track_count']
    )
    
    return genre_comparison.sort_values('total_mentions', ascending=False)

# Usage
genre_analysis = analyze_genres(releases_df, tracks_df)
print("Top 10 genres:")
print(genre_analysis.head(10))
```

### Artist Collaboration Analysis

```python
def analyze_collaborations(tracks_df):
    """Analyze artist collaborations and featured appearances"""
    
    # Identify collaborative tracks (multiple artists)
    collaborative_tracks = tracks_df[
        tracks_df['artist'].str.contains(' feat\\.| ft\\.| featuring | & | and ', 
                                       case=False, na=False)
    ]
    
    # Extract primary and featured artists
    collaborative_tracks['primary_artist'] = collaborative_tracks['artist'].str.extract(
        r'^([^(]*?)(?:\s+(?:feat\.|ft\.|featuring)\s+|$)', expand=False
    ).str.strip()
    
    collaboration_stats = {
        'total_tracks': len(tracks_df),
        'collaborative_tracks': len(collaborative_tracks),
        'collaboration_rate': len(collaborative_tracks) / len(tracks_df) * 100,
        'most_collaborative_artists': collaborative_tracks['primary_artist'].value_counts().head(10)
    }
    
    return collaboration_stats

collab_stats = analyze_collaborations(tracks_df)
print(f"Collaboration rate: {collab_stats['collaboration_rate']:.1f}%")
```

### Release Timeline Analysis

```python
def analyze_release_timeline(releases_df):
    """Analyze release patterns over time"""
    
    # Convert release dates to datetime
    releases_df['release_date'] = pd.to_datetime(releases_df['release_date'])
    
    # Extract date components
    releases_df['year'] = releases_df['release_date'].dt.year
    releases_df['month'] = releases_df['release_date'].dt.month
    releases_df['quarter'] = releases_df['release_date'].dt.quarter
    
    # Yearly release counts
    yearly_releases = releases_df.groupby('year').size()
    
    # Monthly patterns
    monthly_pattern = releases_df.groupby('month').size()
    
    # Quarterly patterns
    quarterly_pattern = releases_df.groupby('quarter').size()
    
    return {
        'yearly': yearly_releases,
        'monthly': monthly_pattern,
        'quarterly': quarterly_pattern,
        'peak_year': yearly_releases.idxmax(),
        'peak_month': monthly_pattern.idxmax(),
        'peak_quarter': quarterly_pattern.idxmax()
    }

timeline = analyze_release_timeline(releases_df)
print(f"Peak release year: {timeline['peak_year']}")
print(f"Peak release month: {timeline['peak_month']}")
```

## Multi-File Analysis

Process multiple DDEX files and create a consolidated dataset:

```python
def create_master_dataset(xml_files):
    """Process multiple DDEX files into master DataFrames"""
    
    parser = DDEXParser()
    all_releases = []
    all_tracks = []
    all_parties = []
    
    for i, xml_file in enumerate(xml_files):
        print(f"Processing file {i+1}/{len(xml_files)}: {xml_file}")
        
        try:
            with open(xml_file, 'r', encoding='utf-8') as f:
                content = f.read()
            
            result = parser.parse(content)
            
            # Convert to DataFrames with source file info
            releases_df = parser.to_dataframe(result, 'releases')
            releases_df['source_file'] = xml_file
            
            tracks_df = parser.to_dataframe(result, 'tracks')
            tracks_df['source_file'] = xml_file
            
            parties_df = parser.to_dataframe(result, 'parties')
            parties_df['source_file'] = xml_file
            
            all_releases.append(releases_df)
            all_tracks.append(tracks_df)
            all_parties.append(parties_df)
            
        except Exception as e:
            print(f"Error processing {xml_file}: {e}")
            continue
    
    # Combine all DataFrames
    master_releases = pd.concat(all_releases, ignore_index=True)
    master_tracks = pd.concat(all_tracks, ignore_index=True)
    master_parties = pd.concat(all_parties, ignore_index=True)
    
    return master_releases, master_tracks, master_parties

# Usage
xml_files = ['release1.xml', 'release2.xml', 'release3.xml']
releases_df, tracks_df, parties_df = create_master_dataset(xml_files)

print(f"Master dataset: {len(releases_df)} releases, {len(tracks_df)} tracks")
```

## Data Quality Analysis

Identify data quality issues in your DDEX files:

```python
def analyze_data_quality(releases_df, tracks_df):
    """Comprehensive data quality analysis"""
    
    quality_report = {}
    
    # Missing data analysis
    quality_report['missing_data'] = {
        'releases': releases_df.isnull().sum().to_dict(),
        'tracks': tracks_df.isnull().sum().to_dict()
    }
    
    # Duplicate analysis
    quality_report['duplicates'] = {
        'duplicate_releases': releases_df.duplicated(['title', 'artist']).sum(),
        'duplicate_tracks': tracks_df.duplicated(['title', 'artist', 'isrc']).sum(),
        'duplicate_isrcs': tracks_df['isrc'].duplicated().sum(),
        'duplicate_upcs': releases_df['upc'].duplicated().sum()
    }
    
    # Format validation
    def validate_isrc(isrc):
        import re
        pattern = r'^[A-Z]{2}-[A-Z0-9]{3}-\d{2}-\d{5}$'
        return bool(re.match(pattern, str(isrc))) if pd.notna(isrc) else False
    
    def validate_upc(upc):
        return len(str(upc)) == 12 and str(upc).isdigit() if pd.notna(upc) else False
    
    quality_report['format_validation'] = {
        'invalid_isrcs': (~tracks_df['isrc'].apply(validate_isrc)).sum(),
        'invalid_upcs': (~releases_df['upc'].apply(validate_upc)).sum()
    }
    
    # Data consistency
    quality_report['consistency'] = {
        'releases_without_tracks': len(releases_df) - tracks_df['release_id'].nunique(),
        'tracks_without_releases': tracks_df[~tracks_df['release_id'].isin(releases_df['release_id'])].shape[0]
    }
    
    return quality_report

quality = analyze_data_quality(releases_df, tracks_df)
print("Data Quality Report:")
for section, data in quality.items():
    print(f"\n{section.upper()}:")
    for key, value in data.items():
        print(f"  {key}: {value}")
```

## Visualization Integration

Create visualizations using matplotlib and seaborn:

```python
import matplotlib.pyplot as plt
import seaborn as sns

def create_analytics_dashboard(releases_df, tracks_df):
    """Create comprehensive analytics dashboard"""
    
    fig, axes = plt.subplots(2, 3, figsize=(18, 12))
    fig.suptitle('DDEX Analytics Dashboard', fontsize=16)
    
    # 1. Release types distribution
    releases_df['release_type'].value_counts().plot(
        kind='bar', ax=axes[0,0], title='Release Types'
    )
    axes[0,0].tick_params(axis='x', rotation=45)
    
    # 2. Track duration distribution
    tracks_df['duration_minutes'] = tracks_df['duration_ms'] / 60000
    tracks_df['duration_minutes'].hist(
        bins=30, ax=axes[0,1], title='Track Duration Distribution'
    )
    axes[0,1].set_xlabel('Duration (minutes)')
    
    # 3. Top artists by track count
    top_artists = tracks_df['artist'].value_counts().head(10)
    top_artists.plot(kind='barh', ax=axes[0,2], title='Top Artists by Track Count')
    
    # 4. Release timeline
    releases_df['release_date'] = pd.to_datetime(releases_df['release_date'])
    releases_df['year'] = releases_df['release_date'].dt.year
    releases_df['year'].value_counts().sort_index().plot(
        kind='line', ax=axes[1,0], title='Releases Over Time'
    )
    
    # 5. Genre distribution (top 15)
    all_genres = releases_df.explode('genres')['genres'].value_counts().head(15)
    all_genres.plot(kind='barh', ax=axes[1,1], title='Top Genres')
    
    # 6. Label distribution (top 10)
    top_labels = releases_df['label'].value_counts().head(10)
    top_labels.plot(kind='pie', ax=axes[1,2], title='Top Labels', legend=False)
    
    plt.tight_layout()
    plt.show()
    
    return fig

# Create dashboard
dashboard = create_analytics_dashboard(releases_df, tracks_df)
```

## Export Options

Export processed DataFrames to various formats:

```python
def export_dataframes(releases_df, tracks_df, parties_df, output_dir='output'):
    """Export DataFrames to multiple formats"""
    
    import os
    os.makedirs(output_dir, exist_ok=True)
    
    # CSV exports
    releases_df.to_csv(f'{output_dir}/releases.csv', index=False)
    tracks_df.to_csv(f'{output_dir}/tracks.csv', index=False)
    parties_df.to_csv(f'{output_dir}/parties.csv', index=False)
    
    # Excel export with multiple sheets
    with pd.ExcelWriter(f'{output_dir}/ddex_data.xlsx') as writer:
        releases_df.to_excel(writer, sheet_name='Releases', index=False)
        tracks_df.to_excel(writer, sheet_name='Tracks', index=False)
        parties_df.to_excel(writer, sheet_name='Parties', index=False)
    
    # Parquet for fast loading
    releases_df.to_parquet(f'{output_dir}/releases.parquet')
    tracks_df.to_parquet(f'{output_dir}/tracks.parquet')
    parties_df.to_parquet(f'{output_dir}/parties.parquet')
    
    # JSON for API consumption
    releases_df.to_json(f'{output_dir}/releases.json', orient='records')
    tracks_df.to_json(f'{output_dir}/tracks.json', orient='records')
    
    print(f"Data exported to {output_dir}/")

# Export all data
export_dataframes(releases_df, tracks_df, parties_df)
```

## Performance Optimization

Optimize DataFrame operations for large datasets:

```python
def optimize_dataframes(releases_df, tracks_df):
    """Optimize DataFrames for better performance and memory usage"""
    
    # Convert object columns to categories for better performance
    categorical_columns = {
        'releases': ['release_type', 'label', 'artist'],
        'tracks': ['artist', 'track_type']
    }
    
    for col in categorical_columns['releases']:
        if col in releases_df.columns:
            releases_df[col] = releases_df[col].astype('category')
    
    for col in categorical_columns['tracks']:
        if col in tracks_df.columns:
            tracks_df[col] = tracks_df[col].astype('category')
    
    # Convert datetime columns
    if 'release_date' in releases_df.columns:
        releases_df['release_date'] = pd.to_datetime(releases_df['release_date'])
    
    # Optimize numeric columns
    numeric_columns = ['duration_ms', 'track_number']
    for col in numeric_columns:
        if col in tracks_df.columns:
            tracks_df[col] = pd.to_numeric(tracks_df[col], downcast='integer')
    
    return releases_df, tracks_df

# Optimize DataFrames
releases_df, tracks_df = optimize_dataframes(releases_df, tracks_df)
```

## Machine Learning Integration

Prepare data for machine learning workflows:

```python
from sklearn.preprocessing import LabelEncoder, StandardScaler
from sklearn.model_selection import train_test_split

def prepare_ml_features(tracks_df):
    """Prepare features for machine learning models"""
    
    # Create feature DataFrame
    ml_features = tracks_df.copy()
    
    # Encode categorical variables
    le = LabelEncoder()
    categorical_features = ['artist', 'genres']
    
    for feature in categorical_features:
        if feature in ml_features.columns:
            ml_features[f'{feature}_encoded'] = le.fit_transform(ml_features[feature].astype(str))
    
    # Create numerical features
    ml_features['title_length'] = ml_features['title'].str.len()
    ml_features['artist_name_length'] = ml_features['artist'].str.len()
    ml_features['has_featuring'] = ml_features['artist'].str.contains('feat|ft\\.', case=False)
    
    # Duration-based features
    ml_features['duration_minutes'] = ml_features['duration_ms'] / 60000
    ml_features['is_short_track'] = ml_features['duration_minutes'] < 2
    ml_features['is_long_track'] = ml_features['duration_minutes'] > 6
    
    return ml_features

# Prepare features
ml_data = prepare_ml_features(tracks_df)
print("ML features prepared:", ml_data.columns.tolist())
```

## TypeScript DataFrame Integration

For TypeScript/Node.js environments, integrate with data analysis libraries:

```typescript
import { DDEXParser } from 'ddex-parser';
import * as DataFrame from 'dataframe-js';

class DDEXDataFrameAnalyzer {
  private parser = new DDEXParser();
  
  async parseToDataFrame(xmlContent: string): Promise<{
    releases: DataFrame,
    tracks: DataFrame,
    parties: DataFrame
  }> {
    const result = await this.parser.parse(xmlContent);
    
    // Convert to DataFrame-js format
    const releasesDF = new DataFrame(result.flat.releases);
    const tracksDF = new DataFrame(result.flat.tracks);
    const partiesDF = new DataFrame(result.graph.parties);
    
    return { releases: releasesDF, tracks: tracksDF, parties: partiesDF };
  }
  
  analyzeGenres(releasesDF: DataFrame): any {
    return releasesDF
      .groupBy('genre')
      .aggregate(group => group.count())
      .sortBy('aggregation', true);
  }
  
  getTopArtists(tracksDF: DataFrame, limit: number = 10): any {
    return tracksDF
      .groupBy('artist')
      .aggregate(group => group.count())
      .sortBy('aggregation', true)
      .head(limit);
  }
}

// Usage
const analyzer = new DDEXDataFrameAnalyzer();
const { releases, tracks } = await analyzer.parseToDataFrame(xmlContent);
const topGenres = analyzer.analyzeGenres(releases);
console.log('Top genres:', topGenres.toCollection());
```

## Next Steps

- [Performance Optimization](./performance) - Optimize DataFrame operations
- [Integration Guides](../integration/database) - Connect DataFrames to databases
- [Advanced Analytics](../advanced/extensions) - Custom analysis workflows