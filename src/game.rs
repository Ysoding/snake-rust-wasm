use std::collections::VecDeque;

use web_sys::console;

use crate::{
    render::PlatformRenderer,
    utils::{emod, lerpf, rand, ring_displace_back},
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
const DIR_QUEUE_CAP: usize = 3 as usize;
const SNAKE_INIT_ROW: i32 = ROWS / 2;
const DIR_LENS: usize = 4;
const KEY_LEFT: &str = "a";
const KEY_RIGHT: &str = "d";
const KEY_UP: &str = "w";
const KEY_DOWN: &str = "s";
const KEY_ACCEPT: &str = " ";
const KEY_RESTART: &str = "r";
const GAMEOVER_EXPLOSION_RADIUS: f32 = 1000.0;
const GAMEOVER_EXPLOSION_MAX_VEL: f32 = 200.0;

struct DeadSnake {
    items: Vec<Rect>,
    vels: Vec<Vec2<f32>>,
    masks: Vec<u8>,
}

impl DeadSnake {
    fn reset(&mut self) {
        self.items.clear();
        self.vels.clear();
        self.masks.clear();
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Direction {
    Right = 0,
    Up = 1,
    Left = 2,
    Down = 3,
}

impl Direction {
    const ALL: [Direction; 4] = [
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
    GamePlay,
    Pause,
    GameOver,
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

    fn center(&self) -> Vec2<f32> {
        Vec2 {
            x: self.x as f32 * CELL_SIZE as f32 + CELL_SIZE as f32 / 2.0,
            y: self.y as f32 * CELL_SIZE as f32 + CELL_SIZE as f32 / 2.0,
        }
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

    fn center(&self) -> Vec2<f32> {
        Vec2 {
            x: self.lens[Direction::Left as usize]
                + (self.lens[Direction::Right as usize] - self.lens[Direction::Left as usize])
                    * 0.5,
            y: self.lens[Direction::Up as usize]
                + (self.lens[Direction::Down as usize] - self.lens[Direction::Up as usize]) * 0.5,
        }
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
    items: VecDeque<Cell>,
}

impl Snake {
    fn contains_cell(&self, cell: &Cell) -> bool {
        self.items.contains(cell)
    }

    fn size(&self) -> usize {
        self.items.len()
    }
}

pub struct Game<P: PlatformRenderer> {
    width: u32,
    height: u32,

    dir: Direction,
    next_dirs: VecDeque<Direction>,

    state: State,
    score: u32,
    step_cooldown: f32,
    eating_egg: bool,
    camera_pos: Vec2<f32>,
    eating_timer: f32,

    body_start_color: u32,
    body_end_color: u32,

    snake: Snake,
    dead_snake: DeadSnake,
    egg: Cell,

    platform_renderer: P,

    #[cfg(feature = "dev")]
    dt_scale: f32,
}

impl<P: PlatformRenderer> Game<P> {
    pub fn new(platform_renderer: P) -> Self {
        Self {
            eating_timer: 0.0,
            width: 0,
            height: 0,
            dir: Direction::Right,
            state: State::GamePlay,
            score: 0,
            snake: Snake {
                items: VecDeque::with_capacity(SNAKE_CAP),
            },
            camera_pos: Vec2::default(),
            platform_renderer,
            eating_egg: false,
            egg: Cell::default(),
            step_cooldown: 0.0,
            #[cfg(feature = "dev")]
            dt_scale: 0.0,
            next_dirs: VecDeque::with_capacity(DIR_QUEUE_CAP),
            dead_snake: DeadSnake {
                items: Vec::new(),
                vels: Vec::new(),
                masks: Vec::new(),
            },
            body_start_color: 0xFF00FF00,
            body_end_color: 0xFF0000FF,
        }
    }

    fn reset(&mut self) {
        let platform_renderer = self.platform_renderer.clone();
        *self = Self::new(platform_renderer);
    }

    pub fn keydown(&mut self, key: &str) {
        #[cfg(feature = "dev")]
        {
            const DEV_DT_SCALE_STEP: f32 = 0.05;
            match key {
                "z" => {
                    self.dt_scale -= DEV_DT_SCALE_STEP;
                    if self.dt_scale < 0.0 {
                        self.dt_scale = 0.0;
                    }
                    console::log_1(&format!("dt scale = {}", self.dt_scale).into());
                }
                "x" => {
                    self.dt_scale += DEV_DT_SCALE_STEP;
                    console::log_1(&format!("dt scale = {}", self.dt_scale).into());
                }
                "c" => {
                    self.dt_scale = 1.0;
                    console::log_1(&format!("dt scale = {}", self.dt_scale).into());
                }
                _ => {}
            }
        }

        match self.state {
            State::GamePlay => match key {
                KEY_UP => {
                    ring_displace_back(&mut self.next_dirs, Direction::Up, DIR_QUEUE_CAP);
                }
                KEY_DOWN => {
                    ring_displace_back(&mut self.next_dirs, Direction::Down, DIR_QUEUE_CAP);
                }
                KEY_LEFT => {
                    ring_displace_back(&mut self.next_dirs, Direction::Left, DIR_QUEUE_CAP);
                }
                KEY_RIGHT => {
                    ring_displace_back(&mut self.next_dirs, Direction::Right, DIR_QUEUE_CAP);
                }
                KEY_ACCEPT => {
                    self.state = State::Pause;
                }
                KEY_RESTART => {
                    self.restart(self.width, self.height);
                }
                _ => {}
            },
            State::Pause => match key {
                KEY_ACCEPT => {
                    self.state = State::GamePlay;
                }
                KEY_RESTART => {
                    self.restart(self.width, self.height);
                }
                _ => {}
            },
            State::GameOver => {
                self.restart(self.width, self.height);
            }
        }
    }

    pub fn restart(&mut self, width: u32, height: u32) {
        self.reset();
        self.width = width;
        self.height = height;

        #[cfg(feature = "dev")]
        {
            self.dt_scale = 1.0;
        }

        self.camera_pos.x = width as f32 / 2.0;
        self.camera_pos.y = height as f32 / 2.0;

        self.state = State::GamePlay;
        self.dir = Direction::Right;
        self.score = 0;

        for i in 0..SNAKE_INIT_SIZE {
            let head = Cell {
                x: i as i32,
                y: SNAKE_INIT_ROW,
            };
            self.snake.items.push_back(head);
        }
        self.dead_snake.reset();

        self.random_egg(true);
    }

    pub fn update(&mut self, dt: f32) {
        let mut dt = dt;
        #[cfg(feature = "dev")]
        {
            dt *= self.dt_scale;
        }

        if self.eating_egg {
            self.eating_timer += dt;
            if self.eating_timer > 1.0 {
                self.eating_egg = false;
                self.eating_timer = 0.0;
            }
        }

        match self.state {
            State::GamePlay => {
                self.step_cooldown -= dt;
                if self.step_cooldown > 0.0 {
                    return;
                }

                if !self.next_dirs.is_empty() {
                    if !self.dir != *self.next_dirs.front().unwrap() {
                        self.dir = *self.next_dirs.front().unwrap();
                    }
                    self.next_dirs.pop_front();
                }

                let next_head = self.snake.items.back().unwrap().advance(self.dir);

                if next_head == self.egg {
                    self.snake.items.push_back(next_head);
                    self.random_egg(false);
                    self.eating_egg = true;
                    self.score += 1;
                } else {
                    if self.snake.contains_cell(&next_head) {
                        self.step_cooldown = 0.0;
                        self.state = State::GameOver;
                        self.init_dead_snake(&next_head);
                        return;
                    } else {
                        self.snake.items.push_back(next_head);
                        self.snake.items.pop_front();
                        self.eating_egg = false;
                    }
                }

                self.step_cooldown = STEP_INTERVAL;
            }
            State::Pause => {}
            State::GameOver => {
                for i in 1..self.dead_snake.items.len() {
                    self.dead_snake.vels[i].x *= 0.99;
                    self.dead_snake.vels[i].y *= 0.99;
                    self.dead_snake.items[i].x += self.dead_snake.vels[i].x * dt;
                    self.dead_snake.items[i].y += self.dead_snake.vels[i].y * dt;
                }
            }
        }
    }

    fn score_text(&self) -> String {
        format!("Score: {}", self.score)
    }

    pub fn render(&self) {
        match self.state {
            State::GamePlay => {
                self.background_render();
                self.egg_render();
                self.snake_render();
                self.fill_text(
                    SCORE_PADDING,
                    SCORE_PADDING,
                    &self.score_text(),
                    SCORE_FONT_SIZE,
                    SCORE_FONT_COLOR,
                );
            }
            State::Pause => {
                self.background_render();
                self.egg_render();
                self.snake_render();
                self.fill_text(
                    SCORE_PADDING,
                    SCORE_PADDING,
                    &self.score_text(),
                    SCORE_FONT_SIZE,
                    SCORE_FONT_COLOR,
                );
                self.fill_text(
                    self.camera_pos.x as i32,
                    self.camera_pos.y as i32,
                    "Pause",
                    PAUSE_FONT_SIZE,
                    PAUSE_FONT_COLOR,
                );
            }
            State::GameOver => {
                self.background_render();
                self.egg_render();
                self.dead_snake_render();
                self.fill_text(
                    SCORE_PADDING,
                    SCORE_PADDING,
                    &self.score_text(),
                    SCORE_FONT_SIZE,
                    SCORE_FONT_COLOR,
                );
                self.fill_text(
                    self.camera_pos.x as i32,
                    self.camera_pos.y as i32,
                    "Game Over",
                    GAMEOVER_FONT_SIZE,
                    GAMEOVER_FONT_COLOR,
                );
            }
        }

        #[cfg(feature = "dev")]
        {
            self.fill_text(
                self.width as i32 - SCORE_PADDING * 5,
                SCORE_PADDING,
                "Dev",
                SCORE_FONT_SIZE,
                SCORE_FONT_COLOR,
            );
            self.stroke_rect(
                Rect {
                    x: 0.0,
                    y: 0.0,
                    w: (COLS * CELL_SIZE) as f32,
                    h: (ROWS * CELL_SIZE) as f32,
                },
                0xFF0000FF,
            );
        }
    }

    fn dead_snake_render(&self) {
        for i in 1..self.dead_snake.items.len() {
            self.fill_rect(self.dead_snake.items.get(i).unwrap(), SNAKE_BODY_COLOR);
            self.fill_fractured_spine(
                self.dead_snake.items.get(i).unwrap().into(),
                *self.dead_snake.masks.get(i).unwrap(),
            );
        }
    }

    fn lerp_color(&self, color1: u32, color2: u32, t: f32) -> u32 {
        let r1 = (color1 >> 16) & 0xFF;
        let g1 = (color1 >> 8) & 0xFF;
        let b1 = color1 & 0xFF;
        let r2 = (color2 >> 16) & 0xFF;
        let g2 = (color2 >> 8) & 0xFF;
        let b2 = color2 & 0xFF;
        let r = (r1 as f32 + (r2 as f32 - r1 as f32) * t).round() as u32;
        let g = (g1 as f32 + (g2 as f32 - g1 as f32) * t).round() as u32;
        let b = (b1 as f32 + (b2 as f32 - b1 as f32) * t).round() as u32;
        0xFF000000 | (r << 16) | (g << 8) | b
    }

    fn snake_render(&self) {
        let t = self.step_cooldown / STEP_INTERVAL;

        let head_cell = self.snake.items.back().unwrap();
        let head_dir = self.dir;
        let mut head_slide_sides: Sides = (&Rect::from(head_cell)).into();
        head_slide_sides.adjust_2_slide_sides(!head_dir, t);

        let tail_cell = self.snake.items.front().unwrap();
        let mut tail_slide_sides: Sides = (&Rect::from(tail_cell)).into();
        let tail_dir = self
            .snake
            .items
            .get(0)
            .unwrap()
            .determine_dir(self.snake.items.get(1).unwrap());
        tail_slide_sides
            .adjust_2_slide_sides(tail_dir, if self.eating_egg { 1.0 } else { 1.0 - t });

        if self.eating_egg {
            // self.fill_cell(head_cell, EGG_BODY_COLOR, 1.0);
            // self.fill_cell(
            //     head_cell,
            //     EGG_SPINE_COLOR,
            //     SNAKE_SPINE_THICKNESS_PERCENT * 2.0,
            // );
            let t = self.eating_timer;
            let color = self.lerp_color(EGG_BODY_COLOR, SNAKE_HEAD_COLOR, t.sin()); // 动态颜色
            self.fill_cell(head_cell, color, 1.0);
        } else {
            self.fill_sides(&head_slide_sides, SNAKE_HEAD_COLOR);
        }

        self.fill_sides(&tail_slide_sides, SNAKE_TAIL_COLOR);

        for i in 1..self.snake.size() - 1 {
            let t = (i - 1) as f32 / (self.snake.size() - 2) as f32;
            let color = self.lerp_color(self.body_start_color, self.body_end_color, t);
            self.fill_cell(self.snake.items.get(i).unwrap(), color, 1.0);
        }

        // body spine
        for i in 1..self.snake.size() - 2 {
            let cell1 = self.snake.items.get(i).unwrap();
            let cell2 = self.snake.items.get(i + 1).unwrap();

            self.fill_spine(cell1.center(), cell1.determine_dir(cell2), CELL_SIZE as f32);
            self.fill_spine(cell2.center(), cell2.determine_dir(cell1), CELL_SIZE as f32);
        }

        // head spine
        {
            let cell1 = self.snake.items.get(self.snake.size() - 2).unwrap();
            let cell2 = self.snake.items.get(self.snake.size() - 1).unwrap();
            let len = lerpf(0.0, CELL_SIZE as f32, 1.0 - t);
            self.fill_spine(cell1.center(), cell1.determine_dir(cell2), len);
            self.fill_spine((*cell2 + (!head_dir).into()).center(), head_dir, len);
        }

        // tail spine
        {
            let cell1 = self.snake.items.get(1).unwrap();
            let cell2 = self.snake.items.get(0).unwrap();
            let len = lerpf(0.0, CELL_SIZE as f32, if self.eating_egg { 0.0 } else { t });
            self.fill_spine(cell1.center(), cell1.determine_dir(cell2), len);
            self.fill_spine((*cell2 + tail_dir.into()).center(), !tail_dir, len);
        }

        #[cfg(feature = "dev")]
        {
            for i in 0..self.snake.size() {
                self.stroke_rect(self.snake.items.get(i).unwrap().into(), 0xFF0000FF);
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
        self.fill_rect(&sides.into(), color);
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
            let t = 1.0 - self.step_cooldown / STEP_INTERVAL;
            let a = lerpf(1.5, 1.0, t * t);
            self.fill_cell(&self.egg, self.color_alpha(EGG_BODY_COLOR, t * t), a);
            self.fill_cell(
                &self.egg,
                self.color_alpha(EGG_SPINE_COLOR, t * t),
                a * (SNAKE_SPINE_THICKNESS_PERCENT * 2.0),
            );
        } else {
            self.fill_cell(&self.egg, EGG_BODY_COLOR, 1.0);
            self.fill_cell(
                &self.egg,
                EGG_SPINE_COLOR,
                SNAKE_SPINE_THICKNESS_PERCENT * 2.0,
            );
        }
    }

    fn color_alpha(&self, color: u32, a: f32) -> u32 {
        let rgb = color & 0x00FF_FFFF;
        let alpha = ((a.clamp(0.0, 1.0) * 255.0).round() as u32) << 24;
        rgb | alpha
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
        self.fill_rect(&self.scale_rect(cell.into(), a), color);
    }

    fn fill_rect(&self, rect: &Rect, color: u32) {
        self.platform_renderer.fill_rect(
            (rect.x - self.camera_pos.x + self.width as f32 / 2.0) as i32,
            (rect.y - self.camera_pos.y + self.height as f32 / 2.0) as i32,
            rect.w as i32,
            rect.h as i32,
            color,
        );
    }

    fn fill_text(&self, x: i32, y: i32, text: &str, size: u32, color: u32) {
        self.platform_renderer.fill_text(x, y, text, size, color);
    }

    fn fill_spine(&self, center: Vec2<f32>, dir: Direction, len: f32) {
        let thicc = CELL_SIZE as f32 * SNAKE_SPINE_THICKNESS_PERCENT;
        let mut sides = Sides {
            lens: vec![0.0; DIR_LENS],
        };
        sides.lens[Direction::Left as usize] = center.x - thicc;
        sides.lens[Direction::Right as usize] = center.x + thicc;
        sides.lens[Direction::Up as usize] = center.y - thicc;
        sides.lens[Direction::Down as usize] = center.y + thicc;

        if dir == Direction::Right || dir == Direction::Down {
            sides.lens[dir as usize] += len;
        }
        if dir == Direction::Left || dir == Direction::Up {
            sides.lens[dir as usize] -= len;
        }
        self.fill_sides(&sides, SNAKE_SPINE_COLOR);
    }

    fn fill_fractured_spine(&self, sides: Sides, mask: u8) {
        let thicc = CELL_SIZE as f32 * SNAKE_SPINE_THICKNESS_PERCENT;
        let center = sides.center();
        for dir in Direction::ALL {
            if (mask & (1 << dir as u8)) != 0 {
                let mut arm = Sides {
                    lens: vec![0.0; DIR_LENS],
                };
                arm.lens[Direction::Left as usize] = center.x - thicc;
                arm.lens[Direction::Right as usize] = center.x + thicc;
                arm.lens[Direction::Up as usize] = center.y - thicc;
                arm.lens[Direction::Down as usize] = center.y + thicc;
                arm.lens[dir as usize] = sides.lens[dir as usize];
                self.fill_sides(&arm, SNAKE_SPINE_COLOR);
            }
        }
    }

    fn init_dead_snake(&mut self, next_head: &Cell) {
        let head_center = next_head.center();
        self.dead_snake.reset();

        for (i, cell) in self.snake.items.iter().enumerate() {
            self.dead_snake.items.push(cell.into());

            if *cell != *next_head {
                let cell_center = cell.center();
                let vel_vec = cell_center - head_center;
                let vel_len = (vel_vec.x.powi(2) + vel_vec.y.powi(2)).sqrt();
                let t = (vel_len / GAMEOVER_EXPLOSION_RADIUS).clamp(0.0, 1.0);
                let t = 1.0 - t;
                let noise_x = (rand() % 1000) as f32 * 0.01;
                let noise_y = (rand() % 1000) as f32 * 0.01;
                let vel_x = (vel_vec.x / vel_len * GAMEOVER_EXPLOSION_MAX_VEL * t) + noise_x;
                let vel_y = (vel_vec.y / vel_len * GAMEOVER_EXPLOSION_MAX_VEL * t) + noise_y;
                self.dead_snake.vels.push(Vec2 { x: vel_x, y: vel_y });
            } else {
                self.dead_snake.vels.push(Vec2 { x: 0.0, y: 0.0 }); // 头部不动
            }

            let mut mask = 0;
            if i > 0 {
                let prev_cell = self.snake.items[i - 1];
                let dir = cell.determine_dir(&prev_cell);
                mask |= 1 << dir as u8;
            }
            if i < self.snake.items.len() - 1 {
                let next_cell = self.snake.items[i + 1];
                let dir = cell.determine_dir(&next_cell);
                mask |= 1 << dir as u8;
            }
            self.dead_snake.masks.push(mask);
        }
    }
}
