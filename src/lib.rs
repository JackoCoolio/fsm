#![feature(is_some_and)]

use std::{borrow::Borrow, collections::HashMap, fmt::Debug, hash::Hash};

pub struct DFA<L, S> {
    states: Vec<(S, HashMap<L, usize>)>,
}

pub struct State<'a, L, S> {
    dfa: &'a DFA<L, S>,
    index: usize,
}

pub struct MutState<'a, L, S> {
    dfa: &'a mut DFA<L, S>,
    index: usize,
}

impl<'a, L, S> MutState<'a, L, S>
where
    L: Eq + Hash,
{
    /// Returns self.
    pub fn set_transition(self, transition: L, to: usize) -> Self {
        self.dfa
            .get_state_transitions_mut(self.index)
            .unwrap()
            .insert(transition, to);
        self
    }

    /// Returns self.
    pub fn add_self_loop(self, transition: L) -> Self {
        let index = self.index;
        self.set_transition(transition, index)
    }

    /// Returns the new state.
    pub fn add_state(mut self, transition: L, state: S) -> Self {
        let index = self.dfa.add_state(state).index;

        self = self.set_transition(transition, index);
        self.dfa.get_state_mut(index).unwrap()
    }

    pub fn extend(self, transition: L, other: DFA<L, S>) {
        let num = self.dfa.states.len();
        self.dfa
            .states
            .extend(other.states.into_iter().map(|(s, mut hm)| {
                // shift
                hm.values_mut().for_each(|v| *v += num);
                (s, hm)
            }));
        self.set_transition(transition, num);
    }
}

impl<'a, L, S> Transition for Option<State<'a, L, S>>
where
    L: Hash + Eq,
{
    type Language = L;
    type Next = Self;

    fn next(&self, transition: &Self::Language) -> Self::Next {
        match self {
            None => None,
            Some(state) => state.next(transition),
        }
    }
}

impl<'a, L, S> Transition for State<'a, L, S>
where
    L: Hash + Eq,
{
    type Language = L;
    type Next = Option<Self>;

    fn next(&self, transition: &Self::Language) -> Self::Next {
        match self
            .dfa
            .get_state_transitions_unchecked(self.index)
            .get(transition)
        {
            None => None,
            Some(index) => Some(State {
                dfa: self.dfa,
                index: *index,
            }),
        }
    }
}

impl<'a, L, S> State<'a, L, S> {
    pub fn value(&self) -> &S {
        self.dfa.get_state_value(self.index).unwrap()
    }
}

pub trait Transition {
    type Language;
    type Next;
    fn next(&self, transition: &Self::Language) -> Self::Next;
}

// a map from L to usize

impl<L, S> DFA<L, S> {
    pub fn new(start: S) -> DFA<L, S> {
        DFA {
            states: vec![(start, HashMap::new())],
        }
    }

    fn get_state_value(&self, index: usize) -> Option<&S> {
        self.states.get(index).map(|s| &s.0)
    }

    fn get_state_transitions(&self, index: usize) -> Option<&HashMap<L, usize>> {
        self.states.get(index).map(|s| &s.1)
    }

    fn get_state_transitions_unchecked(&self, index: usize) -> &HashMap<L, usize> {
        unsafe { &self.states.get_unchecked(index).1 }
    }

    fn get_state_transitions_mut(&mut self, index: usize) -> Option<&mut HashMap<L, usize>> {
        self.states.get_mut(index).map(|s| &mut s.1)
    }

    pub fn get_start(&self) -> State<L, S> {
        State {
            dfa: self,
            index: 0,
        }
    }

    pub fn get_start_mut(&mut self) -> MutState<L, S> {
        MutState {
            dfa: self,
            index: 0,
        }
    }

    pub fn get_state(&self, index: usize) -> Option<State<L, S>> {
        if index >= self.states.len() {
            return None;
        }

        Some(State { dfa: self, index })
    }

    pub fn get_state_mut(&mut self, index: usize) -> Option<MutState<L, S>> {
        if index >= self.states.len() {
            return None;
        }

        Some(MutState { dfa: self, index })
    }

    pub fn add_state(&mut self, state: S) -> MutState<L, S> {
        let index = self.states.len();
        self.states.push((state, HashMap::new()));
        MutState { dfa: self, index }
    }
}

impl<L, S> DFA<L, S>
where
    L: Eq + Hash,
{
    pub fn traverse<'dfa, I>(&'dfa self, inputs: I) -> Option<State<'dfa, L, S>>
    where
        I: Iterator,
        <I as Iterator>::Item: Borrow<L> + Debug,
    {
        let mut curr = self.get_start();
        for input in inputs {
            curr = curr.next(input.borrow())?;
        }

        Some(curr)
    }
}

#[cfg(test)]
mod tests {
    use crate::DFA;

    #[test]
    fn new() {
        let mut dfa = DFA::new(false);
        dfa.get_start_mut()
            .add_state('/', false)
            .add_state('/', true)
            .add_self_loop(' ')
            .add_state('\n', true);

        assert!(dfa.traverse("//   ".chars()).is_some_and(|x| *x.value()));
    }
}
