use bevy::{
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::{dynamics::{Damping, RigidBody}, rapier::geometry::ColliderType, render::RapierDebugRenderPlugin};
use bevy_rapier3d::{
    geometry::Collider,
    parry::shape::Cuboid,
    plugin::{NoUserData, RapierPhysicsPlugin},
    prelude::GravityScale,
};
use core::fmt;
use std::{alloc::System, f32::consts::PI};

#[derive(Resource)]
struct MiniMapConfig {
    cell_size: f32, // Adjust the size of each cell
    cell_mul: f32,
}

#[derive(Component)]
pub struct Player {
    collider: ColliderT,
}

#[derive(Component)]
struct Shape;

#[derive(Component)]
struct MiniMap;

#[derive(Component)]
struct MiniMapCell;

#[derive(Component)]
struct PlayerOnMinimap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColliderTypes {
    Player,
    Wall,
}

#[derive(Component)]
pub struct ColliderT {
    pub collider_type: ColliderTypes,
}
const X_EXTENT: f32 = 14.5;

impl Plugin for Player {
    fn build(&self, app: &mut App) {
        println!("ca marche");
        app.add_plugins(WorldInspectorPlugin::new())
            .add_systems(Update, player_movement)
            .add_systems(Update, player_camera_movement)
            .add_systems(Update, player_camera_follow)
            .add_systems(Update, player_update_minimap)
            .add_systems(Update, player_movement_collision);
    }
}

fn player_update_minimap(
    mut player_dot: Query<&mut Style, With<PlayerOnMinimap>>,
    query: Query<&Transform, With<Player>>,
) {
    for mut player_dot in &mut player_dot {
        for transform in &query {
            println!("{:?}", transform.translation);
            let position = Vec2::new(
                transform.translation.x * 20. + 20.,
                transform.translation.z * 20. + 150.,
            );

            player_dot.left = Val::Px(position.x);
            player_dot.top = Val::Px(position.y);
        }
    }
}

#[derive(Debug, Hash, Eq, PartialEq, Clone, Default)]
pub struct GameState;

impl States for GameState {}

fn main() {
    println!("je suis ici");
    App::new()
        .add_state::<GameState>()
        .add_systems(Startup, (spawn_player, setup.after(spawn_player)))
        .add_systems(Startup, rotate)
        .add_systems(Startup, setup_fps_counter)
        .add_systems(Update, (fps_text_update_system, fps_counter_showhide))
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins((DefaultPlugins, RapierPhysicsPlugin::<NoUserData>::default()))
        .add_plugins(Player {
            collider: ColliderT {
                collider_type: ColliderTypes::Player,
            },
        })
        .insert_resource(MiniMapConfig {
            cell_size: 0.9,
            cell_mul: 30.,
        })
        .run();
}

fn spawn_player(mut commands: Commands) {
    commands
        .spawn((
            Camera3dBundle {
                transform: Transform::from_xyz(2.0, 0.5, -1.0), /*
                                                                Vec3::new(maze_width / 2.0, 0.0, -maze_height / 2.0),
                                                                Vec3::Y,*/
                ..Default::default()
            },
            Player {
                collider: ColliderT {
                    collider_type: ColliderTypes::Player,
                },
            },
            ColliderT {
                collider_type: ColliderTypes::Player,
            },
            Name::new("Player"),
        ))
        .insert(RigidBody::KinematicPositionBased)
        .insert(Collider::cuboid(0.1, 1.0, 0.1))
        .insert(Damping{ linear_damping: 0.5, angular_damping: 100.0});
}

fn setup(
    mut commands: Commands,
    //asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mm_config: Res<MiniMapConfig>,
    //player_transform: Query<&Transform, With<Player>>,
) {
    //println!("c'est bon 2");
    // Define the maze layout. '1' represents a wall, '0' represents an open space.
    let maze_layout = vec![
        vec![1, 1, 1, 1, 1, 1, 1],
        vec![1, 0, 0, 0, 0, 0, 1],
        vec![1, 0, 0, 0, 0, 0, 1],
        vec![1, 0, 0, 1, 0, 0, 1],
        vec![1, 0, 0, 0, 0, 0, 1],
        vec![1, 0, 0, 0, 0, 0, 1],
        vec![1, 1, 1, 1, 1, 1, 1],
    ];

    // Size of each maze case (adjust based on your model's scale)
    let case_size = 1.0;

    // Create mini-map cells
    for (row_index, row) in maze_layout.iter().enumerate() {
        for (col_index, &cell) in row.iter().enumerate() {
            /*let translation = Vec3::new(
                col_index as f32 * case_size,
                row_index as f32 * case_size,
                5.0,
            );*/

            let color = if cell == 1 {
                Color::rgb(0.5, 0.5, 0.5) // Gray square for ground
            } else {
                Color::rgb(0.1, 0.1, 0.1) // Dark square for walls
            };

            commands
                .spawn(NodeBundle {
                    style: Style {
                        left: Val::Px(
                            col_index as f32 * (mm_config.cell_size * mm_config.cell_mul),
                        ),
                        top: Val::Px(row_index as f32 * (mm_config.cell_size * mm_config.cell_mul)),
                        height: Val::Px(mm_config.cell_size * mm_config.cell_mul),
                        width: Val::Px(mm_config.cell_size * mm_config.cell_mul),
                        ..Default::default()
                    },
                    background_color: color.into(),
                    ..Default::default()
                })
                .insert((MiniMapCell, Name::new("MinimapCell")));
        }
    }

    commands.spawn((
        NodeBundle {
            style: Style {
                left: Val::Px(0.),
                top: Val::Px(0.),
                height: Val::Px(mm_config.cell_size * (mm_config.cell_mul / 2.)),
                width: Val::Px(mm_config.cell_size * (mm_config.cell_mul / 2.)),
                //height: Val::Px(30.),
                //width: Val::Px(30.),
                ..default()
            },
            background_color: Color::RED.into(),
            ..default()
        },
        PlayerOnMinimap,
        Name::new("MinimapPlayer"),
    ));

    /*// Calculate the size of the maze based on the layout
    let maze_width = maze_layout[0].len() as f32 * case_size;
    let maze_height = maze_layout.len() as f32 * case_size;*/

    // Spawn maze based on layout
    for (row_index, row) in maze_layout.iter().enumerate() {
        for (col_index, &cell) in row.iter().enumerate() {
            if cell == 0 {
                /*let translation = Vec3::new(
                    col_index as f32 * case_size,
                    0.0,
                    -(row_index as f32) * case_size,
                );*/
                /*commands.spawn(PbrBundle {
                    mesh: asset_server.load("tile.gltf#Mesh0/Primitive0"),
                    material: materials.add(Color::rgb(0.0, 0.0, 1.0).into()), // bleu
                    transform: Transform::from_translation(translation),
                    ..Default::default()
                });*/
            } else if cell == 1 {
                let wall_height = 3.0; // Adjust this value based on your desired height
                let translation = Vec3::new(
                    col_index as f32 * case_size,
                    0.0,
                    -(row_index as f32) * case_size,
                );
                let scale = Vec3::new(1.0, wall_height, 1.0); // Adjust only the Y component for height

                commands
                    .spawn(PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 }).into()),
                        material: materials.add(Color::rgb(00.0, 1.0, 0.0).into()), // Vert
                        /*transform: Transform::from_translation(translation)
                            .mul_transform(Transform::from_scale(scale)),
                        */..Default::default()
                    })
                    .insert(ColliderT {
                        collider_type: ColliderTypes::Wall,
                    })
                     .insert(Collider::cuboid(0.6, 1.0, 0.6))
                     .insert(TransformBundle::from(Transform::from_translation(translation)));

                /*// Spawn a mirrored wall on the opposite side
                let mirrored_translation = Vec3::new(
                    (col_index as f32 * case_size) * -1.0,
                    0.0,
                    -(row_index as f32) * case_size,
                );*/
                /*commands.spawn(PbrBundle {
                    mesh: asset_server.load("tile.gltf#Mesh0/Primitive0"),
                    material: materials.add(Color::rgb(0.3, 0.3, 0.3).into()),
                    transform: Transform::from_translation(mirrored_translation)
                        .mul_transform(Transform::from_scale(scale)),
                    ..Default::default()
                });*/
            }
        }
    }
    let debug_material = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(uv_debug_texture())),
        ..default()
    });

    let shapes = [meshes.add(shape::UVSphere::default().into())];

    let num_shapes = shapes.len();

    for (i, shape) in shapes.into_iter().enumerate() {
        commands.spawn((
            PbrBundle {
                mesh: shape,
                material: debug_material.clone(),
                transform: Transform::from_xyz(
                    -X_EXTENT / 2. + i as f32 / (num_shapes - 1) as f32 * X_EXTENT,
                    2.0,
                    0.0,
                )
                .with_rotation(Quat::from_rotation_x(-PI / 4.)),
                ..default()
            },
            Shape,
        ));
    }

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 9000.0,
            range: 100.,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(8.0, 16.0, 8.0),
        ..default()
    });

    // ground plane
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(shape::Plane::from_size(50.0).into()),
            material: materials.add(Color::rgb(1.0, 0.0, 0.0).into()), // rouge
            ..default()
        })
        .insert(Collider::cuboid(40.0, 1.0, 40.0));

    /*commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 6., 12.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
        ..default()
    });*/

    // Spawns the player and camera together

    // Spawn player
    /* commands.spawn(SceneBundle {
        scene: asset_server.load("alien.gltf#Scene0"),
        //material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
        transform: Transform::from_xyz(0.0, 0.0, 0.0), // Adjust the player position
        ..Default::default()
    });*/
}

fn check_collision<'a>(
    player_position: Vec3,
    colliders: impl Iterator<Item = &'a Transform>,
) -> bool {
    const COLLISION_RADIUS: f32 = 0.7; // Adjust the collision radius as needed

    for collider_transform in colliders {
        // Skip the player's own collider
        if collider_transform.translation == player_position {
            continue;
        }

        let distance_squared = (player_position.x - collider_transform.translation.x).powi(2)
            + (player_position.z - collider_transform.translation.z).powi(2);

        let collision_radius_squared = COLLISION_RADIUS * COLLISION_RADIUS;

        if distance_squared < collision_radius_squared {
            //future collision here
            return true; // Collision detected
        }
    }

    false // No collision
}

fn player_movement_collision(
    query: Query<(&Transform, &ColliderT), With<Player>>,
    colliders: Query<&Transform, With<ColliderT>>,
) {
    for (player_transform, _) in query.iter() {
        let player_position = player_transform.translation;

        if check_collision(player_position, colliders.iter()) {
            println!("Collision detected!");
        }
    }
}

fn player_movement(
    input: Res<Input<KeyCode>>,
    mut query: Query<(&Player, &mut Transform)>,
    //colliders: Query<&Transform, With<Collider>>,
) {
    for (_, mut transform) in query.iter_mut() {
        let mut translation = Vec3::ZERO;
        let mut rotation = Quat::IDENTITY;

        // Retrieve the player's position from the Transform component
        let player_position = transform.translation;

        if input.pressed(KeyCode::Z) {
            translation += transform.forward();
        }
        if input.pressed(KeyCode::S) {
            translation += transform.back();
        }
        if input.pressed(KeyCode::Q) {
            translation += transform.left();
        }
        if input.pressed(KeyCode::D) {
            translation += transform.right();
        }
        if input.pressed(KeyCode::Left) {
            // Rotate the player (and camera) to the left
            rotation *= Quat::from_rotation_y(0.1);
        }
        if input.pressed(KeyCode::Right) {
            // Rotate the player (and camera) to the right
            rotation *= Quat::from_rotation_y(-0.1);
        }

        // Adjust the translation speed
        let speed = 0.05;
        translation *= speed;

        // Move the player based on input
        transform.translation += translation;

        transform.rotate(rotation);
    }
}

fn player_camera_movement(
    time: Res<Time>,
    input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    //println!("player camera");
    for mut transform in query.iter_mut() {
        //println!("player camera for");
        // Implement camera rotation based on input
        if input.pressed(KeyCode::Left) {
            //println!("left");
            transform.rotate(Quat::from_rotation_y(time.delta_seconds()));
        }
        if input.pressed(KeyCode::Right) {
            //println!("right");
            transform.rotate(Quat::from_rotation_y(-time.delta_seconds()));
        }
    }
}

fn player_camera_follow(
    query: Query<(&Player, &Transform)>,
    mut cameras: Query<&mut Transform, Without<Player>>,
) {
    for (_, player_transform) in query.iter() {
        // Implement logic to follow the player's position
        // Adjust the camera's translation based on the player's position
        if let Some(mut camera_transform) = cameras.iter_mut().next() {
            camera_transform.translation.x = player_transform.translation.x;
            camera_transform.translation.y = player_transform.translation.y + 2.0;
        }
    }
}

fn rotate(mut query: Query<&mut Transform, With<Shape>>, time: Res<Time>) {
    for mut transform in &mut query {
        transform.rotate_y(time.delta_seconds() / 2.);
    }
}

/// Creates a colorful test pattern
fn uv_debug_texture() -> Image {
    const TEXTURE_SIZE: usize = 8;

    let mut palette: [u8; 32] = [
        255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255, 102, 255, 102, 255,
        198, 255, 102, 198, 255, 255, 121, 102, 255, 255, 236, 102, 255, 255,
    ];

    let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
    for y in 0..TEXTURE_SIZE {
        let offset = TEXTURE_SIZE * y * 4;
        texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
        palette.rotate_right(4);
    }

    Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb,
    )
}

//FPS display part
//------------------------------------------------------------

use bevy::diagnostic::DiagnosticsStore;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;

/// Marker to find the container entity so we can show/hide the FPS counter
#[derive(Component)]
struct FpsRoot;

/// Marker to find the text entity so we can update it
#[derive(Component)]
struct FpsText;

fn setup_fps_counter(mut commands: Commands) {
    // create our UI root node
    // this is the wrapper/container for the text
    let root = commands
        .spawn((
            FpsRoot,
            NodeBundle {
                // give it a dark background for readability
                background_color: BackgroundColor(Color::BLACK.with_a(0.5)),
                // make it "always on top" by setting the Z index to maximum
                // we want it to be displayed over all other UI
                z_index: ZIndex::Global(i32::MAX),
                style: Style {
                    position_type: PositionType::Absolute,
                    // position it at the top-right corner
                    // 1% away from the top window edge
                    right: Val::Percent(1.),
                    top: Val::Percent(1.),
                    // set bottom/left to Auto, so it can be
                    // automatically sized depending on the text
                    bottom: Val::Auto,
                    left: Val::Auto,
                    // give it some padding for readability
                    padding: UiRect::all(Val::Px(4.0)),
                    ..Default::default()
                },
                ..Default::default()
            },
        ))
        .id();
    // create our text
    let text_fps = commands
        .spawn((
            FpsText,
            TextBundle {
                // use two sections, so it is easy to update just the number
                text: Text::from_sections([
                    TextSection {
                        value: "FPS: ".into(),
                        style: TextStyle {
                            font_size: 16.0,
                            color: Color::WHITE,
                            // if you want to use your game's font asset,
                            // uncomment this and provide the handle:
                            // font: my_font_handle
                            ..default()
                        },
                    },
                    TextSection {
                        value: " N/A".into(),
                        style: TextStyle {
                            font_size: 16.0,
                            color: Color::WHITE,
                            // if you want to use your game's font asset,
                            // uncomment this and provide the handle:
                            // font: my_font_handle
                            ..default()
                        },
                    },
                ]),
                ..Default::default()
            },
        ))
        .id();
    commands.entity(root).push_children(&[text_fps]);
}

fn fps_text_update_system(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut Text, With<FpsText>>,
) {
    for mut text in &mut query {
        // try to get a "smoothed" FPS value from Bevy

        if let Some(value) = diagnostics
            .get(FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|fps| fps.smoothed())
        {
            // Format the number as to leave space for 4 digits, just in case,
            // right-aligned and rounded. This helps readability when the
            // number changes rapidly.
            text.sections[1].value = format!("{value:>4.0}");

            println!("{:?}", value);

            // Let's make it extra fancy by changing the color of the
            // text according to the FPS value:
            text.sections[1].style.color = if value >= 120.0 {
                // Above 120 FPS, use green color
                Color::rgb(0.0, 1.0, 0.0)
            } else if value >= 60.0 {
                // Between 60-120 FPS, gradually transition from yellow to green
                Color::rgb((1.0 - (value - 60.0) / (120.0 - 60.0)) as f32, 1.0, 0.0)
            } else if value >= 30.0 {
                // Between 30-60 FPS, gradually transition from red to yellow
                Color::rgb(1.0, ((value - 30.0) / (60.0 - 30.0)) as f32, 0.0)
            } else {
                // Below 30 FPS, use red color
                Color::rgb(1.0, 0.0, 0.0)
            }
        } else {
            // display "N/A" if we can't get a FPS measurement
            // add an extra space to preserve alignment
            text.sections[1].value = " N/A".into();
            text.sections[1].style.color = Color::WHITE;
        }
    }
}

/// Toggle the FPS counter when pressing F12
fn fps_counter_showhide(mut q: Query<&mut Visibility, With<FpsRoot>>, kbd: Res<Input<KeyCode>>) {
    if kbd.just_pressed(KeyCode::F12) {
        let mut vis = q.single_mut();
        *vis = match *vis {
            Visibility::Hidden => Visibility::Visible,
            _ => Visibility::Hidden,
        };
    }
}
