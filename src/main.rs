use std::env;
use std::process::exit;
use std::time::Duration;
use thirtyfour::prelude::*;
use tokio::time::sleep;
use rusqlite::{Connection, Result};

#[derive(Debug)]
struct Checkin {
    name: String,
    beer_name: String,
    brewery: String,
    drank_at: String,
    purchased_location: String,
    serving: String,
    rating: String,
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let args: Vec<String> = env::args().collect();
    let mut url = "http://localhost:".to_owned();
    if args.len() > 1 {
        url.push_str(args.get(1).unwrap());
    } else {
        eprintln!("No port specified for the webdriver");
        exit(1);
    }

    let caps = DesiredCapabilities::firefox();
    let driver = WebDriver::new(url.as_str(), caps).await?;

    driver.goto("https://untappd.com/login").await?;

    let submit = driver.find(By::Css("input[type='submit']")).await?;
    submit.wait_until().not_displayed().await?;
    sleep(Duration::from_secs(5)).await;
    println!("Letting page load...");
    let total_checkins = driver.find(By::XPath("/html/body/div[4]/div/div[2]/div[1]/div/div[2]/a[1]")).await?;
    total_checkins.click().await?;

    loop {
        let show_more = driver.find_all(By::XPath("//a[contains(.,'Show More')]")).await?;
        if !show_more.is_empty() {
            let show = show_more.get(0).unwrap();
            if show.is_displayed().await? {
                show.click().await?;
                println!("Showing more...");
                sleep(Duration::from_secs(3)).await;
            } else {
                println!("Done");
                break;
            }
        } else {
            println!("Done");
            break;
        }
    }

    println!("Parsing checkins...");
    let checkins = parse_checkins(&driver).await?;
    persist_checkins(checkins)?;

    sleep(Duration::from_secs(3)).await;
    driver.quit().await?;

    Ok(())
}

async fn parse_checkins(driver: &WebDriver) -> Result<Vec<Checkin>, WebDriverError> {
    let activity_stream = driver.find(By::XPath("//*[@id='main-stream']")).await?;
    let checkins = activity_stream.find_all(By::ClassName("item")).await?;
    let mut results: Vec<Checkin> = Vec::new();
    for checkin in checkins {
        let paragraph = checkin.find(By::ClassName("text")).await?;
        let comment = checkin.find(By::ClassName("checkin-comment")).await?;
        let rating_serving = comment.find(By::ClassName("rating-serving")).await?;

        let mut checkin_info: Vec<String> = Vec::with_capacity(4);
        let anchors = paragraph.find_all(By::Tag("a")).await?;
        for elem in anchors {
            let html = elem.inner_html().await?;
            checkin_info.push(html);
        }

        let mut purchased_location = String::from("Unknown");
        let has_purchased = comment.find_all(By::ClassName("purchased")).await?;
        if !has_purchased.is_empty() {
            purchased_location = has_purchased
                .first()
                .unwrap()
                .find(By::Tag("a"))
                .await?
                .inner_html()
                .await?;
        }

        let mut serving = String::from("Unknown");
        let has_serving = rating_serving.find_all(By::ClassName("serving")).await?;
        if !has_serving.is_empty() {
            serving = has_serving
                .first()
                .unwrap()
                .find(By::Tag("span"))
                .await?
                .inner_html()
                .await?;
        }

        let mut rating = String::from("Unrated");
        let has_rating = rating_serving.find_all(By::ClassName("caps ")).await?;
        if !has_rating.is_empty() {
            let rating_opt = has_rating
                .first()
                .unwrap()
                .attr("data-rating")
                .await?;
            if let Some(data_rating) = rating_opt {
                rating = data_rating;
            }
        }
        // TODO: Get checkin date and ID
        let unknown = "Unknown";
        let name = if let Some(elem) = checkin_info.get(0) { elem.to_owned() } else { String::from(unknown) };
        let beer_name = if let Some(elem) = checkin_info.get(1) { elem.to_owned() } else { String::from(unknown) };
        let brewery = if let Some(elem) = checkin_info.get(2) { elem.to_owned() } else { String::from(unknown) };
        let drank_at = if let Some(elem) = checkin_info.get(3) { elem.to_owned() } else { String::from(unknown) };
        let checkin = Checkin {
            name,
            beer_name,
            brewery,
            drank_at,
            purchased_location,
            serving,
            rating,
        };
        results.push(checkin);
    }
    // println!("Size: {}, {:?}", results.len(), results);
    Ok(results)
}

fn persist_checkins(checkins: Vec<Checkin>) -> Result<()> {
    let conn = Connection::open("beer.db")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS checkins (
                id INTEGER PRIMARY KEY,
                name TEXT,
                beer_name TEXT,
                brewery TEXT,
                drank_at TEXT,
                purchase_location TEXT,
                serving TEXT,
                rating TEXT
        )",
        [],
    )?;
    for checkin in checkins {
        conn.execute(
            "INSERT INTO checkins (
                    name,
                    beer_name,
                    brewery,
                    drank_at,
                    purchase_location,
                    serving,
                    rating
                ) VALUES (
                    ?1,
                    ?2,
                    ?3,
                    ?4,
                    ?5,
                    ?6,
                    ?7
                )",
            (
                checkin.name,
                checkin.beer_name,
                checkin.brewery,
                checkin.drank_at,
                checkin.purchased_location,
                checkin.serving,
                checkin.rating),
        )?;
    }
    Ok(())
}