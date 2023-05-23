use std::path::Path;
use image::{GenericImageView, ImageBuffer, ImageFormat, imageops};
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use reqwest::StatusCode;
use thiserror::Error;
use serde_json::Value;
use clap::Parser;
use std::fs;
use std::io;

//https://steamcdn-a.akamaihd.net/steam/apps/{appid}/library_600x900_2x.jpg
//http://api.steampowered.com/IPlayerService/GetOwnedGames/v0001/?key={api_key}&steamid=%{steam_id}}&format=json
//https://store.steampowered.com/api/appdetails?appids={appid}

/// A CLI application to scrape game images from games in your steam library. 
#[derive(Parser)]
struct Cli {
    /// Your Steam ID
    #[arg(long = "steamID")]
    steam_id: String,
    /// A Steam web API key
    #[arg(long = "steamAPI")]
    steam_api_key: String,
    /// Disable image padding
    #[arg(long = "disablePadding", default_value_t = false)]
    disable_padding: bool
}

#[derive(Error, Debug)]
enum SteamError {
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

fn get_game_id_list(steam_id: &String, api_key: &String) -> Result<Vec<String>, SteamError> {
    let response = reqwest::blocking::get(format!("http://api.steampowered.com/IPlayerService/GetOwnedGames/v0001/?key={}&steamid={}&format=json", api_key, steam_id))?;
    if response.status() == StatusCode::FORBIDDEN {
        return Err(SteamError::WrongAPIKey());
    } else if response.status() != StatusCode::OK {
        return Err(SteamError::RequestStatusError(response.status().as_u16()));
    }

    let json: Value = response.json()?;
    let game_count: usize = json["response"]["game_count"].as_i64().ok_or(SteamError::ParseError())?.try_into().unwrap();
    let mut game_list: Vec<String> = Vec::new();
    for game in 0..game_count {
        let app_id = json["response"]["games"][game]["appid"].as_i64().ok_or(SteamError::ParseError())?.to_string().clone();
        game_list.push(app_id);
    }
    Ok(game_list)
}

fn save_image(game_id: &String, with_padding: bool) -> Result<(), SteamError> {
    let response = reqwest::blocking::get(format!("https://steamcdn-a.akamaihd.net/steam/apps/{}/library_600x900_2x.jpg", game_id))?;
    if !response.status().is_success() {
        return Err(SteamError::RequestStatusError(response.status().as_u16()));
    }

    let image_data = response.bytes()?;
    let image = image::load_from_memory(&image_data)?;
    let output_path = format!("./out/{}.png", game_id);
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

fn get_game_name(game_id: &String) -> Result<String, SteamError> {
    let response = reqwest::blocking::get(format!("https://store.steampowered.com/api/appdetails?appids={}", game_id))?;
    if !response.status().is_success() {
        return Err(SteamError::RequestStatusError(response.status().as_u16()));
    }
    let json: Value = response.json()?;
    if !json[game_id]["success"].as_bool().unwrap_or(false) {
        return Err(SteamError::ParseError());
    }
    let name = json[game_id]["data"]["name"].as_str().ok_or(SteamError::ParseError())?;
    Ok(name.to_string())
}

fn main() {
    let args = Cli::parse();
    let args_copy = (args.steam_id.clone(), args.steam_api_key.clone());
   
    match fs::create_dir("./out") {
        Ok(()) => (),
        Err(error) => match error.kind() {
            io::ErrorKind::AlreadyExists => (),
            _ => {
                eprintln!("Error: {}", error);
                std::process::exit(1);
            }
        }
    }
    let games = get_game_id_list(&args_copy.0, &args_copy.1).expect("err");
    let progress_bar: ProgressBar = ProgressBar::new(games.len().try_into().unwrap());
    progress_bar.set_style(ProgressStyle::with_template("{msg}\n{wide_bar} {pos}/{len} {eta}  ").unwrap());
    let mut errors = String::new();
    let mut error_count = 0;
    for game in games {
        progress_bar.set_message(format!("Downloading image {}.png", game));
        if let Err(e) = save_image(&game, !args.disable_padding) {
            let game_name = get_game_name(&game).unwrap_or("?".to_string());
            errors.push_str(&format!("Name: {} AppID: {} Error: {}\n", game_name, game, e));
            error_count += 1;
        }
        progress_bar.inc(1);
    }
    progress_bar.finish_with_message("Done!");
    if error_count > 1 {
        eprint!("Failed to download {error_count} image(s)\n{errors}");
    }
}
