// Weather Service Module

use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::state::{AppState, WeatherData, Location};

#[derive(Debug, Deserialize)]
struct OpenWeatherResponse {
    main: MainWeather,
    weather: Vec<WeatherDescription>,
    wind: Wind,
    name: String,
    sys: Sys,
}

#[derive(Debug, Deserialize)]
struct MainWeather {
    temp: f32,
    feels_like: f32,
    humidity: f32,
    pressure: f32,
}

#[derive(Debug, Deserialize)]
struct WeatherDescription {
    main: String,
    description: String,
    icon: String,
}

#[derive(Debug, Deserialize)]
struct Wind {
    speed: f32,
    deg: f32,
}

#[derive(Debug, Deserialize)]
struct Sys {
    country: String,
}

#[tauri::command]
pub async fn get_weather(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<WeatherData, String> {
    let app_state = state.lock().await;
    
    if let Some(weather) = app_state.weather_data.read().await.clone() {
        // Return cached weather if recent (less than 10 minutes old)
        let age = chrono::Utc::now() - weather.updated_at;
        if age.num_minutes() < 10 {
            return Ok(weather);
        }
    }
    
    // Fetch new weather data
    let config = app_state.config.read().await;
    let location = &config.location;
    
    if let Some(api_key) = &config.weather_api_key {
        match fetch_weather_data(location, api_key).await {
            Ok(weather) => {
                *app_state.weather_data.write().await = Some(weather.clone());
                Ok(weather)
            }
            Err(e) => {
                // Return mock data if fetch fails
                Ok(get_mock_weather())
            }
        }
    } else {
        // No API key, return mock data
        Ok(get_mock_weather())
    }
}

#[tauri::command]
pub async fn update_weather_location(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
    latitude: f64,
    longitude: f64,
    city: String,
) -> Result<(), String> {
    let app_state = state.lock().await;
    let mut config = app_state.config.write().await;
    
    config.location = Location {
        latitude,
        longitude,
        timezone: config.location.timezone.clone(),
        city,
        country: config.location.country.clone(),
    };
    
    // Save config
    drop(config);
    app_state.save_config().await
        .map_err(|e| e.to_string())?;
    
    // Clear weather cache to force refresh
    *app_state.weather_data.write().await = None;
    
    Ok(())
}

async fn fetch_weather_data(location: &Location, api_key: &str) -> Result<WeatherData> {
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?lat={}&lon={}&appid={}&units=imperial",
        location.latitude,
        location.longitude,
        api_key
    );
    
    let response = reqwest::get(&url).await?;
    
    if !response.status().is_success() {
        return Err(anyhow!("Weather API returned error: {}", response.status()));
    }
    
    let data: OpenWeatherResponse = response.json().await?;
    
    Ok(WeatherData {
        temperature: data.main.temp,
        humidity: data.main.humidity,
        pressure: data.main.pressure,
        wind_speed: data.wind.speed,
        wind_direction: data.wind.deg,
        description: data.weather.first()
            .map(|w| w.description.clone())
            .unwrap_or_else(|| "Unknown".to_string()),
        icon: data.weather.first()
            .map(|w| w.icon.clone())
            .unwrap_or_else(|| "01d".to_string()),
        updated_at: chrono::Utc::now(),
    })
}

fn get_mock_weather() -> WeatherData {
    WeatherData {
        temperature: 72.5,
        humidity: 65.0,
        pressure: 1013.25,
        wind_speed: 8.5,
        wind_direction: 180.0,
        description: "Partly cloudy".to_string(),
        icon: "02d".to_string(),
        updated_at: chrono::Utc::now(),
    }
}

pub async fn start_updates(state: Arc<Mutex<AppState>>) {
    // Start background task to update weather every 10 minutes
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(600));
        
        loop {
            interval.tick().await;
            
            let app_state = state.lock().await;
            let config = app_state.config.read().await;
            
            if let Some(api_key) = &config.weather_api_key {
                match fetch_weather_data(&config.location, api_key).await {
                    Ok(weather) => {
                        *app_state.weather_data.write().await = Some(weather);
                    }
                    Err(e) => {
                        eprintln!("Failed to update weather: {}", e);
                    }
                }
            }
        }
    });
}