use std::ops::{Add, Mul};

use bevy::ecs::entity::MapEntities;
use bevy::math::Vec2;
use bevy::prelude::*;
use leafwing_input_manager::action_state::ActionState;
use leafwing_input_manager::input_map::InputMap;
use leafwing_input_manager::prelude::Actionlike;
use leafwing_input_manager::InputManagerBundle;
use serde::{Deserialize, Serialize};
use tracing::info;

use lightyear::client::components::ComponentSyncMode;
use lightyear::prelude::server::{ControlledBy, Replicate, SyncTarget};
use lightyear::prelude::*;
use lightyear::shared::replication::components::NetworkRelevanceMode;
use UserAction;

use crate::shared::color_from_id;

// Player
#[derive(Bundle)]
pub(crate) struct PlayerBundle {
    id: PlayerId,
    player_text: PlayerText,
    position: Position,
    last_position: LastPosition,
    color: PlayerColor,
    replicate: Replicate,
    action_state: ActionState<Inputs>,
}

#[derive(Bundle)]
pub(crate) struct PlayerTextBundle {
    parent: PlayerParent,
    replicate: Replicate,
    player_text: PlayerText,
}

impl PlayerBundle {
    pub(crate) fn new(id: ClientId, position: Vec2) -> Self {
        let color = color_from_id(id);
        let player_text = PlayerText("Text from PlayerBundle".to_string());
        let replicate = Replicate {
            sync: SyncTarget {
                prediction: NetworkTarget::Single(id),
                interpolation: NetworkTarget::AllExceptSingle(id),
            },
            controlled_by: ControlledBy {
                target: NetworkTarget::Single(id),
                ..default()
            },
            // the default is: the replication group id is a u64 value generated from the entity (`entity.to_bits()`)
            group: ReplicationGroup::default(),
            // use network relevance for replication
            relevance_mode: NetworkRelevanceMode::InterestManagement,
            ..default()
        };
        Self {
            id: PlayerId(id),
            player_text,
            position: Position(position),
            last_position: LastPosition(position),
            color: PlayerColor(color),
            replicate,
            action_state: ActionState::default(),
        }
    }
    pub(crate) fn get_input_map() -> InputMap<Inputs> {
        InputMap::new([
            (Inputs::Right, KeyCode::ArrowRight),
            (Inputs::Right, KeyCode::KeyD),
            (Inputs::Left, KeyCode::ArrowLeft),
            (Inputs::Left, KeyCode::KeyA),
            (Inputs::Up, KeyCode::ArrowUp),
            (Inputs::Up, KeyCode::KeyW),
            (Inputs::Down, KeyCode::ArrowDown),
            (Inputs::Down, KeyCode::KeyS),
            (Inputs::Delete, KeyCode::Backspace),
            (Inputs::Spawn, KeyCode::Space),
        ])
    }
}

impl PlayerTextBundle {
    pub(crate) fn new(id: ClientId, parent: Entity) -> Self {
        let player_text = PlayerText("Server should change this...".to_string());
        Self {
            parent: PlayerParent(parent),
            player_text,
            replicate: Replicate {
                sync: SyncTarget {
                    prediction: NetworkTarget::Single(id),
                    interpolation: NetworkTarget::AllExceptSingle(id),
                },
                controlled_by: ControlledBy {
                    target: NetworkTarget::Single(id),
                    ..default()
                },
                // replicate this entity within the same replication group as the parent
                group: ReplicationGroup::default().set_id(parent.to_bits()),
                ..default()
            },
        }
    }
}

// Example of a component that contains an entity.
// This component, when replicated, needs to have the inner entity mapped from the Server world
// to the client World.
// This can be done by calling `app.add_component_map_entities::<PlayerParent>()` in your protocol,
// and deriving the `MapEntities` trait for the component.
#[derive(Component, Deserialize, Serialize, Clone, Debug, PartialEq, Reflect)]
pub struct PlayerParent(pub Entity);

impl MapEntities for PlayerParent {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        self.0 = entity_mapper.map_entity(self.0);
    }
}

// Components

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlayerId(pub ClientId);

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq, Deref, DerefMut)]
pub struct Position(pub(crate) Vec2);

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq, Deref, DerefMut)]
pub struct LastPosition(pub(crate) Vec2);

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlayerText(pub String);

impl Add for Position {
    type Output = Position;
    #[inline]
    fn add(self, rhs: Position) -> Position {
        Position(self.0.add(rhs.0))
    }
}

impl Mul<f32> for &Position {
    type Output = Position;

    fn mul(self, rhs: f32) -> Self::Output {
        Position(self.0 * rhs)
    }
}

#[derive(Component, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct PlayerColor(pub(crate) Color);

#[derive(Component, Deserialize, Serialize, Clone, Debug, PartialEq)]
// Marker component
pub struct CircleMarker;

// Channels

#[derive(Channel)]
pub struct Channel1;

// Messages

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Message1(pub usize);

// Inputs

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Reflect, Clone, Copy, Actionlike)]
pub enum Inputs {
    Up,
    Down,
    Left,
    Right,
    Delete,
    Spawn,
}

// Protocol
pub(crate) struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        // messages
        app.register_message::<Message1>(ChannelDirection::Bidirectional);
        // inputs
        app.add_plugins(LeafwingInputPlugin::<Inputs>::default());
        // components
        app.register_component::<PlayerId>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Once)
            .add_interpolation(ComponentSyncMode::Once);

        app.register_component::<Position>(ChannelDirection::Bidirectional)
            .add_prediction(ComponentSyncMode::Full)
            .add_interpolation(ComponentSyncMode::Full)
            .add_linear_interpolation_fn();

        app.register_component::<PlayerColor>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Once)
            .add_interpolation(ComponentSyncMode::Once);

        app.register_component::<PlayerText>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Simple)
            .add_interpolation(ComponentSyncMode::Simple);

        app.register_component::<CircleMarker>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Once)
            .add_interpolation(ComponentSyncMode::Once);

        app.register_component::<PlayerParent>(ChannelDirection::ServerToClient)
            .add_map_entities()
            .add_prediction(ComponentSyncMode::Once)
            .add_interpolation(ComponentSyncMode::Once);
        // channels
        app.add_channel::<Channel1>(ChannelSettings {
            mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
            ..default()
        });
    }
}
