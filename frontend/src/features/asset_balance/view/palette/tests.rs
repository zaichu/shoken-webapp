use super::*;
use std::collections::HashSet;

#[test]
fn palette_cycles_without_fixing_token_names() {
    assert_eq!(BARS.len(), BANDS.len());
    for palette in [&BARS, &BANDS] {
        assert_eq!(palette.iter().collect::<HashSet<_>>().len(), palette.len());
    }
    for position in 0..BARS.len() * 2 {
        assert_eq!(holding_bar_class(position), BARS[position % BARS.len()]);
        assert_eq!(holding_band_class(position), BANDS[position % BANDS.len()]);
    }
}
