extern crate crayon;
extern crate conrod_crayon;
extern crate conrod_example_shared;
extern crate find_folder;
use crayon::prelude::*;
use crayon::window::device_pixel_ratio;
use conrod_crayon::Renderer;
use conrod_example_shared::{WIN_W, WIN_H};

struct Window {
    renderer: Renderer,
    app: conrod_example_shared::DemoApp,
    ui: conrod_core::Ui,
    ids: conrod_example_shared::Ids,
    image_map: conrod_core::image::Map<TextureHandle>,
    batch: CommandBuffer,
    time: f32,
}

impl Window {
    pub fn build() -> CrResult<Self> {
        
        let mut ui = conrod_core::UiBuilder::new([WIN_W as f64, WIN_H as f64])
            .theme(conrod_example_shared::theme())
            .build();
        let ids = conrod_example_shared::Ids::new(ui.widget_id_generator());
        let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
        let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
        ui.fonts.insert_from_file(font_path).unwrap();
        let mut image_map: conrod_core::image::Map<TextureHandle> = conrod_core::image::Map::new();
        let rust_logo = image_map.insert(load_rust_logo());

        // Demonstration app state that we'll control with our conrod GUI.
        let app = conrod_example_shared::DemoApp::new(rust_logo);
        let dpi_factor = device_pixel_ratio();
        let renderer = conrod_crayon::Renderer::new((WIN_W as f64,WIN_H as f64),  dpi_factor as f64);
        Ok(Window {
            app:app,
            ui:ui,
            ids:ids,
            image_map:image_map,
            renderer:renderer,
            batch: CommandBuffer::new(),
            time: 0.0,
        })
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        /*video::delete_render_texture(self.texture);

        video::delete_mesh(self.pass.mesh);
        video::delete_shader(self.pass.shader);
        video::delete_surface(self.pass.surface);

        video::delete_mesh(self.post_effect.mesh);
        video::delete_shader(self.post_effect.shader);
        video::delete_surface(self.post_effect.surface);
        */
    }
}

impl LifecycleListener for Window {
    fn on_update(&mut self) -> CrResult<()> {
        /*
        let surface = self.pass.surface;
        let dc = Draw::new(self.pass.shader, self.pass.mesh);
        self.batch.draw(dc);
        self.batch.submit(surface)?;

        let surface = self.post_effect.surface;
        let mut dc = Draw::new(self.post_effect.shader, self.post_effect.mesh);
        dc.set_uniform_variable("renderedTexture", self.texture);
        dc.set_uniform_variable("time", self.time);
        self.batch.draw(dc);
        self.batch.submit(surface)?;

        self.time += 0.05;
        */
        Ok(())
    }
}
fn load_rust_logo() -> TextureHandle {
    video::create_texture_from("res:images/rust.png").unwrap()
}
main!({
    let mut params = Params::default();
    params.window.title = "CR: RenderTexture".into();
    params.window.size = (568, 320).into();
    crayon::application::setup(params, Window::build).unwrap();
});