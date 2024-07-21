use std::collections::HashMap;

use macroquad::prelude::*;
use macroquad::rand::*;
use macroquad::ui::root_ui;
use noise::NoiseFn;
use noise::OpenSimplex;

// there's a menu
// you can choose different field
// there's an end screen with stats
// you can view session best
// you can submit score to db from end screen
// you can view top scores from db
// hit objects fade away?
// music (maybe we can have lowpassed menu music)
// sfx
// make stuff look better idk

struct Body {
    pos: Vec2,
    vel: Vec2,
    acc: Vec2,
}

impl Body {
    fn new(pos: Vec2, vel: Vec2, acc: Vec2) -> Body {
        Body { pos, vel, acc }
    }

    fn update_position(&mut self, dt: f32) {
        self.vel += dt * self.acc;
        self.pos += dt * self.vel;
    }

    fn is_in_bounds(&self) -> bool {
        self.pos.x >= 0.0
            && self.pos.x <= screen_width()
            && self.pos.y >= 0.0
            && self.pos.y <= screen_height()
    }

    fn bounds_clamp(&mut self) {
        if self.pos.x < 0.0 {
            self.pos.x = 0.0;
            self.vel.x = BOUNCE_BOOST * PLAYER_MAX_MOVEMENT_SPEED;
            self.acc.x = 0.0;
        }
        if self.pos.x > screen_width() {
            self.pos.x = screen_width();
            self.vel.x = -BOUNCE_BOOST * PLAYER_MAX_MOVEMENT_SPEED;
            self.acc.x = 0.0;
        }
        if self.pos.y < 0.0 {
            self.pos.y = 0.0;
            self.vel.y = BOUNCE_BOOST * PLAYER_MAX_MOVEMENT_SPEED;
            self.acc.y = 0.0;
        }
        if self.pos.y > screen_height() {
            self.pos.y = screen_height();
            self.vel.y = -BOUNCE_BOOST * PLAYER_MAX_MOVEMENT_SPEED;
            self.acc.y = 0.0;
        }
    }
}

const PLAYER_MOVEMENT: f32 = 1000.0;
const PLAYER_MAX_MOVEMENT_SPEED: f32 = 1000.0;
const PROJECTILE_INIT_SPEED: f32 = 1.5 * PLAYER_MAX_MOVEMENT_SPEED;
const ENEMY_INIT_SPEED: f32 = 0.5 * PLAYER_MAX_MOVEMENT_SPEED;
const ENEMY_INERTIA: f32 = 0.01;
const FRICTION: f32 = 800.0;
const VECTOR_FIELD_SCALAR: f32 = 0.01;
const ENEMY_RADIUS: f32 = 50.0;
const MAX_ENEMIES: usize = 5;
const BOUNCE_BOOST: f32 = 1.0;
const GAME_TIME_SECS: f32 = 30.0;

fn translate_pos(pos: Vec2) -> Vec2 {
    pos - Vec2::new(screen_width() / 2.0, screen_height() / 2.0)
}

type VectorFieldGetter = fn(macroquad::math::Vec2) -> macroquad::math::Vec2;

const DUAL_VISION: &'static str = "dual vision";
fn get_vector_field_force_basic(pos: Vec2) -> Vec2 {
    let Vec2 { x, y } = translate_pos(pos);
    VECTOR_FIELD_SCALAR * Vec2::new(x * x - y * y - 4.0, 2.0 * x * y)
}

const CURL_VALLEY: &'static str = "curl valley";
fn get_vector_field_force_curl_noise(pos: Vec2) -> Vec2 {
    const DERIVATIVE_SAMPLE: f64 = 0.001;
    let Vec2 { x: _x, y: _y } = translate_pos(pos) / 400.0;
    let x = _x as f64;
    let y = _y as f64;
    let noise = OpenSimplex::new(1);
    let x1 = noise.get([x + DERIVATIVE_SAMPLE, y]);
    let x2 = noise.get([x - DERIVATIVE_SAMPLE, y]);
    let y1 = noise.get([x, y + DERIVATIVE_SAMPLE]);
    let y2 = noise.get([x, y - DERIVATIVE_SAMPLE]);
    let x_d = x2 - x1;
    let y_d = y2 - y1;
    let angle = y_d.atan2(x_d);
    1000.0 * Vec2::from_angle(angle as f32)
}

const CLOCKBACK: &'static str = "clockback";
fn get_vector_field_force_circular(pos: Vec2) -> Vec2 {
    let Vec2 { x, y } = translate_pos(pos) / 400.0;
    let angle = (-y / x).atan();
    if x < 0.0 {
        2000.0 * -Vec2::from_angle(-angle).rotate(-Vec2::Y)
    } else {
        2000.0 * Vec2::from_angle(-angle).rotate(-Vec2::Y)
    }
}

fn draw_vector_field(get_vector_field_force: VectorFieldGetter) {
    for x in (0..screen_width() as i32).step_by(50) {
        for y in (0..screen_height() as i32).step_by(50) {
            let start = Vec2::new(x as f32, y as f32);
            let force = 0.01 * get_vector_field_force(start);
            let end = start - force;
            draw_circle(start.x, start.y, 2.0, Color::from_hex(0xDDA15E));
            draw_line(
                start.x,
                start.y,
                end.x,
                end.y,
                1.0,
                Color::from_hex(0xDDA15E),
            )
        }
    }
}

const FONT_SIZE: u16 = 40;

fn draw_text_ll(text: &str, x: f32, y: f32, font: Option<&Font>) {
    let size = measure_text(text, font, FONT_SIZE, 1.0);
    draw_text_at(text, x, y - size.height, FONT_SIZE, font)
}

fn draw_text_ur(text: &str, x: f32, y: f32, font: Option<&Font>) {
    let size = measure_text(text, font, FONT_SIZE, 1.0);
    draw_text_at(text, x - size.width, y, FONT_SIZE, font)
}

fn draw_text_ul(text: &str, x: f32, y: f32, font: Option<&Font>) {
    draw_text_at(text, x, y, FONT_SIZE, font)
}

fn draw_text_at(text: &str, x: f32, y: f32, font_size: u16, font: Option<&Font>) {
    draw_text_ex(
        text,
        x,
        y,
        TextParams {
            font_size,
            font,
            color: Color::from_hex(0x101010),
            ..Default::default()
        },
    );
}

fn draw_score_at(text: &str, x: f32, y: f32, font: Option<&Font>) {
    draw_text_at(text, x, y, 12, font);
}

#[derive(PartialEq)]
enum Stage {
    Home,
    Play,
    End,
}

#[macroquad::main("flowfield")]
async fn main() {
    let font = load_ttf_font("./DMSans-Regular.ttf").await.ok();
    set_fullscreen(true);

    let mut session_best_scores: HashMap<&'static str, i32> = HashMap::new();
    let mut stage = Stage::Home;
    let mut current_map = DUAL_VISION;
    let mut get_vector_field_force: VectorFieldGetter = get_vector_field_force_basic;

    let mut secs_left = GAME_TIME_SECS;
    let mut player = Body::new(
        Vec2::new(screen_width() - 30.0, screen_height() - 30.0),
        Vec2::ZERO,
        Vec2::ZERO,
    );
    let mut projectiles: Vec<Body> = vec![];
    let mut enemies: Vec<Body> = vec![];
    let mut num_projectiles = 0;
    let mut num_enemies_shot = 0;

    loop {
        let dt = get_frame_time();
        clear_background(Color::from_hex(0xFEFAE0));
        draw_vector_field(get_vector_field_force);

        // PLAYER
        player.acc = Vec2::ZERO;

        if is_key_down(KeyCode::D) {
            player.acc.x += PLAYER_MOVEMENT;
        }
        if is_key_down(KeyCode::A) {
            player.acc.x += -PLAYER_MOVEMENT;
        }
        if is_key_down(KeyCode::W) {
            player.acc.y += -PLAYER_MOVEMENT;
        }
        if is_key_down(KeyCode::S) {
            player.acc.y += PLAYER_MOVEMENT;
        }

        player.acc += -player.vel.normalize_or_zero() * FRICTION * player.vel.length()
            / PLAYER_MAX_MOVEMENT_SPEED;
        player.acc += get_vector_field_force(player.pos);

        player.bounds_clamp();
        player.update_position(dt);
        draw_circle(player.pos.x, player.pos.y, 15.0, Color::from_hex(0x22577a));

        // PROJECTILES
        if is_mouse_button_down(MouseButton::Left) {
            let init_dir = Vec2::from_array(mouse_position().into()) - player.pos;
            let init_vel = init_dir.normalize_or(Vec2::X) * PROJECTILE_INIT_SPEED;
            projectiles.push(Body::new(player.pos, init_vel, Vec2::ZERO));
            if stage == Stage::Play {
                num_projectiles += 1;
            }
        }

        projectiles
            .iter_mut()
            .for_each(|projectile| projectile.acc += get_vector_field_force(projectile.pos));

        projectiles
            .iter_mut()
            .for_each(|projectile| projectile.update_position(dt));

        projectiles.retain(|projectile| projectile.is_in_bounds());

        projectiles.iter().for_each(|projectile| {
            let before = enemies.len();
            enemies.retain(|enemy| enemy.pos.distance(projectile.pos) > ENEMY_RADIUS);
            let after = enemies.len();
            if stage == Stage::Play {
                num_enemies_shot += before - after;
            }
        });

        projectiles.iter().for_each(|projectile| {
            draw_circle(
                projectile.pos.x,
                projectile.pos.y,
                5.0,
                Color::from_hex(0xbc4749),
            )
        });

        if stage == Stage::Play {
            // ENEMIES
            if enemies.len() < MAX_ENEMIES {
                let pos_l = Vec2::new(-ENEMY_RADIUS, (rand() % screen_height() as u32) as f32);
                let pos_r = Vec2::new(
                    screen_width() + ENEMY_RADIUS,
                    (rand() % screen_height() as u32) as f32,
                );
                let pos_u = Vec2::new((rand() % screen_height() as u32) as f32, -ENEMY_RADIUS);
                let pos_d = Vec2::new(
                    (rand() % screen_height() as u32) as f32,
                    screen_height() + ENEMY_RADIUS,
                );
                let pos = [pos_d, pos_l, pos_r, pos_u][(rand() % 4) as usize];
                let dir = player.pos - pos;
                let vel = dir.normalize_or(Vec2::Y) * ENEMY_INIT_SPEED;
                let enemy = Body::new(pos, vel, Vec2::ZERO);
                enemies.push(enemy);
            }

            enemies
                .iter_mut()
                .for_each(|enemy| enemy.acc += ENEMY_INERTIA * get_vector_field_force(enemy.pos));

            enemies
                .iter_mut()
                .for_each(|enemy| enemy.update_position(dt));

            enemies.retain(|enemy| {
                enemy.pos.distance_squared(player.pos) <= screen_height() * screen_width()
            });

            enemies.iter().for_each(|enemy| {
                draw_circle(
                    enemy.pos.x,
                    enemy.pos.y,
                    ENEMY_RADIUS,
                    Color::from_hex(0xBC6C25),
                )
            });

            draw_text_ul(
                &format!("enemies shot {:.1}", num_enemies_shot),
                0.0,
                40.0,
                font.as_ref(),
            );
            draw_text_ur(
                &format!("time left {:.1} s", secs_left),
                screen_width(),
                40.0,
                font.as_ref(),
            );

            secs_left -= dt;
            if secs_left <= 0.0 {
                stage = Stage::End;
            }
        }

        if stage == Stage::Home {
            draw_text_at("flowfield", 80.0, 200.0, 100, font.as_ref());
            if root_ui().button(Some(Vec2::new(80.0, 500.0)), "play") {
                stage = Stage::Play;
                secs_left = GAME_TIME_SECS;
                num_enemies_shot = 0;
                num_projectiles = 0;
                enemies = vec![];
            }

            draw_score_at("session best", 5.0, 400.0, font.as_ref());

            if root_ui().button(Some(Vec2::new(80.0, 300.0)), DUAL_VISION) {
                current_map = DUAL_VISION;
                get_vector_field_force = get_vector_field_force_basic;
            }
            let score = &session_best_scores
                .get(DUAL_VISION)
                .map_or("unplayed".to_owned(), |score| format!("{}", score));
            draw_score_at(score, 80.0, 400.0, font.as_ref());

            if root_ui().button(Some(Vec2::new(240.0, 300.0)), CURL_VALLEY) {
                current_map = CURL_VALLEY;
                get_vector_field_force = get_vector_field_force_curl_noise;
            }
            let score = &session_best_scores
                .get(CURL_VALLEY)
                .map_or("unplayed".to_owned(), |score| format!("{}", score));
            draw_score_at(score, 240.0, 400.0, font.as_ref());

            if root_ui().button(Some(Vec2::new(400.0, 300.0)), CLOCKBACK) {
                current_map = CLOCKBACK;
                get_vector_field_force = get_vector_field_force_circular;
            }
            let score = &session_best_scores
                .get(CLOCKBACK)
                .map_or("unplayed".to_owned(), |score| format!("{}", score));
            draw_score_at(score, 400.0, 400.0, font.as_ref());

            draw_text_ll(
                "WASD to move, point and click to shoot",
                80.0,
                screen_height(),
                font.as_ref(),
            );
        }

        if stage == Stage::End {
            let final_score = 100 * (num_enemies_shot as i32) - num_projectiles;
            let previous = session_best_scores.get(current_map).unwrap_or(&i32::MIN);
            session_best_scores.insert(current_map, final_score.max(*previous));
            draw_text_at("game over", 80.0, 200.0, 100, font.as_ref());
            draw_text_ul(
                &format!("enemies shot 100 x {}", num_enemies_shot),
                80.0,
                300.0,
                font.as_ref(),
            );
            draw_text_ul(
                &format!("projectiles used -1 x {}", num_projectiles),
                80.0,
                400.0,
                font.as_ref(),
            );
            draw_text_ul(
                &format!("final score {:0}", final_score),
                80.0,
                500.0,
                font.as_ref(),
            );

            if root_ui().button(Some(Vec2::new(80.0, 600.0)), "continue") {
                stage = Stage::Home;
            }
        }

        next_frame().await
    }
}
