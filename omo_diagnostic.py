# omo_diagnostic.py — emit canonical JSON to stderr for steward capture
import json
import hashlib
import sys
import os
import time
from dataclasses import dataclass, asdict
from enum import IntEnum

class Severity(IntEnum):
    INFO = 0
    WARNING = 1
    ERROR = 2

class Category(IntEnum):
    TYPE = 1
    LOGIC = 2
    SECURITY = 4
    RECEIPT = 8
    IDENTITY = 16
    RHYTHM = 32

@dataclass
class Diagnostic:
    version: str = "1.0"
    language: str = "python"
    package: str = ""
    file: str = ""
    line: int = 0
    code: str = ""
    severity: Severity = Severity.ERROR
    category: Category = Category.LOGIC
    message: str = ""
    agent_id: str = ""
    birth_timestamp: int = 0
    tier: str = "apprentice"
    sabbath_active: bool = False
    repair_id: str = ""
    repair_strategy: str = "manual"
    
    def emit(self):
        """Emit canonical JSON to stderr for steward capture"""
        payload = {
            "version": self.version,
            "source": {
                "language": self.language,
                "package": self.package,
                "file": self.file,
                "line": self.line,
            },
            "diagnostic": {
                "code": self.code,
                "severity": self.severity.name.lower(),
                "category": self.category.name.lower(),
                "message": self.message,
                "context": {
                    "agent_id": self.agent_id,
                    "birth_timestamp": self.birth_timestamp,
                    "tier": self.tier,
                    "sabbath_active": self.sabbath_active,
                }
            },
            "repair": {
                "id": self.repair_id,
                "strategy": self.repair_strategy,
            } if self.repair_id else None,
            "audit_trail": {
                "zangbeto_verified": False,
                "timestamp": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
            }
        }
        print(json.dumps(payload), file=sys.stderr, flush=True)
        return payload

if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser(description="Emit diagnostic in OMO format")
    parser.add_argument("--package", default="")
    parser.add_argument("--file", default="")
    parser.add_argument("--line", type=int, default=0)
    parser.add_argument("--code", default="")
    parser.add_argument("--severity", default="error")
    parser.add_argument("--category", default="logic")
    parser.add_argument("--message", default="")
    parser.add_argument("--agent-id", default="")
    parser.add_argument("--repair-id", default="")
    parser.add_argument("--repair-strategy", default="manual")
    args = parser.parse_args()

    d = Diagnostic(
        package=args.package,
        file=args.file,
        line=args.line,
        code=args.code,
        severity=Severity[args.severity.upper()] if args.severity.upper() in Severity.__members__ else Severity.ERROR,
        category=Category[args.category.upper()] if args.category.upper() in Category.__members__ else Category.LOGIC,
        message=args.message,
        agent_id=args.agent_id,
        repair_id=args.repair_id,
        repair_strategy=args.repair_strategy,
    )
    d.emit()
