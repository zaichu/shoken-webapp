use super::*;

#[test]
fn has_current_summary_only_matches_same_generation() {
    let generation = Generation::new(1);
    let other = Generation::new(2);
    let slot: Option<(Generation, Option<u32>)> = Some((generation, Some(7)));
    assert!(has_current_summary(&slot, generation));
    assert!(!has_current_summary(&slot, other));
    assert!(!has_current_summary::<u32>(
        &Some((generation, None)),
        other
    ));
    assert!(!has_current_summary::<u32>(&None, generation));
}

#[test]
fn summary_fetch_blocks_second_request_while_inflight() {
    let generation = Generation::new(1);
    let mut map = HashMap::new();
    assert_eq!(
        begin_summary_fetch(&mut map, generation, SummaryKind::Asset),
        Some(1)
    );
    assert_eq!(
        begin_summary_fetch(&mut map, generation, SummaryKind::Asset),
        None,
        "取得中の同じ集計を重ねて要求しない"
    );
    assert_eq!(
        begin_summary_fetch(&mut map, generation, SummaryKind::Dividend),
        Some(1),
        "別の集計は取得中でも取れる"
    );
    assert!(finish_summary_fetch(
        &mut map,
        generation,
        SummaryKind::Asset,
        1
    ));
    assert_eq!(
        begin_summary_fetch(&mut map, generation, SummaryKind::Asset),
        Some(2),
        "応答後は再取得できる"
    );
}

#[test]
fn summary_fetch_drops_stale_response() {
    let generation = Generation::new(1);
    let mut map = HashMap::new();
    let rev = begin_summary_fetch(&mut map, generation, SummaryKind::Asset).unwrap();
    assert!(finish_summary_fetch(
        &mut map,
        generation,
        SummaryKind::Asset,
        rev
    ));
    let next = begin_summary_fetch(&mut map, generation, SummaryKind::Asset).unwrap();
    assert!(
        !finish_summary_fetch(&mut map, generation, SummaryKind::Asset, rev),
        "古い応答は inflight を下ろさない"
    );
    assert!(
        map.get(&(generation, SummaryKind::Asset))
            .is_some_and(|entry| entry.inflight),
        "新しい取得はまだ取得中のまま"
    );
    assert!(finish_summary_fetch(
        &mut map,
        generation,
        SummaryKind::Asset,
        next
    ));
    // 別世代のキーは同じ rev でも混ざらない
    let other = Generation::new(2);
    assert_eq!(
        begin_summary_fetch(&mut map, other, SummaryKind::Asset),
        Some(1)
    );
}

#[test]
fn settle_summary_keeps_displayed_value_on_refresh_error() {
    let generation = Generation::new(1);
    let error = || -> Result<Option<u32>, ApiError> { Err(ApiError::http(500)) };
    // 裏再取得の失敗では表示済みの値を消さない(書き戻さない)
    let current = Some((generation, Some(260_000u32)));
    assert_eq!(settle_summary(&current, generation, error()), None);
    // 初回の失敗は従来どおり失敗として書く(「一部の集計を取得できませんでした」が出る)
    assert_eq!(
        settle_summary::<u32>(&None, generation, error()),
        Some((generation, None))
    );
    assert_eq!(
        settle_summary(&Some((generation, None::<u32>)), generation, error()),
        Some((generation, None))
    );
    // 別世代の成功値は今の世代の失敗を隠さない
    assert_eq!(
        settle_summary(&Some((Generation::new(0), Some(1u32))), generation, error()),
        Some((generation, None))
    );
    // 成功は常に書く(Ok(None) は「集計なし」の確定)
    assert_eq!(
        settle_summary(&current, generation, Ok(Some(1))),
        Some((generation, Some(1)))
    );
    assert_eq!(
        settle_summary(&current, generation, Ok(None)),
        Some((generation, None))
    );
}
