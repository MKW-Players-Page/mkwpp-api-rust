use serde::Serializer;

pub trait DateAsTimestampNumber {
    fn serialize_as_timestamp<S>(x: &Self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer;
}

impl DateAsTimestampNumber for chrono::NaiveDate {
    fn serialize_as_timestamp<S>(x: &Self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        chrono::DateTime::<chrono::Utc>::serialize_as_timestamp(
            &x.and_hms_opt(0, 0, 0).unwrap().and_utc(),
            s,
        )
    }
}

impl DateAsTimestampNumber for Option<chrono::NaiveDate> {
    fn serialize_as_timestamp<S>(x: &Self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match x {
            Some(v) => chrono::NaiveDate::serialize_as_timestamp(v, s),
            None => s.serialize_none(),
        }
    }
}

impl DateAsTimestampNumber for chrono::DateTime<chrono::Utc> {
    fn serialize_as_timestamp<S>(x: &Self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        s.serialize_i64(x.timestamp())
    }
}

impl DateAsTimestampNumber for Option<chrono::DateTime<chrono::Utc>> {
    fn serialize_as_timestamp<S>(x: &Self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match x {
            Some(v) => chrono::DateTime::<chrono::Utc>::serialize_as_timestamp(v, s),
            None => s.serialize_none(),
        }
    }
}
