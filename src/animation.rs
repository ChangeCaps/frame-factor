use crate::networking::*;
use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        pipeline::{PipelineDescriptor, RenderPipeline},
        render_graph::{base, AssetRenderResourcesNode, RenderGraph, RenderResourcesNode},
        renderer::RenderResources,
        shader::{ShaderDefs, ShaderStages},
    },
};

pub const ANIMATOR_PIPELINE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(PipelineDescriptor::TYPE_UUID, 1684523432231);

#[derive(Serialize, Deserialize, TypeUuid)]
#[uuid = "cbf6ae0d-a8d3-4ed0-9a52-0575b587051a"]
pub struct Animation {
    pub image: String,
    pub columns: u32,
    pub rows: u32,
    pub length: u32,
    pub fps: f32,
    pub looping: bool,
}

#[derive(RenderResources, ShaderDefs)]
pub struct Animator {
    #[shader_def]
    pub texture: Option<Handle<Texture>>,
    pub frame: u32,
    pub columns: u32,
    pub rows: u32,
    #[render_resources(ignore)]
    pub play_time: f32,
    #[render_resources(ignore)]
    pub animation: Option<String>,
    #[render_resources(ignore)]
    pub just_ended: bool,
    #[render_resources(ignore)]
    pub just_advanced: bool,
}

impl Animator {
    pub fn new() -> Self {
        Self {
            texture: None,
            frame: 0,
            columns: 1,
            rows: 1,
            play_time: 0.0,
            animation: None,
            just_ended: false,
            just_advanced: false,
        }
    }

    pub fn frame(&self) -> u32 {
        self.frame
    }

    pub fn advance(&mut self, animation: &Animation, delta_time: f32) {
        if self.play_time == 0.0 {
            self.just_advanced = true;
        }

        self.columns = animation.columns;
        self.rows = animation.rows;

        self.play_time += delta_time;
        let frame_time = 1.0 / animation.fps;

        if self.play_time > frame_time * animation.length as f32 {
            if animation.looping {
                self.play_time = 0.0;
            } else {
                self.stop();
                self.just_ended = true;
            }
        }

        let prev = self.frame;
        self.frame = (self.play_time as f32 / frame_time).floor() as u32;

        if self.frame != prev {
            self.just_advanced = true;
        }
    }

    pub fn is_playing(&self) -> bool {
        self.animation.is_some()
    }

    /// Sets the currently playing animation. This is different from `play`
    /// because it wont reset the play time.
    ///
    /// If `interrupt` is true, then if the currently playing animation is
    /// different from `animation`, will reset play time.
    pub fn set_playing(&mut self, animation: impl Into<String>, interrupt: bool) {
        let animation = animation.into();

        if let Some(current) = &mut self.animation {
            if *current != animation && interrupt {
                self.play(animation);
            } else {
                *current = animation;
            }
        } else {
            self.play(animation);
        }
    }

    pub fn just_ended(&self) -> bool {
        self.just_ended
    }

    pub fn just_advanced(&self) -> bool {
        self.just_advanced
    }

    pub fn play(&mut self, animation: impl Into<String>) {
        self.stop();
        self.animation = Some(animation.into());
    }

    pub fn stop(&mut self) {
        self.play_time = 0.0;
        self.frame = 0;
        self.texture = None;
        self.animation = None;
    }
}

pub fn animator_server_system(
    animations: Res<Assets<Animation>>,
    mut query: Query<&mut Animator>,
) {
    for mut animator in query.iter_mut() {
        animator.just_ended = false;
        animator.just_advanced = false;

        if let Some(animation) = animator.animation.clone() {
            let animation = animations.get(animation.as_str()).unwrap();

            animator.advance(animation, 1.0 / 48.0);
        }
    }
}

pub fn animator_client_system(
    time: Res<Time>,
    textures: Res<Assets<Texture>>,
    animations: Res<Assets<Animation>>,
    mut query: Query<&mut Animator>,
) {
    for mut animator in query.iter_mut() {
        animator.just_ended = false;
        animator.just_advanced = false;

        if let Some(animation) = animator.animation.clone() {
            let animation = animations.get(animation.as_str()).unwrap();

            animator.advance(animation, time.delta_seconds());

            if animator.texture.is_none() {
                animator.texture = Some(textures.get_handle(animation.image.as_str()));
            }
        } else {
            animator.texture = None;
        }
    }
}

#[derive(Bundle)]
pub struct AnimatorBundle {
    pub animator: Animator,
    pub mesh: Handle<Mesh>,
    pub main_pass: base::MainPass,
    pub draw: Draw,
    pub visible: Visible,
    pub render_pipelines: RenderPipelines,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

impl Default for AnimatorBundle {
    fn default() -> Self {
        Self {
            animator: Animator::new(),
            mesh: bevy::sprite::QUAD_HANDLE.typed(),
            main_pass: Default::default(),
            draw: Default::default(),
            visible: Visible {
                is_transparent: true,
                ..Default::default()
            },
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                ANIMATOR_PIPELINE_HANDLE.typed(),
            )]),
            transform: Default::default(),
            global_transform: Default::default(),
        }
    }
}

pub struct AnimationLoader;

crate::ron_loader!(AnimationLoader, "anim" => Animation);

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app_builder: &mut AppBuilder) {
        let is_server = app_builder
            .world()
            .get_resource::<NetworkSettings>()
            .unwrap()
            .is_server;

        app_builder.add_asset::<Animation>();
        app_builder.add_asset_loader(AnimationLoader);

        if is_server {
            app_builder.add_system(animator_server_system.system());
        } else {
            app_builder.add_system(animator_client_system.system());

            let mut render_graph = app_builder
                .world_mut()
                .get_resource_mut::<RenderGraph>()
                .unwrap();

            render_graph.add_system_node("animator", RenderResourcesNode::<Animator>::new(false));

            render_graph
                .add_node_edge("animator", base::node::MAIN_PASS)
                .unwrap();

            let asset_server = app_builder.world().get_resource::<AssetServer>().unwrap();
            asset_server.watch_for_changes().unwrap();

            let vert = asset_server.load("shaders/animator.vert");
            let frag = asset_server.load("shaders/animator.frag");

            app_builder
                .world_mut()
                .get_resource_mut::<Assets<PipelineDescriptor>>()
                .unwrap()
                .set_untracked(
                    ANIMATOR_PIPELINE_HANDLE,
                    PipelineDescriptor::default_config(ShaderStages {
                        vertex: vert,
                        fragment: Some(frag),
                    }),
                );
        }
    }
}
