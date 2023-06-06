use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use clap::Parser;
use tokio::runtime::Runtime;
use std::fs;
use std::io;

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

fn main() {
    let args = Cli::parse();
    let args_copy = (args.steam_id.clone(), args.steam_api_key.clone());
    let destination = "./out";
    let rt = match Runtime::new() {
        Ok(rt) => rt,
        Err(error) => {
            eprintln!("Error: {}", error.to_string());
            std::process::exit(1);
        }
    };

    match fs::create_dir(destination) {
        Ok(()) => (),
        Err(error) => match error.kind() {
            io::ErrorKind::AlreadyExists => (),
            _ => {
                eprintln!("Error: {}", error);
                std::process::exit(1);
            }
        }
    }

    let games = match rt.block_on(steam_scraper::get_game_id_list(&args_copy.0, &args_copy.1)) {
        Ok(games) => games,
        Err(error) => {
            eprintln!("Error: {}", error);
            std::process::exit(1);
        }
    };
    let progress_bar: ProgressBar = ProgressBar::new(games.len().try_into().unwrap());
    progress_bar.set_style(ProgressStyle::with_template("{msg}\n{wide_bar} {pos}/{len} {eta}  ").unwrap());
    let mut errors = String::new();
    let mut error_count = 0;
    for game in games {
        progress_bar.set_message(format!("Downloading image {}.png", game));
        if let Err(e) = rt.block_on(steam_scraper::save_image(&game, !args.disable_padding, destination)) {
            let game_name = rt.block_on(steam_scraper::get_game_name(&game)).unwrap_or("?".to_string());
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
