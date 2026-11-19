use super::*;

#[test]
fn ticks_cover_max_with_fewest_nice_lines() {
    // 1か月だけ大きい年: 上限が最大値を覆う最小の段(0・100万・200万)
    let ticks = axis_ticks(1_612_332.0);
    assert_eq!(ticks, vec![0.0, 1_000_000.0, 2_000_000.0]);
    // 例示の段(0・50万・100万・150万)
    assert_eq!(
        axis_ticks(1_500_000.0),
        vec![0.0, 500_000.0, 1_000_000.0, 1_500_000.0]
    );
}

#[test]
fn ticks_stay_nice_and_separate_for_edge_cases() {
    for max in [
        0.0,
        8_000.0,
        12_000.0,
        25_000.0,
        250_000_000.0,
        3_200_000_000.0,
    ] {
        let ticks = axis_ticks(max);
        assert!(
            (3..=4).contains(&ticks.len()),
            "3〜4本になること: {max} -> {ticks:?}"
        );
        assert_eq!(ticks[0], 0.0);
        assert!(
            ticks.windows(2).all(|pair| pair[1] > pair[0]),
            "重ならないこと: {ticks:?}"
        );
        assert!(
            ticks.iter().all(|tick| tick.fract() == 0.0),
            "きりのよい値であること: {ticks:?}"
        );
    }
    // 全部 0 の年は最小の段で出す
    assert_eq!(axis_ticks(0.0), vec![0.0, 5_000.0, 10_000.0]);
    // 前年なし・小さい最大値は万に届かず千円単位でそろう
    assert_eq!(axis_ticks(8_000.0), vec![0.0, 5_000.0, 10_000.0]);
    // 桁の多い金額は億でそろう
    assert_eq!(
        axis_ticks(250_000_000.0),
        vec![0.0, 100_000_000.0, 200_000_000.0, 300_000_000.0]
    );
}

#[test]
fn small_max_stays_in_ten_thousands() {
    // 1万円以上では目盛りが1万円以上になり、万の書き方にそろう
    assert_eq!(axis_ticks(12_000.0), vec![0.0, 10_000.0, 20_000.0]);
    assert_eq!(axis_ticks(20_000.0), vec![0.0, 10_000.0, 20_000.0]);
}

#[test]
fn end_label_positions_keep_labels_apart_and_inside_the_plot() {
    let separated = end_label_positions(24.0, Some(40.0), 12);
    assert_eq!(separated.current, 24.0);
    assert_eq!(separated.previous, Some(40.0));

    let overlapping = end_label_positions(40.0, Some(42.0), 12);
    assert_eq!(overlapping.current, 47.0);
    assert_eq!(overlapping.previous, Some(35.0));

    let near_top = end_label_positions(2.0, Some(1.0), 12);
    assert_eq!(near_top.current, 7.5);
    assert_eq!(near_top.previous, Some(19.5));
}

#[test]
fn short_labels_use_man_and_oku() {
    assert_eq!(labels::axis_yen_short(0.0), "0");
    assert_eq!(labels::axis_yen_short(5_000.0), "5,000");
    assert_eq!(labels::axis_yen_short(500_000.0), "50万");
    assert_eq!(labels::axis_yen_short(1_500_000.0), "150万");
    assert_eq!(labels::axis_yen_short(15_000.0), "1.5万");
    assert_eq!(labels::axis_yen_short(200_000_000.0), "2億");
    assert_eq!(labels::axis_yen_short(150_000_000.0), "1.5億");
}

#[test]
fn cumulative_sums_accumulate_monthly_amounts() {
    // 1か月だけ大きい年もその月までの累計になる
    let sums = cumulative_sums(&[
        0.0,
        0.0,
        1_612_332.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
    ]);
    assert_eq!(sums[0], 0.0);
    assert_eq!(sums[2], 1_612_332.0);
    assert_eq!(sums[11], 1_612_332.0);
    // 全部 0 の年は 0 のまま
    assert_eq!(cumulative_sums(&[0.0; 12]), vec![0.0; 12]);
    // 桁の多い金額もそのまま足す
    let sums = cumulative_sums(&[
        250_000_000.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        50_000_000.0,
    ]);
    assert_eq!(sums[0], 250_000_000.0);
    assert_eq!(sums[11], 300_000_000.0);
}

#[test]
fn table_current_text_hides_future_months() {
    let sums = cumulative_sums(&[
        0.0, 0.0, 5_000.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    ]);
    // 6月時点: 6月までは累計、7月以降はまだ受取がないため出さない
    assert_eq!(
        table_current_text(6, 6, &sums),
        Some(format::format_currency(5_000.0))
    );
    assert_eq!(table_current_text(7, 6, &sums), None);
    assert_eq!(table_current_text(12, 6, &sums), None);
    // 12月時点: 12月まで出す
    assert_eq!(
        table_current_text(12, 12, &sums),
        Some(format::format_currency(5_000.0))
    );
    assert_eq!(table_current_text(0, 12, &sums), None);
}

#[test]
fn month_for_position_picks_nearest_month() {
    // 幅1200pxでは1か月100px。帯の中心に近い方を選ぶ
    assert_eq!(month_for_position(0.0, 1200.0, 12), 1);
    assert_eq!(month_for_position(50.0, 1200.0, 12), 1);
    assert_eq!(month_for_position(99.0, 1200.0, 12), 1);
    assert_eq!(month_for_position(100.0, 1200.0, 12), 2);
    assert_eq!(month_for_position(1150.0, 1200.0, 12), 12);
    assert_eq!(month_for_position(1199.0, 1200.0, 12), 12);
    // 描画域の外は端に寄せる
    assert_eq!(month_for_position(-10.0, 1200.0, 12), 1);
    assert_eq!(month_for_position(1300.0, 1200.0, 12), 12);
    // 未来月の領域は今月に寄せる(6月時点の右半分は6月)
    assert_eq!(month_for_position(700.0, 1200.0, 6), 6);
    assert_eq!(month_for_position(1199.0, 1200.0, 6), 6);
    assert_eq!(month_for_position(-5.0, 1200.0, 6), 1);
    // 幅が無い・読めないときは1月にする
    assert_eq!(month_for_position(50.0, 0.0, 12), 1);
    assert_eq!(month_for_position(f64::NAN, 1200.0, 12), 1);
    assert_eq!(month_for_position(50.0, f64::NAN, 12), 1);
}

#[test]
fn step_month_moves_within_this_year() {
    assert_eq!(step_month(6, 1, 12), 7);
    assert_eq!(step_month(6, -1, 12), 5);
    // 端では止まる
    assert_eq!(step_month(1, -1, 12), 1);
    assert_eq!(step_month(12, 1, 12), 12);
    // 今月を超えない(6月時点の End は6月)
    assert_eq!(step_month(6, 1, 6), 6);
    assert_eq!(step_month(5, 5, 6), 6);
}

#[test]
fn table_monthly_text_hides_future_months() {
    let monthly = [
        0.0, 0.0, 5_000.0, 0.0, 0.0, 0.0, 0.0, 3_000.0, 0.0, 0.0, 0.0, 0.0,
    ];
    // 6月時点: その月の受取だけ出し、7月以降は出さない
    assert_eq!(
        table_monthly_text(3, 6, &monthly),
        Some(format::format_currency(5_000.0))
    );
    assert_eq!(
        table_monthly_text(6, 6, &monthly),
        Some(format::format_currency(0.0))
    );
    assert_eq!(table_monthly_text(8, 6, &monthly), None);
    assert_eq!(table_monthly_text(12, 6, &monthly), None);
    assert_eq!(table_monthly_text(0, 6, &monthly), None);
}
