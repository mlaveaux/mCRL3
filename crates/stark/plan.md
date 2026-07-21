# STARK: open work

Everything the `merc_stark` crate still owes, in one place. This replaces the
old `EVALUATOR_PLAN.md`, `IR_LOWERING_PLAN.md` and `MISSING_GRAMMAR_FEATURES.md`
— the *design rationale* those carried now lives in the developer documentation
(`merc-website`, `docs/developer/stark.md`); only open work lives here.

Reference implementation: `~/STARK/` — `speclang/` (parser and lowering),
`lib/src/main/java/stark/` (the runtime: `robtl/`, `distance/`,
`perturbation/`, `penalty/`, `feedback/`, `distl/`, `monitors/`,
`SampleSet.java`, `EvolutionSequence.java`), and `cli/` (the interactive
shell).

**What runs today.** Parsing, resolution, type checking and lowering are
complete for every construct the grammar accepts, and all 27
`examples/stark/*.stark` files lower and validate. `eval::Simulation` runs a
single trajectory; `eval::Analysis` samples an ensemble, perturbs a copy of it,
and evaluates `distance` and `formula` declarations under both the three-valued
(`check`, with a bootstrap confidence interval) and boolean (`check_boolean`)
semantics.

---

## 1. Correctness gaps against the Java reference

These are divergences in constructs this crate *does* implement — bugs, not
missing features. Highest priority.

### 1.1 `range [from, to]` is lowered but never enforced

`VariableInfo::range` survives into the IR, `IrProgram::validate` checks it and
`Display` prints it, but nothing in `eval/` ever reads it. In the reference,
`DataState.set` clamps *every* write through `DataRange.apply` (`Math.max(min,
Math.min(max, v))`), so the bound is a runtime invariant on the whole state
vector, not a declaration-site annotation.

Three write paths need the clamp: `Store::new`'s variable initialisation, the
buffered `PendingUpdate` flush in `eval::step`, and the perturbation
assignments in `eval::perturbation`. Note that `from`/`to` are `ExprRef`s, so
they need evaluating once at startup and caching alongside the store rather
than being re-evaluated per write.

### 1.2 `k # step target` idles one tick too many

`StarkControllerStateGenerator.visitControllerStepAtion` builds
`Controller.doTick(k-1, controller)` — `k-1` tick-only wrappers — so `target`
runs `k` ticks after the `step` command, and `k < 1` behaves exactly like
`k == 1`. `eval::step` instead produces `Cursor::Idle { remaining: k }`, which
consumes `k` idle ticks *before* a further tick runs `target`, i.e. `k+1`.
The fix is `k <= 1 => Cursor::Run(target)`, `k > 1 => Cursor::Idle { remaining:
k - 1 }`. Add a test pinning `1 # step s` as equivalent to a bare `step s`.

### 1.3 Value-for-value cross-checks against the Java tool

Unit tests, `tests/simulation.rs` and `tests/verification.rs` all pass, but
nothing has been compared against the Java tool's actual output. For a
*deterministic* spec (no sampling), compare a trajectory — and a
distance/formula verdict — value for value. Stochastic specs can only be
compared distributionally: the RNG stream is deliberately not bit-compatible
with Java's Mersenne Twister, only the distributions match.

### 1.4 Confidence-interval quirks carried over verbatim

Two reference behaviours in `eval/distance.rs` were ported as written and are
easy to have mistranslated. They should be confirmed once 1.3 gives a way to
compare:

- `\U`'s `evalCI` re-seeds its running-left maximum from the left expression
  *at `i`* on every outer iteration, unlike its own `compute`.
- `bootstrapDistance` clamps the interval to `[0, 1]`, assuming penalty values
  are normalised to that range.

---

## 2. Language features absent from the STARK textual language

These match the original ANTLR grammar (`StarkSpecificationLanguage.g4`)
exactly — they are limitations of the STARK *language*, not of this port. Each
entry gives the workaround the ported examples use. Implementing any of them
means extending the grammar past the reference, which is a deliberate decision
to make rather than a gap to close.

- **No `//` line comments.** Only `/* ... */` blocks (`COMMENT: '/*' .*? '*/'`).
  Workaround: block comments everywhere, including short inline notes.

- **No parenthesised grouping in RobTL formulas.** `RobtlFormula` has no
  `'(' robtlFormula ')'` alternative, so `\F[0,H] (!(A && B) || (C && D))`
  cannot be written inline. Workaround: name each sub-formula as its own
  `formula` declaration and compose by reference, as `engine.stark` does with
  `phi_5`/`phi_6`/`phi_7`.

- **No implication operator.** RobTL has `!`, `&&`, `||` but no `->`, even
  though the Java runtime has `ImplicationRobustnessFormula` (see §3.1).
  Workaround: `A -> B` ≡ `!A || B`, combined with the no-parens point above.

- **No `when`-guarded perturbation assignments.** A controller or environment
  assignment can be guarded (`when guard target' = value;`); a
  `PerturbationAssignment` (`target <- value` inside `[...]@time`) cannot.
  Workaround: fold the condition into a ternary that leaves the variable
  unchanged — `target <- (guard ? new_value : target)`.

- **No `let` inside a perturbation's `[...]@time` block.** A controller or
  environment step can bind a shared intermediate once and reuse it across
  several assignments; a perturbation's atomic block is a flat list of
  `target <- expression` pairs with no binding form. This is a real fidelity
  loss, not a style difference: `vehicle`'s `fasterPerturbation` draws *one*
  random offset and derives a fake speed, a fake required distance and a fake
  safety gap from it, whereas each ported assignment must redraw `R[0,1]`
  independently, so the three "sensor" readings are no longer correlated.

- **No primed-variable references inside expressions.** `NEXT_ID` (`x'`)
  appears only in assignment *target* position; an expression can never read
  "the value `x` is about to become". Workaround: a `let` binding stands in —
  `let new_x = ... in { x' = new_x; d' = f(new_x); }`.

- **No array/list types or aggregate functions.** The `.count()`/`.min()`/
  `.max()`/`.mean()` postfix aggregates, the array literal and the `array`
  type are all present in the original `.g4` only as commented-out rules, so
  there is no array `StarkType` either. The `it` iterator primitive *does*
  parse here (`ExpressionKind::Iterator`), but lowering emits
  `ExprNode::Unreachable` for it, because the aggregate context that would
  bind it does not exist. Adding aggregates is what would make `it` reachable.

- **No math constants** (`pi`, `e`, …). Formulas needing `pi` hard-code the
  decimal expansion (`1.5707963267948966` for `pi/2`).

- **No current-step / round-index expression.** `Expression` has no "current
  round index" primitive — not `state.getStep()`, not an implicit loop
  variable — so an "every `k`-th step" effect cannot be expressed at all.
  Unlike the two perturbation gaps above there is no ternary workaround, since
  the condition depends on absolute position in the evolution sequence rather
  than on any variable in the data state. The Java `turtle` example's
  `ChangeDir` gates a speed boost on `state.getStep() % k == 0`; only its
  unconditional heading jitter was portable (see `turtle_hospital.stark`'s
  header). Note that the reference *does* carry a step counter and time fields
  on `DataState` (§3.7) — exposing them would be the enabling change.

---

## 3. Java runtime features with no textual syntax

The Java library is substantially larger than the language that drives it.
Everything below exists in `~/STARK/lib/` but is unreachable from
`StarkSpecificationLanguage.g4`, so it is only usable by writing Java against
the library directly. Each would need both grammar and IR work here. Ordered
roughly by how close it is to what the crate already does.

### 3.1 Operators missing from arenas that otherwise match

Small, self-contained additions to existing IR enums:

- **`ImplicationRobustnessFormula`** — `FormulaIr` has `Not`/`And`/`Or` but no
  `Implies`. Both `BooleanSemanticsVisitor` and `ThreeValuedSemanticsVisitor`
  implement it. Needs a `->` in `RobtlFormula`.
- **`PersistentPerturbation`** and **`AfterPerturbation`** — `PerturbationIr`
  covers `Nil`/`Atomic`/`Sequence`/`Iteration`, matching exactly what
  `StarkPerturbationGenerator` can build. `PersistentPerturbation(body)`
  repeats `body` forever (`step()` returns `Sequential(body.step(), this)`);
  `AfterPerturbation(steps, body)` delays a whole sub-perturbation rather than
  a single atomic block, which `[...]@time` cannot express when the delayed
  thing is a composite.
- **`AtomicDistanceExpression` with a custom ground metric.** The grammar
  exposes only `<p` and `>p`, which lower to `AtomicLeft`/`AtomicRight`
  (`distanceLeq`/`distanceGeq`). Java's plain `AtomicDistanceExpression` takes
  an arbitrary `DoubleBinaryOperator` as the ground distance between two
  penalty samples, with the Wasserstein lifting built on top of it. Would need
  syntax for naming a ground metric.
- **Convex-combination weight validation.** `ConvexCombinationDistanceExpression`
  rejects weights that do not sum to exactly 1. `DistanceIr::LinearCombination`
  accepts any weights, and since they are `ExprRef`s the check would have to be
  a runtime one at construction. Decide whether to enforce it or to document
  the divergence.

### 3.2 Skorokhod distance

`SkorokhodDistanceExpression` computes a retiming-tolerant distance via a
dynamic-programming table over time offsets, parameterised by a retiming
window, a resolution, a direction flag and an average-vs-maximum mode. There is
no `DistanceIr` node and no grammar production for it. Note that its own
`evalCI` throws `UnsupportedOperationException` upstream, so only `compute`
would be portable — meaning it could not appear under a `\D[...]` in a
three-valued `formula`. The `repressilator` example (`Main_Skorokhod.java`)
is the reference use.

### 3.3 Compositional penalties

`PenaltyIr` is a single expression evaluated at every step. Java's
`stark.penalty` package makes a penalty a *coroutine* with the same shape as
`Perturbation`: `AtomicPenalty(afterSteps, expr)`, `SequentialPenalty`,
`IterativePenalty(replica, body)`, `NonePenalty`, with `effect()`/`next()`/
`isDone()` and `effectUpTo(step)`. This lets a penalty change over time — score
one thing for the first `k` steps and another afterwards. `SampleSet` already
has `distanceLeq(Penalty, other, step)` overloads that take one. The grammar
would need a penalty-expression sub-language mirroring `PerturbationExpression`.

### 3.4 Feedback

`stark.feedback` is a whole framework with no syntax at all: `Feedback` has the
same six-case shape as `Perturbation` (`Atomic`/`Delayed`/`Iterative`/`None`/
`Sequential`/`Persistent`) but an `AtomicFeedback` closes the loop — its
`FeedbackFunction` receives the *evolution sequence so far* alongside the
random generator and data state, so the system can react to statistics of its
own ensemble (`SampleSet.mean` over a previous step). `FeedbackSystem` is the
corresponding `SystemState`. This is architecturally the largest gap: nothing
in `eval/` gives a running system access to the sequence it belongs to.

### 3.5 Online monitoring: DisTL, UDisTL and monitors

A second, independent verification formalism. Where RobTL compares a nominal
evolution sequence against a perturbed one via a distance metric (offline, two
trajectories), DisTL evaluates a temporal formula directly against one observed
trajectory (online, incremental).

- `stark.distl` — `True`/`False`/`Negation`/`Conjunction`/`Disjunction`/
  `Implication`/`Always`/`Eventually`/`Until`, plus the two atomic forms
  `TargetDisTLFormula` and `BrinkDisTLFormula`, each carrying a target
  distribution, a penalty (or a compositional `Penalty`, §3.3) and a
  threshold. `DoubleSemanticsVisitor` gives the quantitative semantics.
- `stark.udistl` — `UnboundedUntiluDisTLFormula`; note its semantic evaluation
  throws upstream ("not formally defined") and is only meaningful via a monitor.
- `stark.monitors` — the incremental evaluators (`TargetMonitor`,
  `BrinkMonitor`, `UntilMonitor`, `UnboundedUntilMonitor`, the boolean
  combinators, `DefaultMonitorBuilder`) plus `MonitorBuildingVisitor`.
- `PerceivedSystemState` — a `SystemState` stripped down to its data state,
  which is what monitors consume; `EvolutionSequence.getAsPerceivedSystemStates`
  produces them. It deliberately throws on `sampleNext`.

The `monitoring` example and the monitoring-only parts of `tollbooth` are
ported here only as far as their variable/controller/environment model goes;
the monitored property itself is a comment, not a translation.

### 3.6 Probabilistic and non-deterministic controllers

`eval::step`'s `Cursor` covers `Assign`/`IfThenElse`/`Let`/`Sequence`/`Step`/
`Exec`, matching `AssignmentController`/`IfThenElseController`/`StepController`/
`ExecController`/`NilController` and the flattening of `ParallelController`
into a `Vec<Cursor>`. Three Java controllers have no counterpart:

- `GenerativeChoiceController(p, left, right)` — pick one branch with
  probability `p` and delegate to it for this step.
- `ProbabilisticInterleavingController(p, left, right)` — advance *one* of two
  concurrently-live controllers, chosen with probability `p`; the other keeps
  its cursor. This is a genuinely different composition from the parallel one
  `init a || b` gives, where both advance every tick.
- `RandomChoiceBehaviour` — uniform choice between two behaviours.

The commented-out `controllerProbabilisticBehaviour` /
`controllerProbabilisticItem` rules in the original `.g4`
(`('when' guard)? '[' probability '>' block`) are the intended syntax, and
`controllerSwitchStatement` / `controllerCaseStatment` are commented out
alongside them — `visitControllerCaseStatment` is a bare `//TODO: FIXME!`
upstream, so `switch` is unimplemented in Java too. `lower`'s `Result` and
`DiagnosticKind::NotYetSupported` exist precisely for this class of construct.

### 3.7 Timed systems and the `DataState` clock

`SystemState` here is a store plus cursors. Java's `DataState` additionally
carries `step` (the round index, §2), `timeStep`, `timeReal`, `timeDelta`,
`granularity`, `ctrl_granularity` and `ctrl_timeStep`, and two alternative
system implementations use them:

- `TimedSystem` — a macro-step runs *many* micro-steps, each advancing real
  time by a sampled `generateNextTime`, until the accumulated time crosses the
  next granularity boundary. Sampling is decoupled from the tick.
- `DecoupledTimedSystem` — the same idea with the controller running on its own
  granularity, independent of the environment's.

Both are constructed from Java, never from a specification.

### 3.8 Sequence and sample-set operations

`eval::EvolutionSequence` implements `generate`, `generate_up_to`,
`generate_next`, `apply` (perturbation) and the Wasserstein lifting. Not ported:

- `generateUpToCond(conditions)` / `generateNextStepCond(condition)` — generate
  until a `DataStateBooleanExpression` holds rather than to a fixed step count.
- `select(from, to)` — a sub-sequence view.
- `SampleSet::mean`, `replica(k)`, `applyDistribution` — used by feedback (§3.4).
- `SimulationMonitor` / `ConsoleMonitor` / `SilentMonitor` — progress reporting
  during long ensemble generation, which the CLI would want for `--samples` in
  the thousands.

### 3.9 Path planning

`stark.planning` (`RRTstar`, `RRTstar_vis`, `DefaultMap`, `Pos`, `Goal`,
`Obstacle`) is a support library for the `rover` and `turtle` examples rather
than part of the language. Listed for completeness; porting it is only worth it
if those examples are to run end to end.

---

## 4. Tooling: the CLI

`tools/stark` has `check`, `simulate` and `verify`. The Java `cli/` is an
interactive shell (`StarkScript.g4`) built around a loaded specification and a
mutable analysis configuration. The gaps worth closing, roughly in order of
value:

- **`eval <penalty> at <steps>`** — evaluate a `penalty` declaration over the
  reference sequence and report the per-sample values. No equivalent exists;
  `penalty` declarations are only reachable indirectly, through a `distance`.
- **`compute <distance> after <perturbation> at <n> <steps>`** — report the raw
  distance rather than a formula verdict. `Analysis::distance_under` already
  does the work; only the subcommand is missing.
- **Step ranges.** `stepExpression` is either `at s1, s2, …` or
  `from a to b every k`; `verify --step` takes a single value, so a verdict
  cannot be swept over time in one run.
- **`save in "f.csv"` / `print` / `clear`** — the last result set is retained
  and exportable as CSV. Nothing here persists results.
- **Listing commands** — `formulas`, `penalties`, `distances`,
  `perturbations`, `info`. `check --print-symbols` covers part of this but is
  not per-kind.
- **`set size|m|z|scale|seed`** — the analysis parameters, which here are
  per-invocation flags on `verify`. A REPL would need them as state.
- **Shell plumbing** — `load`, `cd`, `ls`, `cwd`, `quit`. Only relevant if an
  interactive mode is wanted at all; a non-interactive CLI is arguably the
  better fit for this workspace and these belong in the "won't do" column.

---

## 5. Improvements specific to this port

Not gaps against Java — things this implementation should tidy up.

- **Rename `Expression::Normal`'s `std_dev` field to `variance`.** The original
  grammar names the second argument of `N[mean, ...]` `variance`. The parser
  does not care, but the current name asserts a meaning the reference does not,
  which will silently mislead anyone porting a Java model that specifies one or
  the other explicitly. Check what `eval/expr.rs` actually does with it while
  renaming.
- **Spans on perturbation, distance and formula arena nodes.** Expressions and
  slots carry `Span`s; these three arenas do not, so a runtime error inside a
  `distance` can only be anchored to the sub-expression, not to the distance
  operator that failed. Add when the first diagnostic wants one.
- **A `const`/`param` initializer cannot call a function.**
  `UntypedStarkSpecification` buckets declarations by kind, so `resolve.rs`
  works in a fixed kind order — constants and parameters, then types, then
  functions, then variables — rather than in source order. A function
  therefore is not yet declared when a `param` initializer references it, even
  when it appears first in the source. Java's `StarkModelGenerator` walks the
  parse tree in source order and has no such restriction.
  `abz2025_two_lanes_two_cars.stark` works around it by inlining `rss_gap`'s
  formula into `INIT_SAFETY_GAP` for both orderings. Fixing this means either
  preserving a linear source-order declaration list alongside the buckets, or
  hoisting function declarations ahead of constants and parameters (variables
  are already pre-declared for the same reason).
- **Reserved keywords cannot be used as identifiers**, including ones that read
  as ordinary variable names — `distance` is the one that came up (a tractor's
  distance-to-target had to become `dist_to_target`). The set is
  `stark_grammar.pest`'s `KEYWORD` rule; it exists so an `ID` cannot swallow a
  following declaration keyword, but could likely be narrowed with lookahead.
- **Functions return exactly one value.** Java models sometimes compute two
  related outputs from one control law and return a small array; ported as two
  functions that each recompute the shared intermediates (see
  `agriculturalDT.stark`'s `eval_speed_zero`/`eval_steer_zero`). Tuple returns
  would fix this, at the cost of diverging from the grammar.
- **Optimisation passes over the IR** — constant folding, common-subexpression
  elimination, dead-slot elimination. The arena representation was chosen to
  make these straightforward; nothing needs them for correctness, and they
  should wait until a profile says an analysis run is expression-bound.
- **Parallelism.** `SampleSet` uses parallel streams for `evalPenaltyFunction`
  and the bootstrap resampling, and ensemble generation is embarrassingly
  parallel across samples. Everything here is single-threaded. This is the most
  likely source of a large speedup on `verify`, and it interacts with
  reproducibility: per-sample RNG streams have to be derived deterministically
  from the seed rather than drawn from one shared generator.
