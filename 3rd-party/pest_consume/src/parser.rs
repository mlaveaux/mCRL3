use crate::Error;
use crate::Nodes;
use pest::Parser as PestParser;
use pest::RuleType;

/// A trait that provides methods to parse strings.
/// Do not implement manually; instead use the [`parser`] macro provided by this crate.
///
/// [`parser`]: macro@crate::parser
pub trait Parser {
    type Rule: RuleType;
    #[doc(hidden)]
    type AliasedRule: RuleType;
    type Parser: PestParser<Self::Rule>;

    #[doc(hidden)]
    fn rule_alias(rule: Self::Rule) -> Self::AliasedRule;
    #[doc(hidden)]
    fn allows_shortcut(rule: Self::Rule) -> bool;

    /// Parses a `&str` starting from `rule`
    #[allow(clippy::result_large_err)]
    fn parse(rule: Self::Rule, input_str: &str) -> Result<Nodes<'_, Self::Rule, ()>, Error<Self::Rule>> {
        Self::parse_with_userdata(rule, input_str, ())
    }

    /// Parses a `&str` starting from `rule`, carrying `user_data` through the parser methods.
    #[allow(clippy::result_large_err)]
    fn parse_with_userdata<D>(
        rule: Self::Rule,
        input_str: &str,
        user_data: D,
    ) -> Result<Nodes<'_, Self::Rule, D>, Error<Self::Rule>> {
        let pairs = Self::Parser::parse(rule, input_str)?;
        Ok(Nodes::new(input_str, pairs, user_data))
    }
}
