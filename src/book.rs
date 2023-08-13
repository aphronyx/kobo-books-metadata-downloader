use anyhow::Result;
use indicatif::ProgressBar;
use scraper::{Html, Selector};
use std::{
    fmt::Display,
    fs::{create_dir, OpenOptions},
    io::Write,
    path::Path,
};

#[derive(Debug, PartialEq)]
enum Rating {
    NotRated,
    One,
    Two,
    Three,
    Four,
    Five,
}

#[derive(Debug, PartialEq)]
pub struct Metadata {
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
const CSV_FILE_PATH: &str = "./metadata.csv";

pub fn get_id(input: &str) -> Option<String> {
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

pub fn get_metadata(id: &str, pb: &ProgressBar) -> Result<Metadata> {
    let book_page = get_book_page(id)?;
    pb.inc(1);

    let title = get_title(&book_page);
    pb.inc(1);

    let authors = get_authors_str(&book_page);
    pb.inc(1);

    let series_name = get_series_name(&book_page);
    pb.inc(1);

    let series_index = get_series_index(&book_page);
    pb.inc(1);

    let cover = get_cover_url(&book_page);
    pb.inc(1);

    let synopsis = get_synopsis_html(&book_page);
    pb.inc(1);

    let tags = get_tags_str(&book_page);
    pb.inc(1);

    let rating = get_rating(&book_page);
    pb.inc(1);

    let publisher = get_publisher(&book_page);
    pb.inc(1);

    let release_date = get_release_date(&book_page);
    pb.inc(1);

    let language = get_language(&book_page);
    pb.inc(1);

    Ok(Metadata {
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

impl Metadata {
    pub fn append_to_csv_file(self, pb: &ProgressBar) -> Result<()> {
        let img_dir = "./img";
        if !Path::new(img_dir).exists() {
            create_dir(img_dir)?;
        }

        let mut img_name = 1;
        let mut img_path;
        loop {
            img_path = format!("{}/{}.jpg", img_dir, img_name);
            if !Path::new(&img_path).exists() {
                break;
            }
            img_name += 1;
        }

        let mut img_file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&img_path)?;
        let img_response = reqwest::blocking::get(self.cover)?;

        let img = img_response.bytes()?;
        img_file.write_all(&img)?;
        pb.inc(1);

        let mut csv_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(CSV_FILE_PATH)?;
        let line = format!(
            "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"\n",
            self.id,
            self.title,
            self.authors,
            self.series_name.unwrap_or_default(),
            self.series_index
                .map(|index| index.to_string())
                .unwrap_or_default(),
            img_path,
            self.synopsis,
            self.tags,
            self.rating,
            self.publisher,
            self.release_date,
            self.language
        );
        csv_file.write_all(line.as_bytes())?;
        pb.inc(1);

        Ok(())
    }
}

impl Display for Rating {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let rating = match self {
            Rating::NotRated => 0,
            Rating::One => 1,
            Rating::Two => 2,
            Rating::Three => 3,
            Rating::Four => 4,
            Rating::Five => 5,
        };
        write!(f, "{}", rating)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn input_non_kobo_url() {
        let book_name = get_id("done");

        assert_eq!(book_name, None)
    }

    #[test]
    fn input_no_id_url() {
        let book_name = get_id("https://www.kobo.com/tw/zh/ebook/");

        assert_eq!(book_name, None)
    }

    #[test]
    fn input_kobo_book_url() {
        let book_name = get_id("https://www.kobo.com/tw/zh/ebook/tSfRgYbwtzGWxEne-NJKWw");

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

    #[test]
    fn test_book_metadata() -> Result<()> {
        let book_metadata = get_metadata("J2FjG5BoyDiEQfQn-uI4OA", &ProgressBar::hidden())?;

        let test_book_metadata = Metadata {
            id: "J2FjG5BoyDiEQfQn-uI4OA".to_string(),
            title: "不便利的便利店".to_string(),
            authors: "金浩然 （김호연）".to_string(),
            series_name: Some("Soul".to_string()),
            series_index: None,
            cover: "https://cdn.kobo.com/book-images/04b3ec92-aaa7-4757-b1ac-ff143aed0848/1650/2200/100/False/J2FjG5BoyDiEQfQn-uI4OA.jpg".to_string(),
            synopsis: "<p><strong>人生就是會有很多不便利、不舒服，</strong><br>\n<strong>這間有點慘澹的便利店，卻為我們撐起了閃閃發光的空間……</strong></p>\n<p><strong>艱難時刻的光亮之書</strong><br>\n<strong>一間便利店，接通了我們的幸福人生</strong></p>\n<p><strong>★韓國年度最受歡迎小說</strong><br>\n<strong>★銷售破70萬冊，25個都市特選年度之書</strong><br>\n<strong>★Yes24年度之書，韓國各大書店排行榜總冠軍，口碑直追《歡迎光臨夢境百貨》</strong><br>\n<strong>★電子書平台「米莉的書齋」年度圖書第二名</strong><br>\n<strong>★韓國中央圖書館館員推薦之書</strong><br>\n<strong>★售出泰、日、簡中、台灣、越南、印尼等多國版權</strong><br>\n<strong>★影視改編熱烈進行中</strong></p>\n<p>◎全球獨家收錄：作者手寫給台灣讀者的問候箋</p>\n<p>謝哲青＼作家、旅行家<br>\n盧建彰＼導演<br>\n李盈姿＼芒草心慈善協會祕書長<br>\n別家門市＼「超商系」插畫粉絲團<br>\n太咪＼作家、《太咪瘋韓國》版主<br>\n山女孩kit＼作家<br>\n方億玲＼而立書店店長<br>\n徐慧玲＼聆韵企管顧問創辦人──鼓掌推薦</p>\n<p>◎韓國讀者口碑推薦：</p>\n<p>‧這是一本我想推薦給所有人的人生之書。你讀的時候，很可能一會兒哭一會兒笑，但不知不覺間心頭就暖呼呼了。<br>\n‧擦肩而過的人，竟然可以成為彼此生活前進的支撐。一本讓我看到人生力量的書。<br>\n‧我的眼角掛著淚，嘴邊帶著笑。多虧這本書，讓我熬過疫病籠罩的日子。<br>\n‧哭著，笑著，心也跟著暖了。<br>\n‧場景不陌生、人物不陌生，就連裡面的衝突也不陌生，但是人們彼此表達善意卻是這個冷陌時代最需要的態度。</p>\n<p><strong>這間有點不便利，卻讓人想一再前往的便利店，</strong><br>\n<strong>藏著能在艱難生活中給你安慰的各樣物品。</strong></p>\n<p><strong>買一送一的喜悅、三角飯糰模樣的悲傷，</strong><br>\n<strong>以及一萬元所帶來的四次歡笑，</strong><br>\n<strong>充滿特別的故事與奇妙商品組合的便利店，時時歡迎您！</strong></p>\n<p>廉女士搭火車途中，驚覺錢包不見了，此時一通電話來告知，說在車站撿到了包包，還嚅囁詢問能否借用點錢買便當吃。廉女士答應了。</p>\n<p>果然如她所想，對方是一名流浪漢。廉女士在拿回包包時，告知對方，歡迎他來自己經營的便利店吃便當。</p>\n<p>這間便利店生意不太好，店員更是各種邊緣人的組合：上了年紀還為子女操碎了心的婦人；準備公務員考試多年的年輕女孩；五十多歲靠微薄薪水養家的一家之主。而廉女士為了如同家人般的員工，努力把店鋪撐了下來。</p>\n<p>然而，大夜班店員突然辭職，讓她苦惱不已。就在這時，常來吃報廢便當的流浪漢竟陰錯陽差接下這份工作……</p>\n<p>\u{f0d8}</p>\n<p><strong>只差一點點就陷落於孤立和衝突的人生，</strong><br>\n<strong>如何在這個小小的空間裡悄悄獲得喘息？</strong><br>\n<strong>一間不夠便利的便利店，又如何接通大家的幸福人生？</strong></p>\n<p><strong>◎便利店「幫人生加值」小語</strong></p>\n<p>※我問，支持妳的力量究竟是什麼？<br>\n她說，人生本來就是不斷解決問題，既然都要解決問題，那就努力選還可以的問題來解。</p>\n<p>※便利店是個人們來來去去的空間，無論店員還是客人，都只是短暫停留的過客。便利店就像間加油站，讓人們用物品或金錢為自己加值。</p>\n<p>※為什麼開心？因為炸雞？因為爸爸的陪伴？其實無論是什麼都沒關係，因為能一起吃雞的就是家人。</p>\n<p>※人生就是關係，關係的根本就是溝通。我發現只要我們能跟身旁的人交心，幸福其實離我們不遠。</p>\n<p>※巴布狄倫的外婆曾經告訴他，幸福不是在通往目標路途上的某樣東西，而是那條路本身就是幸福。你所遇見的每個人，都在苦苦掙扎著與什麼對抗，所以你必須親切待人。</p>\n<p>【作者簡介】<strong>金浩然（김호연）</strong></p>\n<p><strong>全天候說故事的人</strong><br>\n<strong>人生目標：透過電影、漫畫、小說講述各樣故事</strong></p>\n<p>1974年出生於首爾。畢業於高麗大學人文學院國語國文學科。初入職場時，在電影公司參與創作的劇本《諜變任務》被改編為電影，自此成為編劇。<br>\n第二份工作是擔任漫畫策劃人員，撰寫的《人體實驗區》獲得第一屆富川漫畫故事競賽大獎，自此成了漫畫腳本家。在出版社擔任小說編輯一陣子之後，決定轉換跑道，成為為全職作家。<br>\n他努力實踐「年輕時就該任意揮灑文字」的理念，以長篇小說《望遠洞兄弟》奪下2013年第9屆世界文學獎的優秀獎，展開小說家生涯之路。此後還推出長篇小說《情敵》《幽靈作家》《浮士德》及散文集《每天寫，重新寫，寫到最後》，並參與電影《烈日追殺》的劇本及《南漢山城》的策劃。<br>\n2021年繼《望遠洞兄弟》以後，再度推出描繪鄰里人情的溫暖故事《不便利的便利店》，成為口碑長紅的年度暢銷冠軍，並售出多國版權，影視改編也熱烈進行中。</p>\n<p>＊獲獎紀錄：</p>\n<p>《人體實驗區》獲第一屆富川漫畫故事競賽<br>\n《望遠洞兄弟》獲2013年第9屆世界文學獎優秀獎<br>\n《不便利的便利店》獲韓國超過25個都市選爲年度之書</p>\n<p>譯者 <strong>陳品芳</strong><br>\n政大韓文系畢，曾於台韓兩地職場打滾，目前為韓中專職譯者。熱愛各種二、三次元娛樂，享受在趕稿與耍廢之間穿梭的自由時光。譯有《剝削首爾》《讓尼采當你的心理師》《K-Pop征服世界的秘密》等書。</p>\n".to_string(),
            tags: "小說與文學".to_string(),
            rating: Rating::NotRated,
            publisher: "寂寞".to_string(),
            release_date: "2022-9-1".to_string(),
            language: "中文".to_string(),
        };

        Ok(assert_eq!(book_metadata, test_book_metadata))
    }

    #[test]
    fn test_append_to_csv_file() -> Result<()> {
        let book_metadata = Metadata {
            id: "id".to_string(),
            title: "title".to_string(),
            authors: "auth, ors".to_string(),
            series_name: Some("series name".to_string()),
            series_index: Some(0.0),
            cover: "https://cdn.kobo.com/book-images/04b3ec92-aaa7-4757-b1ac-ff143aed0848/1650/2200/100/False/J2FjG5BoyDiEQfQn-uI4OA.jpg".to_string(),
            synopsis: "<p>synopsis</p>".to_string(),
            tags: "t, a, g, s".to_string(),
            rating: Rating::NotRated,
            publisher: "publisher".to_string(),
            release_date: "0000-0-0".to_string(),
            language: "language".to_string(),
        };
        book_metadata.append_to_csv_file(&ProgressBar::hidden())?;

        let csv_file = fs::read_to_string(CSV_FILE_PATH)?.trim().to_string();
        let test_csv_file = r#""id","title","auth, ors","series name","0","./img/1.jpg","<p>synopsis</p>","t, a, g, s","0","publisher","0000-0-0","language""#;
        assert_eq!(csv_file, test_csv_file);

        let img_path = csv_file.split(r#"",""#).nth(5).unwrap_or_default();
        assert!(Path::new(img_path).exists());

        fs::remove_file(CSV_FILE_PATH)?;
        fs::remove_file(img_path)?;

        Ok(())
    }
}
