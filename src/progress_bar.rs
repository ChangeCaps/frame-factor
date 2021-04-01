use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        pipeline::{PipelineDescriptor, RenderPipeline},
        render_graph::{base, AssetRenderResourcesNode, RenderGraph, RenderResourcesNode},
        renderer::RenderResources,
        shader::ShaderStages,
    },
};
use serde::{Deserialize, Serialize};

const PIPELINE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(PipelineDescriptor::TYPE_UUID, 64184235874523);

#[derive(RenderResources)]
pub struct ProgressBar {
    pub value: f32,
    pub max: f32,
}

#[derive(Serialize, Deserialize, RenderResources, TypeUuid)]
#[uuid = "14452f74-4e78-4aae-a737-76f30962be5f"]
pub struct ProgressBarMaterial {
    pub size: Vec2,
    pub color_light: Color,
    pub color_dark: Color,
    pub color_bg: Color,
    pub color_fg: Color,
}

pub struct ProgressBarLoader;

crate::ron_loader!(ProgressBarLoader, "pb" => ProgressBarMaterial);

#[derive(Bundle)]
pub struct ProgressBarBundle {
    pub progress_bar: ProgressBar,
    pub material: Handle<ProgressBarMaterial>,
    pub mesh: Handle<Mesh>,
    pub main_pass: base::MainPass,
    pub draw: Draw,
    pub visible: Visible,
    pub render_pipelines: RenderPipelines,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

impl Default for ProgressBarBundle {
    fn default() -> Self {
        Self {
            progress_bar: ProgressBar {
                value: 50.0,
                max: 100.0,
            },
            material: Default::default(),
            mesh: bevy::sprite::QUAD_HANDLE.typed(),
            main_pass: Default::default(),
            draw: Default::default(),
            visible: Default::default(),
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                PIPELINE_HANDLE.typed(),
            )]),
            transform: Default::default(),
            global_transform: Default::default(),
        }
    }
}

pub struct ProgressBarPlugin;

impl Plugin for ProgressBarPlugin {
    fn build(&self, app_builder: &mut AppBuilder) {
        app_builder.add_asset::<ProgressBarMaterial>();
        app_builder.add_asset_loader(ProgressBarLoader);

        let asset_server = app_builder.world().get_resource::<AssetServer>().unwrap();
        asset_server.watch_for_changes().unwrap();

        let vert = asset_server.load("shaders/progress_bar.vert");
        let frag = asset_server.load("shaders/progress_bar.frag");

        app_builder
            .world_mut()
            .get_resource_mut::<Assets<PipelineDescriptor>>()
            .unwrap()
            .set_untracked(
                PIPELINE_HANDLE,
                PipelineDescriptor::default_config(ShaderStages {
                    vertex: vert,
                    fragment: Some(frag),
                }),
            );

        let mut render_graph = app_builder
            .world_mut()
            .get_resource_mut::<RenderGraph>()
            .unwrap();

        // progress bar
        render_graph.add_node(
            "progress_bar",
            RenderResourcesNode::<ProgressBar>::new(true),
        );
        render_graph
            .add_node_edge("progress_bar", base::node::MAIN_PASS)
            .unwrap();

        // progress bar material
        render_graph.add_node(
            "progress_bar_material",
            AssetRenderResourcesNode::<ProgressBarMaterial>::new(false),
        );
        render_graph
            .add_node_edge("progress_bar_material", base::node::MAIN_PASS)
            .unwrap();
    }
}
