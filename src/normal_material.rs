use bevy::{reflect::TypeUuid, render::render_resource::{AsBindGroup, ShaderRef}};

use super::*;

#[derive(AsBindGroup, TypeUuid, Clone)]
#[uuid = "f690fdae-d598-45ab-8225-97e2a3f056e0"]
pub struct NormalMaterial {}

impl Material for NormalMaterial {
    fn fragment_shader() -> ShaderRef {
        "normal_material.wgsl".into()
    }
}