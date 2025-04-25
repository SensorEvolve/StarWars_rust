use ggez::{
    audio::{self, SoundSource},
    context::Context,
    event::{self, EventHandler},
    graphics::{self, Color, DrawMode, DrawParam, Image, Mesh, Rect, Text},
    input::keyboard::{KeyCode, KeyInput},
    mint::Point2,
    GameResult,
};
use std::path::PathBuf;

const WINDOW_WIDTH: f32 = 1600.0;
const WINDOW_HEIGHT: f32 = 900.0;
const FPS: u32 = 60;
const MAX_BULLETS: usize = 3;
const BULLET_VEL: f32 = 25.0;

#[derive(Clone, Copy)]
struct Spaceship {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    health: i32,
}

impl Spaceship {
    fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Spaceship {
            x,
            y,
            width,
            height,
            health: 10,
        }
    }

    fn intersects(&self, bullet: &Bullet) -> bool {
        bullet.x >= self.x
            && bullet.x <= self.x + self.width
            && bullet.y >= self.y
            && bullet.y <= self.y + self.height
    }
}

#[derive(Clone, Copy)]
struct Bullet {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

struct GameState {
    rebel: Spaceship,
    imperial: Spaceship,
    rebel_bullets: Vec<Bullet>,
    imperial_bullets: Vec<Bullet>,
    rebel_ship_image: Image,
    imperial_ship_image: Image,
    background_image: Image,
    bullet_hit_sound: audio::Source,
    bullet_fire_sound: audio::Source,
    game_over: bool,
    winner: Option<String>,
}

impl EventHandler for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        // Limit frame rate
        while ctx.time.check_update_time(FPS) {
            // Handle bullet movement and collision
            self.update_rebel_bullets(ctx);
            self.update_imperial_bullets(ctx);

            // Check for game over conditions
            if self.rebel.health <= 0 {
                self.game_over = true;
                self.winner = Some("Imperial wins!".to_string());
            }
            if self.imperial.health <= 0 {
                self.game_over = true;
                self.winner = Some("Rebel wins!".to_string());
            }
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        // Create a new canvas for drawing
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::BLACK);

        // Draw background
        canvas.draw(&self.background_image, DrawParam::default());

        // Draw border
        let border_rect = Rect::new(WINDOW_WIDTH / 2.0 - 5.0, 0.0, 10.0, WINDOW_HEIGHT);
        let border_mesh = Mesh::new_rectangle(ctx, DrawMode::fill(), border_rect, Color::BLACK)?;
        canvas.draw(&border_mesh, DrawParam::default());

        // Draw spaceships
        canvas.draw(
            &self.rebel_ship_image,
            DrawParam::default().dest(Point2 {
                x: self.rebel.x,
                y: self.rebel.y,
            }),
        );
        canvas.draw(
            &self.imperial_ship_image,
            DrawParam::default().dest(Point2 {
                x: self.imperial.x,
                y: self.imperial.y,
            }),
        );

        // Draw bullets
        for bullet in &self.rebel_bullets {
            let bullet_rect = Rect::new(bullet.x, bullet.y, bullet.width, bullet.height);
            let bullet_mesh = Mesh::new_rectangle(ctx, DrawMode::fill(), bullet_rect, Color::BLUE)?;
            canvas.draw(&bullet_mesh, DrawParam::default());
        }

        for bullet in &self.imperial_bullets {
            let bullet_rect = Rect::new(bullet.x, bullet.y, bullet.width, bullet.height);
            let bullet_mesh = Mesh::new_rectangle(ctx, DrawMode::fill(), bullet_rect, Color::RED)?;
            canvas.draw(&bullet_mesh, DrawParam::default());
        }

        // Draw health
        let health_text = Text::new(format!("Rebel Health: {}", self.rebel.health));
        canvas.draw(
            &health_text,
            DrawParam::default().dest(Point2 { x: 10.0, y: 10.0 }),
        );

        let imperial_health_text = Text::new(format!("Imperial Health: {}", self.imperial.health));
        canvas.draw(
            &imperial_health_text,
            DrawParam::default().dest(Point2 {
                x: WINDOW_WIDTH - 200.0,
                y: 10.0,
            }),
        );

        // Draw game over message if game is over
        if self.game_over {
            if let Some(winner) = &self.winner {
                let game_over_text = Text::new(winner.clone());
                canvas.draw(
                    &game_over_text,
                    DrawParam::default().dest(Point2 {
                        x: WINDOW_WIDTH / 2.0 - 100.0,
                        y: WINDOW_HEIGHT / 2.0,
                    }),
                );
            }
        }

        // Finish drawing and present to screen
        canvas.finish(ctx)?;
        Ok(())
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        input: KeyInput,
        _repeated: bool,
    ) -> GameResult {
        let keycode = match input.keycode {
            Some(keycode) => keycode,
            None => return Ok(()),
        };

        match keycode {
            // Imperial ship controls (WASD)
            KeyCode::A => {
                if self.imperial.x > 0.0 {
                    self.imperial.x -= 5.0;
                }
            }
            KeyCode::D => {
                if self.imperial.x < WINDOW_WIDTH / 2.0 - 80.0 {
                    self.imperial.x += 5.0;
                }
            }
            KeyCode::W => {
                if self.imperial.y > 0.0 {
                    self.imperial.y -= 5.0;
                }
            }
            KeyCode::S => {
                if self.imperial.y < WINDOW_HEIGHT - 65.0 {
                    self.imperial.y += 5.0;
                }
            }

            // Rebel ship controls (Arrow keys)
            KeyCode::Left => {
                if self.rebel.x > WINDOW_WIDTH / 2.0 {
                    self.rebel.x -= 5.0;
                }
            }
            KeyCode::Right => {
                if self.rebel.x + self.rebel.width < WINDOW_WIDTH {
                    self.rebel.x += 5.0;
                }
            }
            KeyCode::Up => {
                if self.rebel.y > 0.0 {
                    self.rebel.y -= 5.0;
                }
            }
            KeyCode::Down => {
                if self.rebel.y + self.rebel.height < WINDOW_HEIGHT {
                    self.rebel.y += 5.0;
                }
            }

            // Imperial bullet fire
            KeyCode::LShift => {
                if self.imperial_bullets.len() < MAX_BULLETS {
                    let new_bullet = Bullet {
                        x: self.imperial.x + self.imperial.width,
                        y: self.imperial.y + self.imperial.height / 2.0 - 2.0,
                        width: 10.0,
                        height: 5.0,
                    };
                    self.imperial_bullets.push(new_bullet);
                    // Play sound
                    let _ = self.bullet_fire_sound.play(ctx);
                    self.bullet_fire_sound.set_volume(0.5);
                }
            }

            // Rebel bullet fire
            KeyCode::RAlt => {
                if self.rebel_bullets.len() < MAX_BULLETS {
                    let new_bullet = Bullet {
                        x: self.rebel.x,
                        y: self.rebel.y + self.rebel.height / 2.0 - 2.0,
                        width: 10.0,
                        height: 5.0,
                    };
                    self.rebel_bullets.push(new_bullet);
                    // Play sound
                    let _ = self.bullet_fire_sound.play(ctx);
                    self.bullet_fire_sound.set_volume(0.5);
                }
            }

            _ => {}
        }
        Ok(())
    }
}

impl GameState {
    fn update_rebel_bullets(&mut self, ctx: &mut Context) {
        let mut bullets_to_remove = Vec::new();

        for (i, bullet) in self.rebel_bullets.iter_mut().enumerate() {
            // Move bullet
            bullet.x -= BULLET_VEL;

            // Check collision with imperial ship
            if self.imperial.intersects(bullet) {
                self.imperial.health -= 1;
                bullets_to_remove.push(i);
                // Play hit sound
                let _ = self.bullet_hit_sound.play(ctx);
                self.bullet_hit_sound.set_volume(0.5);
            }

            // Remove if out of bounds
            if bullet.x < 0.0 {
                bullets_to_remove.push(i);
            }
        }

        // Remove bullets in reverse order to avoid index shifting
        bullets_to_remove.sort_by(|a, b| b.cmp(a));
        for i in bullets_to_remove {
            self.rebel_bullets.remove(i);
        }
    }

    fn update_imperial_bullets(&mut self, ctx: &mut Context) {
        let mut bullets_to_remove = Vec::new();

        for (i, bullet) in self.imperial_bullets.iter_mut().enumerate() {
            // Move bullet
            bullet.x += BULLET_VEL;

            // Check collision with rebel ship
            if self.rebel.intersects(bullet) {
                self.rebel.health -= 1;
                bullets_to_remove.push(i);
                // Play hit sound
                let _ = self.bullet_hit_sound.play(ctx);
                self.bullet_hit_sound.set_volume(0.5);
            }

            // Remove if out of bounds
            if bullet.x > WINDOW_WIDTH {
                bullets_to_remove.push(i);
            }
        }

        // Remove bullets in reverse order to avoid index shifting
        bullets_to_remove.sort_by(|a, b| b.cmp(a));
        for i in bullets_to_remove {
            self.imperial_bullets.remove(i);
        }
    }
}

fn main() -> GameResult {
    let resource_dir = PathBuf::from("./assets");

    let context_builder = ggez::ContextBuilder::new("star_wars_rust", "carlos_juan")
        .window_setup(ggez::conf::WindowSetup::default().title("Star Wars Rust Battle"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(WINDOW_WIDTH, WINDOW_HEIGHT))
        .add_resource_path(resource_dir);

    let (ctx, event_loop) = context_builder.build()?;

    // Load audio
    let bullet_hit_sound = audio::Source::new(&ctx, "/explosion.mp3")?;
    let bullet_fire_sound = audio::Source::new(&ctx, "/laser.mp3")?;

    let state = GameState {
        rebel: Spaceship::new(1400.0, 400.0, 80.0, 65.0),
        imperial: Spaceship::new(50.0, 400.0, 80.0, 65.0),
        rebel_bullets: Vec::new(),
        imperial_bullets: Vec::new(),
        rebel_ship_image: Image::from_path(&ctx, "/rebel_spaceship.png")?,
        imperial_ship_image: Image::from_path(&ctx, "/imperial_spaceship.png")?,
        background_image: Image::from_path(&ctx, "/bg_version_2.jpg")?,
        bullet_hit_sound,
        bullet_fire_sound,
        game_over: false,
        winner: None,
    };

    event::run(ctx, event_loop, state)
}
