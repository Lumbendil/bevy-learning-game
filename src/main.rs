use bevy::prelude::*;
use bevy_editor_pls::prelude::*;
use bevy_rapier2d::prelude::*;
use bevy_rand::prelude::*;
use bevy_asset_loader::prelude::*;

mod enemies;

#[derive(AssetCollection, Resource)]
struct MyAssets {
    #[asset(texture_atlas_layout(tile_size_x = 32., tile_size_y = 32., columns = 5, rows = 5))]
    characters_layout: Handle<TextureAtlasLayout>,
    #[asset(path = "32rogues/rogues.png")]
    characters_sprite: Handle<Image>,
    #[asset(texture_atlas_layout(tile_size_x = 32., tile_size_y = 32., columns = 7, rows = 8))]
    enemies_layout: Handle<TextureAtlasLayout>,
    #[asset(path = "32rogues/monsters.png")]
    enemies_sprite: Handle<Image>,
}

#[derive(States, Clone, Copy, Default, Eq, PartialEq, Hash, Debug)]
enum MyStates {
    #[default]
    AssetLoading,
    Gameplay,
}

#[derive(Default, Component)]
struct Player {
    pub speed: f32,
}

#[derive(Default, Component)]
struct MainCamera;

fn setup(
    mut commands: Commands,
    my_assets: Res<MyAssets>,
) {
    // Use only the subset of sprites in the sheet that make up the run animation
    commands.spawn((
        Camera2dBundle {
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        MainCamera,
    ));

    commands.spawn((
        Collider::ball(16.0),
        Sensor,
        ActiveEvents::COLLISION_EVENTS,
        SpriteBundle {
            transform: Transform::from_scale(Vec3::splat(2.0)).with_translation(Vec3::new(0.0, 0.0, 1.0)),
            texture: my_assets.characters_sprite.clone(),
            ..default()
        },
        TextureAtlas {
            layout: my_assets.characters_layout.clone(),
            index: 6,
        },
        Player {
            speed: 50.0,
        },
    ));

    commands.insert_resource(enemies::Spawner {
        timer: Timer::from_seconds(1.0, TimerMode::Repeating),
    });
}

fn keyboard_input(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut p: Query<(&mut Transform, &Player), With<Player>>,
) {
    let mut direction = Vec2::default();
    if keys.pressed(KeyCode::KeyW) {
        direction.y += 1.0;
    }
    if keys.pressed(KeyCode::KeyS) {
        direction.y -= 1.0;
    }
    if keys.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }

    let (mut player_transform, player_info) = p.get_single_mut().expect("Player should always be available");

    let movement = direction * time.delta_seconds() * player_info.speed;
    player_transform.translation += movement.extend(0.0);
}


fn camera_follow(
    p: Query<&Transform, With<Player>>,
    mut c: Query<&mut Transform, (With<MainCamera>, Without<Player>)>,
) {
    let player_transform = p.get_single().expect("Player should always be available");
    let mut camera = c.get_single_mut().expect("Camera should always be available");

    camera.translation = player_transform.translation;
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(EditorPlugin::default())
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins(EntropyPlugin::<WyRand>::default())
        .init_state::<MyStates>()
        .add_loading_state(
            LoadingState::new(MyStates::AssetLoading)
                .continue_to_state(MyStates::Gameplay)
                .load_collection::<MyAssets>(),
        )
        .add_systems(OnEnter(MyStates::Gameplay), setup)
        .add_systems(FixedUpdate, (
            keyboard_input,
            camera_follow.after(keyboard_input),
            enemies::enemy_chase,
            enemies::spawn_enemies,
            enemies::enemy_attack,
        ).chain().run_if(in_state(MyStates::Gameplay)))
        .run();
}
