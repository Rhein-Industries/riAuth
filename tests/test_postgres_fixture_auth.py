"""Selected-function checks; PostgreSQL and product processes are always fake."""

import ast
import copy
import json
import os
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest import mock


ROOT = Path(__file__).resolve().parents[1]
SCRIPTS = (
    ("recovery-drill-postgres.py", "run", "private_file"),
    ("recovery-drill-native-postgres.py", "main", "private_text"),
)
PASSWORD = "fixture-only-" + "c7" * 24


def selected_source(filename, parent, private_writer, run_child):
    """Define only the reviewed file, environment, and nested client functions."""
    tree = ast.parse((ROOT / "scripts" / filename).read_text())
    owner = next(node for node in tree.body
                 if isinstance(node, ast.FunctionDef) and node.name == parent)
    definitions = [node for node in tree.body if isinstance(node, ast.FunctionDef)
                   and node.name in ("require", private_writer, "redact", "postgres_env")]
    definitions += [node for node in ast.walk(owner) if isinstance(node, ast.FunctionDef)
                    and node.name in ("pg", "sql", "pg_config")]
    namespace = {"os": os, "json": json,
                 "subprocess": SimpleNamespace(run=run_child),
                 "pg_bin": Path("/fixture-only-pg"), "pg_port": 54321,
                 "secrets": SimpleNamespace(token_hex=lambda _n: PASSWORD,
                                            token_urlsafe=lambda _n: "fixture-only-user-password"),
                 "sensitive": []}
    module = ast.Module(body=copy.deepcopy(definitions), type_ignores=[])
    exec(compile(ast.fix_missing_locations(module), filename, "exec"), namespace)
    return owner, namespace


def evaluate_nodes(nodes, namespace, filename):
    module = ast.Module(body=copy.deepcopy(nodes), type_ignores=[])
    exec(compile(ast.fix_missing_locations(module), filename, "exec"), namespace)


def calls_named(node, name):
    return [call for call in ast.walk(node) if isinstance(call, ast.Call)
            and isinstance(call.func, ast.Name) and call.func.id == name]


class PostgresFixtureAuth(unittest.TestCase):
    def test_private_password_registration_initdb_and_connection(self):
        for filename, parent, writer in SCRIPTS:
            with self.subTest(script=filename), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                calls = []
                namespace = None

                def fake_child(argv, **options):
                    calls.append((argv, options))
                    self.assertIn(PASSWORD, namespace["sensitive"])
                    self.assertTrue(all(PASSWORD not in str(argument) for argument in argv))
                    if Path(argv[0]).name == "initdb":
                        self.assertIn("--auth=scram-sha-256", argv)
                        self.assertNotIn("--auth=trust", argv)
                        pwfile = Path(argv[argv.index("--pwfile") + 1])
                        self.assertEqual(pwfile.read_text(), PASSWORD + "\n")
                        self.assertEqual(pwfile.stat().st_mode & 0o777, 0o600)
                    return SimpleNamespace(returncode=0, stdout="", stderr="")

                owner, namespace = selected_source(filename, parent, writer, fake_child)
                namespace.update(root=root, pg_data=root / "postgres", source=root / "source")
                body = owner.body
                if parent == "run":
                    body = next(node for node in body if isinstance(node, ast.With)).body
                seed_names = {"admin_password", "user_password", "pg_password", "password_file"}
                seed = [node for node in body if
                        (isinstance(node, ast.Assign) and any(
                            isinstance(target, ast.Name) and target.id in seed_names
                            for target in node.targets)) or
                        (isinstance(node, ast.Expr) and isinstance(node.value, ast.Call) and
                         isinstance(node.value.func, ast.Attribute) and
                         isinstance(node.value.func.value, ast.Name) and
                         node.value.func.value.id == "sensitive") or
                        (isinstance(node, ast.Expr) and isinstance(node.value, ast.Call) and
                         isinstance(node.value.func, ast.Name) and node.value.func.id == writer)]
                evaluate_nodes(seed, namespace, filename)
                attempt = next(node for node in body if isinstance(node, ast.Try))
                init = next(node for node in attempt.body
                            if any(call.args and isinstance(call.args[0], ast.Constant)
                                   and call.args[0].value == "initdb"
                                   for call in calls_named(node, "pg")))
                if parent == "main":
                    write_password = next(node for node in attempt.body
                                          if calls_named(node, "private_text")
                                          and isinstance(calls_named(node, "private_text")[0].args[0], ast.Name)
                                          and calls_named(node, "private_text")[0].args[0].id == "password_file")
                    evaluate_nodes([write_password], namespace, filename)
                evaluate_nodes([init], namespace, filename)
                self.assertEqual(len(calls), 1)
                self.assertEqual(calls[0][1]["env"]["PGPASSWORD"], PASSWORD)
                if parent == "run":
                    namespace["pg_config"]("riauth_source")
                    namespace["pg_config"]("riauth_target")
                    connections = [root / "riauth_source.connection", root / "riauth_target.connection"]
                else:
                    assignment = next(node for node in attempt.body
                                      if isinstance(node, ast.Assign) and
                                      any(isinstance(target, ast.Name) and target.id == "connection"
                                          for target in node.targets))
                    connection_write = next(node for node in attempt.body
                                            if calls_named(node, "private_text") and
                                            isinstance(calls_named(node, "private_text")[0].args[0], ast.Name) and
                                            calls_named(node, "private_text")[0].args[0].id == "connection")
                    evaluate_nodes([assignment, connection_write], namespace, filename)
                    connections = [root / "connection"]
                self.assertEqual(root.stat().st_mode & 0o777, 0o700)
                for connection in connections:
                    self.assertIn("password=" + PASSWORD, connection.read_text())
                    self.assertIn("sslmode=disable connect_timeout=3", connection.read_text())
                    self.assertEqual(connection.stat().st_mode & 0o777, 0o600)
                    with self.assertRaises(FileExistsError):
                        namespace[writer](connection, b"replacement" if writer == "private_file" else "replacement")
                    self.assertIn("password=" + PASSWORD, connection.read_text())

    def test_controlled_clients_and_basebackup_share_private_auth(self):
        inherited = {"PATH": "/fixture-only-path", "PGPASSWORD": "inherited-password",
                     "PGHOST": "foreign-host", "PGSERVICE": "foreign-service",
                     "PGOPTIONS": "foreign-options", "PGPASSFILE": "/foreign-passfile",
                     "PSQLRC": "/foreign-psqlrc"}
        for filename, parent, writer in SCRIPTS:
            with self.subTest(script=filename), mock.patch.dict(os.environ, inherited, clear=True):
                calls = []

                def fake_child(argv, **options):
                    calls.append((argv, options))
                    return SimpleNamespace(returncode=0, stdout="fixture-result\n", stderr="")

                owner, namespace = selected_source(filename, parent, writer, fake_child)
                self.assertEqual(namespace["postgres_env"](), {"PATH": inherited["PATH"]})
                namespace.update(pg_password=PASSWORD, sensitive=[PASSWORD],
                                 occupied=Path("/fixture-only-occupied"), copy=Path("/fixture-only-copy"))
                if parent == "run":
                    self.assertEqual(namespace["sql"]("riauth_source", "select 1"), "fixture-result")
                    self.assertEqual(namespace["sql"]("riauth_target", "select 1"), "fixture-result")
                else:
                    self.assertEqual(namespace["sql"]("select 1"), "fixture-result")
                    self.assertEqual(namespace["sql"]("select 1", "empty_schema"), "fixture-result")
                    attempt = next(node for node in owner.body if isinstance(node, ast.Try))
                    backups = [node for node in attempt.body if any(
                        call.args and isinstance(call.args[0], ast.Constant) and
                        call.args[0].value == "pg_basebackup" for call in calls_named(node, "pg"))]
                    self.assertEqual(len(backups), 2)
                    evaluate_nodes(backups, namespace, filename)
                    self.assertEqual([call[0][call[0].index("-D") + 1] for call in calls[2:]],
                                     ["/fixture-only-occupied", "/fixture-only-copy"])
                for argv, options in calls:
                    self.assertIn("--no-password", argv)
                    self.assertTrue(all(PASSWORD not in str(argument) for argument in argv))
                    self.assertEqual(options["env"], {"PATH": inherited["PATH"], "PGPASSWORD": PASSWORD})

    def test_failure_and_refusal_captures_do_not_return_raw_secret_tails(self):
        for filename, parent, writer in SCRIPTS:
            with self.subTest(script=filename):
                def fake_child(_argv, **_options):
                    return SimpleNamespace(returncode=17, stdout="retained-output",
                                           stderr="prefix:" + PASSWORD + "x" * 290)

                _, namespace = selected_source(filename, parent, writer, fake_child)
                namespace.update(pg_password=PASSWORD, sensitive=[PASSWORD])
                with self.assertRaises(RuntimeError) as caught:
                    namespace["pg"]("createdb", "--no-password", "fixture-db")
                self.assertIn("17", str(caught.exception))
                self.assertNotIn(PASSWORD, str(caught.exception))
                self.assertNotIn("prefix:", str(caught.exception))
                options = {"allow_failure": True} if parent == "run" else {"ok": False}
                result = namespace["pg"]("pg_ctl", "status", **options)
                self.assertEqual(result.returncode, 17)
                self.assertEqual(result.stdout, "retained-output")
                self.assertNotIn(PASSWORD, result.stderr)
                self.assertIn("[redacted]", result.stderr)
                message = namespace["redact"]("failure:" + PASSWORD + "y" * 600, [PASSWORD])[:500]
                self.assertNotIn(PASSWORD, message)
                self.assertIn("[redacted]", message)

    def test_actual_durable_failure_fields_redact_before_retention(self):
        for filename, parent, writer in SCRIPTS:
            with self.subTest(script=filename):
                tree = ast.parse((ROOT / "scripts" / filename).read_text())
                _, namespace = selected_source(filename, parent, writer,
                                               lambda *_args, **_options: self.fail("no child call expected"))
                namespace.update(error=RuntimeError("failure:" + PASSWORD + "x" * 600),
                                 sensitive=[PASSWORD], evidence={}, report={})
                main = next(node for node in tree.body
                            if isinstance(node, ast.FunctionDef) and node.name == "main")
                fields = [node for node in ast.walk(main) if isinstance(node, ast.Assign) and
                          any(isinstance(target, ast.Subscript) and
                              isinstance(target.value, ast.Name) and target.value.id in ("evidence", "report") and
                              isinstance(target.slice, ast.Constant) and
                              target.slice.value in ("failure", "cleanup_error") for target in node.targets)]
                self.assertEqual(len(fields), 1 if parent == "run" else 3)
                evaluate_nodes(fields, namespace, filename)
                encoded = json.dumps(namespace["evidence"] if parent == "run" else namespace["report"])
                self.assertNotIn(PASSWORD, encoded)
                self.assertIn("[redacted]", encoded)
                self.assertNotIn(PASSWORD[-10:], encoded)


if __name__ == "__main__":
    unittest.main()
