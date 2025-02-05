use fastrand::Rng;
use std::cmp::Ordering;

// ================ Simulation ================

enum Player {
  Player1,
  Player2,
}

enum GameResult {
  Player1,
  Player2,
  Draw,
}

enum RoundResult {
  GameResult(GameResult),
  RoundWin(Player),
}

#[derive(Clone)]
struct PlayerDeck {
  deck: Vec<u8>,
  discard: Vec<u8>,
}

impl PlayerDeck {
  fn new(deck: Vec<u8>) -> Self {
    Self {
      deck: Vec::new(),
      discard: deck,
    }
  }

  fn cards(&self) -> usize {
    self.deck.len() + self.discard.len()
  }

  fn draw(&mut self, rng: &mut Rng) -> Option<u8> {
    if self.deck.is_empty() {
      rng.shuffle(&mut self.discard);
      std::mem::swap(&mut self.deck, &mut self.discard);
    }

    self.deck.pop()
  }

  fn win_loot(&mut self, cards: &[u8]) {
    self.discard.extend_from_slice(cards);
  }
}

struct Game {
  rng: Rng,
  player1: PlayerDeck,
  player2: PlayerDeck,

  /// k cards are flipped face-down in a war
  k: u32,

  /// A workspace vector, storing all the cards won in a single round
  work: Vec<u8>,
}

impl Game {
  fn new(rng: Rng, player1: PlayerDeck, player2: PlayerDeck, k: u32) -> Self {
    Self {
      rng,
      player1,
      player2,
      k,
      work: Vec::new(),
    }
  }

  fn play_round(&mut self) -> RoundResult {
    self.work.clear();

    loop {
      // Each player plays a card, if possible. If they are out of cards, they have lost
      let (card1, card2) = match (
        self.player1.draw(&mut self.rng),
        self.player2.draw(&mut self.rng),
      ) {
        (None, None) => return RoundResult::GameResult(GameResult::Draw),
        (None, Some(_)) => return RoundResult::GameResult(GameResult::Player2),
        (Some(_), None) => return RoundResult::GameResult(GameResult::Player1),
        (Some(card1), Some(card2)) => (card1, card2),
      };

      self.work.extend([card1, card2]);

      // If the cards are different, one player wins the round
      // If the cards are equal, each player plays up to `k` face-down cards (leaving at least one card in their deck) and we repeat
      match card1.cmp(&card2) {
        Ordering::Greater => return RoundResult::RoundWin(Player::Player1),
        Ordering::Less => return RoundResult::RoundWin(Player::Player2),

        Ordering::Equal => {
          let n = self.player1.cards().saturating_sub(1).min(self.k as usize);
          self
            .work
            .extend((0..n).map(|_| self.player1.draw(&mut self.rng).unwrap()));

          let n = self.player2.cards().saturating_sub(1).min(self.k as usize);
          self
            .work
            .extend((0..n).map(|_| self.player2.draw(&mut self.rng).unwrap()));
        }
      }
    }
  }

  fn play(&mut self) -> (GameResult, u64) {
    let mut turn = 0;
    loop {
      turn += 1;

      match self.play_round() {
        RoundResult::RoundWin(Player::Player1) => self.player1.win_loot(&self.work),
        RoundResult::RoundWin(Player::Player2) => self.player2.win_loot(&self.work),
        RoundResult::GameResult(result) => return (result, turn),
      }
    }
  }
}

// ================ Scenarios ================

fn mean(data: impl Iterator<Item = f64>, n: usize) -> f64 {
  data.sum::<f64>() / n as f64
}

fn mean_stddev(data: &[f64]) -> (f64, f64) {
  let mu = mean(data.iter().copied(), data.len());
  let variance = mean(data.iter().map(|&x| (x - mu).powi(2)), data.len());
  (mu, variance.sqrt())
}

fn standard_games() {
  fn simulate<F>(n_games: usize, path: Option<&str>, f: F)
  where
    F: Fn(&mut Rng) -> (PlayerDeck, PlayerDeck),
  {
    // Simulate
    let start = std::time::Instant::now();

    let mut rng = Rng::new();
    let (wins, turns): (Vec<_>, Vec<_>) = (0..n_games)
      .map(|_| {
        let (player1, player2) = f(&mut rng);
        let mut game = Game::new(rng.fork(), player1, player2, 3);
        game.play()
      })
      .unzip();

    let elapsed = start.elapsed();

    // Write data, if requested
    if let Some(path) = path {
      let string = serde_json::to_string(&turns).unwrap();
      std::fs::write(path, string).unwrap();
    }

    // Statistics
    let mean_score = mean(
      wins.into_iter().map(|win| match win {
        GameResult::Player1 => 1.0,
        GameResult::Player2 => 0.0,
        GameResult::Draw => 0.5,
      }),
      n_games,
    );

    let turns: Vec<_> = turns.into_iter().map(|x| x as f64).collect();
    let (mean_turns, stddev_turns) = mean_stddev(&turns);

    println!("  {n_games} games in {elapsed:?}");
    println!("  mean score: Player 1 wins {:.1}%", 100.0 * mean_score);
    println!("  mean turns: {mean_turns:.2} +/- {stddev_turns:.2}");
  }

  /// A standard deck of cards with `n` types, `k` copies each
  fn standard_deck(n: u8, k: usize) -> Vec<u8> {
    (1..=n).flat_map(|i| [i].repeat(k)).collect()
  }

  println!("Standard war (shuffled):");
  simulate(1_000_000, Some("data.json"), |rng| {
    let mut deck = standard_deck(13, 4);
    rng.shuffle(&mut deck);

    let player1 = PlayerDeck::new(deck[..26].to_vec());
    let player2 = PlayerDeck::new(deck[26..].to_vec());
    (player1, player2)
  });

  println!();
  println!("Standard war (evenly split):");
  simulate(1_000_000, None, |_| {
    let player1 = PlayerDeck::new(standard_deck(13, 2));
    let player2 = PlayerDeck::new(standard_deck(13, 2));
    (player1, player2)
  });

  println!();
  println!("2-deck war (evenly split):");
  simulate(100_000, None, |_| {
    let player1 = PlayerDeck::new(standard_deck(13, 4));
    let player2 = PlayerDeck::new(standard_deck(13, 4));
    (player1, player2)
  });

  println!();
  println!("Aces vs. the world:");
  simulate(100_000, None, |_| {
    let player1 = PlayerDeck::new([13].repeat(4).to_vec());
    let player2 = PlayerDeck::new((1..=12).flat_map(|i| [i].repeat(4)).collect());
    (player1, player2)
  });
}

fn small_games() {
  let n_games = 100_000;

  use comfy_table::presets::UTF8_FULL;
  use comfy_table::{Cell, Table};

  use std::iter::once;

  fn simulate(n_games: usize, n: u8, k: u32) -> f64 {
    let mut rng = Rng::new();
    let deck = PlayerDeck::new((0..n).collect());

    let turns = (0..n_games).map(|_| {
      let mut game = Game::new(rng.fork(), deck.clone(), deck.clone(), k);
      let (_, turns) = game.play();
      turns as f64
    });

    mean(turns, n_games)
  }

  // Draw table
  let mut table = Table::new();
  table.load_preset(UTF8_FULL);

  let ns = 1..=13;
  let ks = 0..10;

  table.set_header(once(Cell::new("n/k")).chain(ks.clone().map(Cell::new)));

  for n in ns {
    let row = ks.clone().map(|k| {
      let turns = simulate(n_games, n, k);
      format!("{:.1}", turns)
    });

    table.add_row(once(format!("{n}")).chain(row));
  }

  println!();
  println!("Small games:");
  println!("  Each player has deck of n unique cards, and k cards are flipped faced-down in a war");
  println!("  games per cell: {n_games}");
  println!("{table}");
}

fn main() {
  standard_games();
  small_games();
}
