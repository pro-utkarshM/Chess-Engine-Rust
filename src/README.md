---

# **Game Module Documentation**

This module manages the core chess game logic, including move validation, game actions, and status tracking.

---

## **Overview**
The `Game` module provides an abstraction for managing chess games:
- Supports moves using SAN (Standard Algebraic Notation).
- Offers/accepts draws.
- Tracks game status (e.g., checkmate, stalemate, resignation).

---

## **Key Enums**

### **GameAction**
Defines player actions:
- `AcceptDraw`: Accept a draw.
- `MakeMove(String)`: Make a move using SAN.
- `OfferDraw(String)`: Make a move and offer a draw.
- `Resign`: Resign the game.

#### Example:
```rust
let action = GameAction::from("e4"); // Makes the move e4
```

---

### **GameError**
Common errors:
- `AmbiguousMove`: SAN move is ambiguous.
- `GameAlreadyOver`: Action attempted after game ended.
- `InvalidMove`: Move invalid for current board state.
- `InvalidPosition`: FEN string is invalid.

---

### **GameOver**
Possible game-ending states:
- `WhiteCheckmates`, `BlackCheckmates`
- `WhiteResigns`, `BlackResigns`
- `Stalemate`, `DrawAccepted`

---

## **Game Struct**
Manages the game state:
- `board`: Current chessboard state.
- `draw_offered`: Tracks if a draw is offered.
- `status`: Game status (active or ended).

---

## **Core Methods**

1. **`make_move(&mut self, action: &GameAction)`**  
   Executes a player action.  
   **Returns:** `Result<&Option<GameOver>, GameError>`

   Example:
   ```rust
   game.make_move(&GameAction::from("e4"))?;
   ```

2. **`to_fen(&self, halfmove_clock: u8, fullmove_number: u8)`**  
   Converts the current game state to a FEN string.

3. **`get_turn_color(&self)`**  
   Returns the color of the player whose turn it is.

---

## **Tests**
- **`test_game_moves`**: Verifies a sequence of valid moves.
- **`test_fools_mate`**: Simulates Fool's Mate and checks the result.
- **`test_promotion`**: Ensures pawn promotion works correctly.

Example:
```rust
#[test]
fn test_fools_mate() {
    let mut game = Game::default();
    game.make_move(&GameAction::from("f3")).unwrap();
    game.make_move(&GameAction::from("e5")).unwrap();
    game.make_move(&GameAction::from("g4")).unwrap();
    game.make_move(&GameAction::from("Qh4")).unwrap();
    assert_eq!(game.status, Some(GameOver::BlackCheckmates));
}
```

---

## **Purpose**
- Simplifies onboarding for new developers.
- Highlights key functionalities and error handling.
- Reduces time spent reading code.

--- 
