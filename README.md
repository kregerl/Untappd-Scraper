# Untappd Scraper

Untappd scraper is a fast web scraper written in rust utilizing
the [thirtyfour](https://docs.rs/thirtyfour/latest/thirtyfour/) crate and a web driver.  
Its main purpose is to gather all the check-in data for a given user.  
For every check-in the application currently collects:

- User's name
- Beer name
- Brewery name
- Where it was drank at
- Where it was purchased
- The serving style
- Rating

If any of the fields listed above were not filled out in the check-in they will be replaced with "Unknown" except for
rating which is "Unrated".

## Usage
To use the scraper clone the repo and make sure you have firefox and cargo installed and the web driver inside the "driver" 
directory. Currently, only firefox is supported.
Once cloned, run `scrape.sh <port>` where port is the port the web driver will run on.  
Ex: `./scrape.sh 7777`

Once the browser is running login to untappd with your username and password and let the program continue from there.  
The data collected from the scraper will be saved into a sqlite database called `beer.db`.