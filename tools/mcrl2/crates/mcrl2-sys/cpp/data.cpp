#include "atermpp.h"
#include "mcrl2/data/substitutions/mutable_map_substitution.h"
#include "mcrl2/data/replace.h"

#include "mcrl2-sys/cpp/data.h"
#include "mcrl2-sys/src/data.rs.h"

namespace mcrl2::data
{

std::unique_ptr<atermpp::aterm> mcrl2_data_expression_replace_variables(const atermpp::detail::_aterm& term,
    const rust::Vec<assignment_pair>& sigma)
{
  atermpp::unprotected_aterm_core tmp_expr(&term);
  MCRL2_ASSERT(is_data_expression(atermpp::down_cast<atermpp::aterm>(tmp_expr)));

  data::mutable_map_substitution<> tmp;
  for (const auto& assign : sigma)
  {
    atermpp::unprotected_aterm_core tmp_lhs(assign.lhs);
    atermpp::unprotected_aterm_core tmp_rhs(assign.rhs);

    tmp[atermpp::down_cast<data::variable>(tmp_lhs)]
        = atermpp::down_cast<data::data_expression>(tmp_rhs);
  }

  return std::make_unique<atermpp::aterm>(
      replace_variables(atermpp::down_cast<data_expression>(tmp_expr), tmp));
}

}