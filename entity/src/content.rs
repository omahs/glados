//! `SeaORM` Entity. Generated by sea-orm-codegen 0.10.7
use crate::utils;
use alloy_primitives::B256;
use anyhow::Result;
use chrono::{DateTime, Utc};
use ethportal_api::utils::bytes::{hex_encode, hex_encode_compact};
use ethportal_api::OverlayContentKey;
use sea_orm::{entity::prelude::*, ActiveValue::NotSet, Set};
use serde::Deserialize;

/// Portal network sub-protocol. History, state, transactions etc.
#[derive(Debug, Clone, Copy, Eq, PartialEq, EnumIter, DeriveActiveEnum, Deserialize)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum SubProtocol {
    History = 0,
    State = 1,
    Beacon = 2,
}

impl SubProtocol {
    pub fn as_text(&self) -> String {
        match self {
            SubProtocol::History => "History".to_string(),
            SubProtocol::State => "State".to_string(),
            SubProtocol::Beacon => "Beacon".to_string(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct InvalidSubProtocolError;

impl TryFrom<&String> for SubProtocol {
    type Error = InvalidSubProtocolError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "history" => Ok(SubProtocol::History),
            "state" => Ok(SubProtocol::State),
            "beacon" => Ok(SubProtocol::Beacon),
            _ => Err(InvalidSubProtocolError),
        }
    }
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "content")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub protocol_id: SubProtocol,
    #[sea_orm(unique)]
    pub content_key: Vec<u8>,
    #[sea_orm(unique)]
    pub content_id: Vec<u8>,
    pub first_available_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::content_audit::Entity")]
    ContentAudit,
    #[sea_orm(has_many = "super::execution_metadata::Entity")]
    ExecutionMetadata,
}

impl Related<super::content_audit::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ContentAudit.def()
    }
}

impl Related<super::execution_metadata::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ExecutionMetadata.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

pub async fn get_or_create<T: OverlayContentKey>(
    sub_protocol: SubProtocol,
    content_key: &T,
    available_at: DateTime<Utc>,
    conn: &DatabaseConnection,
) -> Result<Model> {
    // First try to lookup an existing entry.
    if let Some(content_key_model) = Entity::find()
        .filter(Column::ProtocolId.eq(sub_protocol))
        .filter(Column::ContentKey.eq(content_key.to_bytes().as_ref()))
        .one(conn)
        .await?
    {
        // If there is an existing record, return it
        return Ok(content_key_model);
    }

    // If no record exists, create one and return it
    let content_key = ActiveModel {
        id: NotSet,
        content_id: Set(content_key.content_id().to_vec()),
        content_key: Set(content_key.to_bytes().to_vec()),
        first_available_at: Set(available_at),
        protocol_id: Set(sub_protocol),
    };
    Ok(content_key.insert(conn).await?)
}

pub async fn get<T: OverlayContentKey>(
    content_key: &T,
    conn: &DatabaseConnection,
) -> Result<Option<Model>> {
    Ok(Entity::find()
        .filter(Column::ContentKey.eq(content_key.to_bytes().as_ref()))
        .one(conn)
        .await?)
}

/// These are helper functions for glados-web.
impl Model {
    pub fn id_as_hash(&self) -> B256 {
        B256::from_slice(&self.content_id)
    }

    pub fn id_as_hex(&self) -> String {
        hex_encode(&self.content_id)
    }

    pub fn id_as_hex_short(&self) -> String {
        hex_encode_compact(&self.content_id)
    }

    pub fn key_as_hash(&self) -> B256 {
        B256::from_slice(&self.content_key)
    }

    pub fn key_as_hex(&self) -> String {
        hex_encode(&self.content_key)
    }

    pub fn key_as_hex_short(&self) -> String {
        hex_encode_compact(&self.content_key)
    }

    pub fn available_at_local_time(&self) -> String {
        self.first_available_at
            .with_timezone(&chrono::Local)
            .to_rfc2822()
    }
    pub fn available_at_humanized(&self) -> String {
        utils::time_ago(self.first_available_at, Utc::now())
    }
}
