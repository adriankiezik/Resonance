pub mod node;
pub mod nodes;

use anyhow::{anyhow, Result};
use bevy_ecs::prelude::{Resource, World};
use node::{RenderContext, RenderNode};
use std::collections::{HashMap, VecDeque};

#[derive(Resource)]
pub struct RenderGraph {
    nodes: HashMap<String, Box<dyn RenderNode>>,
}

impl RenderGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node: Box<dyn RenderNode>) {
        let name = node.name().to_string();
        if self.nodes.contains_key(&name) {
            log::warn!("Render node '{}' already exists, replacing it", name);
        }
        self.nodes.insert(name, node);
    }

    pub fn remove_node(&mut self, name: &str) -> Option<Box<dyn RenderNode>> {
        self.nodes.remove(name)
    }

    pub fn execute(&mut self, world: &mut World, renderer: &mut crate::renderer::Renderer) -> Result<()> {
        if self.nodes.is_empty() {
            return Ok(());
        }

        let execution_order = self.topological_sort()?;

        let output = renderer.surface().get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = renderer.device().create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        let context = RenderContext {
            device: renderer.device(),
            queue: renderer.queue(),
            surface_config: renderer.config(),
            surface_view: &view,
            camera_buffer: renderer.camera_buffer(),
            camera_bind_group: renderer.camera_bind_group(),
            depth_view: renderer.depth_view(),
        };

        for node_name in execution_order {
            let node = self.nodes.get_mut(&node_name).unwrap();
            node.execute(world, &context, &mut encoder)?;
        }

        renderer.queue().submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn topological_sort(&self) -> Result<Vec<String>> {
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut adjacency: HashMap<String, Vec<String>> = HashMap::new();

        for (name, _) in &self.nodes {
            in_degree.insert(name.clone(), 0);
            adjacency.insert(name.clone(), Vec::new());
        }

        for (name, node) in &self.nodes {
            for dep in node.dependencies() {
                if !self.nodes.contains_key(*dep) {
                    return Err(anyhow!(
                        "Render node '{}' depends on '{}', but '{}' does not exist",
                        name,
                        dep,
                        dep
                    ));
                }

                adjacency.get_mut(*dep).unwrap().push(name.clone());
                *in_degree.get_mut(name).unwrap() += 1;
            }
        }

        let mut queue: VecDeque<String> = in_degree
            .iter()
            .filter(|(_, degree)| **degree == 0)
            .map(|(name, _)| name.clone())
            .collect();

        let mut result = Vec::new();

        while let Some(node_name) = queue.pop_front() {
            result.push(node_name.clone());

            if let Some(dependents) = adjacency.get(&node_name) {
                for dependent in dependents {
                    let degree = in_degree.get_mut(dependent).unwrap();
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push_back(dependent.clone());
                    }
                }
            }
        }

        if result.len() != self.nodes.len() {
            let unprocessed: Vec<_> = self.nodes
                .keys()
                .filter(|name| !result.contains(name))
                .collect();
            return Err(anyhow!(
                "Render graph has circular dependencies. Unprocessed nodes: {:?}",
                unprocessed
            ));
        }

        Ok(result)
    }
}

impl Default for RenderGraph {
    fn default() -> Self {
        Self::new()
    }
}
