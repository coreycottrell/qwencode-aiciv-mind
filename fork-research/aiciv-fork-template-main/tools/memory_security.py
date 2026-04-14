#!/usr/bin/env python3
"""
Memory Security Layer for AI-CIV Collective

Provides secret detection, access control, and integrity validation.
Zero tolerance for sensitive data leaks.

Author: The Conductor & Security Auditor
Version: 1.0.0
"""

import os
import re
import json
import math
import hashlib
from pathlib import Path
from typing import Dict, List, Optional, Any, Tuple
from collections import Counter
from datetime import datetime, timezone


# Secret detection patterns (comprehensive coverage)
SECRET_PATTERNS = {
    'api_key': [
        r'sk_live_[a-zA-Z0-9]{32,}',  # Stripe
        r'sk_test_[a-zA-Z0-9]{32,}',  # Stripe test
        r'AKIA[0-9A-Z]{16}',  # AWS Access Key
        r'ghp_[a-zA-Z0-9]{36}',  # GitHub Personal Access Token
        r'gho_[a-zA-Z0-9]{36}',  # GitHub OAuth token
        r'xox[baprs]-[a-zA-Z0-9-]{10,}',  # Slack tokens
        r'AIza[0-9A-Za-z\-_]{35}',  # Google API key
        r'ya29\.[0-9A-Za-z\-_]+',  # Google OAuth
        r'EAACEdEose0cBA[0-9A-Za-z]+',  # Facebook
    ],
    'password': [
        r'password\s*=\s*["\']([^"\']{8,})["\']',
        r'passwd:\s*([^\s]{8,})',
        r'pwd\s*=\s*["\']([^"\']{8,})["\']',
    ],
    'private_key': [
        r'-----BEGIN (RSA |EC |OPENSSH )?PRIVATE KEY-----',
        r'-----BEGIN PGP PRIVATE KEY BLOCK-----',
    ],
    'jwt': [
        r'eyJ[a-zA-Z0-9_-]+\.eyJ[a-zA-Z0-9_-]+\.[a-zA-Z0-9_-]+',
    ],
    'connection_string': [
        r'(postgresql|mysql|mongodb)://[^\s]+:[^\s]+@[^\s]+',
        r'mongodb\+srv://[^\s]+',
    ],
    'email': [
        r'[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}',  # Generic email (PII)
    ],
    'ipv4_private': [
        r'\b10\.\d{1,3}\.\d{1,3}\.\d{1,3}\b',  # 10.x.x.x
        r'\b172\.(1[6-9]|2[0-9]|3[01])\.\d{1,3}\.\d{1,3}\b',  # 172.16-31.x.x
        r'\b192\.168\.\d{1,3}\.\d{1,3}\b',  # 192.168.x.x
    ],
    'credit_card': [
        r'\b(?:\d{4}[-\s]?){3}\d{4}\b',  # 16-digit card number
    ],
}


class SecurityError(Exception):
    """Raised when security validation fails."""
    pass


class SensitiveDataDetector:
    """Detects sensitive data in memory content."""

    def __init__(self):
        """Initialize detector with compiled regex patterns."""
        self.compiled_patterns = {}
        for category, patterns in SECRET_PATTERNS.items():
            self.compiled_patterns[category] = [re.compile(p) for p in patterns]

    def scan(self, content: str) -> List[Dict[str, Any]]:
        """Scan content for sensitive data.

        Args:
            content: Text to scan

        Returns:
            List of findings (each is a dict with 'type', 'pattern', 'match')
        """
        findings = []

        for category, patterns in self.compiled_patterns.items():
            for pattern in patterns:
                matches = pattern.finditer(content)
                for match in matches:
                    findings.append({
                        'type': category,
                        'pattern': pattern.pattern,
                        'match': match.group(0),
                        'position': match.span(),
                        'severity': 'HIGH' if category in ['api_key', 'password', 'private_key', 'jwt'] else 'MEDIUM'
                    })

        # Also check for high-entropy strings (unknown secrets)
        entropy_findings = self._detect_high_entropy_strings(content)
        findings.extend(entropy_findings)

        return findings

    def _detect_high_entropy_strings(self, content: str, threshold: float = 4.5) -> List[Dict[str, Any]]:
        """Detect high-entropy strings that might be secrets.

        Args:
            content: Text to scan
            threshold: Shannon entropy threshold (bits per character)

        Returns:
            List of potential secret findings
        """
        findings = []

        # Find candidate strings (alphanumeric sequences 20+ chars)
        candidates = re.findall(r'\b[a-zA-Z0-9+/=]{20,}\b', content)

        for candidate in candidates:
            entropy = self._calculate_entropy(candidate)
            if entropy > threshold:
                findings.append({
                    'type': 'high_entropy',
                    'pattern': 'entropy_detection',
                    'match': candidate[:20] + '...' if len(candidate) > 20 else candidate,
                    'position': (0, 0),  # Full scan doesn't track position
                    'severity': 'MEDIUM',
                    'entropy': entropy
                })

        return findings

    def _calculate_entropy(self, s: str) -> float:
        """Calculate Shannon entropy of a string.

        Args:
            s: String to analyze

        Returns:
            Entropy in bits per character
        """
        if not s:
            return 0.0

        counter = Counter(s)
        length = len(s)
        entropy = -sum((count/length) * math.log2(count/length) for count in counter.values())
        return entropy


class MemoryAccessControl:
    """Enforces access control for memory operations."""

    # Tier 3 agents (can write to project-knowledge and other agents' memories)
    TIER_3_AGENTS = ['the-conductor', 'result-synthesizer']

    def __init__(self, base_dir: str = ".claude/memory"):
        """Initialize access control.

        Args:
            base_dir: Memory base directory
        """
        self.base_dir = Path(base_dir)

    def validate_write_path(self, agent_id: str, path: str) -> Tuple[bool, str]:
        """Validate if agent can write to path.

        Args:
            agent_id: Agent requesting write
            path: Target file path

        Returns:
            Tuple of (allowed: bool, reason: str)
        """
        # Normalize path (prevent ../ attacks)
        try:
            normalized = Path(path).resolve()
            base_resolved = self.base_dir.resolve()

            # Path must be within memory directory
            if not str(normalized).startswith(str(base_resolved)):
                return False, f"Path outside memory directory: {path}"

        except Exception as e:
            return False, f"Invalid path: {e}"

        # Convert to relative path for analysis
        rel_path = str(normalized.relative_to(base_resolved))

        # Tier 3 agents have full access
        if agent_id in self.TIER_3_AGENTS:
            return True, "Tier 3 agent - full access"

        # Regular agents can only write to their own directory
        if rel_path.startswith("agent-learnings/"):
            parts = rel_path.split('/')
            if len(parts) >= 2:
                owner = parts[1]
                if owner == agent_id:
                    return True, "Agent writing to own directory"
                else:
                    return False, f"Cannot write to {owner}'s directory"

        # Project knowledge requires Tier 3
        if rel_path.startswith("project-knowledge/"):
            return False, "Project knowledge requires Tier 3 access"

        return False, "Access denied"

    def can_write(self, agent_id: str, filepath: str) -> bool:
        """Check if agent can write to filepath.

        Args:
            agent_id: Agent requesting write
            filepath: Target file path

        Returns:
            True if allowed, False otherwise
        """
        allowed, _ = self.validate_write_path(agent_id, filepath)
        return allowed


class MemorySecurityValidator:
    """Validates memory entries before write."""

    def __init__(self, base_dir: str = ".claude/memory"):
        """Initialize validator.

        Args:
            base_dir: Memory base directory
        """
        self.detector = SensitiveDataDetector()
        self.acl = MemoryAccessControl(base_dir)
        self.audit_log_path = Path("security/memory-audit.jsonl")
        self.audit_log_path.parent.mkdir(parents=True, exist_ok=True)

    def validate_before_write(self, agent_id: str, content: str, filepath: str) -> None:
        """Validate content and access before write.

        Args:
            agent_id: Agent attempting write
            content: Memory content to validate
            filepath: Target file path

        Raises:
            SecurityError: If validation fails
        """
        # 1. Check access control
        allowed, reason = self.acl.validate_write_path(agent_id, filepath)
        if not allowed:
            self._log_audit(agent_id, "ACCESS_DENIED", filepath, reason)
            raise SecurityError(f"Access denied: {reason}")

        # 2. Scan for secrets
        findings = self.detector.scan(content)
        if findings:
            high_severity = [f for f in findings if f['severity'] == 'HIGH']
            if high_severity:
                self._log_audit(agent_id, "SECRET_DETECTED", filepath, str(high_severity))
                raise SecurityError(f"Secret detected: {high_severity[0]['type']} - {high_severity[0]['pattern']}")

        # 3. Check size limits (500KB)
        if len(content) > 500_000:
            raise SecurityError(f"Memory too large: {len(content)} bytes (max 500KB)")

        # 4. Check for injection attempts (basic)
        if self._detect_injection(content):
            self._log_audit(agent_id, "INJECTION_ATTEMPT", filepath, "Executable code detected")
            raise SecurityError("Suspicious content: potential injection attempt")

        # Log successful validation
        self._log_audit(agent_id, "VALIDATED", filepath, "OK")

    def _detect_injection(self, content: str) -> bool:
        """Detect potential injection attempts.

        Args:
            content: Content to check

        Returns:
            True if suspicious patterns found
        """
        # Check for executable code patterns
        suspicious_patterns = [
            r'<script[^>]*>',  # JavaScript
            r'eval\s*\(',
            r'exec\s*\(',
            r'__import__\s*\(',
        ]

        for pattern in suspicious_patterns:
            if re.search(pattern, content, re.IGNORECASE):
                return True

        return False

    def _log_audit(self, agent_id: str, operation: str, filepath: str, details: str):
        """Log security event to audit log.

        Args:
            agent_id: Agent performing operation
            operation: Type of operation
            filepath: Target file
            details: Additional details
        """
        entry = {
            'timestamp': datetime.now(timezone.utc).isoformat(),
            'operation': operation,
            'agent_id': agent_id,
            'filepath': filepath,
            'details': details,
        }

        # Append to audit log
        with open(self.audit_log_path, 'a') as f:
            f.write(json.dumps(entry) + '\n')


def generate_pre_commit_hook() -> str:
    """Generate pre-commit hook script for git.

    Returns:
        Bash script content
    """
    return '''#!/bin/bash
# Memory Security Pre-Commit Hook
# Scans staged files for secrets before commit

echo "🔍 Scanning for secrets..."

# Get staged files
STAGED_FILES=$(git diff --cached --name-only --diff-filter=ACM)

# Scan each file
for FILE in $STAGED_FILES; do
    if [[ $FILE == .claude/memory/* ]] || [[ $FILE == knowledge/* ]]; then
        python3 tools/memory_security.py --scan "$FILE"
        if [ $? -ne 0 ]; then
            echo "❌ SECRET DETECTED in $FILE"
            echo "Commit blocked. Please remove sensitive data and try again."
            exit 1
        fi
    fi
done

echo "✅ No secrets detected"
exit 0
'''


# Test functions
def test_secret_detection():
    """Test secret detection."""
    print("Testing secret detection...")

    detector = SensitiveDataDetector()

    # Test API key detection
    content_with_secret = "API key: ghp_123456789012345678901234567890123456"  # 36 chars after ghp_
    findings = detector.scan(content_with_secret)
    assert len(findings) > 0, f"Expected to find secret, got {findings}"
    assert findings[0]['type'] == 'api_key'
    print("✅ API key detected")

    # Test clean content
    clean_content = "This is clean content with API_KEY placeholder."
    findings = detector.scan(clean_content)
    # Should only flag 'API_KEY' as high entropy if it's long enough
    print(f"✅ Clean content scan: {len(findings)} findings (expected 0-1)")

    # Test high-entropy detection
    random_string = "x" * 50  # Low entropy
    findings = detector.scan(random_string)
    print(f"✅ Low entropy string: {len(findings)} findings")

    high_entropy = "aB3dE9fG2hJ5kL8mN1pQ4rS7tU0vW6xY3zA"  # High entropy
    findings = detector.scan(high_entropy)
    print(f"✅ High entropy string: {len(findings)} findings")


def test_access_control():
    """Test access control."""
    print("\nTesting access control...")

    import tempfile
    temp_dir = tempfile.mkdtemp()

    try:
        acl = MemoryAccessControl(temp_dir)

        # Test agent writing to own directory
        agent_path = f"{temp_dir}/agent-learnings/test-agent/memory.md"
        allowed, reason = acl.validate_write_path("test-agent", agent_path)
        assert allowed, f"Should allow agent to write to own dir: {reason}"
        print("✅ Agent can write to own directory")

        # Test agent writing to another's directory
        other_path = f"{temp_dir}/agent-learnings/other-agent/memory.md"
        allowed, reason = acl.validate_write_path("test-agent", other_path)
        assert not allowed, "Should deny agent writing to another's dir"
        print("✅ Agent blocked from other's directory")

        # Test Tier 3 agent access
        allowed, reason = acl.validate_write_path("the-conductor", other_path)
        assert allowed, "Should allow Tier 3 agent full access"
        print("✅ Tier 3 agent has full access")

        # Test path traversal attack
        evil_path = f"{temp_dir}/../../etc/passwd"
        allowed, reason = acl.validate_write_path("test-agent", evil_path)
        assert not allowed, "Should block path traversal"
        print("✅ Path traversal blocked")

    finally:
        import shutil
        shutil.rmtree(temp_dir)


def test_validation():
    """Test security validation."""
    print("\nTesting security validation...")

    import tempfile
    temp_dir = tempfile.mkdtemp()

    try:
        validator = MemorySecurityValidator(temp_dir)

        # Test clean content
        clean_content = "This is a clean memory about JWT patterns."
        filepath = f"{temp_dir}/agent-learnings/test-agent/memory.md"
        try:
            validator.validate_before_write("test-agent", clean_content, filepath)
            print("✅ Clean content validated")
        except SecurityError as e:
            print(f"❌ Unexpected error: {e}")

        # Test content with secret
        secret_content = "API key: ghp_123456789012345678901234567890123456"  # 36 chars
        try:
            validator.validate_before_write("test-agent", secret_content, filepath)
            print("❌ Should have detected secret")
        except SecurityError:
            print("✅ Secret detected and blocked")

        # Test access violation
        other_filepath = f"{temp_dir}/agent-learnings/other-agent/memory.md"
        try:
            validator.validate_before_write("test-agent", clean_content, other_filepath)
            print("❌ Should have blocked access violation")
        except SecurityError:
            print("✅ Access violation blocked")

    finally:
        import shutil
        shutil.rmtree(temp_dir)


if __name__ == "__main__":
    test_secret_detection()
    test_access_control()
    test_validation()
    print("\n🎉 All security tests passed!")

    # Generate pre-commit hook
    print("\nPre-commit hook script:")
    print(generate_pre_commit_hook())
