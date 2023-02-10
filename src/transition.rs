pub struct RealTransition<L> {
    pub symbol: L,
    pub dest: usize,
}

impl<L> RealTransition<L> {
    pub fn new(symbol: L, dest: usize) -> Self {
        RealTransition { symbol, dest }
    }

    /// Gets the symbol associated with this transition.
    #[inline]
    pub fn symbol(&self) -> &L {
        &self.symbol
    }

    /// Gets the destination of this transition.
    #[inline]
    pub fn dest(&self) -> usize {
        self.dest
    }
}

impl<L> TryFrom<MaybeEpsilonTransition<L>> for RealTransition<L>
where
    L: Copy + Clone,
{
    type Error = String;

    fn try_from(value: MaybeEpsilonTransition<L>) -> Result<Self, Self::Error> {
        let symbol = match value.kind {
            MaybeEpsilonTransitionKind::Epsilon => {
                return Err("transition must have a symbol".into())
            }
            MaybeEpsilonTransitionKind::Symbol(sym) => sym,
        };

        Ok(RealTransition {
            symbol,
            dest: value.dest,
        })
    }
}

#[derive(Copy, Clone)]
pub struct MaybeEpsilonTransition<L>
where
    L: Copy + Clone,
{
    pub kind: MaybeEpsilonTransitionKind<L>,
    pub dest: usize,
}

impl<L> MaybeEpsilonTransition<L>
where
    L: Copy + Clone,
{
    pub fn new_symbol(symbol: L, dest: usize) -> Self {
        Self {
            dest,
            kind: MaybeEpsilonTransitionKind::Symbol(symbol),
        }
    }

    pub fn new_epsilon(dest: usize) -> Self {
        Self {
            dest,
            kind: MaybeEpsilonTransitionKind::Epsilon,
        }
    }

    /// Gets the symbol associated with this transition.
    #[inline]
    pub fn symbol(&self) -> Option<&L> {
        self.kind.symbol()
    }

    /// Gets the destination of this transition.
    #[inline]
    pub fn dest(&self) -> usize {
        self.dest
    }

    /// Sets the destination of this transition.
    #[inline]
    pub fn set_dest(&mut self, dest: usize) {
        self.dest = dest;
    }

    /// Returns true if this transition is an epsilon transition.
    pub fn is_epsilon(&self) -> bool {
        match self.kind {
            MaybeEpsilonTransitionKind::Epsilon => true,
            _ => false,
        }
    }
}

impl<L> From<RealTransition<L>> for MaybeEpsilonTransition<L>
where
    L: Copy + Clone,
{
    fn from(value: RealTransition<L>) -> Self {
        MaybeEpsilonTransition {
            kind: MaybeEpsilonTransitionKind::Symbol(value.symbol),
            dest: value.dest,
        }
    }
}

#[derive(Copy, Clone)]
pub enum MaybeEpsilonTransitionKind<L>
where
    L: Copy + Clone,
{
    Epsilon,
    Symbol(L),
}

impl<L> MaybeEpsilonTransitionKind<L>
where
    L: Copy + Clone,
{
    pub fn symbol(&self) -> Option<&L> {
        match self {
            Self::Epsilon => None,
            Self::Symbol(x) => Some(x),
        }
    }
}
