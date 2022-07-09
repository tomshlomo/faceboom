#![warn(clippy::pedantic)]
use std::str::FromStr;
use std::fmt;
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

struct FaceLandMarks {
    nose: PointF,
    bottom_lip: PointF,
    upper_lip: PointF,
}

impl FaceLandMarks {
    fn default() -> FaceLandMarks {
        FaceLandMarks { 
            nose: PointF::new(0.0, 0.0),
            bottom_lip: PointF::new(0.0, 0.0),
            upper_lip: PointF::new(0.0, 0.0),
         }
    }
    fn from_webcam() -> Option<FaceLandMarks> {
        let res2: PyResult<[f32; 6]> = Python::with_gil(|py| {
            let builtins = PyModule::import(py, "a")?;
            let total: [f32; 6] = builtins.getattr("get_coords")?.call1((32.0 as f64,))?.extract()?;
            Ok(total)
        });
        match res2 {
            Ok(x) => Some(FaceLandMarks{
                nose: PointF::new(x[0], x[1]),
                upper_lip: PointF::new(x[2], x[3]),
                bottom_lip: PointF::new(x[4], x[5]),
            }),
            _ => None,
        }
    }

    fn mouth_height(&self) -> f32 {
        (self.upper_lip - self.bottom_lip).mag_sq()
    }
    
}

impl fmt::Display for FaceLandMarks {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      write!(f, "mouth_height: {:.3},  upper_lip: ({:.2}, {:.2}), bottom_lip: ({:.2}, {:.2}), nose: ({:.2}, {:.2})",
      self.mouth_height(), self.upper_lip.x, self.upper_lip.y, self.bottom_lip.x, self.bottom_lip.y, self.nose.x, self.nose.y)
    }
}

struct State {
    player: Player,
    food: Food,
    frame_time: f32,
    mode: GameMode,
    score: i32,
    face_landmarks_ref: FaceLandMarks,
}

impl State {
    fn new() -> Self {
        State {
            player: Player::new(PointF::new(25.0, 25.0)),
            frame_time: 0.0,
            mode: GameMode::Menu,
            food: Food::new(PointF::new(50.0, 25.0)),
            score: 0,
            face_landmarks_ref: FaceLandMarks::default(),
        }
    }

    fn restart(&mut self) {
        self.player = Player::new(PointF::new(25.0, 25.0));
        self.food = Food::new(PointF::new(50.0, 25.0));
        self.frame_time = 0.0;
        self.mode = GameMode::Playing;
        self.score = 0;
        loop {
            match FaceLandMarks::from_webcam() {
                Some(face_landmarks) => {
                    self.face_landmarks_ref = face_landmarks;
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
    
    fn get_nose_game_pos(&mut self, nose_image_pos: PointF) -> PointF {
        // x = x - self.nose_image_x0;
        let scale = PointF::new(-0.07, 0.1);
        let screen = PointF::new(SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32);

        let a = screen / (2.0 * scale);
        let b = screen / 2.0 - a * self.face_landmarks_ref.nose;
        let mut nose_game_pos = a * nose_image_pos + b;
        nose_game_pos.clamp(PointF::zero(), screen);
        nose_game_pos
    }

    fn play(&mut self, ctx: &mut BTerm) {
        ctx.cls_bg(NAVY);
        ctx.set_active_console(1);
        ctx.cls();
        self.player.render(ctx);
        self.food.render(ctx);
        ctx.set_active_console(0);
        // let mouse_pos = INPUT.lock().mouse_tile(0);
        let face_landmarks_opt = FaceLandMarks::from_webcam();
        let display_str = match face_landmarks_opt {
            Some(face_landmarks) => {
                let mouse_pos = self.get_nose_game_pos(face_landmarks.nose);
                self.player.move_(mouse_pos);
                format!("{}", face_landmarks)
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