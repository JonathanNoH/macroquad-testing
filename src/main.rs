use macroquad::prelude::*;

use macroquad_tiled as tiled;

#[macroquad::main("BasicShapes")]
async fn main() {
    let tileset: Texture2D = load_texture("assets/basictiles.png").await.unwrap();
    println!("loaded tileset");

    let tiled_map_json = load_string("assets/testtilemap.json").await.unwrap();
    println!("loaded json");
    let tiled_map = tiled::load_map(
        &tiled_map_json,
        &[("basictiles.png", tileset)],
        &[],
        ).unwrap();
    println!("created tiled_map");
    let npc = load_texture("assets/disciple-45x51.png").await.unwrap();
    let mut npc_pos = vec2(240., 160.);

    let width = 480.;
    let height = 320.;
    loop {
        clear_background(BLACK);
        tiled_map.draw_tiles(
            "Background",
            Rect::new(0.0, 0.0, width, height),
            None,
        );
        tiled_map.draw_tiles(
            "Tile Layer 1",
            Rect::new(0.0, 0.0, width, height),
            None,
        );
        if is_key_down(KeyCode::W) {
           npc_pos.y -= 1.0;
        }
        if is_key_down(KeyCode::A) {
            npc_pos.x -= 1.0;
        }
        if is_key_down(KeyCode::S) {
            npc_pos.y += 1.0;
        }
        if is_key_down(KeyCode::D) {
            npc_pos.x += 1.0;
        }
        draw_texture_ex(
            &npc,
            npc_pos.x,
            npc_pos.y,
            WHITE,
            DrawTextureParams {
                source: Some(Rect::new(0.0, 0.0, 45., 51.)),
                ..Default::default()
            },
        );
        next_frame().await
    }
}
