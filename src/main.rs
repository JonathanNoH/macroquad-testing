use macroquad::prelude::*;
use macroquad::experimental::collections::storage;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::hash::{Hash, Hasher};
/*
use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
    rc::Rc,
};
*/
struct Resources {
    player_texture: Texture2D,
    tower_texture: Texture2D,
    monster_texture: Texture2D,
    bullet_texture: Texture2D
}

use macroquad_tiled as tiled;

// unique ids
static COUNTER: AtomicUsize = AtomicUsize::new(1);
fn get_id() -> usize { COUNTER.fetch_add(1, Ordering::Relaxed) }
/*
// adapted from experimental macrosquad STORAGE
static mut TEXTURES: Option<HashMap<&str, Box<Rc<RefCell<Texture2D>>>>> = None;

fn store_texture(name: &'static str, texture: Texture2D) {
    unsafe {
        if TEXTURES.is_none() {
            TEXTURES = Some(HashMap::new());
        }

        *TEXTURES
            .as_mut()
            .unwrap()
            .insert(name, Box::new(Rc::new(RefCell::new(texture))))
            .expect("ERROR INSERTING TEXTURE")
    };
}

fn get_texture(name: &str) -> &Texture2D {
    unsafe {
        if TEXTURES.is_none() {
            TEXTURES = Some(HashMap::new());
        }
        TEXTURES
            .as_mut()
            .unwrap()
            .get(name)
            .expect("MISSING TEXTURE")
    }
}
*/

struct Player {
    speed: Vec2,
    pos: Vec2,
    facing_right: bool,
    health: f32,
    id: usize,
}


impl Player {
    pub const MOVE_SPEED: f32 = 5.0;
    fn new(pos: Vec2, health: f32) -> Self {
        Self {
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
    strength: f32,
    health: f32,
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
    fn new(cost: i32, weapon: Weapon, pos: Vec2, strength: f32, health: f32) -> Self {
        Self {
            cost,
            weapon,
            pos,
            strength,
            health,
            id: get_id(),
            shot_cooldown: 1.,
            last_shot_time: 1.,
        }
    }
    fn check_shoot(&mut self, projectiles: &mut HashMap<usize, Box<dyn Entity>>, monsters: &HashMap<usize, Box<dyn Entity>>) {
        if get_time() - self.last_shot_time > self.shot_cooldown {
            for (_id, monster) in monsters {
                if monster.get_position().distance(self.pos) < 150. {
                    let velocity = monster.get_position() - self.pos;
                    let bullet = Bullet::new(self.pos, velocity.normalize_or_zero(), self.strength);
                    projectiles.insert(bullet.id(), Box::new(bullet));
                    self.last_shot_time = get_time();
                    return;
                }
            }
        }
    }
}


struct Bullet {
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
    fn new(pos: Vec2, velocity: Vec2, damage: f32) -> Self {
        Self {
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
        let resources = storage::get::<Resources>();
        draw_texture_ex(
            &resources.bullet_texture,
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
        let resources = storage::get::<Resources>();
        draw_texture_ex(
            &resources.tower_texture,
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


impl Player {    
    fn draw(&self) {
        let resources = storage::get::<Resources>();
        draw_texture_ex(
            &resources.player_texture,
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
    fn mut_update(&mut self, towers: &mut HashMap<usize, Box<dyn TowerType>>) {
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
        if is_key_pressed(KeyCode::Space) {
            let tower = Tower::new(30, Weapon::Gun, vec2(160., 160.), 5., 500.);
            towers.insert(tower.id(), Box::new(tower));
        }
    }
    fn id(&self) -> usize { self.id }
}

struct EyeMonster {
    health: f32,
    velocity: Vec2,
    id: usize,
    pos: Vec2,
    facing_right: bool,
}
impl EyeMonster {
    fn new(health: f32, pos: Vec2, velocity: Vec2) -> Self {
        Self {
            health,
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
        let resources = storage::get::<Resources>();
        draw_texture_ex(
            &resources.monster_texture,
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
 
#[macroquad::main("TD Survive")]
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
    let mut monsters: HashMap<usize, Box<dyn Entity>> = HashMap::new();
    let mut projectiles: HashMap<usize, Box<dyn Entity>> = HashMap::new();
    
    
    let player_texture = load_texture("assets/disciple-45x51.png").await.unwrap();
    let tower_texture = load_texture("assets/basictiles.png").await.unwrap();
    let bullet_texture = load_texture("assets/smallbullet.png").await.unwrap();
    let monster_texture = load_texture("assets/eyemonster.png").await.unwrap();
    /*
    store_texture("player", player);
    store_texture("tower", tower_texture);
    store_texture("monster", monster_texture);
    store_texture("bullet", bullet_texture);
    */
    let resources = Resources {
        player_texture,
        tower_texture,
        bullet_texture,
        monster_texture
    };
    storage::store(resources);


    let player_pos = vec2(240., 160.);
    let mut player = Player::new(player_pos, 100.);
//    entities.push(Box::new(player));
    
    let tower = Tower::new(30, Weapon::Gun, vec2(160., 160.), 5., 500.);
//    entities.push(Box::new(tower));
    towers.insert(tower.id(), Box::new(tower));

    let width = tiled_map.raw_tiled_map.tilewidth * tiled_map.raw_tiled_map.width;
    let height = tiled_map.raw_tiled_map.tileheight * tiled_map.raw_tiled_map.height;

    let mut monster_timer_prev = get_time();
    
    
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
        let monster_timer_curr = get_time();
        if monster_timer_curr - monster_timer_prev > 6. {
            let monster = EyeMonster::new(100., vec2(0.,0.), vec2(0.5,0.5));
            monsters.insert(monster.id(), Box::new(monster));
            monster_timer_prev = monster_timer_curr;
        }

        // draw all towers to screen
        for (_id, tower) in &towers {
            tower.draw();
        }
        // remove projectiles out of bounds
        projectiles.retain(|_, value| {
                                let pos = value.get_position();
                                pos.x > 0. && pos.x < width as _ && pos.y > 0. && pos.y < height as _
                            });
        // draw projectiles
        for (_id, projectile) in &mut projectiles {
            projectile.mut_update();
            projectile.draw();
        }
        // despawn monsters when far enough from player
        let player_pos = player.get_position();
        monsters.retain(|_, value| {
            let pos = value.get_position();
            pos.distance(player_pos) < 2000.
        });
        // draw monsters
        for (_id, monster) in &mut monsters {
            monster.mut_update();
            monster.draw();
        }
        // update and draw player
        player.mut_update(&mut towers);
        player.draw();

        next_frame().await
    }
}
