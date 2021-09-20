use ash::Device;
use katla_math::Mat4;
use katla_vulkan::vulkan::CommandBuffer;

pub trait Drawable {
    fn update(&mut self, device: &Device, view: &Mat4, proj: &Mat4);
    fn draw(&self, command_buffer: &CommandBuffer);
}
