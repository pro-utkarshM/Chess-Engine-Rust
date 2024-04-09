use log;
use pixels::{Error, Pixels, SurfaceTexture};
use pretty_env_logger;
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

mod board;
mod piece;
mod cache;

use board::Board;

const WIN_WIDTH: u32 = 854;
const WIN_HEIGHT: u32 = 480;

fn main() -> Result<(), Error> {
    std::env::set_var("RUST_LOG", "chess-engine=info");
    pretty_env_logger::init();

    let event_loop = EventLoop::new();
    let builder = WindowBuilder::new();
    let window_size = LogicalSize::new(WIN_WIDTH, WIN_HEIGHT);
    let window = builder
        .with_title("chess-engine")
        .with_inner_size(window_size)
        .build(&event_loop)
        .unwrap();

    let mut board = Board::default();

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        let board_size = board.get_side_length();
        // TODO: use new_async for web
        Pixels::new(board_size, board_size, surface_texture)?
    };

    event_loop.run(move |event, _, control_flow| {
        control_flow.set_poll();

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                log::info!("The close Button was pressed.");
                control_flow.set_exit();
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                if let Err(err) = pixels.resize_surface(size.width, size.height) {
                    log::error!("Pixels failed to resize error: {}", err);
                    control_flow.set_exit();
                }
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                // redraw here
                board.draw(pixels.get_frame_mut());
                if let Err(err) = pixels.render() {
                    log::error!("pixels.render() failed: {err}");
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            }
            _ => (),
        }
    });
}
