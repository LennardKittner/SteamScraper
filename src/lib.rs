use std::path::Path;
use image::{GenericImageView, ImageBuffer, ImageFormat, imageops};
use reqwest::StatusCode;
use thiserror::Error;
use serde_json::Value;

// Endpoints:
//https://steamcdn-a.akamaihd.net/steam/apps/{appid}/library_600x900_2x.jpg
//http://api.steampowered.com/IPlayerService/GetOwnedGames/v0001/?key={api_key}&steamid=%{steam_id}}&format=json
//https://store.steampowered.com/api/appdetails?appids={appid}


#[derive(Error, Debug)]
pub enum SteamError {
    #[error("request failed")]
    RequestFailed(#[from] reqwest::Error),
    #[error("response parsing failed")]
    ParseError(),
    #[error("wrong api key")]
    WrongAPIKey(),
    #[error("failed with status {0}")]
    RequestStatusError(u16),
    #[error("failed to load image")]
    ImageLoadError(#[from] image::ImageError),
}

pub async fn get_game_id_list(steam_id: &str, api_key: &str) -> Result<Vec<String>, SteamError> {
    let response = reqwest::get(format!("http://api.steampowered.com/IPlayerService/GetOwnedGames/v0001/?key={}&steamid={}&format=json", api_key, steam_id)).await?;
    if response.status() == StatusCode::FORBIDDEN {
        return Err(SteamError::WrongAPIKey());
    } else if response.status() != StatusCode::OK {
        return Err(SteamError::RequestStatusError(response.status().as_u16()));
    }

    let json: Value = response.json().await?;
    let game_count: usize = json["response"]["game_count"].as_i64().ok_or(SteamError::ParseError())?.try_into().unwrap();
    let mut game_list: Vec<String> = Vec::new();
    for game in 0..game_count {
        let app_id = json["response"]["games"][game]["appid"].as_i64().ok_or(SteamError::ParseError())?.to_string();
        game_list.push(app_id);
    }
    Ok(game_list)
}

pub async fn save_image(game_id: &str, with_padding: bool, path: &str) -> Result<(), SteamError> {
    let response = reqwest::get(format!("https://steamcdn-a.akamaihd.net/steam/apps/{}/library_600x900_2x.jpg", game_id)).await?;
    if !response.status().is_success() {
        return Err(SteamError::RequestStatusError(response.status().as_u16()));
    }

    let image_data = response.bytes().await?;
    let image = image::load_from_memory(&image_data)?;
    let output_path = format!("{}/{}.png", path, game_id);
    let output_path = Path::new(&output_path);

    if !with_padding {
        image.save_with_format(output_path, ImageFormat::Png)?;
        return Ok(());
    }

    // Get the original image dimensions
    let original_width: i64 = image.dimensions().0.try_into().unwrap();
    let original_height: i64 = image.dimensions().1.try_into().unwrap();

    // Define the desired resolution
    let desired_width: i64 = 900;
    let desired_height: i64 = 900;

    // Calculate the padding dimensions
    let padding_width: i64 = (desired_width - original_width) / 2;
    let padding_height: i64 = (desired_height - original_height) / 2;

    // Create a new blank canvas with the desired resolution
    let mut canvas = ImageBuffer::new(desired_width.try_into().unwrap(), desired_height.try_into().unwrap());

    // Paste the original image onto the canvas at the appropriate position
    imageops::overlay(&mut canvas, &image, padding_width, padding_height);

    // Save the padded image as a new PNG file
    canvas.save_with_format(output_path, ImageFormat::Png)?;
    Ok(())
}


pub async fn get_game_name(game_id: &str) -> Result<String, SteamError> {
    let response = reqwest::get(format!("https://store.steampowered.com/api/appdetails?appids={}", game_id)).await?;
    if !response.status().is_success() {
        return Err(SteamError::RequestStatusError(response.status().as_u16()));
    }
    let json: Value = response.json().await?;
    if !json[game_id]["success"].as_bool().unwrap_or(false) {
        return Err(SteamError::ParseError());
    }
    let name = json[game_id]["data"]["name"].as_str().ok_or(SteamError::ParseError())?;
    Ok(name.to_string())
}
