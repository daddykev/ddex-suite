# Security

Security considerations and best practices for DDEX Suite implementations.

## Overview

DDEX Suite includes built-in security features:
- XXE (XML External Entity) attack prevention
- Memory-bounded parsing for DoS protection
- Input validation and sanitization
- Secure credential handling
- Audit logging capabilities

## XML Security

### XXE Protection

```typescript
import { DDEXParser, SecurityConfig } from 'ddex-parser';

const parser = new DDEXParser({
  security: {
    // Disable XML external entities
    disableXXE: true,
    
    // Disable DTD processing
    disableDTD: true,
    
    // Entity expansion limits
    maxEntityExpansions: 100,
    maxEntityDepth: 10,
    
    // Memory protection
    maxDocumentSize: 50 * 1024 * 1024, // 50MB
    maxElementDepth: 100
  }
});

// Safe parsing with protection
try {
  const result = await parser.parse(xmlContent);
  console.log('Safely parsed DDEX content');
} catch (error) {
  if (error.code === 'XXE_ATTEMPT') {
    console.error('Blocked XXE attack attempt');
  }
}
```

### Input Validation

```python
from ddex_parser import DDEXParser, SecurityError
import re

class SecureParser:
    def __init__(self):
        self.parser = DDEXParser()
        self.content_validators = [
            self._validate_xml_structure,
            self._validate_content_size,
            self._validate_encoding,
            self._check_suspicious_patterns
        ]
    
    def parse_safely(self, xml_content: str, source: str = "unknown"):
        """Parse XML with comprehensive security validation"""
        
        # Pre-processing validation
        for validator in self.content_validators:
            validator(xml_content, source)
        
        # Parse with security controls
        try:
            result = self.parser.parse(xml_content, security_mode='strict')
            
            # Post-processing validation
            self._validate_parsed_data(result)
            
            # Log successful parsing
            self._audit_log('parse_success', {
                'source': source,
                'size': len(xml_content),
                'releases': len(result.flat.releases)
            })
            
            return result
            
        except SecurityError as e:
            self._audit_log('security_violation', {
                'source': source,
                'violation': e.violation_type,
                'message': str(e)
            })
            raise
    
    def _validate_xml_structure(self, content: str, source: str):
        """Validate basic XML structure"""
        if not content.strip().startswith('<?xml'):
            raise SecurityError('Invalid XML structure', 'MALFORMED_XML')
        
        # Check for suspicious DOCTYPE declarations
        if 'DOCTYPE' in content and 'ENTITY' in content:
            raise SecurityError('Suspicious DOCTYPE with entities', 'SUSPICIOUS_DOCTYPE')
    
    def _validate_content_size(self, content: str, source: str):
        """Validate content size limits"""
        max_size = 100 * 1024 * 1024  # 100MB
        if len(content) > max_size:
            raise SecurityError(f'Content exceeds maximum size ({max_size})', 'CONTENT_TOO_LARGE')
    
    def _validate_encoding(self, content: str, source: str):
        """Validate character encoding"""
        try:
            content.encode('utf-8')
        except UnicodeEncodeError:
            raise SecurityError('Invalid character encoding', 'INVALID_ENCODING')
    
    def _check_suspicious_patterns(self, content: str, source: str):
        """Check for suspicious patterns that might indicate attacks"""
        suspicious_patterns = [
            r'ENTITY\s+\w+\s+SYSTEM',  # External entity references
            r'file://',                 # File system access
            r'http://[^>]+/etc/',      # Suspicious HTTP URLs
            r'<script',                # Script injection
            r'javascript:',            # JavaScript URLs
        ]
        
        for pattern in suspicious_patterns:
            if re.search(pattern, content, re.IGNORECASE):
                raise SecurityError(f'Suspicious pattern detected: {pattern}', 'SUSPICIOUS_CONTENT')
    
    def _audit_log(self, event: str, details: dict):
        """Log security events for auditing"""
        import json
        from datetime import datetime
        
        log_entry = {
            'timestamp': datetime.utcnow().isoformat(),
            'event': event,
            'details': details
        }
        
        # In production, send to secure logging service
        print(f"SECURITY_AUDIT: {json.dumps(log_entry)}")
```

## Credential Security

### Secure Configuration

```typescript
import { SecretManager } from '@google-cloud/secret-manager';
import { DDEXBuilder } from 'ddex-builder';

export class SecureConfigManager {
  private secretClient: SecretManager;
  private cache: Map<string, any> = new Map();

  constructor() {
    this.secretClient = new SecretManager();
  }

  async getSecureConfig(configName: string): Promise<any> {
    // Check cache first (with TTL)
    if (this.cache.has(configName)) {
      const cached = this.cache.get(configName);
      if (Date.now() - cached.timestamp < 300000) { // 5 minute TTL
        return cached.value;
      }
      this.cache.delete(configName);
    }

    try {
      const [version] = await this.secretClient.accessSecretVersion({
        name: `projects/YOUR_PROJECT/secrets/${configName}/versions/latest`
      });

      const payload = version.payload?.data?.toString();
      if (!payload) {
        throw new Error('Secret payload is empty');
      }

      const config = JSON.parse(payload);
      
      // Cache with timestamp
      this.cache.set(configName, {
        value: config,
        timestamp: Date.now()
      });

      return config;

    } catch (error) {
      console.error(`Failed to retrieve secret ${configName}:`, error);
      throw new Error('Configuration unavailable');
    }
  }

  async createSecureDDEXBuilder(): Promise<DDEXBuilder> {
    const config = await this.getSecureConfig('ddex-builder-config');
    
    return new DDEXBuilder({
      // Use secure configuration
      apiKeys: config.apiKeys,
      encryptionKey: config.encryptionKey,
      signingKey: config.signingKey,
      
      // Security settings
      security: {
        validateInputs: true,
        sanitizeOutputs: true,
        enableAuditLogging: true
      }
    });
  }

  clearCache(): void {
    this.cache.clear();
  }
}

// Usage with environment-specific secrets
const configManager = new SecureConfigManager();
const builder = await configManager.createSecureDDEXBuilder();
```

### Data Encryption

```python
from cryptography.fernet import Fernet
from cryptography.hazmat.primitives import hashes
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
import base64
import os

class DDEXEncryption:
    def __init__(self, password: str):
        self.password = password.encode()
        self.salt = os.urandom(16)
        
    def _derive_key(self, salt: bytes = None) -> bytes:
        """Derive encryption key from password"""
        if salt is None:
            salt = self.salt
            
        kdf = PBKDF2HMAC(
            algorithm=hashes.SHA256(),
            length=32,
            salt=salt,
            iterations=100000,
        )
        return base64.urlsafe_b64encode(kdf.derive(self.password))
    
    def encrypt_ddex_data(self, data: str) -> str:
        """Encrypt DDEX XML data"""
        key = self._derive_key()
        fernet = Fernet(key)
        
        encrypted = fernet.encrypt(data.encode())
        
        # Return salt + encrypted data (base64 encoded)
        return base64.b64encode(self.salt + encrypted).decode()
    
    def decrypt_ddex_data(self, encrypted_data: str) -> str:
        """Decrypt DDEX XML data"""
        try:
            # Decode and separate salt from encrypted data
            combined = base64.b64decode(encrypted_data.encode())
            salt = combined[:16]
            encrypted = combined[16:]
            
            # Derive key with original salt
            key = self._derive_key(salt)
            fernet = Fernet(key)
            
            decrypted = fernet.decrypt(encrypted)
            return decrypted.decode()
            
        except Exception as e:
            raise SecurityError(f"Failed to decrypt data: {e}")

# Secure storage of DDEX data
class SecureDDEXStorage:
    def __init__(self, encryption_key: str):
        self.encryption = DDEXEncryption(encryption_key)
    
    def store_ddex(self, xml_content: str, identifier: str) -> str:
        """Store encrypted DDEX data"""
        encrypted = self.encryption.encrypt_ddex_data(xml_content)
        
        # Store in secure database/file system
        storage_key = self._generate_storage_key(identifier)
        self._store_encrypted(storage_key, encrypted)
        
        return storage_key
    
    def retrieve_ddex(self, storage_key: str) -> str:
        """Retrieve and decrypt DDEX data"""
        encrypted = self._retrieve_encrypted(storage_key)
        return self.encryption.decrypt_ddex_data(encrypted)
```

## Access Control

### Role-Based Access Control (RBAC)

```typescript
export enum Permission {
  READ_DDEX = 'ddex:read',
  WRITE_DDEX = 'ddex:write',
  PARSE_DDEX = 'ddex:parse',
  BUILD_DDEX = 'ddex:build',
  ADMIN_DDEX = 'ddex:admin'
}

export interface User {
  id: string;
  email: string;
  roles: string[];
  permissions: Permission[];
}

export class AccessController {
  private rolePermissions: Map<string, Permission[]> = new Map([
    ['viewer', [Permission.READ_DDEX, Permission.PARSE_DDEX]],
    ['editor', [Permission.READ_DDEX, Permission.WRITE_DDEX, Permission.PARSE_DDEX, Permission.BUILD_DDEX]],
    ['admin', [Permission.READ_DDEX, Permission.WRITE_DDEX, Permission.PARSE_DDEX, Permission.BUILD_DDEX, Permission.ADMIN_DDEX]]
  ]);

  hasPermission(user: User, permission: Permission): boolean {
    // Check direct permissions
    if (user.permissions.includes(permission)) {
      return true;
    }

    // Check role-based permissions
    for (const role of user.roles) {
      const rolePerms = this.rolePermissions.get(role) || [];
      if (rolePerms.includes(permission)) {
        return true;
      }
    }

    return false;
  }

  requirePermission(user: User, permission: Permission): void {
    if (!this.hasPermission(user, permission)) {
      throw new SecurityError(`User ${user.id} lacks permission: ${permission}`, 'INSUFFICIENT_PERMISSIONS');
    }
  }
}

// Secure DDEX service with access control
export class SecureDDEXService {
  private parser: DDEXParser;
  private builder: DDEXBuilder;
  private accessControl: AccessController;

  constructor() {
    this.parser = new DDEXParser();
    this.builder = new DDEXBuilder();
    this.accessControl = new AccessController();
  }

  async parseSecure(xmlContent: string, user: User): Promise<any> {
    // Check permissions
    this.accessControl.requirePermission(user, Permission.PARSE_DDEX);
    
    // Audit log
    this.auditLog('parse_attempt', { userId: user.id, contentSize: xmlContent.length });
    
    try {
      const result = await this.parser.parse(xmlContent);
      
      this.auditLog('parse_success', { userId: user.id, releases: result.flat.releases.length });
      return result;
      
    } catch (error) {
      this.auditLog('parse_failure', { userId: user.id, error: error.message });
      throw error;
    }
  }

  async buildSecure(data: any, user: User, options: any = {}): Promise<string> {
    this.accessControl.requirePermission(user, Permission.BUILD_DDEX);
    
    // Additional validation for sensitive operations
    if (options.partnerPreset) {
      this.accessControl.requirePermission(user, Permission.ADMIN_DDEX);
    }
    
    this.auditLog('build_attempt', { userId: user.id, partner: options.partnerPreset });
    
    try {
      const xml = await this.builder.build(data, options);
      
      this.auditLog('build_success', { userId: user.id, outputSize: xml.length });
      return xml;
      
    } catch (error) {
      this.auditLog('build_failure', { userId: user.id, error: error.message });
      throw error;
    }
  }

  private auditLog(event: string, details: any): void {
    const logEntry = {
      timestamp: new Date().toISOString(),
      event,
      details,
      service: 'ddex-suite'
    };
    
    // Send to secure audit logging service
    console.log(`AUDIT: ${JSON.stringify(logEntry)}`);
  }
}
```

## Security Monitoring

```python
import hashlib
import hmac
from datetime import datetime, timedelta
from collections import defaultdict

class SecurityMonitor:
    def __init__(self, alert_threshold: int = 5, time_window: int = 300):
        self.alert_threshold = alert_threshold
        self.time_window = time_window  # seconds
        self.suspicious_activity = defaultdict(list)
        self.blocked_ips = set()
    
    def monitor_request(self, request_info: dict):
        """Monitor incoming requests for suspicious activity"""
        
        client_ip = request_info.get('client_ip')
        user_id = request_info.get('user_id')
        operation = request_info.get('operation')
        
        # Rate limiting
        self._check_rate_limit(client_ip, operation)
        
        # Pattern detection
        self._detect_suspicious_patterns(request_info)
        
        # User behavior analysis
        if user_id:
            self._analyze_user_behavior(user_id, request_info)
    
    def _check_rate_limit(self, client_ip: str, operation: str):
        """Check for excessive request rates"""
        now = datetime.utcnow()
        cutoff = now - timedelta(seconds=self.time_window)
        
        # Clean old entries
        key = f"{client_ip}:{operation}"
        self.suspicious_activity[key] = [
            timestamp for timestamp in self.suspicious_activity[key]
            if timestamp > cutoff
        ]
        
        # Add current request
        self.suspicious_activity[key].append(now)
        
        # Check threshold
        if len(self.suspicious_activity[key]) > self.alert_threshold:
            self._trigger_alert(f"Rate limit exceeded for {client_ip} on {operation}")
            self.blocked_ips.add(client_ip)
    
    def _detect_suspicious_patterns(self, request_info: dict):
        """Detect suspicious request patterns"""
        content = request_info.get('content', '')
        
        suspicious_indicators = [
            'ENTITY',
            'file://',
            'http://localhost',
            '<script',
            '../../',  # Path traversal
            'eval(',
            'javascript:'
        ]
        
        for indicator in suspicious_indicators:
            if indicator in content:
                self._trigger_alert(f"Suspicious content pattern detected: {indicator}")
                break
    
    def _analyze_user_behavior(self, user_id: str, request_info: dict):
        """Analyze user behavior for anomalies"""
        
        # Detect unusual operation patterns
        operation = request_info.get('operation')
        now = datetime.utcnow()
        
        user_key = f"user:{user_id}"
        if user_key not in self.suspicious_activity:
            self.suspicious_activity[user_key] = []
        
        self.suspicious_activity[user_key].append({
            'timestamp': now,
            'operation': operation,
            'ip': request_info.get('client_ip')
        })
        
        # Keep only recent activity
        cutoff = now - timedelta(hours=1)
        self.suspicious_activity[user_key] = [
            activity for activity in self.suspicious_activity[user_key]
            if activity['timestamp'] > cutoff
        ]
        
        # Analyze patterns
        recent_activity = self.suspicious_activity[user_key]
        
        # Check for unusual operation frequency
        operation_count = len([a for a in recent_activity if a['operation'] == operation])
        if operation_count > 50:  # More than 50 operations per hour
            self._trigger_alert(f"Unusual activity frequency for user {user_id}: {operation}")
        
        # Check for IP switching
        unique_ips = len(set(a['ip'] for a in recent_activity))
        if unique_ips > 5:  # More than 5 different IPs in an hour
            self._trigger_alert(f"Multiple IP addresses for user {user_id}")
    
    def _trigger_alert(self, message: str):
        """Trigger security alert"""
        alert = {
            'timestamp': datetime.utcnow().isoformat(),
            'level': 'WARNING',
            'message': message,
            'service': 'ddex-security-monitor'
        }
        
        # In production, send to security team/SIEM
        print(f"SECURITY_ALERT: {alert}")
    
    def is_blocked(self, client_ip: str) -> bool:
        """Check if IP is blocked"""
        return client_ip in self.blocked_ips
    
    def unblock_ip(self, client_ip: str):
        """Manually unblock an IP"""
        self.blocked_ips.discard(client_ip)

# Integration with DDEX service
security_monitor = SecurityMonitor()

def secure_endpoint(request):
    request_info = {
        'client_ip': request.remote_addr,
        'user_id': request.user.id if request.user else None,
        'operation': request.endpoint,
        'content': request.get_data(as_text=True)
    }
    
    # Check if IP is blocked
    if security_monitor.is_blocked(request_info['client_ip']):
        return {'error': 'Access denied'}, 403
    
    # Monitor the request
    security_monitor.monitor_request(request_info)
    
    # Process request normally
    return process_ddex_request(request)
```

## Best Practices

1. **Input Validation**: Always validate XML input before processing
2. **XXE Prevention**: Disable external entity processing
3. **Access Control**: Implement proper authentication and authorization
4. **Encryption**: Encrypt sensitive data at rest and in transit
5. **Audit Logging**: Log all security-relevant events
6. **Rate Limiting**: Implement rate limiting to prevent DoS attacks
7. **Regular Updates**: Keep dependencies updated with security patches
8. **Secure Configuration**: Store sensitive configuration securely
9. **Monitoring**: Implement real-time security monitoring
10. **Incident Response**: Have a plan for security incidents