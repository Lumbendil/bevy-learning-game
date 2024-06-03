use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use bevy_rand::prelude::*;
use rand_core::RngCore;

use crate::{MyAssets, Player};


#[derive(Default, Component)]
pub struct Enemy;

#[derive(Resource)]
pub struct Spawner {
    pub timer: Timer,
}

pub fn spawn_enemies(
    mut commands: Commands,
    time: Res<Time>,
    mut spawn_timer: ResMut<Spawner>,
    my_assets: Res<MyAssets>,
    mut rng: ResMut<GlobalEntropy<WyRand>>
) {
    let first_run = spawn_timer.timer.elapsed_secs() == 0.0;
    // TODO: Better logic for spawning. Most likely using stopwatch
    spawn_timer.timer.tick(time.delta());

    if first_run || spawn_timer.timer.just_finished() {
        let position = 100.0 * (rng.next_u32() as f32 / u32::MAX as f32) + 100.0;
        commands.spawn((
            RigidBody::KinematicPositionBased,
            KinematicCharacterController {
                offset: CharacterLength::Absolute(0.1),
                ..default()
            },
            Collider::ball(16.0),
            Friction::coefficient(0.0),
            Restitution::coefficient(0.3),
            SpriteBundle {
                transform: Transform::from_scale(Vec3::splat(2.0)).with_translation(Vec2::splat(position).extend(0.0)),
                texture: my_assets.enemies_sprite.clone(),
                ..default()
            },
            TextureAtlas {
                layout: my_assets.enemies_layout.clone(),
                index: 0,
            },
            Enemy,
        ));
    }
}

pub fn enemy_chase(
    time: Res<Time>,
    p: Query<&Transform, With<Player>>,
    mut e: Query<(&mut KinematicCharacterController, &Transform), (With<Enemy>, Without<Player>)>,
) {
    let speed = 10.0;
    let player = p.get_single().expect("Player not found");

    for (mut controller, transform) in e.iter_mut() {
        controller.translation = Some((player.translation - transform.translation).truncate().normalize() * speed * time.delta_seconds());
    }
}