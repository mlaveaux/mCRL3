use std::sync::Arc;

use merc_explore::LPS;
use merc_explore::StateEffect;
use merc_explore::Summand;
use merc_utilities::MercError;

use crate::bsgs::Bsgs;
use crate::bsgs::CanonicalizeContext;
use crate::explore_common::ParameterLayoutLPS;

/// Wraps any `LPS<Value = usize>` and canonicalizes every enumerated next-state
/// to the lexicographically smallest orbit representative before passing it to
/// the caller.
///
/// State vectors are laid out as `[eq_idx_0..eq_idx_{offset-1}, param_0..param_{n-1}]`
/// where only positions `param_offset..` are touched by the group action.  Position 0
/// (the equation index) is never permuted.
///
/// # Wrapping order
///
/// When combined with [`merc_explore::CacheLPS`], place the cache *inside* and the
/// quotient *outside*:
/// ```text
/// QuotientLps<CacheLPS<PbesLps>>
/// ```
/// This keeps cache keys narrow (raw, un-canonicalized write positions) and avoids
/// forcing the cache to track all parameters as written.
pub struct QuotientLps<P: ParameterLayoutLPS<Value = usize>> {
    inner: Arc<P>,
    bsgs: Arc<Bsgs>,
    summands: Vec<QuotientSummand<P>>,
    param_offset: usize,
}

/// A single summand of a [`QuotientLps`].
///
/// Delegates enumeration to the corresponding inner summand and canonicalizes
/// each next-state before reporting it.
pub struct QuotientSummand<P: ParameterLayoutLPS<Value = usize>> {
    index: usize,
    inner: Arc<P>,
    bsgs: Arc<Bsgs>,
    param_offset: usize,
    read_positions: Vec<usize>,
}

/// Per-thread enumeration context for a [`QuotientLps`].
pub struct QuotientContext<P: ParameterLayoutLPS<Value = usize>> {
    inner: <P::Summand as Summand>::Context,

    /// Working buffers of [`Bsgs::canonicalize_into`], so that canonicalizing a
    /// next state costs no allocation.
    scratch: CanonicalizeContext,

    /// The canonicalized next state handed to the caller's callback.
    canonical: Vec<usize>,
}

// SAFETY: Neither struct has interior mutability of its own (no UnsafeCell /
// raw pointers). All concurrent access is read-only via `&self`. The only
// non-trivially-Sync field is `Arc<P>`: sharing `&Arc<P>` across threads only
// requires `P: Sync` (which the bound enforces). The stdlib's conservative
// `impl<T: Send + Sync> Sync for Arc<T>` also requires `T: Send` to handle the
// last Arc being dropped on a foreign thread, but `QuotientLps` is not `Send`,
// so that case cannot arise. `Arc<Bsgs>` is unconditionally fine because `Bsgs`
// contains only `usize`, `Vec`, and `HashMap` of plain data, all auto-`Sync`.
unsafe impl<P: ParameterLayoutLPS<Value = usize> + Sync> Sync for QuotientLps<P> {}
unsafe impl<P: ParameterLayoutLPS<Value = usize> + Sync> Sync for QuotientSummand<P> {}

impl<P> QuotientLps<P>
where
    P: ParameterLayoutLPS<Value = usize>,
{
    /// Wraps `inner` in a canonicalizing quotient layer.
    ///
    /// `param_offset` is the first position in the state vector that belongs to
    /// the PBES parameters (always `1` for `PbesSrfLps`, where position 0 is the
    /// equation index).
    pub fn new(inner: P, bsgs: Arc<Bsgs>, param_offset: usize) -> Self {
        let inner = Arc::new(inner);

        let summands = inner
            .summands()
            .iter()
            .enumerate()
            .map(|(i, s)| QuotientSummand {
                index: i,
                inner: Arc::clone(&inner),
                bsgs: Arc::clone(&bsgs),
                param_offset,
                read_positions: s.read_positions().to_vec(),
            })
            .collect();

        QuotientLps {
            inner,
            bsgs,
            summands,
            param_offset,
        }
    }
}

impl<P> LPS for QuotientLps<P>
where
    P: ParameterLayoutLPS<Value = usize>,
{
    type Value = usize;
    type Label = P::Label;
    type StateInfo = P::StateInfo;
    type Summand = QuotientSummand<P>;

    fn initial_state(&self) -> Vec<usize> {
        self.bsgs.canonicalize(&self.inner.initial_state(), self.param_offset)
    }

    fn summands(&self) -> &[Self::Summand] {
        &self.summands
    }

    fn create_context(&self) -> QuotientContext<P> {
        QuotientContext {
            inner: self.inner.create_context(),
            scratch: CanonicalizeContext::default(),
            canonical: Vec::new(),
        }
    }

    fn prepare<'a>(&'a self, context: &mut QuotientContext<P>, state: &'a [usize]) -> impl Iterator<Item = usize> + 'a {
        self.inner.prepare(&mut context.inner, state)
    }

    fn state_info(&self, state: &[usize], context: &QuotientContext<P>) -> P::StateInfo {
        self.inner.state_info(state, &context.inner)
    }
}

impl<P> Summand for QuotientSummand<P>
where
    P: ParameterLayoutLPS<Value = usize>,
{
    type Value = usize;
    type Label = P::Label;
    type Context = QuotientContext<P>;

    fn read_positions(&self) -> &[usize] {
        &self.read_positions
    }

    fn effect(&self) -> StateEffect<'_> {
        // Canonicalization can move a value to any parameter position, and the
        // states it passes through unchanged are of other lengths entirely.
        StateEffect::Opaque
    }

    fn enumerate<F>(&self, context: &mut Self::Context, state: &[usize], mut report: F) -> Result<(), MercError>
    where
        F: FnMut(&Self::Label, &[usize]) -> Result<(), MercError>,
    {
        let bsgs = &self.bsgs;
        let inner = &*self.inner;
        let param_offset = self.param_offset;

        // Destructured so the closure can borrow the canonicalization buffers
        // while the inner summand holds its own context.
        let QuotientContext {
            inner: inner_context,
            scratch,
            canonical,
        } = context;

        self.inner.summands()[self.index].enumerate(inner_context, state, |label, next| {
            match inner.parameter_range(next) {
                Some(range) => {
                    debug_assert_eq!(
                        range.start, param_offset,
                        "the parameter block must start where the group acts"
                    );
                    debug_assert_eq!(range.len(), bsgs.n, "the group must act on the whole parameter block");
                    bsgs.canonicalize_into(next, param_offset, scratch, canonical);
                    report(label, canonical)
                }
                // Sinks and subformula vertices carry no data parameters, so the
                // group does not act on them; permuting their payload would
                // corrupt a priority or an interned formula index.
                None => report(label, next),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use merc_explore::CacheLPS;
    use merc_explore::CachingStrategy;
    use merc_explore::ExplorationStrategy;
    use merc_utilities::MercError;
    use merc_utilities::Timing;
    use merc_vpg::PG;
    use merc_vpg::solve_zielonka;

    use crate::bsgs::Bsgs;
    use crate::explore_common::explore_pbes_impl;
    use crate::explore_pbes::PbesLps;
    use crate::explore_srf::PbesSrfLps;
    use crate::graph_symmetry::GapConfig;
    use crate::permutation::Permutation;

    use super::QuotientLps;

    fn gap_config() -> GapConfig {
        GapConfig {
            executable: "gap".to_string(),
            dump_script: None,
        }
    }

    /// Verifies that wrapping `PbesSrfLps` in `QuotientLps` (with trivial group)
    /// produces the same parity game size as the unwrapped LPS.
    #[test]
    #[cfg_attr(miri, ignore)]
    fn quotient_trivial_group_same_game_size() -> Result<(), MercError> {
        let pbes_text = r#"pbes
nu X(b: Bool) = X(true);
init X(true);"#;
        let pbes = mcrl2::Pbes::from_text(pbes_text)?;

        let lps = PbesSrfLps::new(&pbes)?;
        let n = lps.num_params();
        let bsgs = Arc::new(Bsgs::from_generators(&[], n, &gap_config())?);

        let timing = Timing::new();
        let plain_game = explore_pbes_impl(&lps, ExplorationStrategy::Bfs, &timing)?;

        let lps2 = PbesSrfLps::new(&pbes)?;
        let qlps = QuotientLps::new(lps2, bsgs, 1);
        let quot_game = explore_pbes_impl(&qlps, ExplorationStrategy::Bfs, &timing)?;

        assert_eq!(plain_game.num_of_vertices(), quot_game.num_of_vertices());
        assert_eq!(plain_game.num_of_edges(), quot_game.num_of_edges());
        Ok(())
    }

    /// Verifies that `QuotientLps<CacheLPS<PbesSrfLps>>` compiles and produces a
    /// valid game (wrapping order: cache inside, quotient outside).
    #[test]
    #[cfg_attr(miri, ignore)]
    fn quotient_with_cache_compiles() -> Result<(), MercError> {
        let pbes_text = r#"pbes
nu X(b: Bool) = X(true);
init X(true);"#;
        let pbes = mcrl2::Pbes::from_text(pbes_text)?;
        let lps = PbesSrfLps::new(&pbes)?;
        let n = lps.num_params();
        let bsgs = Arc::new(Bsgs::from_generators(&[], n, &gap_config())?);

        let cached = CacheLPS::new(lps, CachingStrategy::Local);
        let qlps = QuotientLps::new(cached, bsgs, 1);

        let timing = Timing::new();
        let game = explore_pbes_impl(&qlps, ExplorationStrategy::Bfs, &timing)?;
        assert!(game.num_of_vertices() > 0);
        Ok(())
    }

    /// A PBES whose two parameters are interchangeable, explored with the general
    /// (non-SRF) explorer so the game also contains sink and subformula vertices.
    ///
    /// Those carry a priority and an interned formula index where a propositional
    /// variable instantiation carries parameters, so a quotient that permutes them
    /// unconditionally corrupts them — the interned index in particular becomes a
    /// formula that does not exist.
    const SYMMETRIC_PBES: &str = r#"pbes
nu X(m: Nat, n: Nat) = X(m, n) && (Y(n, m) || Y((m + 1) mod 2, n));
mu Y(m: Nat, n: Nat) = X(m, n) || Y((n + 1) mod 2, m);
init X(0, 1);"#;

    #[test]
    #[cfg_attr(miri, ignore)]
    fn quotient_preserves_winner_and_reduces_the_game() -> Result<(), MercError> {
        let pbes = mcrl2::Pbes::from_text(SYMMETRIC_PBES)?;
        let timing = Timing::new();

        let plain = explore_pbes_impl(&PbesLps::new(pbes.clone())?, ExplorationStrategy::Bfs, &timing)?;

        let lps = PbesLps::new(pbes)?;
        let n = lps.num_params();
        let generators = vec![Permutation::from_cycle_notation("(0 1)")?];
        let bsgs = Arc::new(Bsgs::from_generators(&generators, n, &gap_config())?);
        let quotient = explore_pbes_impl(&QuotientLps::new(lps, bsgs, 1), ExplorationStrategy::Bfs, &timing)?;

        assert!(
            quotient.num_of_vertices() < plain.num_of_vertices(),
            "swapping the two parameters is a symmetry, so the quotient must be smaller \
             (plain {} vertices, quotient {})",
            plain.num_of_vertices(),
            quotient.num_of_vertices()
        );

        let (plain_solution, _) = solve_zielonka(&plain, false);
        let (quotient_solution, _) = solve_zielonka(&quotient, false);
        assert_eq!(
            plain_solution[0][0], quotient_solution[0][0],
            "the quotient changed the winner of the initial vertex"
        );
        Ok(())
    }

    /// The SRF backend must reduce and preserve the winner just like the general
    /// one — `--srf --symmetry` layers the quotient over [`PbesSrfLps`].
    ///
    /// The two games are not the same size: SRF normalisation rewrites the
    /// right-hand sides, so this compares the SRF game against its own quotient
    /// rather than against the general explorer's.
    #[test]
    #[cfg_attr(miri, ignore)]
    fn quotient_over_srf_preserves_winner_and_reduces_the_game() -> Result<(), MercError> {
        let pbes = mcrl2::Pbes::from_text(SYMMETRIC_PBES)?;
        let timing = Timing::new();

        let plain = explore_pbes_impl(&PbesSrfLps::new(&pbes)?, ExplorationStrategy::Bfs, &timing)?;

        let lps = PbesSrfLps::new(&pbes)?;
        assert_eq!(
            lps.num_params(),
            2,
            "the generator below is a transposition of the whole parameter vector"
        );
        // Both parameters have the same sort, so the transposition is `(0 1)`
        // whichever of the two orders `unify_parameters` happened to produce.
        let generators = vec![Permutation::from_cycle_notation("(0 1)")?];
        let bsgs = Arc::new(Bsgs::from_generators(&generators, lps.num_params(), &gap_config())?);
        let quotient = explore_pbes_impl(&QuotientLps::new(lps, bsgs, 1), ExplorationStrategy::Bfs, &timing)?;

        assert!(
            quotient.num_of_vertices() < plain.num_of_vertices(),
            "swapping the two parameters is a symmetry, so the quotient must be smaller \
             (plain {} vertices, quotient {})",
            plain.num_of_vertices(),
            quotient.num_of_vertices()
        );

        let (plain_solution, _) = solve_zielonka(&plain, false);
        let (quotient_solution, _) = solve_zielonka(&quotient, false);
        assert_eq!(
            plain_solution[0][0], quotient_solution[0][0],
            "the quotient changed the winner of the initial vertex"
        );
        Ok(())
    }

    /// The same reduction must survive a cache layer underneath the quotient.
    #[test]
    #[cfg_attr(miri, ignore)]
    fn quotient_over_cache_agrees_with_quotient_alone() -> Result<(), MercError> {
        let pbes = mcrl2::Pbes::from_text(SYMMETRIC_PBES)?;
        let generators = vec![Permutation::from_cycle_notation("(0 1)")?];
        let timing = Timing::new();

        let lps = PbesLps::new(pbes.clone())?;
        let n = lps.num_params();
        let bsgs = Arc::new(Bsgs::from_generators(&generators, n, &gap_config())?);
        let uncached = explore_pbes_impl(
            &QuotientLps::new(lps, Arc::clone(&bsgs), 1),
            ExplorationStrategy::Bfs,
            &timing,
        )?;

        let cached = CacheLPS::new(PbesLps::new(pbes)?, CachingStrategy::Local);
        let cached = explore_pbes_impl(&QuotientLps::new(cached, bsgs, 1), ExplorationStrategy::Bfs, &timing)?;

        assert_eq!(uncached.num_of_vertices(), cached.num_of_vertices());
        assert_eq!(uncached.num_of_edges(), cached.num_of_edges());
        Ok(())
    }
}
