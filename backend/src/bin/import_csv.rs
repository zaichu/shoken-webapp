// CSVインポート用の一時スクリプト（バルクインサート版）
use chrono::NaiveDate;
use sqlx::{postgres::PgPoolOptions, Postgres, QueryBuilder};
use std::env;
use std::fs::File;
use std::io::BufReader;

#[derive(Debug)]
struct StockRecord {
    date: NaiveDate,
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

        let date = NaiveDate::parse_from_str(&record[0], "%Y%m%d")?;

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
    let total_batches = records.len().div_ceil(batch_size);

    for (batch_num, chunk) in records.chunks(batch_size).enumerate() {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            "INSERT INTO stock (date, code, name, market_category, industry_code_33, industry_category_33, industry_code_17, industry_category_17, size_code, size_category) ",
        );

        query_builder.push_values(chunk, |mut row, rec| {
            row.push_bind(rec.date);
            row.push_bind(&rec.code);
            row.push_bind(&rec.name);
            row.push_bind(&rec.market_category);
            row.push_bind(&rec.industry_code_33);
            row.push_bind(&rec.industry_category_33);
            row.push_bind(&rec.industry_code_17);
            row.push_bind(&rec.industry_category_17);
            row.push_bind(&rec.size_code);
            row.push_bind(&rec.size_category);
        });

        query_builder.push(
            " ON CONFLICT (date, code) DO UPDATE SET
                name = EXCLUDED.name,
                market_category = EXCLUDED.market_category,
                industry_code_33 = EXCLUDED.industry_code_33,
                industry_category_33 = EXCLUDED.industry_category_33,
                industry_code_17 = EXCLUDED.industry_code_17,
                industry_category_17 = EXCLUDED.industry_category_17,
                size_code = EXCLUDED.size_code,
                size_category = EXCLUDED.size_category",
        );

        query_builder.build().execute(&pool).await?;

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
