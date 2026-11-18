-- 共有のマスタとキャッシュ(利用者の行なし): stock・security_prices・fund_codes・fund_prices・
-- dividend_per_share_cache・market_data_provider_rate_control と、ファンド名の照合関数。
-- app_worker・app_restricted には読み取りのポリシーだけを付け、書き込みは 0106 の限定関数に閉じ込める。
-- RLS を明示するのは、本番(Supabase)の ensure_rls が新しい表の RLS を有効にし、
-- ポリシーが無いと NOBYPASSRLS のロールからエラーなしで 0 行に見えるため。

CREATE TABLE public.stock (
    date date NOT NULL,
    code character varying(10) NOT NULL,
    name public.citext NOT NULL,
    market_category character varying(50) NOT NULL,
    industry_category_33 character varying(100),
    industry_category_17 character varying(100),
    size_category character varying(50),
    CONSTRAINT stock_pkey PRIMARY KEY (code)
);

CREATE INDEX idx_stock_name_trgm ON public.stock USING gin (name public.gin_trgm_ops);

COMMENT ON TABLE public.stock IS '上場銘柄のマスタ(共有)';

ALTER TABLE public.stock ENABLE ROW LEVEL SECURITY;
CREATE POLICY app_worker_shared_read ON public.stock
    FOR SELECT TO app_worker, app_restricted USING (true);


CREATE TABLE public.security_prices (
    security_code character varying(10) NOT NULL,
    current_price numeric DEFAULT 0 NOT NULL,
    price_as_of date,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    prev_close numeric,
    CONSTRAINT security_prices_pkey PRIMARY KEY (security_code)
);

COMMENT ON TABLE public.security_prices IS '銘柄ごとの共有株価';
COMMENT ON COLUMN public.security_prices.price_as_of IS '値として確定した基準日';
COMMENT ON COLUMN public.security_prices.prev_close IS '前日終値';

ALTER TABLE public.security_prices ENABLE ROW LEVEL SECURITY;
CREATE POLICY app_worker_shared_read ON public.security_prices
    FOR SELECT TO app_worker, app_restricted USING (true);


-- ファンド名の照合用正規化。NFKC で全角/半角・括弧のゆれを吸収し、
-- 空白の有無と大文字小文字を無視する。seed 側・保有行側の両方をこの1関数で正規化する
CREATE FUNCTION public.fund_name_normalize(name text) RETURNS text
    LANGUAGE sql IMMUTABLE STRICT
AS $$ SELECT lower(regexp_replace(normalize(name, nfkc), '\s+', '', 'g')) $$;

-- 「いま公表済みのはずの、直近の基準価額の日」(E)。JST 23:00 を過ぎた平日は当日、
-- それより前は前の平日(土日は金曜)。祝日は扱わない(出すべきでない前日比を出さない側に倒す)
CREATE FUNCTION public.fund_expected_nav_date(now_ts timestamptz) RETURNS date
    LANGUAGE sql STABLE STRICT
AS $$
    SELECT d - (CASE extract(isodow FROM d) WHEN 6 THEN 1 WHEN 7 THEN 2 ELSE 0 END)::int
    FROM (
        SELECT CASE WHEN (now_ts AT TIME ZONE 'Asia/Tokyo')::time >= time '23:00'
                    THEN (now_ts AT TIME ZONE 'Asia/Tokyo')::date
                    ELSE (now_ts AT TIME ZONE 'Asia/Tokyo')::date - 1
               END AS d
    ) s
$$;


-- seed は資産運用業協会の NISA 成長投資枠対象商品リストのうち運用会社が三菱UFJアセットのもの
-- (生成手順は scripts/update_fund_codes_seed.py)
CREATE TABLE public.fund_codes (
    normalized_name text NOT NULL,
    association_fund_cd character varying(10) NOT NULL,
    fund_name character varying(200) NOT NULL,
    CONSTRAINT fund_codes_pkey PRIMARY KEY (normalized_name)
);

COMMENT ON TABLE public.fund_codes IS 'ファンド名と投信協会コードの対応表(共有)';
COMMENT ON COLUMN public.fund_codes.normalized_name IS '照合用に正規化したファンド名';

INSERT INTO public.fund_codes (normalized_name, association_fund_cd, fund_name)
SELECT public.fund_name_normalize(v.fund_name), v.association_fund_cd, v.fund_name
FROM (VALUES
-- gen:fund-codes-seed begin
    ('いちよし　ジャパン・ベンチャー・ファンド','0331117B'),
    ('いちよし　公開ベンチャー・ファンド','0331200B'),
    ('これぞ、日本株','0331218B'),
    ('つみたて先進国株式','03313178'),
    ('つみたて先進国株式（為替ヘッジあり）','03317178'),
    ('つみたて全世界株式','03316203'),
    ('つみたて新興国株式','03314178'),
    ('つみたて日本株式（日経平均）','03316178'),
    ('つみたて日本株式（読売３３３）','03311254'),
    ('つみたて日本株式（ＴＯＰＩＸ）','03312178'),
    ('つみたて米国株式（Ｓ＆Ｐ５００）','03315203'),
    ('つみたて４資産均等バランス','03318178'),
    ('つみたて８資産均等バランス','03315178'),
    ('わたしの未来設計＜安定重視型＞（分配コース）','03311189'),
    ('わたしの未来設計＜安定重視型＞（分配抑制コース）','03312189'),
    ('わたしの未来設計＜成長重視型＞（分配コース）','03313189'),
    ('わたしの未来設計＜成長重視型＞（分配抑制コース）','03314189'),
    ('インデックスファンド２２５','03311865'),
    ('インド債券オープン（年１回決算型）','03311177'),
    ('エマージング・ソブリン・オープン（資産成長型）','0331113N'),
    ('エマージング・ソブリン・オープン（資産成長型）為替ヘッジあり','0331213N'),
    ('エマージング・ソブリン・オープン（１年決算型）','03313038'),
    ('エマージング株式オープン','0331307C'),
    ('グリーン・テクノロジー株式ファンド（為替ヘッジあり）','03311216'),
    ('グリーン・テクノロジー株式ファンド（為替ヘッジなし）','03312216'),
    ('グローバルリーダーズ日本株トップ１０インデックスファンド','03311267'),
    ('グローバル・スマート・イノベーション・オープン（年１回決算型）','0331215C'),
    ('グローバル・スマート・イノベーション・オープン（年１回決算型）為替ヘッジあり','03312173'),
    ('グローバル・スマート・イノベーション・オープン（年２回決算型）','0331115C'),
    ('グローバル・スマート・イノベーション・オープン（年２回決算型）為替ヘッジあり','03311173'),
    ('グローバル・ソブリン・オープン（３ヵ月決算型）','0331497C'),
    ('グローバル・ヘルスケア＆バイオ・オープン　Ｂコース（為替ヘッジなし）','03313007'),
    ('グローバル・ヘルスケア＆バイオ・ファンド','03316042'),
    ('グローバル・ヘルスケア＆バイオ・ファンド（為替ヘッジあり）','0331316B'),
    ('グローバル・リート・アクティブファンド（ラップ向け）','0331120C'),
    ('グローバル株式インカム（年１回決算型）','03311255'),
    ('グローバル株式インカム（３ヵ月決算型）予想分配金提示型','03312255'),
    ('サイバーセキュリティ株式オープン（為替ヘッジあり）','03314177'),
    ('サイバーセキュリティ株式オープン（為替ヘッジなし）','03315177'),
    ('サイバーセキュリティ株式オープン＜隔月決算型＞（為替ヘッジなし）予想分配金提示型','03311243'),
    ('サイバーセキュリティ株式オープン＜３ヵ月決算型＞（為替ヘッジなし）','03311252'),
    ('システム・オープン','03311835'),
    ('スマート・クオリティ・オープン（安定型）','0331414B'),
    ('スマート・クオリティ・オープン（安定型）（１年決算型）','03314151'),
    ('スマート・クオリティ・オープン（安定成長型）','0331514B'),
    ('スマート・クオリティ・オープン（安定成長型）（１年決算型）','03315151'),
    ('スマート・クオリティ・オープン（成長型）','0331614B'),
    ('スマート・クオリティ・オープン（成長型）（１年決算型）','03316151'),
    ('トピックスオープン','03311903'),
    ('ベイリー・ギフォード　インパクト投資ファンド','03312196'),
    ('ベイリー・ギフォード世界長期成長株ファンド','03311191'),
    ('モルガン・スタンレー　グローバル・プレミアム株式オープン（為替ヘッジあり）','0331112E'),
    ('モルガン・スタンレー　グローバル・プレミアム株式オープン（為替ヘッジなし）','0331212E'),
    ('モルガン・スタンレー　グローバル・プレミアム株式オープン＜隔月決算型＞（為替ヘッジあり）予想分配金提示型','0331123B'),
    ('モルガン・スタンレー　グローバル・プレミアム株式オープン＜隔月決算型＞（為替ヘッジなし）予想分配金提示型','0331223B'),
    ('モルガン・スタンレー　米国株式インサイト戦略ファンド（為替ヘッジあり）','03311259'),
    ('モルガン・スタンレー　米国株式インサイト戦略ファンド（為替ヘッジなし）','03312259'),
    ('ラップ向けインデックスｆ　先進国リート','03318214'),
    ('ラップ向けインデックスｆ　先進国債券','03317214'),
    ('ラップ向けインデックスｆ　先進国債券（為替ヘッジあり）','03313221'),
    ('ラップ向けインデックスｆ　先進国株式','03316214'),
    ('ラップ向けインデックスｆ　先進国株式（為替ヘッジあり）','03312221'),
    ('ラップ向けインデックスｆ　国内リート','03315214'),
    ('ラップ向けインデックスｆ　国内債券','03314214'),
    ('ラップ向けインデックスｆ　国内株式','03313214'),
    ('ラップ向けインデックスｆ　新興国債券','0331A214'),
    ('ラップ向けインデックスｆ　新興国株式','03319214'),
    ('ワールド・ビューティー・オープン（為替ヘッジあり）','03318177'),
    ('ワールド・ビューティー・オープン（為替ヘッジなし）','03319177'),
    ('三菱ＵＦＪ　インデックス２２５オープン','0331298B'),
    ('三菱ＵＦＪ　インデックス２２５オープン（確定拠出年金）','03312025'),
    ('三菱ＵＦＪ　グローバル・ボンド・オープン（年１回決算型）','0331713C'),
    ('三菱ＵＦＪ　ゴールドファンド＜年１回決算型＞','03311265'),
    ('三菱ＵＦＪ　チャイナオープン','0331196A'),
    ('三菱ＵＦＪ　トピックスインデックスオープン','03312004'),
    ('三菱ＵＦＪ　トピックスオープン','03316013'),
    ('三菱ＵＦＪ　トピックスオープン（確定拠出年金）','0331C01C'),
    ('三菱ＵＦＪ　バリューオープン','03313009'),
    ('三菱ＵＦＪ　ライフセレクトファンド（安定型）','03311008'),
    ('三菱ＵＦＪ　ライフセレクトファンド（安定成長型）','03312008'),
    ('三菱ＵＦＪ　ライフセレクトファンド（成長型）','03313008'),
    ('三菱ＵＦＪ　世界国債インデックスファンド（年１回決算型）','03313083'),
    ('三菱ＵＦＪ　先進国リートインデックスファンド','03312204'),
    ('三菱ＵＦＪ　国内リートインデックスファンド','0331122A'),
    ('三菱ＵＦＪ　国内債券インデックスファンド','03311092'),
    ('三菱ＵＦＪ　外国債券オープン（確定拠出年金）','03314022'),
    ('三菱ＵＦＪ　外国株式インデックスファンド','03312046'),
    ('三菱ＵＦＪ　日本株アクティブオープン','03312986'),
    ('三菱ＵＦＪ　日本株アクティブオープン（確定拠出年金）','03311025'),
    ('三菱ＵＦＪ　日本株アクティブ・ファンド','0331400A'),
    ('三菱ＵＦＪ　日本株オープン「３５」','03311993'),
    ('三菱ＵＦＪ　日本短期債券オープン','03317262'),
    ('三菱ＵＦＪ　海外債券オープン','0331A008'),
    ('三菱ＵＦＪ　海外債券オープン（３ヵ月決算型）','03313022'),
    ('三菱ＵＦＪ　純金ファンド','03311112'),
    ('三菱ＵＦＪ　豪ドル債券インカムオープン（年１回決算型）','0331113C'),
    ('三菱ＵＦＪ　資産配分最適化バランス（Ｒ１）','03311257'),
    ('三菱ＵＦＪ　資産配分最適化バランス（Ｒ２）','03312257'),
    ('三菱ＵＦＪ　資産配分最適化バランス（Ｒ３）','03313257'),
    ('三菱ＵＦＪ　資産配分最適化バランス（Ｒ４）','03314257'),
    ('三菱ＵＦＪ　資産配分最適化バランス（Ｒ５）','03315257'),
    ('三菱ＵＦＪ　Ｊリートオープン（年１回決算型）','0331313B'),
    ('三菱ＵＦＪ　Ｊリートオープン（３ヵ月決算型）','03313046'),
    ('三菱ＵＦＪ　ＮＡＳＤＡＱオープン　Ａコース','03311968'),
    ('三菱ＵＦＪ　ＮＡＳＤＡＱオープン　Ｂコース','03312968'),
    ('三菱ＵＦＪ　Ｓ＆Ｐ５００配当貴族インデックスファンド＜為替ヘッジあり＞','0331313C'),
    ('三菱ＵＦＪ　Ｓ＆Ｐ５００配当貴族インデックスファンド＜為替ヘッジなし＞','0331213C'),
    ('三菱ＵＦＪ　ＴＯＰＩＸ・ファンド','03311013'),
    ('三菱ＵＦＪ／マッコーリー　グローバル・インフラ債券ファンド＜為替ヘッジあり＞（年１回決算型）','03314181'),
    ('三菱ＵＦＪ／マッコーリー　グローバル・インフラ債券ファンド＜為替ヘッジなし＞（年１回決算型）','03315181'),
    ('三菱ＵＦＪ／マッコーリー　グローバル・インフラ債券ファンド＜為替ヘッジなし＞（隔月決算型）','0331323B'),
    ('世界プレミア企業債券ファンド（為替ヘッジあり）','03311161'),
    ('世界メディカル関連株式オープン','03313149'),
    ('優良日本株ファンド','03311093'),
    ('先進国好配当株式ファンド（年２回決算型）','03314162'),
    ('先進国好配当株式ファンド（年２回決算型）為替ヘッジあり','03316172'),
    ('先進国好配当株式ファンド（３ヵ月決算型）','03313162'),
    ('先進国好配当株式ファンド（３ヵ月決算型）為替ヘッジあり','03315172'),
    ('外国債券アクティブファンドセレクション（ラップ専用）','03314191'),
    ('外国株式アクティブファンドセレクション（ラップ専用）','03313191'),
    ('日本厳選プレミアム株式オープン（年２回決算型）','0331114D'),
    ('日本好配当利回り株オープン（３ヵ月決算型）','03311056'),
    ('日経平均高配当利回り株ファンド','0331118B'),
    ('日経平均高配当利回り株ファンドⅡ','0331124B'),
    ('未来イノベーション成長株ファンド','03313183'),
    ('次世代米国代表株ファンド','03311135'),
    ('次世代米国代表株ファンド（為替ヘッジあり）','0331416C'),
    ('次世代ＲＥＩＴオープン＜資産成長型＞（為替ヘッジあり）','03311202'),
    ('次世代ＲＥＩＴオープン＜資産成長型＞（為替ヘッジなし）','03312202'),
    ('米国ＩＰＯニューステージ・ファンド＜為替ヘッジあり＞（年２回決算型）','0331219B'),
    ('米国ＩＰＯニューステージ・ファンド＜為替ヘッジあり＞（資産成長型）','0331119B'),
    ('米国ＩＰＯニューステージ・ファンド＜為替ヘッジなし＞（年２回決算型）','0331419B'),
    ('米国ＩＰＯニューステージ・ファンド＜為替ヘッジなし＞（資産成長型）','0331319B'),
    ('超長期日本国債ファンド（２０４６年満期・限定追加型）','03311269'),
    ('ＧＲＡＮ　ＮＥＸＴ　エネルギー','03312211'),
    ('ＧＲＡＮ　ＮＥＸＴ　テクノロジー','03313211'),
    ('ＧＲＡＮ　ＮＥＸＴ　フィナンシャルズ','03311245'),
    ('ＧＲＡＮ　ＮＥＸＴ　ヘルスサイエンス','03315211'),
    ('ＧＲＡＮ　ＮＥＸＴ　モビリティ','03316211'),
    ('Ｊオープン（小型株）','03311946'),
    ('Ｊ・エクイティ','03317004'),
    ('ＭＵＦＧ　ウェルス・インサイト・ファンド・アネックス（為替リスク軽減型）コンサバティブコース','03312245'),
    ('ＭＵＦＧ　ウェルス・インサイト・ファンド・アネックス（為替リスク軽減型）スタンダードコース','03313245'),
    ('ＭＵＦＧ　ウェルス・インサイト・ファンド（保守型）','03314215'),
    ('ＭＵＦＧ　ウェルス・インサイト・ファンド（標準型）','03315215'),
    ('ＭＵＦＧ　ウェルス・インサイト・ファンド（積極型）','03316215'),
    ('ＮＹダウ・インデックスファンド（為替ヘッジあり）','03314174'),
    ('ＮＹダウ・インデックスファンド（為替ヘッジなし）','03315174'),
    ('ｅＭＡＸＩＳ　インド株式インデックス','03311242'),
    ('ｅＭＡＸＩＳ　ジャパンＥＳＧセレクト・リーダーズインデックス','0331318A'),
    ('ｅＭＡＸＩＳ　バランス（波乗り型）','0331211A'),
    ('ｅＭＡＸＩＳ　バランス（４資産均等型）','03316158'),
    ('ｅＭＡＸＩＳ　バランス（８資産均等型）','0331111A'),
    ('ｅＭＡＸＩＳ　マイマネージャー　１９７０ｓ','0331517A'),
    ('ｅＭＡＸＩＳ　マイマネージャー　１９８０ｓ','0331617A'),
    ('ｅＭＡＸＩＳ　マイマネージャー　１９９０ｓ','0331717A'),
    ('ｅＭＡＸＩＳ　債券バランス（２資産均等型）','03315158'),
    ('ｅＭＡＸＩＳ　先進国リートインデックス（除く日本）','0331709A'),
    ('ｅＭＡＸＩＳ　先進国債券インデックス（除く日本・為替ヘッジあり）','03312167'),
    ('ｅＭＡＸＩＳ　先進国債券インデックス（除く日本）','0331609A'),
    ('ｅＭＡＸＩＳ　先進国株式インデックス（除く日本）','0331509A'),
    ('ｅＭＡＸＩＳ　全世界株式インデックス（除く日本）','03312107'),
    ('ｅＭＡＸＩＳ　国内リートインデックス','0331409A'),
    ('ｅＭＡＸＩＳ　国内債券インデックス','0331309A'),
    ('ｅＭＡＸＩＳ　国内物価連動国債インデックス','0331314B'),
    ('ｅＭＡＸＩＳ　新興国リートインデックス','0331213B'),
    ('ｅＭＡＸＩＳ　新興国債券インデックス','03311109'),
    ('ｅＭＡＸＩＳ　新興国債券インデックス（為替ヘッジあり）','03313167'),
    ('ｅＭＡＸＩＳ　新興国株式インデックス','0331809A'),
    ('ｅＭＡＸＩＳ　日経アジア３００インベスタブル・インデックス','03312181'),
    ('ｅＭＡＸＩＳ　日経半導体株インデックス','03311247'),
    ('ｅＭＡＸＩＳ　日経２２５インデックス','0331109A'),
    ('ｅＭＡＸＩＳ　最適化バランス（マイゴールキーパー）','03312163'),
    ('ｅＭＡＸＩＳ　最適化バランス（マイストライカー）','03316163'),
    ('ｅＭＡＸＩＳ　最適化バランス（マイディフェンダー）','03313163'),
    ('ｅＭＡＸＩＳ　最適化バランス（マイフォワード）','03315163'),
    ('ｅＭＡＸＩＳ　最適化バランス（マイミッドフィルダー）','03314163'),
    ('ｅＭＡＸＩＳ　欧州リートインデックス','03313161'),
    ('ｅＭＡＸＩＳ　米国リートインデックス','03312161'),
    ('ｅＭＡＸＩＳ　豪州リートインデックス','03314161'),
    ('ｅＭＡＸＩＳ　豪州債券インデックス','03311167'),
    ('ｅＭＡＸＩＳ　高配当全世界株式（オール・カントリー）資産成長型','03313269'),
    ('ｅＭＡＸＩＳ　高配当全世界株式（オール・カントリー）配当払出型','03312269'),
    ('ｅＭＡＸＩＳ　ＪＡＰＡＮ　クオリティ１５０インデックス','0331115B'),
    ('ｅＭＡＸＩＳ　ＪＰＸ日経中小型インデックス','03313176'),
    ('ｅＭＡＸＩＳ　ＪＰＸ日経４００インデックス','03311144'),
    ('ｅＭＡＸＩＳ　ＮＡＳＤＡＱ１００インデックス','0331A211'),
    ('ｅＭＡＸＩＳ　ＮＹダウインデックス','03311138'),
    ('ｅＭＡＸＩＳ　Ｎｅｏ　ウェアラブル','03314195'),
    ('ｅＭＡＸＩＳ　Ｎｅｏ　クリーンテック','03312215'),
    ('ｅＭＡＸＩＳ　Ｎｅｏ　コミュニケーションＤＸ','03311223'),
    ('ｅＭＡＸＩＳ　Ｎｅｏ　ドローン','0331118C'),
    ('ｅＭＡＸＩＳ　Ｎｅｏ　ナノテクノロジー','0331218C'),
    ('ｅＭＡＸＩＳ　Ｎｅｏ　バーチャルリアリティ','0331318C'),
    ('ｅＭＡＸＩＳ　Ｎｅｏ　フィンテック','03312195'),
    ('ｅＭＡＸＩＳ　Ｎｅｏ　ロボット','03312188'),
    ('ｅＭＡＸＩＳ　Ｎｅｏ　宇宙開発','03313188'),
    ('ｅＭＡＸＩＳ　Ｎｅｏ　水素エコノミー','03312223'),
    ('ｅＭＡＸＩＳ　Ｎｅｏ　自動運転','03313195'),
    ('ｅＭＡＸＩＳ　Ｎｅｏ　遺伝子工学','03311188'),
    ('ｅＭＡＸＩＳ　Ｎｅｏ　電気自動車','03313215'),
    ('ｅＭＡＸＩＳ　Ｎｅｏ　ＡＩテクノロジー','03311249'),
    ('ｅＭＡＸＩＳ　Ｓ＆Ｐ５００インデックス','0331220C'),
    ('ｅＭＡＸＩＳ　Ｓ＆Ｐ５００クオリティ高配当インデックス','03311231'),
    ('ｅＭＡＸＩＳ　Ｓｌｉｍ　バランス（８資産均等型）','03312175'),
    ('ｅＭＡＸＩＳ　Ｓｌｉｍ　先進国リートインデックス（除く日本）','0331219A'),
    ('ｅＭＡＸＩＳ　Ｓｌｉｍ　先進国債券インデックス（除く日本）','0331A172'),
    ('ｅＭＡＸＩＳ　Ｓｌｉｍ　先進国株式インデックス（除く日本）','03319172'),
    ('ｅＭＡＸＩＳ　Ｓｌｉｍ　先進国株式（含む日本）＜オール先進国＞','0331124A'),
    ('ｅＭＡＸＩＳ　Ｓｌｉｍ　全世界株式（オール・カントリー）','0331418A'),
    ('ｅＭＡＸＩＳ　Ｓｌｉｍ　全世界株式（除く日本）','03316183'),
    ('ｅＭＡＸＩＳ　Ｓｌｉｍ　全世界株式（３地域均等型）','03312184'),
    ('ｅＭＡＸＩＳ　Ｓｌｉｍ　全米株式','03311239'),
    ('ｅＭＡＸＩＳ　Ｓｌｉｍ　国内リートインデックス','0331119A'),
    ('ｅＭＡＸＩＳ　Ｓｌｉｍ　国内債券インデックス','03318172'),
    ('ｅＭＡＸＩＳ　Ｓｌｉｍ　国内株式（日経平均）','03311182'),
    ('ｅＭＡＸＩＳ　Ｓｌｉｍ　国内株式（読売３３３）','03315253'),
    ('ｅＭＡＸＩＳ　Ｓｌｉｍ　国内株式（ＴＯＰＩＸ）','03317172'),
    ('ｅＭＡＸＩＳ　Ｓｌｉｍ　新興国株式インデックス','0331C177'),
    ('ｅＭＡＸＩＳ　Ｓｌｉｍ　米国株式（Ｓ＆Ｐ５００）','03311187'),
    ('ｅＭＡＸＩＳ　ＴＯＰＩＸインデックス','0331209A'),
    ('ｅＭＡＸＩＳ／ＰａｙＰａｙ証券　全世界バランス','03312249')
-- gen:fund-codes-seed end
) AS v(fund_name, association_fund_cd);

ALTER TABLE public.fund_codes ENABLE ROW LEVEL SECURITY;
CREATE POLICY app_worker_shared_read ON public.fund_codes
    FOR SELECT TO app_worker, app_restricted USING (true);


-- 未取得のコードは試行時刻だけの行を持ちうる(security_prices と同じ)
CREATE TABLE public.fund_prices (
    association_fund_cd character varying(10) NOT NULL,
    nav numeric,
    prev_day_change numeric,
    price_as_of date,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    fetch_error text,
    fetch_failed_at timestamp with time zone,
    CONSTRAINT fund_prices_pkey PRIMARY KEY (association_fund_cd)
);

COMMENT ON TABLE public.fund_prices IS 'ファンドごとの共有基準価額';
COMMENT ON COLUMN public.fund_prices.nav IS '共有の基準価額';
COMMENT ON COLUMN public.fund_prices.prev_day_change IS '共有の基準価額の前日比';
COMMENT ON COLUMN public.fund_prices.price_as_of IS '基準価額の基準日';
COMMENT ON COLUMN public.fund_prices.fetch_error IS '直近の基準価額取得の失敗理由';
COMMENT ON COLUMN public.fund_prices.fetch_failed_at IS '直近の基準価額取得の失敗時刻';

ALTER TABLE public.fund_prices ENABLE ROW LEVEL SECURITY;
CREATE POLICY app_worker_shared_read ON public.fund_prices
    FOR SELECT TO app_worker, app_restricted USING (true);


CREATE TABLE public.dividend_per_share_cache (
    security_code character varying(10) NOT NULL,
    dividend_per_share numeric,
    status character varying(20) DEFAULT 'pending'::character varying NOT NULL,
    fetched_at timestamp with time zone,
    provider character varying(50) DEFAULT 'jquants'::character varying NOT NULL,
    error_message text,
    stale_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT dividend_per_share_cache_pkey PRIMARY KEY (security_code)
);

COMMENT ON TABLE public.dividend_per_share_cache IS '1株配当のキャッシュ(共有)';
COMMENT ON COLUMN public.dividend_per_share_cache.status IS '取得状態(ok / zero / error / pending)';
COMMENT ON COLUMN public.dividend_per_share_cache.stale_at IS 'TTL 期限。この時刻を過ぎると再取得対象';

ALTER TABLE public.dividend_per_share_cache ENABLE ROW LEVEL SECURITY;
CREATE POLICY app_worker_shared_read ON public.dividend_per_share_cache
    FOR SELECT TO app_worker, app_restricted USING (true);


CREATE TABLE public.market_data_provider_rate_control (
    provider character varying(50) NOT NULL,
    next_available_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT market_data_provider_rate_control_pkey PRIMARY KEY (provider)
);

COMMENT ON TABLE public.market_data_provider_rate_control IS '外部APIの呼び出し間隔(全インスタンス共有)';
COMMENT ON COLUMN public.market_data_provider_rate_control.next_available_at IS '次の呼び出し可能時刻';

INSERT INTO public.market_data_provider_rate_control (provider, next_available_at)
    VALUES ('jquants', now());

ALTER TABLE public.market_data_provider_rate_control ENABLE ROW LEVEL SECURITY;
CREATE POLICY app_worker_shared_read ON public.market_data_provider_rate_control
    FOR SELECT TO app_worker, app_restricted USING (true);
