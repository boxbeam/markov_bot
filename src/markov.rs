use rand::Rng;
use std::collections::{HashMap, VecDeque};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
enum Symbol {
    Start,
    End,
    Null,
    Char(char),
}

impl Symbol {
    fn char(self) -> Option<char> {
        if let Symbol::Char(c) = self {
            Some(c)
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Token<const N: usize> {
    symbols: [Symbol; N],
}

impl<const N: usize> Token<N> {
    fn new(symbols: &[Symbol]) -> Self {
        let mut array = [Symbol::Null; N];
        for (i, symbol) in symbols.iter().take(N).enumerate() {
            array[i] = *symbol;
        }
        Token { symbols: array }
    }
}

#[derive(Debug)]
pub struct MarkovChain<const LOOKBEHIND: usize> {
    map: HashMap<Token<LOOKBEHIND>, MarkovEntry>,
    entries: VecDeque<String>,
    pub cache_size: usize,
}

impl<const N: usize> Default for MarkovChain<N> {
    fn default() -> Self {
        MarkovChain {
            map: Default::default(),
            entries: Default::default(),
            cache_size: 1000,
        }
    }
}

impl<const N: usize> MarkovChain<N> {
    pub fn digest(&mut self, input: &str) {
        let mut symbols = vec![Symbol::Start];
        symbols.extend(input.chars().map(Symbol::Char));
        symbols.push(Symbol::End);

        for i in 1..symbols.len() {
            let start = i.saturating_sub(N);
            let token: Token<N> = Token::new(&symbols[start..i]);
            let symbol = symbols[i];
            self.map.entry(token).or_default().insert(symbol);
        }

        self.entries.push_back(input.to_string());
        if self.entries.len() > self.cache_size {
            let entry = self.entries.pop_front().unwrap();
            self.undigest(&entry);
        }
    }

    fn undigest(&mut self, input: &str) {
        let mut symbols = vec![Symbol::Start];
        symbols.extend(input.chars().map(Symbol::Char));
        symbols.push(Symbol::End);

        for i in 1..symbols.len() {
            let start = i.saturating_sub(N);
            let token: Token<N> = Token::new(&symbols[start..i]);
            let symbol = symbols[i];
            self.map.entry(token).or_default().remove(symbol);
        }
    }

    pub fn generate(&self) -> String {
        let mut symbols = vec![Symbol::Start];

        while symbols.last().is_none_or(|last| *last != Symbol::End) {
            let start = symbols.len().saturating_sub(N);
            let token = Token::new(&symbols[start..symbols.len()]);

            if let Some(entry) = self.map.get(&token) {
                symbols.push(entry.weighted_random());
            } else {
                symbols.push(Symbol::Null);
            }
        }

        symbols.into_iter().filter_map(Symbol::char).collect()
    }
}

#[derive(Default, Debug)]
struct MarkovEntry {
    total: usize,
    map: HashMap<Symbol, usize>,
}

impl MarkovEntry {
    fn insert(&mut self, symbol: Symbol) {
        self.total += 1;
        *self.map.entry(symbol).or_default() += 1;
    }

    fn remove(&mut self, symbol: Symbol) {
        match self.map.get(&symbol) {
            Some(count) if *count > 1 => {
                self.map.insert(symbol, count - 1);
                self.total -= 1;
            }
            Some(_) => {
                self.map.remove(&symbol);
                self.total -= 1;
            }
            None => {}
        }
    }

    fn weighted_random(&self) -> Symbol {
        let mut weight: isize = rand::thread_rng().gen_range(1..=self.total) as isize;

        for (symbol, symbol_weight) in &self.map {
            weight -= *symbol_weight as isize;
            if weight <= 0 {
                return *symbol;
            }
        }

        unreachable!()
    }
}
