//! The simplest possible example that does something.
extern crate conrod_core;
extern crate conrod_example_shared;
extern crate conrod_ggez;

extern crate gfx;
extern crate gfx_core;

use conrod_ggez::{ggez,map_key};
use ggez::conf::{WindowMode};
use ggez::graphics;
use ggez::{Context, GameResult};
use ggez::event::{self, KeyCode, KeyMods, MouseButton};
use std::time::SystemTime;
use conrod_example_shared::{WIN_W, WIN_H};

use gfx::Device;

const CLEAR_COLOR: [f32; 4] = [0.2, 0.2, 0.2, 1.0];

struct MainState <'a> {
    pos_x: f32,
    app: conrod_example_shared::DemoApp,
    ui: conrod_core::Ui,
    ids: conrod_example_shared::Ids,
    image_map: conrod_core::image::Map<(gfx::handle::ShaderResourceView<gfx_device_gl::Resources, [f32; 4]>, (u32, u32))>,
    renderer: conrod_ggez::Renderer<'a>
}

impl<'a> MainState<'a> {
    fn new(ctx: &mut Context) -> GameResult<Self> {
         // Create Ui and Ids of widgets to instantiate
        let dpi_factor = graphics::hidpi_factor(ctx) as f64;
        let mut ui = conrod_core::UiBuilder::new([WIN_W as f64, WIN_H as f64])
            .theme(conrod_example_shared::theme())
            .build();
        let ids = conrod_example_shared::Ids::new(ui.widget_id_generator());

        // Load font from file
        let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
        let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
        ui.fonts.insert_from_file(font_path).unwrap();
        let rtv = graphics::screen_render_target(ctx);
        
        let factory = graphics::factory(ctx);
        let mut image_map: conrod_core::image::Map<_> = conrod_core::image::Map::new();
        let rust_logo = image_map.insert(load_rust_logo::<conrod_ggez::ColorFormat,_>(factory));
        
        // Demonstration app state that we'll control with our conrod GUI.
        let app = conrod_example_shared::DemoApp::new(rust_logo);
        let renderer = conrod_ggez::Renderer::new(factory, &rtv,  dpi_factor).unwrap();
        let s = MainState { pos_x: 0.0,app:app,ui:ui,ids:ids,image_map:image_map,renderer:renderer};
        Ok(s)
    }
}

impl<'a> event::EventHandler for MainState<'a> {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        self.pos_x = self.pos_x % 800.0 + 1.0;
        let mut ui = self.ui.set_widgets();
        conrod_example_shared::gui(&mut ui, &self.ids, &mut self.app);
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let dpi_factor  =graphics::hidpi_factor(ctx);
        //let dpi_factor  =1.16;
    
        if let Some(primitives) = self.ui.draw_if_changed() {
            {
                println!("get_hidpi_factor {:?} {:?}",dpi_factor,SystemTime::now());
                let (factory, device, mut encoder, _depthview, _colorview) = graphics::gfx_objects(ctx);
                self.renderer.clear(&mut encoder, CLEAR_COLOR);
                let dims = (WIN_W as f32 * dpi_factor, WIN_H as f32 * dpi_factor);

                //Clear the window
                self.renderer.clear(&mut encoder, CLEAR_COLOR);

                self.renderer.fill(&mut encoder,dims,dpi_factor as f64,primitives,&self.image_map);

                self.renderer.draw(factory,&mut encoder,&self.image_map);
                
            }
            graphics::present(ctx)?;
        }
        Ok(())
    }
    fn mouse_button_down_event(&mut self, _ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        conrod_ggez::event::mouse_button_down_event(&mut self.ui,button);
    }

    fn mouse_button_up_event(&mut self, _ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        conrod_ggez::event::mouse_button_up_event(&mut self.ui,button);
    }

    fn mouse_motion_event(&mut self, _ctx: &mut Context, x: f32, y: f32, _xrel: f32, _yrel: f32) {
        conrod_ggez::event::mouse_motion_event(&mut self.ui,x,y);
    }

    fn mouse_wheel_event(&mut self, _ctx: &mut Context, x: f32, y: f32) {
        conrod_ggez::event::mouse_wheel_event(&mut self.ui,x,y);
    }
    
    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        keymod: KeyMods,
        repeat: bool,
    ) {
        //conrod_ggez::event::key_down_event(&mut self.ui,keycode,keymod);
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, keymod: KeyMods) {
        //conrod_ggez::event::key_up_event(&mut self.ui,keycode,keymod);
    }

    fn focus_event(&mut self, _ctx: &mut Context, _gained: bool) {

    }
}
// Load the Rust logo from our assets folder to use as an example image.
fn load_rust_logo<T: gfx::format::TextureFormat,F: gfx::Factory<gfx_device_gl::Resources>>(factory: &mut F) -> (gfx::handle::ShaderResourceView<gfx_device_gl::Resources, <T as gfx::format::Formatted>::View>,(u32,u32)) {
    use gfx::{format, texture};
    use gfx::memory::Bind;
    use gfx::memory::Usage;
    let assets = find_folder::Search::ParentsThenKids(5, 3).for_folder("assets").unwrap();
    let path = assets.join("images/rust.png");
    let rgba_image = image::open(&std::path::Path::new(&path)).unwrap().to_rgba();
    let image_dimensions = rgba_image.dimensions();
    let kind = texture::Kind::D2(
        image_dimensions.0 as texture::Size,
        image_dimensions.1 as texture::Size,
        texture::AaMode::Single
    );
    let info = texture::Info {
        kind: kind,
        levels: 1,
        format: <T::Surface as format::SurfaceTyped>::get_surface_type(),
        bind: Bind::SHADER_RESOURCE,
        usage: Usage::Dynamic,
    };
    let raw = factory.create_texture_raw(
        info,
        Some(<T::Channel as format::ChannelTyped>::get_channel_type()),
        Some((&[rgba_image.into_raw().as_slice()], texture::Mipmap::Provided))).unwrap();
    let tex = gfx_core::memory::Typed::new(raw);
    let view = factory.view_texture_as_shader_resource::<T>(
        &tex, (0,0), format::Swizzle::new()
    ).unwrap();
    (view,image_dimensions)
}
pub fn main() -> GameResult {
    println!("w {:?}, h {:?}",WIN_W,WIN_H);
    let cb = ggez::ContextBuilder::new("super_simple", "ggez")
    .window_mode(
            WindowMode::default()
                .dimensions(WIN_W as f32, WIN_H as f32)
                .resizable(true),
        );
    let (ctx, event_loop) = &mut cb.build()?;
    let state = &mut MainState::new(ctx)?;
    event::run(ctx, event_loop, state)
}
