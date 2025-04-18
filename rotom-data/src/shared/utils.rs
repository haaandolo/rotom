use chrono::Utc;

pub fn current_timestamp_utc() -> u64 {
    Utc::now().timestamp_millis() as u64
}

pub fn snapshot_symbol_default_value() -> String {
    String::from("snapshot")
}

pub fn number_to_precision(places: usize) -> f64 {
    10f64.powi(-(places as i32))
}

pub fn precision_to_number(value: f64) -> usize {
    if value <= 0.0 {
        return 0;
    }

    -value.log10().round() as usize
}

// rtp: Round to precision
pub fn rtp(value: f64, precision: f64) -> f64 {
    if precision <= 0.0 {
        return value;
    }

    let scaling_factor = 1.0 / precision;
    (value * scaling_factor).floor() / scaling_factor
}

#[cfg(test)]
mod test {
    use super::{number_to_precision, precision_to_number, rtp};

    #[test]
    fn test_precision_to_number() {
        let dec1 = 0.1;
        let dec2 = 0.01;
        let dec3 = 0.001;
        let dec4 = 0.0001;
        let dec5 = 0.00001;
        let dec6 = 0.0;
        let dec7 = 1.0;

        let dec1_res = precision_to_number(dec1);
        assert_eq!(dec1_res, 1);

        let dec2_res = precision_to_number(dec2);
        assert_eq!(dec2_res, 2);

        let dec3_res = precision_to_number(dec3);
        assert_eq!(dec3_res, 3);

        let dec4_res = precision_to_number(dec4);
        assert_eq!(dec4_res, 4);

        let dec5_res = precision_to_number(dec5);
        assert_eq!(dec5_res, 5);

        let dec6_res = precision_to_number(dec6);
        assert_eq!(dec6_res, 0);

        let dec7_res = precision_to_number(dec7);
        assert_eq!(dec7_res, 0);
    }

    #[test]
    fn test_number_to_decimal_places() {
        let dp_1 = 1;
        let dp_2 = 3;
        let dp_3 = 5;
        let dp_4 = 10;
        let dp_5 = 0;

        let dp_1_res = number_to_precision(dp_1);
        assert_eq!(dp_1_res, 0.1);

        let dp_2_res = number_to_precision(dp_2);
        assert_eq!(dp_2_res, 0.001);

        let dp_3_res = number_to_precision(dp_3);
        assert_eq!(dp_3_res, 0.00001);

        let dp_4_res = number_to_precision(dp_4);
        assert_eq!(dp_4_res, 0.0000000001);

        let dp_5_res = number_to_precision(dp_5);
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

        let precision_1_res = rtp(value, precison_1);
        assert_eq!(precision_1_res, 10.18);

        let precision_2_res = rtp(value, precison_2);
        assert_eq!(precision_2_res, 10.186);

        let precision_3_res = rtp(value, precison_3);
        assert_eq!(precision_3_res, 10.1867);

        let precision_4_res = rtp(value, precison_4);
        assert_eq!(precision_4_res, 10.1867089);

        let precision_5_res = rtp(value, precison_5);
        assert_eq!(precision_5_res, 10.186708987);

        let precision_6_res = rtp(value, precison_6);
        assert_eq!(precision_6_res, 10.0);

        let precision_7_res = rtp(value, precison_7);
        assert_eq!(precision_7_res, 10.0);
    }
}
