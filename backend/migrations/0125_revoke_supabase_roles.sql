SET LOCAL lock_timeout = '5s';

-- 所有者を移した関数の付与者は新所有者に書き換わるため、移行ロールの直接の
-- REVOKE では外れない。所有者ごと(SET できる所有者になり代わって)に外す。
-- 0100 が所有ロールを CURRENT_USER へ SET 付きで所属させている。
-- 残りがあれば異常として止める(監視ログの件数が 0 にならないままにしない)。
DO $$
DECLARE
    target text;
    grantor text;
    grantor_oid oid;
    obj text;
    me oid;
    leftover integer;
    did_set boolean;
BEGIN
    SELECT oid INTO me FROM pg_catalog.pg_roles WHERE rolname = CURRENT_USER;
    FOREACH target IN ARRAY ARRAY['anon', 'authenticated', 'service_role'] LOOP
        IF EXISTS (SELECT 1 FROM pg_catalog.pg_roles WHERE rolname = target) THEN
            EXECUTE format('ALTER DEFAULT PRIVILEGES IN SCHEMA public REVOKE ALL ON TABLES FROM %I', target);
            EXECUTE format('ALTER DEFAULT PRIVILEGES IN SCHEMA public REVOKE ALL ON SEQUENCES FROM %I', target);
            EXECUTE format('ALTER DEFAULT PRIVILEGES IN SCHEMA public REVOKE ALL ON FUNCTIONS FROM %I', target);
        END IF;
    END LOOP;
    FOR grantor, grantor_oid IN
        SELECT o.rolname, o.oid
        FROM pg_catalog.pg_roles o
        WHERE o.oid = me
           OR (o.oid <> me AND pg_catalog.pg_has_role(CURRENT_USER, o.oid, 'SET')
               AND (EXISTS (SELECT 1 FROM pg_catalog.pg_class c
                            JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
                            WHERE c.relowner = o.oid AND n.nspname = 'public')
                    OR EXISTS (SELECT 1 FROM pg_catalog.pg_proc p
                               WHERE p.proowner = o.oid
                                 AND p.pronamespace = 'public'::pg_catalog.regnamespace)))
    LOOP
        BEGIN
            did_set := (grantor <> CURRENT_USER);
            IF did_set THEN
                EXECUTE format('SET ROLE %I', grantor);
            END IF;
            FOREACH target IN ARRAY ARRAY['anon', 'authenticated', 'service_role'] LOOP
                IF EXISTS (SELECT 1 FROM pg_catalog.pg_roles WHERE rolname = target) THEN
                    FOR obj IN
                        SELECT format('%I.%I', n.nspname, c.relname)
                        FROM pg_catalog.pg_class c
                        JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
                        WHERE n.nspname = 'public' AND c.relowner = grantor_oid
                          AND c.relkind IN ('r', 'p', 'v', 'm')
                    LOOP
                        EXECUTE format('REVOKE ALL ON TABLE %s FROM %I', obj, target);
                    END LOOP;
                    FOR obj IN
                        SELECT format('%I.%I', n.nspname, c.relname)
                        FROM pg_catalog.pg_class c
                        JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
                        WHERE n.nspname = 'public' AND c.relowner = grantor_oid
                          AND c.relkind = 'S'
                    LOOP
                        EXECUTE format('REVOKE ALL ON SEQUENCE %s FROM %I', obj, target);
                    END LOOP;
                    FOR obj IN
                        SELECT format(
                            '%I.%I(%s)',
                            'public',
                            p.proname,
                            pg_catalog.oidvectortypes(p.proargtypes)
                        )
                        FROM pg_catalog.pg_proc p
                        WHERE p.pronamespace = 'public'::pg_catalog.regnamespace
                          AND p.proowner = grantor_oid
                    LOOP
                        EXECUTE format('REVOKE ALL ON FUNCTION %s FROM %I', obj, target);
                    END LOOP;
                END IF;
            END LOOP;
            IF did_set THEN
                EXECUTE 'RESET ROLE';
            END IF;
        EXCEPTION WHEN OTHERS THEN
            IF did_set THEN
                EXECUTE 'RESET ROLE';
            END IF;
            RAISE;
        END;
    END LOOP;
    SELECT count(*) INTO leftover FROM (
        SELECT 1
        FROM pg_catalog.pg_class c
        JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
        JOIN pg_catalog.pg_roles r ON r.rolname IN ('anon', 'authenticated', 'service_role')
        WHERE n.nspname = 'public' AND c.relkind IN ('r', 'p', 'v', 'm', 'S')
          AND EXISTS (SELECT 1 FROM pg_catalog.aclexplode(c.relacl) a WHERE a.grantee = r.oid)
        UNION ALL
        SELECT 1
        FROM pg_catalog.pg_proc p
        JOIN pg_catalog.pg_roles r ON r.rolname IN ('anon', 'authenticated', 'service_role')
        WHERE p.pronamespace = 'public'::pg_catalog.regnamespace
          AND EXISTS (SELECT 1 FROM pg_catalog.aclexplode(p.proacl) a WHERE a.grantee = r.oid)
    ) AS remaining;
    IF leftover > 0 THEN
        RAISE EXCEPTION 'supabase 3 ロールへの権限が % 件残っている', leftover;
    END IF;
END
$$;
