//!
//! This example shows how to use Conrod together with custom Elmesque drawing. (Conrod
//! is based on Elmesque.) As of July 28, 2015, this is broken on retina displays due
//! to https://github.com/tomaka/glutin/issues/503.
//!

extern crate piston_window;
extern crate gfx_graphics;
extern crate elmesque;
#[macro_use] extern crate conrod;
extern crate find_folder;
extern crate num;

use std::ops::DerefMut;
use num::Float;
use piston_window::{
    PistonWindow,
    UpdateEvent,
    WindowSettings
};
use conrod::{
    Colorable,
    Labelable,
    Slider,
    Widget,
};
use gfx_graphics::GlyphCache;

fn main() {
    let opengl_version = piston_window::OpenGL::V3_2;
    
    let window_settings = WindowSettings::new(
          "Control Panel",
            [1200, 800]
        )
        .opengl(opengl_version)
        .exit_on_esc(true);
    
    // PistonWindow has two type parameters, but the default type is
    // PistonWindow<T = (), W: Window = GlutinWindow>. To change the Piston backend,
    // specify a different type in the let binding, e.g.
    // let window: PistonWindow<(), Sdl2Window>.
    let window: PistonWindow = window_settings.build().unwrap();
    
    // Load a font. GlyphCache is provided by gfx_graphics. Other Piston backends provide
    // similar types.
    let assets = find_folder::Search::ParentsThenKids(3, 3)
        .for_folder("assets").unwrap();
    let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
    let glyph_cache = GlyphCache::new(
        &font_path,
        window.factory.borrow().clone()
    ).unwrap();
    
    // Conrod's main object.
    let mut ui = conrod::Ui::new(
        glyph_cache, conrod::Theme::default()
    );
    
    // Total time elapsed since the app started. We'll use this for animation.
    let mut secs = 0.0;
    
    // The current value of the Conrod slider control. We'll update this in response to
    // user input.
    let mut slider_val = 50.0;
    
    for e in window {
        // Invoke Conrod's user input handler.
        ui.handle_event(&e.event.clone().unwrap());
        
        e.draw_2d(|c, g| {
            // We'll draw in two phases. First, we'll do our custom Elmesque drawing,
            // independently of Conrod. Second, we'll draw a Conrod widget.
            
            // Custom Elmesque drawing. We limit the lifetime of the Renderer so that we
            // can reuse c and g later.
            {
                // Elmesque needs a GlypheCache instance. Above, we moved the GlypheCache
                // into the conrod::Ui, so we'll borrow the GlypheCache from there.
                // conrod::Ui wraps the character cache in a RefCell and a
                // conrod::GlypheCache. 
                let mut cc_ref_mut = ui.glyph_cache.deref_mut().borrow_mut();
                
                // Elmesque's renderer. Conrod also uses one of these under the hood, but
                // we can't access that one, so we instantiate our own.
                let mut renderer = elmesque::Renderer::new(c, g)
                    .character_cache(cc_ref_mut.deref_mut());
                
                // Create a rectangle. This call doesn't actually render anything.
                let form = elmesque::form::rect(60.0, 40.0)
                    .filled(elmesque::color::blue())
                    .shift(secs.sin() * 50.0, secs.cos() * 50.0);
                
                // Get the window dimensions.
                let view_dim = c.get_view_size();
                let (w, h) = (view_dim[0], view_dim[1]);
                
                // Render the Elmesque rectangle.
                elmesque::form::collage(w as i32, h as i32, vec![form])
                    .clear(elmesque::color::black())
                    .draw(&mut renderer);
            }
            
            // Create a Conrod slider widget.
            Slider::new(slider_val, 0.0, 500.0)
                // elmesque::color is also re-exported as conrod::color, so we could use
                // it by that name too.
                .color(elmesque::color::rgb(0.0, 0.3, 0.1))
                .label_color(elmesque::color::white())
                .label("Example Slider")
                .react(|val: f32| {
                    // This is called when the user updates the slider.
                    slider_val = val;
                })
                // Add the widget to the conrod::Ui. This schedules the widget it to be
                // drawn when we call Ui::draw.
                .set(SLIDER, &mut ui);
            
            // Draw all Conrod widgets. (We only have one in this instance.)
            ui.draw_if_changed(c, g); 
        });
        
        e.update(|args| {
            secs += args.dt;
        });
    }
}


widget_ids! {
    SLIDER,
}
