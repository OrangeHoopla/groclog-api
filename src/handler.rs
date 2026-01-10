use std::fs;

use axum::{
    Json,
    extract::{Multipart, Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use grocery_to_json::{grocery_list::GroceryList, imageproc::ImageProc, reciept::Reciept, tesseract::Tesseract};
use image::ImageReader;
use uuid::Uuid;

use crate::{
    model::{DB, Todo, UpdateTodoSchema},
    response::{SingleTodoResponse, TodoData, TodoListResponse},
};

pub async fn health_checker_handler() -> impl IntoResponse {
    const MESSAGE: &str = "Build Simple CRUD API in Rust using Axum";

    let json_response = serde_json::json!({
        "status": "success",
        "message": MESSAGE
    });

    Json(json_response)
}

pub async fn todos_list_handler() -> impl IntoResponse {
    let json_response = TodoListResponse {
        status: "success".to_string(),
        results: 0,
        todos: vec![],
    };

    Json(json_response)
}

pub async fn create_todo_handler(
    State(db): State<DB>,
    Json(mut body): Json<Todo>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let mut vec = db.lock().await;

    if let Some(todo) = vec.iter().find(|todo| todo.title == body.title) {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": format!("Todo with title: '{}' already exists", todo.title),
        });
        return Err((StatusCode::CONFLICT, Json(error_response)));
    }

    let uuid_id = Uuid::new_v4();
    let datetime = chrono::Utc::now();

    body.id = Some(uuid_id.to_string());
    body.completed = Some(false);
    body.createdAt = Some(datetime);
    body.updatedAt = Some(datetime);

    let todo = body.to_owned();

    vec.push(body);

    let json_response = SingleTodoResponse {
        status: "success".to_string(),
        data: TodoData { todo },
    };

    Ok((StatusCode::CREATED, Json(json_response)))
}

pub async fn get_todo_handler(
    Path(id): Path<Uuid>,
    State(db): State<DB>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let id = id.to_string();
    let vec = db.lock().await;

    if let Some(todo) = vec.iter().find(|todo| todo.id == Some(id.to_owned())) {
        let json_response = SingleTodoResponse {
            status: "success".to_string(),
            data: TodoData { todo: todo.clone() },
        };
        return Ok((StatusCode::OK, Json(json_response)));
    }

    let error_response = serde_json::json!({
        "status": "fail",
        "message": format!("Todo with ID: {} not found", id)
    });
    Err((StatusCode::NOT_FOUND, Json(error_response)))
}

pub async fn edit_todo_handler(
    Path(id): Path<Uuid>,
    State(db): State<DB>,
    Json(body): Json<UpdateTodoSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let id = id.to_string();
    let mut vec = db.lock().await;

    if let Some(todo) = vec.iter_mut().find(|todo| todo.id == Some(id.clone())) {
        let datetime = chrono::Utc::now();
        let title = body
            .title
            .to_owned()
            .unwrap_or_else(|| todo.title.to_owned());
        let content = body
            .content
            .to_owned()
            .unwrap_or_else(|| todo.content.to_owned());
        let completed = body.completed.unwrap_or(todo.completed.unwrap());
        let payload = Todo {
            id: todo.id.to_owned(),
            title: if !title.is_empty() {
                title
            } else {
                todo.title.to_owned()
            },
            content: if !content.is_empty() {
                content
            } else {
                todo.content.to_owned()
            },
            completed: Some(completed),
            createdAt: todo.createdAt,
            updatedAt: Some(datetime),
        };
        *todo = payload;

        let json_response = SingleTodoResponse {
            status: "success".to_string(),
            data: TodoData { todo: todo.clone() },
        };
        Ok((StatusCode::OK, Json(json_response)))
    } else {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": format!("Todo with ID: {} not found", id)
        });

        Err((StatusCode::NOT_FOUND, Json(error_response)))
    }
}

pub async fn delete_todo_handler(
    Path(id): Path<Uuid>,
    State(db): State<DB>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let id = id.to_string();
    let mut vec = db.lock().await;

    if let Some(pos) = vec.iter().position(|todo| todo.id == Some(id.clone())) {
        vec.remove(pos);
        return Ok((StatusCode::NO_CONTENT, Json("")));
    }

    let error_response = serde_json::json!({
        "status": "fail",
        "message": format!("Todo with ID: {} not found", id)
    });

    Err((StatusCode::NOT_FOUND, Json(error_response)))
}

/*
Takes Uploaded file and stores it locally
returning the id it was saved as to be called upon later
 */
pub async fn upload(_headers: HeaderMap, mut multipart: Multipart) -> impl IntoResponse {
    let id = Uuid::new_v4();

    while let Some(field) = multipart.next_field().await.unwrap() {
        let _name = field.name().unwrap().to_string();
        let data = field.bytes().await.unwrap();
        fs::write(format!("images/{}", id), data).expect("Error writing Image to File");
    }

    let res = process_image(format!("images/{}", id));

    Json(res)
}

fn process_image(location: String) -> GroceryList {
    let mut test: Reciept = ImageReader::open(location).unwrap().try_into().unwrap();
    test.crop_gray();
    test.otsu(1);
    test.apply();
    let wow: GroceryList = test.try_into().unwrap();

    wow
}
