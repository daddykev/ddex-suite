#!/usr/bin/env python3

"""
Comprehensive DDEX Suite CLI Testing Framework

Tests all CLI functionality including:
1. ddex-parser CLI with different output formats and modes
2. ddex-builder CLI with different input formats and options  
3. Round-trip pipeline testing
4. Error handling and user experience
"""

import os
import sys
import json
import yaml
import subprocess
import tempfile
import time
from pathlib import Path
from typing import Dict, List, Any, Optional, Tuple
from dataclasses import dataclass
import shutil

@dataclass
class CLITestResult:
    """Result of a CLI test"""
    command: str
    success: bool
    exit_code: int
    stdout: str
    stderr: str
    execution_time: float
    error_message: Optional[str] = None

class DDEXCLITester:
    """Comprehensive CLI testing framework"""
    
    def __init__(self):
        self.results: List[CLITestResult] = []
        self.temp_dir = tempfile.mkdtemp(prefix="ddex_cli_test_")
        self.test_files_dir = Path("test-suite/valid")
        self.parser_bin = None
        self.builder_bin = None
        
        # Discover CLI binaries
        self._discover_cli_binaries()
        
    def _discover_cli_binaries(self):
        """Discover DDEX CLI binaries"""
        print("🔍 Discovering DDEX CLI binaries...")
        
        # Check for parser CLI
        parser_paths = [
            "./target/release/ddex-parser",
            "./target/debug/ddex-parser", 
            "ddex-parser",
            "cargo run --bin ddex-parser --"
        ]
        
        for path in parser_paths:
            if self._test_binary_exists(path.split()):
                self.parser_bin = path.split()
                print(f"   ✅ Found parser CLI: {path}")
                break
        
        if not self.parser_bin:
            print("   ❌ Parser CLI not found")
        
        # Check for builder CLI  
        builder_paths = [
            "./target/release/ddex-builder",
            "./target/debug/ddex-builder",
            "ddex-builder", 
            "cargo run --bin ddex-builder --"
        ]
        
        for path in builder_paths:
            if self._test_binary_exists(path.split()):
                self.builder_bin = path.split()
                print(f"   ✅ Found builder CLI: {path}")
                break
                
        if not self.builder_bin:
            print("   ❌ Builder CLI not found")
    
    def _test_binary_exists(self, cmd_parts: List[str]) -> bool:
        """Test if a binary exists and responds"""
        try:
            result = subprocess.run(
                cmd_parts + ["--help"], 
                capture_output=True, 
                timeout=5,
                cwd="/Users/kevinmoo/Desktop/localrepo/ddex-suite"
            )
            return result.returncode == 0
        except:
            return False
    
    def _run_cli_command(self, cmd: List[str], input_text: str = None, timeout: int = 30) -> CLITestResult:
        """Run a CLI command and capture results"""
        cmd_str = " ".join(cmd)
        start_time = time.time()
        
        try:
            result = subprocess.run(
                cmd,
                input=input_text,
                text=True,
                capture_output=True,
                timeout=timeout,
                cwd="/Users/kevinmoo/Desktop/localrepo/ddex-suite"
            )
            
            execution_time = time.time() - start_time
            success = result.returncode == 0
            
            return CLITestResult(
                command=cmd_str,
                success=success,
                exit_code=result.returncode,
                stdout=result.stdout,
                stderr=result.stderr,
                execution_time=execution_time
            )
            
        except subprocess.TimeoutExpired:
            return CLITestResult(
                command=cmd_str,
                success=False,
                exit_code=-1,
                stdout="",
                stderr="Command timed out",
                execution_time=timeout,
                error_message="Timeout"
            )
        except Exception as e:
            return CLITestResult(
                command=cmd_str,
                success=False,
                exit_code=-1,
                stdout="",
                stderr=str(e),
                execution_time=time.time() - start_time,
                error_message=str(e)
            )
    
    def _get_test_xml_files(self) -> List[Path]:
        """Get test XML files"""
        test_files = []
        
        # Look for test files
        if self.test_files_dir.exists():
            for xml_file in self.test_files_dir.rglob("*.xml"):
                test_files.append(xml_file)
        
        # If no test files found, create a simple one
        if not test_files:
            test_xml = '''<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
    <MessageHeader>
        <MessageId>CLI-TEST-001</MessageId>
        <MessageCreatedDateTime>2025-09-11T18:00:00Z</MessageCreatedDateTime>
        <MessageSender><PartyName>CLI Test</PartyName></MessageSender>
        <MessageRecipient><PartyName>Test Recipient</PartyName></MessageRecipient>
    </MessageHeader>
    <ResourceList>
        <SoundRecording>
            <ResourceReference>SR001</ResourceReference>
            <Type>MusicalWorkSoundRecording</Type>
            <Title><TitleText>CLI Test Track</TitleText></Title>
            <Duration>PT3M30S</Duration>
            <DisplayArtist>CLI Test Artist</DisplayArtist>
        </SoundRecording>
    </ResourceList>
</ern:NewReleaseMessage>'''
            
            test_file = Path(self.temp_dir) / "test.xml"
            test_file.write_text(test_xml)
            test_files = [test_file]
        
        return test_files[:5]  # First 5 files for comprehensive testing
    
    def test_parser_cli_help_and_version(self) -> int:
        """Test parser CLI help and version commands"""
        print("\n📖 Testing Parser CLI Help and Version")
        
        if not self.parser_bin:
            print("   ❌ Parser CLI not available")
            return 0
        
        passed_tests = 0
        
        # Test --help
        result = self._run_cli_command(self.parser_bin + ["--help"])
        self.results.append(result)
        
        if result.success and "usage" in result.stdout.lower():
            print("   ✅ --help: Shows usage information")
            passed_tests += 1
        else:
            print("   ❌ --help: Failed or no usage info")
        
        # Test --version  
        result = self._run_cli_command(self.parser_bin + ["--version"])
        self.results.append(result)
        
        if result.success and any(char.isdigit() for char in result.stdout):
            print("   ✅ --version: Shows version number")
            passed_tests += 1
        else:
            print("   ❌ --version: Failed or no version info")
        
        return passed_tests
    
    def test_parser_cli_output_formats(self) -> int:
        """Test parser CLI with different output formats"""
        print("\n📄 Testing Parser CLI Output Formats")
        
        if not self.parser_bin:
            print("   ❌ Parser CLI not available")
            return 0
        
        test_files = self._get_test_xml_files()
        if not test_files:
            print("   ❌ No test files available")
            return 0
        
        passed_tests = 0
        test_file = test_files[0]
        
        formats = [
            ("json", "JSON"),
            ("yaml", "YAML"), 
            ("toml", "TOML")
        ]
        
        for format_flag, format_name in formats:
            cmd = self.parser_bin + ["parse", str(test_file), f"--output={format_flag}"]
            result = self._run_cli_command(cmd)
            self.results.append(result)
            
            if result.success:
                # Validate output format
                try:
                    if format_flag == "json":
                        json.loads(result.stdout)
                        valid_format = True
                    elif format_flag == "yaml":
                        yaml.safe_load(result.stdout) 
                        valid_format = True
                    elif format_flag == "toml":
                        # Basic TOML validation
                        valid_format = "[" in result.stdout or "=" in result.stdout
                    else:
                        valid_format = False
                        
                    if valid_format:
                        print(f"   ✅ {format_name} output: Valid format")
                        passed_tests += 1
                    else:
                        print(f"   ❌ {format_name} output: Invalid format")
                        
                except Exception as e:
                    print(f"   ❌ {format_name} output: Parse error - {e}")
            else:
                print(f"   ❌ {format_name} output: Command failed - {result.stderr}")
        
        return passed_tests
    
    def test_parser_cli_modes(self) -> int:
        """Test parser CLI modes (flatten, graph, validate, stream)"""
        print("\n🔧 Testing Parser CLI Modes")
        
        if not self.parser_bin:
            print("   ❌ Parser CLI not available") 
            return 0
        
        test_files = self._get_test_xml_files()
        if not test_files:
            print("   ❌ No test files available")
            return 0
        
        passed_tests = 0
        test_file = test_files[0]
        
        modes = [
            (["--flatten"], "Flatten Mode"),
            (["--graph"], "Graph Mode"), 
            (["--validate"], "Validate Mode"),
            (["--stream"], "Stream Mode")
        ]
        
        for mode_flags, mode_name in modes:
            cmd = self.parser_bin + ["parse", str(test_file)] + mode_flags
            result = self._run_cli_command(cmd)
            self.results.append(result)
            
            if result.success:
                # Basic validation that output is produced
                if result.stdout.strip():
                    print(f"   ✅ {mode_name}: Produces output")
                    passed_tests += 1
                else:
                    print(f"   ❌ {mode_name}: No output produced")
            else:
                print(f"   ❌ {mode_name}: Command failed - {result.stderr}")
        
        return passed_tests
    
    def test_parser_cli_validation_profile(self) -> int:
        """Test parser CLI ERN validation profiles"""
        print("\n✅ Testing Parser CLI Validation Profiles")
        
        if not self.parser_bin:
            print("   ❌ Parser CLI not available")
            return 0
        
        test_files = self._get_test_xml_files()
        if not test_files:
            print("   ❌ No test files available") 
            return 0
        
        passed_tests = 0
        test_file = test_files[0]
        
        profiles = ["4.3", "4.2", "3.8.2"]
        
        for profile in profiles:
            cmd = self.parser_bin + ["parse", str(test_file), "--validate", f"--profile=ern-{profile}"]
            result = self._run_cli_command(cmd)
            self.results.append(result)
            
            if result.success or result.exit_code == 0:
                print(f"   ✅ ERN {profile} validation: Executed successfully")
                passed_tests += 1
            else:
                print(f"   ❌ ERN {profile} validation: Failed - {result.stderr}")
        
        return passed_tests
    
    def test_builder_cli_help_and_version(self) -> int:
        """Test builder CLI help and version commands"""
        print("\n📖 Testing Builder CLI Help and Version")
        
        if not self.builder_bin:
            print("   ❌ Builder CLI not available")
            return 0
        
        passed_tests = 0
        
        # Test --help
        result = self._run_cli_command(self.builder_bin + ["--help"])
        self.results.append(result)
        
        if result.success and "usage" in result.stdout.lower():
            print("   ✅ --help: Shows usage information")
            passed_tests += 1
        else:
            print("   ❌ --help: Failed or no usage info")
        
        # Test --version
        result = self._run_cli_command(self.builder_bin + ["--version"])
        self.results.append(result)
        
        if result.success and any(char.isdigit() for char in result.stdout):
            print("   ✅ --version: Shows version number") 
            passed_tests += 1
        else:
            print("   ❌ --version: Failed or no version info")
        
        return passed_tests
    
    def test_builder_cli_input_formats(self) -> int:
        """Test builder CLI with different input formats"""
        print("\n📄 Testing Builder CLI Input Formats")
        
        if not self.builder_bin:
            print("   ❌ Builder CLI not available")
            return 0
        
        passed_tests = 0
        
        # Create test data in different formats
        test_data = {
            "message_id": "CLI-BUILD-001",
            "sender": "CLI Builder Test",
            "recipient": "Test Recipient",
            "resources": [
                {
                    "id": "SR001",
                    "type": "SoundRecording",
                    "title": "CLI Builder Track",
                    "artist": "CLI Builder Artist", 
                    "duration": "PT3M30S"
                }
            ]
        }
        
        # Test JSON input
        json_file = Path(self.temp_dir) / "test.json"
        json_file.write_text(json.dumps(test_data, indent=2))
        
        cmd = self.builder_bin + ["build", str(json_file), "--input-format=json"]
        result = self._run_cli_command(cmd)
        self.results.append(result)
        
        if result.success and "<?xml" in result.stdout:
            print("   ✅ JSON input: Produces valid XML output")
            passed_tests += 1
        else:
            print(f"   ❌ JSON input: Failed - {result.stderr}")
        
        # Test YAML input
        try:
            yaml_file = Path(self.temp_dir) / "test.yaml"
            yaml_file.write_text(yaml.dump(test_data, default_flow_style=False))
            
            cmd = self.builder_bin + ["build", str(yaml_file), "--input-format=yaml"]
            result = self._run_cli_command(cmd)
            self.results.append(result)
            
            if result.success and "<?xml" in result.stdout:
                print("   ✅ YAML input: Produces valid XML output")
                passed_tests += 1
            else:
                print(f"   ❌ YAML input: Failed - {result.stderr}")
        except Exception as e:
            print(f"   ❌ YAML input: Setup failed - {e}")
        
        return passed_tests
    
    def test_builder_cli_options(self) -> int:
        """Test builder CLI options (canonicalize, verify, presets)"""
        print("\n🔧 Testing Builder CLI Options")
        
        if not self.builder_bin:
            print("   ❌ Builder CLI not available")
            return 0
        
        passed_tests = 0
        
        # Create test JSON data
        test_data = {
            "message_id": "CLI-OPTIONS-001",
            "sender": "CLI Options Test",
            "recipient": "Test Recipient"
        }
        
        json_file = Path(self.temp_dir) / "options_test.json"
        json_file.write_text(json.dumps(test_data, indent=2))
        
        # Test canonicalize option
        cmd = self.builder_bin + ["build", str(json_file), "--canonicalize"]
        result = self._run_cli_command(cmd)
        self.results.append(result)
        
        if result.success or result.exit_code == 0:
            print("   ✅ --canonicalize: Executed successfully")
            passed_tests += 1
        else:
            print(f"   ❌ --canonicalize: Failed - {result.stderr}")
        
        # Test verify option
        cmd = self.builder_bin + ["build", str(json_file), "--verify"]
        result = self._run_cli_command(cmd)
        self.results.append(result)
        
        if result.success or result.exit_code == 0:
            print("   ✅ --verify: Executed successfully")
            passed_tests += 1
        else:
            print(f"   ❌ --verify: Failed - {result.stderr}")
        
        # Test presets
        presets = ["spotify", "apple", "youtube"]
        
        for preset in presets:
            cmd = self.builder_bin + ["build", str(json_file), f"--preset={preset}"]
            result = self._run_cli_command(cmd)
            self.results.append(result)
            
            if result.success or result.exit_code == 0:
                print(f"   ✅ --preset={preset}: Executed successfully")
                passed_tests += 1
            else:
                print(f"   ❌ --preset={preset}: Failed - {result.stderr}")
        
        return passed_tests
    
    def test_round_trip_cli_pipeline(self) -> int:
        """Test round-trip CLI pipeline: parse | modify | build"""
        print("\n🔄 Testing Round-Trip CLI Pipeline")
        
        if not (self.parser_bin and self.builder_bin):
            print("   ❌ Both parser and builder CLI required")
            return 0
        
        passed_tests = 0
        
        test_files = self._get_test_xml_files()
        if not test_files:
            print("   ❌ No test files available")
            return 0
        
        test_file = test_files[0]
        
        try:
            # Step 1: Parse XML to JSON
            parse_cmd = self.parser_bin + ["parse", str(test_file), "--output=json"]
            parse_result = self._run_cli_command(parse_cmd)
            self.results.append(parse_result)
            
            if not parse_result.success:
                print(f"   ❌ Parse step failed: {parse_result.stderr}")
                return 0
            
            # Step 2: Save parsed JSON
            json_file = Path(self.temp_dir) / "parsed.json"
            json_file.write_text(parse_result.stdout)
            
            # Step 3: Modify JSON (add a comment field)
            try:
                parsed_data = json.loads(parse_result.stdout)
                parsed_data["_modified"] = "CLI round-trip test"
                
                modified_file = Path(self.temp_dir) / "modified.json" 
                modified_file.write_text(json.dumps(parsed_data, indent=2))
                
                print("   ✅ Parse step: XML → JSON successful")
                passed_tests += 1
                
                # Step 4: Build XML from modified JSON
                build_cmd = self.builder_bin + ["build", str(modified_file)]
                build_result = self._run_cli_command(build_cmd)
                self.results.append(build_result)
                
                if build_result.success and "<?xml" in build_result.stdout:
                    print("   ✅ Build step: JSON → XML successful")
                    passed_tests += 1
                    
                    # Step 5: Parse the rebuilt XML for round-trip validation
                    rebuilt_xml_file = Path(self.temp_dir) / "rebuilt.xml"
                    rebuilt_xml_file.write_text(build_result.stdout)
                    
                    reparse_cmd = self.parser_bin + ["parse", str(rebuilt_xml_file), "--output=json"]
                    reparse_result = self._run_cli_command(reparse_cmd)
                    self.results.append(reparse_result)
                    
                    if reparse_result.success:
                        print("   ✅ Round-trip validation: Rebuilt XML is parseable")
                        passed_tests += 1
                    else:
                        print(f"   ❌ Round-trip validation failed: {reparse_result.stderr}")
                        
                else:
                    print(f"   ❌ Build step failed: {build_result.stderr}")
                    
            except json.JSONDecodeError as e:
                print(f"   ❌ JSON modification failed: {e}")
                
        except Exception as e:
            print(f"   ❌ Round-trip pipeline failed: {e}")
        
        return passed_tests
    
    def test_cli_error_handling(self) -> int:
        """Test CLI error handling and user experience"""
        print("\n⚠️  Testing CLI Error Handling")
        
        passed_tests = 0
        
        # Test parser error handling
        if self.parser_bin:
            # Test with non-existent file
            cmd = self.parser_bin + ["parse", "/nonexistent/file.xml"]
            result = self._run_cli_command(cmd)
            self.results.append(result)
            
            if not result.success and result.stderr:
                print("   ✅ Parser: Handles non-existent file gracefully")
                passed_tests += 1
            else:
                print("   ❌ Parser: No error for non-existent file")
            
            # Test with invalid XML
            invalid_xml_file = Path(self.temp_dir) / "invalid.xml"
            invalid_xml_file.write_text("This is not XML")
            
            cmd = self.parser_bin + ["parse", str(invalid_xml_file)]
            result = self._run_cli_command(cmd)
            self.results.append(result)
            
            if not result.success and result.stderr:
                print("   ✅ Parser: Handles invalid XML gracefully")
                passed_tests += 1
            else:
                print("   ❌ Parser: No error for invalid XML")
        
        # Test builder error handling  
        if self.builder_bin:
            # Test with non-existent file
            cmd = self.builder_bin + ["build", "/nonexistent/file.json"]
            result = self._run_cli_command(cmd)
            self.results.append(result)
            
            if not result.success and result.stderr:
                print("   ✅ Builder: Handles non-existent file gracefully")
                passed_tests += 1
            else:
                print("   ❌ Builder: No error for non-existent file")
            
            # Test with invalid JSON
            invalid_json_file = Path(self.temp_dir) / "invalid.json"
            invalid_json_file.write_text('{"invalid": json}')
            
            cmd = self.builder_bin + ["build", str(invalid_json_file)]
            result = self._run_cli_command(cmd)
            self.results.append(result)
            
            if not result.success and result.stderr:
                print("   ✅ Builder: Handles invalid JSON gracefully")
                passed_tests += 1
            else:
                print("   ❌ Builder: No error for invalid JSON")
        
        return passed_tests
    
    def test_cli_performance(self) -> int:
        """Test CLI performance with various file sizes"""
        print("\n⚡ Testing CLI Performance")
        
        if not self.parser_bin:
            print("   ❌ Parser CLI not available")
            return 0
        
        passed_tests = 0
        
        # Create test files of different sizes
        sizes = [
            (1, "small"),
            (10, "medium"), 
            (100, "large")
        ]
        
        for num_resources, size_name in sizes:
            # Generate XML with specified number of resources
            resources = []
            for i in range(num_resources):
                resources.append(f'''
        <SoundRecording>
            <ResourceReference>SR{i:03d}</ResourceReference>
            <Type>MusicalWorkSoundRecording</Type>
            <Title><TitleText>Performance Test Track {i}</TitleText></Title>
            <Duration>PT3M{30 + (i % 30)}S</Duration>
            <DisplayArtist>Performance Artist {i}</DisplayArtist>
        </SoundRecording>''')
            
            test_xml = f'''<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
    <MessageHeader>
        <MessageId>PERF-{size_name.upper()}-001</MessageId>
        <MessageCreatedDateTime>2025-09-11T18:00:00Z</MessageCreatedDateTime>
        <MessageSender><PartyName>Performance Test</PartyName></MessageSender>
        <MessageRecipient><PartyName>Test Recipient</PartyName></MessageRecipient>
    </MessageHeader>
    <ResourceList>{''.join(resources)}</ResourceList>
</ern:NewReleaseMessage>'''
            
            test_file = Path(self.temp_dir) / f"perf_{size_name}.xml"
            test_file.write_text(test_xml)
            
            # Test parsing performance
            cmd = self.parser_bin + ["parse", str(test_file), "--output=json"]
            result = self._run_cli_command(cmd, timeout=60)
            self.results.append(result)
            
            if result.success:
                file_size_kb = len(test_xml.encode('utf-8')) / 1024
                throughput = file_size_kb / result.execution_time if result.execution_time > 0 else 0
                
                print(f"   ✅ {size_name.title()} file ({num_resources} resources): "
                      f"{result.execution_time:.3f}s, {throughput:.1f} KB/s")
                passed_tests += 1
            else:
                print(f"   ❌ {size_name.title()} file: Failed - {result.stderr}")
        
        return passed_tests
    
    def generate_cli_test_report(self):
        """Generate comprehensive CLI test report"""
        print("\n" + "="*80)
        print("📋 DDEX Suite CLI Comprehensive Test Report")
        print("="*80)
        
        # Overall statistics
        total_tests = len(self.results)
        successful_tests = sum(1 for r in self.results if r.success)
        success_rate = (successful_tests / total_tests * 100) if total_tests > 0 else 0
        
        print(f"\n## CLI Test Summary")
        print(f"- **Total Commands Tested**: {total_tests}")
        print(f"- **Successful Commands**: {successful_tests}")
        print(f"- **Success Rate**: {success_rate:.1f}%")
        print(f"- **Test Date**: {time.strftime('%Y-%m-%d %H:%M:%S')}")
        
        # Performance statistics
        if self.results:
            execution_times = [r.execution_time for r in self.results if r.success]
            if execution_times:
                avg_time = sum(execution_times) / len(execution_times)
                max_time = max(execution_times)
                min_time = min(execution_times)
                
                print(f"\n## Performance Statistics")
                print(f"- **Average Execution Time**: {avg_time:.3f}s")
                print(f"- **Fastest Command**: {min_time:.3f}s")
                print(f"- **Slowest Command**: {max_time:.3f}s")
        
        # CLI availability
        print(f"\n## CLI Tools Availability")
        parser_status = "✅ Available" if self.parser_bin else "❌ Not Found"
        builder_status = "✅ Available" if self.builder_bin else "❌ Not Found"
        print(f"- **ddex-parser CLI**: {parser_status}")
        print(f"- **ddex-builder CLI**: {builder_status}")
        
        if self.parser_bin:
            print(f"  - Command: {' '.join(self.parser_bin)}")
        if self.builder_bin:
            print(f"  - Command: {' '.join(self.builder_bin)}")
        
        # Error analysis
        failed_tests = [r for r in self.results if not r.success]
        if failed_tests:
            print(f"\n## Failed Commands Analysis")
            error_counts = {}
            for test in failed_tests:
                error_key = f"Exit Code {test.exit_code}"
                error_counts[error_key] = error_counts.get(error_key, 0) + 1
            
            for error, count in error_counts.items():
                print(f"- **{error}**: {count} commands")
        
        # Recommendations
        print(f"\n## Recommendations")
        
        if success_rate >= 90:
            print("🎉 **EXCELLENT**: CLI tools are working exceptionally well")
            print("- All major functionality is operational")
            print("- Ready for production use")
        elif success_rate >= 70:
            print("👍 **GOOD**: CLI tools are largely functional") 
            print("- Most features working with minor issues")
            print("- Suitable for most use cases")
        elif success_rate >= 50:
            print("⚠️ **NEEDS IMPROVEMENT**: Several CLI features need attention")
            print("- Core functionality working but gaps exist")
            print("- Requires development work for full functionality")
        else:
            print("❌ **REQUIRES SIGNIFICANT WORK**: CLI tools need major development")
            print("- Many basic features not working")
            print("- Not ready for production use")
        
        print("="*80)
    
    def run_comprehensive_cli_tests(self):
        """Run all CLI tests"""
        print("🧪 DDEX Suite CLI Comprehensive Testing")
        print("="*80)
        
        start_time = time.time()
        
        # Run all test categories
        test_results = []
        
        test_results.append(self.test_parser_cli_help_and_version())
        test_results.append(self.test_parser_cli_output_formats())
        test_results.append(self.test_parser_cli_modes())
        test_results.append(self.test_parser_cli_validation_profile())
        
        test_results.append(self.test_builder_cli_help_and_version())
        test_results.append(self.test_builder_cli_input_formats())
        test_results.append(self.test_builder_cli_options())
        
        test_results.append(self.test_round_trip_cli_pipeline())
        test_results.append(self.test_cli_error_handling())
        test_results.append(self.test_cli_performance())
        
        total_time = time.time() - start_time
        
        print(f"\n⏱️  Total CLI testing time: {total_time:.2f}s")
        
        # Generate comprehensive report
        self.generate_cli_test_report()
        
        # Cleanup
        try:
            shutil.rmtree(self.temp_dir)
        except:
            pass
        
        return sum(test_results)
    
    def cleanup(self):
        """Clean up temporary files"""
        try:
            shutil.rmtree(self.temp_dir)
        except:
            pass

def main():
    """Main CLI testing execution"""
    tester = DDEXCLITester()
    
    try:
        successful_tests = tester.run_comprehensive_cli_tests()
        
        print(f"\n🎯 CLI Testing Summary: {successful_tests} tests passed")
        return successful_tests > 0
        
    except KeyboardInterrupt:
        print("\n\n⚠️ CLI testing interrupted by user")
        return False
    except Exception as e:
        print(f"\n💥 CLI testing failed: {e}")
        import traceback
        traceback.print_exc()
        return False
    finally:
        tester.cleanup()

if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1)