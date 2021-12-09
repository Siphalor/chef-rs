use std::fmt::{Display, Formatter};
use crate::ast::ingredient::IngredientDefinition;

#[derive(Debug, Clone)]
pub struct Ingredient {
    pub value: f64,
    pub liquid: bool,
}

impl Ingredient {
    pub fn instantiate(def: &IngredientDefinition) -> (String, Ingredient) {
        (def.name().clone(), Ingredient {
            value: def.initial_value().unwrap_or(0.0),
            liquid: def.liquid(),
        })
    }
}

impl Display for Ingredient {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.liquid {
            write!(f, "{}", char::from_u32(self.value as u32).map_or_else(|| "<missingno>".to_string(), |c| c.to_string()))
        } else {
            write!(f, "{}", self.value)
        }
    }
}
