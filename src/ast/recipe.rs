use getset::Getters;
use linked_hash_map::LinkedHashMap;
use pest::iterators::Pair;

use crate::ast::ingredient::IngredientDefinitionList;
use crate::ast::statement::Statement;
use crate::ast::util::{PairsExtensions, ParseResult};
use crate::grammar::Rule;

pub type Recipes = LinkedHashMap<String, Recipe>;

pub fn recipes_from(recipes_rule: Pair<Rule>) -> ParseResult<Recipes> {
    let mut recipe_rules = recipes_rule.into_inner();
    let mut recipes = Recipes::new();
    loop {
        if let Some(recipe_rule) = recipe_rules.try_next(Rule::recipe) {
            let recipe = Recipe::from(recipe_rule)?;
            recipes.insert(recipe.name().clone(), recipe);
        } else {
            break;
        }
    }
    Ok(recipes)
}

#[derive(Debug, Getters)]
pub struct Recipe {
    #[getset(get="pub")]
    name: String,
    #[getset(get="pub")]
    comment: Option<String>,
    #[getset(get="pub")]
    ingredients: IngredientDefinitionList,
    #[getset(get="pub")]
    statements: Vec<Statement>,
}

impl Recipe {
    pub fn new(name: &str, comment: &str) -> Recipe {
        Recipe {
            name: name.to_string(),
            comment: Some(comment.to_string()),
            ingredients: IngredientDefinitionList::empty(),
            statements: Vec::new(),
        }
    }

    pub fn from(recipe_rule: Pair<Rule>) -> ParseResult<Recipe> {
        let recipe_span = recipe_rule.as_span();
        let mut recipe_pairs = recipe_rule.into_inner();
        let name = recipe_pairs.expect_next(Rule::recipeName, &recipe_span)?.as_str().to_lowercase();
        let comment = recipe_pairs.try_next(Rule::recipeComment).map(|val| val.as_str().to_string());
        let ingredient_list = recipe_pairs.expect_next(Rule::ingredientList, &recipe_span).and_then(IngredientDefinitionList::from)?;
        let method_pair = recipe_pairs.expect_next(Rule::method, &recipe_span)?;
        let mut statements = Vec::new();
        for pair in method_pair.into_inner() {
            statements.push(Statement::from(pair)?);
        }

        Ok(Recipe {
            name,
            comment,
            ingredients: ingredient_list,
            statements,
        })
    }
}
