use std::collections::HashMap;

use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::ast::recipe::Recipes;
use crate::ast::statement::Statement;
use crate::ast::util::{BakingDishId, MixingBowlId};
use crate::interpreter::ingredient::Ingredient;
use crate::interpreter::util::{LazyTreeMap, LazyTreeMapParent, read_char, read_number};

pub type MixingBowl = Vec<Ingredient>;
pub type BakingDish = Vec<Ingredient>;
pub type MixingBowls<'a> = LazyTreeMap<'a, MixingBowlId, MixingBowl>;
pub type MixingBowlsParent<'a> = LazyTreeMapParent<'a, MixingBowlId, MixingBowl>;
pub type BakingDishes<'a> = LazyTreeMap<'a, BakingDishId, BakingDish>;
pub type BakingDishesParent<'a> = LazyTreeMapParent<'a, BakingDishId, BakingDish>;
pub type Ingredients = HashMap<String, Ingredient>;

pub type InterpreterResult<T> = Result<T, InterpreterError>;
pub type InterpreterError = String;

pub struct Interpreter {
    recipes: Recipes,
}

impl Interpreter {
    pub fn new(recipes: Recipes) -> Interpreter {
        Interpreter { recipes }
    }

    pub fn run_main(&self) -> InterpreterResult<()> {
        self.run_recipe(self.recipes.front().unwrap().0.clone(), MixingBowlsParent::None, BakingDishesParent::None, &mut String::new())
            .map(|_| ())
    }

    pub fn run_recipe(&self, recipe_name: String, parent_bowls: MixingBowlsParent, parent_dishes: BakingDishesParent, read_buffer: &mut String) -> InterpreterResult<Option<MixingBowl>> {
        let recipe = self.recipes.get(&*recipe_name);
        if let None = recipe {
            return Err(format!("unknown recipe: {}", recipe_name));
        }
        let recipe = recipe.unwrap();

        let mut mixing_bowls = MixingBowls::new(parent_bowls, |_| MixingBowl::new());
        let mut baking_dishes = BakingDishes::new(parent_dishes, |_| BakingDish::new());
        let mut ingredients: Ingredients = recipe.ingredients().definitions().iter().map(Ingredient::instantiate).collect();

        match self.execute_statements(recipe.statements(), &mut mixing_bowls, &mut baking_dishes, &mut ingredients, read_buffer)? {
            ExecutionCode::Break => { return Err("unexpected break".to_string()) }
            _ => {}
        }

        Ok(mixing_bowls.get(&1).map(|bowl| bowl.to_owned()))
    }

    fn execute_statements<'a>(&self, statements: &Vec<Statement>, mixing_bowls: &'a mut MixingBowls, baking_dishes: &mut BakingDishes, ingredients: &mut Ingredients, read_buffer: &mut String) -> InterpreterResult<ExecutionCode> {
        for statement in statements {
            match statement {
                Statement::Read { ingredient: ingredient_name } => {
                    if let Some(ingredient) = ingredients.get_mut(ingredient_name) {
                        if ingredient.liquid {
                            ingredient.value = read_char(read_buffer) as f64;
                        } else {
                            ingredient.value = read_number(read_buffer);
                        }
                    } else {
                        ingredients.insert(ingredient_name.clone(), Ingredient {
                            value: read_number(read_buffer),
                            liquid: false,
                        });
                    }
                }
                Statement::Push { ingredient, mixing_bowl } => {
                    mixing_bowls.get_mut(mixing_bowl.clone()).push(
                        Self::expect_ingredient(&ingredients, ingredient)?.clone()
                    );
                }
                Statement::Pop { ingredient: ingredient_name, mixing_bowl } => {
                    let mixing_bowl = mixing_bowls.get_mut(mixing_bowl.clone());
                    if let Some(value) = mixing_bowl.pop() {
                        ingredients.insert(ingredient_name.clone(), value);
                    } else {
                        return Err("tried to pop from empty bowl!".to_string());
                    }
                }
                Statement::Add { ingredient, mixing_bowl } => {
                    Self::expect_bowl_top_mut(mixing_bowls, mixing_bowl)?.value +=
                        Self::expect_ingredient(&ingredients, ingredient)?.value;
                }
                Statement::Subtract { ingredient, mixing_bowl } => {
                    Self::expect_bowl_top_mut(mixing_bowls, mixing_bowl)?.value -=
                        Self::expect_ingredient(&ingredients, ingredient)?.value;
                }
                Statement::Multiply { ingredient, mixing_bowl } => {
                    Self::expect_bowl_top_mut(mixing_bowls, mixing_bowl)?.value *=
                        Self::expect_ingredient(&ingredients, ingredient)?.value;
                }
                Statement::Divide { ingredient, mixing_bowl } => {
                    Self::expect_bowl_top_mut(mixing_bowls, mixing_bowl)?.value /=
                        Self::expect_ingredient(&ingredients, ingredient)?.value;
                }
                Statement::AddAll { mixing_bowl } => {
                    Self::expect_bowl_top_mut(mixing_bowls, mixing_bowl)?.value +=
                        ingredients.values().filter(|i| !i.liquid).map(|i| i.value).sum::<f64>();
                }
                Statement::ToChar { ingredient } => {
                    Self::expect_ingredient_mut(ingredients, ingredient)?.liquid = true;
                }
                Statement::ToCharAll { mixing_bowl } => {
                    let mixing_bowl = mixing_bowls.get_mut(mixing_bowl.clone());
                    for ingredient in mixing_bowl {
                        ingredient.liquid = true;
                    }
                }
                Statement::MoveDynamic { mixing_bowl, ingredient } => {
                    let amount = Self::expect_ingredient(&ingredients, ingredient)?.value as usize;
                    let mixing_bowl = mixing_bowls.get_mut(mixing_bowl.clone());
                    if let Some(top) = mixing_bowl.pop() {
                        mixing_bowl.insert(mixing_bowl.len() - amount, top);
                    }
                }
                Statement::MoveStatic { mixin_bowl, offset } => {
                    let mixing_bowl = mixing_bowls.get_mut(mixin_bowl.clone());
                    if let Some(top) = mixing_bowl.pop() {
                        mixing_bowl.insert(mixing_bowl.len() - *offset as usize, top);
                    }
                }
                Statement::Shuffle { mixing_bowl } => {
                    let mixing_bowl = mixing_bowls.get_mut(mixing_bowl.clone());
                    let slice = mixing_bowl.as_mut_slice();
                    slice.shuffle(&mut thread_rng());
                }
                Statement::Clear { mixing_bowl } => {
                    mixing_bowls.get_mut(mixing_bowl.clone()).clear();
                }
                Statement::SetResult { mixing_bowl, baking_dish } => {
                    let baking_dish = baking_dishes.get_mut(baking_dish.clone());
                    for ingredient in mixing_bowls.get(mixing_bowl)
                        .ok_or_else(|| format!("no mixing bowl {}", mixing_bowl))?.iter().rev() {
                        baking_dish.push(ingredient.clone());
                    }
                }
                Statement::Loop { test_ingredient, decrement_ingredient, statements: loop_statements } => {
                    while (Self::expect_ingredient(ingredients, test_ingredient)?.value - 0.0).abs() > 0.0000000001 {
                        match self.execute_statements(loop_statements, mixing_bowls, baking_dishes, ingredients, read_buffer)? {
                            ExecutionCode::Normal => {},
                            ExecutionCode::Break => { break },
                            ExecutionCode::Return => { return Ok(ExecutionCode::Return) }
                        }
                        if let Some(decrement_ingredient) = decrement_ingredient {
                            Self::expect_ingredient_mut(ingredients, decrement_ingredient)?.value -= 1.0;
                        }
                    }
                }
                Statement::BreakLoop => {
                    return Ok(ExecutionCode::Break);
                }
                Statement::CallAuxiliary { recipe } => {
                    if let Some(result_bowl) = self.run_recipe(recipe.clone(), MixingBowlsParent::Other(&mixing_bowls), MixingBowlsParent::Other(&baking_dishes), read_buffer)? {
                        let target_bowl = mixing_bowls.get_mut(1);
                        for ingredient in result_bowl.into_iter().rev() {
                            target_bowl.push(ingredient);
                        }
                    }
                }
                Statement::Return { count } => {
                    if *count > 0 {
                        for i in 1..=*count {
                            if let Some(dish) = baking_dishes.get(&i) {
                                if dish.iter().any(|i| i.liquid) {
                                    for ingredient in dish {
                                        print!("{}", ingredient)
                                    }
                                } else {
                                    for ingredient in dish {
                                        print!("{}, ", ingredient)
                                    }
                                }
                            }
                        }
                    }
                    return Ok(ExecutionCode::Return);
                }
                _ => {
                    return Err(format!("unknown statement: {:?}", statement));
                }
            }
        }
        Ok(ExecutionCode::Normal)
    }

    fn expect_ingredient<'a>(ingredients: &'a Ingredients, ingredient_name: &String) -> InterpreterResult<&'a Ingredient> {
        ingredients.get(ingredient_name).ok_or_else(|| format!("no such ingredient: {}", ingredient_name))
    }

    fn expect_ingredient_mut<'a>(ingredients: &'a mut Ingredients, ingredient_name: &String) -> InterpreterResult<&'a mut Ingredient> {
        ingredients.get_mut(ingredient_name).ok_or_else(|| format!("no such ingredient: {}", ingredient_name))
    }

    fn expect_bowl_top_mut<'a>(mixing_bowls: &'a mut MixingBowls, mixing_bowl_id: &MixingBowlId) -> InterpreterResult<&'a mut Ingredient> {
        let mixing_bowl = mixing_bowls.get_mut(mixing_bowl_id.clone());
        mixing_bowl.last_mut().ok_or_else(|| format!("no ingredient in mixing bowl {}", mixing_bowl_id))
    }
}

enum ExecutionCode {
    Normal,
    Return,
    Break,
}
