extern crate crayon;
extern crate conrod_crayon;
extern crate conrod_example_shared;
extern crate conrod_core;
use crayon::prelude::*;
use crayon::window::device_pixel_ratio;
use conrod_crayon::Renderer;
use conrod_example_shared::{WIN_W, WIN_H};
use std::time::SystemTime;
use conrod_core::{color,Colorable, widget, Widget,Positionable,event::{Input},Sizeable};
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
        let mut image_map: conrod_core::image::Map<TextureHandle> = conrod_core::image::Map::new();
        let rust_logo = image_map.insert(load_rust_logo());
        dbg!("l");
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
        //conrod_crayon::events::convert_event(&mut self.ui);
        self.ui.handle_event(Input::Press(conrod_core::input::Button::Mouse(conrod_core::input::state::mouse::Button::Left)));
        {
            let mut ui = &mut self.ui.set_widgets();
            
            const LOGO_SIDE: conrod_core::Scalar = 306.0;
            
            widget::Image::new(self.app.rust_logo)
                .w_h(LOGO_SIDE, LOGO_SIDE)
                .middle()
                .set(self.ids.rust_logo, ui);
            /*   
            widget::Rectangle::fill_with([80.0, 80.0],color::ORANGE)
                .middle()
                .set(self.ids.rust_logo, ui);
            */
        }
        
        let dpi_factor = device_pixel_ratio() as f64;
        //let dpi_factor  =1.16;
        let primitives = self.ui.draw();
        let dims = (WIN_W as f64 * dpi_factor, WIN_H as f64 * dpi_factor);
        self.renderer.fill(dims,dpi_factor as f64,primitives,&self.image_map);
        self.renderer.draw(&mut self.batch,&self.image_map);
        
        Ok(())
    }
}
fn load_rust_logo() -> TextureHandle {
    video::create_texture_from("res:crate.bmp").unwrap()
}
main!({
     #[cfg(not(target_arch = "wasm32"))]
    let res = format!("file://{}/../../assets/crayon/resources/", env!("CARGO_MANIFEST_DIR"));
    #[cfg(target_arch = "wasm32")]
    let res = format!("http://localhost:8080/resources/");
    let mut params = Params::default();
    params.window.title = "CR: RenderTexture".into();
    params.window.size = (464, 434).into();
    params.res.shortcuts.add("res:", res).unwrap();
    params.res.dirs.push("res:".into());
    crayon::application::setup(params, Window::build).unwrap();
});