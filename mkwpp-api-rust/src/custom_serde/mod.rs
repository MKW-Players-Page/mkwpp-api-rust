use serde::{Serialize, Serializer};

pub trait DateAsTimestampNumber {
    fn serialize<S>(x: &Self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer;
}

impl DateAsTimestampNumber for chrono::NaiveDate {
    fn serialize<S>(x: &Self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        x.and_hms_opt(0, 0, 0).unwrap().and_utc().serialize(s)
    }
}

impl DateAsTimestampNumber for chrono::DateTime<chrono::Utc> {
    fn serialize<S>(x: &Self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        s.serialize_i64(x.timestamp())
    }
}

impl<T: DateAsTimestampNumber> DateAsTimestampNumber for Option<T> {
    fn serialize<S>(x: &Self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match x {
            Some(v) => T::serialize(v, s),
            None => s.serialize_none(),
        }
    }
}
