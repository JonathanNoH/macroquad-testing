use macroquad::prelude::*;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::hash::{Hash, Hasher};

use macroquad_tiled as tiled;

// unique ids
static COUNTER: AtomicUsize = AtomicUsize::new(1);
fn get_id() -> usize { COUNTER.fetch_add(1, Ordering::Relaxed) }


struct Player {
    speed: Vec2,
    texture: Texture2D,
    pos: Vec2,
    facing_right: bool,
    health: f32,
    id: usize,
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
            id: get_id(),
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
    id: usize,
}
impl Hash for Tower {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.cost.hash(state);
    }
}
impl PartialEq for Tower {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for Tower {}

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
            id: get_id(),
        }
    }
    fn check_shoot(&self, spawn_list: &mut Vec::<Box<dyn Entity>>, entities: &HashMap<usize, Box<dyn Entity>>) {
        if is_key_down(KeyCode::Space) {
            for (id, entity) in entities {
                if entity.get_position().distance(self.pos) < 50. {
                    let velocity = entity.get_position() - self.pos;
                    let bullet = Bullet::new(self.bullet_texture.clone(), self.pos, velocity.normalize_or_zero());
                    spawn_list.push(Box::new(bullet));
                    return;
                }
            }
        }
    }
}


struct Bullet {
    texture: Texture2D,
    pos: Vec2,
    velocity: Vec2,
    damage: f32,
    id: usize,
}
impl Hash for Bullet {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
impl PartialEq for Bullet {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for Bullet {}

impl Bullet {
    fn new(texture: Texture2D, pos: Vec2, velocity: Vec2) -> Self {
        Self {
            texture,
            pos,
            velocity,
            damage: 10.,
            id: get_id(),
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
    fn id(&self) -> usize { self.id }
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
    fn update(&self, _: &mut Vec::<Box<dyn Entity>>, _: &HashMap<usize, Box<dyn Entity>>) {
        return
    }
    fn draw(&self);
    fn id(&self) -> usize;
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
    fn update(&self, spawn_list: &mut Vec::<Box<dyn Entity>>, entities: &HashMap<usize, Box<dyn Entity>>) {
        Self::check_shoot(&self, spawn_list, &entities);
    }
    fn id(&self) -> usize { self.id }
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
    fn id(&self) -> usize { self.id }
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
//    let mut entities: Vec<Box<dyn Entity>> = vec![];
    let mut entities: HashMap<usize, Box<dyn Entity>> = HashMap::new();
    let mut spawn_list: Vec<Box<dyn Entity>> = vec![];

    let player = load_texture("assets/disciple-45x51.png").await.unwrap();
    let player_pos = vec2(240., 160.);
    let mut player = Player::new(player_pos, player, 100.);
//    entities.push(Box::new(player));
    
    let tower_texture = load_texture("assets/basictiles.png").await.unwrap();
    let bullet_texture = load_texture("assets/smallbullet.png").await.unwrap();
    let mut tower = Tower::new(30, Weapon::Gun, vec2(160., 160.), tower_texture, 5., 500., bullet_texture);
//    entities.push(Box::new(tower));
    entities.insert(tower.id(), Box::new(tower));

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
        for (id, entity) in &mut entities {
            entity.mut_update();
        }
        for (id, entity) in &entities {
            entity.update(&mut spawn_list, &entities);
        }
        // push new entities to entities list
        while spawn_list.len() > 0 {
            let to_spawn = spawn_list.pop().expect("Entity to spawn disappeared");
            entities.insert(to_spawn.id(), to_spawn);
        }
        // draw all entities to screen
        for (id, entity) in &entities {
            entity.draw()
        }
        // update and draw player
        player.mut_update();
        player.update(&mut spawn_list, &entities);
        player.draw();

        next_frame().await
    }
}
