use chess_gui::{run_gui, best_move, Board};

fn main() -> iced::Result {
    run_gui(best_move, Board::default())
}