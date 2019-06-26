extern crate crayon;
extern crate conrod_crayon;
extern crate conrod_example_shared;
#[macro_use] extern crate conrod_core;
extern crate crayon_bytes;

use crayon::prelude::*;
use crayon_bytes::prelude::*;
use crayon::window::device_pixel_ratio;
use conrod_crayon::Renderer;
use conrod_example_shared::{WIN_W, WIN_H};
use std::collections::HashMap;
use conrod_core::{color,Colorable, widget, Widget,Positionable,event::{Input},Sizeable,Labelable};
use conrod_core::text::{Font,FontCollection};
#[derive(Debug, Clone, Copy)]
struct WindowResources {
    b: BytesHandle,
}

impl WindowResources {
    pub fn new() -> CrResult<Self> {
        crayon_bytes::setup()?;
        Ok(WindowResources {
            b: crayon_bytes::create_bytes_from("res:Oswald-Heavy.ttf")?,
        })
    }
}
impl LatchProbe for WindowResources {
    fn is_set(&self) -> bool {
        crayon_bytes::state(self.b) != ResourceState::NotReady
    }
}
widget_ids!(struct Ids { text,canvas,scrollbar });
struct Window {
    text:String,
    renderer: Renderer,
    app: conrod_example_shared::DemoApp,
    ui: conrod_core::Ui,
    ids: Ids,
    image_map: conrod_core::image::Map<TextureHandle>,
    batch: CommandBuffer,
    time: f32,
    resources: WindowResources
}
//crayon_bytes = { git = "https://github.com/alanpoon/crayon.git", branch ="textedit"}
//crayon = { git = "https://github.com/alanpoon/crayon.git", branch ="textedit"}
impl Window {
    pub fn build(resources: &WindowResources) -> CrResult<Self> {
        
        let mut ui = conrod_core::UiBuilder::new([WIN_W as f64, WIN_H as f64])
           // .theme(conrod_example_shared::theme())
            .build();
        //let ids = conrod_example_shared::Ids::new(ui.widget_id_generator());
        
        let ids = Ids::new(ui.widget_id_generator());
        let mut image_map: conrod_core::image::Map<TextureHandle> = conrod_core::image::Map::new();
        let rust_logo = image_map.insert(load_rust_logo());
        dbg!("l");
        // Demonstration app state that we'll control with our conrod GUI.
        let app = conrod_example_shared::DemoApp::new(rust_logo);
        let dpi_factor = device_pixel_ratio();
        println!("dpi {:?}",dpi_factor);
        let renderer = conrod_crayon::Renderer::new((WIN_W as f64,WIN_H as f64),  dpi_factor as f64);
        let f = ui.fonts.insert(load_bold(resources.b));
        ui.theme.font_id = Some(f);
        let mut demo_text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. \
        Mauris aliquet porttitor tellus vel euismod. Integer lobortis volutpat bibendum. Nulla \
        finibus odio nec elit condimentum, rhoncus fermentum purus lacinia. Interdum et malesuada \
        fames ac ante ipsum primis in faucibus. Cras rhoncus nisi nec dolor bibendum pellentesque. \
        Cum sociis natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. \
        Quisque commodo nibh hendrerit nunc sollicitudin sodales. Cras vitae tempus ipsum. Nam \
        magna est, efficitur suscipit dolor eu, consectetur consectetur urna.".to_owned();
        Ok(Window {
            app:app,
            text: demo_text,
            ui:ui,
            ids:ids,
            image_map:image_map,
            renderer:renderer,
            batch: CommandBuffer::new(),
            time: 0.0,
            resources: *resources
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
        conrod_crayon::events::convert_event(&mut self.ui);
        {
            let mut ui = &mut self.ui.set_widgets();
            
            const LOGO_SIDE: conrod_core::Scalar = 306.0;
            /*
            widget::Image::new(self.app.rust_logo)
                .w_h(LOGO_SIDE, LOGO_SIDE)
                .middle()
                .set(self.ids.canvas, ui);
            */
            widget::Canvas::new()
                .scroll_kids_vertically()
                .color(color::BLUE)
                .set(self.ids.canvas, ui);
            
            
            for edit in widget::TextEdit::new(&self.text)
                .color(color::WHITE)
                .font_size(20)
                .padded_w_of(self.ids.canvas, 20.0)
                .mid_top_of(self.ids.canvas)
                .center_justify()
                .line_spacing(2.5)
                .set(self.ids.text,ui){
                    println!("aa{:?}",edit.clone());
                    self.text = edit;
                    
                }
            
            //widget::Scrollbar::y_axis(self.ids.canvas).auto_hide(true).set(self.ids.scrollbar, ui);
            /*
            widget::Rectangle::fill_with([80.0, 80.0],color::ORANGE)
                .middle()
                .set(self.ids.text, ui);
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
fn load_bold(handle:BytesHandle) ->Font{
    FontCollection::from_bytes(crayon_bytes::create_bytes(handle).unwrap()).unwrap().into_font().unwrap()
}
main!({
    #[cfg(not(target_arch = "wasm32"))]
    let res = format!("file://{}/../../assets/crayon/resources/", env!("CARGO_MANIFEST_DIR").replace("\\","/"));
    #[cfg(target_arch = "wasm32")]
    let res = format!("http://localhost:8080/resources/");
    let mut params = Params::default();
    params.window.title = "CR: RenderTexture".into();
    params.window.size = (WIN_W as u32, WIN_H as u32).into();
    params.res.shortcuts.add("res:", res).unwrap();
    params.res.dirs.push("res:".into());
    crayon::application::setup(params,|| {
        let resources = WindowResources::new()?;
        Ok(Launcher::new(resources, |r| Window::build(r)))
    }).unwrap();
});