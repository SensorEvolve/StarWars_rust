use ggez::{
    audio::{self, SoundSource},
    context::Context,
    event::{self, EventHandler},
    graphics::{self, Color, DrawMode, DrawParam, Image, Mesh, Rect, Text, TextFragment},
    input::keyboard::{KeyCode, KeyInput},
    mint::Point2,
    GameResult,
};
use std::collections::HashSet;
use std::f32::consts::FRAC_PI_2;
use std::path::PathBuf;

const WINDOW_WIDTH: f32 = 1600.0;
const WINDOW_HEIGHT: f32 = 900.0;
const FPS: u32 = 60;
const MAX_BULLETS: usize = 3;
const BULLET_VEL: f32 = 10.0;
const SHIP_SPEED: f32 = 5.0;

#[derive(Clone, Copy)]
struct Spaceship {
    x: f32,
    y: f32,
    width: f32,  // visual width on screen (after rotation)
    height: f32, // visual height on screen (after rotation)
    health: i32,
}

impl Spaceship {
    fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Spaceship { x, y, width, height, health: 10 }
    }

    fn intersects(&self, bullet: &Bullet) -> bool {
        bullet.x + bullet.width  >= self.x
            && bullet.x          <= self.x + self.width
            && bullet.y + bullet.height >= self.y
            && bullet.y          <= self.y + self.height
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
    keys_held: HashSet<KeyCode>,
}

impl EventHandler for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        while ctx.time.check_update_time(FPS) {
            if self.game_over {
                return Ok(());
            }

            self.handle_movement();
            self.update_rebel_bullets(ctx);
            self.update_imperial_bullets(ctx);

            if self.rebel.health <= 0 {
                self.rebel.health = 0;
                self.game_over = true;
                self.winner = Some("Imperial Wins!".to_string());
            }
            if self.imperial.health <= 0 {
                self.imperial.health = 0;
                self.game_over = true;
                self.winner = Some("Rebel Wins!".to_string());
            }
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::BLACK);

        canvas.draw(&self.background_image, DrawParam::default());

        // Center divider
        let border_rect = Rect::new(WINDOW_WIDTH / 2.0 - 5.0, 0.0, 10.0, WINDOW_HEIGHT);
        let border_mesh = Mesh::new_rectangle(ctx, DrawMode::fill(), border_rect, Color::BLACK)?;
        canvas.draw(&border_mesh, DrawParam::default());

        // Imperial ship — rotated +90° (clockwise) so it faces RIGHT.
        //
        // With rotation +π/2, ggez pivots around dest.
        // Pixel (px, py) lands at screen (dest.x - py, dest.y + px).
        // The sprite's top-left (0,0) → (dest.x, dest.y)
        // The sprite's top-right (orig_w, 0) → (dest.x, dest.y + orig_w)
        // The sprite's bottom-left (0, orig_h) → (dest.x - orig_h, dest.y)
        // Visual bounding box: [dest.x - orig_h, dest.x] × [dest.y, dest.y + orig_w]
        //
        // To place the box at [ship.x, ship.x + vis_w] where vis_w = orig_h:
        //   dest.x = ship.x + orig_h
        {
            let orig_h = self.imperial_ship_image.height() as f32;
            canvas.draw(
                &self.imperial_ship_image,
                DrawParam::default()
                    .rotation(FRAC_PI_2)
                    .dest(Point2 {
                        x: self.imperial.x + orig_h,
                        y: self.imperial.y,
                    }),
            );
        }

        // Rebel ship — rotated -90° (counter-clockwise) so it faces LEFT.
        //
        // Pixel (px, py) lands at screen (dest.x + py, dest.y - px).
        // Visual bounding box: [dest.x, dest.x + orig_h] × [dest.y - orig_w, dest.y]
        //
        // To place the box at [ship.x, ship.x + vis_w]:
        //   dest.x = ship.x
        //   dest.y = ship.y + orig_w
        {
            let orig_w = self.rebel_ship_image.width() as f32;
            canvas.draw(
                &self.rebel_ship_image,
                DrawParam::default()
                    .rotation(-FRAC_PI_2)
                    .dest(Point2 {
                        x: self.rebel.x,
                        y: self.rebel.y + orig_w,
                    }),
            );
        }

        // Imperial bullets — red, travel right
        for bullet in &self.imperial_bullets {
            let r = Rect::new(bullet.x, bullet.y, bullet.width, bullet.height);
            let m = Mesh::new_rectangle(ctx, DrawMode::fill(), r, Color::RED)?;
            canvas.draw(&m, DrawParam::default());
        }

        // Rebel bullets — blue, travel left
        for bullet in &self.rebel_bullets {
            let r = Rect::new(bullet.x, bullet.y, bullet.width, bullet.height);
            let m = Mesh::new_rectangle(ctx, DrawMode::fill(), r, Color::BLUE)?;
            canvas.draw(&m, DrawParam::default());
        }

        // HUD: Imperial HP on the left, Rebel HP on the right
        let imp_hp = Text::new(
            TextFragment::new(format!("Imperial HP: {}", self.imperial.health)).scale(24.0),
        );
        canvas.draw(
            &imp_hp,
            DrawParam::default()
                .dest(Point2 { x: 10.0, y: 10.0 })
                .color(Color::from_rgb(255, 80, 80)),
        );

        let reb_hp = Text::new(
            TextFragment::new(format!("Rebel HP: {}", self.rebel.health)).scale(24.0),
        );
        canvas.draw(
            &reb_hp,
            DrawParam::default()
                .dest(Point2 { x: WINDOW_WIDTH - 160.0, y: 10.0 })
                .color(Color::from_rgb(80, 160, 255)),
        );

        // Game-over overlay
        if self.game_over {
            let overlay = Mesh::new_rectangle(
                ctx,
                DrawMode::fill(),
                Rect::new(0.0, 0.0, WINDOW_WIDTH, WINDOW_HEIGHT),
                Color::from_rgba(0, 0, 0, 180),
            )?;
            canvas.draw(&overlay, DrawParam::default());

            let go_text = Text::new(TextFragment::new("GAME OVER").scale(90.0));
            canvas.draw(
                &go_text,
                DrawParam::default()
                    .dest(Point2 {
                        x: WINDOW_WIDTH / 2.0 - 230.0,
                        y: WINDOW_HEIGHT / 2.0 - 90.0,
                    })
                    .color(Color::WHITE),
            );

            if let Some(winner) = &self.winner {
                let win_text = Text::new(TextFragment::new(winner.clone()).scale(60.0));
                canvas.draw(
                    &win_text,
                    DrawParam::default()
                        .dest(Point2 {
                            x: WINDOW_WIDTH / 2.0 - 160.0,
                            y: WINDOW_HEIGHT / 2.0 + 20.0,
                        })
                        .color(Color::YELLOW),
                );
            }
        }

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
            Some(k) => k,
            None => return Ok(()),
        };

        self.keys_held.insert(keycode);

        match keycode {
            KeyCode::LShift if !self.game_over => {
                if self.imperial_bullets.len() < MAX_BULLETS {
                    self.imperial_bullets.push(Bullet {
                        x: self.imperial.x + self.imperial.width,
                        y: self.imperial.y + self.imperial.height / 2.0 - 2.5,
                        width: 15.0,
                        height: 5.0,
                    });
                    let _ = self.bullet_fire_sound.play(ctx);
                    self.bullet_fire_sound.set_volume(0.5);
                }
            }
            KeyCode::RAlt if !self.game_over => {
                if self.rebel_bullets.len() < MAX_BULLETS {
                    self.rebel_bullets.push(Bullet {
                        x: self.rebel.x - 15.0,
                        y: self.rebel.y + self.rebel.height / 2.0 - 2.5,
                        width: 15.0,
                        height: 5.0,
                    });
                    let _ = self.bullet_fire_sound.play(ctx);
                    self.bullet_fire_sound.set_volume(0.5);
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn key_up_event(&mut self, _ctx: &mut Context, input: KeyInput) -> GameResult {
        if let Some(k) = input.keycode {
            self.keys_held.remove(&k);
        }
        Ok(())
    }
}

impl GameState {
    fn handle_movement(&mut self) {
        let half = WINDOW_WIDTH / 2.0;

        if self.keys_held.contains(&KeyCode::A) && self.imperial.x > 0.0 {
            self.imperial.x -= SHIP_SPEED;
        }
        if self.keys_held.contains(&KeyCode::D)
            && self.imperial.x + self.imperial.width < half - 5.0
        {
            self.imperial.x += SHIP_SPEED;
        }
        if self.keys_held.contains(&KeyCode::W) && self.imperial.y > 0.0 {
            self.imperial.y -= SHIP_SPEED;
        }
        if self.keys_held.contains(&KeyCode::S)
            && self.imperial.y + self.imperial.height < WINDOW_HEIGHT
        {
            self.imperial.y += SHIP_SPEED;
        }

        if self.keys_held.contains(&KeyCode::Left) && self.rebel.x > half + 5.0 {
            self.rebel.x -= SHIP_SPEED;
        }
        if self.keys_held.contains(&KeyCode::Right)
            && self.rebel.x + self.rebel.width < WINDOW_WIDTH
        {
            self.rebel.x += SHIP_SPEED;
        }
        if self.keys_held.contains(&KeyCode::Up) && self.rebel.y > 0.0 {
            self.rebel.y -= SHIP_SPEED;
        }
        if self.keys_held.contains(&KeyCode::Down)
            && self.rebel.y + self.rebel.height < WINDOW_HEIGHT
        {
            self.rebel.y += SHIP_SPEED;
        }
    }

    fn update_rebel_bullets(&mut self, ctx: &mut Context) {
        let mut to_remove: Vec<usize> = Vec::new();

        for (i, bullet) in self.rebel_bullets.iter_mut().enumerate() {
            bullet.x -= BULLET_VEL;

            if self.imperial.intersects(bullet) {
                self.imperial.health -= 1;
                to_remove.push(i);
                let _ = self.bullet_hit_sound.play(ctx);
                self.bullet_hit_sound.set_volume(0.5);
            } else if bullet.x + bullet.width < 0.0 {
                to_remove.push(i);
            }
        }

        to_remove.sort_unstable_by(|a, b| b.cmp(a));
        to_remove.dedup();
        for i in to_remove {
            self.rebel_bullets.remove(i);
        }
    }

    fn update_imperial_bullets(&mut self, ctx: &mut Context) {
        let mut to_remove: Vec<usize> = Vec::new();

        for (i, bullet) in self.imperial_bullets.iter_mut().enumerate() {
            bullet.x += BULLET_VEL;

            if self.rebel.intersects(bullet) {
                self.rebel.health -= 1;
                to_remove.push(i);
                let _ = self.bullet_hit_sound.play(ctx);
                self.bullet_hit_sound.set_volume(0.5);
            } else if bullet.x > WINDOW_WIDTH {
                to_remove.push(i);
            }
        }

        to_remove.sort_unstable_by(|a, b| b.cmp(a));
        to_remove.dedup();
        for i in to_remove {
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

    let bullet_hit_sound = audio::Source::new(&ctx, "/explosion.mp3")?;
    let bullet_fire_sound = audio::Source::new(&ctx, "/laser.mp3")?;

    let imperial_ship_image = Image::from_path(&ctx, "/imperial_spaceship.png")?;
    let rebel_ship_image    = Image::from_path(&ctx, "/rebel_spaceship.png")?;

    // After a 90° rotation the axes swap:
    //   visual width  on screen = original image height
    //   visual height on screen = original image width
    let imp_vis_w = imperial_ship_image.height() as f32;
    let imp_vis_h = imperial_ship_image.width()  as f32;
    let reb_vis_w = rebel_ship_image.height()    as f32;
    let reb_vis_h = rebel_ship_image.width()     as f32;

    let state = GameState {
        imperial: Spaceship::new(
            50.0,
            (WINDOW_HEIGHT - imp_vis_h) / 2.0,
            imp_vis_w,
            imp_vis_h,
        ),
        rebel: Spaceship::new(
            WINDOW_WIDTH - 50.0 - reb_vis_w,
            (WINDOW_HEIGHT - reb_vis_h) / 2.0,
            reb_vis_w,
            reb_vis_h,
        ),
        rebel_bullets: Vec::new(),
        imperial_bullets: Vec::new(),
        rebel_ship_image,
        imperial_ship_image,
        background_image: Image::from_path(&ctx, "/bg_version_2.jpg")?,
        bullet_hit_sound,
        bullet_fire_sound,
        game_over: false,
        winner: None,
        keys_held: HashSet::new(),
    };

    event::run(ctx, event_loop, state)
}
