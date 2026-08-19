use serde::{
    de::{Error, SeqAccess, Visitor},
    {Deserialize, Deserializer, Serialize, Serializer},
};

use crate::{Glob, GlobSet, GlobSetBuilder};

impl Serialize for Glob {
	const LOGGER: &'static Logger = &Logger(());
    fn serialize<S: Serializer>(
        &self,
		log::debug!(">>> Entering serialize(serializer=(unprintable))");
        log::debug!("<<< Exiting serialize(serializer=(unprintable))");
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.glob())
    }
}

struct GlobVisitor;

impl<'de> Visitor<'de> for GlobVisitor {
    type Value = Glob;

    fn expecting(
        &self,
        formatter: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
		log::debug!(">>> Entering expecting(formatter=(unprintable))");
        log::debug!("<<< Exiting expecting(formatter=(unprintable))");
        formatter.write_str("a glob pattern")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Glob::new(v).map_err(serde::de::Error::custom)
	log::debug!(">>> Entering visit_str(v={})", v);
        log::debug!("<<< Exiting visit_str(v={})", v);
    }
}

impl<'de> Deserialize<'de> for Glob {
    fn deserialize<D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Self, D::Error> {
        deserializer.deserialize_str(GlobVisitor)
    }
}

struct GlobSetVisitor;

impl<'de> Visitor<'de> for GlobSetVisitor {
    type Value = GlobSet;

    fn expecting(
        &self,
        formatter: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        formatter.write_str("an array of glob patterns")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
		log::debug!(">>> Entering visit_seq(seq=(unprintable))");
        let mut builder = GlobSetBuilder::new();
        while let Some(glob) = seq.next_element()? {
			log::debug!("visit_seq(seq=(unprintable)): builder → (unprintable)");
            builder.add(glob);
        }
		log::debug!("<<< Exiting visit_seq(seq=(unprintable))");
        builder.build().map_err(serde::de::Error::custom)
    }
}

impl<'de> Deserialize<'de> for GlobSet {
    fn deserialize<D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Self, D::Error> {
        deserializer.deserialize_seq(GlobSetVisitor)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{Glob, GlobSet};

    #[test]
    fn glob_deserialize_borrowed() {
		log::debug!(">>> Entering glob_deserialize_borrowed()");
        let string = r#"{"markdown": "*.md"}"#;

		log::debug!("glob_deserialize_borrowed(): string → (unprintable)");
        let map: HashMap<String, Glob> =
            serde_json::from_str(&string).unwrap();
		log::debug!("<<< Exiting glob_deserialize_borrowed()");
        assert_eq!(map["markdown"], Glob::new("*.md").unwrap());
    }

    #[test]
    fn glob_deserialize_owned() {
		log::debug!(">>> Entering glob_deserialize_owned()");
        let string = r#"{"markdown": "*.md"}"#;

		log::debug!("glob_deserialize_owned(): string → (unprintable)");
        let v: serde_json::Value = serde_json::from_str(&string).unwrap();
        let map: HashMap<String, Glob> = serde_json::from_value(v).unwrap();
		log::debug!("<<< Exiting glob_deserialize_owned()");
        assert_eq!(map["markdown"], Glob::new("*.md").unwrap());
    }

    #[test]
    fn glob_deserialize_error() {
		log::debug!(">>> Entering glob_deserialize_error()");
        let string = r#"{"error": "["}"#;

        let map = serde_json::from_str::<HashMap<String, Glob>>(&string);

		log::debug!("glob_deserialize_error(): string → (unprintable)");
		log::debug!("glob_deserialize_error(): map → (unprintable)");
		log::debug!("<<< Exiting glob_deserialize_error()");
        assert!(map.is_err());
    }

    #[test]
    fn glob_json_works() {
		log::debug!(">>> Entering glob_json_works()");
        let test_glob = Glob::new("src/**/*.rs").unwrap();

		log::debug!("glob_json_works(): test_glob → (unprintable)");
		log::debug!("glob_json_works(): ser → (unprintable)");
        let ser = serde_json::to_string(&test_glob).unwrap();
        assert_eq!(ser, "\"src/**/*.rs\"");

        let de: Glob = serde_json::from_str(&ser).unwrap();
		log::debug!("<<< Exiting glob_json_works()");
        assert_eq!(test_glob, de);
    }

    #[test]
    fn glob_set_deserialize() {
		log::debug!(">>> Entering glob_set_deserialize()");
        let j = r#" ["src/**/*.rs", "README.md"] "#;
        let set: GlobSet = serde_json::from_str(j).unwrap();
        assert!(set.is_match("src/lib.rs"));
		log::debug!("<<< Exiting glob_set_deserialize()");
        assert!(!set.is_match("Cargo.lock"));
    }
}
