use glam::Mat4;

pub struct Projection {
    pub aspect: f32,
    pub z_far: f32,
    pub z_near: f32
}

impl Projection {
    pub fn new(width: u32, height: u32) -> Projection {
        Projection {
            aspect: width as f32 / height as f32,
            z_far: 10000.0,
            z_near: 0.01,
        }
    }

    pub fn compute_matrix(&self) -> Mat4 {
        glam::camera::rh::proj::directx::perspective(40f32, self.aspect, self.z_near, self.z_far)
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub(super) fn new() -> Self {
        Self {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
        }
    }

    pub(super) fn update_proj(&mut self, projection: &Projection) -> Mat4 {
        let mat4 = projection.compute_matrix();
        self.view_proj = mat4.to_cols_array_2d();
        mat4
    }
}