use gloo_net::http::Request;
use leptos::*;
use serde::{Deserialize, Serialize};

// Base API URL - adjust this to match your backend
const API_BASE: &str = "http://localhost:3000";

// Auth token management
pub fn get_token() -> Option<String> {
    let window = web_sys::window()?;
    let storage = window.local_storage().ok()??;
    storage.get_item("auth_token").ok()?
}

pub fn set_token(token: &str) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.set_item("auth_token", token);
        }
    }
}

pub fn clear_token() {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.remove_item("auth_token");
        }
    }
}

pub fn is_authenticated() -> bool {
    get_token().is_some()
}

// API Models
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct User {
    pub id: i32,
    pub fullname: String,
    pub email: String,
    pub role: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RegisterRequest {
    pub fullname: String,
    pub email: String,
    pub password: String,
    pub role: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UpsertUser {
    pub fullname: String,
    pub email: String,
    pub password: String,
    pub role: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Location {
    pub id: i32,
    pub star_system: String,
    pub area: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UpsertLocation {
    pub star_system: String,
    pub area: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Empire {
    pub id: i32,
    pub name: String,
    pub slogan: String,
    pub location_id: i32,
    pub description: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UpsertEmpire {
    pub name: String,
    pub slogan: String,
    pub location_id: i32,
    pub description: String,
}

// Backend returns just the token string, not an object

// API Functions
pub async fn login(email: String, password: String) -> Result<String, String> {
    let request = LoginRequest { email, password };
    
    let response = Request::post(&format!("{}/users/login", API_BASE))
        .header("Content-Type", "application/json")
        .json(&request)
        .map_err(|e| format!("Failed to create request: {:?}", e))?
        .send()
        .await
        .map_err(|e| format!("Request failed: {:?}", e))?;

    if response.ok() {
        let token: String = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {:?}", e))?;
        
        set_token(&token);
        Ok(token)
    } else {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        Err(format!("Login failed: {}", error_text))
    }
}

pub async fn register(fullname: String, email: String, password: String, role: String) -> Result<User, String> {
    let request = RegisterRequest { fullname, email, password, role };
    
    let response = Request::post(&format!("{}/users", API_BASE))
        .header("Content-Type", "application/json")
        .json(&request)
        .map_err(|e| format!("Failed to create request: {:?}", e))?
        .send()
        .await
        .map_err(|e| format!("Request failed: {:?}", e))?;

    if response.ok() {
        let user: User = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {:?}", e))?;
        Ok(user)
    } else {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        Err(format!("Registration failed: {}", error_text))
    }
}

// Helper function to create authenticated requests
fn authenticated_request(method: &str, url: &str) -> Result<gloo_net::http::RequestBuilder, String> {
    let token = get_token().ok_or("No authentication token found")?;
    
    let request = match method {
        "GET" => Request::get(url),
        "POST" => Request::post(url),
        "PUT" => Request::put(url),
        "DELETE" => Request::delete(url),
        _ => return Err("Unsupported HTTP method".to_string()),
    };
    
    let request = request
        .header("Authorization", &format!("Bearer {}", token))
        .header("Content-Type", "application/json");
    
    Ok(request)
}

// Helper function to handle API response errors
async fn handle_api_error(response: gloo_net::http::Response) -> String {
    if response.status() == 401 {
        "Not authenticated - Please log in".to_string()
    } else {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        format!("Request failed: {}", error_text)
    }
}

// Location API functions
pub async fn get_locations() -> Result<Vec<Location>, String> {
    let response = match authenticated_request("GET", &format!("{}/locations", API_BASE)) {
        Ok(req) => req.send().await.map_err(|e| format!("Request failed: {:?}", e))?,
        Err(auth_error) => {
            if auth_error.contains("No authentication token found") {
                return Err("Not authenticated - Please log in".to_string());
            }
            return Err(auth_error);
        }
    };

    if response.ok() {
        let locations: Vec<Location> = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {:?}", e))?;
        Ok(locations)
    } else {
        Err(handle_api_error(response).await)
    }
}

pub async fn create_location(location: UpsertLocation) -> Result<Location, String> {
    let response = authenticated_request("POST", &format!("{}/locations", API_BASE))?
        .json(&location)
        .map_err(|e| format!("Failed to serialize location: {:?}", e))?
        .send()
        .await
        .map_err(|e| format!("Request failed: {:?}", e))?;

    if response.ok() {
        let location: Location = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {:?}", e))?;
        Ok(location)
    } else {
        Err("Failed to create location".to_string())
    }
}

pub async fn update_location(id: i32, location: UpsertLocation) -> Result<Location, String> {
    let response = authenticated_request("PUT", &format!("{}/locations/{}", API_BASE, id))?
        .json(&location)
        .map_err(|e| format!("Failed to serialize location: {:?}", e))?
        .send()
        .await
        .map_err(|e| format!("Request failed: {:?}", e))?;

    if response.ok() {
        let location: Location = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {:?}", e))?;
        Ok(location)
    } else {
        Err("Failed to update location".to_string())
    }
}

pub async fn delete_location(id: i32) -> Result<(), String> {
    let response = authenticated_request("DELETE", &format!("{}/locations/{}", API_BASE, id))?
        .send()
        .await
        .map_err(|e| format!("Request failed: {:?}", e))?;

    if response.ok() {
        Ok(())
    } else {
        Err("Failed to delete location".to_string())
    }
}

// Empire API functions
pub async fn get_empires() -> Result<Vec<Empire>, String> {
    let response = match authenticated_request("GET", &format!("{}/empires", API_BASE)) {
        Ok(req) => req.send().await.map_err(|e| format!("Request failed: {:?}", e))?,
        Err(auth_error) => {
            if auth_error.contains("No authentication token found") {
                return Err("Not authenticated - Please log in".to_string());
            }
            return Err(auth_error);
        }
    };

    if response.ok() {
        let empires: Vec<Empire> = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {:?}", e))?;
        Ok(empires)
    } else {
        Err(handle_api_error(response).await)
    }
}

pub async fn create_empire(empire: UpsertEmpire) -> Result<Empire, String> {
    let response = authenticated_request("POST", &format!("{}/empires", API_BASE))?
        .json(&empire)
        .map_err(|e| format!("Failed to serialize empire: {:?}", e))?
        .send()
        .await
        .map_err(|e| format!("Request failed: {:?}", e))?;

    if response.ok() {
        let empire: Empire = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {:?}", e))?;
        Ok(empire)
    } else {
        Err("Failed to create empire".to_string())
    }
}

pub async fn update_empire(id: i32, empire: UpsertEmpire) -> Result<Empire, String> {
    let response = authenticated_request("PUT", &format!("{}/empires/{}", API_BASE, id))?
        .json(&empire)
        .map_err(|e| format!("Failed to serialize empire: {:?}", e))?
        .send()
        .await
        .map_err(|e| format!("Request failed: {:?}", e))?;

    if response.ok() {
        let empire: Empire = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {:?}", e))?;
        Ok(empire)
    } else {
        Err("Failed to update empire".to_string())
    }
}

pub async fn delete_empire(id: i32) -> Result<(), String> {
    let response = authenticated_request("DELETE", &format!("{}/empires/{}", API_BASE, id))?
        .send()
        .await
        .map_err(|e| format!("Request failed: {:?}", e))?;

    if response.ok() {
        Ok(())
    } else {
        Err("Failed to delete empire".to_string())
    }
}

// User API functions
pub async fn get_users() -> Result<Vec<User>, String> {
    let response = match authenticated_request("GET", &format!("{}/users", API_BASE)) {
        Ok(req) => req.send().await.map_err(|e| format!("Request failed: {:?}", e))?,
        Err(auth_error) => {
            if auth_error.contains("No authentication token found") {
                return Err("Not authenticated - Please log in".to_string());
            }
            return Err(auth_error);
        }
    };

    if response.ok() {
        let users: Vec<User> = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {:?}", e))?;
        Ok(users)
    } else {
        Err(handle_api_error(response).await)
    }
}

pub async fn get_user(id: i32) -> Result<User, String> {
    let response = authenticated_request("GET", &format!("{}/users/{}", API_BASE, id))?
        .send()
        .await
        .map_err(|e| format!("Request failed: {:?}", e))?;

    if response.ok() {
        let user: User = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {:?}", e))?;
        Ok(user)
    } else {
        Err(handle_api_error(response).await)
    }
}

pub async fn update_user(id: i32, user: UpsertUser) -> Result<User, String> {
    let response = authenticated_request("PUT", &format!("{}/users/{}", API_BASE, id))?
        .json(&user)
        .map_err(|e| format!("Failed to serialize user: {:?}", e))?
        .send()
        .await
        .map_err(|e| format!("Request failed: {:?}", e))?;

    if response.ok() {
        let user: User = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {:?}", e))?;
        Ok(user)
    } else {
        Err("Failed to update user".to_string())
    }
}

pub async fn delete_user(id: i32) -> Result<(), String> {
    let response = authenticated_request("DELETE", &format!("{}/users/{}", API_BASE, id))?
        .send()
        .await
        .map_err(|e| format!("Request failed: {:?}", e))?;

    if response.ok() {
        Ok(())
    } else {
        Err("Failed to delete user".to_string())
    }
}