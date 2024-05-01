use macroquad::prelude::*;

use macroquad_tiled as tiled;

struct Player {
    speed: Vec2,
    texture: Texture2D,
    pos: Vec2,
    facing_right: bool,
    health: f32,
}

impl Player {
    pub const MOVE_SPEED: f32 = 5.0;
    fn new(pos: Vec2, texture: Texture2D, health: f32) -> Self {
        Self {
            texture,
            speed: vec2(0., 0.),
            pos,
            facing_right: true,
            health,
        }
    }
}

struct Tower {
    cost: i32,
    weapon: Weapon,
    pos: Vec2,
    texture: Texture2D,
    strength: f32,
    health: f32,
    bullet_texture: Texture2D,
}

impl Tower {
    fn new(cost: i32, weapon: Weapon, pos: Vec2, texture: Texture2D, strength: f32, health: f32, bullet_texture: Texture2D) -> Self {
        Self {
            cost,
            weapon,
            pos,
            texture,
            strength,
            health,
            bullet_texture,
        }
    }
    fn check_shoot(&self, spawn_list: &mut Vec::<Box<dyn Entity>>, entities: &Vec::<Box<dyn Entity>>) {
        if is_key_down(KeyCode::Space) {
            let pos = entities[0].get_position();
            let velocity = (pos - self.pos);
            let bullet = Bullet::new(self.bullet_texture.clone(), self.pos, velocity.normalize_or_zero());
            spawn_list.push(Box::new(bullet));
        }
    }
}

struct Bullet {
    texture: Texture2D,
    pos: Vec2,
    velocity: Vec2,
    damage: f32,
}

impl Bullet {
    fn new(texture: Texture2D, pos: Vec2, velocity: Vec2) -> Self {
        Self {
            texture,
            pos,
            velocity,
            damage: 10.,
        }
    }
}

impl Position for Bullet {
    fn get_position(&self) -> Vec2 {
        self.pos
    }
}

impl Entity for Bullet {
    fn mut_update(&mut self) {
        if self.pos.x < 300. || self.pos.y < 300. {
            self.pos += self.velocity;
        }
    }
    fn draw(&self) {
        draw_texture_ex(
            &self.texture,
            self.pos.x,
            self.pos.y,
            WHITE,
            DrawTextureParams {
                ..Default::default()
            },
        );
    }
}

enum Weapon {
    Gun
}
/*
enum EntityEnum {
    Tower,
    Player,
    Enemy
}
*/

trait Health {
    fn take_damage(&mut self, amount: f32);
}

trait Position {
    fn get_position(&self) -> Vec2;
}

trait Targetable: Health + Position {}

trait Entity: Position {
    fn mut_update(&mut self);
    fn update(&self, _: &mut Vec::<Box<dyn Entity>>, _: &Vec::<Box<dyn Entity>>) {
        return
    }
    fn draw(&self);
}

impl Entity for Tower {
    fn draw(&self) {
        draw_texture_ex(
            &self.texture,
            self.pos.x,
            self.pos.y,
            WHITE,
            DrawTextureParams {
                source: Some(Rect::new(64., 112., 16., 16.)),
                ..Default::default()
            },
        );
    } 
    fn mut_update(&mut self) {
        return
    }
    fn update(&self, spawn_list: &mut Vec::<Box<dyn Entity>>, entities: &Vec::<Box<dyn Entity>>) {
        Self::check_shoot(&self, spawn_list, &entities);
    }
}
impl Position for Tower {
    fn get_position(&self) -> Vec2 {
        self.pos
    }
}

impl Position for Player {
    fn get_position(&self) -> Vec2 {
        self.pos
    }
}

impl Health for Player {
    fn take_damage(&mut self, amount: f32) {
        self.health -= amount;
        println!("{}", self.health);
    }
}


impl Entity for Player {    
    fn draw(&self) {
        draw_texture_ex(
            &self.texture,
            self.pos.x,
            self.pos.y,
            WHITE,
            DrawTextureParams {
                source: Some(Rect::new(0.0, 0.0, 45., 51.)),
                flip_x : !self.facing_right,
                ..Default::default()
            },
        );
    }
    fn mut_update(&mut self) {
        self.speed = vec2(0.0, 0.0);
        if is_key_down(KeyCode::W) {
            self.speed.y -= Self::MOVE_SPEED;
        }
        if is_key_down(KeyCode::A) {
            self.speed.x -= Self::MOVE_SPEED;
        }
        if is_key_down(KeyCode::S) {
            self.speed.y += Self::MOVE_SPEED;
        }
        if is_key_down(KeyCode::D) {
            self.speed.x += Self::MOVE_SPEED;
        }
        self.pos += self.speed.normalize_or_zero() * Self::MOVE_SPEED;
        if self.speed.x > 0. {
            self.facing_right = true;
        } else if self.speed.x < 0. {
            self.facing_right = false;
        }
    }
}

#[macroquad::main("BasicShapes")]
async fn main() {
    let tileset: Texture2D = load_texture("assets/basictiles.png").await.unwrap();
    tileset.set_filter(FilterMode::Nearest);
    let tiled_map_json = load_string("assets/testtilemap.json").await.unwrap();
    let tiled_map = tiled::load_map(
        &tiled_map_json,
        &[("basictiles.png", tileset)],
        &[],
        ).unwrap();
    let mut entities: Vec<Box<dyn Entity>> = vec![];
    let mut spawn_list: Vec<Box<dyn Entity>> = vec![];

    let player = load_texture("assets/disciple-45x51.png").await.unwrap();
    let player_pos = vec2(240., 160.);
    let mut player = Player::new(player_pos, player, 100.);
    entities.push(Box::new(player));
    
    let tower_texture = load_texture("assets/basictiles.png").await.unwrap();
    let bullet_texture = load_texture("assets/smallbullet.png").await.unwrap();
    let mut tower = Tower::new(30, Weapon::Gun, vec2(160., 160.), tower_texture, 5., 500., bullet_texture);
    entities.push(Box::new(tower));

    let width = tiled_map.raw_tiled_map.tilewidth * tiled_map.raw_tiled_map.width;
    let height = tiled_map.raw_tiled_map.tileheight * tiled_map.raw_tiled_map.height;

    loop {
        // draw background
        clear_background(BLACK);
        tiled_map.draw_tiles(
            "Background",
            Rect::new(0.0, 0.0, width as _, height as _),
            None,
        );
        tiled_map.draw_tiles(
            "Tile Layer 1",
            Rect::new(0.0, 0.0, width as _, height as _),
            None,
        );
        // update actions of entities passing in spawn list in case they spawn new things
        for entity in entities.iter_mut() {
            entity.mut_update();
        }
        for entity in entities.iter() {
            entity.update(&mut spawn_list, &entities);
        }
        // push new entities to entities list
        while spawn_list.len() > 0 {
            entities.push(spawn_list.pop().expect("Could not unwrap."));
        }
        // draw all entities to screen
        for entity in entities.iter() {
            entity.draw()
        }

        next_frame().await
    }
}
