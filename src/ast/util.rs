use pest::iterators::{Pair, Pairs};
use pest::{RuleType};
use crate::grammar::Rule;

pub type ParseResult<T> = Result<T, ParseError>;
pub type MixingBowlId = u8;
pub type BakingDishId = u8;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParseError {
    Generic {
        message: String,
        begin: (usize, usize),
    },
    RuleNotFound {
        message: String,
        begin: (usize, usize),
        end: (usize, usize),
    }
}

pub fn mixin_bowl_from(pair: Pair<Rule>) -> ParseResult<MixingBowlId> {
    pair.into_inner().try_next(Rule::mixingBowlNumber)
        .map(|pair| {
            pair.as_str().parse().map_err(|err| {
                ParseError::Generic {
                    message: format!("failed to parse mixin bowl number: {}", err),
                    begin: pair.as_span().start_pos().line_col(),
                }
            })
        })
        .unwrap_or(Ok(1))
}

pub fn baking_dish_from(pair: Pair<Rule>) -> ParseResult<BakingDishId> {
    pair.into_inner().try_next(Rule::bakingDishNumber)
        .map(|pair| {
            pair.as_str().parse().map_err(|err| {
                ParseError::Generic {
                    message: format!("failed to parse baking dish number: {}", err),
                    begin: pair.as_span().start_pos().line_col(),
                }
            })
        })
        .unwrap_or(Ok(1))
}

pub fn expect_mixing_bowl(pairs: &mut Pairs<Rule>) -> ParseResult<MixingBowlId> {
    pairs.try_next(Rule::mixingBowl)
        .map_or(Ok(1), |val| mixin_bowl_from(val))
}

pub fn expect_baking_dish(pairs: &mut Pairs<Rule>) -> ParseResult<MixingBowlId> {
    pairs.try_next(Rule::bakingDish)
        .map_or(Ok(1), |val| mixin_bowl_from(val))
}

pub trait PairsExtensions<R> {
    fn expect_next(&mut self, rule: R, parent_span: &pest::Span) -> ParseResult<Pair<R>>;
    fn try_next(&mut self, rule: R) -> Option<Pair<R>>;
}

impl<R: RuleType> PairsExtensions<R> for Pairs<'_, R> {
    fn expect_next(&mut self, rule: R, parent_span: &pest::Span) -> ParseResult<Pair<R>> {
        if let Some(next) = self.next() {
            if next.as_rule() == rule {
                Ok(next)
            } else {
                Err(ParseError::RuleNotFound {
                    message: format!("rule not found: expected: {:?}, found {:?}", rule, next.as_rule()),
                    begin: parent_span.start_pos().line_col(),
                    end: parent_span.end_pos().line_col(),
                })
            }
        } else {
            Err(ParseError::RuleNotFound {
                message: format!("no more rules, expected {:?}", rule),
                begin: parent_span.start_pos().line_col(),
                end: parent_span.end_pos().line_col(),
            })
        }
    }

    fn try_next(&mut self, rule: R) -> Option<Pair<R>> {
        self.peek().and_then(|pair| {
            if pair.as_rule() == rule {
                self.next();
                Some(pair)
            } else {
                None
            }
        })
    }
}
