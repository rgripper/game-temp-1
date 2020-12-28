use crate::*;

pub const BOMB: &str = "bomb";

pub fn space_to_set_bomb(
    commands: &mut Commands,
    bomb_material: Res<BombMaterial>,
    keyboard_input: Res<Input<KeyCode>>,
    bomb_position: Query<&Transform, With<Bomb>>,
    mut player_query: Query<(Entity, &Transform, &BombPower, &mut BombNumber)>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        for (entity, transform, power, mut number) in player_query.iter_mut() {
            let position = transform.translation;
            fn handle(n: f32) -> f32 {
                let a = n.floor();
                let b = n.fract();
                if b >= 0.5 {
                    (a + 1.0) * TILE_WIDTH
                } else {
                    a * TILE_WIDTH
                }
            }
            let number_x = position.x / TILE_WIDTH;
            let number_y = position.y / TILE_WIDTH;
            let one = Vec3::new(handle(number_x), handle(number_y), OBJECT_LAYER);

            let mut is_not_exist = true;
            'bomb: for bomb_position in bomb_position.iter() {
                if bomb_position.translation == one {
                    is_not_exist = false;
                    break 'bomb;
                }
            }
            if is_not_exist && number.is_enough() {
                commands
                    .spawn(SpriteBundle {
                        material: bomb_material.0.clone(),
                        sprite: Sprite::new(Vec2::new(TILE_WIDTH as f32, TILE_WIDTH as f32)),
                        transform: Transform::from_translation(one),
                        ..Default::default()
                    })
                    .with(Bomb {
                        timer: Timer::from_seconds(3.0, false),
                        player: entity,
                    })
                    .with(BombPower(power.0));
                number.current += 1;
            }
        }
    }
}

pub fn bomb_trigger(
    commands: &mut Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Bomb, &BombPower, &Transform)>,
    fire_material: Res<FireMaterial>,
    wall_query: Query<&Transform, (With<Wall>, Without<Bomb>, Without<Destructable>)>,
    mut recovery_bomb_number_events: ResMut<Events<RecoveryBombNumberEvent>>,
) {
    for (entity, mut bomb, power, transform) in query.iter_mut() {
        let translation = transform.translation;
        if bomb.timer.tick(time.delta_seconds()).finished() {
            commands
                .spawn(SpriteBundle {
                    material: fire_material.0.clone(),
                    sprite: Sprite::new(Vec2::new(TILE_WIDTH as f32, TILE_WIDTH as f32)),
                    transform: Transform::from_translation(Vec3::new(
                        translation.x,
                        translation.y,
                        OBJECT_LAYER,
                    )),
                    ..Default::default()
                })
                .with(Fire(Timer::from_seconds(0.5, false)));

            let (mut up, mut down, mut left, mut right) = (true, true, true, true);
            for i in 1..=power.0 {
                let i = i as f32;
                if up {
                    let position =
                        Vec3::new(translation.x, translation.y + i * TILE_WIDTH, OBJECT_LAYER);
                    for wall in wall_query.iter() {
                        if aabb_detection(wall.translation.x, wall.translation.y, position) {
                            up = false;
                        }
                    }
                    if up {
                        commands
                            .spawn(SpriteBundle {
                                material: fire_material.0.clone(),
                                sprite: Sprite::new(Vec2::new(
                                    TILE_WIDTH as f32,
                                    TILE_WIDTH as f32,
                                )),
                                transform: Transform::from_translation(position),
                                ..Default::default()
                            })
                            .with(Fire(Timer::from_seconds(0.5, false)));
                    }
                }

                if down {
                    let position =
                        Vec3::new(translation.x, translation.y - i * TILE_WIDTH, OBJECT_LAYER);
                    for wall in wall_query.iter() {
                        if aabb_detection(wall.translation.x, wall.translation.y, position) {
                            down = false;
                        }
                    }
                    if down {
                        commands
                            .spawn(SpriteBundle {
                                material: fire_material.0.clone(),
                                sprite: Sprite::new(Vec2::new(
                                    TILE_WIDTH as f32,
                                    TILE_WIDTH as f32,
                                )),
                                transform: Transform::from_translation(position),
                                ..Default::default()
                            })
                            .with(Fire(Timer::from_seconds(0.5, false)));
                    }
                }

                if left {
                    let position =
                        Vec3::new(translation.x - i * TILE_WIDTH, translation.y, OBJECT_LAYER);
                    for wall in wall_query.iter() {
                        if aabb_detection(wall.translation.x, wall.translation.y, position) {
                            left = false;
                        }
                    }
                    if left {
                        commands
                            .spawn(SpriteBundle {
                                material: fire_material.0.clone(),
                                sprite: Sprite::new(Vec2::new(
                                    TILE_WIDTH as f32,
                                    TILE_WIDTH as f32,
                                )),
                                transform: Transform::from_translation(position),
                                ..Default::default()
                            })
                            .with(Fire(Timer::from_seconds(0.5, false)));
                    }
                }

                if right {
                    let position =
                        Vec3::new(translation.x + i * TILE_WIDTH, translation.y, OBJECT_LAYER);
                    for wall in wall_query.iter() {
                        if aabb_detection(wall.translation.x, wall.translation.y, position) {
                            right = false;
                        }
                    }
                    if right {
                        commands
                            .spawn(SpriteBundle {
                                material: fire_material.0.clone(),
                                sprite: Sprite::new(Vec2::new(
                                    TILE_WIDTH as f32,
                                    TILE_WIDTH as f32,
                                )),
                                transform: Transform::from_translation(position),
                                ..Default::default()
                            })
                            .with(Fire(Timer::from_seconds(0.5, false)));
                    }
                }
            }
            commands.despawn(entity);
            recovery_bomb_number_events.send(RecoveryBombNumberEvent(bomb.player));
        }
    }
}

pub fn recovery_bomb_number(
    recovery_bomb_number_events: Res<Events<RecoveryBombNumberEvent>>,
    mut events_reader: Local<EventReader<RecoveryBombNumberEvent>>,
    mut player_query: Query<(Entity, &mut BombNumber), With<Player>>,
) {
    for entity in events_reader.iter(&recovery_bomb_number_events) {
        let entity = entity.0;
        'bomb_number: for (player, mut number) in player_query.iter_mut() {
            if entity == player {
                number.current -= 1;
                info!("current:{}", number.current);
                info!("max:{}", number.max);
                break 'bomb_number;
            }
        }
    }
}

pub fn despawn_fire(
    commands: &mut Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Fire)>,
) {
    for (entity, mut fire) in query.iter_mut() {
        if fire.0.tick(time.delta_seconds()).finished() {
            commands.despawn(entity);
        }
    }
}
pub fn bomb_block_player(
    commands: &mut Commands,
    bomb_query: Query<(Entity, &Transform), (With<Bomb>, Without<Wall>)>,
    player_query: Query<&Transform, With<Player>>,
) {
    for (entity, bomb_position) in bomb_query.iter() {
        for player_position in player_query.iter() {
            let x = player_position.translation.x;
            let y = player_position.translation.y;
            if !aabb_detection(x, y, bomb_position.translation) {
                commands.insert_one(entity, Wall);
            }
        }
    }
}
pub fn bomb_destruction(
    commands: &mut Commands,
    destructable_wall_query: Query<(Entity, &Transform, &Destructable), With<Destructable>>,
    fire_query: Query<&Transform, With<Fire>>,
    power_buff_material: Res<PowerBuffMaterial>,
    speed_buff_material: Res<SpeedBuffMaterial>,
    bomb_number_buff_material: Res<BombNumberBuffMaterial>,
    mut game_over_events: ResMut<Events<GameOverEvent>>,
) {
    for (entity, transform, destructable) in destructable_wall_query.iter() {
        let position = transform.translation;
        let mut need_destroy = false;
        'fire: for fire in fire_query.iter() {
            if aabb_detection(fire.translation.x, fire.translation.y, position) {
                need_destroy = true;
                break 'fire;
            }
        }

        if need_destroy {
            match destructable {
                Destructable::Player => {
                    game_over_events.send(GameOverEvent(entity));
                }
                Destructable::NormalBox => {
                    commands.despawn(entity);
                }
                Destructable::PowerBuffBox => {
                    commands.despawn(entity);
                    commands
                        .spawn(SpriteBundle {
                            material: power_buff_material.0.clone(),
                            sprite: Sprite::new(Vec2::new(TILE_WIDTH as f32, TILE_WIDTH as f32)),
                            transform: Transform::from_translation(position),
                            ..Default::default()
                        })
                        .with(Buff::PowerBuff);
                }
                Destructable::SpeedBuffBox => {
                    commands.despawn(entity);
                    commands
                        .spawn(SpriteBundle {
                            material: speed_buff_material.0.clone(),
                            sprite: Sprite::new(Vec2::new(TILE_WIDTH as f32, TILE_WIDTH as f32)),
                            transform: Transform::from_translation(position),
                            ..Default::default()
                        })
                        .with(Buff::SpeedBuff);
                }
                Destructable::BombNumberBuffBox => {
                    commands.despawn(entity);
                    commands
                        .spawn(SpriteBundle {
                            material: bomb_number_buff_material.0.clone(),
                            sprite: Sprite::new(Vec2::new(TILE_WIDTH as f32, TILE_WIDTH as f32)),
                            transform: Transform::from_translation(position),
                            ..Default::default()
                        })
                        .with(Buff::BombNumberBuff);
                }
            }
        }
    }
}