#![warn(clippy::pedantic)]
use pyo3::prelude::*;

use bracket_lib::prelude::*;

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const FRAME_DURATION: f32 = 25.0;

struct Player {
    pos: Point,
}

impl Player {
    fn new(point: Point) -> Self {
        Player { pos: point }
    }

    fn move_(&mut self, point: Point) {
        self.pos = point;
    }

    fn render(&self, ctx: &mut BTerm) {
        ctx.set(self.pos.x, self.pos.y, YELLOW, BLACK, to_cp437('@'));
    }
}

struct Food {
    pos: Point,
}

impl Food {
    fn new(point: Point) -> Self {
        Food { pos: point }
    }

    fn render(&self, ctx: &mut BTerm) {
        ctx.set(self.pos.x, self.pos.y, RED, BLACK, to_cp437('*'));
    }

    fn respawn() -> Food {
        let mut random = RandomNumberGenerator::new();
        Food::new(Point::new(
            random.range(0, SCREEN_WIDTH),
            random.range(0, SCREEN_HEIGHT),
        ))
    }
}

enum GameMode {
    Menu,
    Playing,
    End,
}

struct State {
    player: Player,
    food: Food,
    frame_time: f32,
    mode: GameMode,
    score: i32,
    x_image_0: f64,
    y_image_0: f64,
}

impl State {
    fn new() -> Self {
        State {
            player: Player::new(Point::new(25, 25)),
            frame_time: 0.0,
            mode: GameMode::Menu,
            food: Food::new(Point::new(50, 25)),
            score: 0,
            x_image_0: 0.0,
            y_image_0: 0.0,
        }
    }

    fn restart(&mut self) {
        self.player = Player::new(Point::new(25, 25));
        self.food = Food::new(Point::new(50, 25));
        self.frame_time = 0.0;
        self.mode = GameMode::Playing;
        self.score = 0;
        (self.x_image_0, self.y_image_0) = self.get_nose_image_pos();
    }

    fn main_menu(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_centered(5, "Welcome to Snake");
        ctx.print_centered(8, "(P) Play Game");
        ctx.print_centered(9, "(Q) Quit Game");

        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.restart(),
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }

    fn dead(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_centered(5, "You are dead!");
        ctx.print_centered(6, &format!("You earned {} points", self.score));
        ctx.print_centered(8, "(P) Play Again");
        ctx.print_centered(9, "(Q) Quit Game");

        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.restart(),
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }

    fn get_nose_image_pos(&mut self) -> (f64, f64) {
        let res2: PyResult<(f64, f64)> = Python::with_gil(|py| {
            let builtins = PyModule::import(py, "a")?;
            let total: (f64, f64) = builtins.getattr("get_coords")?.call1((32.0 as f64,))?.extract()?;
            Ok(total)
        });
        res2.unwrap()
        // let (x, y) = res2.unwrap();
        // Point::new((x * (SCREEN_WIDTH as f64)) as i32, (y * (SCREEN_HEIGHT as f64)) as i32)
    }
    
    fn get_nose_game_pos(&mut self, x: f64, y: f64) -> Point {
        // x = x - self.nose_image_x0;
        let x_offset = -0.07;
        let a_x = SCREEN_WIDTH as f64 / (2.0 * x_offset);
        let b_x = SCREEN_WIDTH as f64 / 2.0 - a_x * self.x_image_0;
        let x_game = (a_x * x + b_x).clamp(0.0, SCREEN_WIDTH as f64);
        let y_offset = 0.1;
        let a_y = SCREEN_WIDTH as f64 / (2.0 * y_offset);
        let b_y = SCREEN_WIDTH as f64 / 2.0 - a_y * self.y_image_0;
        let y_game = (a_y * y + b_y).clamp(0.0, SCREEN_HEIGHT as f64);

        Point::new(x_game as i32,  y_game as i32)
    }

    fn play(&mut self, ctx: &mut BTerm) {
        ctx.cls_bg(NAVY);
        // let mouse_pos = INPUT.lock().mouse_tile(0);
        let (x_img, y_img) = self.get_nose_image_pos();
        let mouse_pos = self.get_nose_game_pos(x_img, y_img);
        self.player.move_(mouse_pos);
        self.player.render(ctx);
        self.food.render(ctx);
        if self.player.pos == self.food.pos {
            self.score += 1;
            self.food = Food::respawn();
        }

        ctx.print(0, 1, &format!("Score: {}, nose: {}, {}", self.score, x_img, y_img)); // (4)
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        match self.mode {
            GameMode::Menu => self.main_menu(ctx),
            GameMode::End => self.dead(ctx),
            GameMode::Playing => self.play(ctx),
        }
    }
}

fn main()  -> BError {
    let context = BTermBuilder::simple80x50().with_title("Snale").build()?;

    main_loop(context, State::new())
}