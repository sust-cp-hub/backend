use axum::extract::{Path, State};
use axum::http::header;
use axum::response::IntoResponse;
use axum::Json;
use serde_json::{json, Value};

use crate::app_state::AppState;
use crate::errors::AppError;
use crate::models::ranker::{RankerRequest, RankerResponse};
use crate::services::ranker;

// analyze contests and return ranked results
pub async fn analyze(
    State(state): State<AppState>,
    Json(body): Json<RankerRequest>,
) -> Result<Json<Value>, AppError> {
    // validate input
    if body.title.trim().is_empty() {
        return Err(AppError::BadRequest("Title is required".to_string()));
    }
    if body.contest_ids.is_empty() {
        return Err(AppError::BadRequest(
            "At least one contest ID is required".to_string(),
        ));
    }

    // run the ranking algorithm
    let result = ranker::analyze(&body).await?;

    // cache the result for pdf download
    let session_id = uuid::Uuid::new_v4().to_string();
    {
        let mut cache = state.results_cache.lock().unwrap();
        cache.insert(session_id.clone(), result.clone());
    }

    Ok(Json(json!({
        "success": true,
        "session_id": session_id,
        "data": result
    })))
}

// generate and download a branded pdf of the rankings
pub async fn download_pdf(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    // look up cached result
    let result = {
        let cache = state.results_cache.lock().unwrap();
        cache.get(&session_id).cloned()
    };

    let result = result.ok_or(AppError::NotFound(
        "Session not found — please run /analyze first".to_string(),
    ))?;

    let pdf_bytes = generate_pdf(&result)?;

    Ok((
        [
            (header::CONTENT_TYPE, "application/pdf"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"rankings.pdf\"",
            ),
        ],
        pdf_bytes,
    ))
}

// builds a branded pdf document from the ranking results
fn generate_pdf(result: &RankerResponse) -> Result<Vec<u8>, AppError> {
    use genpdf::elements::{Paragraph, TableLayout};
    use genpdf::style;
    use genpdf::Element as _;

    let font_family = genpdf::fonts::from_files("./fonts", "LiberationSans", None)
        .map_err(|e| AppError::InternalError(format!("Failed to load fonts: {}", e)))?;

    let mut doc = genpdf::Document::new(font_family);
    doc.set_title(&result.title);

    let mut decorator = genpdf::SimplePageDecorator::new();
    decorator.set_margins(15);
    doc.set_page_decorator(decorator);

    // header: sust cp geeks branding
    doc.push(
        Paragraph::new("SUST CP Geeks")
            .styled(style::Style::new().bold().with_font_size(20)),
    );
    doc.push(Paragraph::new(""));

    // title
    doc.push(
        Paragraph::new(&result.title)
            .styled(style::Style::new().bold().with_font_size(16)),
    );
    doc.push(Paragraph::new(""));

    // summary line
    doc.push(Paragraph::new(format!(
        "Contests: {}  |  Participants: {}",
        result.total_contests, result.total_participants,
    )));
    doc.push(Paragraph::new(""));

    // rankings table
    let mut table = TableLayout::new(vec![1, 3, 2, 2, 2]);

    // table header
    let mut header_row = table.row();
    header_row.push_element(
        Paragraph::new("Rank").styled(style::Style::new().bold()),
    );
    header_row.push_element(
        Paragraph::new("Handle").styled(style::Style::new().bold()),
    );
    header_row.push_element(
        Paragraph::new("Score").styled(style::Style::new().bold()),
    );
    header_row.push_element(
        Paragraph::new("Solved").styled(style::Style::new().bold()),
    );
    header_row.push_element(
        Paragraph::new("Penalty").styled(style::Style::new().bold()),
    );
    header_row.push().ok();

    // data rows
    for p in &result.rankings {
        let mut row = table.row();
        row.push_element(Paragraph::new(p.rank.to_string()));
        row.push_element(Paragraph::new(&p.handle));
        row.push_element(Paragraph::new(format!("{:.0}", p.total_score)));
        row.push_element(Paragraph::new(p.problems_solved.to_string()));
        row.push_element(Paragraph::new(p.total_penalty.to_string()));
        row.push().ok();
    }

    doc.push(table);
    doc.push(Paragraph::new(""));

    // footer with generation date
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M UTC").to_string();
    doc.push(
        Paragraph::new(format!("Generated on {}", now))
            .styled(style::Style::new().italic().with_font_size(8)),
    );

    // render to bytes
    let mut buf = Vec::new();
    doc.render(&mut buf)
        .map_err(|e| AppError::InternalError(format!("Failed to render PDF: {}", e)))?;

    Ok(buf)
}
