use bevy::{
    prelude::*,
    render::{mesh, render_resource::PrimitiveTopology},
};

use crate::chunk_material::ATTRIBUTE_INDEX;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Index(u32);

#[derive(Default)]
pub struct MeshBuilder {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    texcoords: Vec<[f32; 2]>,
    texindices: Vec<u32>,
    indices: Vec<u32>,
}

impl MeshBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn vertex(&mut self, position: Vec3, normal: Vec3, texcoord: Vec2, texindex: u32) -> Index {
        let index = self.positions.len();
        self.positions.push(position.to_array());
        self.normals.push(normal.to_array());
        self.texcoords.push(texcoord.to_array());
        self.texindices.push(texindex);
        Index(index as u32)
    }

    pub fn indices(&mut self, index: impl IntoIterator<Item = Index>) {
        self.indices.extend(index.into_iter().map(|index| index.0));
    }

    pub fn build(self) -> Mesh {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, self.texcoords);
        mesh.insert_attribute(ATTRIBUTE_INDEX, self.texindices);
        mesh.set_indices(Some(mesh::Indices::U32(self.indices)));
        mesh
    }
}
