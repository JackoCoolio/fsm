use crate::{state::State, transition::RealTransition, nfae::NFAe};

pub struct NFA<L, S> {
    pub states: Vec<State<S, RealTransition<L>>>,
    pub start: usize,
    pub finishes: Vec<usize>,
}

impl<L, S> From<NFAe<L, S>> for NFA<L, S>
where
    L: Copy + Clone,
{
    fn from(value: NFAe<L, S>) -> Self {
        value.into_nfa()
    }
}

impl<L, S> NFA<L, S> {
    pub fn get_state(&self, s: usize) -> Option<&State<S, RealTransition<L>>> {
        self.states.get(s)
    }

    pub fn get_start(&self) -> &State<S, RealTransition<L>> {
        self.get_state(self.start).unwrap()
    }
}

impl<'b, L: 'b, S> NFA<L, S>
where
    L: PartialEq,
{
    pub fn traverse<'a, I: 'b>(&'a self, symbols: I) -> Option<&State<S, RealTransition<L>>>
    where
        I: Iterator<Item = &'b L>,
    {
        let mut curr = self.get_start();

        for symbol in symbols {
            let next_index = curr.next(symbol)?;
            curr = self.get_state(next_index)?;
        }

        Some(curr)
    }
}
