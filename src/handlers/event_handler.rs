use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::NaiveDateTime;
use serde_json::{json, Value};

use crate::app_state::AppState;
use crate::errors::{require_admin_or_manager, AppError};
use crate::models::event::{
    CreateEventInput, Event, EventResponse, TeamInput, TeamMemberWithProfile, TeamWithMembers,
};
use crate::utils::jwt::Claims;
use crate::validation::validate_string;

#[derive(sqlx::FromRow)]
struct TeamRow {
    team_id: i32,
    coach_name: Option<String>,
}

#[derive(sqlx::FromRow)]
struct MemberRow {
    member_id: i32,
    reg_number: String,
    user_id: Option<i32>,
    name: Option<String>,
}

// List all events with their teams and members
pub async fn get_events(
    State(state): State<AppState>,
) -> Result<Json<Value>, AppError> {
    let events = sqlx::query_as::<_, Event>(
        "SELECT * FROM events ORDER BY event_date ASC",
    )
    .fetch_all(&state.pool)
    .await?;

    let mut response_events = Vec::new();

    for event in events {
        // Fetch teams for this event
        let teams_query = sqlx::query_as::<_, TeamRow>(
            "SELECT team_id, coach_name FROM teams WHERE event_id = $1 ORDER BY team_id ASC"
        )
        .bind(event.event_id)
        .fetch_all(&state.pool)
        .await?;

        let mut teams_response = Vec::new();

        for team in teams_query {
            // Fetch team members and join with users to get profile info
            let members = sqlx::query_as::<_, MemberRow>(
                r#"
                SELECT tm.member_id, tm.reg_number, u.user_id, u.name 
                FROM team_members tm
                LEFT JOIN users u ON tm.reg_number = u.reg_number
                WHERE tm.team_id = $1
                ORDER BY tm.member_id ASC
                "#
            )
            .bind(team.team_id)
            .fetch_all(&state.pool)
            .await?;

            let mut members_response = Vec::new();
            for member in members {
                members_response.push(TeamMemberWithProfile {
                    member_id: member.member_id,
                    reg_number: member.reg_number,
                    user_id: member.user_id,
                    name: member.name,
                });
            }

            teams_response.push(TeamWithMembers {
                team_id: team.team_id,
                coach_name: team.coach_name,
                members: members_response,
            });
        }

        response_events.push(EventResponse {
            event_id: event.event_id,
            description: event.description,
            event_date: event.event_date,
            teams: teams_response,
        });
    }

    Ok(Json(json!({
        "success": true,
        "count": response_events.len(),
        "data": response_events
    })))
}

// Create a new event (admin/manager only)
pub async fn create_event(
    claims: Claims,
    State(state): State<AppState>,
    Json(body): Json<CreateEventInput>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    require_admin_or_manager(&claims)?;

    validate_string(&body.description, "Description", 1, 10000)?;

    let event_date = NaiveDateTime::parse_from_str(&body.event_date, "%Y-%m-%dT%H:%M:%S")
        .map_err(|_| AppError::BadRequest("Invalid event_date format (expected YYYY-MM-DDTHH:MM:SS)".to_string()))?;

    let event = sqlx::query_as::<_, Event>(
        r#"INSERT INTO events (description, event_date)
           VALUES ($1, $2) RETURNING *"#,
    )
    .bind(&body.description)
    .bind(event_date)
    .fetch_one(&state.pool)
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "success": true,
            "message": "Event created",
            "data": event
        })),
    ))
}

// Delete an event (admin/manager only)
pub async fn delete_event(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    require_admin_or_manager(&claims)?;

    let result = sqlx::query("DELETE FROM events WHERE event_id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Event not found".to_string()));
    }

    Ok(Json(json!({
        "success": true,
        "message": format!("Event {} deleted", id)
    })))
}

// Update an event (admin/manager only)
pub async fn update_event(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(body): Json<CreateEventInput>,
) -> Result<Json<Value>, AppError> {
    require_admin_or_manager(&claims)?;

    validate_string(&body.description, "Description", 1, 10000)?;

    let event_date = NaiveDateTime::parse_from_str(&body.event_date, "%Y-%m-%dT%H:%M:%S")
        .map_err(|_| AppError::BadRequest("Invalid event_date format (expected YYYY-MM-DDTHH:MM:SS)".to_string()))?;

    let result = sqlx::query("UPDATE events SET description = $1, event_date = $2 WHERE event_id = $3")
        .bind(&body.description)
        .bind(event_date)
        .bind(id)
        .execute(&state.pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Event not found".to_string()));
    }

    Ok(Json(json!({
        "success": true,
        "message": "Event updated successfully"
    })))
}

// Add a team to an event (admin/manager only)
pub async fn add_team(
    claims: Claims,
    State(state): State<AppState>,
    Path(event_id): Path<i32>,
    Json(body): Json<TeamInput>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    require_admin_or_manager(&claims)?;

    if body.members.len() != 3 {
        return Err(AppError::BadRequest("A team must have exactly 3 members".to_string()));
    }

    let mut tx = state.pool.begin().await?;

    let team_id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO teams (event_id, coach_name) VALUES ($1, $2) RETURNING team_id"
    )
    .bind(event_id)
    .bind(&body.coach_name)
    .fetch_one(&mut *tx)
    .await?;

    for reg_number in &body.members {
        validate_string(reg_number, "Registration number", 1, 50)?;
        sqlx::query(
            "INSERT INTO team_members (team_id, reg_number) VALUES ($1, $2)"
        )
        .bind(team_id)
        .bind(reg_number)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "success": true,
            "message": "Team added successfully"
        })),
    ))
}

// Update a team (admin/manager only)
pub async fn update_team(
    claims: Claims,
    State(state): State<AppState>,
    Path((_event_id, team_id)): Path<(i32, i32)>,
    Json(body): Json<TeamInput>,
) -> Result<Json<Value>, AppError> {
    require_admin_or_manager(&claims)?;

    if body.members.len() != 3 {
        return Err(AppError::BadRequest("A team must have exactly 3 members".to_string()));
    }

    let mut tx = state.pool.begin().await?;

    let result = sqlx::query("UPDATE teams SET coach_name = $1 WHERE team_id = $2")
        .bind(&body.coach_name)
        .bind(team_id)
        .execute(&mut *tx)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Team not found".to_string()));
    }

    // Delete existing members
    sqlx::query("DELETE FROM team_members WHERE team_id = $1")
        .bind(team_id)
        .execute(&mut *tx)
        .await?;

    // Insert new members
    for reg_number in &body.members {
        validate_string(reg_number, "Registration number", 1, 50)?;
        sqlx::query(
            "INSERT INTO team_members (team_id, reg_number) VALUES ($1, $2)"
        )
        .bind(team_id)
        .bind(reg_number)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(Json(json!({
        "success": true,
        "message": "Team updated successfully"
    })))
}

// Delete a team (admin/manager only)
pub async fn delete_team(
    claims: Claims,
    State(state): State<AppState>,
    Path((_event_id, team_id)): Path<(i32, i32)>,
) -> Result<Json<Value>, AppError> {
    require_admin_or_manager(&claims)?;

    let result = sqlx::query("DELETE FROM teams WHERE team_id = $1")
        .bind(team_id)
        .execute(&state.pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Team not found".to_string()));
    }

    Ok(Json(json!({
        "success": true,
        "message": format!("Team {} deleted", team_id)
    })))
}
