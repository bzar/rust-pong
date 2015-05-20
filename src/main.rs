extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate rand;

use piston::window::WindowSettings;
use piston::event::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{ GlGraphics, OpenGL };
use opengl_graphics::glyph_cache::GlyphCache;
use piston::input::{ Button, Key };
use std::path::Path;

const PADDLE_SPEED: f64 = 50.0;
const BALL_START_SPEED: f64 = 40.0;
const BALL_SPEED_INCREASE: f64 = 5.0;

type Rectangle = [f64; 4]; // position and extents

fn intersects(a: &Rectangle, b: &Rectangle) -> bool {
    let dx =  if a[0] < b[0] { b[0] - a[0] } else { a[0] - b[0] };
    let dy = if a[1] < b[1] { b[1] - a[1] } else { a[1] - b[1] };
    return dx < a[2] + b[2] && dy < a[3] + b[3];
}

struct Paddle {
    rect: Rectangle,
    vy: f64
}

struct Ball {
    rect: Rectangle,
    v: [f64; 2],
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
        return Ball { rect: [0.0, 0.0, size, size], v: [0.0, 0.0], speed: 0.0 };
    }

    fn set_direction(&mut self, dx: f64, dy: f64) {
        let nx = dx / (dx * dx + dy * dy).sqrt();
        let ny = dy / (dx * dx + dy * dy).sqrt();
        self.v[0] = nx * self.speed;
        self.v[1] = ny * self.speed;
    }

    fn reset(&mut self) {
        use rand::Rand;
        let mut rng = rand::thread_rng();
        self.rect[0] = 100.0;
        self.rect[1] = 100.0;
        self.speed = BALL_START_SPEED;
        let dy = f64::rand(&mut rng) - 0.5;
        let dx = (f64::rand(&mut rng) - 0.5) * 4.0;
        self.set_direction(dx, dy);
    }

}
impl <'a>App<'a> {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const BACKGROUND_COLOR: [f32; 4] = [0.1, 0.1, 0.1, 1.0];
        const PADDLE_COLOR: [f32; 4] = [0.8, 0.8, 0.8, 1.0];
        const BALL_COLOR: [f32; 4] = [0.9, 0.9, 0.9, 1.0];
        const TEXT_COLOR: [f32; 4] = [0.5, 0.5, 0.5, 1.0];

        let ball = rectangle::centered(self.ball.rect);
        let left = rectangle::centered(self.left.rect);
        let right = rectangle::centered(self.right.rect);
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
        self.left.rect[1] += self.left.vy * args.dt;
        self.right.rect[1] += self.right.vy * args.dt;
        self.ball.rect[0] += self.ball.v[0] * args.dt;
        self.ball.rect[1] += self.ball.v[1] * args.dt;

        self.left.rect[1] = self.left.rect[3].max(self.left.rect[1].min(200.0 - self.left.rect[3]));
        self.right.rect[1] = self.right.rect[3].max(self.right.rect[1].min(200.0 - self.right.rect[3]));

        fn paddle_bounce(ball: &mut Ball, paddle: &Paddle) {
            if intersects(&paddle.rect, &ball.rect) {
                let dx = ball.rect[0] - paddle.rect[0];
                let dy = ball.rect[1] - paddle.rect[1];
                ball.speed += BALL_SPEED_INCREASE;
                ball.set_direction(dx, dy);
            }
        }

        paddle_bounce(&mut self.ball, &self.left);
        paddle_bounce(&mut self.ball, &self.right);

        let hit_ceiling = self.ball.rect[1] - self.ball.rect[3] < 0.0;
        let hit_floor = self.ball.rect[1] + self.ball.rect[3] > 200.0;

        if hit_ceiling || hit_floor {
            self.ball.v[1] = -self.ball.v[1];
        }

        let left_goal = self.ball.rect[0] + 2.0 * self.ball.rect[2] < 0.0;
        let right_goal = self.ball.rect[0] - 2.0 * self.ball.rect[2] > 200.0; 

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
                Button::Keyboard(Key::Up) => self.right.vy = -PADDLE_SPEED,
                Button::Keyboard(Key::Down) => self.right.vy = PADDLE_SPEED,
                Button::Keyboard(Key::A) => self.left.vy = -PADDLE_SPEED,
                Button::Keyboard(Key::Z) => self.left.vy = PADDLE_SPEED,
                _ => ()
            }
        } else {
            match button {
                Button::Keyboard(Key::Up) => self.right.vy = 0.0,
                Button::Keyboard(Key::Down) => self.right.vy = 0.0,
                Button::Keyboard(Key::A) => self.left.vy = 0.0,
                Button::Keyboard(Key::Z) => self.left.vy = 0.0,
                _ => ()
            }
        }
    }
}

fn main() {
    let opengl = OpenGL::_3_2;

    // Create an Glutin window.
    let window = Window::new(
        opengl,
        WindowSettings::new(
            "rust-pong",
            [200, 200]
            )
        .exit_on_esc(true)
        );

    if let Ok(glyph_cache) = GlyphCache::new(Path::new("ttf/DejaVuSans.ttf")) {
        let mut app = App {
            gl: GlGraphics::new(opengl),
            glyph_cache: glyph_cache,
            ball: Ball::new(8.0),
            left: Paddle { rect: [16.0, 100.0, 8.0, 24.0], vy: 0.0 },
            right: Paddle { rect: [184.0, 100.0, 8.0, 24.0], vy: 1.0 },
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
    }
}

