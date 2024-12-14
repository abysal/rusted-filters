use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Serialize)]
pub struct IntRange(pub Vec<i32>);

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum StateData {
    Boolean(Vec<bool>),
    IntRange(IntRange),
    String(Vec<String>),
}

impl<'de> Deserialize<'de> for IntRange {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct MinMaxInternal {
            pub min: i32,
            pub max: i32,
        }

        #[derive(Deserialize)]
        #[serde(untagged)]
        enum RawValueRange {
            Array(Vec<i32>),
            MinMax { values: MinMaxInternal },
        }

        let raw = RawValueRange::deserialize(deserializer)?;
        let list = match raw {
            RawValueRange::Array(values) => values,
            RawValueRange::MinMax { values } => (values.min..=values.max).collect(),
        };
        Ok(IntRange(list))
    }
}
