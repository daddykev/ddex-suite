#!/usr/bin/env python3
"""Validate column consistency across all DataFrame methods"""

import sys
sys.path.insert(0, 'packages/ddex-parser/bindings/python')

from ddex_parser import DDEXParser
import pandas as pd

def test_column_consistency():
    print("🔍 Testing Column Consistency\n")
    print("=" * 60)
    
    # Simple test XML
    xml = '''<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
    <MessageHeader>
        <MessageId>MSG123</MessageId>
        <MessageSender><PartyName><FullName>Sender</FullName></PartyName></MessageSender>
        <MessageRecipient><PartyName><FullName>Recipient</FullName></PartyName></MessageRecipient>
        <MessageCreatedDateTime>2025-01-01T00:00:00Z</MessageCreatedDateTime>
    </MessageHeader>
    <ReleaseList>
        <Release>
            <ReleaseReference>R1</ReleaseReference>
            <ReleaseId><GRid>A12345678901234</GRid></ReleaseId>
            <ReferenceTitle><TitleText>Test Album</TitleText></ReferenceTitle>
        </Release>
    </ReleaseList>
</ern:NewReleaseMessage>'''
    
    parser = DDEXParser()
    result = parser.parse(xml)
    
    # Test each schema
    schemas = ['flat', 'releases', 'tracks']
    
    for schema in schemas:
        print(f"\n📊 Schema: {schema}")
        print("-" * 40)
        
        # Get DataFrame from both methods
        df1 = parser.to_dataframe(xml, schema=schema)
        df2 = result.to_dataframe(schema=schema)
        
        print(f"DDEXParser.to_dataframe():     {df1.shape} - Columns: {sorted(df1.columns.tolist())}")
        print(f"ParseResult.to_dataframe():    {df2.shape} - Columns: {sorted(df2.columns.tolist())}")
        
        # Check consistency
        if df1.shape[1] == df2.shape[1]:
            if set(df1.columns) == set(df2.columns):
                print("✅ Perfect consistency!")
            else:
                diff1 = set(df1.columns) - set(df2.columns)
                diff2 = set(df2.columns) - set(df1.columns)
                if diff1:
                    print(f"❌ Parser has extra columns: {diff1}")
                if diff2:
                    print(f"❌ ParseResult has extra columns: {diff2}")
        else:
            print(f"❌ Column count mismatch: {df1.shape[1]} vs {df2.shape[1]}")
    
    print("\n" + "=" * 60)
    print("Column Consistency Test Complete!")

if __name__ == "__main__":
    test_column_consistency()