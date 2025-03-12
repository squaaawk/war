use fastrand::Rng;
use std::cmp::Ordering;

pub enum Player {
  Player1,
  Player2,
}

/// The winner of a game (repeated rounds, until one player has the entire deck).
/// The game may draw if both players flip their last card in a war.
pub enum GameResult {
  Player1,
  Player2,
  Draw,
}

/// The winner of an individual round, which may consist of one or more wars.
enum RoundResult {
  GameResult(GameResult),
  RoundWin(Player),
}

/// The cards owned by one player. Cards are drawn from the deck, until it is empty,
/// at which point the entire discard is shuffled to become the new deck.
#[derive(Clone)]
pub struct PlayerDeck {
  deck: Vec<u8>,
  discard: Vec<u8>,
}

impl PlayerDeck {
  pub fn new(deck: Vec<u8>) -> Self {
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

/// The current state of a game of war.
pub struct Game {
  rng: Rng,
  player1: PlayerDeck,
  player2: PlayerDeck,

  /// k cards are flipped face-down in a war
  k: u32,

  /// A workspace vector, storing all the cards won in a single round
  work: Vec<u8>,
}

impl Game {
  /// Create (but do not simulate) a new game with the given player decks.
  pub fn new(rng: Rng, player1: PlayerDeck, player2: PlayerDeck, k: u32) -> Self {
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

  /// Plays this game to completion, returning the winner and the number of turns taken.
  pub fn play(&mut self) -> (GameResult, u64) {
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
