#[derive(Debug, Clone)]
pub enum PropertyValue {
    String(String),
    Item(String),
    Coord { latitude: f64, longitude: f64 },
}

impl PropertyValue {
    pub fn value(&self) -> &str {
        match self {
            PropertyValue::String(e) => e,
            PropertyValue::Item(e) => e,
            // Note: this method is used only in tests, we can panic
            PropertyValue::Coord { .. } => panic!("unable to convert coord to string"),
        }
    }
}

impl std::ops::Deref for PropertyValue {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.value()
    }
}

/// Simple representation of a wikibase entity
#[derive(Debug, Clone)]
pub struct Entity {
    pub id: String,
    pub properties: std::collections::HashMap<String, Vec<PropertyValue>>,
    pub label: String,
}
