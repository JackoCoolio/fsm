use std::collections::HashMap;

use crate::{
    nfa::{NFABuilderError, NFA},
    state::State,
    transition::MaybeEpsilonTransition,
};

pub type NFAeBuilderError = NFABuilderError;

#[derive(Default)]
pub struct NFAeBuilder<L, S> {
    pub(crate) start: Option<usize>,
    pub(crate) states: Vec<State<S, MaybeEpsilonTransition<L>>>,
}

impl<L, S> NFAeBuilder<L, S> {
    pub fn set_start(&mut self, start: usize) -> &mut Self {
        self.start = Some(start);
        self
    }

    pub fn add_state(&mut self, state: State<S, MaybeEpsilonTransition<L>>) -> &mut Self {
        self.states.push(state);
        self
    }

    pub fn build(self) -> Result<NFAe<L, S>, NFAeBuilderError> {
        let Some(start) = self.start else {
            return Err(NFABuilderError::MissingStartIndex);
        };

        if self.states.is_empty() {
            return Err(NFABuilderError::MissingStates);
        }

        let finish_count = self.states.iter().filter(|&st| st.is_finish()).count();

        if finish_count == 0 {
            return Err(NFABuilderError::MissingFinish);
        }

        if start >= self.states.len() {
            return Err(NFABuilderError::InvalidStartIndex);
        }

        Ok(NFAe {
            start,
            states: self.states,
        })
    }
}

impl<L, S> From<NFAe<L, S>> for NFAeBuilder<L, S> {
    fn from(nfae: NFAe<L, S>) -> Self {
        Self {
            states: nfae.states,
            start: Some(nfae.start),
        }
    }
}

#[test]
fn test_nfae_builder() {
    use crate::transition::RealTransition;

    let mut builder = NFAeBuilder::default();

    let mut start = State::new(false, ());
    let finish = State::new(true, ());

    builder.set_start(0);

    start.add_transition(RealTransition::new('a', 1));

    builder.add_state(start).add_state(finish);

    builder.build().unwrap();
}

pub struct NFAe<L, S> {
    pub(crate) states: Vec<State<S, MaybeEpsilonTransition<L>>>,
    pub(crate) start: usize,
}

impl<L, S> NFAe<L, S> {
    pub fn get_state(&self, state: usize) -> Option<&State<S, MaybeEpsilonTransition<L>>> {
        self.states.get(state)
    }

    pub fn get_state_mut(
        &mut self,
        state: usize,
    ) -> Option<&mut State<S, MaybeEpsilonTransition<L>>> {
        self.states.get_mut(state)
    }

    pub fn get_states(&self) -> &Vec<State<S, MaybeEpsilonTransition<L>>> {
        &self.states
    }

    pub fn get_start(&self) -> &State<S, MaybeEpsilonTransition<L>> {
        self.states.get(self.start).unwrap()
    }

    /// Returns a list of states that can be reached from state `s` through epsilon transitions.
    pub fn epsilon_closure(&self, s: usize) -> Vec<&State<S, MaybeEpsilonTransition<L>>> {
        let mut states = Vec::new();

        // if state is not found, no epsilon-reachable states
        let Some(state) = self.get_state(s) else {
            return Vec::new();
        };

        // self is epsilon-reachable
        states.push(state);

        // add state to closure if epsilon-reachable
        for transition in &state.transitions {
            if transition.is_epsilon() {
                states.extend(self.epsilon_closure(transition.dest()));
            }
        }

        states
    }

    /// Removes non-start states that have no incoming transitions.
    /// These states are unreachable.
    pub fn remove_orphan_states(&mut self) {
        let mut reachable_states = Vec::new();
        reachable_states.push(self.start);

        for state in self.states.iter() {
            for transition in state.transitions.iter() {
                assert!(!transition.is_epsilon());
                reachable_states.push(transition.dest());
            }
        }

        let mut new_states: Vec<State<S, MaybeEpsilonTransition<L>>> = Vec::new();
        let mut reassign_map: HashMap<usize, usize> = HashMap::new();

        for (i, state) in std::mem::take(&mut self.states).into_iter().enumerate() {
            if reachable_states.contains(&i) {
                reassign_map.insert(i, new_states.len());
                new_states.push(state);
            }
        }

        self.states = new_states;

        for state in self.states.iter_mut() {
            for transition in state.transitions.iter_mut() {
                let new_dest = reassign_map.get(&transition.dest()).unwrap();
                transition.set_dest(*new_dest);
            }
        }
    }
}

impl<L, S> NFAe<L, S>
where
    L: Clone,
{
    /// Calls `epsilon_simplify` on all states.
    pub fn epsilon_simplify_all(&mut self) {
        for i in 0..self.states.len() {
            self.epsilon_simplify(i);
        }
    }

    /// "Steals" transitions from epsilon-reachable states and gives them to the specified state.
    pub fn epsilon_simplify(&mut self, s: usize) {
        let epsilon_closure = self.epsilon_closure(s);

        if epsilon_closure.is_empty() {
            return;
        }

        let mut transitions: Vec<MaybeEpsilonTransition<L>> = Vec::new();

        let mut is_finish = self.get_state(s).unwrap().is_finish();

        // steal transitions from epsilon-reachable states
        for epsilon_state in epsilon_closure {
            let new_transitions = epsilon_state
                .transitions
                .iter()
                .cloned()
                .filter(|tr| !tr.is_epsilon());
            transitions.extend(new_transitions);
            // also mark current state as finish if epsilon-reachable state was finish
            if epsilon_state.is_finish() {
                is_finish = true;
            }
        }

        let state = self.get_state_mut(s).unwrap();

        state.finish = is_finish;

        // remove epsilon transitions now
        state.transitions.retain(|tr| !tr.is_epsilon());

        // add on stolen transitions
        state.transitions.extend(transitions);
    }

    /// Converts this NFA-e into an NFA.
    pub fn into_nfa(mut self) -> NFA<L, S> {
        self.epsilon_simplify_all();

        self.remove_orphan_states();

        NFA {
            states: self
                .states
                .into_iter()
                .map(|st| st.try_into().unwrap())
                .collect(),
            start: self.start,
        }
    }
}

#[test]
fn test_convert_to_nfa() {
    use crate::transition::RealTransition;

    let mut builder = NFAeBuilder::default();
    let mut start = State::new(false, ());
    let mut a = State::new(false, ());
    let mut b = State::new(false, ());
    let mut c = State::new(false, ());
    let d = State::new(true, ());

    start
        .add_transition(RealTransition::new('a', 1))
        .add_transition(RealTransition::new('b', 2));
    a.add_transition(MaybeEpsilonTransition::new_epsilon(3));
    b.add_transition(MaybeEpsilonTransition::new_epsilon(3));
    c.add_transition(MaybeEpsilonTransition::new_epsilon(4));

    builder
        .add_state(start)
        .add_state(a)
        .add_state(b)
        .add_state(c)
        .add_state(d);

    builder.set_start(0);

    let mut nfae = builder.build().unwrap();

    assert!(nfae.epsilon_closure(0).len() == 1);
    assert!(nfae.epsilon_closure(1).len() == 3);
    assert!(nfae.epsilon_closure(2).len() == 3);
    assert!(nfae.epsilon_closure(3).len() == 2);
    assert!(nfae.epsilon_closure(4).len() == 1);

    nfae.epsilon_simplify_all();

    assert!(!nfae.get_state(0).unwrap().is_finish());
    assert!(nfae.get_state(1).unwrap().is_finish());
    assert!(nfae.get_state(2).unwrap().is_finish());
    assert!(nfae.get_state(3).unwrap().is_finish());
    assert!(nfae.get_state(4).unwrap().is_finish());

    assert!(nfae.get_states().len() == 5);

    nfae.remove_orphan_states();

    assert_eq!(nfae.get_states().len(), 3);

    let _ = nfae.into_nfa();
}
