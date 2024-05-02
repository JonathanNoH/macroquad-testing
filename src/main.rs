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
    shot_cooldown: f64,
    last_shot_time: f64,
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
            shot_cooldown: 1.,
            last_shot_time: 1.,
        }
    }
    fn check_shoot(&mut self, projectiles: &mut HashMap<usize, Box<dyn Entity>>, monsters: &HashMap<usize, Box<dyn Entity>>) {
        if get_time() - self.last_shot_time > self.shot_cooldown {
            for (id, monster) in monsters {
                if monster.get_position().distance(self.pos) < 150. {
                    let velocity = monster.get_position() - self.pos;
                    let bullet = Bullet::new(self.bullet_texture.clone(), self.pos, velocity.normalize_or_zero(), self.strength);
                    projectiles.insert(bullet.id(), Box::new(bullet));
                    self.last_shot_time = get_time();
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
    fn new(texture: Texture2D, pos: Vec2, velocity: Vec2, damage: f32) -> Self {
        Self {
            texture,
            pos,
            velocity,
            damage,
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
        self.pos += self.velocity;
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

trait TowerType: Position {
    fn mut_update(&mut self, _: &mut HashMap<usize, Box<dyn Entity>>, _: &HashMap<usize, Box<dyn Entity>>);
    fn draw(&self);
    fn id(&self) -> usize;
}

impl TowerType for Tower {
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
    fn mut_update(&mut self, projectiles: &mut HashMap<usize, Box<dyn Entity>> , monsters: &HashMap<usize, Box<dyn Entity>>) {
        Self::check_shoot(self, projectiles, monsters);
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

struct EyeMonster {
    health: f32,
    velocity: Vec2,
    id: usize,
    texture: Texture2D,
    pos: Vec2,
    facing_right: bool,
}
impl EyeMonster {
    fn new(health: f32, texture: Texture2D, pos: Vec2, velocity: Vec2) -> Self {
        Self {
            health,
            texture,
            pos,
            velocity,
            id: get_id(),
            facing_right: velocity.x > 0.,
        }
    }
}

impl Position for EyeMonster {
    fn get_position(&self) -> Vec2 { self.pos }
}

impl Entity for EyeMonster {
    fn id(&self) -> usize { self.id }
    fn draw(&self) {
        draw_texture_ex(
            &self.texture,
            self.pos.x,
            self.pos.y,
            WHITE,
            DrawTextureParams {
                flip_x: !self.facing_right,
                ..Default::default()
            },
        );
    }
    fn mut_update(&mut self) {
        self.pos += self.velocity;
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
//    let mut entities: Vec<Box<dyn Entity>> = vec![];
    let mut towers: HashMap<usize, Box<dyn TowerType>> = HashMap::new();
    let mut spawn_list: Vec<Box<dyn Entity>> = vec![];
    let mut monsters: HashMap<usize, Box<dyn Entity>> = HashMap::new();
    let mut projectiles: HashMap<usize, Box<dyn Entity>> = HashMap::new();

    let player = load_texture("assets/disciple-45x51.png").await.unwrap();
    let player_pos = vec2(240., 160.);
    let mut player = Player::new(player_pos, player, 100.);
//    entities.push(Box::new(player));
    
    let tower_texture = load_texture("assets/basictiles.png").await.unwrap();
    let bullet_texture = load_texture("assets/smallbullet.png").await.unwrap();
    let mut tower = Tower::new(30, Weapon::Gun, vec2(160., 160.), tower_texture, 5., 500., bullet_texture);
//    entities.push(Box::new(tower));
    towers.insert(tower.id(), Box::new(tower));
    let monster_texture = load_texture("assets/eyemonster.png").await.unwrap();

    let width = tiled_map.raw_tiled_map.tilewidth * tiled_map.raw_tiled_map.width;
    let height = tiled_map.raw_tiled_map.tileheight * tiled_map.raw_tiled_map.height;

    let mut monster_timer_prev = get_time();
    let mut monster_timer_curr = get_time();
    
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
        for (_id, tower) in &mut towers {
            tower.mut_update(&mut projectiles, &monsters);
        }
        // check to spawn monster
        monster_timer_curr = get_time();
        if monster_timer_curr - monster_timer_prev > 6. {
            let monster = EyeMonster::new(100., monster_texture.clone(), vec2(0.,0.), vec2(0.5,0.5));
            monsters.insert(monster.id(), Box::new(monster));
            monster_timer_prev = monster_timer_curr;
        }

        // push new entities to entities list
        //while spawn_list.len() > 0 {
        //    let to_spawn = spawn_list.pop().expect("Entity to spawn disappeared");
        //    projectiles.insert(to_spawn.id(), to_spawn);
        //}
        // draw all towers to screen
        for (_id, tower) in &towers {
            tower.draw();
        }
        // remove projectiles out of bounds
        projectiles.retain(|&key, value| {
                                let pos = value.get_position();
                                pos.x > 0. && pos.x < width as _ && pos.y > 0. && pos.y < height as _
                            });
        // draw projectiles
        for (_id, projectile) in &mut projectiles {
            projectile.mut_update();
            projectile.draw();
        }
        // draw monstes
        for (_id, monster) in &mut monsters {
            monster.mut_update();
            monster.draw();
        }
        // update and draw player
        player.mut_update();
        player.draw();

        next_frame().await
    }
}
