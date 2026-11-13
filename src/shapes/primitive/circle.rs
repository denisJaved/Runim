use glam::Vec3;
use crate::prelude::{Color, RIObject, Shape};
use crate::renderer::DrawContext;
use crate::Shape;
use crate::shapes::{ShapeState, Transform2D};

#[derive(Shape, Debug, Clone, PartialEq)]
#[shape(colored)]
pub struct Circle {
    transform: Transform2D,
    color: Color,
    pub radius: f32,
    pub angles: u16
}
impl Shape for Circle {
    fn draw<'a, 'b>(&'b mut self, ctx: &'a mut (dyn DrawContext + 'a), _: Option<&'b mut dyn ShapeState>) {
        ctx.offset2(self.transform.pos);

        let mut last_point = Vec3::new(0.0, -self.radius, 0.0);
        for i in 0..=self.angles {
            let angle = i as f32 / self.angles as f32 * std::f32::consts::PI * 2.0;
            let point = Vec3::new(
                angle.sin() * self.radius,
                -angle.cos() * self.radius,
                0.0
            );
            ctx.triangle(last_point, point, Vec3::ZERO, self.color);
            //ctx.vertex(last_point, Color::RED);
            //ctx.vertex(point, Color::BLUE);
            //ctx.vertex(Vec3::ZERO, Color::GREEN);

            last_point = point;
        }

        ctx.clear_offset();
    }

    fn transform(&self) -> &Transform2D {
        &self.transform
    }

    fn transform_mut(&mut self) -> &mut Transform2D {
        &mut self.transform
    }
}

impl Default for Circle {
    fn default() -> Self {
        Circle {
            radius: 0.5,
            transform: Transform2D::default(),
            color: Color::WHITE,
            angles: 24
        }
    }
}

impl RIObject<Circle> {
    pub fn set_radius(&self, radius: f32) -> &RIObject<Circle> {
        {
            let mut guard = self.shape_mut();
            guard.radius = radius;
        }
        self
    }

    pub fn set_diameter(&self, diameter: f32) -> &RIObject<Circle> {
        {
            let mut guard = self.shape_mut();
            guard.radius = diameter / 2f32;
        }
        self
    }

    pub fn set_angles(&self, angles: u16) -> &RIObject<Circle> {
        {
            let mut guard = self.shape_mut();
            guard.angles = angles;
        }
        self
    }
}