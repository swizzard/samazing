use super::Outcome;
use crate::{
    Direction,
    maze::Maze,
    movement::MazeEvent,
    ui::{self, RoomView, UnseenRoomView},
};
use color_eyre::Result;
use crossterm::event;
use multid::{BoundedIx2, iterators::V2Indices};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::Rect,
    widgets::{StatefulWidget, Widget, canvas::Canvas},
};
use std::{collections::BTreeSet, marker::PhantomData};

pub struct HiddenGame<'a, const N_ROWS: usize, const N_COLS: usize> {
    _marker: PhantomData<&'a mut Maze<N_ROWS, N_COLS>>,
}

impl<'a, const N_ROWS: usize, const N_COLS: usize> HiddenGame<'a, N_ROWS, N_COLS> {
    fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

pub struct HiddenGameState<'a, const N_ROWS: usize, const N_COLS: usize> {
    maze: &'a mut Maze<N_ROWS, N_COLS>,
    seen: BTreeSet<BoundedIx2<N_ROWS, N_COLS>>,
}

impl<'a, const N_ROWS: usize, const N_COLS: usize> HiddenGameState<'a, N_ROWS, N_COLS> {
    fn move_north(&mut self) {
        self.maze.move_north();
    }
    fn move_east(&mut self) {
        self.maze.move_east();
    }
    fn move_south(&mut self) {
        self.maze.move_south();
    }
    fn move_west(&mut self) {
        self.maze.move_west();
    }
    fn insert_current_ix(&mut self) {
        self.seen.insert(self.maze.current_ix);
    }
    fn is_done(&self) -> bool {
        self.maze.is_done()
    }
    fn is_seen(&self, ix: &BoundedIx2<N_ROWS, N_COLS>) -> bool {
        self.seen.contains(ix)
    }
}

impl<'a, const N_ROWS: usize, const N_COLS: usize> StatefulWidget
    for HiddenGame<'a, N_ROWS, N_COLS>
{
    type State = HiddenGameState<'a, N_ROWS, N_COLS>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let c = Canvas::default()
            .x_bounds([ui::MIN_X, ui::MAX_X])
            .y_bounds([ui::MIN_Y, ui::MAX_Y])
            .background_color(ui::BG_COLOR)
            .paint(move |ctx| {
                for ix in V2Indices::<N_ROWS, N_COLS>::new() {
                    let x = -200.0 + ui::ROOM_SIZE * ix.x() as f64;
                    let y = 200.0 - ui::ROOM_SIZE * ix.y() as f64;
                    let label_x = -200.0 + (ui::ROOM_SIZE * ix.x() as f64) + ui::SEG_LEN * 3.5;
                    let label_y = 200.0 - (ui::ROOM_SIZE * ix.y() as f64 + ui::SEG_LEN * 3.5);
                    if ix == state.maze.goal {
                        ctx.print(label_x, label_y, "\u{1f945}")
                    };
                    if state.is_seen(&ix) {
                        let room = &state.maze.rooms[ix];
                        let view = RoomView { x, y, room };
                        ctx.draw(&view);
                        if ix == state.maze.current_ix && ix == state.maze.goal {
                            ctx.print(label_x, label_y, "\u{1f940}")
                        } else if ix == state.maze.current_ix {
                            ctx.print(label_x, label_y, "\u{1f600}")
                        } else if ix == state.maze.goal {
                            ctx.print(label_x, label_y, "\u{1f945}")
                        }
                    } else {
                        let mut unseen: Vec<Direction> = Vec::with_capacity(4);
                        if ix.north().map(|i| !state.seen.contains(&i)).unwrap_or(true) {
                            unseen.push(Direction::North);
                        }
                        if ix.south().map(|i| !state.seen.contains(&i)).unwrap_or(true) {
                            unseen.push(Direction::South);
                        }
                        if ix.east().map(|i| !state.seen.contains(&i)).unwrap_or(true) {
                            unseen.push(Direction::East);
                        }
                        if ix.west().map(|i| !state.seen.contains(&i)).unwrap_or(true) {
                            unseen.push(Direction::West);
                        }
                        ctx.draw(&UnseenRoomView {
                            x,
                            y,
                            hidden_walls: unseen,
                        });
                    }
                }
            });
        Widget::render(c, area, buf);
    }
}

pub fn game<const N_ROWS: usize, const N_COLS: usize>(
    terminal: &mut DefaultTerminal,
    maze: &mut Maze<N_ROWS, N_COLS>,
) -> Result<Outcome> {
    let mut st: HiddenGameState<N_ROWS, N_COLS> = HiddenGameState {
        maze,
        seen: BTreeSet::new(),
    };
    loop {
        st.insert_current_ix();
        terminal.draw(|frame: &mut Frame| {
            frame.render_stateful_widget(HiddenGame::new(), frame.area(), &mut st)
        })?;
        if st.is_done() {
            return Ok(Outcome::Win);
        }
        match event::read()?.into() {
            MazeEvent::MoveN => &st.move_north(),
            MazeEvent::MoveS => &st.move_south(),
            MazeEvent::MoveE => &st.move_east(),
            MazeEvent::MoveW => &st.move_west(),
            MazeEvent::Quit => return Ok(Outcome::Quit),
            _ => &(),
        };
    }
}
