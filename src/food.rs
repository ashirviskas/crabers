use bevy::color::palettes::css::BLUE;
use bevy::prelude::*;
use avian2d::prelude::*;

use rand::RngExt;

use crate::common::*;

pub const FOOD_SIZE: f32 = 10.0;

#[derive(Component)]
pub struct Food {
    pub energy_value: f32,
}

pub fn food_spawner(
    time: Res<Time>,
    mut timer: ResMut<FoodSpawnTimer>,
    mut food_spawn_event: MessageWriter<FoodSpawnEvent>,
    food_query: Query<&Food>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        if food_query.iter().count() >= crate::MAX_FOOD_COUNT {
            return;
        }
        let mut rng = rand::rng();
        let position = Vec2::new(
            rng.random_range((WORLD_SIZE * -1.)..WORLD_SIZE),
            rng.random_range((WORLD_SIZE * -1.)..WORLD_SIZE),
        );
        let energy_value = rng.random_range(5.0..15.0);
        food_spawn_event.write(FoodSpawnEvent {
            transform: Transform::from_translation(position.extend(0.0)),
            food_energy: energy_value,
        });
    }
}

pub fn spawn_food(
    mut commands: Commands<'_, '_>,
    mut food_spawn_event: MessageReader<FoodSpawnEvent>,
) {
    for event in food_spawn_event.read() {
        if event.food_energy < 0. {
            continue;
        }
        commands
            .spawn((
                Sprite {
                    color: Color::from(BLUE),
                    custom_size: Some(Vec2::new(FOOD_SIZE, FOOD_SIZE)),
                    ..default()
                },
                event.transform,
            ))
            .insert(Collider::circle(FOOD_SIZE / 2.0))
            .insert(Food {
                energy_value: event.food_energy,
            })
            .insert(SelectableEntity::Food)
            .insert(EntityType::Food)
            .insert(CollisionLayers::new(
                [Layer::Food],
                [Layer::Food, Layer::Craber, Layer::Vision],
            ))
            .insert(Weight { weight: 1.0 });
    }
}
