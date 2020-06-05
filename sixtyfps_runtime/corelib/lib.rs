/*!

# SixtyFPS runtime library

**NOTE:** This library is an internal crate for the SixtyFPS project.
This crate should not be used directly by application using SixtyFPS.
You should use the `sixtyfps` crate instead
*/

pub mod graphics;
pub mod input;
pub mod layout;

pub mod abi {
    pub mod datastructures;
    pub mod model;
    pub mod primitives;
    pub mod properties;
    pub mod signals;
    pub mod string;
}

//#[cfg(feature = "rtti")]
pub mod rtti;

#[doc(inline)]
pub use abi::string::SharedString;

#[doc(inline)]
pub use abi::properties::{EvaluationContext, Property};

#[doc(inline)]
pub use abi::signals::Signal;

mod item_rendering;

use abi::datastructures::Color;
use winit::platform::desktop::EventLoopExtDesktop;
pub struct MainWindow<GraphicsBackend: graphics::GraphicsBackend> {
    pub graphics_backend: GraphicsBackend,
    event_loop: winit::event_loop::EventLoop<()>,
    pub rendering_cache: graphics::RenderingCache<GraphicsBackend>,
}

impl<GraphicsBackend: graphics::GraphicsBackend> MainWindow<GraphicsBackend> {
    pub fn new(
        graphics_backend_factory: impl FnOnce(
            &winit::event_loop::EventLoop<()>,
            winit::window::WindowBuilder,
        ) -> GraphicsBackend,
    ) -> Self {
        let event_loop = winit::event_loop::EventLoop::new();
        let window_builder = winit::window::WindowBuilder::new();

        let graphics_backend = graphics_backend_factory(&event_loop, window_builder);

        Self { graphics_backend, event_loop, rendering_cache: graphics::RenderingCache::default() }
    }

    pub fn run_event_loop(
        mut self,
        component: vtable::VRef<crate::abi::datastructures::ComponentVTable>,
        mut prepare_rendering_function: impl FnMut(
                vtable::VRef<'_, crate::abi::datastructures::ComponentVTable>,
                &mut GraphicsBackend::RenderingPrimitivesBuilder,
                &mut graphics::RenderingCache<GraphicsBackend>,
            ) + 'static,
        mut render_function: impl FnMut(
                vtable::VRef<'_, crate::abi::datastructures::ComponentVTable>,
                &EvaluationContext,
                &mut GraphicsBackend::Frame,
                &mut graphics::RenderingCache<GraphicsBackend>,
            ) + 'static,
        mut input_function: impl FnMut(
                vtable::VRef<'_, crate::abi::datastructures::ComponentVTable>,
                &EvaluationContext,
                winit::dpi::PhysicalPosition<f64>,
                winit::event::ElementState,
            ) + 'static,
    ) where
        GraphicsBackend: 'static,
    {
        let mut graphics_backend = self.graphics_backend;
        let mut rendering_cache = self.rendering_cache;
        let mut cursor_pos = winit::dpi::PhysicalPosition::new(0., 0.);
        self.event_loop.run_return(move |event, _, control_flow| {
            *control_flow = winit::event_loop::ControlFlow::Wait;

            match event {
                winit::event::Event::WindowEvent {
                    event: winit::event::WindowEvent::CloseRequested,
                    ..
                } => *control_flow = winit::event_loop::ControlFlow::Exit,
                winit::event::Event::RedrawRequested(_) => {
                    {
                        let mut rendering_primitives_builder =
                            graphics_backend.new_rendering_primitives_builder();

                        prepare_rendering_function(
                            component,
                            &mut rendering_primitives_builder,
                            &mut rendering_cache,
                        );

                        graphics_backend.finish_primitives(rendering_primitives_builder);
                    }

                    let window = graphics_backend.window();

                    let size = window.inner_size();
                    let context = EvaluationContext { component: component };
                    let mut frame =
                        graphics_backend.new_frame(size.width, size.height, &Color::WHITE);
                    render_function(component, &context, &mut frame, &mut rendering_cache);
                    graphics_backend.present_frame(frame);
                }
                winit::event::Event::WindowEvent {
                    event: winit::event::WindowEvent::CursorMoved { position, .. },
                    ..
                } => {
                    cursor_pos = position;
                    // TODO: propagate mouse move?
                }

                winit::event::Event::WindowEvent {
                    event: winit::event::WindowEvent::MouseInput { state, .. },
                    ..
                } => {
                    let context = EvaluationContext { component };
                    input_function(component, &context, cursor_pos, state);
                    let window = graphics_backend.window();
                    // FIXME: remove this, it should be based on actual changes rather than this
                    window.request_redraw();
                }

                _ => (),
            }
        });
    }
}

pub fn run_component<GraphicsBackend: graphics::GraphicsBackend + 'static>(
    component: vtable::VRef<crate::abi::datastructures::ComponentVTable>,
    graphics_backend_factory: impl FnOnce(
        &winit::event_loop::EventLoop<()>,
        winit::window::WindowBuilder,
    ) -> GraphicsBackend,
) {
    let main_window = MainWindow::new(graphics_backend_factory);

    main_window.run_event_loop(
        component,
        move |component, mut rendering_primitives_builder, rendering_cache| {
            // Generate cached rendering data once
            crate::abi::datastructures::visit_items(
                component,
                |item, _| {
                    let ctx = EvaluationContext { component };
                    item_rendering::update_item_rendering_data(
                        &ctx,
                        item,
                        rendering_cache,
                        &mut rendering_primitives_builder,
                    );
                },
                (),
            );
        },
        move |component, context, frame, rendering_cache| {
            item_rendering::render_component_items(component, context, frame, &rendering_cache);
        },
        move |component, context, pos, state| {
            input::process_mouse_event(
                component,
                context,
                crate::abi::datastructures::MouseEvent {
                    pos: euclid::point2(pos.x as _, pos.y as _),
                    what: match state {
                        winit::event::ElementState::Pressed => {
                            crate::abi::datastructures::MouseEventType::MousePressed
                        }
                        winit::event::ElementState::Released => {
                            crate::abi::datastructures::MouseEventType::MouseReleased
                        }
                    },
                },
            )
        },
    );
}
