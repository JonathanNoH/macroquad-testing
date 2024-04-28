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
        draw_texture_ex(
            &npc,
            0.,
            0.,
            WHITE,
            DrawTextureParams {
                source: Some(Rect::new(0.0, 0.0, 45., 51.)),
                ..Default::default()
            },
        );
        next_frame().await
    }
}
