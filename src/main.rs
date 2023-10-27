mod book;

use anyhow::Result;
use book::Id;
use indicatif::ProgressBar;
use std::io::stdin;

fn main() -> Result<()> {
    println!("Enter Kobo book URLs:");
    let book_ids = get_book_ids()?;

    let pb = ProgressBar::new(book_ids.len() as u64);
    for book_id in book_ids {
        let book_pb = ProgressBar::new(15);
        book_id
            .get_metadata(&book_pb)?
            .append_to_csv_file(&book_pb)?;
        book_pb.finish_and_clear();
        pb.inc(1);
    }

    Ok(println!("Done!"))
}

fn get_book_ids() -> Result<Vec<String>> {
    let mut book_ids = Vec::<String>::new();

    loop {
        let mut input = String::new();
        stdin().read_line(&mut input)?;

        let line = input.trim();
        if line == "done" {
            break;
        }

        let Some(book_id) = Id::from(line) else {
            println!("Not a Kobo book URL!");
            continue;
        };
        book_ids.push(book_id);
    }

    Ok(book_ids)
}
