#!/usr/bin/env python3
import sys
import pandas as pd

sys.path.insert(0, 'ddex-parser/bindings/python')
sys.path.insert(0, 'ddex-builder/bindings/python')

# Try to import modules
try:
    import ddex_parser
    print("✅ ddex_parser imported")
except ImportError as e:
    print(f"❌ ddex_parser import failed: {e}")
    sys.exit(1)

try:
    import ddex_builder
    print("✅ ddex_builder imported")
    has_builder = True
except ImportError:
    print("⚠️ ddex_builder not available - testing parser DataFrame only")
    ddex_builder = None
    has_builder = False

def test_dataframe_integration():
    print("📊 Testing DataFrame Integration...")
    
    # Create test ERN XML
    test_xml = '''<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
    <MessageHeader>
        <MessageId>MSG123</MessageId>
        <MessageSender><PartyName><FullName>Sender</FullName></PartyName></MessageSender>
        <MessageRecipient><PartyName><FullName>Recipient</FullName></PartyName></MessageRecipient>
        <MessageCreatedDateTime>2025-01-01T00:00:00Z</MessageCreatedDateTime>
    </MessageHeader>
    <ResourceList>
        <SoundRecording>
            <ResourceReference>A1</ResourceReference>
            <ResourceId><ISRC>USRC17607839</ISRC></ResourceId>
            <ReferenceTitle><TitleText>Track 1</TitleText></ReferenceTitle>
            <Duration>PT3M30S</Duration>
        </SoundRecording>
        <SoundRecording>
            <ResourceReference>A2</ResourceReference>
            <ResourceId><ISRC>USRC17607840</ISRC></ResourceId>
            <ReferenceTitle><TitleText>Track 2</TitleText></ReferenceTitle>
            <Duration>PT4M15S</Duration>
        </SoundRecording>
    </ResourceList>
    <ReleaseList>
        <Release>
            <ReleaseReference>R1</ReleaseReference>
            <ReleaseId><GRid>A12345678901234</GRid></ReleaseId>
            <ReferenceTitle><TitleText>Test Album</TitleText></ReferenceTitle>
        </Release>
    </ReleaseList>
</ern:NewReleaseMessage>'''
    
    parser = ddex_parser.DDEXParser()
    
    print("\n1. Testing Parser DataFrame Methods:")
    
    # Test direct to_dataframe method from parser
    try:
        df = parser.to_dataframe(test_xml)
        print(f"✓ Direct to_dataframe() - Shape: {df.shape}")
        print(f"✓ Columns: {list(df.columns)}")
        
        # Show sample data
        print("\n📋 Sample DataFrame content:")
        print(df.head().to_string())
        
        # Analyze content
        if not df.empty:
            print(f"\n🔍 Data Analysis:")
            print(f"  - Non-empty DataFrame: ✓")
            print(f"  - Row count: {len(df)}")
            print(f"  - Column count: {len(df.columns)}")
            
            # Check for key DDEX fields
            key_fields = ['isrc', 'ISRC', 'title', 'release_id', 'artist', 'duration']
            found_fields = [field for field in key_fields if field in df.columns]
            if found_fields:
                print(f"  - Key DDEX fields found: {found_fields}")
            else:
                print("  - No standard DDEX fields detected")
                
            # Check for actual ISRC values
            for col in df.columns:
                if 'isrc' in col.lower():
                    unique_values = df[col].unique()
                    print(f"  - {col} values: {unique_values}")
                    if any('USRC' in str(v) for v in unique_values):
                        print("  - Real ISRC data detected: ✓")
        else:
            print("⚠️ DataFrame is empty")
            
    except Exception as e:
        print(f"⚠️ Direct to_dataframe() error: {e}")
        import traceback
        traceback.print_exc()
    
    # Test different DataFrame formats
    print("\n2. Testing Different DataFrame Formats:")
    formats = ['flat', 'releases', 'tracks']
    
    for fmt in formats:
        try:
            df = parser.to_dataframe(test_xml, format=fmt)
            print(f"✓ Format '{fmt}' - Shape: {df.shape}, Columns: {len(df.columns)}")
            if not df.empty:
                print(f"  Sample data: {df.iloc[0].to_dict() if len(df) > 0 else 'No data'}")
        except Exception as e:
            print(f"⚠️ Format '{fmt}' error: {e}")
    
    # Test parse result DataFrame conversion
    print("\n3. Testing ParseResult DataFrame Conversion:")
    try:
        result = parser.parse(test_xml)
        if hasattr(result, 'to_dataframe'):
            df = result.to_dataframe()
            print(f"✓ ParseResult.to_dataframe() - Shape: {df.shape}")
        else:
            print("⚠️ ParseResult.to_dataframe() method not found")
    except Exception as e:
        print(f"⚠️ ParseResult DataFrame error: {e}")
    
    # Test builder DataFrame support
    print("\n Testing Builder DataFrame Support...")
    builder = ddex_builder.DdexBuilder()
    
    if hasattr(builder, 'from_dataframe'):
        # Create test DataFrame
        test_df = pd.DataFrame({
            'release_id': ['REL001'],
            'title': ['Test Album'],
            'artist': ['Test Artist'],
            'isrc': ['USRC12345678']
        })
        
        try:
            xml = builder.from_dataframe(test_df)
            print("✓ Built XML from DataFrame")
        except Exception as e:
            print(f"⚠️ from_dataframe error: {e}")
    else:
        print("⚠️ from_dataframe() not yet implemented")
    
    print("\n✅ DataFrame integration test complete!")

if __name__ == "__main__":
    test_dataframe_integration()