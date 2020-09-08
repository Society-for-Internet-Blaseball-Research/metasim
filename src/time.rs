use chrono::{Duration, TimeZone, Utc};

// note: only works for season 4 and on
#[allow(clippy::cast_sign_loss, clippy::module_name_repetitions)]
pub fn game_time(season: u16, day: u8) -> u64 {
    let mut date = if day >= 99 {
        Utc.ymd(2020, 8, 29).and_hms(13, 0, 0)
            + Duration::weeks(i64::from(season) - 3)
            + Duration::hours(i64::from(day) - 99)
    } else {
        Utc.ymd(2020, 8, 24).and_hms(16, 0, 0)
            + Duration::weeks(i64::from(season) - 3)
            + Duration::hours(i64::from(day))
    };

    if season == 3 && day >= 59 && day < 99 {
        date = date + Duration::hours(10);
    }
    if season == 3 && day >= 88 && day < 99 {
        date = date + Duration::hours(3);
    }

    debug_assert!(date.timestamp_millis() > 0);
    date.timestamp_millis() as u64
}

#[cfg(test)]
#[test]
fn test_game_time() {
    assert_eq!(game_time(3, 0), 1598284800000);
    assert_eq!(game_time(3, 32), 1598400000000);
    assert_eq!(game_time(3, 65), 1598554800000);
    assert_eq!(game_time(3, 98), 1598684400000);
    assert_eq!(game_time(3, 99), 1598706000000);
    assert_eq!(game_time(3, 111), 1598749200000);

    assert_eq!(game_time(4, 0), 1598889600000);
    assert_eq!(game_time(4, 32), 1599004800000);
    assert_eq!(game_time(4, 65), 1599123600000);
    assert_eq!(game_time(4, 98), 1599242400000);
    assert_eq!(game_time(4, 99), 1599310800000);
    assert_eq!(game_time(4, 112), 1599357600000);
}
