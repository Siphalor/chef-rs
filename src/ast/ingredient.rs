use getset::{CopyGetters, Getters};
use pest::iterators::{Pair, Pairs};
use crate::ast::util::{PairsExtensions, ParseError, ParseResult};
use crate::ast::util::ParseError::Generic;
use crate::grammar::Rule;

#[derive(Debug, CopyGetters, Getters)]
pub struct IngredientDefinition {
    #[getset(get_copy="pub")]
    initial_value: Option<f64>,
    #[getset(get_copy="pub")]
    liquid: bool,
    #[getset(get="pub")]
    name: String,
}

impl IngredientDefinition {
    pub fn from(ingredient_definition_rule: Pair<Rule>) -> ParseResult<IngredientDefinition> {
        let mut value = None;
        let mut liquid = false;
        let mut name = None;
        let list_rule_span = ingredient_definition_rule.as_span();

        for pair in ingredient_definition_rule.into_inner() {
            match pair.as_rule() {
                Rule::ingredientInitialValue => {
                    if let Ok(parsed_value) = pair.as_str().parse() {
                        value = Some(parsed_value);
                    } else {
                        return Err(Generic {
                            message: "failed to parse number".into(),
                            begin: pair.as_span().start_pos().line_col(),
                        });
                    }
                },
                Rule::ingredientMeasureType => {
                    match pair.as_str().to_lowercase().as_str() {
                        "heaped" | "level" => {
                            liquid = false;
                        },
                        _ => {},
                    }
                },
                Rule::ingredientMeasure => {
                    if let Some(measure) = pair.into_inner().next() {
                        match measure.as_rule() {
                            Rule::ingredientMeasureDry => {
                                liquid = false;
                            },
                            Rule::ingredientMeasureLiquid => {
                                liquid = true;
                            },
                            _ => {}
                        }
                    }
                }
                Rule::ingredientName => {
                    name = Some(pair.as_str().to_string());
                }
                _ => {}
            }
        }

        if let Some(name) = name {
            Ok(IngredientDefinition {
                initial_value: value,
                liquid,
                name,
            })
        } else {
            Err(ParseError::RuleNotFound {
                message: "name not found for ingredient".to_string(),
                begin: list_rule_span.start_pos().line_col(),
                end: list_rule_span.end_pos().line_col(),
            })
        }
    }
}

#[derive(Debug, Getters)]
pub struct IngredientDefinitionList {
    #[getset(get="pub")]
    definitions: Vec<IngredientDefinition>,
}

impl IngredientDefinitionList {
    pub fn empty() -> IngredientDefinitionList {
        IngredientDefinitionList {
            definitions: Vec::new(),
        }
    }

    pub fn from(ingredient_list_rule: Pair<Rule>) -> ParseResult<IngredientDefinitionList> {
        let mut definitions = Vec::new();
        for pair in ingredient_list_rule.into_inner() {
            if pair.as_rule() == Rule::ingredientDefinition {
                match IngredientDefinition::from(pair) {
                    Ok(definition) => {
                        definitions.push(definition);
                    }
                    Err(err) => {
                        return Err(err);
                    }
                }
            }
        }
        return Ok(IngredientDefinitionList { definitions })
    }
}

pub fn expect_ingredient_name(pairs: &mut Pairs<Rule>, span: &pest::Span) -> ParseResult<String> {
    pairs.expect_next(Rule::ingredientName, span)
        .map(|pair| pair.as_str().to_lowercase())
}
