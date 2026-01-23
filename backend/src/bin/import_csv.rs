// CSVインポート用の一時スクリプト（バルクインサート版）
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::fs::File;
use std::io::BufReader;

#[derive(Debug)]
struct StockRecord {
    date: String,
    code: String,
    name: String,
    market_category: String,
    industry_code_33: Option<String>,
    industry_category_33: Option<String>,
    industry_code_17: Option<String>,
    industry_category_17: Option<String>,
    size_code: Option<String>,
    size_category: Option<String>,
}

fn null_if_dash(value: &str) -> Option<String> {
    if value == "-" {
        None
    } else {
        Some(value.to_string())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let csv_path = env::args().nth(1).expect("CSV path required");

    println!("データベースに接続中...");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    println!("CSVファイルを読み込み中: {}", csv_path);
    let file = File::open(&csv_path)?;
    let reader = BufReader::new(file);
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(reader);

    // 全レコードを読み込み
    let mut records: Vec<StockRecord> = Vec::new();
    for result in rdr.records() {
        let record = result?;

        let date_str = &record[0];
        let date = format!("{}-{}-{}", &date_str[0..4], &date_str[4..6], &date_str[6..8]);

        records.push(StockRecord {
            date,
            code: record[1].to_string(),
            name: record[2].to_string(),
            market_category: record[3].to_string(),
            industry_code_33: null_if_dash(&record[4]),
            industry_category_33: null_if_dash(&record[5]),
            industry_code_17: null_if_dash(&record[6]),
            industry_category_17: null_if_dash(&record[7]),
            size_code: null_if_dash(&record[8]),
            size_category: null_if_dash(&record[9]),
        });
    }

    println!("総レコード数: {}", records.len());

    // バルクインサート（100件ずつ）
    let batch_size = 100;
    let total_batches = (records.len() + batch_size - 1) / batch_size;

    for (batch_num, chunk) in records.chunks(batch_size).enumerate() {
        // VALUES句を動的に構築
        let mut values_parts: Vec<String> = Vec::new();
        let mut param_idx = 1;

        for _ in chunk {
            let placeholders: Vec<String> = (0..10)
                .map(|i| {
                    let p = format!("${}", param_idx + i);
                    p
                })
                .collect();
            // 日付はキャストが必要
            let mut ph = placeholders;
            ph[0] = format!("{}::date", ph[0]);
            values_parts.push(format!("({})", ph.join(", ")));
            param_idx += 10;
        }

        let sql = format!(
            r#"
            INSERT INTO stock (date, code, name, market_category, industry_code_33, industry_category_33, industry_code_17, industry_category_17, size_code, size_category)
            VALUES {}
            ON CONFLICT (code) DO UPDATE SET
                date = EXCLUDED.date,
                name = EXCLUDED.name,
                market_category = EXCLUDED.market_category,
                industry_code_33 = EXCLUDED.industry_code_33,
                industry_category_33 = EXCLUDED.industry_category_33,
                industry_code_17 = EXCLUDED.industry_code_17,
                industry_category_17 = EXCLUDED.industry_category_17,
                size_code = EXCLUDED.size_code,
                size_category = EXCLUDED.size_category
            "#,
            values_parts.join(", ")
        );

        let mut query = sqlx::query(&sql);
        for rec in chunk {
            query = query
                .bind(&rec.date)
                .bind(&rec.code)
                .bind(&rec.name)
                .bind(&rec.market_category)
                .bind(&rec.industry_code_33)
                .bind(&rec.industry_category_33)
                .bind(&rec.industry_code_17)
                .bind(&rec.industry_category_17)
                .bind(&rec.size_code)
                .bind(&rec.size_category);
        }

        query.execute(&pool).await?;

        println!(
            "Batch {}/{} 完了 ({} / {} records)",
            batch_num + 1,
            total_batches,
            (batch_num + 1) * batch_size.min(records.len() - batch_num * batch_size),
            records.len()
        );
    }

    println!("完了: {} 件のデータをインサートしました", records.len());
    Ok(())
}
