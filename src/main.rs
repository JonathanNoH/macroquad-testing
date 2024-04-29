use macroquad::prelude::*;

use macroquad_tiled as tiled;

struct Player {
    speed: Vec2,
    texture: Texture2D,
    pos: Vec2,
}

impl Player {
    pub const MOVE_SPEED: f32 = 5.0;
    fn new(pos: Vec2, texture: Texture2D) -> Self {
        Self {
            texture,
            speed: vec2(0., 0.),
            pos,
        }
    }
    
    fn draw(&self) {
    draw_texture_ex(
            &self.texture,
            self.pos.x,
            self.pos.y,
            WHITE,
            DrawTextureParams {
                source: Some(Rect::new(0.0, 0.0, 45., 51.)),
                ..Default::default()
            },
        );
    }
    fn update(&mut self) {
        if is_key_down(KeyCode::W) {
            self.pos.y -= Self::MOVE_SPEED;
        }
        if is_key_down(KeyCode::A) {
            self.pos.x -= Self::MOVE_SPEED;
        }
        if is_key_down(KeyCode::S) {
            self.pos.y += Self::MOVE_SPEED;
        }
        if is_key_down(KeyCode::D) {
            self.pos.x += Self::MOVE_SPEED;
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

    let player = load_texture("assets/disciple-45x51.png").await.unwrap();

    let player_pos = vec2(240., 160.);
    let mut player = Player::new(player_pos, player);


    let width = tiled_map.raw_tiled_map.tilewidth * tiled_map.raw_tiled_map.width;
    let height = tiled_map.raw_tiled_map.tileheight * tiled_map.raw_tiled_map.height;

    loop {
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
        player.update();
        player.draw(); 
        next_frame().await
    }
}
