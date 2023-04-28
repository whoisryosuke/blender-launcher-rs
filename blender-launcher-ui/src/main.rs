use bevy::{
    prelude::*,
    render::{camera::Projection, mesh::Indices},
    window::PrimaryWindow,
};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use rfd::FileDialog;

use bevy_blender::*;

#[derive(Default, Resource)]
struct OccupiedScreenSpace {
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
}

const CAMERA_TARGET: Vec3 = Vec3::ZERO;

#[derive(Resource, Deref, DerefMut)]
struct OriginalCameraTransform(Transform);

struct File {
    path: String,
}

#[derive(Resource)]
struct AppState {
    selected_file: Option<usize>,
    files: Vec<File>,
}

struct SpawnEvent;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_plugin(bevy_blender::BlenderPlugin)
        .insert_resource(AppState {
            selected_file: None,
            files: Vec::new(),
        })
        .add_event::<SpawnEvent>()
        .init_resource::<OccupiedScreenSpace>()
        .add_startup_system(setup_system)
        .add_system(test_spawn)
        .add_system(ui_example_system)
        .add_system(update_camera_transform_system)
        .run();
}

fn ui_example_system(
    mut contexts: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut spawn_events: EventWriter<SpawnEvent>,
    mut app_state: ResMut<AppState>,
) {
    let ctx = contexts.ctx_mut();

    occupied_screen_space.left = egui::SidePanel::left("left_panel")
        .resizable(true)
        .show(ctx, |ui| {
            ui.heading("Left Panel");

            // We grab the selected ID to use later
            let selected_id = app_state.selected_file.unwrap_or(0);
            // We also store the original selected file index later to check for changes
            let original_file = app_state.selected_file.clone();

            // We keep the selected file index outside the loop
            // since we mutate `app_state` in loop
            let mut selected_file = None;

            // Loop over all files and show UI for them
            for (index, file) in app_state.files.iter().enumerate() {
                // Is the file selected? Change the name to signify that.
                let is_selected = app_state.selected_file.is_some() && selected_id == index;
                let name = if is_selected {
                    format!("⭐ {}", &file.path)
                } else {
                    file.path.to_string()
                };

                // Render the UI
                if ui.button(name).clicked() {
                    // Did we click?
                    // Toggle the file as selected/unselected.
                    println!("selected {}", file.path);
                    if is_selected {
                        selected_file = None;
                    } else {
                        selected_file = Some(index);
                    }
                }
            }
            // Update the state if we made changes
            if selected_file != original_file {
                app_state.selected_file = selected_file;
            }

            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .width();
    occupied_screen_space.right = egui::SidePanel::right("right_panel")
        .resizable(true)
        .show(ctx, |ui| {
            ui.heading("Right Panel");

            if ui.button("Spawn").clicked() {
                spawn_events.send(SpawnEvent);
            }

            if ui.button("Select file").clicked() {
                let files = FileDialog::new()
                    .add_filter("Blender", &["blend"])
                    .set_directory("/")
                    .pick_files();

                if let Some(file_path_buffers) = files {
                    for file_path_buffer in file_path_buffers {
                        let file_path_option = file_path_buffer.to_str();
                        if let Some(file_path) = file_path_option {
                            println!("{}", file_path);
                            app_state.files.push(File {
                                path: file_path.to_string(),
                            });
                        }
                    }
                }
            }

            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .width();
    occupied_screen_space.top = egui::TopBottomPanel::top("top_panel")
        .resizable(true)
        .show(ctx, |ui| {
            ui.heading("Top Panel");
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .height();
    occupied_screen_space.bottom = egui::TopBottomPanel::bottom("bottom_panel")
        .resizable(true)
        .show(ctx, |ui| {
            ui.heading("Bottom Panel");
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .height();
}

fn test_spawn(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    mut spawn_event: EventReader<SpawnEvent>,
) {
    if spawn_event.is_empty() {
        return;
    }

    for event in spawn_event.iter() {
        // Spawn the Suzanne mesh with the Red material
        commands.spawn(PbrBundle {
            mesh: asset_server.load(blender_mesh!("demo.blend", "Suzanne")),
            material: asset_server.load(blender_material!("demo.blend", "Red")),
            ..Default::default()
        });
    }
}

fn setup_system(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    // Spawn the Suzanne mesh with the Red material
    // commands.spawn(PbrBundle {
    //     mesh: asset_server.load(blender_mesh!("demo.blend", "Suzanne")),
    //     material: asset_server.load(blender_material!("demo.blend", "Red")),
    //     ..Default::default()
    // });

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });

    let camera_pos = Vec3::new(-2.0, 2.5, 5.0);
    let camera_transform =
        Transform::from_translation(camera_pos).looking_at(CAMERA_TARGET, Vec3::Y);
    commands.insert_resource(OriginalCameraTransform(camera_transform));

    commands.spawn(Camera3dBundle {
        transform: camera_transform,
        ..Default::default()
    });
}

fn update_camera_transform_system(
    occupied_screen_space: Res<OccupiedScreenSpace>,
    original_camera_transform: Res<OriginalCameraTransform>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut camera_query: Query<(&Projection, &mut Transform)>,
) {
    let (camera_projection, mut transform) = match camera_query.get_single_mut() {
        Ok((Projection::Perspective(projection), transform)) => (projection, transform),
        _ => unreachable!(),
    };

    let distance_to_target = (CAMERA_TARGET - original_camera_transform.translation).length();
    let frustum_height = 2.0 * distance_to_target * (camera_projection.fov * 0.5).tan();
    let frustum_width = frustum_height * camera_projection.aspect_ratio;

    let window = windows.single();

    let left_taken = occupied_screen_space.left / window.width();
    let right_taken = occupied_screen_space.right / window.width();
    let top_taken = occupied_screen_space.top / window.height();
    let bottom_taken = occupied_screen_space.bottom / window.height();
    transform.translation = original_camera_transform.translation
        + transform.rotation.mul_vec3(Vec3::new(
            (right_taken - left_taken) * frustum_width * 0.5,
            (top_taken - bottom_taken) * frustum_height * 0.5,
            0.0,
        ));
}
