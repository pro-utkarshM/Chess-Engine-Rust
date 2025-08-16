use iced::widget::{button, container, svg, text, Button, Column, Row, Svg};
use iced::{
    executor, theme, Application, Command, Element, Length, Settings, Size, Subscription, Theme,
};
use rand::{seq::SliceRandom, thread_rng};
use std::collections::HashMap;

use chess_engine::*;
pub use chess_engine::Board;
use std::sync::Mutex;

// --- Asset Loading (Loads all SVGs into memory once at startup) ---
lazy_static::lazy_static! {
    static ref PIECE_IMAGES: HashMap<String, svg::Handle> = {
        let mut map = HashMap::new();
        let pieces = ["pawn", "rook", "knight", "bishop", "queen", "king"];
        let colors = [("w", chess_engine::Color::White), ("b", chess_engine::Color::Black)];

        for piece_name in pieces {
            for (color_char, color_enum) in colors {
                let key = format!("{}{:?}", piece_name, color_enum);
                let path = format!("assets/pieces/{}-{}.svg", piece_name, color_char);
                let handle = svg::Handle::from_path(path);
                map.insert(key, handle);
            }
        }
        map
    };

    static ref GET_CPU_MOVE: Mutex<fn(&Board) -> Move> = Mutex::new(best_move);
    static ref STARTING_BOARD: Mutex<Board> = Mutex::new(Board::default());
}

// --- Constants ---
const SQUARE_SIZE: f32 = 64.0;
pub const AI_DEPTH: i32 = if cfg!(debug_assertions) { 2 } else { 4 };
const HUMAN_PLAYER: chess_engine::Color = chess_engine::Color::White;
const AI_PLAYER: chess_engine::Color = chess_engine::Color::Black;

/// Main entry point for launching the GUI.
pub fn run_gui(get_cpu_move: fn(&Board) -> Move, starting_board: Board) -> iced::Result {
    *GET_CPU_MOVE.lock().unwrap() = get_cpu_move;
    *STARTING_BOARD.lock().unwrap() = starting_board;

    GameUI::run(Settings {
        window: iced::window::Settings {
            size: Size::new((SQUARE_SIZE * 8.0) + 350.0, SQUARE_SIZE * 8.0 + 50.0),
            resizable: false,
            ..iced::window::Settings::default()
        },
        ..Settings::default()
    })
}

// --- AI Move Functions ---
pub fn best_move(board: &Board) -> Move { board.get_best_next_move(AI_DEPTH).0 }
pub fn worst_move(board: &Board) -> Move { board.get_worst_next_move(AI_DEPTH).0 }
pub fn random_move(board: &Board) -> Move {
    let moves = board.get_legal_moves();
    let mut rng = thread_rng();
    *moves.choose(&mut rng).unwrap()
}

// --- Application State and Messages ---
#[derive(Default)]
pub struct GameUI {
    board: Board,
    from_square: Option<Position>,
    promotion_state: Option<(Position, Position)>,
    game_over_message: Option<String>,
    captured_white: Vec<Piece>,
    captured_black: Vec<Piece>,
}

#[derive(Debug, Clone)]
pub enum Message {
    SelectSquare(Position),
    Promote(Piece),
    NewGame,
    CpuMove(Move),
}

// --- Main Application Implementation ---
impl Application for GameUI {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let mut game = GameUI::default();
        game.board = *STARTING_BOARD.lock().unwrap();
        (game, Command::none())
    }

    fn title(&self) -> String { String::from("Rust Chess Engine") }

    fn update(&mut self, message: Message) -> Command<Message> {
        if self.game_over_message.is_some() {
            if let Message::NewGame = message { *self = GameUI::default(); self.board = *STARTING_BOARD.lock().unwrap(); }
            return Command::none();
        }

        if let Message::Promote(piece) = message {
            if let Some((from, to)) = self.promotion_state {
                let promo_move = Move::Promotion(from, to, piece);
                self.promotion_state = None;
                return self.play_human_move(promo_move);
            }
        }
        
        if self.promotion_state.is_some() { return Command::none(); }

        match message {
            Message::SelectSquare(pos) => {
                if self.board.get_turn_color() == HUMAN_PLAYER {
                    match self.from_square {
                        None => { if self.board.has_ally_piece(pos, HUMAN_PLAYER) { self.from_square = Some(pos); } }
                        Some(from) => {
                            if is_promotion_move(from, pos, &self.board) {
                                self.promotion_state = Some((from, pos));
                                self.from_square = None;
                            } else {
                                let m = determine_move(from, pos, &self.board);
                                self.from_square = None;
                                return self.play_human_move(m);
                            }
                        }
                    }
                }
            }
            Message::CpuMove(cpu_move) => {
                if self.board.get_turn_color() == AI_PLAYER {
                    if let Some(captured) = self.board.get_piece(cpu_move_target(cpu_move)) { self.add_capture(captured); }
                    let game_result = self.board.play_move(cpu_move);
                    match game_result {
                        GameResult::Continuing(next_board) => self.board = next_board,
                        _ => self.handle_game_over(game_result),
                    }
                }
            }
            _ => {}
        }
        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> { Subscription::none() }

    fn view(&self) -> Element<Message> {
        let board_view = self.chessboard_view();
        let captured_white_text: String = self.captured_white.iter().map(get_symbol_char).collect();
        let captured_black_text: String = self.captured_black.iter().map(get_symbol_char).collect();

        let mut info_panel = Column::new().padding(10).spacing(20).width(Length::Fixed(250.0))
            .push(text(format!("Turn: {}", self.board.get_turn_color())).size(24))
            .push(text(format!("Captured (White):\n{}", captured_black_text)).size(20))  // Fixed: White captured = black pieces
            .push(text(format!("Captured (Black):\n{}", captured_white_text)).size(20)); // Fixed: Black captured = white pieces

        if self.game_over_message.is_some() {
            info_panel = info_panel.push(Button::new(text("New Game")).on_press(Message::NewGame));
            if let Some(msg) = &self.game_over_message {
                info_panel = info_panel.push(text(msg).size(24));
            }
        }

        let main_layout = Row::new().padding(20).spacing(20).push(board_view).push(info_panel);
        container(main_layout).width(Length::Fill).height(Length::Fill).center_x().center_y().into()
    }

    fn theme(&self) -> Self::Theme { Theme::Light }
}

// --- View Rendering Logic ---
impl GameUI {
    fn chessboard_view(&self) -> Element<Message> {
        // Render ranks 7 down to 0 (top to bottom visually)
        (0..8).rev().fold(Column::new().spacing(0), |col, r| {
            let row_element = if self.promotion_state.is_some() && r == self.promotion_state.unwrap().1.get_row() {
                self.promotion_row_view(r)
            } else {
                self.board_row_view(r)
            };
            col.push(row_element)
        }).into()
    }

    fn board_row_view(&self, r: i32) -> Row<Message> {
        (0..8).fold(Row::new().spacing(0), |row, f| {
            let pos = Position::new(r, f);
            row.push(
                Button::new(self.square_content(pos))
                    .width(Length::Fixed(SQUARE_SIZE))
                    .height(Length::Fixed(SQUARE_SIZE))
                    .style(theme::Button::Custom(Box::new(self.square_style(pos))))
                    .on_press(Message::SelectSquare(pos)),
            )
        })
    }
    
    fn promotion_row_view(&self, r: i32) -> Row<Message> {
        let (from, to) = self.promotion_state.unwrap();
        let color = self.board.get_turn_color();
        let promotion_pieces = [
            Piece::Queen(color, to), Piece::Rook(color, to),
            Piece::Bishop(color, to), Piece::Knight(color, to),
        ];

        let mut promotion_buttons = Row::new().spacing(0);
        for piece in promotion_pieces {
            let content: Element<_> = if let Some(handle) = get_image_handle(&piece) {
                Svg::<Theme>::new(handle.clone()).width(Length::Fill).height(Length::Fill).into()
            } else { text("?").size(48).into() };
            
            promotion_buttons = promotion_buttons.push(
                Button::new(content)
                    .width(Length::Fixed(SQUARE_SIZE))
                    .height(Length::Fixed(SQUARE_SIZE))
                    .style(theme::Button::Custom(Box::new(SquareStyle::Promotion)))
                    .on_press(Message::Promote(piece)),
            );
        }

        let mut final_row = Row::new().spacing(0);
        for f in 0..8 {
            if f == from.get_col() {
                final_row = final_row.push(promotion_buttons);
                return final_row; 
            } else if f > from.get_col() && f < from.get_col() + 4 {
                continue;
            } else {
                let pos = Position::new(r, f);
                 final_row = final_row.push(
                    container(text(""))
                        .width(Length::Fixed(SQUARE_SIZE))
                        .height(Length::Fixed(SQUARE_SIZE))
                        .style(theme::Container::Custom(Box::new(self.square_style(pos))))
                );
            }
        }
        final_row
    }
    
    fn square_style(&self, pos: Position) -> SquareStyle {
        if self.from_square == Some(pos) { SquareStyle::Selected } 
        else if (pos.get_row() + pos.get_col()) % 2 == 1 { SquareStyle::Light } 
        else { SquareStyle::Dark }
    }
    
    fn square_content<'a>(&self, pos: Position) -> Element<'a, Message> {
        if let Some(p) = self.board.get_piece(pos) {
            if let Some(handle) = get_image_handle(&p) {
                return container(
                    container(
                        Svg::<Theme>::new(handle.clone())
                            .width(Length::Fixed(52.0))  // Force exact size
                            .height(Length::Fixed(52.0))
                    )
                    .width(Length::Fixed(52.0))
                    .height(Length::Fixed(52.0))
                    .center_x()
                    .center_y()
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
                .center_y()
                .into();
            }
        }
        container(text("")).width(Length::Fill).height(Length::Fill).into()
    }
}

// --- Helper functions for game logic ---
impl GameUI {
    fn play_human_move(&mut self, m: Move) -> Command<Message> {
        if !self.board.is_legal_move(m, HUMAN_PLAYER) { return Command::none(); }
        if let Some(captured) = self.board.get_piece(cpu_move_target(m)) { self.add_capture(captured); }
        let game_result = self.board.play_move(m);
        self.handle_move_result(game_result)
    }

    fn handle_move_result(&mut self, result: GameResult) -> Command<Message> {
        match result {
            GameResult::Continuing(next_board) => {
                self.board = next_board;
                if self.board.get_turn_color() == AI_PLAYER {
                    let board_clone = self.board.clone();
                    return Command::perform(async move { get_move_fn()(&board_clone) }, Message::CpuMove);
                }
            }
            _ => self.handle_game_over(result),
        }
        Command::none()
    }
    
    fn handle_game_over(&mut self, result: GameResult) {
        self.game_over_message = Some(match result {
            GameResult::Victory(winner) => format!("{} wins!", winner),
            GameResult::Stalemate => "Stalemate!".to_string(),
            GameResult::IllegalMove(_) => "Illegal Move!".to_string(),
            GameResult::Continuing(_) => unreachable!(),
        });
    }

    fn add_capture(&mut self, captured: Piece) {
        match captured.get_color() {
            chess_engine::Color::White => self.captured_white.push(captured),
            chess_engine::Color::Black => self.captured_black.push(captured),
        }
    }
}

fn get_symbol_char(piece: &Piece) -> char {
    match piece {
        Piece::King(..) => '♚', Piece::Queen(..) => '♛', Piece::Rook(..) => '♜',
        Piece::Bishop(..) => '♝', Piece::Knight(..) => '♞', Piece::Pawn(..) => '♟',
    }
}

fn get_image_handle(piece: &Piece) -> Option<&'static svg::Handle> {
    let key = format!("{}{:?}", piece.get_name(), piece.get_color());
    PIECE_IMAGES.get(&key)
}

fn get_move_fn() -> fn(&Board) -> Move { *GET_CPU_MOVE.lock().unwrap() }

fn determine_move(from: Position, to: Position, board: &Board) -> Move {
    if let Some(Piece::King(..)) = board.get_piece(from) {
        if (from.get_col() - to.get_col()).abs() == 2 {
            return if to.get_col() > from.get_col() { Move::KingSideCastle } else { Move::QueenSideCastle };
        }
    }
    Move::Piece(from, to)
}

fn is_promotion_move(from: Position, to: Position, board: &Board) -> bool {
    if let Some(Piece::Pawn(color, _)) = board.get_piece(from) {
        let promotion_rank = if color == chess_engine::Color::White { 7 } else { 0 };
        return to.get_row() == promotion_rank;
    }
    false
}

fn cpu_move_target(m: Move) -> Position {
    match m {
        Move::Piece(_, to) | Move::Promotion(_, to, _) => to,
        _ => Position::new(-1, -1),
    }
}

// --- Styling ---
#[derive(Debug, Clone, Copy, PartialEq, Default)]
enum SquareStyle {
    #[default]
    Light, Dark, Selected, Promotion,
}

impl container::StyleSheet for SquareStyle {
    type Style = Theme;
    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(iced::Background::Color(match self {
                SquareStyle::Light => iced::Color::from_rgb8(240, 217, 181),
                SquareStyle::Dark => iced::Color::from_rgb8(181, 136, 99),
                _ => iced::Color::TRANSPARENT,
            })),
            ..Default::default()
        }
    }
}

impl button::StyleSheet for SquareStyle {
    type Style = Theme;
    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(iced::Background::Color(match self {
                SquareStyle::Light => iced::Color::from_rgb8(240, 217, 181),
                SquareStyle::Dark => iced::Color::from_rgb8(181, 136, 99),
                SquareStyle::Selected | SquareStyle::Promotion => iced::Color::from_rgb8(130, 151, 105),
            })),
            border: iced::Border { radius: 0.0.into(), ..Default::default() },
            text_color: iced::Color::BLACK,
            ..Default::default()
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let active = self.active(style);
        button::Appearance {
            background: Some(iced::Background::Color(iced::Color::from_rgb8(110, 130, 90))),
            ..active
        }
    }
}

impl From<SquareStyle> for theme::Button { fn from(style: SquareStyle) -> Self { theme::Button::Custom(Box::new(style)) } }
impl From<SquareStyle> for theme::Container { fn from(style: SquareStyle) -> Self { theme::Container::Custom(Box::new(style)) } }