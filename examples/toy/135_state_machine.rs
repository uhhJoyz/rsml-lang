// Toy 135: a state machine.
//
// What it is: a coin-operated turnstile. It is locked; a coin opens it, a
// push lets one person through and locks it again. It counts coins,
// people let through and pushes refused.
//
// When to use it: any state machine: the transition function is one
// `match` on the pair (state, event) (toy 069), each case building the
// next value with `{ t with .. }` (toy 095); running it is a `fold`, or a
// loop that assigns with `<-` (toy 090).
//
// How to write it: write the table as cases. Rust checks that every pair
// has a case.
//
// Reference: [Patterns](../../docs/notation.md#patterns)

use rsml::rsml;

pub const SUMMARY: &str = "A turnstile: a `match` on (state, event), `{ t with .. }`";

rsml! {
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum State {
    Locked,
    Open,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Event {
    Coin,
    Push,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Turnstile {
    pub state: State,
    pub coins: u32,
    pub passed: u32,
    pub refused: u32,
}

/// A new turnstile: locked, nothing counted.
pub fn locked() -> Turnstile {
    Turnstile { state = State::Locked; coins = 0; passed = 0; refused = 0 }
}

/// The transition function: the turnstile after one event.
pub fn step(t: Turnstile, e: Event) -> Turnstile {
    match (t.state, e) with
    | (State::Locked, Event::Coin) -> { t with state = State::Open; coins = t.coins + 1 }
    | (State::Open, Event::Push) -> { t with state = State::Locked; passed = t.passed + 1 }
    | (State::Open, Event::Coin) -> { t with coins = t.coins + 1 }
    | (State::Locked, Event::Push) -> { t with refused = t.refused + 1 }
}

/// The events written as letters, `c` a coin and `p` a push; other
/// letters are skipped.
pub fn events(letters: &str) -> Vec<Event> {
    // `function` makes a closure that matches its argument (toy 016);
    // `Some Event::Coin` is `Some(Event::Coin)`.
    letters
    |> .chars()
    |> .filter_map(function 'c' -> Some Event::Coin | 'p' -> Some Event::Push | _ -> None)
    |> .collect()
}

/// The turnstile after all the events: `step` is passed by name.
pub fn simulate(evs: &[Event]) -> Turnstile {
    evs |> .iter() |> .copied() |> .fold(locked (), step)
}

/// The state after each event, by a loop over a mutable turnstile.
pub fn trace(evs: &[Event]) -> Vec<State> {
    let mut t = locked () in
    let mut states = Vec.new () in
    for e in evs do
      t <- step t !e;
      states.push(t.state)
    done;
    states
}
}

pub fn run() {
    let evs = events("pp cp cp c");
    println!("events \"pp cp cp c\" = {evs:?}");
    println!("simulate: {:?}", simulate(&evs));
    println!("trace \"cpp\" = {:?}", trace(&events("cpp")));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_day_at_the_turnstile() {
        let evs = events("pp cp cp c");
        assert_eq!(evs.len(), 7);
        let t = simulate(&evs);
        assert_eq!(t.state, State::Open);
        assert_eq!((t.coins, t.passed, t.refused), (3, 2, 2));
    }

    #[test]
    fn states_one_by_one() {
        use State::*;
        assert_eq!(trace(&events("cpp")), [Open, Locked, Locked]);
        assert_eq!(simulate(&[]), locked());
    }
}
