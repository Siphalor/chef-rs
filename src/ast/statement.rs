use pest::iterators::Pair;

use crate::ast::ingredient::expect_ingredient_name;
use crate::ast::util::{BakingDishId, expect_baking_dish, expect_mixing_bowl, mixin_bowl_from, MixingBowlId, PairsExtensions, ParseError, ParseResult};
use crate::grammar::Rule;

#[derive(Debug)]
pub enum Statement {
    Read {
        ingredient: String
    },
    CheckInput {
        ingredient: String
    },
    Push {
        ingredient: String,
        mixing_bowl: MixingBowlId,
    },
    Pop {
        ingredient: String,
        mixing_bowl: MixingBowlId,
    },
    Add {
        ingredient: String,
        mixing_bowl: MixingBowlId,
    },
    Subtract {
        ingredient: String,
        mixing_bowl: MixingBowlId,
    },
    Multiply {
        ingredient: String,
        mixing_bowl: MixingBowlId,
    },
    Divide {
        ingredient: String,
        mixing_bowl: MixingBowlId,
    },
    AddAll {
        mixing_bowl: MixingBowlId,
    },
    ToChar {
        ingredient: String,
    },
    ToCharAll {
        mixing_bowl: MixingBowlId,
    },
    MoveDynamic {
        ingredient: String,
        mixing_bowl: MixingBowlId,
    },
    MoveStatic {
        offset: u64,
        mixin_bowl: MixingBowlId,
    },
    Sort {
        mixing_bowl: MixingBowlId,
    },
    Shuffle {
        mixing_bowl: MixingBowlId,
    },
    Clear {
        mixing_bowl: MixingBowlId,
    },
    SetResult {
        mixing_bowl: MixingBowlId,
        baking_dish: BakingDishId,
    },
    Examine {
        ingredient: String,
    },
    ExamineBowl {
        mixing_bowl: MixingBowlId,
    },
    Loop {
        test_ingredient: String,
        decrement_ingredient: Option<String>,
        statements: Vec<Statement>,
    },
    BreakLoop,
    CallAuxiliary {
        recipe: String,
    },
    Return {
        count: MixingBowlId,
    },
}

impl Statement {
    pub fn from(pair: Pair<Rule>) -> ParseResult<Statement> {
        let span = pair.as_span();
        let rule = pair.as_rule();
        let mut pairs = pair.into_inner();
        match rule {
            Rule::takeStatement => {
                Ok(Statement::Read {
                    ingredient: expect_ingredient_name(&mut pairs, &span)?,
                })
            }
            Rule::checkStatement => {
                Ok(Statement::CheckInput {
                    ingredient: expect_ingredient_name(&mut pairs, &span)?,
                })
            }
            Rule::putStatement => {
                Ok(Statement::Push {
                    ingredient: expect_ingredient_name(&mut pairs, &span)?,
                    mixing_bowl: expect_mixing_bowl(&mut pairs)?,
                })
            }
            Rule::foldStatement => {
                Ok(Statement::Pop {
                    ingredient: expect_ingredient_name(&mut pairs, &span)?,
                    mixing_bowl: expect_mixing_bowl(&mut pairs)?,
                })
            }
            Rule::addStatement => {
                Ok(Statement::Add {
                    ingredient: expect_ingredient_name(&mut pairs, &span)?,
                    mixing_bowl: expect_mixing_bowl(&mut pairs)?,
                })
            }
            Rule::addDryStatement => {
                Ok(Statement::AddAll {
                    mixing_bowl: expect_mixing_bowl(&mut pairs)?,
                })
            }
            Rule::removeStatement => {
                Ok(Statement::Subtract {
                    ingredient: expect_ingredient_name(&mut pairs, &span)?,
                    mixing_bowl: expect_mixing_bowl(&mut pairs)?,
                })
            }
            Rule::combineStatement => {
                Ok(Statement::Multiply {
                    ingredient: expect_ingredient_name(&mut pairs, &span)?,
                    mixing_bowl: expect_mixing_bowl(&mut pairs)?,
                })
            }
            Rule::divideStatement => {
                Ok(Statement::Divide {
                    ingredient: expect_ingredient_name(&mut pairs, &span)?,
                    mixing_bowl: expect_mixing_bowl(&mut pairs)?,
                })
            }
            Rule::liquefyStatement => {
                Ok(Statement::ToChar {
                    ingredient: expect_ingredient_name(&mut pairs, &span)?,
                })
            }
            Rule::liquefyBowlStatement => {
                Ok(Statement::ToCharAll {
                    mixing_bowl: expect_mixing_bowl(&mut pairs)?,
                })
            }
            Rule::stirStatement => {
                Ok(Statement::MoveDynamic {
                    ingredient: expect_ingredient_name(&mut pairs, &span)?,
                    mixing_bowl: expect_mixing_bowl(&mut pairs)?,
                })
            }
            Rule::stirBowlStatement => {
                let mut mixing_bowl = 0;
                let mut offset = 0;
                for pair in pairs {
                    match pair.as_rule() {
                        Rule::mixingBowl => {
                            mixing_bowl = mixin_bowl_from(pair)?;
                        }
                        Rule::stirBowlTime => {
                            offset = pair.as_str().parse().map_err(|err| {
                                ParseError::Generic {
                                    message: format!("failed to parse stir time: {:?}", err),
                                    begin: pair.as_span().start_pos().line_col(),
                                }
                            })?;
                        }
                        _ => {}
                    }
                }
                Ok(Statement::MoveStatic {
                    mixin_bowl: mixing_bowl,
                    offset,
                })
            }
            Rule::shakeBowlStatement => {
                Ok(Statement::Sort {
                    mixing_bowl: expect_mixing_bowl(&mut pairs)?,
                })
            }
            Rule::mixBowlStatement => {
                Ok(Statement::Shuffle {
                    mixing_bowl: expect_mixing_bowl(&mut pairs)?,
                })
            }
            Rule::cleanBowlStatement => {
                Ok(Statement::Clear {
                    mixing_bowl: expect_mixing_bowl(&mut pairs)?,
                })
            }
            Rule::pourBowlStatement => {
                let mut mixing_bowl = 0;
                let next = pairs.peek().unwrap();
                match next.as_rule() {
                    Rule::mixingBowl => {
                        pairs.next();
                        mixing_bowl = mixin_bowl_from(next)?
                    }
                    _ => {}
                }
                Ok(Statement::SetResult {
                    mixing_bowl,
                    baking_dish: expect_baking_dish(&mut pairs)?,
                })
            }
            Rule::examineStatement => {
                if let Some(pair) = pairs.try_next(Rule::ingredientName) {
                    Ok(Statement::Examine {
                        ingredient: pair.as_str().to_lowercase(),
                    })
                } else {
                    Ok(Statement::ExamineBowl {
                        mixing_bowl: expect_mixing_bowl(&mut pairs)?,
                    })
                }
            }
            Rule::loopBlock => {
                let begin = pairs.expect_next(Rule::loopBeginStatement, &span)?;
                let begin_span = begin.as_span();
                let mut begin = begin.into_inner();
                let verb = begin.expect_next(Rule::loopVerb, &begin_span)?.as_str().to_lowercase();
                let test_ingredient = expect_ingredient_name(&mut begin, &begin_span)?;
                let mut decrement_ingredient = None;

                let mut statements = Vec::new();
                for pair in pairs {
                    if pair.as_rule() != Rule::loopEndStatement {
                        statements.push(Statement::from(pair)?);
                    } else {
                        let end_span = pair.as_span();
                        let mut end = pair.into_inner();
                        if end.peek().unwrap().as_rule() == Rule::ingredientName {
                            decrement_ingredient = Some(end.next().unwrap().as_str().to_lowercase());
                        }
                        let end_verb = end.expect_next(Rule::loopVerb, &end_span)?.as_str();
                        if !end_verb.to_lowercase().starts_with(&verb) {
                            return Err(ParseError::Generic {
                                message: format!("loop verbs do not match: {} and {}", verb, end_verb),
                                begin: span.start_pos().line_col(),
                            });
                        }
                        break;
                    }
                }

                Ok(Statement::Loop {
                    statements,
                    test_ingredient,
                    decrement_ingredient,
                })
            }
            Rule::loopBreakStatement => { Ok(Statement::BreakLoop) }
            Rule::serveWithStatement => {
                Ok(Statement::CallAuxiliary {
                    recipe: pairs.expect_next(Rule::recipeName, &span)?.as_str().to_lowercase(),
                })
            }
            Rule::refrigerateStatement => {
                Ok(Statement::Return {
                    count: pairs.try_next(Rule::refrigerateDuration)
                        .map_or(Ok(0), |val| val.as_str().parse())
                        .map_err(|err| ParseError::Generic {
                            message: format!("failed to parse refrigeration duration: {:?}", err),
                            begin: span.start_pos().line_col(),
                        })?
                })
            }
            Rule::servesStatement => {
                Ok(Statement::Return {
                    count: pairs.expect_next(Rule::servesPeople, &span)?.as_str().parse()
                        .map_err(|err| ParseError::Generic {
                            message: format!("failed to parse the amount of people served: {:?}", err),
                            begin: span.start_pos().line_col(),
                        })?
                })
            }
            _ => {
                Err(ParseError::Generic {
                    message: format!("unknown rule: {:?}", rule),
                    begin: span.start_pos().line_col(),
                })
            }
        }
    }
}
