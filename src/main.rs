mod book;

use anyhow::Result;
use std::io::stdin;

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

        let Some(book_id) = book::get_id(&input) else {
            println!("Not a Kobo book URL!");
            continue;
        };
        book_ids.push(book_id);
    }

    for book_id in book_ids {
        let book_metadata = book::get_metadata(&book_id)?;
    }

    Ok(())
}
