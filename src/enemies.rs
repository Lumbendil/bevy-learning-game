use std::time::Duration;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use bevy_rand::prelude::*;
use rand_core::RngCore;

use crate::{Living, MyAssets, Player};


#[derive(Default, Component)]
pub struct Enemy;

#[derive(Default, Component)]
pub struct Fighter {
    damage: i32,
    ticker: Timer,
    collision_left: Option<Duration>,
}

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
    // let first_run = spawn_timer.timer.elapsed_secs() == 0.0;
    // TODO: Better logic for spawning. Most likely using stopwatch
    let first_run = false;
    spawn_timer.timer.tick(time.delta());

    let mut fighter_timer = Timer::from_seconds(1.0, TimerMode::Repeating);
    fighter_timer.pause();

    if first_run || spawn_timer.timer.just_finished() {
        let position = 100.0 * (rng.next_u32() as f32 / u32::MAX as f32) + 100.0;
        commands.spawn((
            RigidBody::Dynamic,
            LockedAxes::ROTATION_LOCKED,
            GravityScale(0.0),
            Velocity {
                ..default()
            },
            AdditionalMassProperties::Mass(1.0),
            Collider::ball(16.0),
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
            Living::new(5),
            Fighter {
                damage: 2,
                ticker: fighter_timer,
                collision_left: None,
            }
        ));
    }
}

pub fn enemy_chase(
    p: Query<&Transform, With<Player>>,
    mut e: Query<(&mut Velocity, &Transform), (With<Enemy>, Without<Player>)>,
) {
    let speed = 10.0;
    let player = p.get_single().expect("Player not found");

    for (mut velocity, transform) in e.iter_mut() {
        velocity.linvel = (player.translation - transform.translation).truncate().normalize() * speed;
    }
}

pub fn enemy_trigger_attack(
    time: Res<Time>,
    mut e: Query<(Entity, &mut Fighter)>,
    mut p: Query<&mut Living, With<Player>>
) {
    for (entity, mut fighter) in e.iter_mut() {
        match fighter.collision_left {
            None => {
                fighter.ticker.tick(time.delta());
            }
            Some(t) => {
                fighter.collision_left = None;
                fighter.ticker.tick(time.elapsed() - t);
            }
        };
        if fighter.ticker.just_finished() {
            let mut player = p.get_single_mut().unwrap();
            player.current_health -= fighter.damage;

            info!("{:?} deal damage {:?} - Current life: {:?}", entity, fighter.damage, player.current_health);
        }

    }
}

pub fn enemy_set_attacking(
    time: Res<Time>,
    mut collision_events: EventReader<CollisionEvent>,
    mut e: Query<&mut Fighter>,
) {

    for collision_event in collision_events.read() {
        match collision_event {
            CollisionEvent::Started(_, target, _) => {
                info!("Collision start with {:?}", target);
                e.get_mut(*target).unwrap().ticker.unpause();
            }
            CollisionEvent::Stopped(_, target, _) => {
                info!("Collision end with {:?}", target);
                let mut fighter = e.get_mut(*target).unwrap();

                fighter.ticker.pause();
                fighter.collision_left = Some(time.elapsed());
            }
        };
    }
}