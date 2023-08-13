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
        .map(|a| a.text().collect())
        .collect::<Vec<String>>()
        .join(", ");

    authors_str
}

fn get_series_name(html: &Html) -> Option<String> {
    let series_name_selector =
        Selector::parse("a[data-track-info='{}']").expect("Invalid selector");
    let series_name = html.select(&series_name_selector).next()?.text().collect();

    Some(series_name)
}

fn get_series_index(html: &Html) -> Option<f64> {
    let series_index_selector =
        Selector::parse("span.sequenced-name-prefix").expect("Invalid selector");
    let series_index = html
        .select(&series_index_selector)
        .next()?
        .text()
        .collect::<String>()
        .replace(|char: char| !(char.is_ascii_digit() || char == '.'), "")
        .parse()
        .ok();

    series_index
}

fn get_cover_url(html: &Html) -> String {
    let cover_selector = Selector::parse("link[as='image']").expect("Invalid selector");
    let cover_url = html
        .select(&cover_selector)
        .next()
        .and_then(|link| link.value().attr("href"))
        .map(|url| url.replace("/353/569/90/", "/1650/2200/100/"))
        .unwrap_or_default();

    cover_url
}

fn get_synopsis_html(html: &Html) -> String {
    let synopsis_selector = Selector::parse("div.synopsis-description").expect("Invalid selector");
    let synopsis_html = html
        .select(&synopsis_selector)
        .next()
        .map(|div| div.inner_html())
        .unwrap_or_default();

    synopsis_html
}

fn get_tags_str(html: &Html) -> String {
    let tag_selector =
        Selector::parse("a.rankingAnchor.description-anchor").expect("Invalid selector");
    let mut tags_vec = html
        .select(&tag_selector)
        .map(|a| a.text().collect())
        .collect::<Vec<String>>();
    tags_vec.sort();
    tags_vec.dedup();
    tags_vec.join(", ")
}

// TODO
fn get_rating(_: &Html) -> Rating {
    Rating::NotRated
}

fn get_publisher(html: &Html) -> String {
    let imprint_selector =
        Selector::parse("a.description-anchor > span").expect("Invalid selector");
    let publishing_company_selector =
        Selector::parse("div.bookitem-secondary-metadata li").expect("Invalid selector");
    let publisher = html.select(&imprint_selector).next().map_or(
        html.select(&publishing_company_selector)
            .next()
            .map(|li| li.text().collect::<String>().trim().to_string())
            .unwrap_or_default(),
        |span| span.text().collect(),
    );

    publisher
}

fn get_release_date(html: &Html) -> String {
    let release_date_selector =
        Selector::parse("div.bookitem-secondary-metadata li > span").expect("Invalid selector");
    let release_date = html
        .select(&release_date_selector)
        .next()
        .map(|span| {
            let mut date = span
                .text()
                .collect::<String>()
                .replace(|char: char| !char.is_ascii_digit(), "-");
            date.pop();

            date
        })
        .unwrap_or_default();

    release_date
}

fn get_language(html: &Html) -> String {
    let language_selector =
        Selector::parse("div.bookitem-secondary-metadata li > span").expect("Invalid selector");
    let language = html
        .select(&language_selector)
        .nth(2)
        .map(|span| span.text().collect())
        .unwrap_or_default();

    language
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

    #[test]
    fn test_book_series_index() -> Result<()> {
        let book_page = get_book_page("YOylwW_Z6jKJP7HpcEr0Ig")?;
        let book_series_index = get_series_index(&book_page);

        Ok(assert_eq!(book_series_index, Some(13.5)))
    }

    #[test]
    fn test_book_cover() -> Result<()> {
        let book_page = get_book_page("tSfRgYbwtzGWxEne-NJKWw")?;
        let book_cover = get_cover_url(&book_page);

        Ok(assert_eq!(book_cover, "https://cdn.kobo.com/book-images/28289ceb-265c-488a-bf08-ae3424588a91/1650/2200/100/False/tSfRgYbwtzGWxEne-NJKWw.jpg"))
    }

    #[test]
    fn test_book_synopsis() -> Result<()> {
        let book_page = get_book_page("tSfRgYbwtzGWxEne-NJKWw")?;
        let book_synopsis =
            get_synopsis_html(&book_page).replace(|char: char| char.is_ascii_control(), "");

        let test_book_synopsis = "<p>美國亞馬遜讀者評鑑最高票，全球暢銷三千萬冊著作已逝奇幻大師，羅伯特．喬丹指定接班人</p><p>09年「時光之輪」接班作《風起雲湧》，打敗丹布朗新書《失落的符號》，空降紐約時報排行榜冠軍作者</p><p>《出版人週刊》、《軌跡雜誌》、《美國圖書館協會誌》、《克科斯評論》極優評價</p><p>美國最大邦諾連鎖書店頭號選書作者、西班牙UPC科幻大獎得主</p><p>2005年出道即獲《浪漫時代 Romantic Times》奇幻史詩大獎</p><p>2006、2007年入選美國科奇幻地位最高約翰．坎伯新人獎</p><p>超級天才新星作家──布蘭登．山德森全新華麗鉅作</p><p>架構壯閣媲美「冰與火之歌」，精采絕妙更勝「夜巡者」</p><p>「這本書有完美縝密的架構……我極度推薦給任何渴求一本好書的讀者。」──羅蘋．荷布（「刺客」系列作者）</p><p>「 我很驕傲、很榮幸、很迫切想要介紹這位作者和他的作品給所有讀者。」──灰鷹／譚光磊（版權經紀人）</p><p>「 一個繁複的革命計畫，透過作者縝密的佈局，逐步實行。小說結構完整，前後緊密聯繫；布蘭登．山德森能否抽空來寫部推理小說？」──紗卡（推理文學研究會MLR）</p><p>一個不可能成功的絕望計畫，而勝利，將是最糟的代價……</p><p>迷霧之子</p><p>首部曲：最後帝國</p><p>Mistborn: The Final Empire</p><p>「他說：任何人都會背叛你，任何人。」</p><p>如果背叛無所不在，如果一切非你以為那樣，</p><p>你有勇氣知道真相嗎？</p><p>這是個英雄殞落，邪惡籠罩的世界，再不見光明與顏色。</p><p>入夜後，迷霧四起，誰也不曉得，藏身在白茫霧色之後的，會是什麼……</p><p>千年前，善惡雙方決戰，良善一方的英雄歷經千辛萬苦，終於抵達傳說中的聖地「昇華之井」，準備和黑暗勢力一決生死。</p><p>可是，命運女神沒有站在良善這方。</p><p>最後，邪惡擊潰英雄，一統天下，並自稱「統御主」，同時建立「最後帝國」，號稱千秋萬代、永不崩塌。至此，世界隨之變遷，從此綠色不再，所有植物都轉為褐黃，天空永遠陰霾，不間斷地下著灰燼，彷彿是浩劫過後的殘破荒地。入夜之後，濃霧四起，籠罩大地。</p><p>統御主如神一般無敵，以絕對的權力和極端的高壓恐怖統治著最後帝國。他更以凶殘的手段鎮壓平民百姓，不分國籍種族通通打為奴隸階級，通稱「司卡」。司卡人活在無止盡的悲慘和恐懼之中，千年來的奴役讓他們早已沒有希望，沒有任何過去的記憶。</p><p>如今，一線生機浮現。二名貴族與司卡混血卻天賦異稟、身負使命的街頭小人物，即將編織一場前所未有的騙局，進行一項絕不可能成功的計畫，只為了獲得最糟糕的代價──勝利……</p><p>迷霧之子三部曲　Mistborn Trilogy──</p><p>首部曲：最後帝國The Final Empire</p><p>二部曲：昇華之井The Well of Ascension 2010年4月出版</p><p>終部曲：永世英雄The Hero of Ages 2010年6月出版</p>";

        Ok(assert_eq!(book_synopsis, test_book_synopsis))
    }

    #[test]
    fn test_book_tags() -> Result<()> {
        let book_page = get_book_page("i-357")?;
        let book_tags = get_tags_str(&book_page);

        let mut test_tags_vec = "青少年 - YA, 漫畫、圖畫小說和漫畫, 兒童, 漫畫、圖像小說與連環漫畫, 科幻小說與奇幻小說, 幻想".split(", ").collect::<Vec<&str>>();
        test_tags_vec.sort();
        test_tags_vec.dedup();
        let test_tags = test_tags_vec.join(", ");

        Ok(assert_eq!(book_tags, test_tags))
    }

    #[test]
    fn test_book_publisher() -> Result<()> {
        let book_page = get_book_page("silent-witch-1")?;
        let book_publisher = get_publisher(&book_page);

        Ok(assert_eq!(book_publisher, "台灣角川"))
    }

    #[test]
    fn test_book_release_date() -> Result<()> {
        let book_page = get_book_page("silent-witch-1")?;
        let book_release_date = get_release_date(&book_page);

        Ok(assert_eq!(book_release_date, "2022-5-27"))
    }

    #[test]
    fn test_book_language() -> Result<()> {
        let book_page = get_book_page("mistborn-trilogy")?;
        let book_language = get_language(&book_page);

        Ok(assert_eq!(book_language, "英文"))
    }
}
