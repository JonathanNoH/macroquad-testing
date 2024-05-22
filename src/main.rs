use macroquad::prelude::*;
use macroquad::experimental::collections::storage;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::cell::RefCell;

use macroquad_tiled as tiled;

struct Resources {
    player_texture: Texture2D,
    tower_texture: Texture2D,
    eye_monster_texture: Texture2D,
    tower_destroyer_texture: Texture2D,
    bullet_texture: Texture2D
}
static RED_HEALTH_BAR: DrawRectangleParams =
    DrawRectangleParams {
        color: Color{ r:0.82,
        b: 0.122,
        g: 0.051,
        a: 1. },
        rotation: 0.,
        offset: vec2(0.,0.)
    };
static GREEN_HEALTH_BAR: DrawRectangleParams = DrawRectangleParams {
    color: Color{
        r: 0.129,
        g: 0.859,
        b: 0.18,
        a: 1.,
    },
    rotation: 0.,
    offset: vec2(0., 0.)
};
static EYE_MONSTER_SPEED: f32 = 1.;
static TOWER_DESTROYER_SPEED: f32 = 0.8;

// unique ids
static COUNTER: AtomicUsize = AtomicUsize::new(1);
fn get_id() -> usize { COUNTER.fetch_add(1, Ordering::Relaxed) }

struct Tower {
    cost: i32,
    pos: Vec2,
    strength: f32,
    health: f32,
    max_health: f32,
    id: usize,
    shot_cooldown: f64,
    last_shot_time: f64,
    hitbox: Rect,
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
    fn new(cost: i32, pos: Vec2, strength: f32, health: f32) -> Self {
        Self {
            cost,
            pos,
            strength,
            health,
            max_health: health,
            id: get_id(),
            shot_cooldown: 1.,
            last_shot_time: 1.,
            hitbox: Rect::new(pos.x, pos.y, 16., 16.),
        }
    }
    fn check_shoot(&mut self, projectiles: &mut HashMap<usize, Rc<RefCell<dyn Projectile>>>, monsters: &HashMap<usize, Rc<RefCell<dyn Monster>>>, entities: &Vec<Rc<RefCell<dyn Entity>>>) {
        if get_time() - self.last_shot_time > self.shot_cooldown {
            for (_id, monster_ref) in monsters {
                let monster = monster_ref.borrow();
                if monster.get_position().distance(self.pos) < 150. {
                    let velocity = monster.get_position() - self.pos;
                    let bullet = Rc::new(RefCell::new(Bullet::new(self.pos, velocity.normalize_or_zero(), self.strength)));
                    projectiles.insert(bullet.id(), Rc::clone(&bullet));
                    entities.push(Rc::clone(&bullet));
                    self.last_shot_time = get_time();
                    return;
                }
            }
        }
    }
}

trait TowerType: Position {
    fn mut_update(&mut self, _: &mut HashMap<usize, Rc<RefCell<dyn Projectile>>>, _: &HashMap<usize, Rc<RefCell<dyn Monster>>>, _: &HashMap<usize, Rc<RefCell<TowerDestroyer>>>);
    fn id(&self) -> usize;
    fn hitbox(&self) -> Rect;
}

impl Entity for Tower {
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
        if self.health / self.max_health < 0.9999 {
            draw_health_bar(self.pos, 16., self.health, self.max_health);
        }
    }
}

impl TowerType for Tower {
    fn mut_update(&mut self, projectiles: &mut HashMap<usize, Rc<RefCell<dyn Projectile>>> , monsters: &HashMap<usize, Rc<RefCell<dyn Monster>>>, tower_destroyers: &HashMap<usize, Rc<RefCell<TowerDestroyer>>>, entities: &Vec<Rc<RefCell<dyn Entity>>>) {
        Self::check_shoot(self, projectiles, monsters, entities);
        for (_, tower_destroyer_ref) in tower_destroyers {
            let tower_destroyer = tower_destroyer_ref.borrow_mut();
            if tower_destroyer.hitbox().overlaps(&self.hitbox()) {
                self.health -= tower_destroyer.damage();
            }
        }
    }

    fn id(&self) -> usize { self.id }
    fn hitbox(&self) -> Rect { self.hitbox }
}
impl Position for Tower {
    fn get_position(&self) -> Vec2 {
        self.pos
    }
}


struct Bullet {
    pos: Vec2,
    velocity: Vec2,
    damage: f32,
    id: usize,
    hitbox: Rect,
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
            hitbox: Rect::new(pos.x,pos.y,18.,16.),
        }
    }

}

impl Position for Bullet {
    fn get_position(&self) -> Vec2 {
        self.pos
    }
}

trait Projectile: Position {
    fn id(&self) -> usize;
    fn hitbox(&self) -> Rect;
    fn mut_update(&mut self);
    fn damage(&self) -> f32;
}
impl Entity for Bullet {
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
}

impl Projectile for Bullet {
    fn mut_update(&mut self) {
        self.pos += self.velocity;
        self.hitbox.move_to(self.pos);
    }
    fn id(&self) -> usize { self.id }
    fn hitbox(&self) -> Rect {
        self.hitbox
    }
    fn damage(&self) -> f32 { self.damage }
}

trait Health {
    fn take_damage(&mut self, amount: f32);
}

fn draw_health_bar(pos: Vec2, width: f32, health: f32, max_health: f32) {
    draw_rectangle_ex(pos.x, pos.y-8., width, 3., RED_HEALTH_BAR.clone());
    if health > 0. {
        draw_rectangle_ex(pos.x, pos.y-8., width*(health/max_health), 3., GREEN_HEALTH_BAR.clone());
    }
}

trait Position {
    fn get_position(&self) -> Vec2;
}

trait Targetable: Health + Position {}

trait Entity: Position {
    fn draw(&self);
}

struct Player {
    speed: Vec2,
    pos: Vec2,
    facing_right: bool,
    health: f32,
    max_health: f32,
    id: usize,
    hitbox: Rect,
}


impl Player {
    pub const MOVE_SPEED: f32 = 5.0;
    fn new(pos: Vec2, health: f32) -> Self {
        Self {
            speed: vec2(0., 0.),
            pos,
            facing_right: true,
            health,
            max_health: health,
            id: get_id(),
            hitbox: Rect::new(pos.x+5.,pos.y+5.,35.,41.,),
        }
    }
}



impl Position for Player {
    fn get_position(&self) -> Vec2 {
        self.pos
    }
}

impl Health for Player {
    fn take_damage(&mut self, amount: f32) {
        if self.health > 0. {
            self.health -= amount;
        }
        if self.health < 0. {
            self.health = 0.;
        }
    }
}

impl Entity for Player {
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
        draw_health_bar(self.pos, 45., self.health, self.max_health);
    }
}
impl Player {

    fn mut_update(&mut self, towers: &mut HashMap<usize, Rc<RefCell<dyn TowerType>>>) {
        // apply movement
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
        self.hitbox = Rect::new(self.pos.x+5., self.pos.y+5., 35., 41.);
        // place towers
        if is_key_pressed(KeyCode::Space) {
            let tower = Rc::new(RefCell::new(Tower::new(30, self.pos, 5., 500.)));
            towers.insert(tower.borrow().id(), Rc::clone(&tower));
        }
    }
    fn id(&self) -> usize { self.id }
    fn hitbox(&self) -> Rect { self.hitbox }
}

struct EyeMonster {
    health: f32,
    max_health: f32,
    velocity: Vec2,
    id: usize,
    pos: Vec2,
    facing_right: bool,
    hitbox: Rect,
    damage: f32,
    damage_cd: f64,
    last_damage_time: f64,
}
impl EyeMonster {
    fn new(health: f32, pos: Vec2, velocity: Vec2) -> Self {
        Self {
            health,
            max_health: health,
            pos,
            velocity,
            id: get_id(),
            facing_right: velocity.x > 0.,
            hitbox: Rect::new(pos.x, pos.y, 60., 54.),
            damage: 5.,
            damage_cd: 1.,
            last_damage_time: 0.,
        }
    }

}

trait Monster: Position {
    fn id(&self) -> usize;
    fn mut_update(&mut self, _: &mut HashMap<usize, Rc<RefCell<dyn Projectile>>>, _: &HashMap<usize, Rc<RefCell<dyn TowerType>>>, _: Vec2);
    fn health(&self) -> f32;
    fn damage(&self) -> f32;
    fn damage_cd(&self) -> f64;
    fn last_damage_time(&self) -> f64;
    fn set_last_damage_time(&mut self);
    fn hitbox(&self) -> Rect;
}
impl Entity for EyeMonster {
    fn draw(&self) {
        let resources = storage::get::<Resources>();
        draw_texture_ex(
            &resources.eye_monster_texture,
            self.pos.x,
            self.pos.y,
            WHITE,
            DrawTextureParams {
                flip_x: !self.facing_right,
                ..Default::default()
            },
        );
        draw_health_bar(self.pos, 60., self.health, self.max_health);
    }
}
impl Monster for EyeMonster {
    fn id(&self) -> usize { self.id }
    fn mut_update(&mut self, projectiles: &mut HashMap<usize, Rc<RefCell<dyn Projectile>>>, _: &HashMap<usize, Rc<RefCell<dyn TowerType>>>, target: Vec2) {
        let distance_vector = vec2(target.x - self.pos.x, target.y - self.pos.y);
        self.velocity = distance_vector.normalize_or_zero()*EYE_MONSTER_SPEED;
        self.pos += self.velocity;
        self.hitbox.move_to(self.pos);
        projectiles.retain(|_, projectile_ref| {
            let projectile = projectile_ref.borrow();
            let mut retain = true;
            if self.hitbox().overlaps(&projectile.hitbox()) {
                self.health -= projectile.damage();
                retain = false;
            }
            retain
        });
    }
    fn health(&self) -> f32 { self.health }
    fn damage(&self) -> f32 { self.damage }
    fn damage_cd(&self) -> f64 { self.damage_cd }
    fn last_damage_time(&self) -> f64 {self.last_damage_time}
    fn set_last_damage_time(&mut self) {
        self.last_damage_time = get_time();
    }
    fn hitbox(&self) -> Rect {
        self.hitbox
    }
}

impl Position for EyeMonster {
    fn get_position(&self) -> Vec2 { self.pos }
}

struct TowerDestroyer {
    health: f32,
    max_health: f32,
    velocity: Vec2,
    id: usize,
    pos: Vec2,
    facing_right: bool,
    hitbox: Rect,
    damage: f32,
    damage_cd: f64,
    last_damage_time: f64,
    target: usize
}
impl TowerDestroyer {
    fn new(health: f32, pos: Vec2, velocity: Vec2, target: usize) -> Self {
        Self {
            health,
            max_health: health,
            pos,
            velocity,
            id: get_id(),
            facing_right: velocity.x > 0.,
            hitbox: Rect::new(pos.x, pos.y, 51., 54.),
            damage: 10.,
            damage_cd: 1.5,
            last_damage_time: 0.,
            target,
        }
    }
}
impl Entity for TowerDestroyer {
    fn draw(&self) {
        let resources = storage::get::<Resources>();
        draw_texture_ex(
            &resources.tower_destroyer_texture,
            self.pos.x,
            self.pos.y,
            WHITE,
            DrawTextureParams {
                flip_x: !self.facing_right,
                ..Default::default()
            },
        );
        draw_health_bar(self.pos, 60., self.health, self.max_health);
    }
}
impl Monster for TowerDestroyer {
    fn id(&self) -> usize { self.id }

    // modifies projectiles
    fn mut_update(&mut self, projectiles: &mut HashMap<usize, Rc<RefCell<dyn Projectile>>>, towers: &HashMap<usize, Rc<RefCell<dyn TowerType>>>, player_target: Vec2) {
        let target_tower = towers.get(&self.target);
        let target = match target_tower {
            Some(tower) => tower.borrow().get_position(),
            None => player_target
        };
        let distance_vector = vec2(target.x - self.pos.x, target.y - self.pos.y);
        self.velocity = distance_vector.normalize_or_zero()*EYE_MONSTER_SPEED;
        self.pos += self.velocity;
        self.hitbox.move_to(self.pos);
        projectiles.retain(|_, projectile_ref| {
            let projectile = projectile_ref.borrow();
            let mut retain = true;
            if self.hitbox().overlaps(&projectile.hitbox()) {
                self.health -= projectile.damage();
                retain = false;
            }
            retain
        });
    }
    fn health(&self) -> f32 { self.health }
    fn damage(&self) -> f32 { self.damage }
    fn damage_cd(&self) -> f64 { self.damage_cd }
    fn last_damage_time(&self) -> f64 {self.last_damage_time}
    fn set_last_damage_time(&mut self) {
        self.last_damage_time = get_time();
    }
    fn hitbox(&self) -> Rect {
        self.hitbox
    }
}
impl Position for TowerDestroyer {
    fn get_position(&self) -> Vec2 { self.pos }
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
    let mut entities: Vec<Rc<RefCell<dyn Entity>>> = vec![];
    let mut towers: HashMap<usize, Rc<RefCell<dyn TowerType>>> = HashMap::new();
    let mut monsters: HashMap<usize, Rc<RefCell <dyn Monster>>> = HashMap::new();
    let mut projectiles: HashMap<usize, Rc<RefCell <dyn Projectile>>> = HashMap::new();
    let mut tower_destroyers: HashMap<usize, Rc<RefCell<TowerDestroyer>>> = HashMap::new();


    let player_texture = load_texture("assets/disciple-45x51.png").await.unwrap();
    let tower_texture = load_texture("assets/basictiles.png").await.unwrap();
    let bullet_texture = load_texture("assets/smallbullet.png").await.unwrap();
    let eye_monster_texture = load_texture("assets/eyemonster.png").await.unwrap();
    let tower_destroyer_texture = load_texture("assets/towerdestroyer.png").await.unwrap();

    let resources = Resources {
        player_texture,
        tower_texture,
        bullet_texture,
        eye_monster_texture,
        tower_destroyer_texture,
    };
    storage::store(resources);


    let player_pos = vec2(240., 160.);
    let mut player = Player::new(player_pos, 100.);

    let tower = Rc::new(RefCell::new(Tower::new(30, vec2(160., 160.), 5., 500.)));
    entities.push(Rc::clone(&tower));
    towers.insert(tower.id(), Rc::clone(&tower));

    let width = tiled_map.raw_tiled_map.tilewidth * tiled_map.raw_tiled_map.width;
    let height = tiled_map.raw_tiled_map.tileheight * tiled_map.raw_tiled_map.height;

    let mut eye_monster_timer_prev = get_time();
    let mut tower_destroyer_timer = get_time();

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
            tower.mut_update(&mut projectiles, &monsters, &tower_destroyers, &mut entities);
        }
        // check to spawn monster
        let timer_curr = get_time();
        // spawn eye monster
        if timer_curr - eye_monster_timer_prev > 6. {
            let monster = Rc::new(EyeMonster::new(
                100.,
                vec2(0.,0.),
                player.get_position().normalize_or_zero()*EYE_MONSTER_SPEED
            ));
            entities.push(Rc::clone(&monster));
            monsters.insert(monster.id(), Rc::clone(&monster));
            eye_monster_timer_prev = timer_curr;
        }
        // spawn tower destroyer
        if timer_curr - tower_destroyer_timer > 10. {
            let new_monster_position = vec2(0.,0.);
            let random_tower = towers.values().next().borrow();
            let mut smallest_distance = match random_tower {
                Some(tower) => tower.get_position().distance(new_monster_position),
                None => 0.
            };
            let mut target_tower = None;
            let mut tower_id: usize = 0;
            for (id, tower) in &towers {
                if smallest_distance > new_monster_position.distance(tower.get_position()) {
                    smallest_distance = new_monster_position.distance(tower.get_position());
                    target_tower = Some(id);
                }
            }
            let velocity;
            match target_tower {
                None => {
                    velocity = vec2(player.get_position().x - new_monster_position.x, player.get_position().y - new_monster_position.y);
                }
                Some(id) => {
                    let tower = towers.get(id).unwrap();
                    velocity = vec2(tower.get_position().x - new_monster_position.x, tower.get_position().y - new_monster_position.y);
                    tower_id = id.clone();
                }
            }
            let monster = Rc::new(TowerDestroyer::new(
                60.,
                new_monster_position,
                velocity.normalize_or_zero()*TOWER_DESTROYER_SPEED,
                tower_id,
            ));
            entities.push(Rc::clone(&monster));
            monsters.insert(monster.id(), Rc::clone(&monster));
            tower_destroyers.insert(monster.id(), Rc::clone(&monster));
            tower_destroyer_timer = timer_curr;
        }


        // remove projectiles out of bounds
        projectiles.retain(|_, projectile| {
                                let pos = projectile.borrow().get_position();
                                pos.x > 0. && pos.x < width as _ && pos.y > 0. && pos.y < height as _
                            });
        // despawn monsters when far enough from player
        // or when monster is dead (health < 0)
        let player_pos = player.get_position();
        monsters.retain(|_, monster_ref| {
            let monster = monster_ref.borrow();
            let mut retain = true;
            let pos = monster.get_position();
            if pos.distance(player_pos) > 2000. {
                retain = false;
            }
            if monster.health() < 0. {
                retain = false
            }
            retain
        });
        // update movement and check damage
        let player_pos = player.get_position();
        let player_hitbox = player.hitbox();
        for (_id, ref_monster) in &mut monsters {
            let mut monster = ref_monster.borrow_mut();
            monster.mut_update(&mut projectiles, &towers, player_pos);
            if monster.hitbox().overlaps(&player_hitbox) {
                if timer_curr - monster.last_damage_time() > monster.damage_cd() {
                    player.take_damage(monster.damage());
                    monster.set_last_damage_time();
                }
            }
        }

        // update player
        player.mut_update(&mut towers);

        for entity in entities {
            entity.borrow().draw();
        }

        next_frame().await
    }
}
