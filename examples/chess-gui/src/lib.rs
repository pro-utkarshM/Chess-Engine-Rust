use iced::widget::{button, container, text, Button, Column, Row};
use iced::{
    executor, theme, Application, Command, Element, Length, Settings, Size, Subscription, Theme,
};
use rand::{seq::SliceRandom, thread_rng};

use chess_engine::*;
pub use chess_engine::Board;
use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref GET_CPU_MOVE: Mutex<fn(&Board) -> Move> = Mutex::new(best_move);
    static ref STARTING_BOARD: Mutex<Board> = Mutex::new(Board::default());
}

const SQUARE_SIZE: f32 = 64.0;
pub const AI_DEPTH: i32 = if cfg!(debug_assertions) { 2 } else { 4 };

pub fn run_gui(get_cpu_move: fn(&Board) -> Move, starting_board: Board) -> iced::Result {
    {
        let mut x = GET_CPU_MOVE.lock().unwrap();
        *x = get_cpu_move;
        let mut x = STARTING_BOARD.lock().unwrap();
        *x = starting_board;
    };

    GameUI::run(Settings {
        window: iced::window::Settings {
            size: Size::new((SQUARE_SIZE * 8.0) + 200.0, SQUARE_SIZE * 8.0),
            resizable: false,
            ..iced::window::Settings::default()
        },
        ..Settings::default()
    })
}

// --- Helper Functions ---
pub fn get_symbol(piece: &Piece) -> &str {
    match piece {
        Piece::King(c, _) => if *c == chess_engine::Color::White { "♔" } else { "♚" },
        Piece::Queen(c, _) => if *c == chess_engine::Color::White { "♕" } else { "♛" },
        Piece::Rook(c, _) => if *c == chess_engine::Color::White { "♖" } else { "♜" },
        Piece::Bishop(c, _) => if *c == chess_engine::Color::White { "♗" } else { "♝" },
        Piece::Knight(c, _) => if *c == chess_engine::Color::White { "♘" } else { "♞" },
        Piece::Pawn(c, _) => if *c == chess_engine::Color::White { "♙" } else { "♟" },
    }
}

pub fn best_move(board: &Board) -> Move {
    board.get_best_next_move(AI_DEPTH).0
}

pub fn worst_move(board: &Board) -> Move {
    board.get_worst_next_move(AI_DEPTH).0
}

pub fn random_move(board: &Board) -> Move {
    let moves = board.get_legal_moves();
    let mut rng = thread_rng();
    *moves.choose(&mut rng).unwrap()
}

// --- The Application State ---
pub struct GameUI {
    board: Board,
    from_square: Option<Position>,
    game_over_message: Option<String>,
    captured_white: Vec<Piece>,
    captured_black: Vec<Piece>,
}

#[derive(Debug, Clone)]
pub enum Message {
    SelectSquare(Position),
    NewGame,
    CpuMove(Move),
}

// --- Application Logic ---
impl Application for GameUI {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let starting_board = *STARTING_BOARD.lock().unwrap();
        (
            Self {
                board: starting_board,
                from_square: None,
                game_over_message: None,
                captured_white: Vec::new(),
                captured_black: Vec::new(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Rust Chess Engine")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        if self.game_over_message.is_some() {
            if let Message::NewGame = message {
                self.board = *STARTING_BOARD.lock().unwrap();
                self.from_square = None;
                self.game_over_message = None;
                self.captured_black.clear();
                self.captured_white.clear();
            }
            return Command::none();
        }

        match message {
            Message::SelectSquare(pos) => {
                match self.from_square {
                    None => {
                        if self.board.has_ally_piece(pos, self.board.get_turn_color()) {
                             self.from_square = Some(pos);
                        }
                    }
                    Some(from) => {
                        let m = determine_move(from, pos, self.board.get_turn_color(), &self.board);
                        if self.board.is_legal_move(m, self.board.get_turn_color()) {
                            if let Some(captured) = self.board.get_piece(pos) {
                                match captured.get_color() {
                                    chess_engine::Color::White => self.captured_white.push(captured),
                                    chess_engine::Color::Black => self.captured_black.push(captured),
                                }
                            }
                            self.from_square = None;
                            let game_result = self.board.play_move(m);
                            match game_result {
                                GameResult::Continuing(next_board) => {
                                    self.board = next_board;
                                    let board_clone = self.board.clone();
                                    let get_move_fn = *GET_CPU_MOVE.lock().unwrap();
                                    return Command::perform(
                                        async move { get_move_fn(&board_clone) },
                                        Message::CpuMove,
                                    );
                                }
                                _ => {
                                    self.handle_game_over(game_result);
                                }
                            }
                        } else {
                            self.from_square = if self.board.has_ally_piece(pos, self.board.get_turn_color()) { Some(pos) } else { None };
                        }
                    }
                }
            }
            Message::CpuMove(cpu_move) => {
                 if let Some(captured) = self.board.get_piece(cpu_move_target(cpu_move)) {
                    match captured.get_color() {
                        chess_engine::Color::White => self.captured_white.push(captured),
                        chess_engine::Color::Black => self.captured_black.push(captured),
                    }
                }
                let game_result = self.board.play_move(cpu_move);
                match game_result {
                    GameResult::Continuing(next_board) => {
                        self.board = next_board;
                    }
                    _ => {
                        self.handle_game_over(game_result);
                    }
                }
            }
            Message::NewGame => {
                 self.board = *STARTING_BOARD.lock().unwrap();
                 self.from_square = None;
                 self.game_over_message = None;
                 self.captured_black.clear();
                 self.captured_white.clear();
            }
        }
        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }

    fn view(&self) -> Element<Message> {
        let chessboard = (0..8).rev().fold(Column::new().spacing(0), |col, r| {
            let row = (0..8).fold(Row::new().spacing(0), |row, f| {
                let pos = Position::new(r, f);
                let piece = self.board.get_piece(pos);
                let is_light = (r + f) % 2 == 1;

                let square_style = if self.from_square == Some(pos) {
                    SquareStyle::Selected
                } else if is_light {
                    SquareStyle::Light
                } else {
                    SquareStyle::Dark
                };

                let content = if let Some(p) = piece {
                    text(get_symbol(&p)).size(48)
                } else {
                    text(" ").size(48)
                };

                row.push(
                    Button::new(content)
                        .width(Length::Fixed(SQUARE_SIZE))
                        .height(Length::Fixed(SQUARE_SIZE))
                        .style(theme::Button::Custom(Box::new(square_style)))
                        .on_press(Message::SelectSquare(pos)),
                )
            });
            col.push(row)
        });

        let captured_white_text = self.captured_white.iter().map(get_symbol).collect::<String>();
        let captured_black_text = self.captured_black.iter().map(get_symbol).collect::<String>();

        let mut info_panel = Column::new()
            .padding(10)
            .spacing(20)
            .width(Length::Fixed(200.0))
            .push(text(format!("Turn: {}", self.board.get_turn_color())).size(24))
            .push(text(format!("Captured (Black):\n{}", captured_white_text)).size(16))
            .push(text(format!("Captured (White):\n{}", captured_black_text)).size(16));
        
        if self.game_over_message.is_some() {
            info_panel = info_panel.push(Button::new(text("New Game")).on_press(Message::NewGame));
             if let Some(msg) = &self.game_over_message {
                 info_panel = info_panel.push(text(msg).size(24));
            }
        }

        let main_layout = Row::new()
            .padding(20)
            .spacing(20)
            .push(chessboard)
            .push(info_panel);

        container(main_layout)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn theme(&self) -> Self::Theme {
        Theme::Light
    }
}

fn determine_move(from: Position, to: Position, color: chess_engine::Color, board: &Board) -> Move {
    if let Some(Piece::King(..)) = board.get_piece(from) {
        if (from.get_col() - to.get_col()).abs() == 2 {
            if to.get_col() > from.get_col() {
                return Move::KingSideCastle;
            } else {
                return Move::QueenSideCastle;
            }
        }
    }
     if let Some(Piece::Pawn(..)) = board.get_piece(from) {
        if (color == chess_engine::Color::White && to.get_row() == 7) || (color == chess_engine::Color::Black && to.get_row() == 0) {
            return Move::Promotion(from, to, Piece::Queen(color, to));
        }
    }
    Move::Piece(from, to)
}

fn cpu_move_target(m: Move) -> Position {
    match m {
        Move::Piece(_, to) | Move::Promotion(_, to, _) => to,
        _ => Position::new(-1, -1),
    }
}

impl GameUI {
    fn handle_game_over(&mut self, result: GameResult) {
        self.game_over_message = Some(match result {
             GameResult::Victory(winner) => format!("{} wins!", winner),
             GameResult::Stalemate => "Stalemate!".to_string(),
             GameResult::IllegalMove(_) => "Illegal Move!".to_string(),
             GameResult::Continuing(_) => unreachable!(),
        });
    }
}

// --- Styling ---
#[derive(Debug, Clone, Copy, PartialEq)]
enum SquareStyle {
    Light,
    Dark,
    Selected,
}

impl button::StyleSheet for SquareStyle {
    type Style = Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(iced::Background::Color(match self {
                SquareStyle::Light => iced::Color::from_rgb8(240, 217, 181),
                SquareStyle::Dark => iced::Color::from_rgb8(181, 136, 99),
                SquareStyle::Selected => iced::Color::from_rgb8(130, 151, 105),
            })),
            border: iced::Border {
                radius: 0.0.into(),
                ..iced::Border::default()
            },
            text_color: iced::Color::BLACK,
            ..button::Appearance::default()
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let active = self.active(style);
        button::Appearance {
            shadow_offset: active.shadow_offset + iced::Vector::new(0.0, 1.0),
            ..active
        }
    }
}

impl From<SquareStyle> for theme::Button {
     fn from(style: SquareStyle) -> Self {
         theme::Button::Custom(Box::new(style))
     }
}