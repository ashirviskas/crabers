use bevy::prelude::*;
use bevy::color::palettes::css::BLUE;
// use bevy_rapier2d::prelude::*;
use avian2d::prelude::*;

use rand::Rng;

use crate::common::*;

pub const FOOD_SIZE: f32 = 10.0;

#[derive(Resource)]
pub struct FoodSpawnTimer(pub Timer);

#[derive(Component)]
pub struct Food {
    pub energy_value: f32,
}

pub fn food_spawner(mut commands: Commands, time: Res<Time>, mut timer: ResMut<FoodSpawnTimer>) {
    if timer.0.tick(time.delta()).just_finished() {
        let mut rng = rand::thread_rng();
        let position = Vec2::new(
            rng.gen_range((WORLD_SIZE * -1.)..WORLD_SIZE),
            rng.gen_range((WORLD_SIZE * -1.)..WORLD_SIZE),
        );
        let energy_value = rng.gen_range(5.0..15.0);

        commands
            .spawn(SpriteBundle {
                sprite: Sprite {
                    color: Color::from(BLUE),
                    custom_size: Some(Vec2::new(FOOD_SIZE, FOOD_SIZE)),
                    ..Default::default()
                },
                transform: Transform::from_translation(position.extend(0.0)),
                ..Default::default()
            })
            .insert(Collider::from(Circle::new(FOOD_SIZE / 2.0)))
            .insert(Food { energy_value })
            .insert(SelectableEntity::Food)
            .insert(EntityType::Food)
            .insert(CollisionLayers::new(
                [Layer::Food],
                [Layer::Food, Layer::Craber, Layer::Vision],
            ))
            .insert(Weight { weight: 1.0 });
        // .insert(ActiveEvents::COLLISION_EVENTS);
    }
}
