"""Regression checks for the A03 source-level model boundary guards."""

import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest


CHECKER = Path(__file__).resolve().parents[1] / "scripts/check-module-boundaries.py"


class ModelBoundaryTests(unittest.TestCase):
    def setUp(self) -> None:
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        self.root = Path(directory.name)
        (self.root / "scripts").mkdir()
        (self.root / "src").mkdir()
        shutil.copyfile(CHECKER, self.root / "scripts/check-module-boundaries.py")
        for module in ("exchange", "jose", "encryption"):
            (self.root / f"src/{module}.rs").write_text("\n")

    def check_source(self, source: str) -> tuple[subprocess.CompletedProcess[str], int]:
        (self.root / "src/model.rs").write_text(source + "\n")
        graph_path = self.root / "graph.json"
        result = subprocess.run(
            [sys.executable, str(self.root / "scripts/check-module-boundaries.py"),
             "--json", str(graph_path)],
            cwd=self.root,
            capture_output=True,
            text=True,
            check=False,
        )
        graph = json.loads(graph_path.read_text())
        return result, graph["boundary_checks"]["model_legacy_client_config_reference_files"]

    def test_direct_and_grouped_legacy_types_fail(self) -> None:
        cases = [
            ("type Alias = crate::jose::ClientAuthMethod;", "jose::ClientAuthMethod"),
            ("type Alias = crate::jose::MachineTrust;", "jose::MachineTrust"),
            ("type Alias = crate::exchange::ExchangePolicy;", "exchange::ExchangePolicy"),
            ("type Alias = crate::encryption::EncryptionKey;", "encryption::EncryptionKey"),
            ("use crate::jose::{ClientAuthMethod};", "jose::ClientAuthMethod"),
            ("use crate::jose::{MachineTrust};", "jose::MachineTrust"),
            ("use crate::exchange::{ExchangePolicy};", "exchange::ExchangePolicy"),
            ("use crate::encryption::{EncryptionKey};", "encryption::EncryptionKey"),
            ("use crate::{jose::{ClientAuthMethod}};", "jose::ClientAuthMethod"),
            ("use crate::{jose::{MachineTrust}};", "jose::MachineTrust"),
            ("use crate::{exchange::{ExchangePolicy}};", "exchange::ExchangePolicy"),
            ("use crate::{encryption::{EncryptionKey}};", "encryption::EncryptionKey"),
            (
                "use crate::{\n exchange::{ExchangeGrant, ExchangePolicy as Policy},\n model::Client,\n};",
                "exchange::ExchangePolicy",
            ),
            (
                "use crate::exchange::{ExchangeGrant, // existing grant\n ExchangePolicy};",
                "exchange::ExchangePolicy",
            ),
            (
                "use crate::{exchange::ExchangeGrant, // comment\n exchange::ExchangePolicy};",
                "exchange::ExchangePolicy",
            ),
            (
                'const URL: &str = "https://example.test/a//b";\n'
                'use crate::exchange::{ExchangeGrant, // existing grant\n ExchangePolicy};',
                "exchange::ExchangePolicy",
            ),
            (
                'const URL: &str = r#"https://example.test/a//b"#;\n'
                'use crate::{exchange::ExchangeGrant, // comment\n exchange::ExchangePolicy};',
                "exchange::ExchangePolicy",
            ),
        ]
        for source, expected in cases:
            with self.subTest(source=source):
                result, count = self.check_source(source)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn("legacy client configuration types", result.stderr)
                self.assertIn(expected, result.stderr)
                self.assertEqual(count, 1)

    def test_embedded_record_adapter_paths_fail(self) -> None:
        cases = [
            "type Alias = crate::exchange::ExchangeGrant;",
            "type Alias = crate::authenticator::TotpSettings;",
            "type Alias = crate::claims::Policy;",
            "type Alias = crate::assurance::ClaimsRequest;",
            "type Alias = crate::source::SourceIdentity;",
            "use crate::exchange::{ExchangeGrant};",
            "use crate::{exchange::{ExchangeGrant}};",
            "use crate::{exchange::{ExchangeGrant}, model::client_config::ExchangePolicy};",
            "use crate::exchange::{ExchangeGrant, // crate::exchange::ExchangePolicy\n};",
            "use crate::{exchange::ExchangeGrant, // crate::exchange::ExchangePolicy\n};",
            'const SAMPLE: &str = "crate::exchange::ExchangePolicy // literal";\n'
            'use crate::exchange::ExchangeGrant;',
            'const SAMPLE: &str = r#"crate::exchange::ExchangePolicy // literal"#;\n'
            'use crate::exchange::ExchangeGrant;',
        ]
        for source in cases:
            with self.subTest(source=source):
                result, count = self.check_source(source)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn("model refers to protocol", result.stderr)
                self.assertEqual(count, 0)

    def test_shared_model_paths_pass(self) -> None:
        cases = [
            "use crate::model::client_config::{ClientAuthMethod, MachineTrust, ExchangePolicy, EncryptionKey};",
            "use crate::{model::{client_config::{ClientAuthMethod, MachineTrust, ExchangePolicy, EncryptionKey}}};",
            "use crate::model::exchange::ExchangeGrant;",
            "use crate::model::claims::{ClaimMapping, Policy};",
        ]
        for source in cases:
            with self.subTest(source=source):
                result, count = self.check_source(source)
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual(count, 0)


if __name__ == "__main__":
    unittest.main()
