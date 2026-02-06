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
        .section{{margin-top:24px}}
        </style></head><body>
        <h1>Plus — Яндекс</h1>
        <form class='search' action='https://yandex.ru/search/'><input name='text' placeholder='Поиск в Яндексе'><button>Найти</button></form>
        <div class='grid'>{}</div>
        <div class='section'>
          <h3>Часто посещаемые</h3>
          <div class='grid'><div class='tile'>Добавьте сайт</div><div class='tile'>Добавьте сайт</div><div class='tile'>Добавьте сайт</div><div class='tile'>Добавьте сайт</div></div>
          <button style='margin-top:12px'>Редактировать плитки</button>
        </div>
        </body></html>"#,
        tiles
    )
}

pub fn diagnostics_html() -> String {
    r#"<!doctype html><html><head><meta charset='utf-8'><style>
    body{font-family:Inter,Arial;background:#0f172a;color:#e2e8f0;margin:0;padding:24px}
    h1{margin:0 0 16px 0}
    .card{background:#111827;border-radius:12px;padding:16px;margin-bottom:12px}
    button{padding:10px 14px;border:none;border-radius:10px;background:#38bdf8;color:#0f172a;font-weight:600}
    pre{white-space:pre-wrap}
    </style></head><body>
    <h1>Plus Diagnostics</h1>
    <div class='card'>
      <strong>Runtime</strong>
      <pre id='runtime'>Loading...</pre>
    </div>
    <div class='card'>
      <strong>Adblock</strong>
      <pre id='adblock'>Loading...</pre>
    </div>
    <div class='card'>
      <strong>Check IP</strong><br><br>
      <button onclick='checkIp()'>Check IP</button>
      <pre id='ip'></pre>
    </div>
    <script>
      async function refresh(){
        const runtime = await fetch('plus://diagnostics').then(r=>r.json());
        document.getElementById('runtime').textContent = JSON.stringify(runtime, null, 2);
        const adblock = await fetch('plus://adblock').then(r=>r.json());
        document.getElementById('adblock').textContent = JSON.stringify(adblock, null, 2);
      }
      async function checkIp(){
        const res = await fetch('https://api.ipify.org?format=json').then(r=>r.json());
        document.getElementById('ip').textContent = JSON.stringify(res, null, 2);
      }
      refresh();
    </script>
    </body></html>"#
        .to_string()
}

pub fn settings_html() -> String {
    r#"<!doctype html><html><head><meta charset='utf-8'><style>
    body{font-family:Inter,Arial;background:#0f172a;color:#e2e8f0;margin:0;padding:24px}
    .layout{display:grid;grid-template-columns:220px 1fr;gap:20px}
    .menu{background:#111827;border-radius:12px;padding:12px}
    .menu div{padding:10px;border-radius:10px;color:#cbd5f1}
    .menu div.active{background:#1f2937;color:#fff}
    .content{background:#111827;border-radius:12px;padding:16px}
    input{padding:10px;border-radius:10px;border:1px solid #1f2937;background:#0b1220;color:#fff;width:100%}
    </style></head><body>
    <h1>Настройки</h1>
    <div class="layout">
      <div class="menu">
        <div class="active">Общие</div>
        <div>Внешний вид</div>
        <div>Поиск</div>
        <div>Конфиденциальность</div>
        <div>VPN</div>
        <div>AdBlock</div>
        <div>Загрузки</div>
        <div>О программе</div>
      </div>
      <div class="content">
        <h3>Поиск по настройкам</h3>
        <input placeholder="Поиск настроек">
      </div>
    </div>
    </body></html>"#
        .to_string()
}

pub fn app_shell_html() -> String {
    r#"<!doctype html><html><head><meta charset='utf-8'><style>
    :root{--bg:#0b0f17;--panel:#111827;--text:#e5e7eb;--muted:#94a3b8;--accent:#f5c400;--border:#1f2937}
    *{box-sizing:border-box} body{margin:0;font-family:Inter,Arial;background:var(--bg);color:var(--text)}
    .tabs{height:42px;display:flex;align-items:center;gap:6px;padding:0 10px;background:#0f172a;border-bottom:1px solid var(--border)}
    .tab{padding:8px 12px;border-radius:10px;background:#0b1220;color:var(--muted);display:flex;gap:8px;align-items:center}
    .tab.active{background:#182235;color:var(--text);outline:1px solid #243041}
    .tab .close{cursor:pointer;opacity:.6}
    .spacer{flex:1}
    .icon-btn{width:28px;height:28px;border-radius:8px;display:grid;place-items:center;background:#111827;color:var(--text);cursor:pointer}
    .nav{height:48px;display:flex;align-items:center;gap:8px;padding:0 12px;background:#0f172a;border-bottom:1px solid var(--border)}
    .omnibox{flex:1;display:flex;align-items:center;background:#111827;border-radius:12px;padding:6px 10px;border:1px solid #1f2937}
    .omnibox input{flex:1;background:transparent;border:none;color:var(--text);outline:none}
    .content{height:calc(100vh - 90px);position:relative}
    .view{position:absolute;inset:0;border:0;width:100%;height:100%}
    .progress{position:absolute;top:0;left:0;height:2px;background:var(--accent);width:0;transition:width .2s}
    .grid{display:grid;grid-template-columns:repeat(4,minmax(0,1fr));gap:12px}
    .tile{background:#111827;border-radius:12px;padding:16px;text-align:center;transition:transform .15s, box-shadow .15s}
    .tile:hover{transform:translateY(-2px);box-shadow:0 8px 16px rgba(0,0,0,.2)}
    .center{max-width:980px;margin:0 auto;padding:24px}
    .search{display:flex;gap:8px;margin:16px 0}
    .search input{flex:1;padding:12px;border-radius:12px;border:1px solid #1f2937;background:#0b1220;color:var(--text)}
    .search button{padding:12px 16px;border-radius:12px;border:none;background:var(--accent);color:#111}
    .toast{position:fixed;right:16px;bottom:16px;background:#111827;padding:10px 14px;border-radius:10px;border:1px solid #1f2937;opacity:0;transition:opacity .2s}
    .toast.show{opacity:1}
    </style></head><body>
    <div class="tabs" id="tabs"></div>
    <div class="nav">
      <div class="icon-btn" onclick="goBack()">◀</div>
      <div class="icon-btn" onclick="goForward()">▶</div>
      <div class="icon-btn" onclick="reloadTab()">⟳</div>
      <div class="icon-btn" onclick="goHome()">⌂</div>
      <div class="omnibox"><input id="omnibox" placeholder="Введите адрес или запрос в Яндексе"></div>
      <div class="icon-btn" onclick="toggleBookmark()">☆</div>
      <div class="icon-btn" onclick="openDiagnostics()">🛡</div>
    </div>
    <div class="content">
      <div class="progress" id="progress"></div>
      <iframe class="view" id="view"></iframe>
    </div>
    <div class="toast" id="toast"></div>
    <script>
      const tabs = [];
      let active = 0;
      const view = document.getElementById('view');
      const omnibox = document.getElementById('omnibox');
      function renderTabs(){
        const bar = document.getElementById('tabs');
        bar.innerHTML='';
        tabs.forEach((t,i)=>{
          const el = document.createElement('div');
          el.className='tab'+(i===active?' active':'');
          el.innerHTML=`<span>${t.title||'Новая вкладка'}</span><span class="close" onclick="closeTab(${i})">✕</span>`;
          el.onclick=()=>activateTab(i);
          bar.appendChild(el);
        });
        const add = document.createElement('div');
        add.className='icon-btn';
        add.textContent='+';
        add.onclick=()=>newTab();
        bar.appendChild(add);
        const spacer = document.createElement('div'); spacer.className='spacer'; bar.appendChild(spacer);
        const downloads = document.createElement('div'); downloads.className='icon-btn'; downloads.textContent='⬇'; bar.appendChild(downloads);
        const vpn = document.createElement('div'); vpn.className='icon-btn'; vpn.textContent='VPN'; bar.appendChild(vpn);
        const ad = document.createElement('div'); ad.className='icon-btn'; ad.textContent='AD'; bar.appendChild(ad);
        const settings = document.createElement('div'); settings.className='icon-btn'; settings.textContent='⚙'; settings.onclick=()=>openSettings(); bar.appendChild(settings);
      }
      function newTab(url){
        tabs.push({url:url||'plus://newtab', title:'Новая вкладка'});
        active = tabs.length-1;
        navigate(tabs[active].url);
        renderTabs();
      }
      function closeTab(idx){
        tabs.splice(idx,1);
        if(active>=tabs.length) active=tabs.length-1;
        if(active<0){ newTab(); return; }
        navigate(tabs[active].url);
        renderTabs();
      }
      function activateTab(idx){
        active=idx;
        navigate(tabs[active].url);
        renderTabs();
      }
      function navigate(input){
        const url = window.plusNavigate ? window.plusNavigate(input) : input;
        tabs[active].url = url;
        view.src = url;
        omnibox.value = url;
      }
      function goBack(){ view.contentWindow.history.back(); }
      function goForward(){ view.contentWindow.history.forward(); }
      function reloadTab(){ view.contentWindow.location.reload(); }
      function goHome(){ navigate('plus://newtab'); }
      function openDiagnostics(){ navigate('plus://diagnostics-ui'); }
      function openSettings(){ navigate('plus://settings'); }
      function toggleBookmark(){ showToast('Закладка добавлена'); }
      function showToast(text){ const t=document.getElementById('toast'); t.textContent=text; t.classList.add('show'); setTimeout(()=>t.classList.remove('show'),1200); }
      omnibox.addEventListener('keydown', (e)=>{ if(e.key==='Enter'){ navigate(omnibox.value); }});
      window.addEventListener('keydown', (e)=>{
        if((e.ctrlKey||e.metaKey)&&e.key==='t'){ newTab(); }
        if((e.ctrlKey||e.metaKey)&&e.key==='w'){ closeTab(active); }
        if((e.ctrlKey||e.metaKey)&&e.key==='l'){ omnibox.focus(); omnibox.select(); }
      });
      newTab();
    </script>
    </body></html>"#
        .to_string()
}

pub fn new_tab_data_url() -> String {
    format!("data:text/html,{}", urlencoding::encode(&new_tab_html()))
}
