-- app_restricted の users と監査表の権限を必要な操作だけに絞る(#2124)。
-- users はコードが直接 UPDATE する login_notify_enabled・updated_at だけを列単位で残す
-- (login_notify.rs の通知設定の更新)。email・google_id などの UPDATE は拒否される。
-- SELECT・INSERT・DELETE は残す(認証関数の参照・upsert・退会の CASCADE)。
-- login_events は INSERT・SELECT・DELETE を残し UPDATE を外す
-- (history.rs の記録と上限の刈り込みが通り、UPDATE は使わない)。
-- login_notify_tokens は SELECT・INSERT を残し UPDATE・DELETE を外す
-- (issue_token の発行と recent_send_count の参照が通り、直接の UPDATE・DELETE は使わない。
-- 退会の CASCADE は子の DELETE 権限なしでも通る)。
-- admin_login_views は SELECT・INSERT を残し UPDATE・DELETE を外す
-- (admin.rs の閲覧記録の INSERT が通り、UPDATE・DELETE は使わない)。
-- 旧 Worker も同じコード経路のため、外す操作は現行コードが使わないものだけにする。
SET LOCAL lock_timeout = '5s';

REVOKE UPDATE ON public.users FROM app_restricted;
GRANT UPDATE (login_notify_enabled, updated_at) ON public.users TO app_restricted;

REVOKE UPDATE ON public.login_events FROM app_restricted;

REVOKE UPDATE, DELETE ON public.login_notify_tokens FROM app_restricted;

REVOKE UPDATE, DELETE ON public.admin_login_views FROM app_restricted;
