from ddex_builder import DdexBuilder
from ddex_parser import DDEXParser

# Parse any ERN 3.8.x, 4.2 or 4.3 message, then analyze it with pandas
with open("release.xml", encoding="utf-8") as f:
    message = DDEXParser().parse(f.read())
tracks = message.to_dataframe("tracks")
print(tracks[["track_title", "isrc", "duration", "disc_number"]])
print("total minutes:", tracks["duration"].sum() / 60)


# Build ERN 4.3: a required field that is missing fails the build, naming the field
def party(name, party_id):
    return {"party_name": [{"text": name}], "party_id": party_id}


xml = DdexBuilder().build({
    "header": {
        "message_sender": party("Example Label", "PADPIDA0000000001"),
        "message_recipient": party("Example DSP", "PADPIDA0000000002"),
    },
    "version": "4.3",
    "avs_version_id": "12",
    "language_and_script_code": "en",
    "releases": [{
        "release_id": "A1-ABCDE-0000000001-M",
        "title": [{"text": "Example Single"}],
        "artist": "Example Artist",
        "label": "Example Label",
        "release_type": "Single",
        "genre": "Pop",
        "parental_warning_type": "NotExplicit",
        "tracks": [{
            "track_id": "T1",
            "isrc": "ZZ0000000001",
            "title": "Example Track",
            "duration": "PT3M42S",
            "artist": "Example Artist",
            "resource_type": "MusicalWorkSoundRecording",
            "parental_warning_type": "NotExplicit",
        }],
    }],
    "deals": [],
})
print(xml)
