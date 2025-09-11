# Automation

Automate DDEX processing workflows using various automation tools and frameworks.

## Overview

Automation enables:
- Scheduled batch processing
- Event-driven processing
- CI/CD integration
- Workflow orchestration
- Monitoring and alerting

## GitHub Actions

### Automated DDEX Validation

```yaml
name: DDEX Validation
on:
  push:
    paths:
      - 'ddex/**/*.xml'
  pull_request:
    paths:
      - 'ddex/**/*.xml'

jobs:
  validate-ddex:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '18'
          
      - name: Install DDEX Suite
        run: npm install -g ddex-parser ddex-builder
        
      - name: Validate DDEX Files
        run: |
          for file in ddex/**/*.xml; do
            echo "Validating $file"
            ddex-parser validate "$file" --strict
            
            if [ $? -ne 0 ]; then
              echo "❌ Validation failed for $file"
              exit 1
            else
              echo "✅ $file is valid"
            fi
          done
          
      - name: Generate Report
        run: |
          ddex-parser batch-validate ddex/ --format json > validation-report.json
          
      - name: Upload Report
        uses: actions/upload-artifact@v3
        with:
          name: ddex-validation-report
          path: validation-report.json
```

### Automated Release Processing

```yaml
name: Process DDEX Releases
on:
  schedule:
    - cron: '0 2 * * *'  # Daily at 2 AM
  workflow_dispatch:

jobs:
  process-releases:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Python
        uses: actions/setup-python@v4
        with:
          python-version: '3.11'
          
      - name: Install dependencies
        run: |
          pip install ddex-parser ddex-builder pandas
          
      - name: Process New Releases
        env:
          DATABASE_URL: ${{ secrets.DATABASE_URL }}
        run: |
          python scripts/process_daily_releases.py
          
      - name: Deploy Updates
        if: success()
        run: |
          python scripts/deploy_processed_releases.py
```

## Apache Airflow

### DDEX Processing DAG

```python
from airflow import DAG
from airflow.operators.python import PythonOperator
from airflow.providers.postgres.operators.postgres import PostgresOperator
from datetime import datetime, timedelta
import pandas as pd
from ddex_parser import DDEXParser
from ddex_builder import DDEXBuilder

default_args = {
    'owner': 'data-team',
    'depends_on_past': False,
    'start_date': datetime(2023, 1, 1),
    'email_on_failure': True,
    'email_on_retry': False,
    'retries': 2,
    'retry_delay': timedelta(minutes=5)
}

dag = DAG(
    'ddex_processing',
    default_args=default_args,
    description='Process DDEX files daily',
    schedule_interval='@daily',
    catchup=False,
    tags=['ddex', 'etl']
)

def extract_ddex_files(**context):
    """Extract new DDEX files from input directory"""
    import os
    from pathlib import Path
    
    input_dir = Path('/data/ddex/incoming')
    processed_dir = Path('/data/ddex/processed')
    
    new_files = []
    for xml_file in input_dir.glob('*.xml'):
        if not (processed_dir / xml_file.name).exists():
            new_files.append(str(xml_file))
    
    # Store file list in XCom
    context['task_instance'].xcom_push(key='new_files', value=new_files)
    return len(new_files)

def parse_ddex_files(**context):
    """Parse DDEX files and extract data"""
    parser = DDEXParser()
    file_list = context['task_instance'].xcom_pull(key='new_files')
    
    parsed_data = []
    for file_path in file_list:
        try:
            with open(file_path, 'r') as f:
                result = parser.parse(f.read())
            
            # Convert to DataFrame format
            df = parser.to_dataframe(f.read())
            parsed_data.append({
                'file_path': file_path,
                'data': df.to_dict('records'),
                'metadata': {
                    'version': result.version,
                    'processing_time_ms': result.processing_time_ms
                }
            })
            
        except Exception as e:
            print(f"Error parsing {file_path}: {e}")
            # Log error but continue processing
    
    context['task_instance'].xcom_push(key='parsed_data', value=parsed_data)
    return len(parsed_data)

def load_to_database(**context):
    """Load parsed data to database"""
    import psycopg2
    from sqlalchemy import create_engine
    
    parsed_data = context['task_instance'].xcom_pull(key='parsed_data')
    engine = create_engine(os.environ['DATABASE_URL'])
    
    for item in parsed_data:
        df = pd.DataFrame(item['data'])
        
        # Load to database
        df.to_sql('ddex_releases', engine, if_exists='append', index=False)
        
        # Mark file as processed
        processed_path = f"/data/ddex/processed/{Path(item['file_path']).name}"
        shutil.move(item['file_path'], processed_path)

def generate_reports(**context):
    """Generate daily processing reports"""
    import matplotlib.pyplot as plt
    
    parsed_data = context['task_instance'].xcom_pull(key='parsed_data')
    
    # Generate summary report
    report = {
        'date': context['ds'],
        'files_processed': len(parsed_data),
        'total_releases': sum(len(item['data']) for item in parsed_data),
        'processing_times': [item['metadata']['processing_time_ms'] for item in parsed_data]
    }
    
    # Create visualization
    fig, ax = plt.subplots(figsize=(10, 6))
    ax.hist(report['processing_times'], bins=20)
    ax.set_xlabel('Processing Time (ms)')
    ax.set_ylabel('Count')
    ax.set_title(f'DDEX Processing Times - {context["ds"]}')
    plt.savefig(f'/data/reports/processing_times_{context["ds"]}.png')
    
    return report

# Define tasks
extract_task = PythonOperator(
    task_id='extract_ddex_files',
    python_callable=extract_ddex_files,
    dag=dag
)

parse_task = PythonOperator(
    task_id='parse_ddex_files',
    python_callable=parse_ddex_files,
    dag=dag
)

load_task = PythonOperator(
    task_id='load_to_database',
    python_callable=load_to_database,
    dag=dag
)

report_task = PythonOperator(
    task_id='generate_reports',
    python_callable=generate_reports,
    dag=dag
)

# Create table if not exists
create_table = PostgresOperator(
    task_id='create_table',
    postgres_conn_id='ddex_postgres',
    sql="""
    CREATE TABLE IF NOT EXISTS ddex_releases (
        id SERIAL PRIMARY KEY,
        ddex_id VARCHAR(255),
        title VARCHAR(500),
        artist VARCHAR(500),
        release_date DATE,
        processed_at TIMESTAMP DEFAULT NOW()
    );
    """,
    dag=dag
)

# Set dependencies
create_table >> extract_task >> parse_task >> load_task >> report_task
```

## Jenkins Pipeline

```groovy
pipeline {
    agent any
    
    parameters {
        choice(
            name: 'VALIDATION_LEVEL',
            choices: ['basic', 'standard', 'strict'],
            description: 'DDEX validation level'
        )
        booleanParam(
            name: 'DEPLOY_AFTER_PROCESSING',
            defaultValue: false,
            description: 'Deploy processed files automatically'
        )
    }
    
    environment {
        DDEX_INPUT_DIR = '/data/ddex/incoming'
        DDEX_OUTPUT_DIR = '/data/ddex/processed'
        DATABASE_URL = credentials('database-url')
    }
    
    stages {
        stage('Setup') {
            steps {
                script {
                    sh 'npm install -g ddex-parser ddex-builder'
                    sh 'pip install ddex-parser ddex-builder pandas'
                }
            }
        }
        
        stage('Discover Files') {
            steps {
                script {
                    def files = sh(
                        script: "find ${DDEX_INPUT_DIR} -name '*.xml' -type f",
                        returnStdout: true
                    ).trim().split('\n')
                    
                    env.FILE_COUNT = files.size().toString()
                    writeFile file: 'file_list.txt', text: files.join('\n')
                }
            }
        }
        
        stage('Validate DDEX Files') {
            parallel {
                stage('Schema Validation') {
                    steps {
                        script {
                            sh """
                                while IFS= read -r file; do
                                    echo "Validating schema for \$file"
                                    ddex-parser validate "\$file" --level ${params.VALIDATION_LEVEL}
                                done < file_list.txt
                            """
                        }
                    }
                }
                
                stage('Content Validation') {
                    steps {
                        script {
                            sh """
                                python3 << 'EOF'
from ddex_parser import DDEXParser
import sys

parser = DDEXParser()
failed_files = []

with open('file_list.txt', 'r') as f:
    for line in f:
        file_path = line.strip()
        if not file_path:
            continue
            
        try:
            with open(file_path, 'r') as xml_file:
                result = parser.parse(xml_file.read())
                print(f"✅ Content valid: {file_path}")
        except Exception as e:
            print(f"❌ Content validation failed: {file_path} - {e}")
            failed_files.append(file_path)

if failed_files:
    print(f"\\nFailed files: {len(failed_files)}")
    sys.exit(1)
else:
    print(f"\\nAll files passed content validation")
EOF
                            """
                        }
                    }
                }
            }
        }
        
        stage('Process Files') {
            steps {
                script {
                    sh """
                        python3 << 'EOF'
import os
import json
from pathlib import Path
from ddex_parser import DDEXParser
from ddex_builder import DDEXBuilder

parser = DDEXParser()
builder = DDEXBuilder()

input_dir = Path("${DDEX_INPUT_DIR}")
output_dir = Path("${DDEX_OUTPUT_DIR}")
output_dir.mkdir(exist_ok=True)

processing_results = []

with open('file_list.txt', 'r') as f:
    for line in f:
        file_path = line.strip()
        if not file_path:
            continue
            
        try:
            # Parse
            with open(file_path, 'r') as xml_file:
                content = xml_file.read()
                result = parser.parse(content)
            
            # Convert to flattened format
            flat_data = result.flat.to_dict()
            
            # Save processed data
            output_file = output_dir / f"{Path(file_path).stem}_processed.json"
            with open(output_file, 'w') as f:
                json.dump(flat_data, f, indent=2, default=str)
            
            processing_results.append({
                'input_file': file_path,
                'output_file': str(output_file),
                'status': 'success',
                'releases_count': len(result.flat.releases),
                'tracks_count': len(result.flat.tracks)
            })
            
            print(f"✅ Processed: {file_path}")
            
        except Exception as e:
            processing_results.append({
                'input_file': file_path,
                'status': 'error',
                'error': str(e)
            })
            print(f"❌ Failed to process: {file_path} - {e}")

# Save processing summary
with open('processing_summary.json', 'w') as f:
    json.dump(processing_results, f, indent=2)

print(f"\\nProcessing complete. Results saved to processing_summary.json")
EOF
                    """
                }
            }
        }
        
        stage('Generate Reports') {
            steps {
                script {
                    sh """
                        python3 << 'EOF'
import json
import matplotlib.pyplot as plt
from datetime import datetime

# Load processing results
with open('processing_summary.json', 'r') as f:
    results = json.load(f)

# Generate summary statistics
total_files = len(results)
successful = len([r for r in results if r['status'] == 'success'])
failed = len([r for r in results if r['status'] == 'error'])

total_releases = sum(r.get('releases_count', 0) for r in results if r['status'] == 'success')
total_tracks = sum(r.get('tracks_count', 0) for r in results if r['status'] == 'success')

# Create summary report
summary = {
    'date': datetime.now().isoformat(),
    'total_files': total_files,
    'successful_files': successful,
    'failed_files': failed,
    'success_rate': f"{(successful/total_files*100):.1f}%" if total_files > 0 else "0%",
    'total_releases': total_releases,
    'total_tracks': total_tracks
}

print("\\nProcessing Summary:")
for key, value in summary.items():
    print(f"{key}: {value}")

# Save summary
with open('build_summary.json', 'w') as f:
    json.dump(summary, f, indent=2)
EOF
                    """
                }
            }
        }
        
        stage('Deploy') {
            when {
                expression { params.DEPLOY_AFTER_PROCESSING }
            }
            steps {
                script {
                    sh """
                        # Deploy processed files to production
                        rsync -av ${DDEX_OUTPUT_DIR}/ production:/data/ddex/current/
                        
                        # Update database with processing results
                        python3 scripts/update_database.py processing_summary.json
                        
                        # Notify stakeholders
                        curl -X POST "${SLACK_WEBHOOK_URL}" \\
                             -H 'Content-type: application/json' \\
                             --data '{"text":"DDEX processing completed. Processed ${FILE_COUNT} files."}'
                    """
                }
            }
        }
    }
    
    post {
        always {
            archiveArtifacts artifacts: '*.json, *.png', fingerprint: true
            
            publishHTML([
                allowMissing: false,
                alwaysLinkToLastBuild: true,
                keepAll: true,
                reportDir: '.',
                reportFiles: 'build_summary.json',
                reportName: 'DDEX Processing Report'
            ])
        }
        
        failure {
            emailext (
                subject: "DDEX Processing Pipeline Failed - Build ${BUILD_NUMBER}",
                body: """
                The DDEX processing pipeline has failed.
                
                Build: ${BUILD_URL}
                Files processed: ${env.FILE_COUNT}
                
                Please check the logs for details.
                """,
                to: "${env.NOTIFICATION_EMAIL}"
            )
        }
        
        success {
            script {
                def summary = readJSON file: 'build_summary.json'
                slackSend(
                    channel: '#ddex-processing',
                    color: 'good',
                    message: """
                    ✅ DDEX Processing Complete
                    • Files: ${summary.successful_files}/${summary.total_files} successful
                    • Releases: ${summary.total_releases}
                    • Tracks: ${summary.total_tracks}
                    • Success Rate: ${summary.success_rate}
                    """
                )
            }
        }
    }
}
```

## Azure DevOps Pipeline

```yaml
trigger:
  branches:
    include:
    - main
  paths:
    include:
    - ddex/**/*.xml

pool:
  vmImage: 'ubuntu-latest'

variables:
  - group: ddex-processing
  - name: DDEX_INPUT_PATH
    value: '$(System.DefaultWorkingDirectory)/ddex/incoming'
  - name: DDEX_OUTPUT_PATH
    value: '$(System.DefaultWorkingDirectory)/ddex/processed'

stages:
- stage: Validate
  displayName: 'Validate DDEX Files'
  jobs:
  - job: ValidateFiles
    displayName: 'Validate DDEX Files'
    steps:
    - task: NodeTool@0
      inputs:
        versionSpec: '18.x'
    
    - script: |
        npm install -g ddex-parser ddex-builder
      displayName: 'Install DDEX Suite'
    
    - script: |
        find $(DDEX_INPUT_PATH) -name "*.xml" -type f | while read file; do
          echo "Validating $file"
          ddex-parser validate "$file" --strict
          if [ $? -ne 0 ]; then
            echo "##vso[task.logissue type=error]Validation failed for $file"
            exit 1
          fi
        done
      displayName: 'Run DDEX Validation'

- stage: Process
  displayName: 'Process DDEX Files'
  dependsOn: Validate
  condition: succeeded()
  jobs:
  - job: ProcessFiles
    displayName: 'Process DDEX Files'
    steps:
    - task: UsePythonVersion@0
      inputs:
        versionSpec: '3.11'
    
    - script: |
        pip install ddex-parser ddex-builder pandas
      displayName: 'Install Python dependencies'
    
    - script: |
        python << 'EOF'
        import os
        import json
        from pathlib import Path
        from ddex_parser import DDEXParser
        
        parser = DDEXParser()
        input_path = Path("$(DDEX_INPUT_PATH)")
        output_path = Path("$(DDEX_OUTPUT_PATH)")
        output_path.mkdir(exist_ok=True)
        
        results = []
        
        for xml_file in input_path.glob("*.xml"):
            try:
                with open(xml_file, 'r') as f:
                    content = f.read()
                    result = parser.parse(content)
                
                # Save as JSON
                output_file = output_path / f"{xml_file.stem}_processed.json"
                with open(output_file, 'w') as f:
                    json.dump(result.flat.to_dict(), f, indent=2, default=str)
                
                results.append({
                    'file': str(xml_file),
                    'status': 'success',
                    'output': str(output_file)
                })
                
                print(f"✅ Processed: {xml_file}")
                
            except Exception as e:
                results.append({
                    'file': str(xml_file),
                    'status': 'error',
                    'error': str(e)
                })
                print(f"❌ Failed: {xml_file} - {e}")
        
        # Save results summary
        with open('processing_results.json', 'w') as f:
            json.dump(results, f, indent=2)
        
        print(f"Processing complete. {len(results)} files processed.")
        EOF
      displayName: 'Process DDEX Files'
    
    - task: PublishBuildArtifacts@1
      inputs:
        pathToPublish: '$(DDEX_OUTPUT_PATH)'
        artifactName: 'processed-ddex'
      displayName: 'Publish Processed Files'
    
    - task: PublishBuildArtifacts@1
      inputs:
        pathToPublish: 'processing_results.json'
        artifactName: 'processing-results'
      displayName: 'Publish Processing Results'
```

## AWS Step Functions

```json
{
  "Comment": "DDEX Processing Workflow",
  "StartAt": "ValidateInput",
  "States": {
    "ValidateInput": {
      "Type": "Task",
      "Resource": "arn:aws:states:::lambda:invoke",
      "Parameters": {
        "FunctionName": "ddex-validate-files",
        "Payload.$": "$"
      },
      "Next": "ProcessFiles",
      "Catch": [
        {
          "ErrorEquals": ["States.TaskFailed"],
          "Next": "HandleValidationError"
        }
      ]
    },
    "ProcessFiles": {
      "Type": "Map",
      "ItemsPath": "$.validFiles",
      "MaxConcurrency": 5,
      "Iterator": {
        "StartAt": "ParseDDEX",
        "States": {
          "ParseDDEX": {
            "Type": "Task",
            "Resource": "arn:aws:states:::lambda:invoke",
            "Parameters": {
              "FunctionName": "ddex-parse-file",
              "Payload.$": "$"
            },
            "Next": "SaveToDatabase"
          },
          "SaveToDatabase": {
            "Type": "Task",
            "Resource": "arn:aws:states:::dynamodb:putItem",
            "Parameters": {
              "TableName": "ddex-releases",
              "Item.$": "$.parsedData"
            },
            "End": true
          }
        }
      },
      "Next": "GenerateReport"
    },
    "GenerateReport": {
      "Type": "Task",
      "Resource": "arn:aws:states:::lambda:invoke",
      "Parameters": {
        "FunctionName": "ddex-generate-report",
        "Payload.$": "$"
      },
      "Next": "NotifyComplete"
    },
    "NotifyComplete": {
      "Type": "Task",
      "Resource": "arn:aws:states:::sns:publish",
      "Parameters": {
        "TopicArn": "arn:aws:sns:us-east-1:123456789012:ddex-processing-complete",
        "Message.$": "$.reportSummary"
      },
      "End": true
    },
    "HandleValidationError": {
      "Type": "Task",
      "Resource": "arn:aws:states:::sns:publish",
      "Parameters": {
        "TopicArn": "arn:aws:sns:us-east-1:123456789012:ddex-processing-error",
        "Message": "DDEX validation failed"
      },
      "End": true
    }
  }
}
```

## Monitoring and Alerting

### Prometheus Metrics

```python
from prometheus_client import Counter, Histogram, Gauge, start_http_server
from ddex_parser import DDEXParser
import time

# Metrics
ddex_files_processed = Counter('ddex_files_processed_total', 'Total DDEX files processed', ['status'])
ddex_processing_duration = Histogram('ddex_processing_duration_seconds', 'DDEX processing duration')
ddex_files_in_queue = Gauge('ddex_files_in_queue', 'Number of DDEX files in processing queue')

class MonitoredDDEXProcessor:
    def __init__(self):
        self.parser = DDEXParser()
    
    @ddex_processing_duration.time()
    def process_file(self, file_path):
        try:
            with open(file_path, 'r') as f:
                result = self.parser.parse(f.read())
            
            ddex_files_processed.labels(status='success').inc()
            return result
            
        except Exception as e:
            ddex_files_processed.labels(status='error').inc()
            raise e

# Start metrics server
start_http_server(8000)
```

### Grafana Dashboard

```json
{
  "dashboard": {
    "title": "DDEX Processing Dashboard",
    "panels": [
      {
        "title": "Files Processed",
        "type": "stat",
        "targets": [
          {
            "expr": "rate(ddex_files_processed_total[5m])",
            "legendFormat": "Files/sec"
          }
        ]
      },
      {
        "title": "Processing Duration",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, ddex_processing_duration_seconds_bucket)",
            "legendFormat": "95th percentile"
          }
        ]
      },
      {
        "title": "Queue Size",
        "type": "graph",
        "targets": [
          {
            "expr": "ddex_files_in_queue",
            "legendFormat": "Queue size"
          }
        ]
      }
    ]
  }
}
```

## Best Practices

1. **Error Handling**: Implement comprehensive error handling and recovery
2. **Monitoring**: Add metrics and alerts for processing failures
3. **Scalability**: Design for horizontal scaling with queue-based processing
4. **Idempotency**: Ensure processing can be safely retried
5. **Validation**: Always validate inputs before processing
6. **Logging**: Implement structured logging for debugging
7. **Testing**: Include automated testing in your pipelines
8. **Security**: Secure credentials and API keys properly
9. **Documentation**: Document your automation workflows
10. **Backup**: Implement backup strategies for processed data