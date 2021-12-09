use pest::iterators::Pairs;
use pest::Parser;

#[derive(pest_derive::Parser)]
#[grammar = "grammar/chef.pest"]
pub struct ChefParser;

pub fn parse(input: &str) -> Result<Pairs<Rule>, pest::error::Error<Rule>> {
    return ChefParser::parse(Rule::recipe, input)
}
