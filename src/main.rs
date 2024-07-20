use macroquad::prelude::*;

// you should be able to move around
// you should be able to shoot projectiles
// the projectiles should follow vector field
// maybe vector field should also affect player velocity
// there should be enemies
// enemies also move around
// hitting enemies increases your score
// there's a time limit

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

    fn bounds_clamp(&mut self) {
        if self.pos.x < 0.0 {
            self.pos.x = 0.0;
            self.vel.x = 0.0;
            self.acc.x = 0.0;
        }
        if self.pos.x > screen_width() {
            self.pos.x = screen_width();
            self.vel.x = 0.0;
            self.acc.x = 0.0;
        }
        if self.pos.y < 0.0 {
            self.pos.y = 0.0;
            self.vel.y = 0.0;
            self.acc.y = 0.0;
        }
        if self.pos.y > screen_height() {
            self.pos.y = screen_height();
            self.vel.y = 0.0;
            self.acc.y = 0.0;
        }
    }
}

const PLAYER_MOVEMENT: f32 = 1000.0;
const PLAYER_MAX_MOVEMENT_SPEED: f32 = 1000.0;
const PROJECTILE_INIT_SPEED: f32 = 1.5 * PLAYER_MAX_MOVEMENT_SPEED;
const FRICTION: f32 = 800.0;

#[macroquad::main("flowfield")]
async fn main() {
    request_new_screen_size(1000.0, 1000.0);

    let mut player = Body::new(
        Vec2::new(screen_width() - 30.0, screen_height() - 30.0),
        Vec2::ZERO,
        Vec2::ZERO,
    );

    let mut projectiles: Vec<Body> = vec![];

    loop {
        let dt = get_frame_time();
        clear_background(Color::from_hex(0xFEFAE0));

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

        player.bounds_clamp();
        player.update_position(dt);

        if is_mouse_button_down(MouseButton::Left) {
            let init_dir = Vec2::from_array(mouse_position().into()) - player.pos;
            let init_vel = init_dir.normalize_or(Vec2::X) * PROJECTILE_INIT_SPEED;
            projectiles.push(Body::new(player.pos, init_vel, Vec2::ZERO));
        }

        projectiles
            .iter_mut()
            .for_each(|projectile| projectile.update_position(dt));

        draw_circle(player.pos.x, player.pos.y, 15.0, Color::from_hex(0x22577a));
        projectiles.iter().for_each(|projectile| {
            draw_circle(
                projectile.pos.x,
                projectile.pos.y,
                5.0,
                Color::from_hex(0xbc4749),
            )
        });

        next_frame().await
    }
}
