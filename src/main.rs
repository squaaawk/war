mod sim;

use fastrand::Rng;
use sim::{Game, GameResult, PlayerDeck};

/// Computes the mean of an iterator of f64s.
fn mean(data: impl Iterator<Item = f64>, n: usize) -> f64 {
  data.sum::<f64>() / n as f64
}

/// Computes the mean and standard deviation of an iterator of f64s.
fn mean_stddev(data: &[f64]) -> (f64, f64) {
  let mu = mean(data.iter().copied(), data.len());
  let variance = mean(data.iter().map(|&x| (x - mu).powi(2)), data.len());
  (mu, variance.sqrt())
}

/// A standard deck of cards with `n` types, `k` copies each.
fn standard_deck(n: u8, k: usize) -> Vec<u8> {
  (1..=n).flat_map(|i| [i].repeat(k)).collect()
}

/// Simulates a bunch of games using the given function to determine the player's initial decks.
/// If a path is given, saves a list of the individual game lengths as a json file.
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

/// Simulates a large number of games of a few game setups, and prints out information about them.
/// Additionally writes out `standard_war.json`, a list of game lengths for standard war games.
fn standard_games() {
  println!("Standard war (shuffled):");
  simulate(1_000_000, Some("standard_war.json"), |rng| {
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
  println!("12-deck war (evenly split):");
  simulate(10_000, None, |_| {
    let player1 = PlayerDeck::new(standard_deck(13, 4 * 6));
    let player2 = PlayerDeck::new(standard_deck(13, 4 * 6));
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

/// Simulates a large number of small-deck games with various number of flipped cards
/// in a war, and pretty-prints them in a well-formatted table
fn small_games() {
  let n_games = 100_000;

  use comfy_table::presets::UTF8_FULL;
  use comfy_table::{Cell, Table};

  use std::iter::once;

  /// Simulates a bunch of games where each player has `n` unique cards and `k` cards are flipped
  /// in a war, returning the av
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
