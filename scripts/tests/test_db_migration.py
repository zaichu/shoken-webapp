import importlib.util
import os
from pathlib import Path
import subprocess
import tempfile
import time
import unittest
import uuid


ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "scripts/db_migration.py"


class MigrationTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.container = "migration-test-" + uuid.uuid4().hex
        subprocess.run(
            [
                "docker",
                "run",
                "--detach",
                "--rm",
                "--name",
                cls.container,
                "--publish",
                "127.0.0.1::5432",
                "--env",
                "POSTGRES_HOST_AUTH_METHOD=trust",
                "postgres:16-alpine",
            ],
            check=True,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        cls.addClassCleanup(
            subprocess.run,
            ["docker", "rm", "--force", cls.container],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        port = (
            subprocess.run(
                ["docker", "port", cls.container, "5432/tcp"],
                check=True,
                capture_output=True,
                text=True,
            )
            .stdout.strip()
            .rsplit(":", 1)[1]
        )
        cls.base_env = {
            **os.environ,
            "PGHOST": "127.0.0.1",
            "PGPORT": port,
            "PGUSER": "postgres",
            "PGDATABASE": "postgres",
            "PGCONNECT_TIMEOUT": "2",
        }
        for _ in range(60):
            result = subprocess.run(
                ["psql", "-XwAt", "-c", "SELECT 1"],
                env=cls.base_env,
                capture_output=True,
            )
            if result.returncode == 0:
                break
            time.sleep(0.5)
        else:
            raise AssertionError("Docker PostgreSQL の起動に失敗")

    def setUp(self):
        self.assertTrue(SCRIPT.exists(), "データ移行スクリプトが必要")
        spec = importlib.util.spec_from_file_location("db_migration", SCRIPT)
        self.migration = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(self.migration)
        self.folder = tempfile.TemporaryDirectory(dir=ROOT)
        self.addCleanup(self.folder.cleanup)
        self.archive = Path(self.folder.name) / "data.sql"
        self.urls = {}
        for side in ("source", "target"):
            name = side + uuid.uuid4().hex
            self.sql("postgres", 'CREATE DATABASE "' + name + '"')
            self.urls[side] = (
                "postgresql://postgres@127.0.0.1:"
                + self.base_env["PGPORT"]
                + "/"
                + name
            )
            self.migration.prepare(self.urls[side])
            self.sql(self.urls[side], "CREATE SEQUENCE migration_test_seq START 7")
        self.source = self.urls["source"]
        self.target = self.urls["target"]

    def sql(self, database, sql):
        env = (
            self.migration.client_env(database)
            if "://" in database
            else {**self.base_env, "PGDATABASE": database}
        )
        result = subprocess.run(
            ["psql", "-XwqAt", "-v", "ON_ERROR_STOP=1"],
            input=sql,
            env=env,
            capture_output=True,
            text=True,
        )
        self.assertEqual(result.returncode, 0, "テスト用 SQL の実行に失敗")
        return result.stdout.strip()

    def seed(self):
        self.sql(
            self.source, (ROOT / "scripts/tests/db_migration_fixture.sql").read_text()
        )

    def test_roundtrip_all_tables_keys_rows_views_and_sequence(self):
        self.seed()
        self.migration.export_data(self.source, self.archive)
        self.assertEqual(self.archive.stat().st_mode & 0o777, 0o600)
        checkpoint = self.migration.import_data(self.source, self.target, self.archive)
        self.migration.verify(self.source, self.target, checkpoint)
        tables = self.sql(
            self.source,
            """
            SELECT tablename FROM pg_tables WHERE schemaname = 'public' ORDER BY 1;
        """,
        ).splitlines()
        self.assertEqual(len(tables), 15)
        for table in tables:
            where = (
                " WHERE expires_at > '" + checkpoint + "'"
                if table == "sessions"
                else ""
            )
            source_rows = self.sql(
                self.source,
                'COPY (SELECT * FROM public."' + table + '"' + where + ") TO STDOUT",
            )
            target_rows = self.sql(
                self.target, 'COPY (SELECT * FROM public."' + table + '") TO STDOUT'
            )
            self.assertTrue(source_rows, "全テーブルに架空データが必要: " + table)
            self.assertEqual(
                sorted(source_rows.splitlines()),
                sorted(target_rows.splitlines()),
                table,
            )
        self.assertEqual(self.sql(self.target, "SELECT count(*) FROM sessions"), "1")
        self.assertEqual(
            self.sql(self.target, "SELECT count(*) FROM csv_import_history"), "2"
        )
        for view in ("asset_balances_view", "fund_balances_view"):
            self.assertEqual(
                self.sql(self.source, "SELECT count(*) FROM " + view),
                self.sql(self.target, "SELECT count(*) FROM " + view),
            )
        self.assertEqual(
            self.sql(self.source, "SELECT nextval('migration_test_seq')"),
            self.sql(self.target, "SELECT nextval('migration_test_seq')"),
        )
        with self.assertRaises(self.migration.MigrationError):
            self.migration.import_data(self.source, self.target, self.archive)

    def test_import_failure_rolls_back_seed_replacement(self):
        self.seed()
        self.migration.export_data(self.source, self.archive)
        before = self.sql(self.target, "COPY fund_codes TO STDOUT")
        with self.archive.open("a") as archive:
            archive.write("\nSELECT 1 / 0;\n")
        with self.assertRaises(self.migration.MigrationError):
            self.migration.import_data(self.source, self.target, self.archive)
        self.assertEqual(self.sql(self.target, "SELECT count(*) FROM users"), "0")
        self.assertEqual(self.sql(self.target, "COPY fund_codes TO STDOUT"), before)

    def test_verify_detects_equal_count_with_different_key(self):
        self.seed()
        self.migration.export_data(self.source, self.archive)
        checkpoint = self.migration.import_data(self.source, self.target, self.archive)
        self.sql(self.target, "UPDATE stock SET code = '9998'")
        with self.assertRaises(self.migration.MigrationError):
            self.migration.verify(self.source, self.target, checkpoint)

    def test_command_failure_does_not_disclose_environment(self):
        marker = uuid.uuid4().hex
        env = {
            **os.environ,
            "SOURCE_DATABASE_URL": self.source + "?sslmode=" + marker,
            "TARGET_DATABASE_URL": self.target + "?sslmode=" + marker,
        }
        for action in ("export", "prepare"):
            result = subprocess.run(
                ["python3", str(SCRIPT), action, str(self.archive)],
                env=env,
                capture_output=True,
                text=True,
            )
            self.assertNotEqual(result.returncode, 0)
            self.assertNotIn(marker, result.stdout + result.stderr)
        self.assertFalse(self.archive.exists())

    def test_export_does_not_overwrite_existing_file(self):
        self.archive.write_text("existing")
        with self.assertRaises(self.migration.MigrationError):
            self.migration.export_data(self.source, self.archive)
        self.assertEqual(self.archive.read_text(), "existing")

    def test_import_rejects_migration_mismatch_before_writing(self):
        self.seed()
        self.migration.export_data(self.source, self.archive)
        self.sql(
            self.source,
            "UPDATE _sqlx_migrations SET checksum = decode('00', 'hex') WHERE version = 17",
        )
        with self.assertRaises(self.migration.MigrationError):
            self.migration.import_data(self.source, self.target, self.archive)
        self.assertEqual(self.sql(self.target, "SELECT count(*) FROM users"), "0")

    def test_verify_detects_data_change_with_same_keys(self):
        self.seed()
        self.migration.export_data(self.source, self.archive)
        checkpoint = self.migration.import_data(self.source, self.target, self.archive)
        self.sql(self.target, "UPDATE dividends SET net_amount_received = 1")
        with self.assertRaises(self.migration.MigrationError):
            self.migration.verify(self.source, self.target, checkpoint)


if __name__ == "__main__":
    unittest.main()
