use std::ffi::CString;
use std::num::NonZeroU32;

use glutin::config::Config;
use glutin::context::{ContextApi, ContextAttributesBuilder, NotCurrentGlContext, PossiblyCurrentContext, Version};
use glutin::display::{GetGlDisplay, GlDisplay};
use glutin::surface::{GlSurface, Surface, SwapInterval, WindowSurface};
use glutin_winit::GlWindow;
use raw_window_handle::HasRawWindowHandle;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::window::{Fullscreen, Window};

use crate::camera::Camera;
use crate::font::FontRenderer;
use crate::game::Menu;
use crate::gl;
use crate::model;
use crate::npc::NPCManager;
use crate::player::Player;
use crate::settings::CameraSettings;
use crate::ui::UIManager;
use crate::vehicle::VehicleManager;
use crate::world::World;

pub struct Renderer {
    pub(crate) window: Window,
    surface: Surface<WindowSurface>,
    context: PossiblyCurrentContext,
    vsync: bool,
    font: FontRenderer,
    fullscreen_mode: u8,
    resolution_width: u32,
    resolution_height: u32,
}

impl Renderer {
    pub fn new(
        window: Window,
        config: Config,
        settings: &crate::settings::Settings,
    ) -> Self {
        let display = config.display();
        let raw_window_handle = window.raw_window_handle();

        // Create the OpenGL context first. Display mode changes are deliberately
        // not performed here; fullscreen is applied by Game after the event loop
        // has started, using the same path as an in-game display-mode change.
        let context = [
            ContextAttributesBuilder::new()
                .with_context_api(ContextApi::OpenGl(Some(Version::new(2, 1))))
                .build(Some(raw_window_handle)),
            ContextAttributesBuilder::new().build(Some(raw_window_handle)),
        ]
        .into_iter()
        .find_map(|attributes| unsafe { display.create_context(&config, &attributes).ok() })
        .expect("Could not create an OpenGL context");

        let surface_attributes = window.build_surface_attributes(Default::default());
        let surface = unsafe {
            display
                .create_window_surface(&config, &surface_attributes)
                .expect("Could not create the OpenGL window surface")
        };

        // `make_current` is provided by NotCurrentGlContext in glutin 0.31.
        let context = context
            .make_current(&surface)
            .expect("Could not make OpenGL context current");

        gl::load_with(|symbol| match CString::new(symbol) {
            Ok(symbol) => display.get_proc_address(&symbol).cast(),
            Err(_) => std::ptr::null(),
        });

        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LEQUAL);
            gl::ClearColor(0.38, 0.58, 0.82, 1.0);
            gl::ShadeModel(gl::SMOOTH);
            gl::Enable(gl::LIGHTING);
            gl::Enable(gl::LIGHT0);
            gl::Enable(gl::COLOR_MATERIAL);
            gl::ColorMaterial(gl::FRONT_AND_BACK, gl::AMBIENT_AND_DIFFUSE);
            gl::Enable(gl::NORMALIZE);

            if settings.graphics.msaa_samples > 0 {
                gl::Enable(gl::MULTISAMPLE);
            } else {
                gl::Disable(gl::MULTISAMPLE);
            }

            let size = window.inner_size();
            gl::Viewport(0, 0, size.width as i32, size.height as i32);
        }

        model::set_texture_filtering(settings.graphics.texture_filtering);
        let font = FontRenderer::new(26.0)
            .unwrap_or_else(|e| panic!("Font initialization failed: {}", e));

        let renderer = Self {
            window,
            surface,
            context,
            vsync: settings.graphics.vsync,
            font,
            fullscreen_mode: settings.graphics.fullscreen_mode,
            resolution_width: settings.graphics.resolution_width,
            resolution_height: settings.graphics.resolution_height,
        };

        renderer.apply_vsync();
        renderer
    }

    fn apply_vsync(&self) {
        let interval = if self.vsync {
            SwapInterval::Wait(NonZeroU32::new(1).unwrap())
        } else {
            SwapInterval::DontWait
        };

        if let Err(e) = self.surface.set_swap_interval(&self.context, interval) {
            log::warn!("Could not change swap interval: {}", e);
        }
    }

    pub fn toggle_vsync(&mut self) {
        self.vsync = !self.vsync;
        self.apply_vsync();
    }

    pub fn vsync(&self) -> bool {
        self.vsync
    }

    pub fn supported_resolutions(&self) -> Vec<(u32, u32)> {
        let Some(monitor) = self.window.current_monitor() else {
            return Vec::new();
        };

        let mut result = Vec::new();

        for mode in monitor.video_modes() {
            let size = mode.size();
            let resolution = (size.width, size.height);

            if size.width >= 640 && size.height >= 480 && !result.contains(&resolution) {
                result.push(resolution);
            }
        }

        result.sort();
        result
    }

    pub fn set_display_mode(&mut self, mode: u8, width: u32, height: u32) {
        let mode = mode.min(2);

        match mode {
            0 => {
                self.window.set_fullscreen(None);
                self.resolution_width = width;
                self.resolution_height = height;
                let _ = self
                    .window
                    .request_inner_size(PhysicalSize::new(width, height));
            }
            1 => {
                self.resolution_width = width;
                self.resolution_height = height;
                self.window
                    .set_fullscreen(Some(Fullscreen::Borderless(None)));
            }
            2 => {
                // Exclusive fullscreen requires an actual monitor VideoMode.
                // This is intentionally the same operation used whether the
                // mode is selected at startup or changed from the settings menu.
                let preferred = self.window.current_monitor().and_then(|monitor| {
                    monitor
                        .video_modes()
                        .filter(|video_mode| {
                            let size = video_mode.size();
                            size.width == width && size.height == height
                        })
                        .max_by_key(|video_mode| video_mode.refresh_rate_millihertz())
                });

                if let Some(video_mode) = preferred {
                    log::info!(
                        "Applying exclusive fullscreen {}x{} @ {} mHz",
                        width,
                        height,
                        video_mode.refresh_rate_millihertz()
                    );
                    self.resolution_width = width;
                    self.resolution_height = height;
                    self.window
                        .set_fullscreen(Some(Fullscreen::Exclusive(video_mode)));
                } else {
                    log::warn!(
                        "Requested exclusive resolution {}x{} is not supported; using borderless fullscreen",
                        width,
                        height
                    );
                    self.window
                        .set_fullscreen(Some(Fullscreen::Borderless(None)));
                }
            }
            _ => unreachable!(),
        }

        self.fullscreen_mode = mode;
        self.window.request_redraw();
    }

    pub fn set_fullscreen_mode(&mut self, mode: u8) {
        self.set_display_mode(mode, self.resolution_width, self.resolution_height);
    }

    pub fn fullscreen_mode(&self) -> u8 {
        self.fullscreen_mode
    }

    pub fn resolution(&self) -> (u32, u32) {
        (self.resolution_width, self.resolution_height)
    }

    pub fn is_fullscreen(&self) -> bool {
        self.window.fullscreen().is_some()
    }

    pub fn toggle_fullscreen(&mut self) {
        if self.is_fullscreen() {
            self.set_fullscreen_mode(0);
        } else {
            self.set_fullscreen_mode(if self.fullscreen_mode == 0 {
                1
            } else {
                self.fullscreen_mode
            });
        }
    }

    pub fn set_texture_filtering(&self, mode: u8) {
        model::set_texture_filtering(mode);
    }

    pub fn set_cursor_visible(&self, visible: bool) {
        self.window.set_cursor_visible(visible);
    }

    pub fn center_cursor(&self) {
        let size = self.window.inner_size();
        let _ = self.window.set_cursor_position(PhysicalPosition::new(
            size.width as f64 / 2.0,
            size.height as f64 / 2.0,
        ));
    }

    pub fn window_center(&self) -> (f64, f64) {
        let size = self.window.inner_size();
        (size.width as f64 / 2.0, size.height as f64 / 2.0)
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        let (Some(width), Some(height)) = (NonZeroU32::new(width), NonZeroU32::new(height)) else {
            return;
        };

        self.surface.resize(&self.context, width, height);

        unsafe {
            gl::Viewport(0, 0, width.get() as i32, height.get() as i32);
        }
    }

    pub fn render(
        &self,
        camera: &Camera,
        player: &Player,
        world: &World,
        _npc_manager: &NPCManager,
        _vehicle_manager: &VehicleManager,
        _ui_manager: &UIManager,
        menu: Option<Menu>,
        selected: usize,
        camera_settings: &CameraSettings,
        texture_filtering: u8,
        msaa_samples: u8,
        fullscreen_mode: u8,
        width: u32,
        height: u32,
        aiming: bool,
    ) {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            let projection = camera.projection_matrix();
            let view = camera.view_matrix();

            gl::MatrixMode(gl::PROJECTION);
            gl::LoadMatrixf(projection.to_cols_array().as_ptr());
            gl::MatrixMode(gl::MODELVIEW);
            gl::LoadMatrixf(glam::Mat4::IDENTITY.to_cols_array().as_ptr());
            gl::LoadMatrixf(view.to_cols_array().as_ptr());

            let light_position: [f32; 4] = [-20.0, 30.0, 20.0, 1.0];
            let light_ambient: [f32; 4] = [0.22, 0.22, 0.22, 1.0];
            let light_diffuse: [f32; 4] = [0.90, 0.90, 0.90, 1.0];

            gl::Lightfv(gl::LIGHT0, gl::POSITION, light_position.as_ptr());
            gl::Lightfv(gl::LIGHT0, gl::AMBIENT, light_ambient.as_ptr());
            gl::Lightfv(gl::LIGHT0, gl::DIFFUSE, light_diffuse.as_ptr());

            model::draw_map();

            for object in world.objects.values() {
                if !object.asset.is_empty() {
                    model::draw_asset(
                        &object.asset,
                        object.position,
                        object.rotation,
                        object.scale,
                    );
                }
            }

            model::draw_player(player.position, player.rotation);
            gl::Disable(gl::LIGHTING);

            if let Some(menu) = menu {
                draw_menu(
                    &self.font,
                    menu,
                    selected,
                    self.vsync,
                    camera_settings,
                    texture_filtering,
                    msaa_samples,
                    fullscreen_mode,
                    width,
                    height,
                );
            } else if aiming {
                draw_crosshair();
            } else {
                draw_center_dot();
            }

            gl::Enable(gl::LIGHTING);
            gl::Flush();
        }

        if let Err(e) = self.surface.swap_buffers(&self.context) {
            log::error!("Failed to swap buffers: {}", e);
        }
    }
}

fn filter_name(mode: u8) -> &'static str {
    match mode {
        1 => "TRILINEAR",
        2 => "OFF",
        _ => "BILINEAR",
    }
}

fn fullscreen_name(mode: u8) -> &'static str {
    match mode {
        1 => "BORDERLESS",
        2 => "EXCLUSIVE",
        _ => "WINDOWED",
    }
}

fn msaa_name(samples: u8) -> String {
    if samples == 0 {
        "OFF".into()
    } else {
        format!("{}x", samples)
    }
}

unsafe fn draw_center_dot() {
    gl::MatrixMode(gl::PROJECTION);
    gl::LoadMatrixf(
        glam::Mat4::orthographic_rh_gl(-1.0, 1.0, -1.0, 1.0, -1.0, 1.0)
            .to_cols_array()
            .as_ptr(),
    );
    gl::MatrixMode(gl::MODELVIEW);
    gl::LoadMatrixf(glam::Mat4::IDENTITY.to_cols_array().as_ptr());
    gl::Disable(gl::DEPTH_TEST);
    gl::Color3f(1.0, 1.0, 1.0);
    gl::PointSize(5.0);
    gl::Begin(gl::POINTS);
    gl::Vertex3f(0.0, 0.0, 0.0);
    gl::End();
    gl::Enable(gl::DEPTH_TEST);
}

unsafe fn draw_crosshair() {
    gl::MatrixMode(gl::PROJECTION);
    gl::LoadMatrixf(
        glam::Mat4::orthographic_rh_gl(-1.0, 1.0, -1.0, 1.0, -1.0, 1.0)
            .to_cols_array()
            .as_ptr(),
    );
    gl::MatrixMode(gl::MODELVIEW);
    gl::LoadMatrixf(glam::Mat4::IDENTITY.to_cols_array().as_ptr());
    gl::Disable(gl::DEPTH_TEST);
    gl::Color3f(1.0, 1.0, 1.0);
    gl::LineWidth(2.0);

    let gap = 0.012;
    let length = 0.035;

    gl::Begin(gl::LINES);
    gl::Vertex3f(-gap - length, 0.0, 0.0);
    gl::Vertex3f(-gap, 0.0, 0.0);
    gl::Vertex3f(gap, 0.0, 0.0);
    gl::Vertex3f(gap + length, 0.0, 0.0);
    gl::Vertex3f(0.0, -gap - length, 0.0);
    gl::Vertex3f(0.0, -gap, 0.0);
    gl::Vertex3f(0.0, gap, 0.0);
    gl::Vertex3f(0.0, gap + length, 0.0);
    gl::End();

    gl::Enable(gl::DEPTH_TEST);
}

unsafe fn draw_menu(
    font: &FontRenderer,
    menu: Menu,
    selected: usize,
    vsync: bool,
    camera: &CameraSettings,
    filtering: u8,
    msaa: u8,
    fullscreen: u8,
    width: u32,
    height: u32,
) {
    gl::MatrixMode(gl::PROJECTION);
    gl::LoadMatrixf(
        glam::Mat4::orthographic_rh_gl(-1.0, 1.0, -1.0, 1.0, -1.0, 1.0)
            .to_cols_array()
            .as_ptr(),
    );
    gl::MatrixMode(gl::MODELVIEW);
    gl::LoadMatrixf(glam::Mat4::IDENTITY.to_cols_array().as_ptr());
    gl::Disable(gl::DEPTH_TEST);
    gl::Enable(gl::BLEND);
    gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
    gl::Color4f(0.015, 0.02, 0.03, 0.84);

    gl::Begin(gl::QUADS);
    gl::Vertex3f(-1.0, -1.0, 0.0);
    gl::Vertex3f(1.0, -1.0, 0.0);
    gl::Vertex3f(1.0, 1.0, 0.0);
    gl::Vertex3f(-1.0, 1.0, 0.0);
    gl::End();

    let (title, items): (&str, &[&str]) = match menu {
        Menu::Pause => ("PAUSED", &["RESUME", "SETTINGS", "QUIT GAME"]),
        Menu::Settings => ("SETTINGS", &["CONTROLS", "DISPLAY & GRAPHICS", "BACK"]),
        Menu::Controls => ("CONTROLS", &["Mouse Sensitivity", "Invert X", "Invert Y", "BACK"]),
        Menu::Display => (
            "DISPLAY & GRAPHICS",
            &[
                "Texture Filtering",
                "MSAA",
                "Resolution",
                "Fullscreen Mode",
                "VSync",
                "APPLY",
                "BACK",
            ],
        ),
    };

    font.draw_text(title, -0.30, 0.68, [255, 255, 255, 255]);

    for (i, label) in items.iter().enumerate() {
        let y = 0.40 - (i as f32) * 0.12;
        let active = i == selected;
        let color = if active {
            [255, 255, 255, 255]
        } else {
            [215, 215, 220, 255]
        };

        font.draw_text(if active { ">" } else { " " }, -0.60, y, color);
        font.draw_text(label, -0.52, y, color);
    }

    match menu {
        Menu::Controls => {
            let values = [
                format!("{:.3}", camera.mouse_sensitivity),
                if camera.invert_x { "ON".into() } else { "OFF".into() },
                if camera.invert_y { "ON".into() } else { "OFF".into() },
            ];

            for (i, value) in values.iter().enumerate() {
                font.draw_text(value, 0.28, 0.40 - (i as f32) * 0.12, [150, 230, 170, 255]);
            }
        }
        Menu::Display => {
            let values = [
                filter_name(filtering).to_string(),
                msaa_name(msaa),
                format!("{}x{}", width, height),
                fullscreen_name(fullscreen).to_string(),
                if vsync { "ON".into() } else { "OFF".into() },
            ];

            for (i, value) in values.iter().enumerate() {
                font.draw_text(value, 0.28, 0.40 - (i as f32) * 0.12, [150, 230, 170, 255]);
            }
        }
        _ => {}
    }

    font.draw_text("UP/DOWN: select", -0.60, -0.72, [170, 170, 180, 255]);
    font.draw_text("LEFT/RIGHT: change", 0.00, -0.72, [170, 170, 180, 255]);
    font.draw_text("ENTER: select", -0.60, -0.83, [170, 170, 180, 255]);
    font.draw_text("ESC: back", 0.35, -0.83, [170, 170, 180, 255]);

    gl::Disable(gl::BLEND);
    gl::Enable(gl::DEPTH_TEST);
}
