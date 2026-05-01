use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, FromRow, Serialize)]
pub struct Event {
    pub event_id: i32,
    pub description: String,
    pub event_date: NaiveDateTime,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Debug, FromRow, Serialize)]
pub struct Team {
    pub team_id: i32,
    pub event_id: Option<i32>,
    pub coach_name: Option<String>,
}

#[derive(Debug, FromRow, Serialize)]
pub struct TeamMember {
    pub member_id: i32,
    pub team_id: Option<i32>,
    pub reg_number: String,
}

// Payload for creating an event
#[derive(Debug, Deserialize)]
pub struct CreateEventInput {
    pub description: String,
    pub event_date: String, // Expecting ISO8601 or specific format
}

// Payload for creating or updating a team
#[derive(Debug, Deserialize)]
pub struct TeamInput {
    pub coach_name: Option<String>,
    pub members: Vec<String>, // Exactly 3 registration numbers
}

// Struct for the aggregated response
#[derive(Debug, Serialize)]
pub struct TeamMemberWithProfile {
    pub member_id: i32,
    pub reg_number: String,
    pub user_id: Option<i32>,
    pub name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TeamWithMembers {
    pub team_id: i32,
    pub coach_name: Option<String>,
    pub members: Vec<TeamMemberWithProfile>,
}

#[derive(Debug, Serialize)]
pub struct EventResponse {
    pub event_id: i32,
    pub description: String,
    pub event_date: NaiveDateTime,
    pub teams: Vec<TeamWithMembers>,
}
