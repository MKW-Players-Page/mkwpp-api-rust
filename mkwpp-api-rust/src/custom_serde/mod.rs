use serde::{Deserializer, Serializer};

pub trait DateAsTimestampNumber {
    fn serialize_as_timestamp<S>(x: &Self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer;

    fn deserialize_from_timestamp<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
        Self: Sized;
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

    fn deserialize_from_timestamp<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
        Self: Sized,
    {
        let x: i64 = serde::de::Deserialize::deserialize(deserializer)?;

        chrono::DateTime::from_timestamp(x, 0)
            .ok_or(serde::de::Error::custom(
                "Could not convert timestamp to date",
            ))
            .map(|x| x.date_naive())
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

    fn deserialize_from_timestamp<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
        Self: Sized,
    {
        let x: Option<i64> = serde::de::Deserialize::deserialize(deserializer)?;
        match x {
            None => Ok(None),
            Some(x) => chrono::DateTime::from_timestamp(x, 0)
                .ok_or(serde::de::Error::custom(
                    "Could not convert timestamp to date",
                ))
                .map(|x| Some(x.date_naive())),
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

    fn deserialize_from_timestamp<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
        Self: Sized,
    {
        let x: i64 = serde::de::Deserialize::deserialize(deserializer)?;
        chrono::DateTime::from_timestamp(x, 0).ok_or(serde::de::Error::custom(
            "Could not convert timestamp to date",
        ))
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

    fn deserialize_from_timestamp<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
        Self: Sized,
    {
        let x: Option<i64> = serde::de::Deserialize::deserialize(deserializer)?;
        match x {
            None => Ok(None),
            Some(x) => chrono::DateTime::from_timestamp(x, 0)
                .ok_or(serde::de::Error::custom(
                    "Could not convert timestamp to date",
                ))
                .map(Some),
        }
    }
}

impl DateAsTimestampNumber for chrono::DateTime<chrono::Local> {
    fn serialize_as_timestamp<S>(x: &Self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        s.serialize_i64(x.timestamp())
    }

    fn deserialize_from_timestamp<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
        Self: Sized,
    {
        let x: chrono::DateTime<chrono::Utc> = serde::de::Deserialize::deserialize(deserializer)?;
        Ok(x.with_timezone(&chrono::Local))
    }
}

impl DateAsTimestampNumber for Option<chrono::DateTime<chrono::Local>> {
    fn serialize_as_timestamp<S>(x: &Self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match x {
            Some(v) => chrono::DateTime::<chrono::Local>::serialize_as_timestamp(v, s),
            None => s.serialize_none(),
        }
    }

    fn deserialize_from_timestamp<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
        Self: Sized,
    {
        let x: Option<chrono::DateTime<chrono::Utc>> =
            serde::de::Deserialize::deserialize(deserializer)?;
        Ok(x.map(|x| x.with_timezone(&chrono::Local)))
    }
}
