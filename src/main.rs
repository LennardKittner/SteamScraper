use std::path::Path;
use image::{GenericImageView, ImageBuffer, ImageFormat, imageops};
use reqwest::StatusCode;
use thiserror::Error;
use serde_json::Value;
use clap::Parser;
use std::fs;
use std::io;
use std::thread;

//https://steamcdn-a.akamaihd.net/steam/apps/{appid}/library_600x900_2x.jpg
//http://api.steampowered.com/IPlayerService/GetOwnedGames/v0001/?key={api_key}&steamid=%{steam_id}}&format=json

#[derive(Parser)]
struct Cli {
    /// Your steam ID
    #[arg(long = "steamID")]
    steam_id: String,
    /// A steam web API key
    #[arg(long = "steamAPI")]
    steam_api_key: String,
}

#[derive(Error, Debug)]
pub enum SteamError {
    #[error("request failed")]
    RequestFailed(#[from] reqwest::Error),
    #[error("response parsing failed")]
    ParseError(),
    #[error("player not found")]
    PlayerNotFound(),
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
    let game_count: usize = json["response"]["game_count"].as_i64().ok_or(SteamError::ParseError())?.try_into().unwrap(); // error handling
    let mut game_list: Vec<String> = Vec::new();
    for game in 0..game_count {
        let app_id = json["response"]["games"][game]["appid"].as_i64().ok_or(SteamError::ParseError())?.to_string().clone();
        game_list.push(app_id);
    }
    Ok(game_list)
}

//TODO: error handling
async fn save_image(game_id: String) -> Result<(), SteamError> {
    let response = reqwest::get(format!("https://steamcdn-a.akamaihd.net/steam/apps/{}/library_600x900_2x.jpg", game_id)).await?;
    if !response.status().is_success() {
        //TODO: for some games the images does not exist
        return Err(SteamError::RequestStatusError(response.status().as_u16()));
    }

    let image_data = response.bytes().await.expect("err");
    let image = image::load_from_memory(&image_data).expect("err");
 
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
    let output_path = format!("./out/{}.png", game_id);
    let output_path = Path::new(&output_path);
    canvas.save_with_format(output_path, ImageFormat::Png)
        .expect("Failed to save padded image.");
    println!("image {}", game_id);
    Ok(())
}

//TODO: error handling
//TODO: progress bar
#[tokio::main]
async fn main() {
    let args = Cli::parse();
    let args_copy = (args.steam_id.clone(), args.steam_api_key.clone());
    let handle = thread::spawn(move || {
        get_game_id_list(&args_copy.0, &args_copy.1).expect("err")
    });

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
    let games = handle.join().unwrap();
    let mut join_handles = Vec::new();

    for game in games {
        let app_id = game.clone();
        let join_handle = tokio::spawn(save_image(app_id));
        join_handles.push(join_handle);
    }

    for join_handle in join_handles {
        match join_handle.await {
            Ok(_) => (),
            Err(error) => {
                eprintln!("Error: {}", error);
                std::process::exit(1);
            }
        }
    }
}
