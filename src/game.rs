use std::collections::VecDeque;

use web_sys::console;

use crate::{
    render::PlatformRenderer,
    utils::{emod, lerpf, rand},
};

// Constants
const CELL_SIZE: i32 = 100;
const COLS: i32 = 16;
const ROWS: i32 = 9;
const BACKGROUND_COLOR: u32 = 0xFF181818;
const CELL1_COLOR: u32 = BACKGROUND_COLOR;
const CELL2_COLOR: u32 = 0xFF183018;
const SNAKE_HEAD_COLOR: u32 = 0xFF00FF00;
const SNAKE_BODY_COLOR: u32 = 0xFF32CD32;
const SNAKE_TAIL_COLOR: u32 = 0xFF228B22;
const SNAKE_SPINE_COLOR: u32 = 0xFF006400;
const EGG_BODY_COLOR: u32 = 0xFF31A6FF;
const EGG_SPINE_COLOR: u32 = 0xFF3166BB;
const SNAKE_SPINE_THICKNESS_PERCENT: f32 = 0.05;
const SNAKE_INIT_SIZE: usize = 3;
const STEP_INTERVAL: f32 = 0.125;
const SCORE_PADDING: i32 = 100;
const SCORE_FONT_SIZE: u32 = 48;
const SCORE_FONT_COLOR: u32 = 0xFFFFFFFF;
const PAUSE_FONT_COLOR: u32 = SCORE_FONT_COLOR;
const PAUSE_FONT_SIZE: u32 = SCORE_FONT_SIZE;
const GAMEOVER_FONT_COLOR: u32 = SCORE_FONT_COLOR;
const GAMEOVER_FONT_SIZE: u32 = SCORE_FONT_SIZE;
const RANDOM_EGG_MAX_ATTEMPTS: u32 = 1000;
const SNAKE_CAP: usize = (ROWS * COLS) as usize;
const SNAKE_INIT_ROW: i32 = ROWS / 2;
const DIR_LENS: usize = 4;

#[derive(Clone, Copy)]
enum Direction {
    Right = 0,
    Up = 1,
    Left = 2,
    Down = 3,
}

impl Direction {
    pub const ALL: [Direction; 4] = [
        Direction::Right,
        Direction::Up,
        Direction::Left,
        Direction::Down,
    ];
}

impl Into<Cell> for Direction {
    fn into(self) -> Cell {
        match self {
            Direction::Right => Cell { x: 1, y: 0 },
            Direction::Left => Cell { x: -1, y: 0 },
            Direction::Up => Cell { x: 0, y: -1 },
            Direction::Down => Cell { x: 0, y: 1 },
        }
    }
}

impl std::ops::Not for Direction {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Direction::Right => Direction::Left,
            Direction::Up => Direction::Down,
            Direction::Left => Direction::Right,
            Direction::Down => Direction::Up,
        }
    }
}

enum State {
    Gameplay,
    Pause,
    Gameover,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
struct Vec2<I> {
    x: I,
    y: I,
}

impl<I> std::ops::Sub for Vec2<I>
where
    I: std::ops::Sub<Output = I>,
{
    type Output = Vec2<I>;

    fn sub(self, rhs: Self) -> Self::Output {
        Vec2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl<I> std::ops::Add for Vec2<I>
where
    I: std::ops::Add<Output = I>,
{
    type Output = Vec2<I>;

    fn add(self, rhs: Self) -> Self::Output {
        Vec2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

struct Rect {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

impl From<&Cell> for Rect {
    fn from(cell: &Cell) -> Self {
        Rect {
            x: (cell.x * CELL_SIZE) as f32,
            y: (cell.y * CELL_SIZE) as f32,
            w: CELL_SIZE as f32,
            h: CELL_SIZE as f32,
        }
    }
}

impl From<&Sides> for Rect {
    fn from(sides: &Sides) -> Self {
        Rect {
            x: sides.lens[Direction::Left as usize],
            y: sides.lens[Direction::Up as usize],
            w: sides.lens[Direction::Right as usize] - sides.lens[Direction::Left as usize],
            h: sides.lens[Direction::Down as usize] - sides.lens[Direction::Up as usize],
        }
    }
}

type Cell = Vec2<i32>;

impl Cell {
    fn determine_dir(&self, another: &Cell) -> Direction {
        for dir in Direction::ALL {
            if self.advance(dir) == *another {
                return dir;
            }
        }
        unreachable!()
    }

    fn advance(&self, dir: Direction) -> Cell {
        let dir_cell: Cell = dir.into();
        let mut res: Cell = dir_cell + *self;
        res.wrap_by_game_size();
        res
    }

    fn wrap_by_game_size(&mut self) {
        self.x = emod(self.x, COLS);
        self.y = emod(self.y, ROWS);
    }
}

struct Sides {
    lens: Vec<f32>,
}

impl Sides {
    fn adjust_2_slide_sides(&mut self, dir: Direction, t: f32) {
        let d = self.lens[dir as usize] - self.lens[!dir as usize];
        self.lens[dir as usize] += lerpf(0.0, d, t);
        self.lens[!dir as usize] += lerpf(0.0, d, t);
    }
}

impl From<&Rect> for Sides {
    fn from(rect: &Rect) -> Self {
        let mut res = Sides {
            lens: vec![0.0; DIR_LENS],
        };

        res.lens[Direction::Left as usize] = rect.x;
        res.lens[Direction::Right as usize] = rect.x + rect.w;

        res.lens[Direction::Up as usize] = rect.y;
        res.lens[Direction::Down as usize] = rect.y + rect.h;

        res
    }
}

struct Snake {
    body: VecDeque<Cell>,
}

impl Snake {
    fn contains_cell(&self, cell: &Cell) -> bool {
        self.body.contains(cell)
    }

    fn size(&self) -> usize {
        self.body.len()
    }
}

pub struct Game<P: PlatformRenderer> {
    width: u32,
    height: u32,

    dir: Direction,
    state: State,
    score: u32,
    step_cooldown: f32,

    snake: Snake,
    egg: Cell,

    camera_pos: Vec2<f32>,

    platform_renderer: P,

    eating_egg: bool,

    #[cfg(feature = "dev")]
    dt_scale: f32,
}

impl<P: PlatformRenderer> Game<P> {
    pub fn new(platform_renderer: P) -> Self {
        Self {
            width: 0,
            height: 0,
            dir: Direction::Right,
            state: State::Gameplay,
            score: 0,
            snake: Snake {
                body: VecDeque::with_capacity(SNAKE_CAP),
            },
            camera_pos: Vec2::default(),
            platform_renderer,
            eating_egg: false,
            egg: Cell::default(),
            step_cooldown: 0.0,
            #[cfg(feature = "dev")]
            dt_scale: 0.0,
        }
    }

    pub fn restart(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;

        #[cfg(feature = "dev")]
        {
            self.dt_scale = 1.0;
        }

        self.camera_pos.x = width as f32 / 2.0;
        self.camera_pos.y = height as f32 / 2.0;

        self.state = State::Gameplay;
        self.dir = Direction::Right;
        self.score = 0;

        for i in 0..SNAKE_INIT_SIZE {
            let head = Cell {
                x: i as i32,
                y: SNAKE_INIT_ROW,
            };
            self.snake.body.push_back(head);
        }

        self.random_egg(true);
    }

    pub fn update(&mut self, dt: f32) {
        let mut dt = dt;
        #[cfg(feature = "dev")]
        {
            dt *= self.dt_scale;
        }

        match self.state {
            State::Gameplay => {
                self.step_cooldown -= dt;
                if self.step_cooldown > 0.0 {
                    return;
                }

                let next_head = self.snake.body.back().unwrap().advance(self.dir);
                self.snake.body.push_back(next_head);
                self.snake.body.pop_front();
                self.eating_egg = false;

                self.step_cooldown = STEP_INTERVAL;
            }
            State::Pause => {}
            State::Gameover => {}
        }
    }

    pub fn render(&self) {
        match self.state {
            State::Gameplay => {
                self.background_render();
                self.egg_render();
                self.snake_render();
            }
            State::Pause => {}
            State::Gameover => {}
        }
    }

    fn snake_render(&self) {
        let t = self.step_cooldown / STEP_INTERVAL;

        let head_cell = self.snake.body.back().unwrap();
        let head_dir = self.dir;
        let mut head_slide_sides: Sides = (&Rect::from(head_cell)).into();
        head_slide_sides.adjust_2_slide_sides(!head_dir, t);

        let tail_cell = self.snake.body.front().unwrap();
        let mut tail_slide_sides: Sides = (&Rect::from(tail_cell)).into();
        let tail_dir = self
            .snake
            .body
            .get(0)
            .unwrap()
            .determine_dir(self.snake.body.get(1).unwrap());
        tail_slide_sides
            .adjust_2_slide_sides(tail_dir, if self.eating_egg { 1.0 } else { 1.0 - t });

        if self.eating_egg {
            self.fill_cell(head_cell, EGG_BODY_COLOR, 1.0);
            self.fill_cell(
                head_cell,
                EGG_SPINE_COLOR,
                SNAKE_SPINE_THICKNESS_PERCENT * 2.0,
            );
        }

        // self.fill_sides(&head_slide_sides, SNAKE_HEAD_COLOR);
        // self.fill_sides(&tail_slide_sides, SNAKE_TAIL_COLOR);
        self.fill_sides(&head_slide_sides, SNAKE_BODY_COLOR);
        self.fill_sides(&tail_slide_sides, SNAKE_BODY_COLOR);

        for i in 1..self.snake.size() - 1 {
            self.fill_cell(self.snake.body.get(i).unwrap(), SNAKE_BODY_COLOR, 1.0);
        }

        #[cfg(feature = "dev")]
        {
            for i in 0..self.snake.size() {
                console::log_1(&"test".into());
                self.stroke_rect(self.snake.body.get(i).unwrap().into(), 0xFF0000FF);
            }
        }
    }

    fn stroke_rect(&self, rect: Rect, color: u32) {
        self.platform_renderer.stroke_rect(
            (rect.x - self.camera_pos.x) as i32 + (self.width / 2) as i32,
            (rect.y - self.camera_pos.y) as i32 + (self.height / 2) as i32,
            rect.w as i32,
            rect.h as i32,
            color,
        );
    }

    fn fill_sides(&self, sides: &Sides, color: u32) {
        self.fill_rect(sides.into(), color);
    }

    fn random_egg(&mut self, first: bool) {
        let (col1, col2, row1, row2) = (0, COLS - 1, 0, ROWS - 1);
        let mut attempt = 0;
        loop {
            self.egg.x = (rand() % (col2 - col1 + 1) as u32) as i32 + col1;
            self.egg.y = (rand() % (row2 - row1 + 1) as u32) as i32 + row1;
            attempt += 1;

            if !(self.snake.contains_cell(&self.egg) || (first && self.egg.y == SNAKE_INIT_ROW))
                || attempt >= RANDOM_EGG_MAX_ATTEMPTS
            {
                break;
            }
        }
        if attempt >= RANDOM_EGG_MAX_ATTEMPTS {
            panic!("Max egg placement attempts reached");
        }
    }

    fn egg_render(&self) {
        if self.eating_egg {
        } else {
            self.fill_cell(&self.egg, EGG_BODY_COLOR, 1.0);
            self.fill_cell(
                &self.egg,
                EGG_SPINE_COLOR,
                SNAKE_SPINE_THICKNESS_PERCENT * 2.0,
            );
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
                self.fill_cell(&cell, color, 1.0);
            }
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

    fn fill_cell(&self, cell: &Cell, color: u32, a: f32) {
        self.fill_rect(self.scale_rect(cell.into(), a), color);
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
