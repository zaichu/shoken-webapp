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
fn yoy_difference_returns_none_without_previous() {
    assert_eq!(yoy_difference(20_000.0, Some(14_000.0)), Some(6_000.0));
    assert_eq!(yoy_difference(10_000.0, Some(14_000.0)), Some(-4_000.0));
    assert_eq!(yoy_difference(0.0, Some(0.0)), Some(0.0));
    // 前年なしのときは差を出さない
    assert_eq!(yoy_difference(20_000.0, None), None);
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
