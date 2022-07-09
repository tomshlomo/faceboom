#![warn(clippy::pedantic)]
use std::str::FromStr;

use pyo3::prelude::*;

use bracket_lib::prelude::*;

const SCREEN_WIDTH: f32 = 80.0;
const SCREEN_HEIGHT: f32 = 50.0;
const FRAME_DURATION: f32 = 25.0;
const DRAGON_FRAMES : [u16; 6] = [ 64, 1, 2, 3, 2, 1 ];
const FOOD_FRAMES : [u16; 1] = [ 4 ];

struct Player {
    pos: PointF,
}

impl Player {
    fn new(pos: PointF) -> Self {
        Player { pos: pos }
    }

    fn move_(&mut self, pos: PointF) {
        self.pos = pos;
    }

    fn render(&self, ctx: &mut BTerm) {
        // ctx.set_active_console(1);
        ctx.set_fancy(
            self.pos,
            1000000,
            Degrees::new(0.0),
            PointF::new(2.0, 2.0),
            WHITE,
            NAVY,
            DRAGON_FRAMES[0],
        );
        // ctx.set_active_console(0);
        // ctx.set(self.pos.x, self.pos.y, YELLOW, BLACK, to_cp437('@'));
    }
}

struct Food {
    pos: PointF,
}

impl Food {
    fn new(pos: PointF) -> Self {
        Food { pos: pos }
    }

    fn render(&self, ctx: &mut BTerm) {
        // ctx.set_active_console(1);
        ctx.set_fancy(
            self.pos,
            1,
            Degrees::new(0.0),
            PointF::new(2.0, 2.0),
            RED,
            NAVY,
            FOOD_FRAMES[0],
        );
        // ctx.set_active_console(0);
        // ctx.set(self.pos.x, self.pos.y, RED, BLACK, to_cp437('*'));
    }

    fn respawn() -> Food {
        let mut random = RandomNumberGenerator::new();
        Food::new(PointF::new(
            random.range(0.0, SCREEN_WIDTH),
            random.range(0.0, SCREEN_HEIGHT),
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
            player: Player::new(PointF::new(25.0, 25.0)),
            frame_time: 0.0,
            mode: GameMode::Menu,
            food: Food::new(PointF::new(50.0, 25.0)),
            score: 0,
            x_image_0: 0.0,
            y_image_0: 0.0,
        }
    }

    fn restart(&mut self) {
        self.player = Player::new(PointF::new(25.0, 25.0));
        self.food = Food::new(PointF::new(50.0, 25.0));
        self.frame_time = 0.0;
        self.mode = GameMode::Playing;
        self.score = 0;
        loop {
            match self.get_nose_image_pos() {
                Some((x, y)) => {
                    self.x_image_0 = x;
                    self.y_image_0 = y;
                    break;
                }
                None => {},
            }
        }
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

    fn get_nose_image_pos(&mut self) -> Option<(f64, f64)> {
        let res2: PyResult<(f64, f64)> = Python::with_gil(|py| {
            let builtins = PyModule::import(py, "a")?;
            let total: (f64, f64) = builtins.getattr("get_coords")?.call1((32.0 as f64,))?.extract()?;
            Ok(total)
        });
        match res2 {
            Ok(x) => Some(x),
            _ => None,
        }
        // res2.unwrap()
        // let (x, y) = res2.unwrap();
        // PointF::new((x * (SCREEN_WIDTH as f64)) as i32, (y * (SCREEN_HEIGHT as f64)) as i32)
    }
    
    fn get_nose_game_pos(&mut self, x: f64, y: f64) -> PointF {
        // x = x - self.nose_image_x0;
        let x_offset = -0.07;
        let a_x = SCREEN_WIDTH as f64 / (2.0 * x_offset);
        let b_x = SCREEN_WIDTH as f64 / 2.0 - a_x * self.x_image_0;
        let x_game = (a_x * x + b_x).clamp(0.0, SCREEN_WIDTH as f64);
        let y_offset = 0.1;
        let a_y = SCREEN_WIDTH as f64 / (2.0 * y_offset);
        let b_y = SCREEN_WIDTH as f64 / 2.0 - a_y * self.y_image_0;
        let y_game = (a_y * y + b_y).clamp(0.0, SCREEN_HEIGHT as f64);

        PointF::new(x_game as f32,  y_game as f32)
    }

    fn play(&mut self, ctx: &mut BTerm) {
        ctx.cls_bg(NAVY);
        ctx.set_active_console(1);
        ctx.cls();
        self.player.render(ctx);
        self.food.render(ctx);
        ctx.set_active_console(0);
        // let mouse_pos = INPUT.lock().mouse_tile(0);
        let nose_pos = self.get_nose_image_pos();
        let display_str = match nose_pos {
            Some((x_img, y_img)) => {
                let mouse_pos = self.get_nose_game_pos(x_img, y_img);
                self.player.move_(mouse_pos);
                format!("Nose: {:.3}, {:.3}", x_img, y_img)
            },
            None => {
                String::from_str("No nose found!").unwrap()
            },
        };
        let d = (self.player.pos - self.food.pos).mag_sq();
        if d <= 10.0 {
            self.score += 1;
            self.food = Food::respawn();
        }

        ctx.print(0, 1, &format!("Score: {}, {}", self.score, display_str)); // (4)
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
    let context = BTermBuilder::new()
        .with_font("../resources/flappy32.png", 32, 32)
        .with_simple_console(SCREEN_WIDTH as i32, SCREEN_HEIGHT as i32, "../resources/flappy32.png")
        .with_fancy_console(SCREEN_WIDTH as i32, SCREEN_HEIGHT as i32, "../resources/flappy32.png")
        .with_title("FaceBoom")
        .with_tile_dimensions(16, 16)
        .build()?;
    // let context = BTermBuilder::simple80x50().with_title("Snale").build()?;

    main_loop(context, State::new())
}