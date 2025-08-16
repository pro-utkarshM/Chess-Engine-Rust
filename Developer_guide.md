# Rust Chess Engine: Developer's Guide

## 1. Introduction

Welcome to the Rust Chess Engine project! This document serves as a guide for developers looking to understand, contribute to, or extend the codebase.

The project consists of two main parts:
1.  A core **chess engine library** (`chess-engine`) written in pure Rust. It is responsible for all game logic, including move generation, validation, and state management. It is designed to be completely independent of any user interface.
2.  A graphical **user interface** (`chess-gui`), built using the `iced` framework, which acts as a front-end to the core engine. It allows a human player to play against the AI.

The engine's AI is built upon the **Minimax algorithm** with **Alpha-Beta pruning** for optimization, a classic and effective approach for turn-based strategy games.

## 2. Project Structure

The project is organized as a Cargo workspace, which allows the engine and the GUI to be managed as separate but related crates.

```
Chess-Engine-Rust/
│
├── Cargo.toml          # Defines the workspace and the main `chess-engine` crate.
│
├── src/                # Source code for the `chess-engine` library.
│   ├── lib.rs          # Main library file, defines core enums (Move, Color, etc.).
│   ├── board.rs        # Contains the Board struct, the heart of game state.
│   ├── piece.rs        # Defines the Piece enum and its associated logic.
│   ├── position.rs     # Defines the Position struct for board coordinates.
│   ├── game.rs         # High-level game management wrapper.
│   └── util.rs         # FEN/SAN parsing utilities.
│
├── examples/
│   ├── terminal.rs     # A simple terminal-based interface for the engine.
│   └── chess-gui/      # The main graphical user interface crate.
│       ├── Cargo.toml  # Dependencies for the GUI (iced, rand, etc.).
│       └── src/
│           ├── lib.rs  # The core logic for the iced application.
│           └── bins/
│               └── best.rs # The main executable for the GUI.
│
└── assets/
    └── pieces/         # Contains the SVG images for the chess pieces.```
```

## 3. Core Engine (`chess-engine` crate)

The `chess-engine` crate is the foundation of the project. It knows the rules of chess but knows nothing about how to display a board or handle user input.

### Key Modules and Concepts

*   **`board.rs`**: This is the most critical module. The `Board` struct represents the entire state of a chess game at any given moment. It tracks:
    *   The position of every piece on its 64 squares.
    *   Whose turn it is (`Color`).
    *   Castling rights for both players.
    *   The en passant square, if any.
    *   It contains all the logic for move validation (`is_legal_move`) and application (`play_move`).

*   **`piece.rs`**: The `Piece` enum defines the six types of pieces (King, Queen, etc.), each holding its `Color` and `Position`. This module also contains the logic for how each piece moves and its material and positional value.

*   **`lib.rs` and the `Evaluate` Trait**: `lib.rs` exports the public API of the engine. The `Evaluate` trait is the key abstraction for the AI. It provides a generic interface for:
    *   **`value_for()`**: Evaluating the board from a player's perspective. A positive score is good, negative is bad.
    *   **`get_legal_moves()`**: Generating all possible moves for the current player.
    *   **`apply_eval_move()`**: Applying a move to get a new board state.
    *   **`get_best_next_move()` & `minimax()`**: This is the AI. It recursively explores future moves to a certain depth, assuming the opponent will always make their best move, and chooses the path that leads to the best outcome for itself.

## 4. Graphical User Interface (`chess-gui` example)

The GUI is built using the `iced` framework, which follows **The Elm Architecture (Model-View-Update)**.

*   **Model (`GameUI` struct)**: This struct holds the *entire state* of the UI. It contains the `Board`, the position of the currently selected piece (`from_square`), a flag for pawn promotion (`promotion_state`), and lists of captured pieces. The UI is a direct function of this state.

*   **View (`view` method)**: This function is called every time the state changes. It takes the current `GameUI` state (`&self`) and returns a tree of widgets that `iced` draws to the screen. It does not contain any game logic; it only knows how to display the current state.

*   **Update (`update` method)**: This is the "brain" of the UI. It receives `Message` enums (e.g., `SelectSquare(Position)`) that are triggered by user interactions. Its job is to process the message, update the `GameUI` state accordingly, and optionally return a `Command` to perform an asynchronous task.

### Key Logic Flows

*   **Human Move:**
    1.  User clicks a square -> `Button` sends `Message::SelectSquare(pos)`.
    2.  The `update` function receives the message. If it's the human's turn and a piece was selected, it stores the position in `from_square`.
    3.  User clicks a second square -> another `SelectSquare` message is sent.
    4.  The `update` function now has a `from` and a `to`. It constructs a `Move`, validates it, and calls `play_human_move`.

*   **AI Move (Asynchronous Task):**
    1.  After the human's move is processed, `play_human_move` returns a `Command::perform`.
    2.  This command runs the `get_best_next_move` function in a background thread, preventing the UI from freezing while the AI is "thinking."
    3.  When the AI calculation is finished, the `Command` sends the result (the chosen `Move`) back to the `update` function as a `Message::CpuMove`.
    4.  The `update` function receives `CpuMove`, applies it to the board, and the game continues.

*   **Pawn Promotion:**
    1.  The `update` function detects that a human has moved a pawn to the final rank.
    2.  Instead of making a move, it sets the `promotion_state` variable.
    3.  The `view` function now sees that `promotion_state` is active and renders the promotion choice UI instead of the normal 8th rank.
    4.  The user clicks a promotion piece (e.g., Queen), sending a `Message::Promote(piece)`.
    5.  The `update` function receives this message, constructs the final `Move::Promotion`, and plays it.

## 5. How to Build and Run

### Prerequisites

You need the Rust toolchain and platform-specific build dependencies for the windowing system.

**On Debian/Ubuntu:**
```bash
sudo apt update
sudo apt install -y pkg-config libx11-dev libxcursor-dev libxrandr-dev libxinerama-dev libxi-dev libgl1-mesa-dev libwayland-dev libxkbcommon-dev
```

### Running the GUI Application

The graphics library can sometimes have issues with hybrid GPU setups (NVIDIA/AMD) on Linux. The most reliable way to run the application is by forcing it to use the OpenGL backend.

```bash
WGPU_BACKEND=gl cargo run -p chess-gui --bin best
```

### Running the Terminal Interface

For environments without a graphical desktop (like a server or a CI/CD runner), or for quick, dependency-free testing, use the terminal version:

```bash
cargo run --bin chess
```

### Running Tests

To run all unit and integration tests for both the engine and the GUI:

```bash
cargo test --all
```

## 6. How to Contribute and Extend

This project is a solid foundation. Here are some ideas for extending it:

*   **AI Improvements:**
    *   Implement different AI difficulty levels by changing the `AI_DEPTH`.
    *   Add more advanced evaluation metrics to the `piece.rs` weighted values.
    *   Implement move ordering to make alpha-beta pruning even more effective.
*   **UI Features:**
    *   Add a "Hint" button that highlights the AI's best move for the human player.
    *   Display the game's move history (PGN) in the side panel.
    *   Allow the human to play as Black.
*   **Game Logic:**
    *   Implement threefold repetition detection for draws.
    *   Add support for the fifty-move rule.

If you wish to contribute, please follow standard development practices:
1.  Run `cargo fmt` to format your code.
2.  Run `cargo clippy` to check for common mistakes.
3.  Ensure `cargo test --all` passes before submitting a pull request.

---