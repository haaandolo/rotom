use chrono::Utc;

pub fn current_timestamp_utc() -> u64 {
    Utc::now().timestamp_millis() as u64
}

pub fn snapshot_symbol_default_value() -> String {
    String::from("snapshot")
}

pub fn decimal_places_to_number(places: usize) -> f64 {
    10f64.powi(-(places as i32))
}

pub fn round_float_to_precision(value: f64, precision: f64) -> f64 {
    if precision <= 0.0 {
        return value;
    }

    (value / precision).floor() * precision
}

#[cfg(test)]
mod test {
    use super::{decimal_places_to_number, round_float_to_precision};

    #[test]
    fn test_decimal_places_to_number() {
        let dp_1 = 1;
        let dp_2 = 3;
        let dp_3 = 5;
        let dp_4 = 10;
        let dp_5 = 0;

        let dp_1_res = decimal_places_to_number(dp_1);
        assert_eq!(dp_1_res, 0.1);

        let dp_2_res = decimal_places_to_number(dp_2);
        assert_eq!(dp_2_res, 0.001);

        let dp_3_res = decimal_places_to_number(dp_3);
        assert_eq!(dp_3_res, 0.00001);

        let dp_4_res = decimal_places_to_number(dp_4);
        assert_eq!(dp_4_res, 0.0000000001);

        let dp_5_res = decimal_places_to_number(dp_5);
        assert_eq!(dp_5_res, 1.0);
    }

    #[test]
    fn test_round_to_precision() {
        let value = 10.186708987;
        let precison_1 = 0.01;
        let precison_2 = 0.001;
        let precison_3 = 0.0001;
        let precison_4 = 0.0000001;
        let precison_5 = 0.0;
        let precison_6 = 1.0;
        let precison_7 = 2.0;

        let precision_1_res = round_float_to_precision(value, precison_1);
        assert_eq!(precision_1_res, 10.18);

        let precision_2_res = round_float_to_precision(value, precison_2);
        assert_eq!(precision_2_res, 10.186);

        let precision_3_res = round_float_to_precision(value, precison_3);
        assert_eq!(precision_3_res, 10.1867);

        let precision_4_res = round_float_to_precision(value, precison_4);
        assert_eq!(precision_4_res, 10.1867089);

        let precision_5_res = round_float_to_precision(value, precison_5);
        assert_eq!(precision_5_res, 10.186708987);

        let precision_6_res = round_float_to_precision(value, precison_6);
        assert_eq!(precision_6_res, 10.0);

        let precision_7_res = round_float_to_precision(value, precison_7);
        assert_eq!(precision_7_res, 10.0);
    }
}
