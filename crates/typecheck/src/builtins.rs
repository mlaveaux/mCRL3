use std::sync::LazyLock;

use merc_syntax::UntypedDataSpecification;

/// The five built-in basic sorts. They are always present in a specification,
/// resolve to primitives, and may not receive user constructors.
pub(crate) const BASIC_SORT_NAMES: [&str; 5] = ["Bool", "Pos", "Nat", "Int", "Real"];

/// Whether `name` is one of the [`BASIC_SORT_NAMES`].
pub(crate) fn is_basic_sort_name(name: &str) -> bool {
    BASIC_SORT_NAMES.contains(&name)
}

/// The polymorphic built-in operators that exist for *every* sort: the
/// comparison operators and the conditional `if`. Their sort variable `S`
/// remains an unresolved `Reference` node, instantiated with fresh unification
/// variables per occurrence, exactly like the container operations — they feed
/// `POLYMORPHIC_SIGNATURE` alongside the containers, so inference resolves `==`
/// and `|>` through one mechanism.
///
/// These operators are built in and never declared in a `spec/*.mcrl2` file, so
/// this template is written inline rather than bundled. It is the single source
/// of the built-in scheme *names* (see [`builtin_scheme_names`]) and their
/// *sorts* (via `POLYMORPHIC_SIGNATURE`).
pub(crate) static BUILTIN_SCHEME_TEMPLATE: LazyLock<UntypedDataSpecification> = LazyLock::new(|| {
    UntypedDataSpecification::parse(
        "map ==: S # S -> Bool; !=: S # S -> Bool; \
             <: S # S -> Bool; <=: S # S -> Bool; >: S # S -> Bool; >=: S # S -> Bool; \
             if: Bool # S # S -> S;",
    )
    .expect("the built-in scheme template parses")
});

/// The names of the polymorphic built-in schemes, derived from
/// [`BUILTIN_SCHEME_TEMPLATE`] so the list has a single definition. These names
/// are usable without a declaration, so the well-formedness and reserved-name
/// checks admit them.
pub(crate) fn builtin_scheme_names() -> impl Iterator<Item = &'static str> {
    BUILTIN_SCHEME_TEMPLATE
        .map_declarations
        .iter()
        .map(|decl| decl.identifier.as_str())
}

/// The arithmetic operator names that take the deterministic promotion fast path
/// during inference (see the module-level note on the numeric fast path).
///
/// `+`/`-`/`*` are included even though they are *also* the Set/Bag operations,
/// because the fast path is only taken when a name has no container meaning;
/// `gen_name` intersects this set with that condition.
pub(crate) const NUMERIC_FAMILY: [&str; 9] = ["+", "-", "*", "/", "div", "mod", "exp", "max", "min"];

/// Whether `name` is one of the [`NUMERIC_FAMILY`] operators.
pub(crate) fn is_numeric_family(name: &str) -> bool {
    NUMERIC_FAMILY.contains(&name)
}
