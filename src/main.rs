use std::io::{stdin, Result};

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

    Ok(())
}

fn get_book_id(input: &str) -> Option<String> {
    let is_not_kobo_book_url = !input.contains("https://www.kobo.com/tw/zh/ebook/");
    if is_not_kobo_book_url {
        return None;
    }

    let book_id = input
        .rsplit_once('/')
        .map(|(_, substring)| substring.trim().to_string())
        .filter(|id| !id.is_empty())?;

    Some(book_id)
}
