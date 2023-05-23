# SteamScraper
A CLI application to scrape game images from games in your Steam library.

## Compatibility

The tool has been tested on Linux, Windows, and macOS.

## Usage

```
Usage: steam_scraper --steamID <STEAM_ID> --steamAPI <STEAM_API_KEY>

Options:
      --steamID <STEAM_ID>        Your Steam ID
      --steamAPI <STEAM_API_KEY>  A Steam web API key
      --disablePadding            Disable image padding
  -h, --help                      Print help
```

If available, the Steam library thumbnail will be downloaded for each game in your Steam library.
The PNG images are saved inside the `out` directory.
If padding is disabled, the images will have a resolution of 600x900. By default, the images will be padded to have a resolution of 900x900.

Your Steam ID can be found on [SteamDB](https://steamdb.info/), and you can get a Steam web API key with the following link: [https://steamcommunity.com/dev/apikey](https://steamcommunity.com/dev/apikey).

## Building

You can build SteamScraper like any other Rust application that uses Cargo:

```
cargo build --release
```

You can also download a pre-compiled version from the [releases](https://github.com/LennardKittner/SteamScraper/releases).

## FAQ

**Why do I need my own Steam web API key?**

There are a couple of reasons. I don't want to include my API key in the binary because it is hard to protect it from exfiltration. Another option would be to let the app connect to a proxy and forward the request with my API key. However, this would require me to host a server, and the Steam API rate limit could become a problem.

**If I don't have a domain, how do I get a Steam API key?**

I'm not sure, but I just used the URL of this repository and it worked, so you could try to fork it and use that URL.

**Why is the process of downloading the images so slow?**

Originally, I downloaded the images in parallel, and it was much faster. However, that resulted in DNS errors. This issue has been documented before: [https://github.com/seanmonstar/reqwest/issues/1285](https://github.com/seanmonstar/reqwest/issues/1285), and may get resolved in the future.

## Other apps

This application was designed to work with my other application that sets Discord activities directly from Steam: [Steam2Discord](https://github.com/LennardKittner/Steam2Discord).