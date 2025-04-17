use std::collections::VecDeque;

use crate::{
    render::PlatformRenderer,
    utils::{ilerpf, lerpf},
};

// Constants
const CELL_SIZE: i32 = 100;
const COLS: i32 = 16;
const ROWS: i32 = 9;
const BACKGROUND_COLOR: u32 = 0xFF181818;
const CELL1_COLOR: u32 = BACKGROUND_COLOR;
const CELL2_COLOR: u32 = 0xFF183018;
const SNAKE_BODY_COLOR: u32 = 0xFF189018;
const SNAKE_SPINE_COLOR: u32 = 0xFF185018;
const EGG_BODY_COLOR: u32 = 0xFF31A6FF;
const EGG_SPINE_COLOR: u32 = 0xFF3166BB;
const SNAKE_SPINE_THICKNESS_PERCENT: f32 = 0.05;
const SNAKE_INIT_SIZE: usize = 3;
const STEP_INTERVAL: f32 = 0.125;
const KEY_LEFT: char = 'a';
const KEY_RIGHT: char = 'd';
const KEY_UP: char = 'w';
const KEY_DOWN: char = 's';
const KEY_ACCEPT: char = ' ';
const KEY_RESTART: char = 'r';
const RAND_A: u64 = 6364136223846793005;
const RAND_C: u64 = 1442695040888963407;
const SCORE_PADDING: i32 = 100;
const SCORE_FONT_SIZE: u32 = 48;
const SCORE_FONT_COLOR: u32 = 0xFFFFFFFF;
const PAUSE_FONT_COLOR: u32 = SCORE_FONT_COLOR;
const PAUSE_FONT_SIZE: u32 = SCORE_FONT_SIZE;
const GAMEOVER_FONT_COLOR: u32 = SCORE_FONT_COLOR;
const GAMEOVER_FONT_SIZE: u32 = SCORE_FONT_SIZE;
const RANDOM_EGG_MAX_ATTEMPTS: u32 = 1000;
const SNAKE_CAP: usize = (ROWS * COLS) as usize;
const DIR_QUEUE_CAP: usize = 3;
const SNAKE_INIT_ROW: i32 = ROWS / 2;

enum Dir {
    Right,
    Up,
    Left,
    Down,
}

enum State {
    Gameplay,
    Pause,
    Gameover,
}

#[derive(Default)]
struct Vec2<I> {
    x: I,
    y: I,
}

struct Rect {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

type Cell = Vec2<i32>;

struct Snake {
    body: VecDeque<Cell>,
    begin: u32,
}

pub struct Game<P: PlatformRenderer> {
    width: u32,
    height: u32,

    dir: Dir,
    state: State,
    score: u32,

    snake: Snake,

    camera_pos: Vec2<f32>,

    platform_renderer: P,
}

impl<P: PlatformRenderer> Game<P> {
    pub fn new(platform_renderer: P) -> Self {
        Self {
            width: 0,
            height: 0,
            dir: Dir::Right,
            state: State::Gameplay,
            score: 0,
            snake: Snake {
                body: VecDeque::with_capacity(SNAKE_CAP),
                begin: 0,
            },
            camera_pos: Vec2::default(),
            platform_renderer,
        }
    }

    pub fn restart(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;

        self.camera_pos.x = width as f32 / 2.0;
        self.camera_pos.y = height as f32 / 2.0;

        self.state = State::Gameplay;
        self.dir = Dir::Right;
        self.score = 0;
    }

    pub fn update(&mut self, dt: f64) {}

    pub fn render(&self) {
        match self.state {
            State::Gameplay => {
                self.background_render();
            }
            State::Pause => {}
            State::Gameover => {}
        }
    }

    fn background_render(&self) {
        let col1 = ((self.camera_pos.x - self.width as f32 * 0.5) as i32 - CELL_SIZE) / CELL_SIZE;
        let col2 = ((self.camera_pos.x + self.width as f32 * 0.5) as i32 + CELL_SIZE) / CELL_SIZE;

        let row1 = ((self.camera_pos.y - self.height as f32 * 0.5) as i32 - CELL_SIZE) / CELL_SIZE;
        let row2 = ((self.camera_pos.y + self.height as f32 * 0.5) as i32 + CELL_SIZE) / CELL_SIZE;

        for col in col1..=col2 {
            for row in row1..=row2 {
                let color = if (row + col) % 2 == 0 {
                    CELL1_COLOR
                } else {
                    CELL2_COLOR
                };
                let cell = Cell { x: col, y: row };
                self.fill_cell(cell, color, 1.0);
            }
        }
    }

    fn cell_2_rect(&self, cell: Cell) -> Rect {
        Rect {
            x: (cell.x * CELL_SIZE) as f32,
            y: (cell.y * CELL_SIZE) as f32,
            w: CELL_SIZE as f32,
            h: CELL_SIZE as f32,
        }
    }

    fn scale_rect(&self, r: Rect, a: f32) -> Rect {
        let mut r = r;
        r.x = lerpf(r.x, r.x + r.w * 0.5, 1.0 - a);
        r.y = lerpf(r.y, r.y + r.h * 0.5, 1.0 - a);
        r.w = lerpf(0.0, r.w, a);
        r.h = lerpf(0.0, r.h, a);
        return r;
    }

    fn fill_cell(&self, cell: Cell, color: u32, a: f32) {
        self.fill_rect(self.scale_rect(self.cell_2_rect(cell), a), color);
    }

    fn fill_rect(&self, rect: Rect, color: u32) {
        self.platform_renderer.fill_rect(
            (rect.x - self.camera_pos.x + self.width as f32 / 2.0) as i32,
            (rect.y - self.camera_pos.y + self.height as f32 / 2.0) as i32,
            rect.w as i32,
            rect.h as i32,
            color,
        );
    }
}
