use anyhow::Result;
use scraper::{Html, Selector};
use std::io::stdin;

enum Rating {
    NotRated,
    One,
    Two,
    Three,
    Four,
    Five,
}

struct Book {
    id: String,
    title: String,
    authors: String,
    series_name: Option<String>,
    series_index: Option<f64>,
    cover: String,
    synopsis: String,
    tags: String,
    rating: Rating,
    publisher: String,
    release_date: String,
    language: String,
}

const BOOK_PATH: &str = "https://www.kobo.com/tw/zh/ebook/";

fn main() -> Result<()> {
    let mut book_ids = Vec::<String>::new();

    println!("Enter Kobo book URLs:");
    loop {
        let mut input = String::new();
        stdin().read_line(&mut input)?;

        if input.trim() == "done" {
            println!("Done!");
            break;
        }

        let Some(book_id) = get_book_id(&input) else {
            println!("Not a Kobo book URL!");
            continue;
        };
        book_ids.push(book_id);
    }

    for book_id in book_ids {
        let book_metadata = get_book_metadata(&book_id)?;
    }

    Ok(())
}

fn get_book_id(input: &str) -> Option<String> {
    let is_not_kobo_book_url = !input.contains(BOOK_PATH);
    if is_not_kobo_book_url {
        return None;
    }

    let book_id = input
        .rsplit_once('/')
        .map(|(_, substring)| substring.trim().to_string())
        .filter(|id| !id.is_empty())?;

    Some(book_id)
}

fn get_book_metadata(id: &str) -> Result<Book> {
    let book_page = get_book_page(id)?;

    let title = get_title(&book_page);

    let authors = get_authors_str(&book_page);

    let series_name = get_series_name(&book_page);

    let series_index = get_series_index(&book_page);

    let cover = get_cover_url(&book_page);

    let synopsis = get_synopsis_html(&book_page);

    let tags = get_tags_str(&book_page);

    let rating = get_rating(&book_page);

    let publisher = get_publisher(&book_page);

    let release_date = get_release_date(&book_page);

    let language = get_language(&book_page);

    Ok(Book {
        id: id.to_string(),
        title,
        authors,
        series_name,
        series_index,
        cover,
        synopsis,
        tags,
        rating,
        publisher,
        release_date,
        language,
    })
}

fn get_book_page(id: &str) -> Result<Html> {
    let book_page_url = format!("{}{}", BOOK_PATH, id);
    let book_page_response = reqwest::blocking::get(book_page_url)?;
    let book_page_html = book_page_response.text()?;
    let book_page = Html::parse_document(&book_page_html);

    Ok(book_page)
}

fn get_title(html: &Html) -> String {
    let title_selector = Selector::parse("div.item-info > h1").expect("Invalid selector");
    let title = html
        .select(&title_selector)
        .next()
        .map(|h1| h1.text().collect::<String>().trim().to_string())
        .unwrap_or_default();

    title
}

fn get_authors_str(html: &Html) -> String {
    let authors_selector = Selector::parse("a.contributor-name").expect("Invalid selector");
    let authors_str = html
        .select(&authors_selector)
        .map(|a| a.text().collect::<String>())
        .collect::<Vec<String>>()
        .join(", ");

    authors_str
}

fn get_series_name(html: &Html) -> Option<String> {
    let series_name_selector =
        Selector::parse("a[data-track-info='{}']").expect("Invalid selector");
    let series_name = html
        .select(&series_name_selector)
        .next()?
        .text()
        .collect::<String>();

    Some(series_name)
}

fn get_series_index(html: &Html) -> Option<f64> {
    todo!()
}

fn get_cover_url(html: &Html) -> String {
    todo!()
}

fn get_synopsis_html(html: &Html) -> String {
    todo!()
}

fn get_tags_str(html: &Html) -> String {
    todo!()
}

fn get_rating(html: &Html) -> Rating {
    todo!()
}

fn get_publisher(html: &Html) -> String {
    todo!()
}

fn get_release_date(html: &Html) -> String {
    todo!()
}

fn get_language(html: &Html) -> String {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_non_kobo_url() {
        let book_name = get_book_id("done");

        assert_eq!(book_name, None)
    }

    #[test]
    fn input_no_id_url() {
        let book_name = get_book_id("https://www.kobo.com/tw/zh/ebook/");

        assert_eq!(book_name, None)
    }

    #[test]
    fn input_kobo_book_url() {
        let book_name = get_book_id("https://www.kobo.com/tw/zh/ebook/tSfRgYbwtzGWxEne-NJKWw");

        assert_eq!(book_name, Some("tSfRgYbwtzGWxEne-NJKWw".to_string()))
    }

    #[test]
    fn test_book_title() -> Result<()> {
        let book_page = get_book_page("tSfRgYbwtzGWxEne-NJKWw")?;
        let book_title = get_title(&book_page);

        Ok(assert_eq!(book_title, "迷霧之子首部曲：最後帝國"))
    }

    #[test]
    fn test_book_authors() -> Result<()> {
        let book_page = get_book_page("let-it-snow-5")?;
        let book_authors = get_authors_str(&book_page);

        Ok(assert_eq!(
            book_authors,
            "John Green, Lauren Myracle, Maureen Johnson"
        ))
    }

    #[test]
    fn test_book_series_name() -> Result<()> {
        let book_page = get_book_page("defiant-68")?;
        let book_series_name = get_series_name(&book_page);

        Ok(assert_eq!(
            book_series_name,
            Some("The Skyward Series".to_string())
        ))
    }
}
