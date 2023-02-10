use crate::transition::{MaybeEpsilonTransition, RealTransition};

pub struct State<S, T> {
    /// Arbitrary user data held by this state.
    pub data: S,
    /// The transitions from this state.
    pub(crate) transitions: Vec<T>,
    /// Whether or not this state is a finish state.
    pub finish: bool,
}

impl<L, S> TryFrom<State<S, MaybeEpsilonTransition<L>>> for State<S, RealTransition<L>>
{
    type Error = <RealTransition<L> as TryFrom<MaybeEpsilonTransition<L>>>::Error;

    fn try_from(value: State<S, MaybeEpsilonTransition<L>>) -> Result<Self, Self::Error> {
        let transitions = {
            let mut transitions: Vec<RealTransition<L>> = Vec::new();
            for transition in value.transitions {
                transitions.push(transition.try_into()?);
            }
            transitions
        };

        Ok(State {
            data: value.data,
            transitions,
            finish: value.finish,
        })
    }
}

impl<L, S> From<State<S, RealTransition<L>>> for State<S, MaybeEpsilonTransition<L>>
{
    fn from(value: State<S, RealTransition<L>>) -> Self {
        State {
            data: value.data,
            transitions: value.transitions.into_iter().map(|tr| tr.into()).collect(),
            finish: value.finish,
        }
    }
}

impl<L, S> State<S, RealTransition<L>>
where
    L: PartialEq,
{
    /// Finds the state index that the given symbol transitions from this state to.
    pub fn next(&self, symbol: &L) -> Vec<usize> {
        self.transitions
            .iter()
            .filter(|tr| tr.symbol() == symbol)
            .map(|tr| tr.dest())
            .collect()
    }
}

impl<S, T> State<S, T> {
    /// Creates a new State with the given internal data.
    pub fn new(finish: bool, data: S) -> Self {
        State {
            data,
            transitions: Vec::new(),
            finish,
        }
    }

    pub fn is_finish(&self) -> bool {
        self.finish
    }

    pub fn add_transition(&mut self, transition: impl Into<T>) -> &mut Self {
        self.transitions.push(transition.into());
        self
    }

    pub fn add_transitions<I, U>(&mut self, transitions: I) -> &mut Self
    where
        U: Into<T>,
        I: Iterator<Item = U>,
    {
        self.transitions.extend(transitions.map(|tr| tr.into()));
        self
    }
}
