use bevy::{
    app::{App, Plugin, Startup},
    camera::Camera2d,
    core_pipeline::core_2d::graph::{Core2d, Node2d},
    ecs::{system::Commands, world::Mut},
    render::{RenderApp, render_graph::RenderGraph},
    ui_render::graph::NodeUi,
};

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

pub fn move_ui_pass_to_before_postprocessing_end_node(app: &mut App) {
    if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
        render_app
            .world_mut()
            .resource_scope(|_, mut graph: Mut<RenderGraph>| {
                if let Some(graph_2d) = graph.get_sub_graph_mut(Core2d) {
                    let _ = graph_2d
                        .remove_node_edge(Node2d::EndMainPassPostProcessing, NodeUi::UiPass);
                }
            });
    } else {
        panic!("no render app");
    }
}

pub struct RenderSimPlugin;

impl Plugin for RenderSimPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera);
        move_ui_pass_to_before_postprocessing_end_node(app);
    }
}
