#!/usr/bin/env python3
import argparse
from datetime import datetime
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
from urllib.parse import parse_qsl, unquote, urlsplit


ROOT = Path(__file__).resolve().parents[1]
PSQL = ["psql", "-XwqAt", "-v", "ON_ERROR_STOP=1"]
SEED_TABLES = {"fund_codes", "market_data_provider_rate_control", "_sqlx_migrations"}


class MigrationError(Exception):
    pass


def client_env(url):
    parsed = urlsplit(url)
    if (
        parsed.scheme not in {"postgres", "postgresql"}
        or not parsed.hostname
        or not parsed.path[1:]
    ):
        raise MigrationError("接続環境変数には PostgreSQL URL が必要です。")
    env = {
        key: value
        for key, value in os.environ.items()
        if key not in {"SOURCE_DATABASE_URL", "TARGET_DATABASE_URL"}
        and not key.startswith("PG")
    }
    env.update(
        PGDATABASE=unquote(parsed.path[1:]),
        PGHOST=parsed.hostname,
        PGPORT=str(parsed.port or 5432),
        PGUSER=unquote(parsed.username or "postgres"),
        PGPASSWORD=unquote(parsed.password or ""),
        PGCONNECT_TIMEOUT="15",
        PGOPTIONS="-c timezone=UTC -c datestyle=ISO,YMD",
        PGCLIENTENCODING="UTF8",
    )
    parameters = {
        "sslmode": "PGSSLMODE",
        "channel_binding": "PGCHANNELBINDING",
        "sslrootcert": "PGSSLROOTCERT",
    }
    for key, value in parse_qsl(parsed.query, strict_parsing=True):
        if key not in parameters:
            raise MigrationError("未対応の接続パラメーターです。")
        env[parameters[key]] = value
    return env


def query(url, sql, options=()):
    result = subprocess.run(
        PSQL + list(options),
        input=sql.encode(),
        env=client_env(url),
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
    )
    if result.returncode:
        raise MigrationError(
            "DB 操作に失敗しました。接続設定と実行条件を確認してください。"
        )
    return result.stdout.decode().rstrip("\n")


def identifier(name):
    if not re.fullmatch(r"[a-z_][a-z_0-9]*", name):
        raise MigrationError("未対応の DB オブジェクト名です。")
    return '"' + name + '"'


def tables(url):
    rows = query(
        url,
        """
        SELECT json_build_object('name', c.relname, 'keys', ARRAY(
            SELECT a.attname FROM pg_index i
            CROSS JOIN LATERAL unnest(i.indkey) WITH ORDINALITY k(attnum, position)
            JOIN pg_attribute a ON a.attrelid = c.oid AND a.attnum = k.attnum
            WHERE i.indrelid = c.oid AND i.indisprimary ORDER BY k.position
        )) FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace
        WHERE n.nspname = 'public' AND c.relkind = 'r' ORDER BY c.relname;
    """,
    )
    result = [json.loads(row) for row in rows.splitlines()]
    if not result or any(not row["keys"] for row in result):
        raise MigrationError("全テーブルに主キーが必要です。")
    for row in result:
        identifier(row["name"])
        for key in row["keys"]:
            identifier(key)
    return result


def prepare(target):
    if (
        query(target, "SELECT count(*) FROM pg_tables WHERE schemaname = 'public'")
        != "0"
    ):
        raise MigrationError("prepare は public にテーブルのない DB 専用です。")
    statements = [
        """
        CREATE TABLE public._sqlx_migrations (
            version bigint PRIMARY KEY, description text NOT NULL,
            installed_on timestamptz NOT NULL DEFAULT now(), success boolean NOT NULL,
            checksum bytea NOT NULL, execution_time bigint NOT NULL
        );
    """
    ]
    metadata = []
    files = sorted((ROOT / "backend/migrations").glob("[0-9][0-9][0-9][0-9]_*.sql"))
    if [int(path.name[:4]) for path in files] != list(range(18)):
        raise MigrationError("0000〜0017 のマイグレーションを確認してください。")
    for path in files:
        content = path.read_bytes()
        version = int(path.name[:4])
        description = path.stem[5:].replace("_", " ")
        checksum = hashlib.sha384(content).hexdigest()
        statements.append(content.decode())
        if version == 0:
            statements.append("""
                ALTER EXTENSION citext SET SCHEMA public;
                ALTER EXTENSION pg_trgm SET SCHEMA public;
                ALTER EXTENSION pgcrypto SET SCHEMA public;
            """)
        metadata.append(
            {"version": version, "description": description, "checksum": checksum}
        )
    statements.append("""
        INSERT INTO public._sqlx_migrations
            (version, description, success, checksum, execution_time)
        SELECT version, description, true, decode(checksum, 'hex'), 0
        FROM json_to_recordset(:'migrations'::json)
            AS migration(version bigint, description text, checksum text);
    """)
    query(
        target,
        "\n".join(statements),
        ["--single-transaction", "-v", "migrations=" + json.dumps(metadata)],
    )


def export_data(source, archive):
    archive = Path(archive)
    try:
        with open(
            archive, "xb", opener=lambda path, flags: os.open(path, flags, 0o600)
        ) as output:
            result = subprocess.run(
                [
                    "pg_dump",
                    "--data-only",
                    "--schema=public",
                    "--no-owner",
                    "--no-privileges",
                ],
                env=client_env(source),
                stdout=output,
                stderr=subprocess.DEVNULL,
            )
            if result.returncode:
                raise MigrationError(
                    "書き出しに失敗しました。接続設定を確認してください。"
                )
    except FileExistsError:
        raise MigrationError(
            "書き出し先が存在します。新しいファイル名を指定してください。"
        ) from None
    except BaseException:
        archive.unlink(missing_ok=True)
        raise


def check_schema(source, target):
    source_tables = tables(source)
    if source_tables != tables(target):
        raise MigrationError("移行元と移行先のテーブル・主キーが一致しません。")
    sql = "SELECT version, encode(checksum, 'hex'), success FROM public._sqlx_migrations ORDER BY version"
    if query(source, sql) != query(target, sql):
        raise MigrationError("マイグレーション履歴が一致しません。移行を中止します。")
    return source_tables


def import_data(source, target, archive):
    metadata = check_schema(source, target)
    names = ["public." + identifier(row["name"]) for row in metadata]
    sql = "LOCK TABLE " + ", ".join(names) + " IN ACCESS EXCLUSIVE MODE;\n"
    for row in metadata:
        if row["name"] not in SEED_TABLES:
            sql += (
                "DO $guard$ BEGIN IF EXISTS (SELECT 1 FROM public."
                + identifier(row["name"])
                + ") THEN RAISE EXCEPTION 'target not empty'; "
                "END IF; END $guard$;\n"
            )
    sql += "TRUNCATE " + ", ".join(names) + ";\n"
    sql += Path(archive).read_text() + "\n"
    sql += "DELETE FROM public.sessions WHERE expires_at <= :'cutoff'::timestamptz;\n"
    checkpoint = query(target, "SELECT clock_timestamp()::text")
    query(target, sql, ["--single-transaction", "-v", "cutoff=" + checkpoint])
    return checkpoint


def verify(source, target, checkpoint):
    datetime.fromisoformat(checkpoint)
    metadata = check_schema(source, target)
    for row in metadata:
        table = "public." + identifier(row["name"])
        keys = ", ".join(identifier(key) for key in row["keys"])
        source_where = (
            " WHERE expires_at > :'cutoff'::timestamptz"
            if row["name"] == "sessions"
            else ""
        )
        options = ["-v", "cutoff=" + checkpoint]
        for columns in (keys, "*"):
            prefix = "COPY (SELECT " + columns + " FROM " + table
            suffix = " ORDER BY " + keys + ") TO STDOUT;"
            expected = query(source, prefix + source_where + suffix, options)
            actual = query(target, prefix + suffix)
            if sorted(expected.splitlines()) != sorted(actual.splitlines()):
                raise MigrationError(
                    "件数・主キー・データの比較が一致しません。付け替えは禁止です。"
                )
    sequences = (
        "SELECT sequencename FROM pg_sequences WHERE schemaname = 'public' ORDER BY 1"
    )
    source_sequences = query(source, sequences)
    if source_sequences != query(target, sequences):
        raise MigrationError("シーケンスの一覧が一致しません。")
    for name in source_sequences.splitlines():
        sql = "SELECT last_value, is_called FROM public." + identifier(name)
        if query(source, sql) != query(target, sql):
            raise MigrationError("シーケンスの現在値が一致しません。")


def main():
    parser = argparse.ArgumentParser(
        description="接続情報は環境変数からのみ読み取ります。"
    )
    parser.add_argument("action", choices=["prepare", "export", "import", "verify"])
    parser.add_argument("archive", nargs="?", type=Path)
    args = parser.parse_args()
    try:
        source = os.environ.get("SOURCE_DATABASE_URL")
        target = os.environ.get("TARGET_DATABASE_URL")
        if (args.action != "prepare" and not source) or (
            args.action != "export" and not target
        ):
            raise MigrationError("必要な接続環境変数が未設定です。")
        if args.action != "prepare" and args.archive is None:
            raise MigrationError("データファイルのパスを指定してください。")
        if args.action == "prepare":
            prepare(target)
        elif args.action == "export":
            export_data(source, args.archive)
        elif args.action == "import":
            checkpoint_file = args.archive.with_suffix(".checkpoint")
            with open(
                checkpoint_file,
                "x",
                opener=lambda path, flags: os.open(path, flags, 0o600),
            ) as output:
                try:
                    output.write(import_data(source, target, args.archive))
                except BaseException:
                    checkpoint_file.unlink(missing_ok=True)
                    raise
        else:
            verify(source, target, args.archive.with_suffix(".checkpoint").read_text())
        print("処理が完了しました。")
        return 0
    except (Exception, KeyboardInterrupt):
        print(
            "処理に失敗しました。接続設定・空の移行先・マイグレーション・手順を確認してください。",
            file=sys.stderr,
        )
        return 1


if __name__ == "__main__":
    sys.exit(main())
