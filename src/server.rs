use bevy::prelude::*;
use bevy::utils::Duration;
use bevy::utils::HashMap;
use leafwing_input_manager::prelude::{ActionState, InputMap};

use lightyear::prelude::server::*;
use lightyear::prelude::*;

use crate::protocol::*;
use crate::shared;
use crate::shared::{color_from_id, shared_movement_behaviour};

const GRID_SIZE: f32 = 200.0;
const NUM_CIRCLES: i32 = 10;
const INTEREST_RADIUS: f32 = 150.0;

// Plugin for server-specific logic
pub struct ExampleServerPlugin;

impl Plugin for ExampleServerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Global>();
        app.add_systems(Startup, init);
        // the physics/FixedUpdates systems that consume inputs should be run in this set
        app.add_systems(FixedUpdate, movement);
        app.add_systems(
            Update,
            (
                handle_connections,
                // we don't have to run interest management every tick, only every time
                // we are buffering replication messages
                interest_management.in_set(ReplicationSet::SendMessages),
                receive_message,
                check_timers,
            ),
        );
    }
}

#[derive(Resource, Default)]
pub(crate) struct Global {
    pub client_id_to_entity_id: HashMap<ClientId, Entity>,
    pub client_id_to_room_id: HashMap<ClientId, RoomId>,
}

pub(crate) fn init(mut commands: Commands) {
    commands.start_server();
    commands.spawn(
        TextBundle::from_section(
            "Server",
            TextStyle {
                font_size: 30.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            align_self: AlignSelf::End,
            ..default()
        }),
    );

    // spawn dots in a grid
    for x in -NUM_CIRCLES..NUM_CIRCLES {
        for y in -NUM_CIRCLES..NUM_CIRCLES {
            commands.spawn((
                Position(Vec2::new(x as f32 * GRID_SIZE, y as f32 * GRID_SIZE)),
                CircleMarker,
                Replicate {
                    // use rooms for replication
                    relevance_mode: NetworkRelevanceMode::InterestManagement,
                    ..default()
                },
            ));
        }
    }
}

/// Server connection system, create a player upon connection
pub(crate) fn handle_connections(
    mut room_manager: ResMut<RoomManager>,
    mut connections: EventReader<ConnectEvent>,
    mut commands: Commands,
) {
    for connection in connections.read() {
        let client_id = connection.client_id;
        let entity = commands.spawn(PlayerBundle::new(client_id, Vec2::ZERO)).id();
        let text_entity = commands.spawn(PlayerTextBundle::new(client_id, entity)).id();

        // we can control the player visibility in a more static manner by using rooms
        // we add all clients to a room, as well as all player entities
        // this means that all clients will be able to see all player entities
        room_manager.add_client(client_id, RoomId(0));
        room_manager.add_entity(entity, RoomId(0));
        commands.entity(entity).insert(TimerComponent(Timer::from_seconds(5.0, TimerMode::Once)));
        commands.entity(text_entity).insert(TimerComponent(Timer::from_seconds(5.0, TimerMode::Once)));
    }
}


#[derive(Component)]
pub struct TimerComponent(Timer);
pub(crate) fn check_timers(mut commands: Commands,
    mut timers: Query<(Entity, &mut PlayerText, &mut TimerComponent)>,
    time: Res<Time>
) {
    for (entity, mut player_text, mut timer) in &mut timers {
        timer.0.tick(time.delta());

        if timer.0.finished() {
            info!("Timer finished");
            // TODO - Why is this not replicating to the client?
            player_text.0 = "Server changed".to_string();
            commands.entity(entity).remove::<TimerComponent>();
        }
    }
}

pub(crate) fn receive_message(mut messages: EventReader<MessageEvent<Message1>>) {
    for message in messages.read() {
        info!("recv message");
    }
}

/// Here we perform more "immediate" interest management: we will make a circle visible to a client
/// depending on the distance to the client's entity
pub(crate) fn interest_management(
    mut relevance_manager: ResMut<RelevanceManager>,
    mut room_manager: ResMut<RoomManager>,
    mut player_query: Query<
        (&PlayerId, Entity, Ref<Position>, &mut LastPosition),
        (Without<CircleMarker>, With<ReplicationTarget>),
    >,
    circle_query: Query<(Entity, &Position), (With<CircleMarker>, With<ReplicationTarget>)>,
) {
    for (client_id, entity, position, last_position) in player_query.iter_mut() {
        if position.is_changed() {
            let last_room = RoomId((last_position.0.x / 200.0) as i32 as u64);
            let new_room = RoomId((position.0.x / 200.0) as i32 as u64);

            // TODO - Leaving the room and coming back breaks the replication?
            if last_room != new_room {
                info!("Client {} moved to room {} from room {}", client_id.0, new_room.0, last_room.0);
                room_manager.remove_client(client_id.0, last_room);
                room_manager.remove_entity(entity, last_room);
                room_manager.add_client(client_id.0, new_room);
                room_manager.add_entity(entity, new_room);
            }
            
            // in real game, you would have a spatial index (kd-tree) to only find entities within a certain radius
            for (circle_entity, circle_position) in circle_query.iter() {
                let distance = position.distance(**circle_position);
                if distance < INTEREST_RADIUS {
                    relevance_manager.gain_relevance(client_id.0, circle_entity);
                } else {
                    relevance_manager.lose_relevance(client_id.0, circle_entity);
                }
            }
        }
    }
    for (client_id, entity, position, mut last_position) in player_query.iter_mut() {
        last_position.0 = position.0;
    }
}

/// Read client inputs and move players
pub(crate) fn movement(
    mut position_query: Query<(&mut Position, &ActionState<Inputs>), Without<InputMap<Inputs>>>,
) {
    for (position, input) in position_query.iter_mut() {
        shared_movement_behaviour(position, input);
    }
}
