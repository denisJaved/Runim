use glam::{Vec2, Vec3};
use image::DynamicImage;
use runim_derive::AsAny;
use crate::prelude::{RIObject, Rectangle};
use crate::renderer::{DrawContext, ImageHandle};
use crate::Shape;
use crate::shapes::{Shape, ShapeState, Transform2D};

#[derive(Shape, Debug)]
pub struct ImageRectangle {
    width: f32,
    height: f32,
    transform: Transform2D,
    image: DynamicImage,
    state: Option<ImageRectState>
}

#[derive(AsAny, Debug)]
struct ImageRectState {
    handle: Option<ImageHandle>
}

impl ShapeState for ImageRectState {}

impl ImageRectangle {
    pub fn new(image: DynamicImage) -> ImageRectangle {
        ImageRectangle {
            width: image.width() as f32,
            height: image.height() as f32,
            transform: Default::default(),
            image,
            state: Some(ImageRectState {
                handle: None
            }),
        }
    }
}

impl Shape for ImageRectangle {
    fn draw<'a, 'b>(&'b mut self, ctx: &'a mut (dyn DrawContext + 'a), state: Option<&'b mut dyn ShapeState>) {
        use glam::{vec3, vec2};

        let state = match state {
            None => self.state.as_mut().unwrap(),
            Some(s) => s.as_any_mut().downcast_mut::<ImageRectState>().unwrap()
        };
        let handle = match state.handle.as_ref() {
            Some(t) => t,
            None => {
                state.handle = Some(ctx.load_image(&self.image).unwrap());
                state.handle.as_ref().unwrap()
            }
        };

        ctx.offset(vec3(
            self.transform.pos.x - self.width / 2.0,
            self.transform.pos.y - self.height / 2.0,
            0.0
        ));
        // ctx.tex_vertex(vec3(self.width, 0.0, 0.0), vec2(1.0, 0.0), handle);
        // ctx.tex_vertex(vec3(self.width, self.height, 0.0), vec2(1.0, 1.0), handle);
        // ctx.tex_vertex(vec3(0.0, self.height, 0.0), vec2(0.0, 1.0), handle);
        //
        // ctx.tex_vertex(vec3(0.0, 0.0, 0.0), vec2(0.0, 0.0), handle);
        // ctx.tex_vertex(vec3(self.width, 0.0, 0.0), vec2(1.0, 0.0), handle);
        // ctx.tex_vertex(vec3(0.0, self.height, 0.0), vec2(0.0, 1.0), handle);

        ctx.tex_quad(self.width, self.height, handle);

        ctx.clear_offset();
    }

    fn transform(&self) -> &Transform2D {
        &self.transform
    }

    fn transform_mut(&mut self) -> &mut Transform2D {
        &mut self.transform
    }
    fn extract_state(&self) -> Option<&dyn ShapeState> {
        self.state.as_ref().map(|f| f.as_shape_state())
    }
    fn extract_state_mut(&mut self) -> Option<&mut dyn ShapeState> {
        self.state.as_mut().map(|f| f.as_shape_state_mut())
    }
}

impl Clone for ImageRectangle {
    fn clone(&self) -> Self {
        ImageRectangle {
            width: self.width,
            height: self.width,
            transform: self.transform.clone(),
            image: self.image.clone(),
            state: Some(ImageRectState {
                handle: None
            })
        }
    }
}

impl RIObject<ImageRectangle> {
    pub fn set_size(&self, width: f32, height: f32) -> &RIObject<ImageRectangle> {
        {
            let mut guard = self.shape_mut();
            guard.width = width;
            guard.height = height;
        }
        self
    }
}
