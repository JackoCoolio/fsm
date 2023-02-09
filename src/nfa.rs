use crate::{state::State, transition::RealTransition, nfae::NFAe};

#[derive(Default)]
pub struct NFABuilder<L, S> {
    pub states: Vec<State<S, RealTransition<L>>>,
    pub start: Option<usize>,
}

impl<L, S> NFABuilder<L, S> {
    pub fn add_state(&mut self, state: State<S, RealTransition<L>>) -> &mut Self {
        self.states.push(state);
        self
    }

    pub fn set_start(&mut self, start: usize) -> &mut Self {
        self.start = Some(start);
        self
    }

    pub fn build(self) -> Result<NFA<L, S>, String> {
        let Some(start) = self.start else {
            return Err("must specify a start index".into());
        };

        if self.states.is_empty() {
            return Err("NFA must have at least one state".into());
        }

        let finish_count = self.states.iter().filter(|&st| st.is_finish()).count();
        if finish_count == 0 {
            return Err("NFA must have at least one finish".into());
        }

        if start >= self.states.len() {
            return Err("start index must be valid".into());
        }

        Ok(NFA {
            start,
            states: self.states,
        })
    }
}

impl<L, S> From<NFA<L, S>> for NFABuilder<L, S> {
    fn from(NFA { states, start }: NFA<L, S>) -> Self {
        Self {
            states,
            start: Some(start),
        }
    }
}

pub struct NFA<L, S> {
    pub states: Vec<State<S, RealTransition<L>>>,
    pub start: usize,
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
    pub fn traverse_from<'a, I: 'b>(&'a self, from: usize, mut symbols: I) -> Vec<&State<S, RealTransition<L>>>
    where
        I: Iterator<Item = &'b L> + Clone,
    {
        let Some(curr) = self.get_state(from) else {
            return Vec::new();
        };

        let mut ends = Vec::new();

        let Some(symbol) = symbols.next() else {
            return vec![curr];
        };

        let next_indices = curr.next(symbol);
        for index in next_indices {
            ends.extend(self.traverse_from(index, symbols.clone()));
        }

        ends
    }

    pub fn traverse<'a, I: 'b>(&'a self, symbols: I) -> Vec<&State<S, RealTransition<L>>>
    where
        I: Iterator<Item = &'b L> + Clone,
    {
        self.traverse_from(self.start, symbols)
    }
}

#[test]
fn test_nfa_traverse() {
    let mut nfa = NFABuilder::default();
    let mut start = State::new(false, 0);
    let mut x = State::new(false, 1);
    let mut y = State::new(false, 2);
    let z = State::new(true, 3);

    start
        .add_transition(RealTransition::new('a', 1))
        .add_transition(RealTransition::new('a', 2));

    x.add_transition(RealTransition::new('b', 3));
    y.add_transition(RealTransition::new('c', 3));

    nfa
        .add_state(start)
        .add_state(x)
        .add_state(y)
        .add_state(z);

    nfa.set_start(0);

    let nfa = nfa.build().unwrap();

    assert!(nfa.traverse(vec!['a'].iter()).iter().map(|st| *st.data()).collect::<Vec<_>>() == vec![1, 2]);
    assert!(nfa.traverse(vec!['a', 'b'].iter()).first().unwrap().data == 3);
}
