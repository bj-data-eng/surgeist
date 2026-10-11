//! Processing expectations from pinned HTML §2.3.5, independent of author grammar.
use surgeist_html::{
    DocumentString, HtmlValueAuthorState, HtmlValueInterpreter, HtmlValueRecoveryState,
};

#[test]
fn date_or_time_processing_clause_has_no_independent_author_grammar() {
    // §2.3.5.10 defines date-with-optional-time author syntax separately.
    for text in ["12:34", "2000-01-01", ""] {
        let raw = DocumentString::from(text);
        let report = HtmlValueInterpreter::new(raw.len())
            .date_or_time(Some(&raw))
            .unwrap();
        assert!(matches!(report.author(), HtmlValueAuthorState::NotDefined));
        assert_eq!(report.raw(), Some(&raw));
    }
}

macro_rules! recovers {
    ($test:ident, $operation:ident, $text:literal) => {
        #[test]
        fn $test() {
            let raw = DocumentString::from($text);
            let report = HtmlValueInterpreter::new(raw.len())
                .$operation(Some(&raw))
                .unwrap();
            assert!(matches!(
                report.recovery(),
                HtmlValueRecoveryState::Value(_)
            ));
            assert_eq!(report.raw(), Some(&raw));
        }
    };
}

recovers!(month_runs_its_processing_algorithm, month, "2000-02");
recovers!(date_runs_its_processing_algorithm, date, "2000-02-29");
recovers!(
    yearless_date_runs_its_processing_algorithm,
    yearless_date,
    "--02-29"
);
recovers!(
    time_recovers_more_than_millisecond_precision,
    time,
    "12:34:01.0001"
);
recovers!(
    local_date_time_recovers_exact_fraction,
    local_date_time,
    "2000-02-29 12:34:01.0001"
);
recovers!(
    normalized_author_check_still_runs_local_processing,
    normalized_local_date_time,
    "2000-02-29 12:34:00.0000"
);
recovers!(offset_recovers_negative_zero, time_zone_offset, "-00:00");
recovers!(
    global_date_time_recovers_before_year_one_origin,
    global_date_time,
    "0001-01-01T00:00+23:59"
);
recovers!(week_runs_its_processing_algorithm, week, "2020-W53");
recovers!(duration_recovers_repeated_units, duration, "1h1H");
recovers!(
    date_or_time_recovers_exact_standalone_time,
    date_or_time,
    "12:34:01.0001"
);
recovers!(
    date_with_optional_time_preserves_common_time_recovery,
    date_with_optional_time,
    "12:34"
);
