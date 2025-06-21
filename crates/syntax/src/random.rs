use arbitrary::Unstructured;

use crate::SortExpression;




fn arbitrary_sort_expression(u: &mut Unstructured<'_>) -> Result<SortExpression, arbitrary::Error> {
    let sort: SortExpression = u.arbitrary()?;

    Ok(sort)    
}