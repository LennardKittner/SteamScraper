use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use clap::Parser;
use steam_scraper::SteamError;
use tokio::runtime::Runtime;
use std::collections::HashSet;
use std::fs;
use std::io::ErrorKind;

/// A CLI application to scrape game images from games in your steam library. 
#[derive(Parser)]
struct CLI {
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

fn main() -> Result<(), SteamError> {
    let args = CLI::parse();
    let args_copy = (args.steam_id.clone(), args.steam_api_key.clone());
    let destination = "./out";
    let rt = Runtime::new()?;

    match fs::create_dir(destination) {
        Ok(()) => (),
        Err(error) => match error.kind() {
            ErrorKind::AlreadyExists => (),
            _ => {
                return Err(error.into())
            }
        }
    }

    let games = rt.block_on(steam_scraper::get_game_id_list(&args_copy.0, &args_copy.1))?;
    let dir_content: HashSet<String> = fs::read_dir(destination)?.filter_map(|entry| 
        match entry {
            Ok(entry) => Some(entry.file_name().to_string_lossy().into_owned()),
            _ => None
        }
    ).collect();

    let progress_bar: ProgressBar = ProgressBar::new(games.len().try_into().unwrap());
    progress_bar.set_style(ProgressStyle::with_template("{msg}\n{wide_bar} {pos}/{len} {eta}  ").unwrap());
    let mut errors = String::new();
    let mut error_count = 0;
    for game in games {
        progress_bar.set_message(format!("Downloading image {}.png", game));
        if dir_content.contains(&format!("{game}.png")) {
            progress_bar.inc(1);
            continue;
        }
        if let Err(e) = rt.block_on(steam_scraper::save_image(&game, !args.disable_padding, destination)) {
            let game_name = rt.block_on(steam_scraper::get_game_name(&game)).unwrap_or("?".to_string());
            errors.push_str(&format!("Name: {} AppID: {} Error: {}\n", game_name, game, e));
            error_count += 1;
        }
        progress_bar.inc(1);
    }
    progress_bar.finish_with_message("Done!");
    if error_count >= 1 {
        eprint!("Failed to download {error_count} image{}\n{errors}", if error_count > 1 {"s"} else {""});
    }
    Ok(())
}
