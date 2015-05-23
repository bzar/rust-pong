extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate rand;
extern crate cgmath;

use piston::window::WindowSettings;
use piston::event::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{ GlGraphics, OpenGL };
use opengl_graphics::glyph_cache::GlyphCache;
use piston::input::{ Button, Key };
use std::path::Path;
use cgmath::{ Vector2, Vector, EuclideanVector };

const AREA_WIDTH: f64 = 200.0;
const AREA_HEIGHT: f64 = 200.0;
const PADDLE_SPEED: f64 = 50.0;
const BALL_START_SPEED: f64 = 40.0;
const BALL_SPEED_INCREASE: f64 = 5.0;

const FONT_PATH: &'static str = "ttf/DejaVuSans.ttf";

const BACKGROUND_COLOR: [f32; 4] = [0.1, 0.1, 0.1, 1.0];
const PADDLE_COLOR: [f32; 4] = [0.8, 0.8, 0.8, 1.0];
const BALL_COLOR: [f32; 4] = [0.9, 0.9, 0.9, 1.0];
const TEXT_COLOR: [f32; 4] = [0.5, 0.5, 0.5, 1.0];

struct Rectangle {
    center: Vector2<f64>,
    extent: Vector2<f64>
}

impl Rectangle {
    fn new(vals: [f64; 4]) -> Rectangle {
        return Rectangle {
            center: Vector2::new(vals[0], vals[1]),
            extent: Vector2::new(vals[2], vals[3])
        };
    }

    fn as_array(&self) -> [f64; 4] {
        return [self.center.x, self.center.y, self.extent.x, self.extent.y];
    }
    fn intersects(&self, other: &Rectangle) -> bool{
        let delta = self.center - other.center;
        let size = self.extent + other.extent;
        return delta.x.abs() <= size.x && delta.y.abs() <= size.y;
    }
}

struct Paddle {
    rect: Rectangle,
    v: Vector2<f64>
}

struct Ball {
    rect: Rectangle,
    v: Vector2<f64>,
    speed: f64
}

pub struct App<'a> {
    gl: GlGraphics,
    glyph_cache: GlyphCache<'a>,
    ball: Ball,
    left: Paddle,
    right: Paddle,
    left_score: u8,
    right_score: u8
}

impl Ball {
    fn new(size: f64) -> Ball {
        return Ball { 
            rect: Rectangle::new([0.0, 0.0, size, size]), 
            v: Vector2 {x: 0.0, y: 0.0}, 
            speed: 0.0 
        };
    }

    fn set_direction(&mut self, d: &Vector2<f64>) {
        self.v = d.normalize_to(self.speed);
    }

    fn reset(&mut self) {
        use rand::Rand;
        let mut rng = rand::thread_rng();
        self.rect.center = Vector2 { x: 100.0, y: 100.0 };
        self.speed = BALL_START_SPEED;
        let dy = f64::rand(&mut rng) - 0.5;
        let dx = if bool::rand(&mut rng) { -0.5 } else { 0.5 };
        self.set_direction(&Vector2 { x: dx, y: dy });
    }

}
impl <'a>App<'a> {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        let ball = rectangle::centered(self.ball.rect.as_array());
        let left = rectangle::centered(self.left.rect.as_array());
        let right = rectangle::centered(self.right.rect.as_array());
        let character_cache = &mut self.glyph_cache;
        let left_score_string = self.left_score.to_string();
        let right_score_string = self.right_score.to_string();

        self.gl.draw(args.viewport(), |c, gl| {
            clear(BACKGROUND_COLOR, gl);

            rectangle(PADDLE_COLOR, left, c.transform, gl);
            rectangle(PADDLE_COLOR, right, c.transform, gl);
            rectangle(BALL_COLOR, ball, c.transform, gl);

            let text = text::Text::colored(TEXT_COLOR, 20);
            text.draw(&left_score_string, character_cache,
                      default_draw_state(), c.transform.trans(5.0, 20.0), gl);
            text.draw(&right_score_string, character_cache,
                      default_draw_state(), c.transform.trans(180.0, 20.0), gl);
        });

    }

    fn update(&mut self, args: &UpdateArgs) {
        self.left.rect.center.add_self_v(&self.left.v.mul_s(args.dt));
        self.right.rect.center.add_self_v(&self.right.v.mul_s(args.dt));
        self.ball.rect.center.add_self_v(&self.ball.v.mul_s(args.dt));

        fn clamp_rect_y(rect: &Rectangle, min: f64, max: f64) -> f64 {
            return (rect.extent.y + min).max(rect.center.y.min(max - rect.extent.y));
        }
        self.left.rect.center.y = clamp_rect_y(&self.left.rect, 0.0, AREA_HEIGHT);
        self.right.rect.center.y = clamp_rect_y(&self.right.rect, 0.0, AREA_HEIGHT);

        fn paddle_bounce(ball: &mut Ball, paddle: &Paddle) {
            if ball.rect.intersects(&paddle.rect) {
                let d = ball.rect.center - paddle.rect.center;
                ball.speed += BALL_SPEED_INCREASE;
                ball.set_direction(&d);
            }
        }

        paddle_bounce(&mut self.ball, &self.left);
        paddle_bounce(&mut self.ball, &self.right);

        let hit_ceiling = self.ball.rect.center.y - self.ball.rect.extent.y < 0.0;
        let hit_floor = self.ball.rect.center.y + self.ball.rect.extent.y > AREA_HEIGHT;

        if hit_ceiling || hit_floor {
            self.ball.v.y = -self.ball.v.y;
        }

        let left_goal = self.ball.rect.center.x + self.ball.rect.extent.y < 0.0;
        let right_goal = self.ball.rect.center.x - self.ball.rect.extent.y > AREA_HEIGHT; 

        if left_goal {
            self.ball.reset();
            self.right_score += 1;
        } else if right_goal {
            self.ball.reset();
            self.left_score += 1;
        }
    }

    fn control(&mut self, button: Button, pressed: bool) {
        if pressed {
            match button {
                Button::Keyboard(Key::Up) => self.right.v.y = -PADDLE_SPEED,
                Button::Keyboard(Key::Down) => self.right.v.y = PADDLE_SPEED,
                Button::Keyboard(Key::A) => self.left.v.y = -PADDLE_SPEED,
                Button::Keyboard(Key::Z) => self.left.v.y = PADDLE_SPEED,
                _ => ()
            }
        } else {
            match button {
                Button::Keyboard(Key::Up) => self.right.v.y = 0.0,
                Button::Keyboard(Key::Down) => self.right.v.y = 0.0,
                Button::Keyboard(Key::A) => self.left.v.y = 0.0,
                Button::Keyboard(Key::Z) => self.left.v.y = 0.0,
                _ => ()
            }
        }
    }
}

fn main() {
    let opengl = OpenGL::_3_2;

    let window = Window::new(
        opengl,
        WindowSettings::new(
            "rust-pong",
            [AREA_WIDTH as u32, AREA_HEIGHT as u32]
            )
        .exit_on_esc(true)
        );

    if let Ok(glyph_cache) = GlyphCache::new(Path::new(FONT_PATH)) {
        let left = Paddle { 
            rect: Rectangle::new([16.0, AREA_HEIGHT/2.0, 8.0, 24.0]), 
            v: Vector2 { x: 0.0, y: 0.0 } 
        };
        let right = Paddle { 
            rect: Rectangle::new([AREA_WIDTH - 16.0, AREA_HEIGHT/2.0, 8.0, 24.0]), 
            v: Vector2 { x: 0.0, y: 0.0 } 
        };

        let mut app = App {
            gl: GlGraphics::new(opengl),
            glyph_cache: glyph_cache,
            ball: Ball::new(8.0),
            left: left,
            right: right,
            left_score: 0,
            right_score: 0
        };

        app.ball.reset();

        for e in window.events() {
            if let Some(r) = e.render_args() {
                app.render(&r);
            }

            if let Some(u) = e.update_args() {
                app.update(&u);
            }

            if let Some(button) = e.press_args() {
                app.control(button, true);
            }

            if let Some(button) = e.release_args() {
                app.control(button, false);
            }
        }
    } else {
        println!("Could not load font at {}", FONT_PATH);
    }
}

