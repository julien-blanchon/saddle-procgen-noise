use saddle_procgen_noise_example_common as support;

use bevy::{
    asset::RenderAssetUsages, mesh::Indices, prelude::*, render::render_resource::PrimitiveTopology,
};
use saddle_procgen_noise::{GridRequest2, GridSpace2, NoiseBuilder, generate_heightmap};

#[derive(Resource)]
struct TerrainState {
    seed: u32,
    height_scale: f32,
}

impl Default for TerrainState {
    fn default() -> Self {
        Self {
            seed: 7,
            height_scale: 30.0,
        }
    }
}

#[derive(Component)]
struct TerrainMesh;

#[derive(Component)]
struct OverlayText;

fn main() {
    let mut app = App::new();
    support::apply_window_defaults(
        &mut app,
        "noise terrain — 3D heightmap from noise",
        (1400, 900),
        Color::srgb(0.4, 0.55, 0.75),
    );
    app.init_resource::<TerrainState>()
        .add_systems(Startup, setup)
        .add_systems(Update, (handle_input, rotate_terrain));
    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    state: Res<TerrainState>,
) {
    // Camera
    commands.spawn((
        Name::new("Terrain Camera"),
        Camera3d::default(),
        Transform::from_xyz(0.0, 50.0, 80.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Light
    commands.spawn((
        Name::new("Terrain Light"),
        DirectionalLight {
            illuminance: 8000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, 0.4, 0.0)),
    ));

    // Generate terrain
    let mesh = generate_terrain_mesh(state.seed, state.height_scale);
    commands.spawn((
        Name::new("Terrain Mesh"),
        TerrainMesh,
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.35, 0.55, 0.30),
            perceptual_roughness: 0.9,
            double_sided: true,
            cull_mode: None,
            ..default()
        })),
    ));

    // Overlay
    commands.spawn((
        Name::new("Terrain Overlay"),
        OverlayText,
        Text::new(format!(
            "TERRAIN FROM NOISE\n\nSeed: {} [Space to change]\nHeight: {:.0} [Up/Down]\n\nAuto-rotating",
            state.seed, state.height_scale
        )),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: px(12),
            ..default()
        },
    ));
}

fn handle_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<TerrainState>,
    mut meshes: ResMut<Assets<Mesh>>,
    terrain: Query<&Mesh3d, With<TerrainMesh>>,
    mut text: Single<&mut Text, With<OverlayText>>,
) {
    let mut changed = false;

    if keys.just_pressed(KeyCode::Space) {
        state.seed = state.seed.wrapping_add(1);
        changed = true;
    }
    if keys.just_pressed(KeyCode::ArrowUp) {
        state.height_scale = (state.height_scale + 5.0).min(100.0);
        changed = true;
    }
    if keys.just_pressed(KeyCode::ArrowDown) {
        state.height_scale = (state.height_scale - 5.0).max(5.0);
        changed = true;
    }

    if changed {
        if let Ok(mesh_handle) = terrain.single() {
            if let Some(mesh) = meshes.get_mut(&mesh_handle.0) {
                *mesh = generate_terrain_mesh(state.seed, state.height_scale);
            }
        }
        **text = format!(
            "TERRAIN FROM NOISE\n\nSeed: {} [Space to change]\nHeight: {:.0} [Up/Down]\n\nAuto-rotating",
            state.seed, state.height_scale
        )
        .into();
    }
}

fn rotate_terrain(time: Res<Time>, mut query: Query<&mut Transform, With<TerrainMesh>>) {
    for mut transform in query.iter_mut() {
        transform.rotate_y(0.15 * time.delta_secs());
    }
}

fn generate_terrain_mesh(seed: u32, height_scale: f32) -> Mesh {
    let resolution = 128;
    let size = 80.0;

    let recipe = NoiseBuilder::perlin()
        .seed(seed)
        .fbm()
        .octaves(6)
        .frequency(1.3)
        .lacunarity(2.1)
        .gain(0.48)
        .build();

    let grid_request = GridRequest2 {
        size: UVec2::new(resolution, resolution),
        space: GridSpace2 {
            min: Vec2::new(-3.0, -3.0),
            max: Vec2::new(3.0, 3.0),
        },
    };

    let heightmap = generate_heightmap(&recipe, &grid_request);

    let mut positions = Vec::with_capacity((resolution * resolution) as usize);
    let mut normals = Vec::with_capacity((resolution * resolution) as usize);
    let mut uvs = Vec::with_capacity((resolution * resolution) as usize);

    let half_size = size / 2.0;

    for y in 0..resolution {
        for x in 0..resolution {
            let u = x as f32 / (resolution - 1) as f32;
            let v = y as f32 / (resolution - 1) as f32;
            let idx = (y * resolution + x) as usize;
            let height = heightmap[idx] * height_scale;

            positions.push([-half_size + u * size, height, -half_size + v * size]);
            uvs.push([u, v]);
            normals.push([0.0, 1.0, 0.0]); // Placeholder, computed below
        }
    }

    // Compute normals from heightmap
    for y in 0..resolution {
        for x in 0..resolution {
            let idx = (y * resolution + x) as usize;
            let get_h = |gx: i32, gy: i32| -> f32 {
                let cx = gx.clamp(0, resolution as i32 - 1) as u32;
                let cy = gy.clamp(0, resolution as i32 - 1) as u32;
                heightmap[(cy * resolution + cx) as usize] * height_scale
            };

            let hx = x as i32;
            let hy = y as i32;
            let dx = get_h(hx + 1, hy) - get_h(hx - 1, hy);
            let dz = get_h(hx, hy + 1) - get_h(hx, hy - 1);
            let step = size / (resolution - 1) as f32 * 2.0;
            let normal = Vec3::new(-dx / step, 1.0, -dz / step).normalize();
            normals[idx] = normal.into();
        }
    }

    // Build indices
    let mut indices = Vec::with_capacity(((resolution - 1) * (resolution - 1) * 6) as usize);
    for y in 0..(resolution - 1) {
        for x in 0..(resolution - 1) {
            let tl = y * resolution + x;
            let tr = tl + 1;
            let bl = tl + resolution;
            let br = bl + 1;
            indices.extend_from_slice(&[tl, bl, tr, tr, bl, br]);
        }
    }

    // Color vertices by height
    let colors: Vec<[f32; 4]> = heightmap
        .iter()
        .map(|h| {
            let t = h.clamp(0.0, 1.0);
            if t < 0.3 {
                // Water to shore
                let local = t / 0.3;
                [0.1 + 0.1 * local, 0.2 + 0.3 * local, 0.6 - 0.2 * local, 1.0]
            } else if t < 0.6 {
                // Grass
                let local = (t - 0.3) / 0.3;
                [
                    0.2 + 0.15 * local,
                    0.5 + 0.05 * local,
                    0.25 + 0.05 * local,
                    1.0,
                ]
            } else if t < 0.85 {
                // Rock
                let local = (t - 0.6) / 0.25;
                [
                    0.45 + 0.2 * local,
                    0.42 + 0.2 * local,
                    0.35 + 0.2 * local,
                    1.0,
                ]
            } else {
                // Snow
                let local = (t - 0.85) / 0.15;
                [
                    0.8 + 0.15 * local,
                    0.8 + 0.15 * local,
                    0.85 + 0.1 * local,
                    1.0,
                ]
            }
        })
        .collect();

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
    .with_inserted_indices(Indices::U32(indices))
}
