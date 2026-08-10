use std::sync::Arc;

use merc_explore::LPS;
use merc_explore::Summand;
use merc_utilities::MercError;

use crate::bsgs::Bsgs;

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
/// QuotientLps<CacheLPS<PbesSrfLps>>
/// ```
/// This keeps cache keys narrow (raw, un-canonicalized write positions) and avoids
/// forcing the cache to track all parameters as written.
pub(crate) struct QuotientLps<P: LPS<Value = usize>> {
    inner: Arc<P>,
    bsgs: Arc<Bsgs>,
    summands: Vec<QuotientSummand<P>>,
    param_offset: usize,
}

/// A single summand of a [`QuotientLps`].
///
/// Delegates enumeration to the corresponding inner summand and canonicalizes
/// each next-state before reporting it.
pub(crate) struct QuotientSummand<P: LPS<Value = usize>> {
    index: usize,
    inner: Arc<P>,
    bsgs: Arc<Bsgs>,
    param_offset: usize,
    read_positions: Vec<usize>,
    /// Widened to cover all positions in the state vector: canonicalization can
    /// permute any parameter, so a cache layer above must treat every position as
    /// a potential write.
    write_positions: Vec<usize>,
}

/// Per-thread enumeration context for a [`QuotientLps`].
pub(crate) struct QuotientContext<P: LPS<Value = usize>> {
    inner: <P::Summand as Summand>::Context,
}

// SAFETY: Neither struct has interior mutability of its own (no UnsafeCell /
// raw pointers). All concurrent access is read-only via `&self`. The only
// non-trivially-Sync field is `Arc<P>`: sharing `&Arc<P>` across threads only
// requires `P: Sync` (which the bound enforces). The stdlib's conservative
// `impl<T: Send + Sync> Sync for Arc<T>` also requires `T: Send` to handle the
// last Arc being dropped on a foreign thread, but `QuotientLps` is not `Send`,
// so that case cannot arise. `Arc<Bsgs>` is unconditionally fine because `Bsgs`
// contains only `usize`, `Vec`, and `HashMap` of plain data, all auto-`Sync`.
unsafe impl<P: LPS<Value = usize> + Sync> Sync for QuotientLps<P> {}
unsafe impl<P: LPS<Value = usize> + Sync> Sync for QuotientSummand<P> {}

impl<P> QuotientLps<P>
where
    P: LPS<Value = usize>,
{
    /// Wraps `inner` in a canonicalizing quotient layer.
    ///
    /// `param_offset` is the first position in the state vector that belongs to
    /// the PBES parameters (always `1` for `PbesSrfLps`, where position 0 is the
    /// equation index).
    pub(crate) fn new(inner: P, bsgs: Arc<Bsgs>, param_offset: usize) -> Self {
        let inner = Arc::new(inner);
        let state_len = param_offset + bsgs.n;
        let all_write_positions: Vec<usize> = (0..state_len).collect();

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
                write_positions: all_write_positions.clone(),
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
    P: LPS<Value = usize>,
{
    type Value = usize;
    type Label = P::Label;
    type StateInfo = P::StateInfo;
    type Summand = QuotientSummand<P>;

    fn initial_state(&self) -> Vec<usize> {
        self.bsgs
            .canonicalize(&self.inner.initial_state(), self.param_offset)
    }

    fn summands(&self) -> &[Self::Summand] {
        &self.summands
    }

    fn create_context(&self) -> QuotientContext<P> {
        QuotientContext {
            inner: self.inner.create_context(),
        }
    }

    fn prepare<'a>(
        &'a self,
        context: &mut QuotientContext<P>,
        state: &'a [usize],
    ) -> impl Iterator<Item = usize> + 'a {
        self.inner.prepare(&mut context.inner, state)
    }

    fn state_info(&self, state: &[usize], context: &QuotientContext<P>) -> P::StateInfo {
        self.inner.state_info(state, &context.inner)
    }
}

impl<P> Summand for QuotientSummand<P>
where
    P: LPS<Value = usize>,
{
    type Value = usize;
    type Label = P::Label;
    type Context = QuotientContext<P>;

    fn read_positions(&self) -> &[usize] {
        &self.read_positions
    }

    fn write_positions(&self) -> &[usize] {
        &self.write_positions
    }

    fn enumerate<F>(&self, context: &mut Self::Context, state: &[usize], mut report: F) -> Result<(), MercError>
    where
        F: FnMut(&Self::Label, &[usize]) -> Result<(), MercError>,
    {
        let bsgs = &self.bsgs;
        let param_offset = self.param_offset;
        self.inner.summands()[self.index].enumerate(&mut context.inner, state, |label, next| {
            let canon = bsgs.canonicalize(next, param_offset);
            report(label, &canon)
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
    use merc_vpg::Player;
    use merc_vpg::Priority;

    use crate::bsgs::Bsgs;
    use crate::explore_common::run_explore_parity_game;
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
        let plain_game = run_explore_parity_game(&lps, ExplorationStrategy::Bfs, &timing)?;

        let lps2 = PbesSrfLps::new(&pbes)?;
        let qlps = QuotientLps::new(lps2, bsgs, 1);
        let quot_game = run_explore_parity_game(&qlps, ExplorationStrategy::Bfs, &timing)?;

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
        let game = run_explore_parity_game(&qlps, ExplorationStrategy::Bfs, &timing)?;
        assert!(game.num_of_vertices() > 0);
        Ok(())
    }
}
