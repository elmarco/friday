use std::cmp::Ordering;
use std::iter;

extern crate rand;
use rand::Rng;

use crate::cards::{Capacity, Card, CardKind, Hazard, HazardCard, Step, CARDS};

fn shuffle<T>(vec: &mut Vec<T>) {
    let mut slice = vec.as_mut_slice();
    rand::thread_rng().shuffle(&mut slice);
}

#[derive(Debug, Clone)]
pub enum Using<'a> {
    None,
    Draw(i8),
    Destroy(usize),
    Double,
    Copy,
    Sort(Vec<Card<'a>>),
    Swap(usize, i8),
    UnderDeck(usize),
}

#[derive(Debug, Clone)]
pub enum State<'a> {
    None,
    ChooseHazard(Vec<Card<'a>>),
    ChoosePirate,
    Fighting(Card<'a>, u8, Vec<Card<'a>>, Vec<Card<'a>>, Using<'a>),
    Ended(bool),
}

#[derive(Debug)]
pub enum Event<'a> {
    Start,
    HazardChoice(Option<usize>),
    Choice(usize),
    ChoiceUnder(usize, bool),
    Fight,
    Use(usize),
    Win,
    Lose(&'a mut [usize]),
    Continue,
    Break,
    Sort(Vec<usize>, bool),
}

#[derive(Debug)]
pub struct Friday<'a> {
    pub level: Level,
    pub life_points: isize,
    pub step: Step,
    pub step_modif: usize,
    pub destroyed: Vec<Card<'a>>,
    pub aging_deck: Vec<Card<'a>>,
    pub fighting_deck: Vec<Card<'a>>,
    pub fighting_discard: Vec<Card<'a>>,
    pub hazard_deck: Vec<Card<'a>>,
    pub hazard_discard: Vec<Card<'a>>,
    pub pirate_cards: Vec<Card<'a>>,
    pub pirate_cards_won: Vec<Card<'a>>,
    pub state: State<'a>,
}

type Level = usize;

impl<'a> Friday<'a> {
    pub fn new(level: Level) -> Self {
        let mut friday = Self {
            level,
            life_points: if level == 4 { 18 } else { 20 },
            step: Step::Green,
            step_modif: 0,
            destroyed: vec![],
            aging_deck: Self::make_aging_deck(level),
            fighting_deck: Self::make_fighting_deck(),
            fighting_discard: vec![],
            hazard_deck: Self::make_hazard_deck(),
            hazard_discard: vec![],
            pirate_cards: Self::take_two_pirate_cards(),
            pirate_cards_won: vec![],
            state: State::None,
        };
        if level >= 2 {
            friday.aging();
        }
        friday.next(Event::Start).unwrap();
        friday
    }

    fn end_fight(&mut self, cards: &[Card<'a>]) -> Result<(), String> {
        self.step_modif = 0;
        for c in cards {
            if c.destroy {
                self.destroyed.push(c.clone());
            } else {
                if let Capacity::EndLife(n) = c.description.kind.to_fighting_card().capacity {
                    self.modify_life(n as isize)?;
                }
                self.fighting_discard.push(c.clone());
            }
        }

        self.state = State::None;
        self.next(Event::Start)
    }

    pub fn max_life_points(&self) -> u8 {
        if self.level == 4 {
            20
        } else {
            22
        }
    }

    fn modify_life(&mut self, points: isize) -> Result<(), String> {
        self.life_points += points;

        if self.life_points > self.max_life_points() as isize {
            self.life_points = self.max_life_points() as isize;
        } else if self.life_points < 0 {
            self.life_points = 0;
            self.end_game(false);
            return Err("You died!".to_string());
        }

        Ok(())
    }

    fn use_draw_card(
        &mut self,
        using: &mut Using,
        left: i8,
        right: &mut Vec<Card<'a>>,
    ) -> Result<(), String> {
        right.push(self.fighting_deck_pop(true)?);

        let left = left - 1;
        if left > 0 {
            *using = Using::Draw(left);
        } else {
            *using = Using::None;
        }
        Ok(())
    }

    fn use_sort(&mut self, using: &mut Using<'a>) -> Result<(), String> {
        match using {
            Using::Sort(cards) => {
                if cards.len() >= 3 {
                    return Err("Already drawn 3 cards".to_string());
                }
                cards.push(self.fighting_deck_pop(true)?);
            }
            _ => panic!("Invalid sort"),
        }
        Ok(())
    }

    fn use_card(
        &mut self,
        c: usize,
        using: &mut Using<'a>,
        left: &mut Vec<Card<'a>>,
        right: &mut Vec<Card<'a>>,
    ) -> Result<(), String> {
        let card = left.iter_mut().chain(right.iter_mut()).nth(c).unwrap();
        match card.description.kind.to_fighting_card().capacity {
            Capacity::Life(lp) => {
                self.modify_life(lp as isize)?;
            }
            Capacity::Card(n) => {
                self.use_draw_card(using, n, right)?;
            }
            Capacity::Destroy => {
                *using = Using::Destroy(c);
            }
            Capacity::Double => {
                *using = Using::Double;
            }
            Capacity::Copy => {
                *using = Using::Copy;
            }
            Capacity::LowerStep => {
                self.step_modif += 1;
            }
            Capacity::Sort => {
                *using = Using::Sort(vec![]);
                self.use_sort(using)?;
            }
            Capacity::Swap(n) => {
                *using = Using::Swap(c, n);
            }
            Capacity::UnderDeck => {
                *using = Using::UnderDeck(c);
            }
            _ => {
                *using = Using::None;
            }
        }
        Ok(())
    }

    pub fn free_cards(&self, card: &HazardCard) -> u8 {
        if let Hazard::PirateHazard = card.hazard {
            return self
                .hazard_deck
                .iter()
                .chain(self.hazard_discard.iter())
                .map(|c| c.description.kind.to_hazard_card().free_cards)
                .sum();
        }
        card.free_cards
    }

    pub fn next(&mut self, event: Event) -> Result<(), String> {
        let mut state = self.state.clone();
        match (&mut state, event) {
            (State::None, Event::Start) => {
                if let Some(cards) = self.hazard_pop() {
                    self.state = State::ChooseHazard(cards);
                } else if self.pirate_cards.is_empty() {
                    self.end_game(true);
                } else if self.pirate_cards.len() == 1 {
                    let c = self.pirate_cards.pop().unwrap();
                    self.state = State::Fighting(c, 0, vec![], vec![], Using::None);
                } else {
                    self.state = State::ChoosePirate;
                }
            }
            (State::ChooseHazard(ref mut h), Event::HazardChoice(c)) => {
                if let Some(c) = c {
                    if c >= h.len() {
                        return Err("Invalid choice".to_string());
                    }
                    let card = h.remove(c);
                    self.hazard_discard.append(h);
                    self.state = State::Fighting(card, 0, vec![], vec![], Using::None);
                } else if h.len() == 1 {
                    self.state = State::None;
                    return self.next(Event::Start);
                } else {
                    return Err("You must choose a card".to_string());
                }
            }
            (State::ChoosePirate, Event::Choice(c)) => {
                if c >= self.pirate_cards.len() {
                    return Err("Invalid pirate card".to_string());
                }
                let card = self.pirate_cards.swap_remove(c);
                self.state = State::Fighting(card, 0, vec![], vec![], Using::None);
            }
            (
                State::Fighting(c, ref mut used_free, ref mut left, ref mut right, using),
                Event::Fight,
            ) => {
                let has_stop = |c: &Card<'a>| {
                    !c.destroy && c.description.kind.to_fighting_card().capacity == Capacity::Stop
                };

                let free = if left.iter().any(has_stop) {
                    false
                } else {
                    *used_free < self.free_cards(&c.description.kind.to_hazard_card())
                };

                let card = self.fighting_deck_pop(free)?;
                if free {
                    left.push(card);
                    *used_free += 1;
                } else {
                    right.push(card);
                }

                *using = Using::None;
                self.state = state;
            }
            (
                State::Fighting(_, _, ref mut left, ref mut right, using @ Using::None),
                Event::Use(c),
            ) => {
                let card = left.iter_mut().chain(right.iter_mut()).nth(c);
                if let Some(card) = card {
                    if card.tapped {
                        return Err("Card already used".to_string());
                    }
                    if card.destroy {
                        return Err("Card is to be destroyed".to_string());
                    }
                    card.tapped = true;
                    self.use_card(c, using, left, right)?;
                } else {
                    return Err("Invalid card".to_string());
                }
                self.state = state;
            }
            (State::Fighting(_, _, _, ref mut right, using @ Using::Draw(_)), Event::Continue) => {
                if let Using::Draw(n) = *using {
                    self.use_draw_card(using, n, right)?;
                    self.state = state;
                }
            }
            (State::Fighting(_, _, _, _, using), Event::Break) => {
                *using = Using::None;
                self.state = state;
            }
            (State::Fighting(_, _, left, right, using @ Using::Destroy(_)), Event::Choice(c)) => {
                if let Using::Destroy(d) = *using {
                    if c == d {
                        return Err("You can't destroy the current card".to_string());
                    }
                }
                let card = left.iter_mut().chain(right.iter_mut()).nth(c); // FIXME: make me lambda
                if let Some(card) = card {
                    if card.destroy {
                        return Err("Card already destroyed".to_string());
                    }
                    card.destroy = true;
                    *using = Using::None;
                    self.state = state;
                }
            }
            (State::Fighting(_, _, left, right, using @ Using::Double), Event::Choice(c)) => {
                let card = left.iter_mut().chain(right.iter_mut()).nth(c);
                if let Some(card) = card {
                    if card.destroy {
                        return Err("Card is to be destroyed".to_string());
                    }
                    if card.double {
                        return Err("Card already doubled".to_string());
                    }
                    card.double = true;
                    *using = Using::None;
                    self.state = state;
                }
            }
            (State::Fighting(_, _, left, right, using @ Using::Copy), Event::Choice(c)) => {
                let card = left.iter().chain(right.iter()).nth(c);
                if let Some(card) = card {
                    if card.destroy {
                        return Err("Card is to be destroyed".to_string());
                    }
                    *using = Using::None;
                    self.use_card(c, using, left, right)?;
                    self.state = state;
                }
            }
            (State::Fighting(_, _, _, _, using @ Using::Sort(_)), Event::Continue) => {
                self.use_sort(using)?;
                self.state = state;
            }
            (State::Fighting(_, _, left, right, using @ Using::Swap(_, _)), Event::Choice(c)) => {
                if let Using::Swap(swap_card, n) = using {
                    if c == *swap_card {
                        return Err("You can't swap the current card".to_string());
                    }

                    let card = left.iter_mut().chain(right.iter_mut()).nth(c);
                    if let Some(card) = card {
                        if card.destroy {
                            return Err("Card is to be destroyed".to_string());
                        }

                        self.fighting_discard.push(card.clone());
                        *card = self.fighting_deck_pop(true)?;

                        *n -= 1;
                        if *n == 0 {
                            *using = Using::None;
                        }
                        self.state = state;
                    }
                }
            }
            (
                State::Fighting(_, _, left, right, using @ Using::UnderDeck(_)),
                Event::ChoiceUnder(mut c, replace),
            ) => {
                if let Using::UnderDeck(d) = *using {
                    if c == d {
                        return Err("You can't swap the current card".to_string());
                    }
                }
                if c >= left.len() && replace {
                    return Err("Can't replace a card from the right pile".to_string());
                }

                let mut pile = left;
                if c >= pile.len() {
                    c -= pile.len();
                    if c >= right.len() {
                        return Err("Invalid card".to_string());
                    }
                    pile = right;
                }
                if pile[c].destroy {
                    return Err("Card is already detroyed".to_string());
                }
                let mut card = pile.remove(c);
                card.reset();
                self.fighting_deck.insert(0, card.clone());

                if replace {
                    pile.insert(c, self.fighting_deck_pop(true)?);
                }

                *using = Using::None;
                self.state = state;
            }
            (
                State::Fighting(_, _, _, _, using @ Using::Sort(_)),
                Event::Sort(ref mut order, destroy),
            ) => {
                match using {
                    Using::Sort(v) => {
                        let mut check = order.clone();
                        check.sort();
                        check.dedup();
                        let mut order = &order[..];
                        if v.len() != check.len() {
                            return Err("Invalid sort".to_string());
                        }
                        if destroy {
                            if let Some(c) = v.get(order[0]) {
                                self.destroyed.push(c.clone());
                            } else {
                                return Err("Invalid sort".to_string());
                            }
                            order = &order[1..];
                        }
                        for i in order {
                            if let Some(c) = v.get(*i) {
                                self.fighting_deck.push(c.clone());
                            } else {
                                return Err("Invalid sort".to_string());
                            }
                        }
                    }
                    _ => panic!("Invalid sort state"),
                }
                *using = Using::None;
                self.state = state;
            }
            (State::Fighting(ref c, used_free, ref mut left, ref mut right, _), Event::Win) => {
                if *used_free == 0 {
                    return Err("You must draw at least one card".to_string());
                }
                if self.fight_diff().unwrap_or(-1) < 0 {
                    return Err("It's not possible to win!".to_string());
                }
                if c.description.is_pirate() {
                    self.pirate_cards_won.push(c.clone());
                } else {
                    self.fighting_discard.push(c.clone());
                }
                let cards = [&left[..], &right[..]].concat();
                return self.end_fight(&cards);
            }
            (
                State::Fighting(ref c, used_free, ref left, ref right, _),
                Event::Lose(ref mut discard),
            ) => {
                if c.description.is_pirate() {
                    return Err("Arrr, you can't decide to lose against a pirate!".to_string());
                }
                if *used_free == 0 {
                    return Err("You must first play a card".to_string());
                }
                discard.sort_unstable_by(|a, b| b.cmp(a));
                let mut discard = discard.to_vec();
                discard.dedup();
                let mut cost = 0;
                let mut concat = [&left[..], &right[..]].concat();
                for i in discard.iter() {
                    if let Some(d) = concat.get(*i) {
                        cost += d.description.get_destroy_value().unwrap();
                    } else {
                        return Err("Invalid discard #".to_string());
                    }
                }
                let diff = self.fight_diff().unwrap();
                if cost > diff.abs() {
                    return Err(format!(
                        "Not enough diff:{} for this much discard:{}",
                        diff, cost
                    ));
                }
                self.modify_life(diff)?;
                self.hazard_discard.push(c.clone());
                for i in discard.iter() {
                    let destroyed = concat.swap_remove(*i);
                    self.destroyed.push(destroyed);
                }
                return self.end_fight(&concat);
            }
            (s, e) => {
                return Err(format!("Wrong state, event combination: {:#?} {:#?}", s, e).to_string());
            }
        }
        Ok(())
    }

    fn end_game(&mut self, won: bool) {
        if let Some(left) = self.get_left() {
            self.fighting_discard.extend(left.clone());
        }
        if let Some(right) = self.get_right() {
            self.fighting_discard.extend(right.clone());
        }
        self.state = State::Ended(won);
    }

    pub fn score(&self) -> isize {
        let mut score: isize = self
            .fighting_discard
            .iter()
            .map(|c| match &c.description.kind {
                CardKind::Starting(card) => card.fighting_value,
                CardKind::HazardKnowledge(_, knowledge) => knowledge.fighting_value,
                CardKind::AgingNormal(_) | CardKind::AgingDifficult(_) => -5,
                _ => panic!("Bad fighting card"),
            })
            .sum();
        score += self.pirate_cards_won.len() as isize * 15isize;
        if let State::Ended(true) = self.state {
            score += self.life_points * 5;
        }
        score -= self
            .hazard_deck
            .iter()
            .chain(self.hazard_discard.iter())
            .count() as isize
            * 3isize;
        score
    }

    pub fn get_left(&self) -> Option<&Vec<Card<'a>>> {
        match &self.state {
            State::Fighting(_, _, left, _, _) => Some(left),
            _ => None,
        }
    }

    pub fn get_right(&self) -> Option<&Vec<Card<'a>>> {
        match &self.state {
            State::Fighting(_, _, _, right, _) => Some(right),
            _ => None,
        }
    }

    fn get_fight_value(&self) -> Option<isize> {
        match &self.state {
            State::Fighting(_, _, left, right, _) => {
                let mut cards = left.iter().chain(right.iter()).collect::<Vec<_>>();
                let maxzero = cards
                    .iter()
                    .filter(|c| {
                        !c.destroy
                            && c.description.kind.to_fighting_card().capacity == Capacity::MaxZero
                    })
                    .count();

                if let State::Fighting(c, _, _, _, _) = &self.state {
                    if let Hazard::PirateHalf(_) = c.description.kind.to_hazard_card().hazard {
                        // FIXME: like the rest, untested & buggy ;)
                        let half = (cards.len() + 1) / 2;
                        cards.sort_unstable_by(|a, b| {
                            if b.description.is_aging_normal() || b.description.is_aging_difficult()
                            {
                                return Ordering::Less;
                            } else {
                                b.get_fighting_value().cmp(&a.get_fighting_value())
                            }
                        });
                        cards = cards[..half].to_vec();
                    }
                }

                let mut vals = cards
                    .iter()
                    .map(|c| c.get_fighting_value())
                    .collect::<Vec<_>>();
                vals.sort_unstable();

                let mut val = vals[..vals.len() - maxzero].iter().sum();

                if let State::Fighting(c, _, left, right, _) = &self.state {
                    if let Hazard::PirateAdd(_) = c.description.kind.to_hazard_card().hazard {
                        val += left.len() as isize + right.len() as isize;
                    }
                }
                Some(val)
            }
            _ => None,
        }
    }

    fn hazard_level(&self, hazard: &Hazard, step: Step) -> u8 {
        match *hazard {
            Hazard::Leveled(l) => l[step as usize],
            Hazard::Pirate(l) => l,
            Hazard::PirateTwiceLife(l) => l,
            Hazard::PirateAging => {
                let n_aging = if self.level <= 2 { 10 } else { 11 };
                (n_aging - self.aging_deck.len()) as u8 * 2
            }
            Hazard::PirateHalf(l) => l,
            Hazard::PirateHazard => self
                .hazard_deck
                .iter()
                .chain(self.hazard_discard.iter())
                .map(|c| self.hazard_level(&c.description.kind.to_hazard_card().hazard, Step::Red))
                .sum(),
            Hazard::PirateAdd(l) => l,
        }
    }

    pub fn fight_diff(&self) -> Option<isize> {
        match &self.state {
            State::Fighting(c, _, _, _, _) => {
                let now = self.get_fight_value().unwrap_or(0);
                let mut step = self.step;
                for _ in 0..self.step_modif {
                    step = step.prev();
                }
                let objective =
                    self.hazard_level(&c.description.kind.to_hazard_card().hazard, step) as isize;
                Some(now - objective)
            }
            _ => None,
        }
    }

    fn hazard_pop(&mut self) -> Option<Vec<Card<'a>>> {
        if self.hazard_deck.is_empty() {
            self.step = self.step.next();
            self.hazard_deck.append(&mut self.hazard_discard);
            shuffle(&mut self.hazard_deck);
        }
        if self.step == Step::Pirate {
            return None;
        }
        let n = self.hazard_deck.len().saturating_sub(2);
        Some(self.hazard_deck.split_off(n))
    }

    fn aging(&mut self) {
        self.fighting_deck.append(&mut self.fighting_discard);
        if let Some(add_aging) = self.aging_deck.pop() {
            self.fighting_deck.push(add_aging);
        }
        for c in &mut self.fighting_deck {
            c.reset();
        }
        shuffle(&mut self.fighting_deck);
    }

    fn fighting_deck_pop(&mut self, free: bool) -> Result<Card<'a>, String> {
        let twice = if let State::Fighting(c, _, _, _, _) = &self.state {
            if let Hazard::PirateTwiceLife(_) = c.description.kind.to_hazard_card().hazard {
                true
            } else {
                false
            }
        } else {
            false
        };

        let cost = if twice { -2 } else { -1 };
        if !free {
            self.modify_life(cost)?;
        }
        if self.fighting_deck.is_empty() {
            self.aging();
        }
        if self.fighting_deck.is_empty() {
            self.end_game(false);
            return Err("Fighting deck is empty".to_string());
        }
        Ok(self.fighting_deck.pop().unwrap())
    }

    fn take_two_pirate_cards() -> Vec<Card<'a>> {
        let mut pirates: Vec<_> = CARDS.iter().filter(|c| c.is_pirate()).collect();
        rand::thread_rng().shuffle(&mut pirates);
        pirates[..2].iter().map(|c| Card::new(c)).collect()
    }

    fn make_aging_deck(level: Level) -> Vec<Card<'a>> {
        let mut deck: Vec<_> = CARDS
            .iter()
            .filter(|c| c.is_aging_difficult())
            .flat_map(|c| iter::repeat(c).take(c.start_qty))
            .map(|c| Card::new(c))
            .collect();
        shuffle(&mut deck);
        let mut normal: Vec<_> = CARDS
            .iter()
            .filter(|c| c.is_aging_normal())
            .filter(|c| match level {
                1 | 2 => !c.is_very_stupid(),
                _ => true,
            })
            .flat_map(|c| iter::repeat(c).take(c.start_qty))
            .map(|c| Card::new(c))
            .collect();
        shuffle(&mut normal);
        deck.append(&mut normal);
        deck
    }

    fn make_fighting_deck() -> Vec<Card<'a>> {
        let mut start_cards: Vec<_> = CARDS
            .iter()
            .filter(|c| c.is_starting())
            .flat_map(|c| iter::repeat(c).take(c.start_qty))
            .map(|c| Card::new(c))
            .collect();
        shuffle(&mut start_cards);
        start_cards
    }

    fn make_hazard_deck() -> Vec<Card<'a>> {
        let mut hazard: Vec<_> = CARDS
            .iter()
            .filter(|c| c.is_hazard_knowledge())
            .flat_map(|c| iter::repeat(c).take(c.start_qty))
            .map(|c| Card::new(c))
            .collect();
        shuffle(&mut hazard);
        hazard
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        let mut game = Friday::new(1);

        assert_eq!(game.aging_deck.len(), 10);
        assert_eq!(game.fighting_deck.len(), 18);
        assert_eq!(game.hazard_deck.len(), 28);
        assert_eq!(game.pirate_cards.len(), 2);

        game.next(Event::HazardChoice(Some(0))).unwrap();
        game.next(Event::Fight).unwrap();

        let game = Friday::new(2);

        assert_eq!(game.aging_deck.len(), 9);
        assert_eq!(game.fighting_deck.len(), 19);
        assert_eq!(game.hazard_deck.len(), 28);
        assert_eq!(game.pirate_cards.len(), 2);

        let game = Friday::new(3);

        assert_eq!(game.aging_deck.len(), 10);
        assert_eq!(game.fighting_deck.len(), 19);
        assert_eq!(game.hazard_deck.len(), 28);
        assert_eq!(game.pirate_cards.len(), 2);

        let game = Friday::new(4);

        assert_eq!(game.aging_deck.len(), 10);
        assert_eq!(game.fighting_deck.len(), 19);
        assert_eq!(game.hazard_deck.len(), 28);
        assert_eq!(game.pirate_cards.len(), 2);
    }
}
