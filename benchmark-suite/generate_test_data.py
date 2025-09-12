#!/usr/bin/env python3
"""
Generate test files of varying sizes for DDEX Suite performance benchmarking.
Creates files from 1KB to 100MB by scaling up base ERN structures.
"""

import os
import sys
import xml.etree.ElementTree as ET
from datetime import datetime
import random
import string

def generate_random_id(length=8):
    """Generate random alphanumeric ID"""
    return ''.join(random.choices(string.ascii_uppercase + string.digits, k=length))

def create_base_ern():
    """Create a base ERN 4.3 structure"""
    return """<?xml version='1.0' encoding='utf-8'?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43" MessageSchemaVersionId="ern/43" LanguageAndScriptCode="en">
  <MessageHeader>
    <MessageThreadId>BENCH_MSG_{thread_id}</MessageThreadId>
    <MessageId>MSG_{msg_id}</MessageId>
    <MessageSender>
      <PartyId>PADPIDA2014120301</PartyId>
      <PartyName>Benchmark Sender</PartyName>
    </MessageSender>
    <MessageRecipient>
      <PartyId>PADPIDA2014120302</PartyId>
      <PartyName>Benchmark Recipient</PartyName>
    </MessageRecipient>
    <MessageCreatedDateTime>{timestamp}</MessageCreatedDateTime>
  </MessageHeader>
  <ReleaseList>
    {releases}
  </ReleaseList>
  <ResourceList>
    {resources}
  </ResourceList>
  <DealList>
    {deals}
  </DealList>
</ern:NewReleaseMessage>"""

def create_release(release_num):
    """Create a single release element"""
    return f"""    <Release>
      <ReleaseReference>R{release_num}</ReleaseReference>
      <ReleaseId>
        <GRid>A1-BENCH-GRID-{release_num:04d}</GRid>
      </ReleaseId>
      <ReferenceTitle>
        <TitleText>Benchmark Release {release_num} - Generated for Performance Testing</TitleText>
      </ReferenceTitle>
      <DisplayArtist>
        <PartyName>
          <FullName>Benchmark Artist {release_num}</FullName>
        </PartyName>
      </DisplayArtist>
      <AdditionalTitle>
        <TitleText>Alternative Title for Release {release_num}</TitleText>
      </AdditionalTitle>
    </Release>"""

def create_sound_recording(track_num, release_num):
    """Create a sound recording resource"""
    return f"""    <SoundRecording>
      <ResourceReference>A{release_num}_{track_num}</ResourceReference>
      <SoundRecordingId>
        <ISRC>USBENCH{release_num:02d}{track_num:05d}</ISRC>
      </SoundRecordingId>
      <ReferenceTitle>
        <TitleText>Track {track_num} from Release {release_num} - Benchmark Audio</TitleText>
      </ReferenceTitle>
      <DisplayArtist>
        <PartyName>
          <FullName>Benchmark Artist {release_num}</FullName>
        </PartyName>
      </DisplayArtist>
      <Duration>PT3M45S</Duration>
    </SoundRecording>"""

def create_deal(release_num):
    """Create a deal for a release"""
    return f"""    <ReleaseDeal>
      <DealReleaseReference>R{release_num}</DealReleaseReference>
      <Deal>
        <TerritoryCode>Worldwide</TerritoryCode>
        <StartDate>2025-01-01</StartDate>
        <EndDate>2026-12-31</EndDate>
      </Deal>
    </ReleaseDeal>"""

def generate_file(target_size_bytes, output_path):
    """Generate a DDEX file of approximately target_size_bytes"""
    print(f"Generating {output_path} (target: {target_size_bytes/1024:.1f}KB)")
    
    # Start with base structure
    thread_id = generate_random_id()
    msg_id = generate_random_id(12)
    timestamp = datetime.now().isoformat()
    
    releases = []
    resources = []
    deals = []
    
    # Estimate size per release (about 500 bytes base + 300 per track)
    base_size = 800  # Base XML structure
    size_per_release = 500
    size_per_track = 300
    tracks_per_release = 5  # Average tracks per release
    
    current_size = base_size
    release_num = 1
    
    while current_size < target_size_bytes:
        # Create release
        releases.append(create_release(release_num))
        deals.append(create_deal(release_num))
        current_size += size_per_release
        
        # Create tracks for this release
        for track_num in range(1, tracks_per_release + 1):
            if current_size >= target_size_bytes:
                break
            resources.append(create_sound_recording(track_num, release_num))
            current_size += size_per_track
        
        release_num += 1
        
        # Safety check to avoid infinite loop
        if release_num > 10000:
            break
    
    # Build final XML
    xml_content = create_base_ern().format(
        thread_id=thread_id,
        msg_id=msg_id,
        timestamp=timestamp,
        releases='\n'.join(releases),
        resources='\n'.join(resources),
        deals='\n'.join(deals)
    )
    
    # Write to file
    with open(output_path, 'w', encoding='utf-8') as f:
        f.write(xml_content)
    
    # Check actual size
    actual_size = os.path.getsize(output_path)
    print(f"  Generated: {actual_size/1024:.1f}KB ({release_num-1} releases)")
    
    return actual_size

def main():
    """Generate benchmark test files"""
    os.makedirs('test-data', exist_ok=True)
    
    # File sizes to generate (in bytes)
    sizes = [
        (1024, "1kb.xml"),           # 1KB
        (5 * 1024, "5kb.xml"),       # 5KB  
        (10 * 1024, "10kb.xml"),     # 10KB
        (50 * 1024, "50kb.xml"),     # 50KB
        (100 * 1024, "100kb.xml"),   # 100KB
        (500 * 1024, "500kb.xml"),   # 500KB
        (1024 * 1024, "1mb.xml"),    # 1MB
        (5 * 1024 * 1024, "5mb.xml"), # 5MB
        (10 * 1024 * 1024, "10mb.xml"), # 10MB
        (25 * 1024 * 1024, "25mb.xml"), # 25MB
        (50 * 1024 * 1024, "50mb.xml"), # 50MB
        (100 * 1024 * 1024, "100mb.xml"), # 100MB
    ]
    
    print("🚀 Generating DDEX benchmark test files...")
    print("=" * 50)
    
    for target_size, filename in sizes:
        filepath = os.path.join('test-data', filename)
        try:
            actual_size = generate_file(target_size, filepath)
            size_mb = actual_size / (1024 * 1024)
            if size_mb >= 1:
                print(f"✅ {filename}: {size_mb:.1f}MB")
            else:
                print(f"✅ {filename}: {actual_size/1024:.1f}KB")
        except Exception as e:
            print(f"❌ Failed to generate {filename}: {e}")
    
    print("=" * 50)
    print("🎯 Benchmark test data generation completed!")

if __name__ == "__main__":
    main()