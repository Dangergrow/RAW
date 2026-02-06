use serde::{Deserialize, Serialize};
use url::Url;

pub const YANDEX_SEARCH: &str = "https://yandex.ru/search/?text=";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YandexService {
    pub name: &'static str,
    pub url: &'static str,
}

pub fn yandex_tiles() -> Vec<YandexService> {
    vec![
        YandexService {
            name: "Поиск",
            url: "https://yandex.ru",
        },
        YandexService {
            name: "Почта",
            url: "https://mail.yandex.ru",
        },
        YandexService {
            name: "Диск",
            url: "https://disk.yandex.ru",
        },
        YandexService {
            name: "Музыка",
            url: "https://music.yandex.ru",
        },
        YandexService {
            name: "Карты",
            url: "https://yandex.ru/maps",
        },
        YandexService {
            name: "Маркет",
            url: "https://market.yandex.ru",
        },
        YandexService {
            name: "Новости",
            url: "https://dzen.ru/news",
        },
        YandexService {
            name: "Погода",
            url: "https://yandex.ru/pogoda",
        },
    ]
}

pub fn omnibox_to_url(input: &str) -> String {
    if let Ok(parsed) = Url::parse(input) {
        return parsed.to_string();
    }
    if input.contains('.') && !input.contains(' ') {
        return format!("https://{}", input);
    }
    format!("{}{}", YANDEX_SEARCH, urlencoding::encode(input))
}

pub fn new_tab_html() -> String {
    let mut tiles = String::new();
    for tile in yandex_tiles() {
        tiles.push_str(&format!(
            "<a class='tile' href='{}'>{}</a>",
            tile.url, tile.name
        ));
    }
    format!(
        r#"<!doctype html><html><head><meta charset='utf-8'><style>
        body{{font-family:Inter,Arial;background:#0e1117;color:#f5f7fa;margin:0;padding:32px}}
        h1{{margin:0 0 16px 0}}
        .search{{display:flex;gap:8px;margin-bottom:18px}}
        input{{flex:1;padding:12px;border-radius:10px;border:1px solid #2f3542;background:#111827;color:#fff}}
        button{{padding:12px 16px;border-radius:10px;border:none;background:#ffcc00;color:#111;font-weight:600}}
        .grid{{display:grid;grid-template-columns:repeat(4,minmax(0,1fr));gap:10px}}
        .tile{{display:block;padding:14px;border-radius:10px;background:#1f2937;color:#fff;text-decoration:none;text-align:center}}
        </style></head><body>
        <h1>Plus — Яндекс</h1>
        <form class='search' action='https://yandex.ru/search/'><input name='text' placeholder='Поиск в Яндексе'><button>Найти</button></form>
        <div class='grid'>{}</div>
        </body></html>"#,
        tiles
    )
}
