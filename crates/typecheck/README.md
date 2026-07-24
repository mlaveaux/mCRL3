
# Is the query caching actually useful?

Various passes already go over the full AST to perform various syntactic
operations.

# Can we merge the checks on the user and system specs more?

Yes, the system spec declares illegal names, but the user spec can also declare
illegal names. The checks are similar, but not identical.

# Why don't we type check the system spec?

