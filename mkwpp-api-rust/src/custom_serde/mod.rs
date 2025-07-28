use serde::{Deserializer, Serializer, ser::SerializeSeq};

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

pub trait ChadsoftIDConversion {
    fn serialize_as_string<S>(x: &Self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer;

    fn deserialize_from_string<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
        Self: Sized;
}

impl ChadsoftIDConversion for i64 {
    fn serialize_as_string<S>(x: &Self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let x = (*x) as u64;
        let x = format!("{x:016X}");
        s.serialize_str(&x)
    }

    fn deserialize_from_string<'de, D>(deserializer: D) -> Result<i64, D::Error>
    where
        D: Deserializer<'de>,
        Self: Sized,
    {
        let x: &str = serde::de::Deserialize::deserialize(deserializer)?;
        let x = u64::from_str_radix(x, 16)
            .map_err(|_| serde::de::Error::custom("Could not convert timestamp to date"))?;
        Ok(x as i64)
    }
}
impl ChadsoftIDConversion for Vec<i64> {
    fn serialize_as_string<S>(x: &Self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = s.serialize_seq(Some(x.len()))?;
        for x in x {
            seq.serialize_element(&format!("{x:016X}"))?;
        }
        seq.end()
    }

    fn deserialize_from_string<'de, D>(deserializer: D) -> Result<Vec<i64>, D::Error>
    where
        D: Deserializer<'de>,
        Self: Sized,
    {
        let x: Vec<&str> = serde::de::Deserialize::deserialize(deserializer)?;
        x.into_iter()
            .map(|x| {
                u64::from_str_radix(x, 16)
                    .map(|x| x as i64)
                    .map_err(|_| serde::de::Error::custom("Could not convert timestamp to date"))
            })
            .collect()
    }
}
